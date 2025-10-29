use alloy_primitives::utils::eip191_hash_message;
use alloy_signer::Signature;
use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use hex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::info;

use crate::{types::*, AppState};

#[derive(Debug, Deserialize)]
struct WebhookShare {
    payload: Value,
    signature: Option<String>,
    message: Option<Value>,
    hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AllowlistResponse {
    #[serde(rename = "allowlistedKeys")]
    allowlisted_keys: Vec<String>,
    count: usize,
    timestamp: u64,
}

pub async fn fetch_data(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let mut results: Vec<WebhookShare> = Vec::new();

    if let Some(res) = body.get("results") {
        if let Ok(parsed) = serde_json::from_value::<Vec<WebhookShare>>(res.clone()) {
            results = parsed;
        }
    }

    if results.is_empty() {
        info!("Missing or empty results field in webhook body.");
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "message": "Missing results field in response."
            })),
        );
    }

    info!("{} share(s) received...", results.len());

    // Prepare a single BinanceProofPayload that will merge all shares
    let mut aggregated_payload = BinanceProofPayload::default();

    for share in results {
        let payload = &share.payload;
        // info!(
        //     "Payload:\n{}",
        //     serde_json::to_string_pretty(payload).unwrap_or_default()
        // );

        // Try to deserialize the payload into a BinanceProofPayload fragment
        if let Ok(partial_payload) = serde_json::from_value::<BinanceProofPayload>(payload.clone()) {
            aggregated_payload = merge_binance_payloads(aggregated_payload, partial_payload);
        }

        if let (Some(signature_hex), Some(message_val)) =
            (share.signature.clone(), share.message.clone())
        {
            let signature_hex = signature_hex.trim_start_matches("0x");
            let signature_bytes = match hex::decode(signature_hex) {
                Ok(bytes) => bytes,
                Err(_) => {
                    info!("Invalid hex in signature: {}", signature_hex);
                    continue;
                }
            };

            let message = serde_json::to_string(&message_val).unwrap_or_default();
            let signature = match Signature::try_from(signature_bytes.as_slice()) {
                Ok(sig) => sig,
                Err(_) => {
                    info!("Failed to parse signature bytes");
                    continue;
                }
            };

            let msg_hash = eip191_hash_message(message.as_bytes());
            let recovered_address = match signature.recover_address_from_msg(msg_hash.as_slice()) {
                Ok(addr) => addr,
                Err(_) => {
                    info!("Failed to recover address from signature");
                    continue;
                }
            };

            // Fetch allowlist
            let client = Client::new();
            let response = client
                .get("https://verifier.opacity.network/api/public-keys")
                .send()
                .await;

            let response = match response {
                Ok(resp) => resp,
                Err(e) => {
                    info!("Failed to fetch allowlist: {}", e);
                    continue;
                }
            };

            let allowlist: AllowlistResponse = match response.json().await {
                Ok(data) => data,
                Err(e) => {
                    info!("Failed to parse allowlist response: {}", e);
                    continue;
                }
            };

            let recovered_hex = format!("{:?}", recovered_address);
            let is_valid = allowlist
                .allowlisted_keys
                .iter()
                .any(|k| k.eq_ignore_ascii_case(&recovered_hex));

            info!("Allowlist contains address? {}", is_valid);

            // Generate and store proof if valid
            // if is_valid {
                let verified_proof = generate_proof(aggregated_payload.clone(), "binance_combined");
                {
                    let mut proofs = state.proof_store.write().unwrap();
                    proofs
                        .entry(recovered_hex.clone())
                        .or_insert_with(Vec::new)
                        .push(verified_proof);
                }
                info!("Stored aggregated proof for {}", recovered_hex);
            // }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "success": true
        })),
    )
}

/// Merge two BinanceProofPayloads intelligently
fn merge_binance_payloads(mut base: BinanceProofPayload, update: BinanceProofPayload) -> BinanceProofPayload {
    if update.profile.is_some() {
        base.profile = update.profile;
    }
    if update.wallet_balance.is_some() {
        base.wallet_balance = update.wallet_balance;
    }
    if let Some(mut trades) = update.spot_trades {
        base.spot_trades.get_or_insert(Vec::new()).append(&mut trades);
    }
    if let Some(mut futures_income) = update.futures_income_history {
        base.futures_income_history
            .get_or_insert(Vec::new())
            .append(&mut futures_income);
    }
    base
}

/// Create a proof from aggregated Binance payload
pub fn generate_proof(payload: BinanceProofPayload, alias: &str) -> VerifiedProof {
    // Serialize payload for hashing
    let payload_str = serde_json::to_string(&payload).unwrap_or_default();

    // Hash to ensure uniqueness
    let mut hasher = Sha256::new();
    hasher.update(payload_str);
    let proof_hash = format!("{:x}", hasher.finalize());

    VerifiedProof {
        proof_hash,
        alias: alias.to_string(),
        status: ProofStatus::Verified,
        verified_at: Utc::now().timestamp(),
        payload,
    }
}
