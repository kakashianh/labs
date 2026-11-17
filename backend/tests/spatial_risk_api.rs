use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::{db, router::router, state::AppState};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage, ImageExt,
};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn risk_classification_persists_thresholds_and_aggregates_all_buildings() {
    let container = GenericImage::new("postgis/postgis", "17-3.5")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_exposed_port(5432.tcp())
        .with_env_var("POSTGRES_USER", "gis_test")
        .with_env_var("POSTGRES_PASSWORD", "gis_test")
        .with_env_var("POSTGRES_DB", "gis_test")
        .start()
        .await
        .expect("start PostGIS");
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://gis_test:gis_test@127.0.0.1:{port}/gis_test");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .unwrap();
    db::run_migrations(&pool).await.unwrap();

    let region = insert_region(&pool, "risk-region").await;
    let low_building = insert_building(&pool, region, "way/92001", 92001).await;
    let high_building = insert_building(&pool, region, "way/92002", 92002).await;
    let missing_population_building =
        insert_building(&pool, region, "way/92003", 92003).await;
    let simulation_id: Uuid = sqlx::query_scalar(
        "INSERT INTO public.simulation_runs (request_id, request_payload, status) VALUES ($1, '{}'::jsonb, 'completed') RETURNING id",
    )
    .bind(Uuid::new_v4())
    .fetch_one(&pool)
    .await
    .unwrap();

    for (building_id, population, infected) in [
        (low_building, 100.0_f64, 5.0_f64),
        (high_building, 900.0, 135.0),
    ] {
        sqlx::query(
            "INSERT INTO public.simulation_building_states (simulation_id,building_id,population,initial_s,initial_i,initial_r) VALUES ($1,$2,$3,$4,$5,0)",
        )
        .bind(simulation_id)
        .bind(building_id)
        .bind(population)
        .bind(population - infected)
        .bind(infected)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO public.simulation_building_results (simulation_id,building_id,time_step,time,susceptible,infected,recovered) VALUES ($1,$2,0,0,$3,$4,0)",
        )
        .bind(simulation_id)
        .bind(building_id)
        .bind(population - infected)
        .bind(infected)
        .execute(&pool)
        .await
        .unwrap();
    }
    sqlx::query(
        "INSERT INTO public.simulation_building_exclusions (simulation_id,building_id,reason_code) VALUES ($1,$2,'missing_population')",
    )
    .bind(simulation_id)
    .bind(missing_population_building)
    .execute(&pool)
    .await
    .unwrap();

    let app = router(AppState::new(pool.clone()));
    let configured = app
        .clone()
        .oneshot(post_json(
            format!("/api/v1/simulations/{simulation_id}/risk-classification"),
            json!({"risk_low_max": 0.05, "risk_medium_max": 0.15}),
        ))
        .await
        .unwrap();
    assert_eq!(configured.status(), StatusCode::OK);

    let listed = app
        .clone()
        .oneshot(get_request(format!(
            "/api/v1/simulations/{simulation_id}/risk-results?time_step=0&page=1&page_size=1"
        )))
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let listed = read_json(listed).await;
    assert_eq!(listed["pagination"]["total"], 3);
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);

    let aggregate = app
        .clone()
        .oneshot(get_request(format!(
            "/api/v1/simulations/{simulation_id}/risk-aggregate?time_step=0&region_id={region}"
        )))
        .await
        .unwrap();
    assert_eq!(aggregate.status(), StatusCode::OK);
    let aggregate = read_json(aggregate).await;
    let summary = &aggregate["data"][0];
    assert_eq!(summary["total_building_count"], 3);
    assert_eq!(summary["unclassified_building_count"], 1);
    assert!((summary["infection_ratio"].as_f64().unwrap() - 0.14).abs() < 1e-12);
    assert_eq!(summary["buckets"][0]["risk_level"], "low");
    assert_eq!(summary["buckets"][0]["building_count"], 1);
    assert_eq!(summary["buckets"][2]["risk_level"], "high");
    assert_eq!(summary["buckets"][2]["building_count"], 1);
    assert_eq!(summary["buckets"][3]["risk_level"], "unclassified");

    let invalid = app
        .clone()
        .oneshot(post_json(
            format!("/api/v1/simulations/{simulation_id}/risk-classification"),
            json!({"risk_low_max": 0.2, "risk_medium_max": 0.1}),
        ))
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);

    let missing_step = app
        .oneshot(get_request(format!(
            "/api/v1/simulations/{simulation_id}/risk-results?time_step=99"
        )))
        .await
        .unwrap();
    assert_eq!(missing_step.status(), StatusCode::NOT_FOUND);
    pool.close().await;
}

async fn insert_region(pool: &sqlx::PgPool, code: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO public.regions (code,name,boundary_method,boundary) VALUES ($1,$1,'fixture',ST_GeomFromText('MULTIPOLYGON(((106 10,107 10,107 11,106 11,106 10)))',4326)) RETURNING id",
    )
    .bind(code)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn insert_building(
    pool: &sqlx::PgPool,
    region: Uuid,
    external_id: &str,
    osm_id: i64,
) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO public.buildings (external_id,osm_type,osm_way_id,region_id,footprint) VALUES ($1,'way',$2,$3,ST_GeomFromText('MULTIPOLYGON(((106.1 10.1,106.2 10.1,106.2 10.2,106.1 10.2,106.1 10.1)))',4326)) RETURNING id",
    )
    .bind(external_id)
    .bind(osm_id)
    .bind(region)
    .fetch_one(pool)
    .await
    .unwrap()
}

fn get_request(uri: String) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

fn post_json(uri: String, value: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(value.to_string()))
        .unwrap()
}

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
