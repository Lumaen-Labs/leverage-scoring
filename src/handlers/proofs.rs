use crate::{types::*, AppState};
use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use std::sync::Arc;

pub async fn submit_proof(
    State(state): State<Arc<AppState>>,
    Json(submission): Json<ProofSubmission>,
) -> Result<Json<ProofResponse>, (StatusCode, String)> {
    tracing::info!("Received proof submission for wallet: {}", submission.wallet);

    // Step 1: Verify the proof
    // TODO: REPLACE WITH OPACITY VERIFICATION AND UPDATE THE STRUCTURE
    // pub struct VerifiedProof {
    //     pub proof_hash: String,
    //     pub alias: String,
    //     pub status: ProofStatus,
    //     pub verified_at: i64,
    //     pub payload: BinanceProofPayload,
    // }

    // let verified_proof = state
    //     .proof_verifier
    //     .verify_proof(&submission)
    //     .await
    //     .map_err(|e| {
    //         tracing::error!("Proof verification failed: {}", e);
    //         (StatusCode::BAD_REQUEST, format!("Verification failed: {}", e))
    //     })?;

    // let proof_hash = verified_proof.proof_hash.clone();

    // Step 2: Check for replay attack
    // TODO: REMOVE?
    // let existing_proofs = state.proof_store.read().unwrap();
    // let user_proofs = existing_proofs
    //     .get(&submission.wallet)
    //     .map(|v| v.as_slice())
    //     .unwrap_or(&[]);

    // state
    //     .proof_verifier
    //     .check_replay(&proof_hash, user_proofs)
    //     .map_err(|e| {
    //         tracing::error!("Replay attack detected: {}", e);
    //         (StatusCode::CONFLICT, format!("Replay attack: {}", e))
    //     })?;

    // drop(existing_proofs);

    // Step 3: Store verified proof
    // {
    //     let mut proofs = state.proof_store.write().unwrap();
    //     proofs
    //         .entry(submission.wallet.clone())
    //         .or_insert_with(Vec::new)
    //         .push(verified_proof);
    // }

    // Step 4: Trigger scoring job asynchronously
    let state_clone = Arc::clone(&state);
    let wallet_clone = submission.wallet.clone();

    // Run scoring and capture hash
    let result = trigger_scoring(&state_clone, &wallet_clone).await;
    match result {
        Ok(latest_hash) => {
            Ok(Json(ProofResponse {
                proof_hash: latest_hash, // Return the hash from scoring
                status: ProofStatus::Verified,
            }))
        }
        Err(e) => {
            tracing::error!("Scoring job failed for {}: {}", wallet_clone, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Scoring failed: {}", e),
            ))
        }
    }
}

async fn trigger_scoring(state: &AppState, wallet: &str) -> anyhow::Result<String> {
    tracing::info!("Starting scoring job for wallet: {}", wallet);

    // Get all verified proofs for this user
    let proofs = {
        let store = state.proof_store.read().unwrap();
        store.get(wallet).cloned().unwrap_or_default()
    };

    if proofs.is_empty() {
        tracing::warn!("No proofs found for wallet: {}", wallet);
        return Err(anyhow::anyhow!("No proofs found for wallet"));
    }

    // Compute score
    let components = state.scoring_engine.compute_score(wallet, &proofs).await?;
    let total_score = components.total_score();

    tracing::info!("Computed score for {}: {}", wallet, total_score);

    // Get latest proof hash
    let latest_proof_hash = proofs.last().map(|p| p.proof_hash.clone()).unwrap();

    // Submit to Solana
    let expiry = Utc::now().timestamp() + (30 * 24 * 60 * 60); // 30 days
    let tx_sig = state
        .solana_client
        .update_credit_score(wallet, total_score, &latest_proof_hash, expiry)
        .await?;

    tracing::info!("Submitted credit score to Solana, tx: {}", tx_sig);

    // Store score
    let credit_score = CreditScore {
        user_wallet: wallet.to_string(),
        score_total: total_score,
        components,
        last_updated: Utc::now().timestamp(),
        proof_hashes: proofs.iter().map(|p| p.proof_hash.clone()).collect(),
        onchain_tx_sig: Some(tx_sig),
    };

    {
        let mut scores = state.score_store.write().unwrap();
        scores.insert(wallet.to_string(), credit_score);
    }

    Ok(latest_proof_hash)
}