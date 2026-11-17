//! APIs for per-building risk classification and regional summaries.
use crate::{
    error::{AppError, Result},
    simulation::risk::{summarize_region, BuildingRisk, BuildingRiskInput, RiskThresholds},
    state::AppState,
};
use axum::{extract::{Path, Query, State}, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

const DEFAULT_PAGE_SIZE: u64 = 100;
const MAX_PAGE_SIZE: u64 = 500;

#[derive(Debug, Default, Deserialize)]
pub struct RiskQuery {
    time_step: Option<String>,
    region_id: Option<String>,
    page: Option<String>,
    page_size: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RiskThresholdRequest {
    risk_low_max: Option<f64>,
    risk_medium_max: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RiskConfigurationResponse {
    simulation_id: Uuid,
    thresholds: RiskThresholds,
}

#[derive(Debug, Serialize)]
pub struct RiskResultsResponse {
    simulation_id: Uuid,
    time_step: i32,
    thresholds: RiskThresholds,
    data: Vec<BuildingRisk>,
    pagination: RiskPagination,
}

#[derive(Debug, Serialize)]
pub struct RiskPagination {
    page: u64,
    page_size: u64,
    total: u64,
    total_pages: u64,
}

#[derive(Debug, Serialize)]
pub struct RiskAggregateResponse {
    simulation_id: Uuid,
    time_step: i32,
    thresholds: RiskThresholds,
    data: Vec<crate::simulation::risk::RegionRiskSummary>,
}

struct ValidatedRiskQuery {
    time_step: i32,
    region_id: Option<Uuid>,
    page: u64,
    page_size: u64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/simulations/:simulation_id/risk-classification", post(set_thresholds))
        .route("/api/v1/simulations/:simulation_id/risk-results", get(list_risk_results))
        .route("/api/v1/simulations/:simulation_id/risk-aggregate", get(get_risk_aggregate))
}

fn validate_query(query: RiskQuery) -> Result<ValidatedRiskQuery> {
    let time_step = query.time_step.as_deref().ok_or_else(|| AppError::BadRequest("time_step is required".into()))?
        .parse::<i32>().map_err(|_| AppError::BadRequest("time_step must be a non-negative integer".into()))?;
    if time_step < 0 { return Err(AppError::BadRequest("time_step must be a non-negative integer".into())); }
    let region_id = query.region_id.as_deref().map(|v| Uuid::parse_str(v)
        .map_err(|_| AppError::BadRequest("region_id must be a valid UUID".into()))).transpose()?;
    let page = parse_positive(query.page.as_deref(), "page", 1)?;
    let page_size = parse_positive(query.page_size.as_deref(), "page_size", DEFAULT_PAGE_SIZE)?;
    if page_size > MAX_PAGE_SIZE { return Err(AppError::BadRequest(format!("page_size must not exceed {MAX_PAGE_SIZE}"))); }
    Ok(ValidatedRiskQuery { time_step, region_id, page, page_size })
}

fn parse_positive(value: Option<&str>, field: &str, default: u64) -> Result<u64> {
    let Some(value) = value else { return Ok(default); };
    let parsed = value.parse::<u64>().map_err(|_| AppError::BadRequest(format!("{field} must be a positive integer")))?;
    if parsed == 0 { return Err(AppError::BadRequest(format!("{field} must be greater than zero"))); }
    Ok(parsed)
}

async fn completed_run_config(db: &sqlx::PgPool, simulation_id: Uuid) -> Result<serde_json::Value> {
    let row = sqlx::query("SELECT status, simulation_config FROM public.simulation_runs WHERE id = $1")
        .bind(simulation_id).fetch_optional(db).await?.ok_or(AppError::NotFound)?;
    let status: String = row.try_get("status")?;
    if status != "completed" { return Err(AppError::Conflict("simulation_not_completed".into())); }
    Ok(row.try_get::<Option<serde_json::Value>, _>("simulation_config")?.unwrap_or_else(|| serde_json::json!({})))
}

fn thresholds_from_config(config: &serde_json::Value) -> Result<RiskThresholds> {
    let thresholds: RiskThresholds = match config.get("risk_classification") {
        Some(value) => serde_json::from_value(value.clone()).map_err(|e| AppError::Internal(e.into()))?,
        None => RiskThresholds::default(),
    };
    thresholds.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(thresholds)
}

async fn set_thresholds(
    State(state): State<AppState>,
    Path(simulation_id): Path<Uuid>,
    Json(request): Json<RiskThresholdRequest>,
) -> Result<Json<RiskConfigurationResponse>> {
    let mut tx = state.db.begin().await?;
    let row = sqlx::query("SELECT status, simulation_config FROM public.simulation_runs WHERE id = $1 FOR UPDATE")
        .bind(simulation_id).fetch_optional(&mut *tx).await?.ok_or(AppError::NotFound)?;
    let status: String = row.try_get("status")?;
    if status != "completed" { return Err(AppError::Conflict("simulation_not_completed".into())); }

    let existing: Option<serde_json::Value> = row.try_get("simulation_config")?;
    let mut config = existing.unwrap_or_else(|| serde_json::json!({}));
    let previous = thresholds_from_config(&config).unwrap_or_default();
    let thresholds = RiskThresholds {
        version: "risk-v1".into(),
        source: if request.risk_low_max.is_some() || request.risk_medium_max.is_some() { "request_override".into() } else { previous.source },
        risk_low_max: request.risk_low_max.unwrap_or(previous.risk_low_max),
        risk_medium_max: request.risk_medium_max.unwrap_or(previous.risk_medium_max),
    };
    thresholds.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    config["risk_classification"] = serde_json::to_value(&thresholds).map_err(|e| AppError::Internal(e.into()))?;
    sqlx::query("UPDATE public.simulation_runs SET simulation_config = $2 WHERE id = $1")
        .bind(simulation_id).bind(&config).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(RiskConfigurationResponse { simulation_id, thresholds }))
}

async fn load_building_risks(
    db: &sqlx::PgPool,
    simulation_id: Uuid,
    query: &ValidatedRiskQuery,
) -> Result<(RiskThresholds, Vec<BuildingRisk>, f64)> {
    let config = completed_run_config(db, simulation_id).await?;
    let thresholds = thresholds_from_config(&config)?;
    let time = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT MIN(time) FROM public.simulation_building_results WHERE simulation_id=$1 AND time_step=$2"
    ).bind(simulation_id).bind(query.time_step).fetch_one(db).await?.ok_or(AppError::NotFound)?;
    if let Some(region_id) = query.region_id {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.regions WHERE id=$1)")
            .bind(region_id).fetch_one(db).await?;
        if !exists { return Err(AppError::NotFound); }
    }
    let rows = sqlx::query(
        r#"SELECT r.building_id, b.region_id, s.population, r.infected
           FROM public.simulation_building_results r
           JOIN public.simulation_building_states s ON s.simulation_id=r.simulation_id AND s.building_id=r.building_id
           JOIN public.buildings b ON b.id=r.building_id
           WHERE r.simulation_id=$1 AND r.time_step=$2 AND ($3::uuid IS NULL OR b.region_id=$3)
           UNION ALL
           SELECT e.building_id, b.region_id, NULL::DOUBLE PRECISION AS population, 0::DOUBLE PRECISION AS infected
           FROM public.simulation_building_exclusions e
           JOIN public.buildings b ON b.id=e.building_id
           WHERE e.simulation_id=$1 AND e.reason_code='missing_population'
             AND ($3::uuid IS NULL OR b.region_id=$3)
           ORDER BY region_id, building_id"#
    ).bind(simulation_id).bind(query.time_step).bind(query.region_id).fetch_all(db).await?;
    let data = rows.into_iter().map(|row| {
        let input = BuildingRiskInput {
            building_id: row.try_get("building_id")?,
            region_id: row.try_get("region_id")?,
            population: row.try_get("population")?,
            infected: row.try_get("infected")?,
        };
        thresholds.classify(input).map_err(|e| AppError::BadRequest(e.to_string()))
    }).collect::<Result<Vec<_>>>()?;
    Ok((thresholds, data, time))
}

