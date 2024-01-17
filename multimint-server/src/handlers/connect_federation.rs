use axum::{Json, extract::State, http::StatusCode};
use fedimint_core::api::InviteCode;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use serde_json::{Value, json};

use crate::{error::AppError, AppState};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectFedPayload {
    pub invite_code: InviteCode,
}

#[axum_macros::debug_handler]
pub async fn handle_connect_federation(
        State(mut state): State<AppState>,
        Json(req): Json<ConnectFedPayload>,
    ) -> Result<Json<Value>, AppError> {
        let invite_code = req.invite_code;

        // Register the federation
        match state.multimint.register_new(invite_code).await {
            Ok(_) => {},
            Err(e) => {
                return Err(AppError::new(StatusCode::CONFLICT, e));
            }
        }

        Ok(Json(json!({
            "status": "ok",
        })))

       
        }
