use axum::{
    Json,
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::doc_parser::{ItemKind, SearchIndex};
use crate::search::{self, SearchResult};
use crate::src_signature_extractor::SourceSignatureExtractor;

/// Shared application state.
pub struct AppState {
    pub index: SearchIndex,
    pub sig_extractor: Option<SourceSignatureExtractor>,
}

/// Query parameters for the search endpoint.
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// Search query string (required).
    pub q: Option<String>,
    /// Optional filter by item kind (e.g., "struct", "function", "trait").
    pub kind: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
}

/// Response body for the search endpoint.
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    /// The original query string.
    pub query: String,
    /// Total number of matching results.
    pub total_results: usize,
    /// The search results.
    pub results: Vec<SearchResult>,
}

/// Response body for the health endpoint.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub indexed_items: usize,
}

/// Build the Axum router with all routes and middleware.
pub fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_handler))
        .route("/search", get(search_handler))
        .layer(cors)
        .with_state(state)
}

/// Health check endpoint.
async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        indexed_items: state.index.len(),
    })
}

/// Search endpoint.
async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let query = match params.q {
        Some(ref q) if !q.trim().is_empty() => q.trim().to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Missing or empty query parameter 'q'. Usage: /search?q=<keyword>"
                })),
            )
                .into_response();
        }
    };

    // Parse the optional kind filter
    let kind_filter = params
        .kind
        .as_ref()
        .and_then(|k| ItemKind::from_filter(k));

    if params.kind.is_some() && kind_filter.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!(
                    "Invalid kind filter '{}'. Valid values: module, struct, enum, function, trait, method, constant, type, macro, variant",
                    params.kind.as_ref().unwrap()
                )
            })),
        )
            .into_response();
    }

    let mut results = search::search(
        &state.index,
        &query,
        kind_filter.as_ref(),
        params.limit,
    );

    if let Some(extractor) = &state.sig_extractor {
        for result in &mut results {
            if result.signature.is_none() {
                result.signature =
                    extractor.extract_signature(&result.name, &result.kind, &result.path);
            }
        }
    }

    let response = SearchResponse {
        query,
        total_results: results.len(),
        results,
    };

    (StatusCode::OK, Json(serde_json::to_value(response).unwrap())).into_response()
}
