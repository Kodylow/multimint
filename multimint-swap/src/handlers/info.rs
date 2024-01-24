use axum::{Json, extract::State};
use anyhow::Result;
use serde_json::{Value, json};
use crate::{error::AppError, AppState};

#[axum_macros::debug_handler]
pub async fn handle_info(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let info = state.multimint.info().await?;
    Ok(Json(json!(info)))
}
