//! Read API for spatial infection hotspots using connected components.
use crate::{
    error::{AppError, Result},
    simulation::risk::{BuildingRiskInput, RiskLevel, RiskThresholds},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

const DEFAULT_RADIUS_M: f64 = 100.0;
const MAX_RADIUS_M: f64 = 5_000.0;
const ALGORITHM_VERSION: &str = "hotspot-v1";

#[derive(Debug, Default, Deserialize)]
pub struct HotspotQuery {
    time_step: Option<String>,
    region_id: Option<String>,
    minimum_risk_level: Option<String>,
    radius_m: Option<String>,
    include_singletons: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ValidatedHotspotQuery {
    time_step: i32,
    region_id: Option<Uuid>,
    minimum_risk_level: RiskLevel,
    radius_m: f64,
    include_singletons: bool,
}

#[derive(Debug, Serialize)]
pub struct HotspotConfiguration {
    pub algorithm: &'static str,
    pub algorithm_version: &'static str,
    pub minimum_risk_level: RiskLevel,
    pub risk_thresholds: RiskThresholds,
    pub radius_m: f64,
    pub include_singletons: bool,
}

#[derive(Debug, Serialize)]
pub struct Hotspot {
    pub cluster_id: Uuid,
    pub simulation_id: Uuid,
    pub time_step: i32,
    pub region_id: Uuid,
    pub member_count: i64,
    pub population_total: f64,
    pub infected_total: f64,
    pub infection_ratio: f64,
    pub risk_level: RiskLevel,
    pub building_ids: Vec<Uuid>,
    pub geometry: Value,
}

#[derive(Debug, Serialize)]
pub struct HotspotsResponse {
    pub simulation_id: Uuid,
    pub time_step: i32,
    pub region_id: Option<Uuid>,
    pub configuration: HotspotConfiguration,
    pub hotspot_count: usize,
    pub hotspots: Vec<Hotspot>,
}

#[derive(Debug, Clone)]
struct CandidateBuilding {
    building_id: Uuid,
    region_id: Uuid,
    population: f64,
    infected: f64,
}

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/v1/simulations/:simulation_id/hotspots",
        get(list_hotspots),
    )
}

impl HotspotQuery {
    fn validate(self) -> Result<ValidatedHotspotQuery> {
        let time_step = self
            .time_step
            .as_deref()
            .ok_or_else(|| AppError::BadRequest("time_step is required".into()))?
            .parse::<i32>()
            .map_err(|_| AppError::BadRequest("time_step must be a non-negative integer".into()))?;
        if time_step < 0 {
            return Err(AppError::BadRequest(
                "time_step must be a non-negative integer".into(),
            ));
        }

        let region_id = self
            .region_id
            .as_deref()
            .map(|value| {
                Uuid::parse_str(value)
                    .map_err(|_| AppError::BadRequest("region_id must be a valid UUID".into()))
            })
            .transpose()?;

        let minimum_risk_level = match self.minimum_risk_level.as_deref().unwrap_or("high") {
            "medium" => RiskLevel::Medium,
            "high" => RiskLevel::High,
            _ => {
                return Err(AppError::BadRequest(
                    "minimum_risk_level must be medium or high".into(),
                ))
            }
        };

        let radius_m = match self.radius_m.as_deref() {
            Some(value) => value
                .parse::<f64>()
                .map_err(|_| AppError::BadRequest("radius_m must be a finite number".into()))?,
            None => DEFAULT_RADIUS_M,
        };
        if !radius_m.is_finite() || radius_m <= 0.0 || radius_m > MAX_RADIUS_M {
            return Err(AppError::BadRequest(format!(
                "radius_m must be greater than 0 and no greater than {MAX_RADIUS_M}"
            )));
        }

        let include_singletons = match self.include_singletons.as_deref().unwrap_or("true") {
            "true" => true,
            "false" => false,
            _ => {
                return Err(AppError::BadRequest(
                    "include_singletons must be true or false".into(),
                ))
            }
        };

        Ok(ValidatedHotspotQuery {
            time_step,
            region_id,
            minimum_risk_level,
            radius_m,
            include_singletons,
        })
    }
}

fn risk_rank(level: RiskLevel) -> u8 {
    match level {
        RiskLevel::Low => 0,
        RiskLevel::Medium => 1,
        RiskLevel::High => 2,
        RiskLevel::Unclassified => 0,
    }
}

fn thresholds_from_config(config: &Value) -> Result<RiskThresholds> {
    let thresholds = match config.get("risk_classification") {
        Some(value) => serde_json::from_value(value.clone())
            .map_err(|error| AppError::Internal(error.into()))?,
        None => RiskThresholds::default(),
    };
    thresholds
        .validate()
        .map_err(|error| AppError::BadRequest(error.to_string()))?;
    Ok(thresholds)
}

