//! Region and building detail queries for PostgreSQL/PostGIS.
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug)]
pub struct RegionListRecord { pub id: Uuid, pub code: String, pub name: String, pub description: Option<String> }
#[derive(Debug)]
pub struct RegionDetailRecord {
    pub id: Uuid, pub code: String, pub name: String, pub description: Option<String>,
    pub boundary_method: String, pub source_reference: Option<String>, pub geometry: Value,
}
#[derive(Debug)]
pub struct BuildingDetailRecord {
    pub id: Uuid, pub region_id: Uuid, pub external_id: String, pub name: Option<String>,
    pub building_type: Option<String>, pub geometry: Value,
}

pub async fn list_regions(pool: &PgPool) -> Result<Vec<RegionListRecord>, sqlx::Error> {
    let rows = sqlx::query("SELECT id, code, name, description FROM public.regions ORDER BY code ASC")
        .fetch_all(pool).await?;
    rows.into_iter().map(|row| Ok(RegionListRecord {
        id: row.try_get("id")?, code: row.try_get("code")?, name: row.try_get("name")?,
        description: row.try_get("description")?,
    })).collect()
}

pub async fn get_region(pool: &PgPool, id: Uuid) -> Result<Option<RegionDetailRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL search_path TO public, extensions").execute(&mut *tx).await?;
    let row = sqlx::query(r#"SELECT id, code, name, description, boundary_method, source_reference,
        ST_AsGeoJSON(boundary)::jsonb AS geometry FROM public.regions WHERE id = $1"#)
        .bind(id).fetch_optional(&mut *tx).await?;
    tx.commit().await?;
    row.map(|row| Ok(RegionDetailRecord {
        id: row.try_get("id")?, code: row.try_get("code")?, name: row.try_get("name")?,
        description: row.try_get("description")?, boundary_method: row.try_get("boundary_method")?,
        source_reference: row.try_get("source_reference")?, geometry: row.try_get("geometry")?,
    })).transpose()
}

pub async fn get_building_detail(pool: &PgPool, id: Uuid) -> Result<Option<BuildingDetailRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL search_path TO public, extensions").execute(&mut *tx).await?;
    let row = sqlx::query(r#"SELECT id, region_id, external_id, name, building_type,
        ST_AsGeoJSON(footprint)::jsonb AS geometry FROM public.buildings WHERE id = $1"#)
        .bind(id).fetch_optional(&mut *tx).await?;
    tx.commit().await?;
    row.map(|row| Ok(BuildingDetailRecord {
        id: row.try_get("id")?, region_id: row.try_get("region_id")?,
        external_id: row.try_get("external_id")?, name: row.try_get("name")?,
        building_type: row.try_get("building_type")?, geometry: row.try_get("geometry")?,
    })).transpose()
}
