//! Building-level epidemic risk classification, independent from HTTP and SQL.
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const DEFAULT_LOW_MAX: f64 = 0.05;
pub const DEFAULT_MEDIUM_MAX: f64 = 0.15;
pub const DEFAULT_VERSION: &str = "risk-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel { Low, Medium, High, Unclassified }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskThresholds {
    pub version: String,
    pub source: String,
    pub risk_low_max: f64,
    pub risk_medium_max: f64,
}

impl Default for RiskThresholds {
    fn default() -> Self {
        Self { version: DEFAULT_VERSION.into(), source: "project_default".into(),
            risk_low_max: DEFAULT_LOW_MAX, risk_medium_max: DEFAULT_MEDIUM_MAX }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildingRiskInput {
    pub building_id: Uuid,
    pub region_id: Uuid,
    pub population: Option<f64>,
    pub infected: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BuildingRisk {
    pub building_id: Uuid,
    pub region_id: Uuid,
    pub population: Option<f64>,
    pub infected: f64,
    pub infection_ratio: Option<f64>,
    pub risk_level: RiskLevel,
    pub unclassified_reason: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RiskBucket {
    pub risk_level: RiskLevel,
    pub building_count: i64,
    pub population: f64,
    pub infected: f64,
    pub infection_ratio: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RegionRiskSummary {
    pub region_id: Uuid,
    pub total_building_count: i64,
    pub classified_building_count: i64,
    pub unclassified_building_count: i64,
    pub population: f64,
    pub infected: f64,
    pub infection_ratio: Option<f64>,
    pub buckets: Vec<RiskBucket>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RiskError {
    #[error("risk thresholds must be finite and within [0, 1]")]
    InvalidThresholdRange,
    #[error("risk_low_max must be less than risk_medium_max")]
    InvalidThresholdOrder,
    #[error("building population and infected values must be finite and non-negative")]
    InvalidBuildingValues,
}

impl RiskThresholds {
    pub fn validate(&self) -> Result<(), RiskError> {
        if !self.risk_low_max.is_finite() || !self.risk_medium_max.is_finite()
            || !(0.0..=1.0).contains(&self.risk_low_max)
            || !(0.0..=1.0).contains(&self.risk_medium_max) {
            return Err(RiskError::InvalidThresholdRange);
        }
        if self.risk_low_max >= self.risk_medium_max {
            return Err(RiskError::InvalidThresholdOrder);
        }
        Ok(())
    }

    pub fn classify(&self, input: BuildingRiskInput) -> Result<BuildingRisk, RiskError> {
        self.validate()?;
        if !input.infected.is_finite() || input.infected < 0.0
            || input.population.is_some_and(|n| !n.is_finite() || n < 0.0) {
            return Err(RiskError::InvalidBuildingValues);
        }
        let Some(population) = input.population.filter(|n| *n > 0.0) else {
            return Ok(BuildingRisk {
                building_id: input.building_id, region_id: input.region_id,
                population: input.population, infected: input.infected,
                infection_ratio: None, risk_level: RiskLevel::Unclassified,
                unclassified_reason: Some(if input.population.is_none() { "missing_population" } else { "zero_population" }),
            });
        };
        if input.infected > population {
            return Err(RiskError::InvalidBuildingValues);
        }
        let ratio = input.infected / population;
        let level = if ratio <= self.risk_low_max { RiskLevel::Low }
            else if ratio <= self.risk_medium_max { RiskLevel::Medium }
            else { RiskLevel::High };
        Ok(BuildingRisk {
            building_id: input.building_id, region_id: input.region_id,
            population: Some(population), infected: input.infected,
            infection_ratio: Some(ratio), risk_level: level, unclassified_reason: None,
        })
    }
}

pub fn summarize_region(region_id: Uuid, items: &[BuildingRisk]) -> RegionRiskSummary {
    let levels = [RiskLevel::Low, RiskLevel::Medium, RiskLevel::High, RiskLevel::Unclassified];
    let mut buckets = Vec::with_capacity(4);
    for level in levels {
        let selected: Vec<_> = items.iter().filter(|item| item.region_id == region_id && item.risk_level == level).collect();
        let population: f64 = selected.iter().filter_map(|item| item.population.filter(|n| *n > 0.0)).sum();
        let infected: f64 = selected.iter().map(|item| item.infected).sum();
        buckets.push(RiskBucket {
            risk_level: level, building_count: selected.len() as i64,
            population, infected,
            infection_ratio: if level == RiskLevel::Unclassified || population <= 0.0 { None } else { Some(infected / population) },
        });
    }
    let classified: Vec<_> = items.iter().filter(|item| item.region_id == region_id && item.risk_level != RiskLevel::Unclassified).collect();
    let population: f64 = classified.iter().filter_map(|item| item.population).sum();
    let infected: f64 = classified.iter().map(|item| item.infected).sum();
    let total_building_count = items.iter().filter(|item| item.region_id == region_id).count() as i64;
    let unclassified_building_count = buckets[3].building_count;
    RegionRiskSummary {
        region_id, total_building_count,
        classified_building_count: total_building_count - unclassified_building_count,
        unclassified_building_count, population, infected,
        infection_ratio: if population > 0.0 { Some(infected / population) } else { None },
        buckets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input(population: Option<f64>, infected: f64) -> BuildingRiskInput {
        BuildingRiskInput { building_id: Uuid::new_v4(), region_id: Uuid::nil(), population, infected }
    }

    #[test]
    fn classifies_exact_threshold_boundaries() {
        let t = RiskThresholds::default();
        assert_eq!(t.classify(input(Some(100.0), 5.0)).unwrap().risk_level, RiskLevel::Low);
        assert_eq!(t.classify(input(Some(100.0), 15.0)).unwrap().risk_level, RiskLevel::Medium);
        assert_eq!(t.classify(input(Some(100.0), 15.01)).unwrap().risk_level, RiskLevel::High);
    }

    #[test]
    fn rejects_invalid_thresholds_and_values() {
        let mut t = RiskThresholds::default();
        t.risk_low_max = 0.2;
        assert_eq!(t.validate(), Err(RiskError::InvalidThresholdOrder));
        t.risk_low_max = f64::NAN;
        assert_eq!(t.validate(), Err(RiskError::InvalidThresholdRange));
        assert_eq!(RiskThresholds::default().classify(input(Some(10.0), 11.0)), Err(RiskError::InvalidBuildingValues));
    }

    #[test]
    fn missing_and_zero_population_are_unclassified() {
        let t = RiskThresholds::default();
        assert_eq!(t.classify(input(None, 0.0)).unwrap().unclassified_reason, Some("missing_population"));
        let zero = t.classify(input(Some(0.0), 0.0)).unwrap();
        assert_eq!(zero.risk_level, RiskLevel::Unclassified);
        assert_eq!(zero.infection_ratio, None);
    }

    #[test]
    fn regional_ratio_is_weighted_by_population_and_tracks_unclassified() {
        let t = RiskThresholds::default();
        let a = t.classify(input(Some(100.0), 10.0)).unwrap();
        let b = t.classify(input(Some(900.0), 90.0)).unwrap();
        let missing = t.classify(input(None, 0.0)).unwrap();
        let summary = summarize_region(Uuid::nil(), &[a, b, missing]);
        assert!((summary.infection_ratio.unwrap() - 0.1).abs() < 1e-12);
        assert_eq!(summary.unclassified_building_count, 1);
        assert_eq!(summary.total_building_count, 3);
    }

    #[test]
    fn empty_region_has_zero_counts_and_null_ratio() {
        let summary = summarize_region(Uuid::nil(), &[]);
        assert_eq!(summary.total_building_count, 0);
        assert_eq!(summary.infection_ratio, None);
    }
}