async fn load_run_configuration(
    db: &sqlx::PgPool,
    simulation_id: Uuid,
) -> Result<RiskThresholds> {
    let row = sqlx::query(
        "SELECT status, simulation_config FROM public.simulation_runs WHERE id = $1",
    )
    .bind(simulation_id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)?;

    let status: String = row.try_get("status")?;
    if status != "completed" {
        return Err(AppError::Conflict("simulation_not_completed".into()));
    }
    let config: Option<Value> = row.try_get("simulation_config")?;
    thresholds_from_config(&config.unwrap_or_else(|| serde_json::json!({})))
}

async fn validate_time_step(
    db: &sqlx::PgPool,
    simulation_id: Uuid,
    time_step: i32,
) -> Result<f64> {
    sqlx::query_scalar::<_, Option<f64>>(
        "SELECT MIN(time) FROM public.simulation_building_results WHERE simulation_id=$1 AND time_step=$2",
    )
    .bind(simulation_id)
    .bind(time_step)
    .fetch_one(db)
    .await?
    .ok_or(AppError::NotFound)
}

async fn validate_region(db: &sqlx::PgPool, region_id: Option<Uuid>) -> Result<()> {
    if let Some(region_id) = region_id {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.regions WHERE id=$1)")
                .bind(region_id)
                .fetch_one(db)
                .await?;
        if !exists {
            return Err(AppError::NotFound);
        }
    }
    Ok(())
}

async fn load_candidates(
    db: &sqlx::PgPool,
    simulation_id: Uuid,
    query: ValidatedHotspotQuery,
    thresholds: &RiskThresholds,
) -> Result<Vec<CandidateBuilding>> {
    let rows = sqlx::query(
        r#"SELECT b.id AS building_id, b.region_id, s.population, r.infected,
                  (ST_SRID(b.footprint) = 4326 AND ST_IsValid(b.footprint)
                   AND NOT ST_IsEmpty(b.footprint)) AS geometry_valid
           FROM public.simulation_building_results r
           JOIN public.simulation_building_states s
             ON s.simulation_id = r.simulation_id AND s.building_id = r.building_id
           JOIN public.buildings b ON b.id = r.building_id
           WHERE r.simulation_id = $1 AND r.time_step = $2
             AND ($3::uuid IS NULL OR b.region_id = $3)
           ORDER BY b.region_id, b.id"#,
    )
    .bind(simulation_id)
    .bind(query.time_step)
    .bind(query.region_id)
    .fetch_all(db)
    .await?;

    let mut candidates = Vec::new();
    for row in rows {
        let building_id: Uuid = row.try_get("building_id")?;
        let region_id: Uuid = row.try_get("region_id")?;
        let population: f64 = row.try_get("population")?;
        let infected: f64 = row.try_get("infected")?;
        let risk = thresholds
            .classify(BuildingRiskInput {
                building_id,
                region_id,
                population: Some(population),
                infected,
            })
            .map_err(|error| AppError::BadRequest(error.to_string()))?;

        if risk_rank(risk.risk_level) < risk_rank(query.minimum_risk_level) || infected <= 0.0 {
            continue;
        }

        let geometry_valid: bool = row.try_get("geometry_valid")?;
        if !geometry_valid {
            return Err(AppError::BadRequest(format!(
                "candidate building {building_id} must have a valid non-empty EPSG:4326 footprint"
            )));
        }

        candidates.push(CandidateBuilding {
            building_id,
            region_id,
            population,
            infected,
        });
    }
    Ok(candidates)
}

fn find_root(parent: &mut [usize], mut index: usize) -> usize {
    while parent[index] != index {
        parent[index] = parent[parent[index]];
        index = parent[index];
    }
    index
}

fn connected_components(
    candidates: &[CandidateBuilding],
    edges: &[(Uuid, Uuid)],
    include_singletons: bool,
) -> Vec<(Uuid, Vec<Uuid>)> {
    let indexes: HashMap<Uuid, usize> = candidates
        .iter()
        .enumerate()
        .map(|(index, item)| (item.building_id, index))
        .collect();
    let mut parent: Vec<usize> = (0..candidates.len()).collect();

    for (left, right) in edges {
        if let (Some(&left_index), Some(&right_index)) = (indexes.get(left), indexes.get(right)) {
            let left_root = find_root(&mut parent, left_index);
            let right_root = find_root(&mut parent, right_index);
            if left_root != right_root {
                let (root, child) = if left_root < right_root {
                    (left_root, right_root)
                } else {
                    (right_root, left_root)
                };
                parent[child] = root;
            }
        }
    }

    let mut groups: BTreeMap<Uuid, Vec<Uuid>> = BTreeMap::new();
    for (index, item) in candidates.iter().enumerate() {
        let root = find_root(&mut parent, index);
        groups
            .entry(candidates[root].building_id)
            .or_default()
            .push(item.building_id);
    }

    let mut components = Vec::new();
    for members in groups.into_values() {
        if !include_singletons && members.len() == 1 {
            continue;
        }
        let cluster_id = *members.iter().min().expect("component is not empty");
        let mut members = members;
        members.sort_unstable();
        components.push((cluster_id, members));
    }
    components.sort_by_key(|(cluster_id, _)| *cluster_id);
    components
}

