//! Building list endpoint backed by PostGIS.

use crate::{
    db,
    error::{AppError, Result},
    state::AppState,
};
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 100;
const MAX_PAGE_SIZE: u64 = 500;

#[derive(Debug, Default, Deserialize)]
pub struct BuildingListQuery {
    page: Option<String>,
    page_size: Option<String>,
    region_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BuildingListResponse {
    pub data: Vec<BuildingDto>,
    pub pagination: PaginationDto,
}

#[derive(Debug, Serialize)]
pub struct BuildingDto {
    pub id: Uuid,
    pub region_id: Uuid,
    pub external_id: String,
    pub name: Option<String>,
    pub building_type: Option<String>,
    pub geometry: Value,
}

#[derive(Debug, Serialize)]
pub struct PaginationDto {
    pub page: u64,
    pub page_size: u64,
    pub total: i64,
    pub total_pages: u64,
}

#[derive(Debug, PartialEq, Eq)]
struct ValidatedQuery {
    page: u64,
    page_size: u64,
    offset: i64,
    region_id: Option<Uuid>,
}

impl BuildingListQuery {
    fn validate(self) -> Result<ValidatedQuery> {
        let page = parse_positive(self.page.as_deref(), "page", DEFAULT_PAGE)?;
        let page_size = parse_positive(self.page_size.as_deref(), "page_size", DEFAULT_PAGE_SIZE)?;

        if page_size > MAX_PAGE_SIZE {
            return Err(AppError::BadRequest(format!(
                "page_size must not exceed {MAX_PAGE_SIZE}"
            )));
        }

        let offset = page
            .checked_sub(1)
            .and_then(|value| value.checked_mul(page_size))
            .and_then(|value| i64::try_from(value).ok())
            .ok_or_else(|| AppError::BadRequest("pagination offset is out of range".into()))?;

        let region_id = self
            .region_id
            .as_deref()
            .map(|value| {
                Uuid::parse_str(value)
                    .map_err(|_| AppError::BadRequest("region_id must be a valid UUID".into()))
            })
            .transpose()?;

        Ok(ValidatedQuery {
            page,
            page_size,
            offset,
            region_id,
        })
    }
}

fn parse_positive(value: Option<&str>, field: &str, default: u64) -> Result<u64> {
    let Some(value) = value else {
        return Ok(default);
    };

    let parsed = value
        .parse::<u64>()
        .map_err(|_| AppError::BadRequest(format!("{field} must be a positive integer")))?;

    if parsed == 0 {
        return Err(AppError::BadRequest(format!(
            "{field} must be greater than zero"
        )));
    }

    Ok(parsed)
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/buildings", get(list_buildings))
}

pub async fn list_buildings(
    State(state): State<AppState>,
    Query(query): Query<BuildingListQuery>,
) -> Result<Json<BuildingListResponse>> {
    let query = query.validate()?;
    let page_size = i64::try_from(query.page_size)
        .map_err(|_| AppError::BadRequest("page_size is out of range".into()))?;

    let (total, records) =
        db::buildings::list_buildings(&state.db, query.region_id, page_size, query.offset).await?;
    let data = records
        .into_iter()
        .map(|record| BuildingDto {
            id: record.id,
            region_id: record.region_id,
            external_id: record.external_id,
            name: record.name,
            building_type: record.building_type,
            geometry: record.geometry,
        })
        .collect::<Vec<_>>();

    let total_pages = if total == 0 {
        0
    } else {
        (total as u64).div_ceil(query.page_size)
    };

    Ok(Json(BuildingListResponse {
        data,
        pagination: PaginationDto {
            page: query.page,
            page_size: query.page_size,
            total,
            total_pages,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_defaults_are_applied() {
        let query = BuildingListQuery::default()
            .validate()
            .expect("valid defaults");
        assert_eq!(query.page, 1);
        assert_eq!(query.page_size, 100);
        assert_eq!(query.offset, 0);
        assert_eq!(query.region_id, None);
    }

    #[test]
    fn query_parses_region_uuid_and_offset() {
        let region = Uuid::parse_str("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa").unwrap();
        let query = BuildingListQuery {
            page: Some("3".into()),
            page_size: Some("25".into()),
            region_id: Some(region.to_string()),
        }
        .validate()
        .expect("valid query");

        assert_eq!(query.offset, 50);
        assert_eq!(query.region_id, Some(region));
    }

    #[test]
    fn query_rejects_zero_and_non_numeric_values() {
        for query in [
            BuildingListQuery {
                page: Some("0".into()),
                ..Default::default()
            },
            BuildingListQuery {
                page: Some("abc".into()),
                ..Default::default()
            },
            BuildingListQuery {
                page_size: Some("0".into()),
                ..Default::default()
            },
        ] {
            assert!(query.validate().is_err());
        }
    }

    #[test]
    fn query_rejects_page_size_above_limit_and_invalid_uuid() {
        let too_large = BuildingListQuery {
            page_size: Some("501".into()),
            ..Default::default()
        };
        assert!(too_large.validate().is_err());

        let invalid_uuid = BuildingListQuery {
            region_id: Some("not-a-uuid".into()),
            ..Default::default()
        };
        assert!(invalid_uuid.validate().is_err());
    }

    #[test]
    fn query_rejects_offset_overflow() {
        let query = BuildingListQuery {
            page: Some(u64::MAX.to_string()),
            page_size: Some("500".into()),
            ..Default::default()
        };
        assert!(query.validate().is_err());
    }
}
