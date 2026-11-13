//! Region list and detail endpoints.

use crate::{
    db,
    error::{AppError, Result},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct RegionListItem {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegionDetail {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub boundary_method: String,
    pub source_reference: Option<String>,
    pub geometry: Value,
}

#[derive(Debug, Serialize)]
pub struct DataResponse<T> {
    pub data: T,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/regions", get(list_regions))
        .route("/api/v1/regions/:region_id", get(get_region))
}

pub async fn list_regions(
    State(state): State<AppState>,
) -> Result<Json<DataResponse<Vec<RegionListItem>>>> {
    let records = db::regions::list_regions(&state.db).await?;
    let data = records
        .into_iter()
        .map(|r| RegionListItem {
            id: r.id,
            code: r.code,
            name: r.name,
            description: r.description,
        })
        .collect();
    Ok(Json(DataResponse { data }))
}

pub async fn get_region(
    State(state): State<AppState>,
    Path(region_id): Path<String>,
) -> Result<Json<DataResponse<RegionDetail>>> {
    let id = Uuid::parse_str(&region_id)
        .map_err(|_| AppError::BadRequest("region_id must be a valid UUID".into()))?;
    let r = db::regions::get_region(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(DataResponse {
        data: RegionDetail {
            id: r.id,
            code: r.code,
            name: r.name,
            description: r.description,
            boundary_method: r.boundary_method,
            source_reference: r.source_reference,
            geometry: r.geometry,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_detail_serializes_nullable_fields_and_geojson() {
        let dto = RegionDetail {
            id: Uuid::nil(),
            code: "test".into(),
            name: "Test".into(),
            description: None,
            boundary_method: "fixture".into(),
            source_reference: None,
            geometry: serde_json::json!({"type":"MultiPolygon","coordinates":[]}),
        };
        let value = serde_json::to_value(dto).unwrap();
        assert!(value["description"].is_null());
        assert!(value["source_reference"].is_null());
        assert_eq!(value["geometry"]["type"], "MultiPolygon");
    }
}
