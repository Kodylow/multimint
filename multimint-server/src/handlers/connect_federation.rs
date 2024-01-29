use axum::{Json, extract::State, http::StatusCode};
use fedimint_core::api::InviteCode;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use serde_json::{Value, json};

use crate::{error::AppError, AppState};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectFedPayload {
    pub invite_code: InviteCode,
    pub set_default: bool,
}

#[axum_macros::debug_handler]
pub async fn handle_connect_federation(
        State(mut state): State<AppState>,
        Json(req): Json<ConnectFedPayload>,
    ) -> Result<Json<Value>, AppError> {
        // Register the federation
        match state.multimint.register_new(req.invite_code, req.set_default).await {
            Ok(_) => {},
            Err(e) => {
                return Err(AppError::new(StatusCode::CONFLICT, e));
            }
        }

        Ok(Json(json!({
            "status": "ok",
        })))

       
        }
