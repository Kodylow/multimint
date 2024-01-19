use axum::{Json, extract::State};
use fedimint_core::config::FederationId;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use serde_json::{Value, json};
use crate::{error::AppError, AppState};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfoResponse {
    #[serde(rename = "federationIds")]
    pub federation_ids: Vec<FederationId>,
}

#[axum_macros::debug_handler]
pub async fn handle_info(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let clients = state.multimint.clients.lock().await;
    let federation_ids = clients.keys().cloned().collect();
    Ok(Json(json!(InfoResponse {
        federation_ids,
    })))
}
