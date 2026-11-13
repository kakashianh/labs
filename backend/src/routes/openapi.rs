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
