//! Read APIs for persisted building-level spatial SIR results.
use crate::{
    error::{AppError, Result},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 100;
const MAX_PAGE_SIZE: u64 = 500;

#[derive(Debug, Default, Deserialize)]
pub struct SpatialResultsQuery {
    time_step: Option<String>,
    region_id: Option<String>,
    page: Option<String>,
    page_size: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct ValidatedQuery {
    time_step: i32,
    region_id: Option<Uuid>,
    page: u64,
    page_size: u64,
    offset: i64,
}

#[derive(Debug, Serialize)]
pub struct SpatialBuildingResult {
    pub building_id: Uuid,
    pub region_id: Uuid,
    pub population: f64,
    pub susceptible: f64,
    pub infected: f64,
    pub recovered: f64,
}

#[derive(Debug, Serialize)]
pub struct ResultsPagination {
    pub page: u64,
    pub page_size: u64,
    pub total: i64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize)]
pub struct SpatialResultsResponse {
    pub simulation_id: Uuid,
    pub time_step: i32,
    pub time: f64,
    pub data: Vec<SpatialBuildingResult>,
    pub pagination: ResultsPagination,
}

#[derive(Debug, Serialize)]
pub struct SpatialSirAggregate {
    pub simulation_id: Uuid,
    pub time_step: i32,
    pub time: f64,
    pub building_count: i64,
    pub population: f64,
    pub susceptible: f64,
    pub infected: f64,
    pub recovered: f64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/simulations/:simulation_id/results", get(list_results))
        .route("/api/v1/simulations/:simulation_id/aggregate", get(get_aggregate))
}

impl SpatialResultsQuery {
    fn validate(self) -> Result<ValidatedQuery> {
        let time_step = self.time_step.as_deref()
            .ok_or_else(|| AppError::BadRequest("time_step is required".into()))?
            .parse::<i32>()
            .map_err(|_| AppError::BadRequest("time_step must be a non-negative integer".into()))?;
        if time_step < 0 {
            return Err(AppError::BadRequest("time_step must be a non-negative integer".into()));
        }

        let region_id = self.region_id.as_deref().map(|value| {
            Uuid::parse_str(value)
                .map_err(|_| AppError::BadRequest("region_id must be a valid UUID".into()))
        }).transpose()?;
        let page = parse_positive(self.page.as_deref(), "page", DEFAULT_PAGE)?;
        let page_size = parse_positive(self.page_size.as_deref(), "page_size", DEFAULT_PAGE_SIZE)?;
        if page_size > MAX_PAGE_SIZE {
            return Err(AppError::BadRequest(format!("page_size must not exceed {MAX_PAGE_SIZE}")));
        }
        let offset = page.checked_sub(1)
            .and_then(|value| value.checked_mul(page_size))
            .and_then(|value| i64::try_from(value).ok())
            .ok_or_else(|| AppError::BadRequest("pagination offset is out of range".into()))?;

        Ok(ValidatedQuery { time_step, region_id, page, page_size, offset })
    }
}

fn parse_positive(value: Option<&str>, field: &str, default: u64) -> Result<u64> {
    let Some(value) = value else { return Ok(default); };
    let parsed = value.parse::<u64>()
        .map_err(|_| AppError::BadRequest(format!("{field} must be a positive integer")))?;
    if parsed == 0 {
        return Err(AppError::BadRequest(format!("{field} must be greater than zero")));
    }
    Ok(parsed)
}

async fn ensure_completed_run(db: &sqlx::PgPool, simulation_id: Uuid) -> Result<()> {
    let status: String = sqlx::query_scalar(
        "SELECT status FROM public.simulation_runs WHERE id = $1"
    )
    .bind(simulation_id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)?;

    if status != "completed" {
        return Err(AppError::Conflict("simulation_not_completed".into()));
    }
    Ok(())
}

async fn validate_time_step(db: &sqlx::PgPool, simulation_id: Uuid, time_step: i32) -> Result<f64> {
    sqlx::query_scalar(
        "SELECT MIN(time) FROM public.simulation_building_results WHERE simulation_id = $1 AND time_step = $2"
    )
    .bind(simulation_id)
    .bind(time_step)
    .fetch_one(db)
    .await?
    .ok_or_else(|| AppError::NotFound)
}

async fn validate_region(db: &sqlx::PgPool, region_id: Option<Uuid>) -> Result<()> {
    if let Some(region_id) = region_id {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM public.regions WHERE id = $1)"
        )
        .bind(region_id)
        .fetch_one(db)
        .await?;
        if !exists {
            return Err(AppError::NotFound);
        }
    }
    Ok(())
}

