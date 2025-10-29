pub mod asset;
pub mod behavior;
pub mod history;
pub mod onchain;

use crate::types::*;
use anyhow::Result;

pub struct ScoringEngine {
    onchain_analyzer: onchain::OnchainAnalyzer,
    asset_scorer: asset::AssetScorer,
    behavior_scorer: behavior::BehaviorScorer,
    history_scorer: history::HistoryScorer,
}

impl ScoringEngine {
    pub fn new() -> Self {
        Self {
            onchain_analyzer: onchain::OnchainAnalyzer::new(),
            asset_scorer: asset::AssetScorer::new(),
            behavior_scorer: behavior::BehaviorScorer::new(),
            history_scorer: history::HistoryScorer::new(),
        }
    }

    /// Compute complete score for a user
    pub async fn compute_score(
        &self,
        wallet: &str,
        proofs: &[VerifiedProof],
    ) -> Result<ScoreComponents> {
        // Merge all Binance payloads from verified proofs
        let merged_payload = self.merge_binance_payloads(proofs);

        // Compute individual scores in parallel
        let onchain_score = self.onchain_analyzer.analyze(wallet).await?;
        let asset_score = self.asset_scorer.score(&merged_payload)?;
        let behavior_score = self.behavior_scorer.score(&merged_payload)?;
        let history_score = self.history_scorer.score(wallet)?;

        Ok(ScoreComponents {
            onchain_score,
            asset_score,
            behavior_score,
            history_score,
            zktls_binance_payload: Some(merged_payload),
        })
    }

    fn merge_binance_payloads(&self, proofs: &[VerifiedProof]) -> BinanceProofPayload {
        let mut merged = BinanceProofPayload::default();

        for proof in proofs {
            let payload = &proof.payload;

            if merged.profile.is_none() && payload.profile.is_some() {
                merged.profile = payload.profile.clone();
            }
            if merged.wallet_balance.is_none() && payload.wallet_balance.is_some() {
                merged.wallet_balance = payload.wallet_balance.clone();
            }
            
            // Merge trades
            if let Some(trades) = &payload.spot_trades {
                merged.spot_trades.get_or_insert_with(Vec::new).extend(trades.clone());
            }
            
            // Merge income history
            if let Some(income) = &payload.futures_income_history {
                merged.futures_income_history.get_or_insert_with(Vec::new).extend(income.clone());
            }
        }

        merged
    }
}