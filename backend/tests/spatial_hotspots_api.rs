use axum::{
    body::Body,
    http::{Request, StatusCode},
};
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
async fn hotspot_api_builds_transitive_components_and_returns_union_geometry() {
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
        .expect("connect");
    db::run_migrations(&pool).await.expect("run migrations");

    let region_id = insert_region(&pool, "hotspot-region").await;
    let a = insert_building(&pool, region_id, "way/93001", 93001, 106.10000).await;
    let b = insert_building(&pool, region_id, "way/93002", 93002, 106.10015).await;
    let c = insert_building(&pool, region_id, "way/93003", 93003, 106.10030).await;
    let isolated = insert_building(&pool, region_id, "way/93004", 93004, 106.11000).await;

    let simulation_id: Uuid = sqlx::query_scalar(
        "INSERT INTO public.simulation_runs (request_id, request_payload, status) VALUES ($1, '{}'::jsonb, 'completed') RETURNING id",
    )
    .bind(Uuid::new_v4())
    .fetch_one(&pool)
    .await
    .unwrap();

    for building_id in [a, b, c, isolated] {
        sqlx::query(
            "INSERT INTO public.simulation_building_states (simulation_id,building_id,population,initial_s,initial_i,initial_r) VALUES ($1,$2,100,80,20,0)",
        )
        .bind(simulation_id)
        .bind(building_id)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO public.simulation_building_results (simulation_id,building_id,time_step,time,susceptible,infected,recovered) VALUES ($1,$2,0,0,80,20,0)",
        )
        .bind(simulation_id)
        .bind(building_id)
        .execute(&pool)
        .await
        .unwrap();
    }

    let medium_building = insert_building(
        &pool,
        region_id,
        "way/93006",
        93006,
        106.15000,
    )
    .await;
    sqlx::query(
        "INSERT INTO public.simulation_building_states (simulation_id,building_id,population,initial_s,initial_i,initial_r) VALUES ($1,$2,100,90,10,0)",
    )
    .bind(simulation_id)
    .bind(medium_building)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO public.simulation_building_results (simulation_id,building_id,time_step,time,susceptible,infected,recovered) VALUES ($1,$2,0,0,90,10,0)",
    )
    .bind(simulation_id)
    .bind(medium_building)
    .execute(&pool)
    .await
    .unwrap();

    let low_region_id = insert_region(&pool, "hotspot-low-region").await;
    let low_building = insert_building(
        &pool,
        low_region_id,
        "way/93005",
        93005,
        106.20000,
    )
    .await;
    sqlx::query(
        "INSERT INTO public.simulation_building_states (simulation_id,building_id,population,initial_s,initial_i,initial_r) VALUES ($1,$2,100,99,1,0)",
    )
    .bind(simulation_id)
    .bind(low_building)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO public.simulation_building_results (simulation_id,building_id,time_step,time,susceptible,infected,recovered) VALUES ($1,$2,0,0,99,1,0)",
    )
    .bind(simulation_id)
    .bind(low_building)
    .execute(&pool)
    .await
    .unwrap();

    let distance_ab: f64 = sqlx::query_scalar(
        "SELECT ST_Distance(a.footprint::geography,b.footprint::geography) FROM public.buildings a CROSS JOIN public.buildings b WHERE a.id=$1 AND b.id=$2",
    )
    .bind(a)
    .bind(b)
    .fetch_one(&pool)
    .await
    .unwrap();
    let distance_ac: f64 = sqlx::query_scalar(
        "SELECT ST_Distance(a.footprint::geography,b.footprint::geography) FROM public.buildings a CROSS JOIN public.buildings b WHERE a.id=$1 AND b.id=$2",
    )
    .bind(a)
    .bind(c)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(distance_ab <= 15.0);
    assert!(distance_ac > 15.0);

    let app = router(AppState::new(pool.clone()));
    let response = app
        .clone()
        .oneshot(get_request(format!(
            "/api/v1/simulations/{simulation_id}/hotspots?time_step=0&radius_m={distance_ab}"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = read_json(response).await;
    assert_eq!(body["hotspot_count"], 2);
    assert_eq!(body["configuration"]["algorithm"], "connected_components");
    assert_eq!(body["configuration"]["algorithm_version"], "hotspot-v1");
    assert_eq!(body["configuration"]["minimum_risk_level"], "high");
    assert_eq!(body["configuration"]["include_singletons"], true);
    assert_eq!(body["configuration"]["radius_m"], distance_ab);

    let cluster = body["hotspots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hotspot| hotspot["member_count"] == 3)
        .expect("A-B-C should form one transitive component");
    let mut expected_ids = vec![a, b, c];
    expected_ids.sort_unstable();
    let expected_cluster_id = expected_ids[0];
    assert_eq!(cluster["cluster_id"], expected_cluster_id.to_string());
    assert_eq!(cluster["building_ids"], serde_json::json!(expected_ids));
    assert_eq!(cluster["population_total"], 300.0);
    assert_eq!(cluster["infected_total"], 60.0);
    assert_eq!(cluster["infection_ratio"], 0.2);
    assert_eq!(cluster["risk_level"], "high");
    assert_eq!(cluster["geometry"]["type"], "MultiPolygon");
    assert!(!cluster["geometry"]["coordinates"].as_array().unwrap().is_empty());

    let without_singletons = app
        .clone()
        .oneshot(get_request(format!(
            "/api/v1/simulations/{simulation_id}/hotspots?time_step=0&radius_m={distance_ab}&include_singletons=false"
        )))
        .await
        .unwrap();
    assert_eq!(without_singletons.status(), StatusCode::OK);
    let without_singletons = read_json(without_singletons).await;
    assert_eq!(without_singletons["hotspot_count"], 1);
    assert_eq!(without_singletons["hotspots"][0]["member_count"], 3);

    let medium_candidates = app
        .clone()
        .oneshot(get_request(format!(
            "/api/v1/simulations/{simulation_id}/hotspots?time_step=0&minimum_risk_level=medium&radius_m={distance_ab}"
        )))
        .await
        .unwrap();
    assert_eq!(medium_candidates.status(), StatusCode::OK);
    let medium_candidates = read_json(medium_candidates).await;
    assert_eq!(medium_candidates["hotspot_count"], 3);
    let medium_building_id = medium_building.to_string();
    assert!(
        medium_candidates["hotspots"]
            .as_array()
            .unwrap()
            .iter()
            .any(|hotspot| {
                hotspot["building_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|id| id == &medium_building_id)
            })
    );

    let no_hotspots = app
        .clone()
        .oneshot(get_request(format!(
            "/api/v1/simulations/{simulation_id}/hotspots?time_step=0&region_id={low_region_id}"
        )))
        .await
        .unwrap();
    assert_eq!(no_hotspots.status(), StatusCode::OK);
    let no_hotspots = read_json(no_hotspots).await;
    assert_eq!(no_hotspots["hotspot_count"], 0);
    assert_eq!(no_hotspots["hotspots"], serde_json::json!([]));

    let invalid_radius = app
        .clone()
        .oneshot(get_request(format!(
            "/api/v1/simulations/{simulation_id}/hotspots?time_step=0&radius_m=5001"
        )))
        .await
        .unwrap();
    assert_eq!(invalid_radius.status(), StatusCode::BAD_REQUEST);

    let missing_step = app
        .oneshot(get_request(format!(
            "/api/v1/simulations/{simulation_id}/hotspots?time_step=99"
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
    region_id: Uuid,
    external_id: &str,
    osm_way_id: i64,
    min_longitude: f64,
) -> Uuid {
    let max_longitude = min_longitude + 0.00005;
    let wkt = format!(
        "MULTIPOLYGON((({min_longitude} 10.1,{max_longitude} 10.1,{max_longitude} 10.10005,{min_longitude} 10.10005,{min_longitude} 10.1)))"
    );
    sqlx::query_scalar(
        "INSERT INTO public.buildings (external_id,osm_type,osm_way_id,region_id,footprint) VALUES ($1,'way',$2,$3,ST_GeomFromText($4,4326)) RETURNING id",
    )
    .bind(external_id)
    .bind(osm_way_id)
    .bind(region_id)
    .bind(wkt)
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

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
