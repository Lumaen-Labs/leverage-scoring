use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSubmission {
    pub wallet: String,
    pub alias: String,
    pub proof_id: String,
    pub proof_blob: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResponse {
    pub proof_hash: String,
    pub status: ProofStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProofStatus {
    Received,
    Verified,
    Rejected,
}

// Binance-specific proof payload structures
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BinanceProofPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<BinanceProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_balance: Option<WalletBalance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spot_trades: Option<Vec<SpotTrade>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub futures_income_history: Option<Vec<FuturesIncome>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub total_balance: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotTrade {
    pub symbol: String,
    pub price: f64,
    pub qty: f64,
    pub side: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotOrder {
    pub symbol: String,
    pub status: String,
    pub price: f64,
    pub qty: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesIncome {
    pub symbol: String,
    pub income_type: String,
    pub amount: f64,
    pub timestamp: i64,
}

// Score structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreComponents {
    pub onchain_score: u16,      // 0-400
    pub asset_score: u16,         // 0-300
    pub behavior_score: u16,      // 0-200
    pub history_score: u16,       // 0-100
    pub zktls_binance_payload: Option<BinanceProofPayload>,
}

impl ScoreComponents {
    pub fn total_score(&self) -> u16 {
        self.onchain_score
            .saturating_add(self.asset_score)
            .saturating_add(self.behavior_score)
            .saturating_add(self.history_score)
            .min(1000)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditScore {
    pub user_wallet: String,
    pub score_total: u16,
    pub components: ScoreComponents,
    pub last_updated: i64,
    pub proof_hashes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onchain_tx_sig: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreResponse {
    pub score_total: u16,
    pub components: ScoreComponents,
    pub last_updated: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onchain_tx_sig: Option<String>,
}

// In-memory storage (replace with DB later)
pub type ProofStore = HashMap<String, Vec<VerifiedProof>>;
pub type ScoreStore = HashMap<String, CreditScore>;

#[derive(Debug, Clone)]
pub struct VerifiedProof {
    pub proof_hash: String,
    pub alias: String,
    pub status: ProofStatus,
    pub verified_at: i64,
    pub payload: BinanceProofPayload,
}