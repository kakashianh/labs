//! SIR simulation HTTP endpoint. The numerical model remains in the domain module.

use crate::{
    error::{AppError, Result},
    simulation::sir::{simulate as run_simulation, SirError, SirInput},
    state::AppState,
};
use axum::{
    extract::{rejection::JsonRejection, Json},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SirSimulationRequest {
    pub population: f64,
    pub susceptible: f64,
    pub infected: f64,
    pub recovered: f64,
    pub beta: f64,
    pub gamma: f64,
    pub dt: f64,
    pub steps: usize,
}

impl From<SirSimulationRequest> for SirInput {
    fn from(request: SirSimulationRequest) -> Self {
        Self {
            population: request.population,
            susceptible: request.susceptible,
            infected: request.infected,
            recovered: request.recovered,
            beta: request.beta,
            gamma: request.gamma,
            dt: request.dt,
            steps: request.steps,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SirSimulationPoint {
    pub time_step: usize,
    pub time: f64,
    pub susceptible: f64,
    pub infected: f64,
    pub recovered: f64,
}

#[derive(Debug, Serialize)]
pub struct SirSimulationResponse {
    pub population: f64,
    pub point_count: usize,
    pub points: Vec<SirSimulationPoint>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/simulations/sir", post(simulate_sir))
}

pub async fn simulate_sir(
    payload: std::result::Result<Json<SirSimulationRequest>, JsonRejection>,
) -> Result<Json<SirSimulationResponse>> {
    // Normalize Axum's malformed/missing/wrong-type JSON rejections to the
    // Backend's existing {"error": "..."} HTTP 400 contract.
    let Json(request) =
        payload.map_err(|_| AppError::BadRequest("invalid JSON request body".into()))?;

    let result = run_simulation(request.into()).map_err(map_sir_error)?;
    let points = result
        .points
        .into_iter()
        .map(|point| SirSimulationPoint {
            time_step: point.time_step,
            time: point.time,
            susceptible: point.susceptible,
            infected: point.infected,
            recovered: point.recovered,
        })
        .collect::<Vec<_>>();

    Ok(Json(SirSimulationResponse {
        population: result.population,
        point_count: points.len(),
        points,
    }))
}

fn map_sir_error(error: SirError) -> AppError {
    match error {
        SirError::NumericalInstability => {
            AppError::Internal(anyhow::anyhow!("SIR simulation numerical instability"))
        }
        validation_error => AppError::BadRequest(validation_error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    fn app() -> Router {
        // The SIR route does not access the database; a lazy pool satisfies
        // the shared AppState type without requiring a running Postgres.
        let db = PgPoolOptions::new()
            .connect_lazy("postgres://appuser:password@localhost/appdb")
            .expect("valid lazy database URL");
        router().with_state(AppState::new(db))
    }

    async fn post_json(body: &str) -> axum::response::Response {
        app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/simulations/sir")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_owned()))
                    .expect("valid request"),
            )
            .await
            .expect("router response")
    }

    async fn response_json(response: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
            .await
            .expect("response body");
        serde_json::from_slice(&bytes).expect("valid JSON response")
    }

    fn valid_request() -> serde_json::Value {
        serde_json::json!({
            "population": 1000.0,
            "susceptible": 990.0,
            "infected": 10.0,
            "recovered": 0.0,
            "beta": 0.3,
            "gamma": 0.1,
            "dt": 0.1,
            "steps": 4
        })
    }

    #[tokio::test]
    async fn valid_request_returns_initial_state_and_every_time_step() {
        let response = post_json(&valid_request().to_string()).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = response_json(response).await;
        assert_eq!(body["population"], 1000.0);
        assert_eq!(body["point_count"], 5);
        let points = body["points"].as_array().expect("points array");
        assert_eq!(points.len(), 5);
        assert_eq!(points[0]["time_step"], 0);
        assert_eq!(points[0]["time"], 0.0);
        assert_eq!(points[0]["susceptible"], 990.0);
        assert_eq!(points[0]["infected"], 10.0);
        assert_eq!(points[0]["recovered"], 0.0);

        for (index, point) in points.iter().enumerate() {
            assert_eq!(point["time_step"], index);
            assert!((point["time"].as_f64().unwrap() - index as f64 * 0.1).abs() < 1e-12);
            let susceptible = point["susceptible"].as_f64().unwrap();
            let infected = point["infected"].as_f64().unwrap();
            let recovered = point["recovered"].as_f64().unwrap();
            assert!(susceptible.is_finite() && infected.is_finite() && recovered.is_finite());
            assert!(susceptible >= 0.0 && infected >= 0.0 && recovered >= 0.0);
            assert!((susceptible + infected + recovered - 1000.0).abs() < 1e-6);
        }
    }

    #[tokio::test]
    async fn malformed_missing_and_wrong_type_json_return_bad_request() {
        let missing_field = r#"{"population":1000}"#;
        let wrong_type = r#"{"population":"many"}"#;

        for body in ["{", missing_field, wrong_type] {
            let response = post_json(body).await;
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let value = response_json(response).await;
            assert_eq!(value["error"], "invalid JSON request body");
        }
    }

    #[tokio::test]
    async fn invalid_domain_parameters_and_excessive_steps_return_bad_request() {
        let mut invalid = valid_request();
        invalid["beta"] = serde_json::json!(-0.1);
        let response = post_json(&invalid.to_string()).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let mut excessive = valid_request();
        excessive["steps"] = serde_json::json!(100_001);
        let response = post_json(&excessive.to_string()).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn repeated_requests_are_deterministic_and_isolated() {
        let app = app();
        let first_request = valid_request();

        let first_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/simulations/sir")
                    .header("content-type", "application/json")
                    .body(Body::from(first_request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first_response.status(), StatusCode::OK);
        let first_body = response_json(first_response).await;

        let mut different_request = valid_request();
        different_request["beta"] = serde_json::json!(0.0);
        different_request["gamma"] = serde_json::json!(0.0);
        different_request["steps"] = serde_json::json!(2);
        let second_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/simulations/sir")
                    .header("content-type", "application/json")
                    .body(Body::from(different_request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second_response.status(), StatusCode::OK);
        let second_body = response_json(second_response).await;

        let repeated_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/simulations/sir")
                    .header("content-type", "application/json")
                    .body(Body::from(first_request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(repeated_response.status(), StatusCode::OK);
        let repeated_body = response_json(repeated_response).await;

        assert_eq!(first_body, repeated_body, "same input must produce the same series");
        assert_ne!(first_body, second_body, "different inputs must produce independent series");
        for point in second_body["points"].as_array().unwrap() {
            assert_eq!(point["susceptible"], 990.0);
            assert_eq!(point["infected"], 10.0);
            assert_eq!(point["recovered"], 0.0);
        }
    }

    #[tokio::test]
    async fn valid_json_with_invalid_population_state_and_time_parameters_returns_bad_request() {
        let mut invalid_population = valid_request();
        invalid_population["population"] = serde_json::json!(0.0);

        let mut invalid_initial_total = valid_request();
        invalid_initial_total["susceptible"] = serde_json::json!(989.0);

        let mut invalid_dt = valid_request();
        invalid_dt["dt"] = serde_json::json!(0.0);

        let mut invalid_steps = valid_request();
        invalid_steps["steps"] = serde_json::json!(0);

        for invalid in [
            invalid_population,
            invalid_initial_total,
            invalid_dt,
            invalid_steps,
        ] {
            let response = post_json(&invalid.to_string()).await;
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let body = response_json(response).await;
            assert!(body["error"].is_string());
        }
    }

    #[tokio::test]
    async fn numerical_instability_returns_generic_internal_error() {
        let mut unstable = valid_request();
        unstable["susceptible"] = serde_json::json!(999.0);
        unstable["infected"] = serde_json::json!(1.0);
        unstable["beta"] = serde_json::json!(f64::MAX);
        unstable["dt"] = serde_json::json!(2.0);
        unstable["steps"] = serde_json::json!(1);

        let response = post_json(&unstable.to_string()).await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = response_json(response).await;
        let error = body["error"].as_str().expect("error string");
        assert_eq!(error, "Internal server error");
        assert!(!error.to_lowercase().contains("numerical"));
        assert!(!error.to_lowercase().contains("stack"));
    }
}
