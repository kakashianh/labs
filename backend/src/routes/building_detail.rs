//! Building detail endpoint.
use crate::{db, error::{AppError, Result}, state::AppState};
use axum::{extract::{Path, State}, routing::get, Json, Router};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct BuildingDetail {
    pub id: Uuid, pub region_id: Uuid, pub external_id: String, pub name: Option<String>,
    pub building_type: Option<String>, pub geometry: Value,
}
#[derive(Debug, Serialize)]
pub struct DataResponse<T> { pub data: T }

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/buildings/:building_id", get(get_building))
}
pub async fn get_building(State(state): State<AppState>, Path(building_id): Path<String>)
    -> Result<Json<DataResponse<BuildingDetail>>> {
    let id = Uuid::parse_str(&building_id)
        .map_err(|_| AppError::BadRequest("building_id must be a valid UUID".into()))?;
    let r = db::regions::get_building_detail(&state.db, id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(DataResponse { data: BuildingDetail {
        id:r.id, region_id:r.region_id, external_id:r.external_id, name:r.name,
        building_type:r.building_type, geometry:r.geometry,
    }}))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn building_detail_keeps_nullable_fields_and_geojson() {
        let dto=BuildingDetail { id:Uuid::nil(), region_id:Uuid::nil(), external_id:"way/1".into(),
            name:None, building_type:None, geometry:serde_json::json!({"type":"MultiPolygon","coordinates":[]}) };
        let value=serde_json::to_value(dto).unwrap();
        assert!(value["name"].is_null());
        assert!(value["building_type"].is_null());
        assert_eq!(value["geometry"]["type"], "MultiPolygon");
    }
}
