use crate::{error::AppError, AppState};
use anyhow::Result;
use axum::{extract::State, Json};
use serde_json::{json, Value};

#[axum_macros::debug_handler]
pub async fn handle_info(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let info = state.multimint.info().await?;
    Ok(Json(json!(info)))
}
