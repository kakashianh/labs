//! Deterministic building-level infection pressure model.
//!
//! This module computes pressure only. Time integration and S/I/R transitions
//! belong to the spatial simulation engine (KAN-24).
//!
//! The model is an academic building-level assumption, not a calibrated
//! epidemiological model or medical guidance.

use std::collections::{HashMap, HashSet};

use thiserror::Error;
use uuid::Uuid;

pub const DEFAULT_NEIGHBOR_INFLUENCE_ALPHA: f64 = 1.0;
pub const MAX_INFECTION_RADIUS_M: f64 = 5_000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildingInfectionInput {
    pub building_id: Uuid,
    pub population: f64,
    pub infected: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeighborEdge {
    pub source_building_id: Uuid,
    pub target_building_id: Uuid,
    pub distance_m: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfectionPressureConfig {
    pub radius_m: f64,
    pub beta: f64,
    pub alpha: f64,
}

impl InfectionPressureConfig {
    pub fn new(radius_m: f64, beta: f64) -> Self {
        Self {
            radius_m,
            beta,
            alpha: DEFAULT_NEIGHBOR_INFLUENCE_ALPHA,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfectionPressure {
    pub building_id: Uuid,
    pub local_prevalence: f64,
    pub neighbor_prevalence: f64,
    pub infection_pressure: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InfectionPressureError {
    #[error("radius_m must be finite, greater than zero, and no greater than 5000")]
    InvalidRadius,
    #[error("beta and alpha must be finite and non-negative")]
    InvalidRateOrMultiplier,
    #[error("building population must be finite and greater than zero")]
    InvalidPopulation,
    #[error("infected count must be finite and between zero and population")]
    InvalidInfected,
    #[error("building_id values must be unique")]
    DuplicateBuilding,
    #[error("neighbor edge distance must be finite and non-negative")]
    InvalidDistance,
    #[error("neighbor edge cannot connect a building to itself")]
    SelfEdge,
    #[error("neighbor edge references a building outside the eligible input set")]
    UnknownBuilding,
    #[error("duplicate undirected neighbor edge")]
    DuplicateEdge,
    #[error("non-finite infection pressure intermediate or output")]
    NumericalInstability,
}

/// Calculate local and distance-weighted neighbor prevalence for each eligible building.
///
/// Edges may be supplied in either direction; each undirected pair is used
/// symmetrically. Edges outside radius_m are ignored. At exactly the radius,
/// the edge has zero weight and therefore contributes nothing.
pub fn calculate_infection_pressures(
    buildings: &[BuildingInfectionInput],
    edges: &[NeighborEdge],
    config: InfectionPressureConfig,
) -> Result<Vec<InfectionPressure>, InfectionPressureError> {
    validate_config(config)?;

    let mut states = HashMap::with_capacity(buildings.len());
    for building in buildings {
        if !building.population.is_finite() || building.population <= 0.0 {
            return Err(InfectionPressureError::InvalidPopulation);
        }
        if !building.infected.is_finite()
            || building.infected < 0.0
            || building.infected > building.population
        {
            return Err(InfectionPressureError::InvalidInfected);
        }
        if states.insert(building.building_id, *building).is_some() {
            return Err(InfectionPressureError::DuplicateBuilding);
        }
    }

    let mut adjacency: HashMap<Uuid, Vec<(Uuid, f64)>> = HashMap::with_capacity(buildings.len());
    let mut seen_edges = HashSet::with_capacity(edges.len());

    for edge in edges {
        if !edge.distance_m.is_finite() || edge.distance_m < 0.0 {
            return Err(InfectionPressureError::InvalidDistance);
        }
        if edge.source_building_id == edge.target_building_id {
            return Err(InfectionPressureError::SelfEdge);
        }
        if !states.contains_key(&edge.source_building_id)
            || !states.contains_key(&edge.target_building_id)
        {
            return Err(InfectionPressureError::UnknownBuilding);
        }

        let (left, right) = if edge.source_building_id < edge.target_building_id {
            (edge.source_building_id, edge.target_building_id)
        } else {
            (edge.target_building_id, edge.source_building_id)
        };
        if !seen_edges.insert((left, right)) {
            return Err(InfectionPressureError::DuplicateEdge);
        }

        if edge.distance_m <= config.radius_m {
            let weight = (1.0 - edge.distance_m / config.radius_m).max(0.0);
            adjacency
                .entry(edge.source_building_id)
                .or_default()
                .push((edge.target_building_id, weight));
            adjacency
                .entry(edge.target_building_id)
                .or_default()
                .push((edge.source_building_id, weight));
        }
    }

    let mut ordered_buildings: Vec<_> = buildings.iter().collect();
    ordered_buildings.sort_by_key(|building| building.building_id);

    let mut output = Vec::with_capacity(ordered_buildings.len());
    for building in ordered_buildings {
        let local_prevalence = building.infected / building.population;
        let mut weighted_neighbor_prevalence = 0.0;
        let mut total_weight = 0.0;

        if let Some(neighbors) = adjacency.get_mut(&building.building_id) {
            neighbors.sort_by_key(|(neighbor_id, _)| *neighbor_id);
            for (neighbor_id, weight) in neighbors.iter() {
                if *weight == 0.0 {
                    continue;
                }
                let neighbor = states
                    .get(neighbor_id)
                    .ok_or(InfectionPressureError::UnknownBuilding)?;
                let neighbor_prevalence = neighbor.infected / neighbor.population;
                weighted_neighbor_prevalence += *weight * neighbor_prevalence;
                total_weight += *weight;
            }
        }

        let neighbor_prevalence = if total_weight > 0.0 {
            weighted_neighbor_prevalence / total_weight
        } else {
            0.0
        };
        let infection_pressure =
            config.beta * (local_prevalence + config.alpha * neighbor_prevalence);

        if !local_prevalence.is_finite()
            || !neighbor_prevalence.is_finite()
            || !infection_pressure.is_finite()
        {
            return Err(InfectionPressureError::NumericalInstability);
        }

        output.push(InfectionPressure {
            building_id: building.building_id,
            local_prevalence,
            neighbor_prevalence,
            infection_pressure,
        });
    }

    Ok(output)
}

fn validate_config(config: InfectionPressureConfig) -> Result<(), InfectionPressureError> {
    if !config.radius_m.is_finite()
        || config.radius_m <= 0.0
        || config.radius_m > MAX_INFECTION_RADIUS_M
    {
        return Err(InfectionPressureError::InvalidRadius);
    }
    if !config.beta.is_finite()
        || config.beta < 0.0
        || !config.alpha.is_finite()
        || config.alpha < 0.0
    {
        return Err(InfectionPressureError::InvalidRateOrMultiplier);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn building(building_id: Uuid, population: f64, infected: f64) -> BuildingInfectionInput {
        BuildingInfectionInput {
            building_id,
            population,
            infected,
        }
    }

    fn edge(source: Uuid, target: Uuid, distance_m: f64) -> NeighborEdge {
        NeighborEdge {
            source_building_id: source,
            target_building_id: target,
            distance_m,
        }
    }

    fn config(radius_m: f64, beta: f64) -> InfectionPressureConfig {
        InfectionPressureConfig::new(radius_m, beta)
    }

    fn pressure(values: &[InfectionPressure], building_id: Uuid) -> &InfectionPressure {
        values
            .iter()
            .find(|value| value.building_id == building_id)
            .expect("building pressure exists")
    }

    #[test]
    fn computes_local_only_pressure_without_neighbors() {
        let result =
            calculate_infection_pressures(&[building(id(1), 100.0, 20.0)], &[], config(100.0, 0.4))
                .unwrap();

        let value = pressure(&result, id(1));
        assert_eq!(value.local_prevalence, 0.2);
        assert_eq!(value.neighbor_prevalence, 0.0);
        assert!((value.infection_pressure - 0.08).abs() < 1e-12);
    }

    #[test]
    fn combines_local_and_neighbor_prevalence_with_alpha_one() {
        let result = calculate_infection_pressures(
            &[building(id(1), 100.0, 20.0), building(id(2), 100.0, 50.0)],
            &[edge(id(1), id(2), 50.0)],
            config(100.0, 0.2),
        )
        .unwrap();

        let value = pressure(&result, id(1));
        assert!((value.local_prevalence - 0.2).abs() < 1e-12);
        assert!((value.neighbor_prevalence - 0.5).abs() < 1e-12);
        assert!((value.infection_pressure - 0.14).abs() < 1e-12);
    }

    #[test]
    fn uninfected_neighbor_contributes_zero_prevalence() {
        let result = calculate_infection_pressures(
            &[building(id(1), 100.0, 10.0), building(id(2), 100.0, 0.0)],
            &[edge(id(1), id(2), 10.0)],
            config(100.0, 0.5),
        )
        .unwrap();

        assert_eq!(pressure(&result, id(1)).neighbor_prevalence, 0.0);
    }

    #[test]
    fn closer_neighbor_has_greater_weight() {
        let result = calculate_infection_pressures(
            &[
                building(id(1), 100.0, 0.0),
                building(id(2), 100.0, 100.0),
                building(id(3), 100.0, 0.0),
            ],
            &[edge(id(1), id(2), 10.0), edge(id(1), id(3), 80.0)],
            config(100.0, 1.0),
        )
        .unwrap();

        let value = pressure(&result, id(1));
        let expected = (0.9 * 1.0 + 0.2 * 0.0) / (0.9 + 0.2);
        assert!((value.neighbor_prevalence - expected).abs() < 1e-12);
    }

    #[test]
    fn edge_at_radius_has_zero_weight_and_outside_edge_is_ignored() {
        let result = calculate_infection_pressures(
            &[
                building(id(1), 100.0, 0.0),
                building(id(2), 100.0, 100.0),
                building(id(3), 100.0, 100.0),
            ],
            &[edge(id(1), id(2), 100.0), edge(id(1), id(3), 101.0)],
            config(100.0, 1.0),
        )
        .unwrap();

        assert_eq!(pressure(&result, id(1)).neighbor_prevalence, 0.0);
    }

    #[test]
    fn zero_total_weight_never_divides_by_zero() {
        let result = calculate_infection_pressures(
            &[building(id(1), 100.0, 0.0), building(id(2), 100.0, 100.0)],
            &[edge(id(1), id(2), 100.0)],
            config(100.0, 1.0),
        )
        .unwrap();

        assert_eq!(pressure(&result, id(1)).neighbor_prevalence, 0.0);
        assert!(result
            .iter()
            .all(|value| value.infection_pressure.is_finite()));
    }

    #[test]
    fn beta_zero_disables_pressure() {
        let result = calculate_infection_pressures(
            &[building(id(1), 100.0, 10.0), building(id(2), 100.0, 20.0)],
            &[edge(id(1), id(2), 1.0)],
            config(100.0, 0.0),
        )
        .unwrap();

        assert!(result.iter().all(|value| value.infection_pressure == 0.0));
    }

    #[test]
    fn alpha_is_configurable_and_defaults_to_one() {
        let mut settings = config(100.0, 0.5);
        assert_eq!(settings.alpha, 1.0);
        settings.alpha = 2.0;

        let result = calculate_infection_pressures(
            &[building(id(1), 100.0, 10.0), building(id(2), 100.0, 50.0)],
            &[edge(id(1), id(2), 0.0)],
            settings,
        )
        .unwrap();

        assert!((pressure(&result, id(1)).infection_pressure - 0.55).abs() < 1e-12);
    }

    #[test]
    fn output_is_stable_when_input_order_changes() {
        let buildings = [
            building(id(1), 100.0, 10.0),
            building(id(2), 200.0, 20.0),
            building(id(3), 50.0, 5.0),
        ];
        let edges = [edge(id(1), id(2), 10.0), edge(id(1), id(3), 20.0)];
        let forward =
            calculate_infection_pressures(&buildings, &edges, config(100.0, 0.3)).unwrap();
        let reverse = calculate_infection_pressures(
            &[buildings[2], buildings[1], buildings[0]],
            &[edges[1], edges[0]],
            config(100.0, 0.3),
        )
        .unwrap();

        assert_eq!(forward, reverse);
    }

    #[test]
    fn rejects_invalid_population_and_infected_state() {
        assert_eq!(
            calculate_infection_pressures(&[building(id(1), 0.0, 0.0)], &[], config(100.0, 0.2)), 
            Err(InfectionPressureError::InvalidPopulation)
        );
        assert_eq!(
            calculate_infection_pressures(
                &[building(id(1), f64::INFINITY, 0.0)],
                &[],
                config(100.0, 0.2)
            ),
            Err(InfectionPressureError::InvalidPopulation)
        );
        for infected in [-1.0, 101.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                calculate_infection_pressures(
                    &[building(id(1), 100.0, infected)],
                    &[],
                    config(100.0, 0.2)
                ),
                Err(InfectionPressureError::InvalidInfected)
            );
        }
    }

    #[test]
    fn rejects_invalid_config_and_distances() {
        for radius in [0.0, -1.0, f64::NAN, f64::INFINITY, 5_000.1] {
            assert_eq!(
                calculate_infection_pressures(&[], &[], config(radius, 0.2)),
                Err(InfectionPressureError::InvalidRadius)
            );
        }
        for (beta, alpha) in [
            (-1.0, 1.0),
            (f64::NAN, 1.0),
            (0.2, -1.0),
            (0.2, f64::INFINITY),
        ] {
            let mut settings = config(100.0, beta);
            settings.alpha = alpha;
            assert_eq!(
                calculate_infection_pressures(&[], &[], settings),
                Err(InfectionPressureError::InvalidRateOrMultiplier)
            );
        }
        for distance in [-1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                calculate_infection_pressures(
                    &[building(id(1), 100.0, 0.0), building(id(2), 100.0, 0.0)],
                    &[edge(id(1), id(2), distance)],
                    config(100.0, 0.2)
                ),
                Err(InfectionPressureError::InvalidDistance)
            );
        }
    }

    #[test]
    fn rejects_duplicate_buildings_self_edges_unknown_and_duplicate_edges() {
        let duplicate = [building(id(1), 100.0, 0.0), building(id(1), 100.0, 0.0)];
        assert_eq!(
            calculate_infection_pressures(&duplicate, &[], config(100.0, 0.2)),
            Err(InfectionPressureError::DuplicateBuilding)
        );

        let buildings = [building(id(1), 100.0, 0.0), building(id(2), 100.0, 0.0)];
        assert_eq!(
            calculate_infection_pressures(
                &buildings,
                &[edge(id(1), id(1), 0.0)],
                config(100.0, 0.2)
            ),
            Err(InfectionPressureError::SelfEdge)
        );
        assert_eq!(
            calculate_infection_pressures(
                &buildings,
                &[edge(id(1), id(3), 0.0)],
                config(100.0, 0.2)
            ),
            Err(InfectionPressureError::UnknownBuilding)
        );
        assert_eq!(
            calculate_infection_pressures(
                &buildings,
                &[edge(id(1), id(2), 10.0), edge(id(2), id(1), 10.0)],
                config(100.0, 0.2)
            ),
            Err(InfectionPressureError::DuplicateEdge)
        );
    }

    #[test]
    fn weighted_neighbor_prevalence_stays_in_unit_interval() {
        let result = calculate_infection_pressures(
            &[
                building(id(1), 100.0, 20.0),
                building(id(2), 100.0, 0.0),
                building(id(3), 100.0, 100.0),
            ],
            &[edge(id(1), id(2), 10.0), edge(id(1), id(3), 20.0)],
            config(100.0, 0.3),
        )
        .unwrap();

        assert!(result
            .iter()
            .all(|value| (0.0..=1.0).contains(&value.neighbor_prevalence)));
    }

    #[test]
    fn eligible_input_excludes_buildings_without_manual_population() {
        // The caller passes only KAN-22 eligible states. An excluded building
        // must not be present in the state map or in the adjacency edge set.
        let result =
            calculate_infection_pressures(&[building(id(1), 100.0, 10.0)], &[], config(100.0, 0.3))
                .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].building_id, id(1));
    }

    #[test]
    fn rejects_non_finite_pressure_output() {
        let mut settings = config(100.0, f64::MAX);
        settings.alpha = f64::MAX;
        assert_eq!(
            calculate_infection_pressures(
                &[building(id(1), 100.0, 100.0), building(id(2), 100.0, 100.0),],
                &[edge(id(1), id(2), 0.0)],
                settings
            ),
            Err(InfectionPressureError::NumericalInstability)
        );
    }
}