async fn list_results(
    State(state): State<AppState>,
    Path(simulation_id): Path<Uuid>,
    Query(query): Query<SpatialResultsQuery>,
) -> Result<Json<SpatialResultsResponse>> {
    let query = query.validate()?;
    ensure_completed_run(&state.db, simulation_id).await?;
    let time = validate_time_step(&state.db, simulation_id, query.time_step).await?;
    validate_region(&state.db, query.region_id).await?;

    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*)::BIGINT
           FROM public.simulation_building_results r
           JOIN public.simulation_building_states s
             ON s.simulation_id = r.simulation_id AND s.building_id = r.building_id
           JOIN public.buildings b ON b.id = r.building_id
           WHERE r.simulation_id = $1 AND r.time_step = $2
             AND ($3::uuid IS NULL OR b.region_id = $3)"#
    )
    .bind(simulation_id)
    .bind(query.time_step)
    .bind(query.region_id)
    .fetch_one(&state.db)
    .await?;

    let rows = sqlx::query(
        r#"SELECT r.building_id, b.region_id, s.population,
                  r.susceptible, r.infected, r.recovered
           FROM public.simulation_building_results r
           JOIN public.simulation_building_states s
             ON s.simulation_id = r.simulation_id AND s.building_id = r.building_id
           JOIN public.buildings b ON b.id = r.building_id
           WHERE r.simulation_id = $1 AND r.time_step = $2
             AND ($3::uuid IS NULL OR b.region_id = $3)
           ORDER BY r.building_id ASC
           LIMIT $4 OFFSET $5"#
    )
    .bind(simulation_id)
    .bind(query.time_step)
    .bind(query.region_id)
    .bind(i64::try_from(query.page_size).map_err(|_| AppError::BadRequest("page_size is out of range".into()))?)
    .bind(query.offset)
    .fetch_all(&state.db)
    .await?;

    let data = rows.into_iter().map(|row| {
        Ok(SpatialBuildingResult {
            building_id: row.try_get("building_id")?,
            region_id: row.try_get("region_id")?,
            population: row.try_get("population")?,
            susceptible: row.try_get("susceptible")?,
            infected: row.try_get("infected")?,
            recovered: row.try_get("recovered")?,
        })
    }).collect::<std::result::Result<Vec<_>, sqlx::Error>>()?;

    let total_pages = if total == 0 { 0 } else { (total as u64).div_ceil(query.page_size) };
    Ok(Json(SpatialResultsResponse {
        simulation_id,
        time_step: query.time_step,
        time,
        data,
        pagination: ResultsPagination {
            page: query.page,
            page_size: query.page_size,
            total,
            total_pages,
        },
    }))
}

async fn get_aggregate(
    State(state): State<AppState>,
    Path(simulation_id): Path<Uuid>,
    Query(query): Query<SpatialResultsQuery>,
) -> Result<Json<SpatialSirAggregate>> {
    let query = query.validate()?;
    ensure_completed_run(&state.db, simulation_id).await?;
    let time = validate_time_step(&state.db, simulation_id, query.time_step).await?;
    validate_region(&state.db, query.region_id).await?;

    let row = sqlx::query(
        r#"SELECT COUNT(*)::BIGINT AS building_count,
                  COALESCE(SUM(s.population), 0)::DOUBLE PRECISION AS population,
                  COALESCE(SUM(r.susceptible), 0)::DOUBLE PRECISION AS susceptible,
                  COALESCE(SUM(r.infected), 0)::DOUBLE PRECISION AS infected,
                  COALESCE(SUM(r.recovered), 0)::DOUBLE PRECISION AS recovered
           FROM public.simulation_building_results r
           JOIN public.simulation_building_states s
             ON s.simulation_id = r.simulation_id AND s.building_id = r.building_id
           JOIN public.buildings b ON b.id = r.building_id
           WHERE r.simulation_id = $1 AND r.time_step = $2
             AND ($3::uuid IS NULL OR b.region_id = $3)"#
    )
    .bind(simulation_id)
    .bind(query.time_step)
    .bind(query.region_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(SpatialSirAggregate {
        simulation_id,
        time_step: query.time_step,
        time,
        building_count: row.try_get("building_count")?,
        population: row.try_get("population")?,
        susceptible: row.try_get("susceptible")?,
        infected: row.try_get("infected")?,
        recovered: row.try_get("recovered")?,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_requires_non_negative_time_step_and_applies_pagination_defaults() {
        let query = SpatialResultsQuery { time_step: Some("0".into()), ..Default::default() }
            .validate().unwrap();
        assert_eq!(query.time_step, 0);
        assert_eq!(query.page, 1);
        assert_eq!(query.page_size, 100);
        assert_eq!(query.offset, 0);
    }

    #[test]
    fn query_rejects_missing_negative_or_invalid_time_step() {
        for time_step in [None, Some("-1".into()), Some("abc".into())] {
            assert!(SpatialResultsQuery { time_step, ..Default::default() }.validate().is_err());
        }
    }

    #[test]
    fn query_rejects_invalid_region_and_pagination() {
        assert!(SpatialResultsQuery {
            time_step: Some("1".into()), region_id: Some("invalid".into()), ..Default::default()
        }.validate().is_err());
        assert!(SpatialResultsQuery {
            time_step: Some("1".into()), page_size: Some("501".into()), ..Default::default()
        }.validate().is_err());
        assert!(SpatialResultsQuery {
            time_step: Some("1".into()), page: Some("0".into()), ..Default::default()
        }.validate().is_err());
    }

    #[test]
    fn query_computes_stable_page_offset() {
        let query = SpatialResultsQuery {
            time_step: Some("2".into()), page: Some("3".into()), page_size: Some("25".into()),
            ..Default::default()
        }.validate().unwrap();
        assert_eq!(query.offset, 50);
    }
}
