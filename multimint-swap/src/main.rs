use axum::{
    Router,  routing::post,
};

// use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::info;
use anyhow::Result;

pub mod config;
pub mod handlers;
pub mod error;

use config::CONFIG;

use crate::handlers::swap::handle_swap;

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
        .route("/swap", post(handle_swap))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", CONFIG.host, CONFIG.port))
        .await
        .unwrap();

    info!("Listening on {}", CONFIG.port);

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