async fn load_edges(
    db: &sqlx::PgPool,
    candidates: &[CandidateBuilding],
    radius_m: f64,
) -> Result<Vec<(Uuid, Uuid)>> {
    if candidates.len() < 2 {
        return Ok(Vec::new());
    }
    let ids: Vec<Uuid> = candidates.iter().map(|item| item.building_id).collect();
    let rows = sqlx::query(
        r#"SELECT a.id AS left_id, b.id AS right_id
           FROM public.buildings a
           JOIN public.buildings b
             ON a.id < b.id AND a.region_id = b.region_id
           WHERE a.id = ANY($1::uuid[]) AND b.id = ANY($1::uuid[])
             AND ST_Distance(a.footprint::geography, b.footprint::geography) <= $2
           ORDER BY a.id, b.id"#,
    )
    .bind(&ids)
    .bind(radius_m)
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| Ok((row.try_get("left_id")?, row.try_get("right_id")?)))
        .collect::<std::result::Result<Vec<_>, sqlx::Error>>()
        .map_err(AppError::Database)
}

async fn load_cluster_geometries(
    db: &sqlx::PgPool,
    components: &[(Uuid, Vec<Uuid>)],
) -> Result<HashMap<Uuid, Value>> {
    if components.is_empty() {
        return Ok(HashMap::new());
    }

    let mut building_ids = Vec::new();
    let mut cluster_ids = Vec::new();
    for (cluster_id, members) in components {
        for building_id in members {
            building_ids.push(*building_id);
            cluster_ids.push(*cluster_id);
        }
    }

    let rows = sqlx::query(
        r#"WITH membership AS (
             SELECT * FROM UNNEST($1::uuid[], $2::uuid[]) AS m(building_id, cluster_id)
           ), merged AS (
             SELECT m.cluster_id,
                    ST_Multi(ST_CollectionExtract(
                      ST_UnaryUnion(ST_Collect(b.footprint)), 3
                    )) AS geom
             FROM membership m
             JOIN public.buildings b ON b.id = m.building_id
             GROUP BY m.cluster_id
           )
           SELECT cluster_id, ST_SRID(geom) AS srid, ST_IsValid(geom) AS is_valid,
                  ST_IsEmpty(geom) AS is_empty, ST_AsGeoJSON(geom) AS geometry
           FROM merged ORDER BY cluster_id"#,
    )
    .bind(&building_ids)
    .bind(&cluster_ids)
    .fetch_all(db)
    .await?;

    let mut geometries = HashMap::new();
    for row in rows {
        let cluster_id: Uuid = row.try_get("cluster_id")?;
        let srid: i32 = row.try_get("srid")?;
        let is_valid: bool = row.try_get("is_valid")?;
        let is_empty: bool = row.try_get("is_empty")?;
        let geometry: Option<String> = row.try_get("geometry")?;
        if srid != 4326 || !is_valid || is_empty {
            return Err(AppError::BadRequest(format!(
                "hotspot geometry for cluster {cluster_id} is invalid or not EPSG:4326"
            )));
        }
        let geometry = geometry.ok_or_else(|| {
            AppError::BadRequest(format!("hotspot geometry for cluster {cluster_id} is empty"))
        })?;
        let value: Value = serde_json::from_str(&geometry)
            .map_err(|error| AppError::Internal(error.into()))?;
        if value.get("type").and_then(Value::as_str) != Some("MultiPolygon") {
            return Err(AppError::BadRequest(format!(
                "hotspot geometry for cluster {cluster_id} is not a MultiPolygon"
            )));
        }
        geometries.insert(cluster_id, value);
    }
    Ok(geometries)
}

