use crate::cache::SignalCache;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::get;
use axum::Router;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone)]
pub struct ApiState {
    pub signals: SignalCache,
    pub configured_paths: Arc<HashSet<String>>,
}

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/signals", get(list_signals))
        .route("/signals/:path", get(get_signal))
        .with_state(state)
}

async fn list_signals(State(state): State<ApiState>) -> Response {
    let map = state.signals.read().unwrap();
    let mut all: Vec<_> = map.values().cloned().collect();
    all.sort_by(|a, b| a.path.cmp(&b.path));
    Json(all).into_response()
}

async fn get_signal(State(state): State<ApiState>, Path(path): Path<String>) -> Response {
    if !state.configured_paths.contains(&path) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "path not configured",
                "configured": state.configured_paths.iter().collect::<Vec<_>>(),
            })),
        )
            .into_response();
    }
    let map = state.signals.read().unwrap();
    match map.get(&path) {
        Some(signal) => Json(signal.clone()).into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "no value received yet", "path": path })),
        )
            .into_response(),
    }
}
