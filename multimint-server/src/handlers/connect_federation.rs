use anyhow::Result;
use axum::{extract::State, http::StatusCode, Json};
use fedimint_core::api::InviteCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{error::AppError, AppState};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectFedPayload {
    pub invite_code: InviteCode,
}

#[axum_macros::debug_handler]
pub async fn handle_connect_federation(
    State(mut state): State<AppState>,
    Json(req): Json<ConnectFedPayload>,
) -> Result<Json<Value>, AppError> {
    // Register the federation
    match state.multimint.register_new(req.invite_code).await {
        Ok(_) => {}
        Err(e) => {
            return Err(AppError::new(StatusCode::CONFLICT, e));
        }
    }

    Ok(Json(json!({
        "status": "ok",
    })))
}
