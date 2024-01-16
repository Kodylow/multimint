use axum::{
    extract::{Query, Request},
    middleware::{from_fn, from_fn_with_state, Next},
    Router, body::Body, Extension, http::StatusCode, response::Response, routing::get,
};
use config::CONFIG;
use fedimint_client::ClientArc;
use fedimint_core::config::FederationId;
// use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::info;
use anyhow::Result;

pub mod config;
pub mod multimint;

#[derive(Debug, Clone)]
pub struct AppState {
    pub multimint: multimint::MultiMint,
}

pub struct Params {
    federation_id: FederationId,
}

/// Middleware to check that they used a valid federation_id and only pass on the associated clientArc
async fn select_federation(
    mut req: Request<Body>,
    state: AppState,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let body = req.body_mut();
    if let Some(client_arc) = state.multimint.mint_map.get(&query.federation_id) {
        req.extensions_mut().insert(client_arc.clone());
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn handle_test(
    Extension(client_arc): Extension<ClientArc>,
    _req: Request<Body>,
) -> Result<String, StatusCode> {
    // just return test string
    info!("client_arc: {:?}", client_arc);
    Ok("test".to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let multimint = multimint::MultiMint::new().unwrap();
    multimint.register(CONFIG.invite_code);

    for (federation_id, federation) in multimint.mint_map.iter() {
        info!("federation_id: {:?}", federation_id);
        info!("federation: {:?}", federation);
    }

    let state = AppState { multimint };
    let app = Router::new()
        .route("/test", get(handle_test))
        .layer(from_fn_with_state(state, select_federation));
    // .layer(ValidateRequestHeaderLayer::bearer(&CONFIG.password.clone()));

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", CONFIG.host, CONFIG.port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
