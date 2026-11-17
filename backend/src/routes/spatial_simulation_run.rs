//! Executes and atomically persists a spatial building-level SIR run.
use crate::{
    db::building_neighbors::{self, BuildingAdjacencyError},
    error::{AppError, Result},
    simulation::spatial_sir::{
        simulate_spatial_sir, BuildingSirState, SpatialSirConfig, SpatialSirError, SpatialSirPoint,
        MAX_BUILDINGS,
    },
    state::AppState,
};
use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder, Row};
use uuid::Uuid;

const INSERT_BATCH_SIZE: usize = 500;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct RunSpatialSirRequest {
    pub radius_m: f64,
    pub beta: f64,
    pub gamma: f64,
    pub dt: f64,
    pub steps: usize,
}

#[derive(Debug, Serialize)]
pub struct RunSpatialSirResponse {
    pub simulation_id: Uuid,
    pub status: &'static str,
    pub building_count: usize,
    pub point_count: usize,
    pub time_step_count: usize,
    pub excluded_invalid_geometry_count: i64,
    pub idempotent_replay: bool,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/simulations/:simulation_id/run",
            post(run_spatial_sir),
        )
        .route(
            "/api/v1/simulations/:simulation_id",
            get(get_spatial_sir_run),
        )
}

#[derive(Debug, Serialize)]
pub struct SpatialSirRunSummaryResponse {
    pub simulation_id: Uuid,
    pub status: String,
    pub population_source: String,
    pub eligible_building_count: i64,
    pub excluded_building_count: i64,
    pub result_point_count: i64,
    pub time_step_count: i64,
    pub completed_at: Option<String>,
}

pub async fn get_spatial_sir_run(
    State(state): State<AppState>,
    Path(simulation_id): Path<Uuid>,
) -> Result<Json<SpatialSirRunSummaryResponse>> {
    let row = sqlx::query(
        r#"SELECT r.id, r.status, r.population_source, TO_CHAR(r.completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS completed_at,
                  (SELECT COUNT(*) FROM public.simulation_building_states s
                   WHERE s.simulation_id = r.id) AS eligible_building_count,
                  (SELECT COUNT(*) FROM public.simulation_building_exclusions e
                   WHERE e.simulation_id = r.id) AS excluded_building_count,
                  (SELECT COUNT(*) FROM public.simulation_building_results p
                   WHERE p.simulation_id = r.id) AS result_point_count,
                  (SELECT COUNT(DISTINCT time_step) FROM public.simulation_building_results p
                   WHERE p.simulation_id = r.id) AS time_step_count
           FROM public.simulation_runs r WHERE r.id = $1"#,
    )
    .bind(simulation_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(SpatialSirRunSummaryResponse {
        simulation_id: row.try_get("id")?,
        status: row.try_get("status")?,
        population_source: row.try_get("population_source")?,
        eligible_building_count: row.try_get("eligible_building_count")?,
        excluded_building_count: row.try_get("excluded_building_count")?,
        result_point_count: row.try_get("result_point_count")?,
        time_step_count: row.try_get("time_step_count")?,
        completed_at: row.try_get("completed_at")?,
    }))
}