async fn list_risk_results(
    State(state): State<AppState>, Path(simulation_id): Path<Uuid>, Query(query): Query<RiskQuery>,
) -> Result<Json<RiskResultsResponse>> {
    let query = validate_query(query)?;
    let (thresholds, all, _) = load_building_risks(&state.db, simulation_id, &query).await?;
    let total = all.len() as u64;
    let offset = query.page.saturating_sub(1).saturating_mul(query.page_size);
    let data = all.into_iter().skip(offset as usize).take(query.page_size as usize).collect();
    Ok(Json(RiskResultsResponse {
        simulation_id, time_step: query.time_step, thresholds, data,
        pagination: RiskPagination { page: query.page, page_size: query.page_size, total,
            total_pages: if total == 0 { 0 } else { total.div_ceil(query.page_size) } },
    }))
}

async fn get_risk_aggregate(
    State(state): State<AppState>, Path(simulation_id): Path<Uuid>, Query(query): Query<RiskQuery>,
) -> Result<Json<RiskAggregateResponse>> {
    let query = validate_query(query)?;
    let (thresholds, items, _) = load_building_risks(&state.db, simulation_id, &query).await?;
    let region_ids: Vec<Uuid> = if let Some(id) = query.region_id { vec![id] } else {
        let mut ids: Vec<_> = items.iter().map(|item| item.region_id).collect();
        ids.sort_unstable(); ids.dedup(); ids
    };
    let data = region_ids.into_iter().map(|id| summarize_region(id, &items)).collect();
    Ok(Json(RiskAggregateResponse { simulation_id, time_step: query.time_step, thresholds, data }))
}
