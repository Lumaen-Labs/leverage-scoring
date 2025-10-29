// src/scorer/asset.rs  
use crate::types::BinanceProofPayload;
use anyhow::Result;

pub struct AssetScorer;

impl AssetScorer {
    pub fn new() -> Self {
        Self
    }

    pub fn score(&self, payload: &BinanceProofPayload) -> Result<u16> {
        let mut score = 0u16;

        // CEX Balance (0-120 points)
        if let Some(balance) = &payload.wallet_balance {
            score += self.score_balance(balance.total_balance);
        }

        // Wallet age & diversity (0-50 points)
        score += self.score_diversity(payload);

        Ok(score.min(300))
    }

    fn score_balance(&self, balance: f64) -> u16 {
        match balance {
            b if b >= 50_000.0 => 120,
            b if b >= 10_000.0 => 90,
            b if b >= 1_000.0 => 30,
            _ => 10,
        }
    }

    fn score_diversity(&self, payload: &BinanceProofPayload) -> u16 {
        let mut points = 0u16;

        // Has profile
        if payload.profile.is_some() {
            points += 10;
        }

        // Has spot trades
        if let Some(trades) = &payload.spot_trades {
            if !trades.is_empty() {
                points += 20;
            }
        }

        // Has futures activity
        if let Some(income) = &payload.futures_income_history {
            if !income.is_empty() {
                points += 20;
            }
        }

        points
    }
}