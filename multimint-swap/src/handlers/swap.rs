use std::{time::Duration, collections::BTreeMap};

use axum::{Json, extract::State, http::StatusCode};
use fedimint_core::{config::FederationId, Amount};
use fedimint_mint_client::{OOBNotes, MintClientModule, SelectNotesWithAtleastAmount};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use futures_util::StreamExt;
use fedimint_client::ClientArc;
use crate::{error::AppError, AppState};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwapPayload {
    pub to_federation_id: FederationId,
    pub from_federation_id: FederationId,
    pub from_ecash: OOBNotes,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwapResponse {
    pub status: String,
    pub ecash: OOBNotes,
}

#[axum_macros::debug_handler]
pub async fn handle_swap(
    State(state): State<AppState>,
    Json(req): Json<SwapPayload>,
) -> Result<Json<Value>, AppError> {
    let clients = state.multimint.clients.lock().await;
    let (from_client, to_client) = get_clients(&clients, &req)?;

    let amount = req.from_ecash.total_amount();
    check_balance(&to_client, amount).await?;
    let notes = perform_swap(to_client, from_client, req.from_ecash, amount).await?;
    Ok(Json(json!(SwapResponse {
        status: "ok".to_string(),
        ecash: notes,
    })))
}

fn get_clients(
    clients: &BTreeMap<FederationId, ClientArc>,
    req: &SwapPayload,
) -> Result<(ClientArc, ClientArc), AppError> {
    let from_client = clients.get(&req.from_federation_id)
        .ok_or_else(|| AppError::new(StatusCode::BAD_REQUEST, anyhow!("This swap does not have a client for the from_federation_id")))?;
    let to_client = clients.get(&req.to_federation_id)
        .ok_or_else(|| AppError::new(StatusCode::BAD_REQUEST, anyhow!("This swap does not have a client for the to_federation_id")))?;
    Ok((from_client.clone(), to_client.clone()))
}

async fn check_balance(to_client: &ClientArc, amount: Amount) -> Result<(), AppError> {
    let to_client_ecash = to_client.get_balance().await;
    if to_client_ecash < amount {
        return Err(AppError::new(StatusCode::BAD_REQUEST, anyhow!("Not enough ecash to perform this swap")));
    };
    Ok(())
}

async fn perform_swap(to_client: ClientArc, from_client: ClientArc, from_ecash: OOBNotes, amount: Amount) -> Result<OOBNotes, AppError> {
    let from_mint = from_client.get_first_module::<MintClientModule>();
    let to_mint = to_client.get_first_module::<MintClientModule>();

    let operation_id = to_mint.reissue_external_notes(from_ecash, ()).await?;
    let mut updates = from_mint
        .subscribe_reissue_external_notes(operation_id)
        .await
        .unwrap()
        .into_stream();

    let mut notes = None;

    while let Some(update) = updates.next().await {
        match update {
            fedimint_mint_client::ReissueExternalNotesState::Done => {
                let timeout = Duration::from_secs(3600);
                let (_, new_notes) = to_mint
                    .spend_notes_with_selector(&SelectNotesWithAtleastAmount, amount, timeout, ())
                    .await?;
                notes = Some(new_notes);
            },
            fedimint_mint_client::ReissueExternalNotesState::Failed(e) => {
                return Err(AppError::new(StatusCode::INTERNAL_SERVER_ERROR, anyhow!(e)));
            },
            _ => {}
        }
    }
    notes.ok_or_else(|| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, anyhow!("Failed to get notes after swap")))
}
