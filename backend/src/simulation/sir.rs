//! Deterministic, aggregate SIR simulation using explicit Euler integration.
//!
//! beta and gamma are rates in inverse time units; dt uses the same unit.
//! Population values are f64 expected/aggregate counts and may be fractional.
//! The initial state is included as the first point (time_step = 0).

use thiserror::Error;

pub const MAX_STEPS: usize = 100_000;
pub const MAX_SIMULATION_DURATION: f64 = 10_000.0;
pub const POPULATION_ABS_TOLERANCE: f64 = 1e-9;
pub const POPULATION_REL_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SirInput {
    pub population: f64,
    pub susceptible: f64,
    pub infected: f64,
    pub recovered: f64,
    pub beta: f64,
    pub gamma: f64,
    pub dt: f64,
    pub steps: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SirPoint {
    pub time_step: usize,
    pub time: f64,
    pub susceptible: f64,
    pub infected: f64,
    pub recovered: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SirResult {
    pub population: f64,
    pub points: Vec<SirPoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SirError {
    #[error("population must be finite and greater than zero")]
    InvalidPopulation,
    #[error("initial susceptible, infected and recovered values must be finite and non-negative")]
    InvalidInitialState,
    #[error("initial susceptible + infected + recovered must equal population within tolerance")]
    InitialPopulationMismatch,
    #[error("beta and gamma must be finite and non-negative")]
    InvalidRate,
    #[error("dt must be finite and greater than zero")]
    InvalidTimeStep,
    #[error("steps must be between 1 and {MAX_STEPS}")]
    InvalidStepCount,
    #[error("simulation duration must be finite and no greater than {MAX_SIMULATION_DURATION}")]
    InvalidSimulationDuration,
    #[error("non-finite intermediate or output encountered during simulation")]
    NumericalInstability,
}

pub fn simulate(input: SirInput) -> Result<SirResult, SirError> {
    validate(input)?;

    let mut susceptible = input.susceptible;
    let mut infected = input.infected;
    let mut recovered = input.recovered;
    let mut points = Vec::with_capacity(input.steps.saturating_add(1));

    points.push(SirPoint { time_step: 0, time: 0.0, susceptible, infected, recovered });

    for time_step in 1..=input.steps {
        // Both flows are computed from the same beginning-of-step state.
        let raw_infection_flow =
            input.beta * (susceptible / input.population) * infected * input.dt;
        let raw_recovery_flow = input.gamma * infected * input.dt;
        if !raw_infection_flow.is_finite() || !raw_recovery_flow.is_finite() {
            return Err(SirError::NumericalInstability);
        }
        let infection_flow = raw_infection_flow.min(susceptible);
        let recovery_flow = raw_recovery_flow.min(infected);

        susceptible -= infection_flow;
        infected += infection_flow - recovery_flow;
        recovered += recovery_flow;

        let tolerance = population_tolerance(input.population);
        susceptible = clamp_roundoff(susceptible, tolerance)?;
        infected = clamp_roundoff(infected, tolerance)?;
        recovered = clamp_roundoff(recovered, tolerance)?;

        let total = susceptible + infected + recovered;
        if !total.is_finite()
            || !susceptible.is_finite()
            || !infected.is_finite()
            || !recovered.is_finite()
            || (total - input.population).abs() > tolerance
        {
            return Err(SirError::NumericalInstability);
        }

        let time = time_step as f64 * input.dt;
        if !time.is_finite() {
            return Err(SirError::NumericalInstability);
        }
        points.push(SirPoint { time_step, time, susceptible, infected, recovered });
    }

    Ok(SirResult { population: input.population, points })
}

pub fn validate(input: SirInput) -> Result<(), SirError> {
    if !input.population.is_finite() || input.population <= 0.0 {
        return Err(SirError::InvalidPopulation);
    }
    if !input.susceptible.is_finite()
        || !input.infected.is_finite()
        || !input.recovered.is_finite()
        || input.susceptible < 0.0
        || input.infected < 0.0
        || input.recovered < 0.0
    {
        return Err(SirError::InvalidInitialState);
    }

    let initial_total = input.susceptible + input.infected + input.recovered;
    if !initial_total.is_finite()
        || (initial_total - input.population).abs() > population_tolerance(input.population)
    {
        return Err(SirError::InitialPopulationMismatch);
    }

    if !input.beta.is_finite() || !input.gamma.is_finite() || input.beta < 0.0 || input.gamma < 0.0 {
        return Err(SirError::InvalidRate);
    }
    if !input.dt.is_finite() || input.dt <= 0.0 {
        return Err(SirError::InvalidTimeStep);
    }
    if input.steps == 0 || input.steps > MAX_STEPS {
        return Err(SirError::InvalidStepCount);
    }

    let duration = input.dt * input.steps as f64;
    if !duration.is_finite() || duration > MAX_SIMULATION_DURATION {
        return Err(SirError::InvalidSimulationDuration);
    }
    Ok(())
}

fn population_tolerance(population: f64) -> f64 {
    POPULATION_ABS_TOLERANCE + POPULATION_REL_TOLERANCE * population
}

fn clamp_roundoff(value: f64, tolerance: f64) -> Result<f64, SirError> {
    if !value.is_finite() || value < -tolerance {
        return Err(SirError::NumericalInstability);
    }
    Ok(if value < 0.0 { 0.0 } else { value })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> SirInput {
        SirInput { population: 1_000.0, susceptible: 990.0, infected: 10.0,
            recovered: 0.0, beta: 0.3, gamma: 0.1, dt: 0.1, steps: 20 }
    }
    fn last(result: &SirResult) -> SirPoint { *result.points.last().unwrap() }

    #[test]
    fn initial_state_and_every_step_are_returned() {
        let result = simulate(input()).unwrap();
        assert_eq!(result.points.len(), 21);
        assert_eq!(result.points[0].time_step, 0);
        assert_eq!(result.points[0].time, 0.0);
        assert_eq!(result.points[0].susceptible, 990.0);
        assert_eq!(result.points[20].time_step, 20);
        assert!((result.points[20].time - 2.0).abs() < 1e-12);
    }

    #[test]
    fn normal_run_preserves_population_and_non_negative_values() {
        for point in simulate(input()).unwrap().points {
            assert!(point.susceptible >= 0.0 && point.infected >= 0.0 && point.recovered >= 0.0);
            assert!((point.susceptible + point.infected + point.recovered - 1_000.0).abs() < 1e-6);
        }
    }

    #[test]
    fn beta_zero_disables_infection_but_allows_recovery() {
        let mut value = input(); value.beta = 0.0;
        let result = simulate(value).unwrap();
        assert_eq!(last(&result).susceptible, value.susceptible);
        assert!(last(&result).infected < value.infected && last(&result).recovered > 0.0);
    }

    #[test]
    fn gamma_zero_disables_recovery() {
        let mut value = input(); value.gamma = 0.0;
        let result = simulate(value).unwrap();
        assert!(last(&result).susceptible < value.susceptible && last(&result).infected > value.infected);
        assert_eq!(last(&result).recovered, value.recovered);
    }

    #[test]
    fn zero_infected_and_zero_susceptible_are_boundaries() {
        let mut value = input(); value.susceptible = 1_000.0; value.infected = 0.0;
        assert!(simulate(value).unwrap().points.iter().all(|p| p.infected == 0.0 && p.recovered == 0.0));
        value = input(); value.susceptible = 0.0; value.infected = 10.0; value.recovered = 990.0;
        assert!(simulate(value).unwrap().points.iter().all(|p| p.susceptible == 0.0));
    }

    #[test]
    fn all_recovered_and_zero_rates_are_stable() {
        let mut value = input(); value.susceptible = 0.0; value.infected = 0.0; value.recovered = 1_000.0;
        assert!(simulate(value).unwrap().points.iter().all(|p| p.recovered == 1_000.0));
        value = input(); value.beta = 0.0; value.gamma = 0.0;
        assert!(simulate(value).unwrap().points.iter().all(|p| p.susceptible == 990.0 && p.infected == 10.0 && p.recovered == 0.0));
    }

    #[test]
    fn one_step_and_small_dt_work() {
        let mut value = input(); value.steps = 1;
        assert_eq!(simulate(value).unwrap().points.len(), 2);
        value.steps = 2; value.dt = 1e-8;
        assert_eq!(simulate(value).unwrap().points.len(), 3);
    }

    #[test]
    fn smaller_dt_produces_finite_convergent_trend() {
        let mut coarse = input(); coarse.steps = 10; coarse.dt = 0.2;
        let mut fine = coarse; fine.steps = 20; fine.dt = 0.1;
        let a = last(&simulate(coarse).unwrap()); let b = last(&simulate(fine).unwrap());
        assert!(b.susceptible.is_finite() && b.infected.is_finite() && b.recovered.is_finite());
        assert!((a.infected - b.infected).abs() < 1.0);
    }

    #[test]
    fn same_input_is_deterministic() { assert_eq!(simulate(input()).unwrap(), simulate(input()).unwrap()); }

    #[test]
    fn rejects_invalid_population_and_state() {
        let mut value = input(); value.population = 0.0;
        assert_eq!(validate(value), Err(SirError::InvalidPopulation));
        value = input(); value.susceptible = -1.0;
        assert_eq!(validate(value), Err(SirError::InvalidInitialState));
        value = input(); value.infected = f64::NAN;
        assert_eq!(validate(value), Err(SirError::InvalidInitialState));
    }

    #[test]
    fn validates_initial_population_tolerance() {
        let mut value = input(); value.recovered = 1.0;
        assert_eq!(validate(value), Err(SirError::InitialPopulationMismatch));
        value = input(); value.susceptible += 1e-8;
        assert!(validate(value).is_ok());
    }

    #[test]
    fn rejects_invalid_rates_dt_steps_and_duration() {
        let mut value = input(); value.beta = -0.1;
        assert_eq!(validate(value), Err(SirError::InvalidRate));
        value = input(); value.gamma = f64::INFINITY;
        assert_eq!(validate(value), Err(SirError::InvalidRate));
        value = input(); value.dt = 0.0;
        assert_eq!(validate(value), Err(SirError::InvalidTimeStep));
        value = input(); value.steps = 0;
        assert_eq!(validate(value), Err(SirError::InvalidStepCount));
        value.steps = MAX_STEPS + 1;
        assert_eq!(validate(value), Err(SirError::InvalidStepCount));
        value = input(); value.dt = MAX_SIMULATION_DURATION; value.steps = 2;
        assert_eq!(validate(value), Err(SirError::InvalidSimulationDuration));
    }

    #[test]
    fn rejects_overflowing_intermediate() {
        let mut value = input(); value.beta = f64::MAX; value.steps = 1;
        assert_eq!(simulate(value), Err(SirError::NumericalInstability));
    }
}