pub async fn run_spatial_sir(
    State(state): State<AppState>,
    Path(simulation_id): Path<Uuid>,
    payload: std::result::Result<Json<RunSpatialSirRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<RunSpatialSirResponse>)> {
    let Json(request) =
        payload.map_err(|_| AppError::BadRequest("invalid JSON request body".into()))?;
    let config = SpatialSirConfig {
        radius_m: request.radius_m,
        beta: request.beta,
        gamma: request.gamma,
        dt: request.dt,
        steps: request.steps,
    };

    let selected_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.simulation_building_states WHERE simulation_id = $1",
    )
    .bind(simulation_id)
    .fetch_one(&state.db)
    .await?;
    if selected_count > MAX_BUILDINGS as i64 {
        return Err(AppError::BadRequest(format!(
            "simulation supports at most {MAX_BUILDINGS} eligible buildings"
        )));
    }
    let rows = sqlx::query(
        "SELECT building_id, population, initial_s, initial_i, initial_r FROM public.simulation_building_states WHERE simulation_id = $1 ORDER BY building_id",
    )
    .bind(simulation_id)
    .fetch_all(&state.db)
    .await?;
    if rows.is_empty() {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.simulation_runs WHERE id = $1)")
                .bind(simulation_id)
                .fetch_one(&state.db)
                .await?;
        return if exists {
            Err(AppError::BadRequest(
                "simulation has no eligible building states".into(),
            ))
        } else {
            Err(AppError::NotFound)
        };
    }

    let buildings = rows
        .into_iter()
        .map(|row| {
            Ok(BuildingSirState {
                building_id: row.try_get("building_id")?,
                population: row.try_get("population")?,
                susceptible: row.try_get("initial_s")?,
                infected: row.try_get("initial_i")?,
                recovered: row.try_get("initial_r")?,
            })
        })
        .collect::<std::result::Result<Vec<_>, sqlx::Error>>()?;

    let building_ids: Vec<_> = buildings
        .iter()
        .map(|building| building.building_id)
        .collect();
    let adjacency =
        building_neighbors::build_building_adjacency(&state.db, &building_ids, request.radius_m)
    .await
    .map_err(map_adjacency_error)?;
    if adjacency.missing_building_count != 0 {
        return Err(AppError::Conflict(
            "simulation references buildings that no longer exist".into(),
        ));
    }
    let edges = adjacency
        .edges
        .into_iter()
        .map(|edge| crate::simulation::infection_pressure::NeighborEdge {
            source_building_id: edge.building_a_id,
            target_building_id: edge.building_b_id,
            distance_m: edge.distance_m,
        })
        .collect::<Vec<_>>();

    let result = simulate_spatial_sir(&buildings, &edges, config).map_err(map_engine_error)?;
    let config_json =
        serde_json::to_value(&request).map_err(|error| AppError::Internal(error.into()))?;

    // Lock the run after computation. Concurrent retries serialize here: the first
    // persists all points and marks completed; the next observes the same config.
    let mut tx = state.db.begin().await?;
    let run = sqlx::query(
        "SELECT status, simulation_config FROM public.simulation_runs WHERE id = $1 FOR UPDATE",
    )
    .bind(simulation_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;
    let status: String = run.try_get("status")?;
    let existing_config: Option<serde_json::Value> = run.try_get("simulation_config")?;

    if status == "completed" {
        if existing_config.as_ref() != Some(&config_json) {
            return Err(AppError::Conflict(
                "simulation_id has already completed with a different configuration".into(),
            ));
        }
        let counts = sqlx::query(
            "SELECT COUNT(DISTINCT building_id)::BIGINT AS building_count, COUNT(*)::BIGINT AS point_count, COUNT(DISTINCT time_step)::BIGINT AS time_step_count FROM public.simulation_building_results WHERE simulation_id = $1",
        )
        .bind(simulation_id)
        .fetch_one(&mut *tx)
        .await?;
        let response = RunSpatialSirResponse {
            simulation_id,
            status: "completed",
            building_count: counts.try_get::<i64, _>("building_count")? as usize,
            point_count: counts.try_get::<i64, _>("point_count")? as usize,
            time_step_count: counts.try_get::<i64, _>("time_step_count")? as usize,
            excluded_invalid_geometry_count: adjacency.excluded_invalid_geometry_count,
            idempotent_replay: true,
        };
        tx.commit().await?;
        return Ok((StatusCode::OK, Json(response)));
    }
    if status != "initialized" {
        return Err(AppError::Conflict(
            "simulation is not in an executable state".into(),
        ));
    }

    for chunk in result.points.chunks(INSERT_BATCH_SIZE) {
        let mut query = QueryBuilder::<Postgres>::new(
            "INSERT INTO public.simulation_building_results (simulation_id, building_id, time_step, time, susceptible, infected, recovered) ",
        );
        query.push_values(chunk, |mut values, point: &SpatialSirPoint| {
            values
                .push_bind(simulation_id)
                .push_bind(point.building_id)
                .push_bind(point.time_step as i32)
                .push_bind(point.time)
                .push_bind(point.susceptible)
                .push_bind(point.infected)
                .push_bind(point.recovered);
        });
        query.build().execute(&mut *tx).await?;
    }

    sqlx::query(
        "UPDATE public.simulation_runs SET status = 'completed', simulation_config = $2, completed_at = NOW() WHERE id = $1",
    )
    .bind(simulation_id)
    .bind(&config_json)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(RunSpatialSirResponse {
            simulation_id,
            status: "completed",
            building_count: result.building_count,
            point_count: result.point_count,
            time_step_count: request.steps + 1,
            excluded_invalid_geometry_count: adjacency.excluded_invalid_geometry_count,
            idempotent_replay: false,
        }),
    ))
}

fn map_adjacency_error(error: BuildingAdjacencyError) -> AppError {
    match error {
        BuildingAdjacencyError::InvalidRadius => AppError::BadRequest(
            "radius_m must be finite, greater than zero, and no greater than 5000".into(),
        ),
        BuildingAdjacencyError::Database(error) => AppError::Database(error),
    }
}

fn map_engine_error(error: SpatialSirError) -> AppError {
    match error {
        SpatialSirError::NumericalInstability
        | SpatialSirError::InfectionPressure(
            crate::simulation::infection_pressure::InfectionPressureError::NumericalInstability,
        ) => AppError::Internal(anyhow::anyhow!("spatial SIR numerical instability")),
        validation_error => AppError::BadRequest(validation_error.to_string()),
    }
}