async fn list_hotspots(
    State(state): State<AppState>,
    Path(simulation_id): Path<Uuid>,
    Query(query): Query<HotspotQuery>,
) -> Result<Json<HotspotsResponse>> {
    let query = query.validate()?;
    let thresholds = load_run_configuration(&state.db, simulation_id).await?;
    let _time = validate_time_step(&state.db, simulation_id, query.time_step).await?;
    validate_region(&state.db, query.region_id).await?;

    let candidates = load_candidates(&state.db, simulation_id, query, &thresholds).await?;
    let edges = load_edges(&state.db, &candidates, query.radius_m).await?;
    let components = connected_components(&candidates, &edges, query.include_singletons);
    let geometries = load_cluster_geometries(&state.db, &components).await?;

    let candidate_by_id: HashMap<Uuid, &CandidateBuilding> = candidates
        .iter()
        .map(|item| (item.building_id, item))
        .collect();
    let mut hotspots = Vec::with_capacity(components.len());

    for (cluster_id, building_ids) in components {
        let first = candidate_by_id
            .get(&building_ids[0])
            .expect("component members are candidates");
        let population_total: f64 = building_ids
            .iter()
            .map(|id| candidate_by_id[id].population)
            .sum();
        let infected_total: f64 = building_ids
            .iter()
            .map(|id| candidate_by_id[id].infected)
            .sum();
        let infection_ratio = infected_total / population_total;
        let aggregate_risk = thresholds
            .classify(BuildingRiskInput {
                building_id: cluster_id,
                region_id: first.region_id,
                population: Some(population_total),
                infected: infected_total,
            })
            .map_err(|error| AppError::BadRequest(error.to_string()))?;

        let geometry = geometries.get(&cluster_id).cloned().ok_or_else(|| {
            AppError::BadRequest(format!("missing geometry for hotspot {cluster_id}"))
        })?;
        hotspots.push(Hotspot {
            cluster_id,
            simulation_id,
            time_step: query.time_step,
            region_id: first.region_id,
            member_count: building_ids.len() as i64,
            population_total,
            infected_total,
            infection_ratio,
            risk_level: aggregate_risk.risk_level,
            building_ids,
            geometry,
        });
    }

    let hotspot_count = hotspots.len();
    Ok(Json(HotspotsResponse {
        simulation_id,
        time_step: query.time_step,
        region_id: query.region_id,
        configuration: HotspotConfiguration {
            algorithm: "connected_components",
            algorithm_version: ALGORITHM_VERSION,
            minimum_risk_level: query.minimum_risk_level,
            risk_thresholds: thresholds,
            radius_m: query.radius_m(),
            include_singletons: query.include_singletons,
        },
        hotspot_count,
        hotspots,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: Uuid, region_id: Uuid) -> CandidateBuilding {
        CandidateBuilding {
            building_id: id,
            region_id,
            population: 100.0,
            infected: 20.0,
        }
    }

    #[test]
    fn query_applies_defaults_and_rejects_invalid_values() {
        let valid = HotspotQuery {
            time_step: Some("0".into()),
            ..Default::default()
        }
        .validate()
        .unwrap();
        assert_eq!(valid.minimum_risk_level, RiskLevel::High);
        assert_eq!(valid.radius_m, DEFAULT_RADIUS_M);
        assert!(valid.include_singletons);

        for query in [
            HotspotQuery { time_step: None, ..Default::default() },
            HotspotQuery { time_step: Some("-1".into()), ..Default::default() },
            HotspotQuery { time_step: Some("x".into()), ..Default::default() },
            HotspotQuery {
                time_step: Some("1".into()),
                minimum_risk_level: Some("low".into()),
                ..Default::default()
            },
            HotspotQuery {
                time_step: Some("1".into()),
                radius_m: Some("NaN".into()),
                ..Default::default()
            },
            HotspotQuery {
                time_step: Some("1".into()),
                radius_m: Some("5001".into()),
                ..Default::default()
            },
            HotspotQuery {
                time_step: Some("1".into()),
                include_singletons: Some("yes".into()),
                ..Default::default()
            },
        ] {
            assert!(query.validate().is_err());
        }
    }

    #[test]
    fn connected_components_are_transitive_and_deterministic() {
        let region = Uuid::new_v4();
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);
        let c = Uuid::from_u128(3);
        let d = Uuid::from_u128(4);
        let candidates = vec![
            candidate(a, region),
            candidate(b, region),
            candidate(c, region),
            candidate(d, region),
        ];
        let components = connected_components(&candidates, &[(a, b), (b, c)], true);
        assert_eq!(components, vec![(a, vec![a, b, c]), (d, vec![d])]);
    }

    #[test]
    fn singleton_components_can_be_excluded() {
        let region = Uuid::new_v4();
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);
        let c = Uuid::from_u128(3);
        let candidates = vec![candidate(a, region), candidate(b, region), candidate(c, region)];
        let components = connected_components(&candidates, &[(a, b)], false);
        assert_eq!(components, vec![(a, vec![a, b])]);
    }

    #[test]
    fn cluster_id_is_smallest_member_uuid() {
        let region = Uuid::new_v4();
        let a = Uuid::from_u128(30);
        let b = Uuid::from_u128(10);
        let c = Uuid::from_u128(20);
        let candidates = vec![candidate(a, region), candidate(b, region), candidate(c, region)];
        let components = connected_components(&candidates, &[(a, b), (b, c)], true);
        assert_eq!(components[0].0, b);
        assert_eq!(components[0].1, vec![b, c, a]);
    }
}
