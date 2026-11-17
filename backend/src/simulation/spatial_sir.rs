//! Synchronous building-level SIR simulation using the KAN-23 infection-pressure model.
//!
//! This is a deterministic domain engine. Persistence and HTTP concerns live in routes.

use std::collections::HashSet;

use thiserror::Error;
use uuid::Uuid;

use super::infection_pressure::{
    calculate_infection_pressures, BuildingInfectionInput, InfectionPressureConfig,
    InfectionPressureError, NeighborEdge,
};
use super::sir::{
    MAX_SIMULATION_DURATION, MAX_STEPS, POPULATION_ABS_TOLERANCE,
    POPULATION_REL_TOLERANCE,
};

pub const MAX_BUILDINGS: usize = 500;
pub const MAX_RESULT_ROWS: usize = 250_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildingSirState {
    pub building_id: Uuid,
    pub population: f64,
    pub susceptible: f64,
    pub infected: f64,
    pub recovered: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialSirConfig {
    pub radius_m: f64,
    pub beta: f64,
    pub gamma: f64,
    pub dt: f64,
    pub steps: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialSirPoint {
    pub building_id: Uuid,
    pub time_step: usize,
    pub time: f64,
    pub susceptible: f64,
    pub infected: f64,
    pub recovered: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpatialSirResult {
    pub building_count: usize,
    pub point_count: usize,
    pub points: Vec<SpatialSirPoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SpatialSirError {
    #[error("building selection must contain between 1 and {MAX_BUILDINGS} unique buildings")]
    InvalidBuildingCount,
    #[error("building population must be finite and greater than zero")]
    InvalidPopulation,
    #[error("building S/I/R values must be finite and non-negative")]
    InvalidState,
    #[error("building S + I + R must equal population within tolerance")]
    PopulationMismatch,
    #[error("gamma must be finite and non-negative")]
    InvalidGamma,
    #[error("dt must be finite and greater than zero")]
    InvalidTimeStep,
    #[error("steps must be between 1 and {MAX_STEPS}")]
    InvalidStepCount,
    #[error("simulation duration must be finite and no greater than {MAX_SIMULATION_DURATION}")]
    InvalidDuration,
    #[error("requested result rows exceed the configured maximum of {MAX_RESULT_ROWS}")]
    ResultLimitExceeded,
    #[error("non-finite intermediate or output encountered during simulation")]
    NumericalInstability,
    #[error(transparent)]
    InfectionPressure(#[from] InfectionPressureError),
}

/// Runs all buildings from a shared immutable timestep snapshot. Result ordering is
/// stable by (time_step, building_id), regardless of input building order.
pub fn simulate_spatial_sir(
    buildings: &[BuildingSirState],
    edges: &[NeighborEdge],
    config: SpatialSirConfig,
) -> Result<SpatialSirResult, SpatialSirError> {
    validate(buildings, config)?;

    let mut current = buildings.to_vec();
    current.sort_by_key(|state| state.building_id);
    let mut points = Vec::with_capacity(current.len() * (config.steps + 1));
    append_snapshot(&mut points, &current, 0, 0.0);

    for time_step in 1..=config.steps {
        let infection_inputs: Vec<_> = current
            .iter()
            .map(|state| BuildingInfectionInput {
                building_id: state.building_id,
                population: state.population,
                infected: state.infected,
            })
            .collect();
        let pressures = calculate_infection_pressures(
            &infection_inputs,
            edges,
            InfectionPressureConfig::new(config.radius_m, config.beta),
        )?;

        let mut next = Vec::with_capacity(current.len());
        for (state, pressure) in current.iter().zip(pressures.iter()) {
            debug_assert_eq!(state.building_id, pressure.building_id);
            let raw_infection = pressure.infection_pressure * state.susceptible * config.dt;
            let raw_recovery = config.gamma * state.infected * config.dt;
            if !raw_infection.is_finite() || !raw_recovery.is_finite() {
                return Err(SpatialSirError::NumericalInstability);
            }
            let infection = raw_infection.min(state.susceptible);
            let recovery = raw_recovery.min(state.infected);
            let tolerance = population_tolerance(state.population);
            let susceptible = clamp_roundoff(state.susceptible - infection, tolerance)?;
            let infected = clamp_roundoff(state.infected + infection - recovery, tolerance)?;
            let recovered = clamp_roundoff(state.recovered + recovery, tolerance)?;
            validate_next_state(state.population, susceptible, infected, recovered, tolerance)?;
            next.push(BuildingSirState {
                building_id: state.building_id,
                population: state.population,
                susceptible,
                infected,
                recovered,
            });
        }

        let time = time_step as f64 * config.dt;
        if !time.is_finite() {
            return Err(SpatialSirError::NumericalInstability);
        }
        // Swap only after every building in this timestep has passed validation.
        current = next;
        append_snapshot(&mut points, &current, time_step, time);
    }

    Ok(SpatialSirResult {
        building_count: current.len(),
        point_count: points.len(),
        points,
    })
}

fn validate(
    buildings: &[BuildingSirState],
    config: SpatialSirConfig,
) -> Result<(), SpatialSirError> {
    if buildings.is_empty() || buildings.len() > MAX_BUILDINGS {
        return Err(SpatialSirError::InvalidBuildingCount);
    }
    let rows = buildings
        .len()
        .checked_mul(config.steps.saturating_add(1))
        .ok_or(SpatialSirError::ResultLimitExceeded)?;
    if rows > MAX_RESULT_ROWS {
        return Err(SpatialSirError::ResultLimitExceeded);
    }
    if !config.gamma.is_finite() || config.gamma < 0.0 {
        return Err(SpatialSirError::InvalidGamma);
    }
    if !config.dt.is_finite() || config.dt <= 0.0 {
        return Err(SpatialSirError::InvalidTimeStep);
    }
    if config.steps == 0 || config.steps > MAX_STEPS {
        return Err(SpatialSirError::InvalidStepCount);
    }
    let duration = config.dt * config.steps as f64;
    if !duration.is_finite() || duration > MAX_SIMULATION_DURATION {
        return Err(SpatialSirError::InvalidDuration);
    }

    let mut ids = HashSet::with_capacity(buildings.len());
    for state in buildings {
        if !ids.insert(state.building_id) {
            return Err(SpatialSirError::InvalidBuildingCount);
        }
        if !state.population.is_finite() || state.population <= 0.0 {
            return Err(SpatialSirError::InvalidPopulation);
        }
        if !state.susceptible.is_finite()
            || !state.infected.is_finite()
            || !state.recovered.is_finite()
            || state.susceptible < 0.0
            || state.infected < 0.0
            || state.recovered < 0.0
        {
            return Err(SpatialSirError::InvalidState);
        }
        let total = state.susceptible + state.infected + state.recovered;
        if !total.is_finite()
            || (total - state.population).abs() > population_tolerance(state.population)
        {
            return Err(SpatialSirError::PopulationMismatch);
        }
    }
    Ok(())
}

fn validate_next_state(
    population: f64,
    susceptible: f64,
    infected: f64,
    recovered: f64,
    tolerance: f64,
) -> Result<(), SpatialSirError> {
    let total = susceptible + infected + recovered;
    if !susceptible.is_finite()
        || !infected.is_finite()
        || !recovered.is_finite()
        || !total.is_finite()
        || (total - population).abs() > tolerance
    {
        return Err(SpatialSirError::NumericalInstability);
    }
    Ok(())
}

fn population_tolerance(population: f64) -> f64 {
    POPULATION_ABS_TOLERANCE + POPULATION_REL_TOLERANCE * population
}

fn clamp_roundoff(value: f64, tolerance: f64) -> Result<f64, SpatialSirError> {
    if !value.is_finite() || value < -tolerance {
        return Err(SpatialSirError::NumericalInstability);
    }
    Ok(if value < 0.0 { 0.0 } else { value })
}

fn append_snapshot(
    output: &mut Vec<SpatialSirPoint>,
    states: &[BuildingSirState],
    time_step: usize,
    time: f64,
) {
    output.extend(states.iter().map(|state| SpatialSirPoint {
        building_id: state.building_id,
        time_step,
        time,
        susceptible: state.susceptible,
        infected: state.infected,
        recovered: state.recovered,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u128) -> Uuid { Uuid::from_u128(value) }
    fn state(building_id: Uuid, population: f64, susceptible: f64, infected: f64, recovered: f64) -> BuildingSirState {
        BuildingSirState { building_id, population, susceptible, infected, recovered }
    }
    fn edge(a: Uuid, b: Uuid, distance_m: f64) -> NeighborEdge {
        NeighborEdge { source_building_id: a, target_building_id: b, distance_m }
    }
    fn config(steps: usize) -> SpatialSirConfig {
        SpatialSirConfig { radius_m: 100.0, beta: 0.4, gamma: 0.1, dt: 0.1, steps }
    }
    fn point(result: &SpatialSirResult, building_id: Uuid, time_step: usize) -> &SpatialSirPoint {
        result.points.iter().find(|p| p.building_id == building_id && p.time_step == time_step).unwrap()
    }

    #[test]
    fn no_seed_never_generates_infection() {
        let result = simulate_spatial_sir(&[state(id(1), 100.0, 100.0, 0.0, 0.0)], &[], config(3)).unwrap();
        assert!(result.points.iter().all(|p| p.infected == 0.0 && p.recovered == 0.0));
    }

    #[test]
    fn local_only_matches_single_building_sir_equation() {
        let result = simulate_spatial_sir(&[state(id(1), 100.0, 90.0, 10.0, 0.0)], &[], config(1)).unwrap();
        let p = point(&result, id(1), 1);
        let infection = 0.4 * (10.0 / 100.0) * 90.0 * 0.1;
        let recovery = 0.1 * 10.0 * 0.1;
        assert!((p.susceptible - (90.0 - infection)).abs() < 1e-12);
        assert!((p.infected - (10.0 + infection - recovery)).abs() < 1e-12);
        assert!((p.recovered - recovery).abs() < 1e-12);
    }

    #[test]
    fn infected_neighbor_spreads_using_kan23_pressure() {
        let buildings = [state(id(1), 100.0, 100.0, 0.0, 0.0), state(id(2), 100.0, 90.0, 10.0, 0.0)];
        let result = simulate_spatial_sir(&buildings, &[edge(id(1), id(2), 50.0)], config(1)).unwrap();
        assert!(point(&result, id(1), 1).infected > 0.0);
    }

    #[test]
    fn infection_does_not_cascade_within_the_same_timestep() {
        let buildings = [
            state(id(1), 100.0, 90.0, 10.0, 0.0),
            state(id(2), 100.0, 100.0, 0.0, 0.0),
            state(id(3), 100.0, 100.0, 0.0, 0.0),
        ];
        let edges = [edge(id(1), id(2), 10.0), edge(id(2), id(3), 10.0)];
        let result = simulate_spatial_sir(&buildings, &edges, config(2)).unwrap();

        assert!(point(&result, id(2), 1).infected > 0.0);
        assert_eq!(point(&result, id(3), 1).infected, 0.0);
        assert!(point(&result, id(3), 2).infected > 0.0);
    }

    #[test]
    fn edge_outside_radius_does_not_spread_infection() {
        let buildings = [state(id(1), 100.0, 100.0, 0.0, 0.0), state(id(2), 100.0, 90.0, 10.0, 0.0)];
        let result = simulate_spatial_sir(&buildings, &[edge(id(1), id(2), 101.0)], config(1)).unwrap();
        assert_eq!(point(&result, id(1), 1).infected, 0.0);
    }

    #[test]
    fn every_building_conserves_population_at_every_step() {
        let buildings = [state(id(1), 100.0, 90.0, 10.0, 0.0), state(id(2), 250.0, 245.0, 5.0, 0.0)];
        let result = simulate_spatial_sir(&buildings, &[edge(id(1), id(2), 10.0)], config(10)).unwrap();
        for p in &result.points {
            let population = if p.building_id == id(1) { 100.0 } else { 250.0 };
            assert!((p.susceptible + p.infected + p.recovered - population).abs() < 1e-6);
            assert!(p.susceptible >= 0.0 && p.infected >= 0.0 && p.recovered >= 0.0);
            assert!(p.susceptible.is_finite() && p.infected.is_finite() && p.recovered.is_finite());
        }
    }

    #[test]
    fn same_input_is_deterministic_and_permutation_invariant() {
        let a = state(id(1), 100.0, 90.0, 10.0, 0.0);
        let b = state(id(2), 100.0, 100.0, 0.0, 0.0);
        let edges = [edge(id(1), id(2), 10.0)];
        let first = simulate_spatial_sir(&[a, b], &edges, config(4)).unwrap();
        let second = simulate_spatial_sir(&[b, a], &edges, config(4)).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn rejects_invalid_state_and_resource_limits() {
        assert_eq!(simulate_spatial_sir(&[], &[], config(1)), Err(SpatialSirError::InvalidBuildingCount));
        assert_eq!(simulate_spatial_sir(&[state(id(1), 10.0, 9.0, 2.0, 0.0)], &[], config(1)), Err(SpatialSirError::PopulationMismatch));
        let mut huge = config(1000);
        huge.steps = 1000;
        assert_eq!(simulate_spatial_sir(&[state(id(1), 10.0, 10.0, 0.0, 0.0); 251], &[], huge), Err(SpatialSirError::ResultLimitExceeded));
    }

    #[test]
    fn invalid_rates_and_time_parameters_are_rejected() {
        let building = [state(id(1), 10.0, 10.0, 0.0, 0.0)];
        let mut value = config(1);
        value.gamma = f64::NAN;
        assert_eq!(simulate_spatial_sir(&building, &[], value), Err(SpatialSirError::InvalidGamma));
        value = config(1);
        value.dt = 0.0;
        assert_eq!(simulate_spatial_sir(&building, &[], value), Err(SpatialSirError::InvalidTimeStep));
    }
}
