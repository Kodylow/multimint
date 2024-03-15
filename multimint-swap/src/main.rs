use axum::{
    routing::{get, post},
    Router,
};

// use tower_http::validate_request::ValidateRequestHeaderLayer;
use anyhow::Result;
use tracing::info;

pub mod config;
pub mod error;
pub mod handlers;

use config::CONFIG;

use crate::handlers::{info::handle_info, swap::handle_swap};

#[derive(Debug, Clone)]
pub struct AppState {
    pub multimint: multimint::MultiMint,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let multimint = multimint::MultiMint::new(CONFIG.data_dir.clone()).await?;

    let state = AppState { multimint };
    let app = Router::new()
        .route("/info", get(handle_info))
        .route("/swap", post(handle_swap))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", CONFIG.host, CONFIG.port))
        .await
        .unwrap();

    info!("Listening on {}", CONFIG.port);

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
