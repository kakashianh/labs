use axum::{body::Body, http::{Request, StatusCode}};
use backend::{db, router::router, state::AppState};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage, ImageExt,
};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn spatial_results_and_aggregate_are_consistent_and_pagination_independent() {
    let container = GenericImage::new("postgis/postgis", "17-3.5")
        .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
        .with_exposed_port(5432.tcp())
        .with_env_var("POSTGRES_USER", "gis_test")
        .with_env_var("POSTGRES_PASSWORD", "gis_test")
        .with_env_var("POSTGRES_DB", "gis_test")
        .start().await.expect("start Testcontainers PostGIS");
    let port = container.get_host_port_ipv4(5432).await.expect("mapped port");
    let url = format!("postgres://gis_test:gis_test@127.0.0.1:{port}/gis_test");
    let pool = PgPoolOptions::new().max_connections(5).connect(&url).await.expect("connect");
    db::run_migrations(&pool).await.expect("run migrations");

    let region_a = insert_region(&pool, "results-a").await;
    let region_b = insert_region(&pool, "results-b").await;
    let building_a = insert_building(&pool, region_a, "way/91001", 91001).await;
    let building_b = insert_building(&pool, region_a, "way/91002", 91002).await;

    let simulation_id: Uuid = sqlx::query_scalar(
        "INSERT INTO public.simulation_runs (request_id, request_payload, status) VALUES ($1, '{}'::jsonb, 'initialized') RETURNING id"
    ).bind(Uuid::new_v4()).fetch_one(&pool).await.unwrap();

    for (building_id, population, infected) in [(building_a, 100.0_f64, 10.0_f64), (building_b, 200.0, 20.0)] {
        sqlx::query(
            "INSERT INTO public.simulation_building_states (simulation_id,building_id,population,initial_s,initial_i,initial_r) VALUES ($1,$2,$3,$4,$5,0)"
        ).bind(simulation_id).bind(building_id).bind(population).bind(population-infected).bind(infected).execute(&pool).await.unwrap();

        for (time_step, time, susceptible, current_infected, recovered) in [
            (0_i32, 0.0_f64, population-infected, infected, 0.0_f64),
            (1_i32, 0.1_f64, population-infected-2.0, infected+1.0, 1.0_f64),
        ] {
            sqlx::query(
                "INSERT INTO public.simulation_building_results (simulation_id,building_id,time_step,time,susceptible,infected,recovered) VALUES ($1,$2,$3,$4,$5,$6,$7)"
            ).bind(simulation_id).bind(building_id).bind(time_step).bind(time).bind(susceptible).bind(current_infected).bind(recovered).execute(&pool).await.unwrap();
        }
    }
    sqlx::query("UPDATE public.simulation_runs SET status='completed', completed_at=NOW() WHERE id=$1")
        .bind(simulation_id).execute(&pool).await.unwrap();

    let app = router(AppState::new(pool.clone()));
    let first_page = app.clone().oneshot(get_request(format!(
        "/api/v1/simulations/{simulation_id}/results?time_step=1&page=1&page_size=1"
    ))).await.unwrap();
    assert_eq!(first_page.status(), StatusCode::OK);
    let first_page = read_json(first_page).await;
    assert_eq!(first_page["data"].as_array().unwrap().len(), 1);
    assert_eq!(first_page["pagination"]["total"], 2);
    assert_eq!(first_page["time"], 0.1);

    let second_page = app.clone().oneshot(get_request(format!(
        "/api/v1/simulations/{simulation_id}/results?time_step=1&page=2&page_size=1"
    ))).await.unwrap();
    let second_page = read_json(second_page).await;
    assert_eq!(second_page["data"].as_array().unwrap().len(), 1);
    assert_ne!(first_page["data"][0]["building_id"], second_page["data"][0]["building_id"]);

    let aggregate = app.clone().oneshot(get_request(format!(
        "/api/v1/simulations/{simulation_id}/aggregate?time_step=1"
    ))).await.unwrap();
    assert_eq!(aggregate.status(), StatusCode::OK);
    let aggregate = read_json(aggregate).await;
    assert_eq!(aggregate["building_count"], 2);
    assert_eq!(aggregate["population"], 300.0);
    assert_eq!(aggregate["susceptible"], 278.0);
    assert_eq!(aggregate["infected"], 31.0);
    assert_eq!(aggregate["recovered"], 2.0);

    let scoped_empty = app.clone().oneshot(get_request(format!(
        "/api/v1/simulations/{simulation_id}/aggregate?time_step=1&region_id={region_b}"
    ))).await.unwrap();
    assert_eq!(scoped_empty.status(), StatusCode::OK);
    let scoped_empty = read_json(scoped_empty).await;
    assert_eq!(scoped_empty["building_count"], 0);
    assert_eq!(scoped_empty["population"], 0.0);

    let unknown_step = app.clone().oneshot(get_request(format!(
        "/api/v1/simulations/{simulation_id}/results?time_step=99"
    ))).await.unwrap();
    assert_eq!(unknown_step.status(), StatusCode::NOT_FOUND);

    let initialized_id: Uuid = sqlx::query_scalar(
        "INSERT INTO public.simulation_runs (request_id, request_payload) VALUES ($1, '{}'::jsonb) RETURNING id"
    ).bind(Uuid::new_v4()).fetch_one(&pool).await.unwrap();
    let not_ready = app.clone().oneshot(get_request(format!(
        "/api/v1/simulations/{initialized_id}/aggregate?time_step=0"
    ))).await.unwrap();
    assert_eq!(not_ready.status(), StatusCode::CONFLICT);
    let not_ready_body = read_json(not_ready).await;
    assert_eq!(not_ready_body["error"], "simulation_not_completed");

    let invalid_page = app.oneshot(get_request(format!(
        "/api/v1/simulations/{simulation_id}/results?time_step=1&page_size=501"
    ))).await.unwrap();
    assert_eq!(invalid_page.status(), StatusCode::BAD_REQUEST);
    pool.close().await;
}

async fn insert_region(pool: &sqlx::PgPool, code: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO public.regions (code,name,boundary_method,boundary) VALUES ($1,$1,'fixture',ST_GeomFromText('MULTIPOLYGON(((106 10,107 10,107 11,106 11,106 10)))',4326)) RETURNING id"
    ).bind(code).fetch_one(pool).await.unwrap()
}

async fn insert_building(pool: &sqlx::PgPool, region_id: Uuid, external_id: &str, osm_way_id: i64) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO public.buildings (external_id,osm_type,osm_way_id,region_id,footprint) VALUES ($1,'way',$2,$3,ST_GeomFromText('MULTIPOLYGON(((106.1 10.1,106.2 10.1,106.2 10.2,106.1 10.2,106.1 10.1)))',4326)) RETURNING id"
    ).bind(external_id).bind(osm_way_id).bind(region_id).fetch_one(pool).await.unwrap()
}

fn get_request(uri: String) -> Request<Body> {
    Request::builder().method("GET").uri(uri).body(Body::empty()).unwrap()
}

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
