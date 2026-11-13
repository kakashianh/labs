use axum::{body::Body, http::{Request, StatusCode}};
use backend::{db, router::router, state::AppState};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use testcontainers::{core::{IntoContainerPort, WaitFor}, runners::AsyncRunner, GenericImage, ImageExt};
use tower::ServiceExt;

#[tokio::test]
async fn region_and_building_detail_apis_work_against_postgis() {
    let container = GenericImage::new("postgis/postgis", "17-3.5")
        .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
        .with_exposed_port(5432.tcp())
        .with_env_var("POSTGRES_USER", "gis_test")
        .with_env_var("POSTGRES_PASSWORD", "gis_test")
        .with_env_var("POSTGRES_DB", "gis_test")
        .start().await.expect("start PostGIS container");
    let port=container.get_host_port_ipv4(5432).await.expect("mapped port");
    let url=format!("postgres://gis_test:gis_test@127.0.0.1:{port}/gis_test");
    let pool=PgPoolOptions::new().max_connections(5).connect(&url).await.expect("connect DB");
    db::run_migrations(&pool).await.expect("run migrations");

    let region_id: uuid::Uuid=sqlx::query_scalar(
        r#"INSERT INTO public.regions (code,name,description,boundary_method,source_reference,boundary)
        VALUES ('A-REGION','Region A',NULL,'fixture',NULL,
        ST_GeomFromText('MULTIPOLYGON(((106 10,107 10,107 11,106 11,106 10)))',4326)) RETURNING id"#)
        .fetch_one(&pool).await.expect("insert region");
    let other_region: uuid::Uuid=sqlx::query_scalar(
        r#"INSERT INTO public.regions (code,name,boundary_method,boundary)
        VALUES ('B-REGION','Region B','fixture',
        ST_GeomFromText('MULTIPOLYGON(((107 10,108 10,108 11,107 11,107 10)))',4326)) RETURNING id"#)
        .fetch_one(&pool).await.expect("insert second region");
    let building_id: uuid::Uuid=sqlx::query_scalar(
        r#"INSERT INTO public.buildings
        (external_id,osm_type,osm_way_id,region_id,name,building_type,footprint)
        VALUES ('way/1001','way',1001,$1,NULL,NULL,
        ST_GeomFromText('MULTIPOLYGON(((106.1 10.1,106.2 10.1,106.2 10.2,106.1 10.2,106.1 10.1)))',4326))
        RETURNING id"#)
        .bind(region_id).fetch_one(&pool).await.expect("insert building");

    let app=router(AppState::new(pool.clone()));
    let list=app.clone().oneshot(Request::builder().uri("/api/v1/regions").body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(list.status(),StatusCode::OK);
    let body=axum::body::to_bytes(list.into_body(),1024*1024).await.unwrap();
    let json:Value=serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"].as_array().unwrap().len(),2);
    assert_eq!(json["data"][0]["code"],"A-REGION");

    let region=app.clone().oneshot(Request::builder().uri(format!("/api/v1/regions/{region_id}")).body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(region.status(),StatusCode::OK);
    let body=axum::body::to_bytes(region.into_body(),1024*1024).await.unwrap();
    let json:Value=serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["geometry"]["type"],"MultiPolygon");
    assert!(json["data"]["description"].is_null());
    assert!(json["data"]["source_reference"].is_null());

    let building=app.clone().oneshot(Request::builder().uri(format!("/api/v1/buildings/{building_id}")).body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(building.status(),StatusCode::OK);
    let body=axum::body::to_bytes(building.into_body(),1024*1024).await.unwrap();
    let json:Value=serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["region_id"],region_id.to_string());
    assert_eq!(json["data"]["geometry"]["type"],"MultiPolygon");
    assert!(json["data"]["name"].is_null());
    assert!(json["data"]["building_type"].is_null());

    for uri in ["/api/v1/regions/not-a-uuid","/api/v1/buildings/not-a-uuid"] {
        let response=app.clone().oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(),StatusCode::BAD_REQUEST, "{uri}");
    }
    let missing=uuid::Uuid::parse_str("cccccccc-cccc-4ccc-8ccc-cccccccccccc").unwrap();
    for uri in [format!("/api/v1/regions/{missing}"),format!("/api/v1/buildings/{missing}")] {
        let response=app.clone().oneshot(Request::builder().uri(uri.clone()).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(),StatusCode::NOT_FOUND, "{uri}");
    }

    let docs=app.clone().oneshot(Request::builder().uri("/api-docs/openapi.json").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(docs.status(),StatusCode::OK);
    let body=axum::body::to_bytes(docs.into_body(),1024*1024).await.unwrap();
    let spec:Value=serde_json::from_slice(&body).unwrap();
    assert!(spec["paths"]["/api/v1/regions"]["get"].is_object());
    assert!(spec["paths"]["/api/v1/regions/{region_id}"]["get"].is_object());
    assert!(spec["paths"]["/api/v1/buildings/{building_id}"]["get"].is_object());
    assert!(other_region != region_id);

    // A closed pool simulates an unavailable database; the public response must
    // remain generic and must not disclose SQL or connection details.
    pool.close().await;
    let unavailable_app=router(AppState::new(pool));
    let unavailable=unavailable_app.oneshot(
        Request::builder().uri("/api/v1/regions").body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(unavailable.status(),StatusCode::INTERNAL_SERVER_ERROR);
    let body=axum::body::to_bytes(unavailable.into_body(),1024*1024).await.unwrap();
    let error:Value=serde_json::from_slice(&body).unwrap();
    assert_eq!(error["error"],"Internal server error");
    assert!(!body.windows(b"postgres://".len()).any(|window| window==b"postgres://"));
}
