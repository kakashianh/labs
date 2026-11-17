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
async fn spatial_sir_initialization_persists_states_exclusions_and_idempotency() {
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
        .expect("mapped port");
    let url = format!("postgres://gis_test:gis_test@127.0.0.1:{port}/gis_test");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("connect");
    db::run_migrations(&pool).await.expect("run migrations");

    let region_id: Uuid = sqlx::query_scalar(
        "INSERT INTO public.regions (code,name,boundary_method,boundary) VALUES ('sir-test','SIR test','fixture',ST_GeomFromText('MULTIPOLYGON(((106 10,107 10,107 11,106 11,106 10)))',4326)) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("insert region");
    let mut building_a = insert_building(&pool, region_id, "way/81001", 81001).await;
    let mut building_b = insert_building(&pool, region_id, "way/81002", 81002).await;
    // Keep the eligible building's UUID smaller so the excluded building would
    // have appeared as a candidate in the old unfiltered lateral scan.
    if building_a > building_b {
        std::mem::swap(&mut building_a, &mut building_b);
    }

    let app = router(AppState::new(pool.clone()));
    let request_id = Uuid::new_v4();
    let payload = json!({
        "request_id": request_id,
        "buildings": [
            {"building_id": building_a, "population": 100.0, "initial_infected": 5.0},
            {"building_id": building_b}
        ]
    });
    let response = app
        .clone()
        .oneshot(post_request(payload.clone()))
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = read_json(response).await;
    assert_eq!(body["eligible"].as_array().unwrap().len(), 1);
    assert_eq!(body["excluded"][0]["reason_code"], "missing_population");
    assert_eq!(body["eligible"][0]["initial_s"], 95.0);
    assert_eq!(body["eligible"][0]["initial_i"], 5.0);
    assert_eq!(body["eligible"][0]["initial_r"], 0.0);
    assert_eq!(body["idempotent_replay"], false);
    let simulation_id = body["simulation_id"].as_str().unwrap();

    let replay = app
        .clone()
        .oneshot(post_request(payload.clone()))
        .await
        .expect("retry");
    assert_eq!(replay.status(), StatusCode::OK);
    let replay_body = read_json(replay).await;
    assert_eq!(replay_body["simulation_id"], simulation_id);
    assert_eq!(replay_body["idempotent_replay"], true);

    let mut conflicting_payload = payload.clone();
    conflicting_payload["buildings"][0]["population"] = json!(101.0);
    let conflict = app
        .clone()
        .oneshot(post_request(conflicting_payload))
        .await
        .expect("conflict");
    assert_eq!(conflict.status(), StatusCode::CONFLICT);

    let simulation_uuid = Uuid::parse_str(simulation_id).unwrap();
    let initialized_summary = app
        .clone()
        .oneshot(get_run_request(simulation_uuid))
        .await
        .expect("get initialized run");
    assert_eq!(initialized_summary.status(), StatusCode::OK);
    let initialized_summary = read_json(initialized_summary).await;
    assert_eq!(initialized_summary["status"], "initialized");
    assert_eq!(initialized_summary["eligible_building_count"], 1);
    assert_eq!(initialized_summary["excluded_building_count"], 1);

    let states: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.simulation_building_states WHERE simulation_id = $1",
    )
    .bind(simulation_uuid)
    .fetch_one(&pool)
    .await
    .unwrap();
    let exclusions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.simulation_building_exclusions WHERE simulation_id = $1",
    )
    .bind(simulation_uuid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(states, 1);
    assert_eq!(exclusions, 1);

    // The adjacency graph must contain only eligible simulation buildings;
    // building_b is spatially overlapping but excluded due to missing population.
    let eligible_only_adjacency =
        db::building_neighbors::build_building_adjacency(&pool, &[building_a], 100.0)
            .await
            .expect("build eligible-only adjacency");
    assert!(eligible_only_adjacency.edges.is_empty());

    sqlx::query(
        "CREATE FUNCTION public.reject_spatial_sir_result() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'injected persistence failure'; END; $$",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "CREATE TRIGGER reject_spatial_sir_result BEFORE INSERT ON public.simulation_building_results FOR EACH ROW EXECUTE FUNCTION public.reject_spatial_sir_result()",
    )
    .execute(&pool)
    .await
    .unwrap();

    let failed_run = app
        .clone()
        .oneshot(post_run_request(
            simulation_uuid,
            json!({"radius_m": 100.0, "beta": 0.3, "gamma": 0.1, "dt": 0.1, "steps": 2}),
        ))
        .await
        .expect("failed persistence request");
    let failed_status = failed_run.status();
    let failed_body = read_json(failed_run).await;
    assert_eq!(
        failed_status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "unexpected run error response: {failed_body}"
    );
    let failed_status: String =
        sqlx::query_scalar("SELECT status FROM public.simulation_runs WHERE id = $1")
            .bind(simulation_uuid)
            .fetch_one(&pool)
            .await
            .unwrap();
    let failed_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.simulation_building_results WHERE simulation_id = $1",
    )
    .bind(simulation_uuid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(failed_status, "initialized");
    assert_eq!(failed_rows, 0);
    sqlx::query("DROP TRIGGER reject_spatial_sir_result ON public.simulation_building_results")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DROP FUNCTION public.reject_spatial_sir_result()")
        .execute(&pool)
        .await
        .unwrap();

    let run_payload = json!({
        "radius_m": 100.0,
        "beta": 0.3,
        "gamma": 0.1,
        "dt": 0.1,
        "steps": 2
    });
    let run_response = app
        .clone()
        .oneshot(post_run_request(simulation_uuid, run_payload.clone()))
        .await
        .expect("run simulation");
    assert_eq!(run_response.status(), StatusCode::CREATED);
    let run_body = read_json(run_response).await;
    assert_eq!(run_body["status"], "completed");
    assert_eq!(run_body["building_count"], 1);
    assert_eq!(run_body["point_count"], 3);
    assert_eq!(run_body["time_step_count"], 3);
    assert_eq!(run_body["idempotent_replay"], false);

    let persisted: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.simulation_building_results WHERE simulation_id = $1",
    )
    .bind(simulation_uuid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(persisted, 3);
    let completed_summary = app
        .clone()
        .oneshot(get_run_request(simulation_uuid))
        .await
        .expect("get completed run");
    let completed_summary = read_json(completed_summary).await;
    assert_eq!(completed_summary["status"], "completed");
    assert_eq!(completed_summary["result_point_count"], 3);
    assert_eq!(completed_summary["time_step_count"], 3);
    assert!(completed_summary["completed_at"].is_string());

    let bad_states: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.simulation_building_results WHERE simulation_id = $1 AND abs((susceptible + infected + recovered) - 100.0) > 1e-6",
    )
    .bind(simulation_uuid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(bad_states, 0);

    let run_replay = app
        .clone()
        .oneshot(post_run_request(simulation_uuid, run_payload.clone()))
        .await
        .expect("replay simulation");
    assert_eq!(run_replay.status(), StatusCode::OK);
    assert_eq!(read_json(run_replay).await["idempotent_replay"], true);

    let mut different_config = run_payload;
    different_config["beta"] = json!(0.4);
    let run_conflict = app
        .clone()
        .oneshot(post_run_request(simulation_uuid, different_config))
        .await
        .expect("conflicting simulation replay");
    assert_eq!(run_conflict.status(), StatusCode::CONFLICT);

    let all_excluded_payload = json!({
        "request_id": Uuid::new_v4(),
        "buildings": [{"building_id": building_a}, {"building_id": building_b}]
    });
    let all_excluded = app
        .clone()
        .oneshot(post_request(all_excluded_payload.clone()))
        .await
        .expect("all excluded");
    assert_eq!(all_excluded.status(), StatusCode::BAD_REQUEST);
    let run_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM public.simulation_runs WHERE request_id = $1")
            .bind(
                all_excluded_payload["request_id"]
                    .as_str()
                    .unwrap()
                    .parse::<Uuid>()
                    .unwrap(),
            )
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(run_count, 0);

    let unknown_payload = json!({
        "request_id": Uuid::new_v4(),
        "buildings": [{"building_id": Uuid::new_v4(), "population": 10.0, "initial_infected": 0.0}]
    });
    let unknown = app
        .oneshot(post_request(unknown_payload))
        .await
        .expect("unknown");
    assert_eq!(unknown.status(), StatusCode::BAD_REQUEST);
    pool.close().await;
}

async fn insert_building(
    pool: &sqlx::PgPool,
    region_id: Uuid,
    external_id: &str,
    osm_way_id: i64,
) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO public.buildings (external_id,osm_type,osm_way_id,region_id,footprint) VALUES ($1,'way',$2,$3,ST_GeomFromText('MULTIPOLYGON(((106.1 10.1,106.2 10.1,106.2 10.2,106.1 10.2,106.1 10.1)))',4326)) RETURNING id",
    )
    .bind(external_id)
    .bind(osm_way_id)
    .bind(region_id)
    .fetch_one(pool)
    .await
    .expect("insert building")
}

fn post_request(payload: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/simulations/initialize")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .expect("build request")
}

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).expect("JSON")
}

fn post_run_request(simulation_id: Uuid, payload: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!("/api/v1/simulations/{simulation_id}/run"))
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .expect("build run request")
}

fn get_run_request(simulation_id: Uuid) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!("/api/v1/simulations/{simulation_id}"))
        .body(Body::empty())
        .expect("build get run request")
}
