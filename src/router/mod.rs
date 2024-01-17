pub mod handlers;
pub mod error;

#[derive(Debug, Clone)]
pub struct AppState {
    pub multimint: crate::MultiMint,
}
