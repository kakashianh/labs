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
async fn building_neighbor_api_uses_metric_footprint_distance_and_batch_adjacency() {
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
        .expect("start PostGIS container");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("mapped port");
    let url = format!("postgres://gis_test:gis_test@127.0.0.1:{port}/gis_test");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("connect DB");
    db::run_migrations(&pool).await.expect("run migrations");

    let region_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO public.regions
           (code,name,boundary_method,boundary)
           VALUES ('NEIGHBOR-REGION','Neighbor Region','fixture',
           ST_GeomFromText('MULTIPOLYGON(((106 10,107 10,107 11,106 11,106 10)))',4326))
           RETURNING id"#,
    )
    .fetch_one(&pool)
    .await
    .expect("insert region");

    async fn insert_building(
        pool: &sqlx::PgPool,
        region_id: Uuid,
        external_id: &str,
        wkt: &str,
    ) -> Uuid {
        sqlx::query_scalar(
            r#"INSERT INTO public.buildings
               (external_id,osm_type,osm_way_id,region_id,footprint)
               VALUES ($1,'way',abs(hashtext($1))::bigint,$2,ST_GeomFromText($3,4326))
               RETURNING id"#,
        )
        .bind(external_id)
        .bind(region_id)
        .bind(wkt)
        .fetch_one(pool)
        .await
        .expect("insert building fixture")
    }

    let center = insert_building(
        &pool,
        region_id,
        "way/neighbor-center",
        "MULTIPOLYGON(((106 10,106.001 10,106.001 10.001,106 10.001,106 10)))",
    )
    .await;
    let touching = insert_building(
        &pool,
        region_id,
        "way/neighbor-touching",
        "MULTIPOLYGON(((106.001 10,106.002 10,106.002 10.001,106.001 10.001,106.001 10)))",
    )
    .await;
    let near = insert_building(
        &pool,
        region_id,
        "way/neighbor-near",
        "MULTIPOLYGON(((106.0011 10,106.0021 10,106.0021 10.001,106.0011 10.001,106.0011 10)))",
    )
    .await;
    let far = insert_building(
        &pool,
        region_id,
        "way/neighbor-far",
        "MULTIPOLYGON(((106.02 10,106.021 10,106.021 10.001,106.02 10.001,106.02 10)))",
    )
    .await;
    let invalid = insert_building(
        &pool,
        region_id,
        "way/neighbor-invalid",
        "MULTIPOLYGON(((106.01 10.01,106.012 10.012,106.01 10.012,106.012 10.01,106.01 10.01)))",
    )
    .await;

    let missing_id = Uuid::new_v4();
    let app = router(AppState::new(pool.clone()));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/buildings/{center}/neighbors?radius_m=20&limit=10"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["center_building_id"], center.to_string());
    assert_eq!(json["distance_semantics"], "footprint_to_footprint");
    assert_eq!(json["excluded_invalid_geometry_count"], 1);
    let neighbors = json["data"].as_array().unwrap();
    assert_eq!(neighbors.len(), 2);
    assert_eq!(neighbors[0]["building_id"], touching.to_string());
    assert_eq!(neighbors[0]["distance_m"], 0.0);
    assert_eq!(neighbors[1]["building_id"], near.to_string());
    let near_distance = neighbors[1]["distance_m"].as_f64().unwrap();
    assert!(near_distance > 10.0 && near_distance < 12.0);
    assert!(!neighbors
        .iter()
        .any(|item| item["building_id"] == far.to_string()));
    assert!(!neighbors
        .iter()
        .any(|item| item["building_id"] == invalid.to_string()));

    // ST_DWithin is inclusive at the exact footprint distance.
    let exact_radius_m: f64 = sqlx::query_scalar(
        r#"SELECT ST_Distance(center.footprint::geography, neighbor.footprint::geography)
           FROM public.buildings center
           JOIN public.buildings neighbor ON neighbor.id = $2
           WHERE center.id = $1"#,
    )
    .bind(center)
    .bind(near)
    .fetch_one(&pool)
    .await
    .expect("calculate exact footprint distance");
    let included_at_exact_boundary: bool = sqlx::query_scalar(
        r#"SELECT ST_DWithin(center.footprint::geography, neighbor.footprint::geography,
                            ST_Distance(center.footprint::geography, neighbor.footprint::geography))
           FROM public.buildings center
           JOIN public.buildings neighbor ON neighbor.id = $2
           WHERE center.id = $1"#,
    )
    .bind(center)
    .bind(near)
    .fetch_one(&pool)
    .await
    .expect("check inclusive PostGIS radius boundary");
    assert!(included_at_exact_boundary);

    let boundary_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/buildings/{center}/neighbors?radius_m={}",
                    exact_radius_m + 0.001
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(boundary_response.status(), StatusCode::OK);
    let boundary_body = axum::body::to_bytes(boundary_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let boundary_json: Value = serde_json::from_slice(&boundary_body).unwrap();
    assert!(boundary_json["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["building_id"] == near.to_string()));

    let query_plan: Value = sqlx::query_scalar(
        r#"
        EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)
        WITH valid_buildings AS MATERIALIZED (
            SELECT id, footprint
            FROM public.buildings
            WHERE id <> $1
              AND ST_SRID(footprint) = 4326
              AND NOT ST_IsEmpty(footprint)
              AND ST_IsValid(footprint)
        )
        SELECT neighbor.id AS building_id,
               ST_Distance(center.footprint::geography,
                           neighbor.footprint::geography) AS distance_m
        FROM public.buildings AS center
        JOIN valid_buildings AS neighbor
          ON ST_DWithin(center.footprint::geography,
                        neighbor.footprint::geography, $2)
        WHERE center.id = $1
        ORDER BY distance_m ASC, neighbor.id ASC
        LIMIT $3
        "#,
    )
    .bind(center)
    .bind(20.0_f64)
    .bind(10_i64)
    .fetch_one(&pool)
    .await
    .expect("EXPLAIN ANALYZE neighbor query");
    assert!(query_plan[0]["Plan"].is_object());
    println!("PostGIS neighbor EXPLAIN ANALYZE: {query_plan}");

    // The batch graph is undirected: each pair occurs once and invalid geometries
    // never enter the graph.
    let missing_building = Uuid::new_v4();
    let adjacency = db::building_neighbors::build_building_adjacency(
        &pool,
        &[center, touching, near, far, invalid, missing_building],
        20.0,
    )
    .await
    .expect("build adjacency");
    assert!(adjacency.edges.len() >= 2);
    assert_eq!(adjacency.excluded_invalid_geometry_count, 1);
    assert_eq!(adjacency.missing_building_count, 1);
    assert!(adjacency
        .edges
        .iter()
        .all(|edge| edge.building_a_id < edge.building_b_id));
    assert!(adjacency
        .edges
        .iter()
        .all(|edge| { edge.building_a_id != invalid && edge.building_b_id != invalid }));
    assert!(adjacency.edges.iter().any(|edge| {
        edge.building_a_id == center && edge.building_b_id == touching
            || edge.building_a_id == touching && edge.building_b_id == center
    }));

    for (uri, expected) in [
        (
            format!("/api/v1/buildings/{center}/neighbors?limit=10"),
            StatusCode::BAD_REQUEST,
        ),
        (
            format!("/api/v1/buildings/{center}/neighbors?radius_m=0"),
            StatusCode::BAD_REQUEST,
        ),
        (
            format!("/api/v1/buildings/{center}/neighbors?radius_m=5001"),
            StatusCode::BAD_REQUEST,
        ),
        (
            format!("/api/v1/buildings/{center}/neighbors?limit=501"),
            StatusCode::BAD_REQUEST,
        ),
        (
            "/api/v1/buildings/not-a-uuid/neighbors".to_string(),
            StatusCode::BAD_REQUEST,
        ),
        (
            format!("/api/v1/buildings/{invalid}/neighbors"),
            StatusCode::BAD_REQUEST,
        ),
        (
            format!("/api/v1/buildings/{missing_id}/neighbors"),
            StatusCode::NOT_FOUND,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(uri.clone())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected, "{uri}");
    }

    let empty = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/buildings/{far}/neighbors?radius_m=1"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(empty.status(), StatusCode::OK);
    let body = axum::body::to_bytes(empty.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"].as_array().unwrap().len(), 0);
}
