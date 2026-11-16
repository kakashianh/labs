//! Footprint-to-footprint neighbor queries backed by PostGIS.

use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug)]
pub struct BuildingNeighborRecord {
    pub building_id: Uuid,
    pub distance_m: f64,
}

#[derive(Debug)]
pub struct BuildingAdjacencyRecord {
    pub building_a_id: Uuid,
    pub building_b_id: Uuid,
    pub distance_m: f64,
}

#[derive(Debug)]
pub struct NeighborSearchResult {
    pub neighbors: Vec<BuildingNeighborRecord>,
    pub excluded_invalid_geometry_count: i64,
}

#[derive(Debug)]
pub struct BuildingAdjacencyResult {
    pub edges: Vec<BuildingAdjacencyRecord>,
    pub excluded_invalid_geometry_count: i64,
    pub missing_building_count: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum NeighborSearchError {
    #[error("Building not found")]
    NotFound,
    #[error("Building footprint is empty, invalid, or has an unexpected SRID")]
    InvalidCenterGeometry,
    #[error("Database query failed: {0}")]
    Database(#[from] sqlx::Error),
}

/// Find neighbors using the minimum distance between polygon footprints.
/// Geography casts make all distance values meters for the EPSG:4326 source data.
pub async fn find_building_neighbors(
    pool: &PgPool,
    center_id: Uuid,
    radius_m: f64,
    limit: i64,
) -> Result<NeighborSearchResult, NeighborSearchError> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL search_path TO public, extensions")
        .execute(&mut *tx)
        .await?;

    let center_valid: Option<bool> = sqlx::query_scalar(
        r#"SELECT ST_SRID(footprint) = 4326
            AND NOT ST_IsEmpty(footprint)
            AND ST_IsValid(footprint)
           FROM public.buildings WHERE id = $1"#,
    )
    .bind(center_id)
    .fetch_optional(&mut *tx)
    .await?;

    match center_valid {
        None => return Err(NeighborSearchError::NotFound),
        Some(false) => return Err(NeighborSearchError::InvalidCenterGeometry),
        Some(true) => {}
    }

    let excluded_invalid_geometry_count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM public.buildings
           WHERE ST_SRID(footprint) <> 4326
              OR ST_IsEmpty(footprint)
              OR NOT ST_IsValid(footprint)"#,
    )
    .fetch_one(&mut *tx)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT neighbor.id AS building_id,
               ST_Distance(center.footprint::geography,
                           neighbor.footprint::geography) AS distance_m
        FROM public.buildings AS center
        JOIN LATERAL (
            SELECT candidate.id, candidate.footprint
            FROM public.buildings AS candidate
            WHERE candidate.id <> center.id
              AND ST_SRID(candidate.footprint) = 4326
              AND NOT ST_IsEmpty(candidate.footprint)
              AND ST_IsValid(candidate.footprint)
              AND ST_DWithin(center.footprint::geography,
                             candidate.footprint::geography, $2)
        ) AS neighbor ON TRUE
        WHERE center.id = $1
        ORDER BY distance_m ASC, neighbor.id ASC
        LIMIT $3
        "#,
    )
    .bind(center_id)
    .bind(radius_m)
    .bind(limit)
    .fetch_all(&mut *tx)
    .await?;

    let neighbors = rows
        .into_iter()
        .map(|row| {
            Ok(BuildingNeighborRecord {
                building_id: row.try_get("building_id")?,
                distance_m: row.try_get("distance_m")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    tx.commit().await?;
    Ok(NeighborSearchResult {
        neighbors,
        excluded_invalid_geometry_count,
    })
}

/// Build an undirected adjacency edge set for a simulation's selected buildings.
/// Each pair is returned once (building_a_id < building_b_id).
pub async fn build_building_adjacency(
    pool: &PgPool,
    building_ids: &[Uuid],
    radius_m: f64,
) -> Result<BuildingAdjacencyResult, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL search_path TO public, extensions")
        .execute(&mut *tx)
        .await?;

    let diagnostics = sqlx::query(
        r#"
        WITH requested AS (
            SELECT DISTINCT unnest($1::uuid[]) AS id
        )
        SELECT
            COUNT(*) FILTER (WHERE b.id IS NULL) AS missing_building_count,
            COUNT(*) FILTER (
                WHERE b.id IS NOT NULL
                  AND (
                      ST_SRID(b.footprint) <> 4326
                      OR ST_IsEmpty(b.footprint)
                      OR NOT ST_IsValid(b.footprint)
                  )
            ) AS excluded_invalid_geometry_count
        FROM requested r
        LEFT JOIN public.buildings b ON b.id = r.id
        "#,
    )
    .bind(building_ids)
    .fetch_one(&mut *tx)
    .await?;
    let missing_building_count: i64 = diagnostics.try_get("missing_building_count")?;
    let excluded_invalid_geometry_count: i64 =
        diagnostics.try_get("excluded_invalid_geometry_count")?;

    let rows = sqlx::query(
        r#"
        WITH selected_buildings AS MATERIALIZED (
            SELECT id, footprint
            FROM public.buildings
            WHERE id = ANY($1::uuid[])
              AND ST_SRID(footprint) = 4326
              AND NOT ST_IsEmpty(footprint)
              AND ST_IsValid(footprint)
        )
        SELECT a.id AS building_a_id,
               b.id AS building_b_id,
               ST_Distance(a.footprint::geography,
                           b.footprint::geography) AS distance_m
        FROM selected_buildings AS a
        JOIN LATERAL (
            SELECT candidate.id, candidate.footprint
            FROM public.buildings AS candidate
            WHERE candidate.id > a.id
              AND ST_SRID(candidate.footprint) = 4326
              AND NOT ST_IsEmpty(candidate.footprint)
              AND ST_IsValid(candidate.footprint)
              AND ST_DWithin(a.footprint::geography,
                             candidate.footprint::geography, $2)
        ) AS b ON TRUE
        ORDER BY a.id ASC, b.id ASC
        "#,
    )
    .bind(building_ids)
    .bind(radius_m)
    .fetch_all(&mut *tx)
    .await?;

    let edges = rows
        .into_iter()
        .map(|row| {
            Ok(BuildingAdjacencyRecord {
                building_a_id: row.try_get("building_a_id")?,
                building_b_id: row.try_get("building_b_id")?,
                distance_m: row.try_get("distance_m")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    tx.commit().await?;
    Ok(BuildingAdjacencyResult {
        edges,
        excluded_invalid_geometry_count,
        missing_building_count,
    })
}
