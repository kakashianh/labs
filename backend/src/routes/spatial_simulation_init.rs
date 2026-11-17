//! Persisted initialization of SIR state for an explicit set of buildings.
use crate::{error::{AppError, Result}, state::AppState};
use axum::{extract::rejection::JsonRejection, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InitializeSpatialSirRequest {
    pub request_id: Uuid,
    pub buildings: Vec<BuildingInitialInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BuildingInitialInput {
    pub building_id: Uuid,
    pub population: Option<f64>,
    pub initial_infected: Option<f64>,
    #[serde(default)]
    pub initial_recovered: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct InitializedBuilding {
    pub building_id: Uuid,
    pub population: f64,
    pub initial_s: f64,
    pub initial_i: f64,
    pub initial_r: f64,
}

#[derive(Debug, Serialize)]
pub struct ExcludedBuilding {
    pub building_id: Uuid,
    pub reason_code: String,
}

#[derive(Debug, Serialize)]
pub struct InitializeSpatialSirResponse {
    pub simulation_id: Uuid,
    pub status: String,
    pub population_source: String,
    pub idempotent_replay: bool,
    pub eligible: Vec<InitializedBuilding>,
    pub excluded: Vec<ExcludedBuilding>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/simulations/initialize", post(initialize))
}

async fn initialize(
    state: axum::extract::State<AppState>,
    payload: std::result::Result<Json<InitializeSpatialSirRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<InitializeSpatialSirResponse>)> {
    let Json(request) = payload
        .map_err(|_| AppError::BadRequest("invalid JSON request body".into()))?;
    validate_request(&request)?;

    let request_json = serde_json::to_value(&request)
        .map_err(|error| AppError::Internal(error.into()))?;
    let ids: Vec<Uuid> = request.buildings.iter().map(|item| item.building_id).collect();

    let mut tx = state.db.begin().await?;
    sqlx::query("SET LOCAL search_path TO public, extensions")
        .execute(&mut *tx).await?;

    let found: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM public.buildings WHERE id = ANY($1::uuid[]) ORDER BY id"
    ).bind(&ids).fetch_all(&mut *tx).await?;
    if found.len() != ids.len() {
        return Err(AppError::BadRequest("one or more building_id values do not exist".into()));
    }

    // The unique request_id serializes concurrent retries. A conflicting insert waits
    // for the first transaction, then the persisted payload determines replay vs conflict.
    let inserted = sqlx::query(
        "INSERT INTO public.simulation_runs (request_id, request_payload) VALUES ($1, $2) ON CONFLICT (request_id) DO NOTHING"
    ).bind(request.request_id).bind(&request_json).execute(&mut *tx).await?;

    let (simulation_id, replay) = if inserted.rows_affected() == 1 {
        let id: Uuid = sqlx::query_scalar(
            "SELECT id FROM public.simulation_runs WHERE request_id = $1"
        ).bind(request.request_id).fetch_one(&mut *tx).await?;

        for item in &request.buildings {
            match item.population {
                None => {
                    sqlx::query(
                        "INSERT INTO public.simulation_building_exclusions (simulation_id, building_id, reason_code) VALUES ($1, $2, 'missing_population')"
                    ).bind(id).bind(item.building_id).execute(&mut *tx).await?;
                }
                Some(population) => {
                    let infected = item.initial_infected.expect("validated for eligible building");
                    let recovered = item.initial_recovered.unwrap_or(0.0);
                    let susceptible = population - infected - recovered;
                    sqlx::query(
                        "INSERT INTO public.simulation_building_states (simulation_id, building_id, population, initial_s, initial_i, initial_r) VALUES ($1, $2, $3, $4, $5, $6)"
                    ).bind(id).bind(item.building_id).bind(population).bind(susceptible).bind(infected).bind(recovered)
                    .execute(&mut *tx).await?;
                }
            }
        }
        (id, false)
    } else {
        let row = sqlx::query(
            "SELECT id, request_payload FROM public.simulation_runs WHERE request_id = $1 FOR UPDATE"
        ).bind(request.request_id).fetch_one(&mut *tx).await?;
        let existing_payload: serde_json::Value = row.try_get("request_payload")?;
        if existing_payload != request_json {
            return Err(AppError::Conflict("request_id was already used with a different payload".into()));
        }
        (row.try_get("id")?, true)
    };

    let state_rows = sqlx::query(
        "SELECT building_id, population, initial_s, initial_i, initial_r FROM public.simulation_building_states WHERE simulation_id = $1 ORDER BY building_id"
    ).bind(simulation_id).fetch_all(&mut *tx).await?;
    let eligible = state_rows.into_iter().map(|row| Ok(InitializedBuilding {
        building_id: row.try_get("building_id")?,
        population: row.try_get("population")?,
        initial_s: row.try_get("initial_s")?,
        initial_i: row.try_get("initial_i")?,
        initial_r: row.try_get("initial_r")?,
    })).collect::<std::result::Result<Vec<_>, sqlx::Error>>()?;

    let exclusion_rows = sqlx::query(
        "SELECT building_id, reason_code FROM public.simulation_building_exclusions WHERE simulation_id = $1 ORDER BY building_id"
    ).bind(simulation_id).fetch_all(&mut *tx).await?;
    let excluded = exclusion_rows.into_iter().map(|row| Ok(ExcludedBuilding {
        building_id: row.try_get("building_id")?,
        reason_code: row.try_get("reason_code")?,
    })).collect::<std::result::Result<Vec<_>, sqlx::Error>>()?;

    if eligible.is_empty() {
        return Err(AppError::BadRequest("no eligible buildings: population is missing for every building".into()));
    }

    tx.commit().await?;
    let status = if replay { StatusCode::OK } else { StatusCode::CREATED };
    Ok((status, Json(InitializeSpatialSirResponse {
        simulation_id,
        status: "initialized".into(),
        population_source: "manual".into(),
        idempotent_replay: replay,
        eligible,
        excluded,
    })))
}

fn validate_request(request: &InitializeSpatialSirRequest) -> Result<()> {
    if request.buildings.is_empty() {
        return Err(AppError::BadRequest("buildings must contain at least one item".into()));
    }
    let mut seen = HashSet::with_capacity(request.buildings.len());
    for item in &request.buildings {
        if !seen.insert(item.building_id) {
            return Err(AppError::BadRequest("duplicate building_id values are not allowed".into()));
        }
        for (name, value) in [
            ("population", item.population),
            ("initial_infected", item.initial_infected),
            ("initial_recovered", item.initial_recovered),
        ] {
            if let Some(value) = value {
                if !value.is_finite() || value < 0.0 || (name == "population" && value <= 0.0) {
                    return Err(AppError::BadRequest(format!("{name} must be finite and non-negative; population must be greater than zero")));
                }
            }
        }
        if item.population.is_some() {
            let infected = item.initial_infected.ok_or_else(|| AppError::BadRequest(
                "initial_infected is required when population is provided".into()
            ))?;
            let recovered = item.initial_recovered.unwrap_or(0.0);
            let population = item.population.expect("checked above");
            if infected + recovered > population || !(infected + recovered).is_finite() {
                return Err(AppError::BadRequest("initial_infected + initial_recovered must not exceed population".into()));
            }
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    fn request(items: Vec<BuildingInitialInput>) -> InitializeSpatialSirRequest {
        InitializeSpatialSirRequest {
            request_id: Uuid::nil(),
            buildings: items,
        }
    }

    fn building(population: Option<f64>, infected: Option<f64>, recovered: Option<f64>) -> BuildingInitialInput {
        BuildingInitialInput {
            building_id: Uuid::new_v4(),
            population,
            initial_infected: infected,
            initial_recovered: recovered,
        }
    }

    #[test]
    fn accepts_manual_population_and_explicit_zero_infected() {
        assert!(validate_request(&request(vec![building(Some(100.0), Some(0.0), None)])).is_ok());
    }

    #[test]
    fn missing_population_is_eligible_for_exclusion_without_infected_seed() {
        assert!(validate_request(&request(vec![building(None, None, None)])).is_ok());
    }

    #[test]
    fn requires_initial_infected_when_population_is_present() {
        assert!(validate_request(&request(vec![building(Some(100.0), None, None)])).is_err());
    }

    #[test]
    fn rejects_invalid_population_and_over_population() {
        assert!(validate_request(&request(vec![building(Some(0.0), Some(0.0), None)])).is_err());
        assert!(validate_request(&request(vec![building(Some(100.0), Some(80.0), Some(21.0))])).is_err());
        assert!(validate_request(&request(vec![building(Some(f64::INFINITY), Some(0.0), None)])).is_err());
    }

    #[test]
    fn rejects_duplicate_building_ids() {
        let first = building(Some(100.0), Some(1.0), None);
        let mut second = building(Some(200.0), Some(0.0), None);
        second.building_id = first.building_id;
        assert!(validate_request(&request(vec![first, second])).is_err());
    }

    #[test]
    fn rejects_negative_seed_values() {
        assert!(validate_request(&request(vec![building(Some(100.0), Some(-1.0), None)])).is_err());
        assert!(validate_request(&request(vec![building(Some(100.0), Some(1.0), Some(-1.0))])).is_err());
    }

    #[test]
    fn rejects_empty_building_selection() {
        assert!(validate_request(&request(vec![])).is_err());
    }
}
