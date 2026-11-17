//! OpenAPI specification and interactive API documentation.

use crate::state::AppState;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api-docs/openapi.json", get(openapi_json))
        .route("/docs", get(swagger_ui))
}

async fn openapi_json() -> impl IntoResponse {
    (
        [("content-type", "application/json; charset=utf-8")],
        include_str!("../../openapi/openapi.json"),
    )
}

async fn swagger_ui() -> Html<&'static str> {
    Html(include_str!("../../openapi/swagger-ui.html"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn openapi_spec_is_valid_and_documents_building_list() {
        let spec: serde_json::Value =
            serde_json::from_str(include_str!("../../openapi/openapi.json"))
                .expect("OpenAPI JSON must be valid");
        assert_eq!(spec["openapi"], "3.0.3");
        assert!(spec["paths"]["/api/v1/buildings"]["get"].is_object());
        assert!(spec["paths"]["/api/v1/regions"]["get"].is_object());
        assert!(spec["paths"]["/api/v1/regions/{region_id}"]["get"].is_object());
        assert!(spec["paths"]["/api/v1/buildings/{building_id}"]["get"].is_object());
        assert!(spec["paths"]["/api/v1/buildings/{building_id}/neighbors"]["get"].is_object());
        assert!(spec["components"]["schemas"]["BuildingNeighborsResponse"].is_object());
        let neighbor_parameters = spec["paths"]["/api/v1/buildings/{building_id}/neighbors"]["get"]
            ["parameters"]
            .as_array()
            .expect("neighbor query parameters");
        assert!(neighbor_parameters.iter().any(|parameter| {
            parameter["name"] == "radius_m" && parameter["required"].as_bool() == Some(true)
        }));
        assert!(neighbor_parameters.iter().any(|parameter| {
            parameter["name"] == "limit" && parameter["required"].as_bool() == Some(false)
        }));
        assert!(spec["paths"]["/api/v1/simulations/sir"]["post"].is_object());
        assert!(spec["components"]["schemas"]["SirSimulationRequest"].is_object());
        assert!(spec["components"]["schemas"]["SirSimulationResponse"].is_object());
        assert!(spec["components"]["schemas"]["SirSimulationPoint"].is_object());
        assert_eq!(
            spec["paths"]["/api/v1/regions/{region_id}"]["get"]["parameters"][0]["schema"]
                ["format"],
            "uuid"
        );
        assert!(spec["components"]["schemas"]["RegionDetail"].is_object());
        assert!(spec["components"]["schemas"]["BuildingDetail"].is_object());
        assert_eq!(
            spec["paths"]["/api/v1/buildings"]["get"]["parameters"][1]["schema"]["maximum"],
            500
        );
        assert_eq!(
            spec["components"]["schemas"]["MultiPolygon"]["properties"]["type"]["enum"][0],
            "MultiPolygon"
        );
    }
}
