//! Building queries for PostgreSQL/PostGIS.

use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug)]
pub struct BuildingRecord {
    pub id: Uuid,
    pub region_id: Uuid,
    pub external_id: String,
    pub name: Option<String>,
    pub building_type: Option<String>,
    pub geometry: Value,
}

/// Return the total number of matching buildings and one stable page.
///
/// PostGIS may live in public (local Compose) or extensions (Supabase), so
/// spatial functions are resolved with both schemas in the transaction-local
/// search path.
pub async fn list_buildings(
    pool: &PgPool,
    region_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> Result<(i64, Vec<BuildingRecord>), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL search_path TO public, extensions")
        .execute(&mut *tx)
        .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.buildings b WHERE ($1::uuid IS NULL OR b.region_id = $1)",
    )
    .bind(region_id)
    .fetch_one(&mut *tx)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT b.id, b.region_id, b.external_id, b.name, b.building_type,
               ST_AsGeoJSON(b.footprint)::jsonb AS geometry
        FROM public.buildings AS b
        WHERE ($1::uuid IS NULL OR b.region_id = $1)
        ORDER BY b.id ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(region_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut *tx)
    .await?;

    let records = rows
        .into_iter()
        .map(|row| {
            Ok(BuildingRecord {
                id: row.try_get("id")?,
                region_id: row.try_get("region_id")?,
                external_id: row.try_get("external_id")?,
                name: row.try_get("name")?,
                building_type: row.try_get("building_type")?,
                geometry: row.try_get("geometry")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    tx.commit().await?;
    Ok((total, records))
}
