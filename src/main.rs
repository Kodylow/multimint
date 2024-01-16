use std::str::FromStr;

use axum::{
    extract::{Query, Request, State},
    middleware::{from_fn, from_fn_with_state, Next},
    Router, body::Body, Extension, http::StatusCode, response::Response, routing::get,
};
use config::CONFIG;
use fedimint_client::ClientArc;
use fedimint_core::{config::FederationId, api::InviteCode};
// use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::info;
use anyhow::Result;

pub mod handlers;
pub mod config;
pub mod multimint;
pub mod db;
pub mod client;
pub mod error;

#[derive(Debug, Clone)]
pub struct AppState {
    pub multimint: multimint::MultiMint,
}

// /// Middleware to check that they used a valid federation_id and only pass on the associated clientArc
// async fn select_federation(
//     state: AppState,
//     next: Next,
// ) -> Result<Response<Body>, StatusCode> {
//     let body = req.body_mut();
//     if let Some(client_arc) = state.multimint.clients.get(&query.federation_id) {
//         req.extensions_mut().insert(client_arc.clone());
//         Ok(next.run(req).await)
//     } else {
//         Err(StatusCode::BAD_REQUEST)
//     }
// }

async fn handle_test(
    State(state): State<AppState>,
) -> Result<String, StatusCode> {
    // just return test string
    for (federation_id, _federation) in state.multimint.clients.lock().await.iter() {
        info!("federation_id: {:?}", federation_id);
    }

    Ok("test".to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let multimint = multimint::MultiMint::new(CONFIG.data_dir.clone()).await?;
    // multimint.register_new(CONFIG.invite_code.clone()).await?;
    // let new_code = InviteCode::from_str("fed11qgqrgvnhwden5te0v9k8q6rp9ekh2arfdeukuet595cr2ttpd3jhq6rzve6zuer9wchxvetyd938gcewvdhk6tcqqysptkuvknc7erjgf4em3zfh90kffqf9srujn6q53d6r056e4apze5cw27h75").unwrap();
    // multimint.register_new(new_code).await?;

    for (federation_id, _federation) in multimint.clients.lock().await.iter() {
        info!("federation_id: {:?}", federation_id);
    }

    let state = AppState { multimint };
    let app = Router::new()
        .route("/", get(handle_test))
        .with_state(state);
        
        // .layer(from_fn_with_state(state, select_federation));
    // .layer(ValidateRequestHeaderLayer::bearer(&CONFIG.password.clone()));

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", CONFIG.host, CONFIG.port))
        .await
        .unwrap();

    info!("Listening on {}", CONFIG.port);

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
