use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod score;
mod routes;
mod types;
mod solana_client;

use std::sync::{Arc, RwLock};
use score::ScoringEngine;
use types::{ProofStore, ScoreStore};
use anchor_client::{
    solana_sdk::signature::read_keypair_file
};

pub struct AppState {
    // pub proof_verifier: ProofVerifier,
    pub scoring_engine: ScoringEngine,
    pub solana_client: solana_client::SolanaClient,
    pub proof_store: Arc<RwLock<ProofStore>>,
    pub score_store: Arc<RwLock<ScoreStore>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // load environment variables
    let cfg = config::Config::from_env();

    let payer = read_keypair_file(&cfg.keypair_path)?;
    let authority_keypair = Arc::new(payer);    

    // let verifier = ProofVerifier::new();
    let engine = ScoringEngine::new();
    let client = solana_client::SolanaClient::new(
        &cfg.rpc_url,
        authority_keypair,
        &cfg.credit_score_program_id,
    ).expect("Failed to initialize SolanaClient");

    let state = Arc::new(AppState {
        //proof_verifier: verifier,
        scoring_engine: engine,
        solana_client: client,
        proof_store: Arc::new(RwLock::new(ProofStore::new())),
        score_store: Arc::new(RwLock::new(ScoreStore::new())),
    });

    let app = routes::create_router(state);

    // bind to Render-compatible host and port
    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("🚀 Server running on http://{}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server crashed unexpectedly");

    Ok(())
}
