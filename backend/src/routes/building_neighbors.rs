//! Building neighbor search endpoint.

use crate::{
    db::building_neighbors::{self, NeighborSearchError},
    error::{AppError, Result},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const DEFAULT_RADIUS_M: f64 = 100.0;
const MAX_RADIUS_M: f64 = 5_000.0;
const DEFAULT_LIMIT: i64 = 100;
const MAX_LIMIT: i64 = 500;

#[derive(Debug, Default, Deserialize)]
pub struct NeighborQuery {
    radius_m: Option<String>,
    limit: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BuildingNeighborDto {
    pub building_id: Uuid,
    pub distance_m: f64,
}

#[derive(Debug, Serialize)]
pub struct BuildingNeighborsResponse {
    pub center_building_id: Uuid,
    pub radius_m: f64,
    pub distance_semantics: &'static str,
    pub excluded_invalid_geometry_count: i64,
    pub data: Vec<BuildingNeighborDto>,
}

impl NeighborQuery {
    fn validate(self) -> Result<(f64, i64)> {
        let radius_m = match self.radius_m {
            Some(value) => value.parse::<f64>().map_err(|_| {
                AppError::BadRequest("radius_m must be a finite number".into())
            })?,
            None => DEFAULT_RADIUS_M,
        };
        if !radius_m.is_finite() || radius_m <= 0.0 || radius_m > MAX_RADIUS_M {
            return Err(AppError::BadRequest(format!(
                "radius_m must be greater than 0 and no greater than {MAX_RADIUS_M}"
            )));
        }

        let limit = match self.limit {
            Some(value) => value.parse::<i64>().map_err(|_| {
                AppError::BadRequest("limit must be a positive integer".into())
            })?,
            None => DEFAULT_LIMIT,
        };
        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(AppError::BadRequest(format!(
                "limit must be between 1 and {MAX_LIMIT}"
            )));
        }
        Ok((radius_m, limit))
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/v1/buildings/:building_id/neighbors",
        get(get_building_neighbors),
    )
}

pub async fn get_building_neighbors(
    State(state): State<AppState>,
    Path(building_id): Path<String>,
    Query(query): Query<NeighborQuery>,
) -> Result<Json<BuildingNeighborsResponse>> {
    let center_id = Uuid::parse_str(&building_id)
        .map_err(|_| AppError::BadRequest("building_id must be a valid UUID".into()))?;
    let (radius_m, limit) = query.validate()?;

    let result = building_neighbors::find_building_neighbors(
        &state.db, center_id, radius_m, limit,
    )
    .await
    .map_err(|error| match error {
        NeighborSearchError::NotFound => AppError::NotFound,
        NeighborSearchError::InvalidCenterGeometry => AppError::BadRequest(
            "center building footprint must be valid, non-empty EPSG:4326 geometry".into(),
        ),
        NeighborSearchError::Database(error) => AppError::Database(error),
    })?;

    Ok(Json(BuildingNeighborsResponse {
        center_building_id: center_id,
        radius_m,
        distance_semantics: "footprint_to_footprint",
        excluded_invalid_geometry_count: result.excluded_invalid_geometry_count,
        data: result
            .neighbors
            .into_iter()
            .map(|neighbor| BuildingNeighborDto {
                building_id: neighbor.building_id,
                distance_m: neighbor.distance_m,
            })
            .collect(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_applied() {
        assert_eq!(
            NeighborQuery::default().validate().unwrap(),
            (DEFAULT_RADIUS_M, DEFAULT_LIMIT)
        );
    }

    #[test]
    fn invalid_radius_and_limit_are_rejected() {
        for query in [
            NeighborQuery { radius_m: Some("NaN".into()), ..Default::default() },
            NeighborQuery { radius_m: Some("0".into()), ..Default::default() },
            NeighborQuery { radius_m: Some("5000.1".into()), ..Default::default() },
            NeighborQuery { limit: Some("0".into()), ..Default::default() },
            NeighborQuery { limit: Some("501".into()), ..Default::default() },
            NeighborQuery { limit: Some("many".into()), ..Default::default() },
        ] {
            assert!(query.validate().is_err());
        }
    }
}
