use axum::{
    extract::State,
    Router,  http::StatusCode, routing::get,
};

use multimint::{config::CONFIG, router::AppState};
// use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::info;
use anyhow::Result;

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

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", CONFIG.host, CONFIG.port))
        .await
        .unwrap();

    info!("Listening on {}", CONFIG.port);

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
