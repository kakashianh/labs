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

#[tokio::test]
async fn buildings_api_works_against_real_postgis() {
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
        .expect("start Testcontainers PostGIS");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("resolve mapped PostGIS port");
    let database_url = format!("postgres://gis_test:gis_test@127.0.0.1:{port}/gis_test");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("connect to Testcontainers PostGIS");
    db::run_migrations(&pool)
        .await
        .expect("apply application migrations");

    let region_a: uuid::Uuid = sqlx::query_scalar(
        r#"INSERT INTO public.regions (code,name,boundary_method,boundary)
           VALUES ('test-a','Test A','fixture',ST_GeomFromText('MULTIPOLYGON(((106 10,107 10,107 11,106 11,106 10)))',4326))
           RETURNING id"#,
    )
    .fetch_one(&pool)
    .await
    .expect("insert region A");
    let region_b: uuid::Uuid = sqlx::query_scalar(
        r#"INSERT INTO public.regions (code,name,boundary_method,boundary)
           VALUES ('test-b','Test B','fixture',ST_GeomFromText('MULTIPOLYGON(((107 10,108 10,108 11,107 11,107 10)))',4326))
           RETURNING id"#,
    )
    .fetch_one(&pool)
    .await
    .expect("insert region B");

    for (external_id, osm_way_id, region_id, name) in [
        ("way/1001", 1001_i64, region_a, Some("Building A")),
        ("way/1002", 1002_i64, region_a, None),
        ("way/2001", 2001_i64, region_b, Some("Building B")),
    ] {
        sqlx::query(
            r#"INSERT INTO public.buildings
               (external_id,osm_type,osm_way_id,region_id,name,building_type,footprint)
               VALUES ($1,'way',$2,$3,$4,'residential',
                 ST_GeomFromText('MULTIPOLYGON(((106.1 10.1,106.2 10.1,106.2 10.2,106.1 10.2,106.1 10.1)))',4326))"#,
        )
        .bind(external_id)
        .bind(osm_way_id)
        .bind(region_id)
        .bind(name)
        .execute(&pool)
        .await
        .expect("insert building fixture");
    }

    let app = router(AppState::new(pool.clone()));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/buildings?region_id={region_a}&page=1&page_size=10"
                ))
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("call Axum router");
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("read body");
    let json: Value = serde_json::from_slice(&body).expect("valid JSON response");
    assert_eq!(json["data"].as_array().expect("data array").len(), 2);
    assert_eq!(json["pagination"]["total"], 2);
    assert_eq!(json["pagination"]["total_pages"], 1);
    assert!(json["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["name"].is_null()));
    assert!(json["data"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["geometry"]["type"] == "MultiPolygon"));

    let unfiltered_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/buildings?page_size=100")
                .body(Body::empty())
                .expect("build unfiltered request"),
        )
        .await
        .expect("call unfiltered route");
    assert_eq!(unfiltered_response.status(), StatusCode::OK);
    let unfiltered_body = axum::body::to_bytes(unfiltered_response.into_body(), 1024 * 1024)
        .await
        .expect("read unfiltered body");
    let unfiltered_json: Value =
        serde_json::from_slice(&unfiltered_body).expect("valid unfiltered JSON");
    assert_eq!(unfiltered_json["data"].as_array().unwrap().len(), 3);
    assert_eq!(unfiltered_json["pagination"]["total"], 3);

    let first_page_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/buildings?region_id={region_a}&page=1&page_size=1"
                ))
                .body(Body::empty())
                .expect("build first-page request"),
        )
        .await
        .expect("call first-page route");
    let second_page_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/buildings?region_id={region_a}&page=2&page_size=1"
                ))
                .body(Body::empty())
                .expect("build second-page request"),
        )
        .await
        .expect("call second-page route");
    let first_page_body = axum::body::to_bytes(first_page_response.into_body(), 1024 * 1024)
        .await
        .expect("read first-page body");
    let second_page_body = axum::body::to_bytes(second_page_response.into_body(), 1024 * 1024)
        .await
        .expect("read second-page body");
    let first_page_json: Value =
        serde_json::from_slice(&first_page_body).expect("valid first-page JSON");
    let second_page_json: Value =
        serde_json::from_slice(&second_page_body).expect("valid second-page JSON");
    assert_eq!(first_page_json["pagination"]["total_pages"], 2);
    assert_eq!(second_page_json["pagination"]["total_pages"], 2);
    assert_eq!(first_page_json["data"].as_array().unwrap().len(), 1);
    assert_eq!(second_page_json["data"].as_array().unwrap().len(), 1);
    assert!(
        first_page_json["data"][0]["id"].as_str().unwrap()
            < second_page_json["data"][0]["id"].as_str().unwrap()
    );

    let empty_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/buildings?region_id={region_b}&page=99"))
                .body(Body::empty())
                .expect("build empty-page request"),
        )
        .await
        .expect("call empty-page route");
    assert_eq!(empty_response.status(), StatusCode::OK);
    let empty_body = axum::body::to_bytes(empty_response.into_body(), 1024 * 1024)
        .await
        .expect("read empty body");
    let empty_json: Value = serde_json::from_slice(&empty_body).expect("valid empty JSON");
    assert_eq!(empty_json["data"].as_array().unwrap().len(), 0);
    assert_eq!(empty_json["pagination"]["total"], 1);

    let no_matches_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/buildings?region_id=cccccccc-cccc-4ccc-8ccc-cccccccccccc")
                .body(Body::empty())
                .expect("build no-matches request"),
        )
        .await
        .expect("call no-matches route");
    assert_eq!(no_matches_response.status(), StatusCode::OK);
    let no_matches_body = axum::body::to_bytes(no_matches_response.into_body(), 1024 * 1024)
        .await
        .expect("read no-matches body");
    let no_matches_json: Value =
        serde_json::from_slice(&no_matches_body).expect("valid no-matches JSON");
    assert_eq!(no_matches_json["data"].as_array().unwrap().len(), 0);
    assert_eq!(no_matches_json["pagination"]["total"], 0);
    assert_eq!(no_matches_json["pagination"]["total_pages"], 0);

    let docs_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api-docs/openapi.json")
                .body(Body::empty())
                .expect("build OpenAPI request"),
        )
        .await
        .expect("call OpenAPI route");
    assert_eq!(docs_response.status(), StatusCode::OK);
    let docs_body = axum::body::to_bytes(docs_response.into_body(), 1024 * 1024)
        .await
        .expect("read OpenAPI body");
    let docs_json: Value = serde_json::from_slice(&docs_body).expect("valid OpenAPI JSON");
    assert!(docs_json["paths"]["/api/v1/buildings"]["get"].is_object());

    let invalid_response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/buildings?region_id=not-a-uuid")
                .body(Body::empty())
                .expect("build invalid request"),
        )
        .await
        .expect("call invalid route");
    assert_eq!(invalid_response.status(), StatusCode::BAD_REQUEST);
    pool.close().await;
}
