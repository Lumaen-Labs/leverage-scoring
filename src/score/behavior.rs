use crate::types::BinanceProofPayload;
use anyhow::Result;

pub struct BehaviorScorer;

impl BehaviorScorer {
    pub fn new() -> Self {
        Self
    }

    pub fn score(&self, payload: &BinanceProofPayload) -> Result<u16> {
        let mut score = 0i32;

        // Net realized PnL (0-100 points)
        score += self.score_pnl(payload);

        // Trade frequency (0-60 points)
        score += self.score_frequency(payload);

        // Liquidation penalty (up to -60 points)
        score += self.liquidation_penalty(payload);

        Ok(score.max(0).min(200) as u16)
    }

    fn score_pnl(&self, payload: &BinanceProofPayload) -> i32 {
        if let Some(income) = &payload.futures_income_history {
            let total_pnl: f64 = income
                .iter()
                .filter(|i| i.income_type == "REALIZED_PNL")
                .map(|i| i.amount)
                .sum();

            match total_pnl {
                pnl if pnl >= 10_000.0 => 100,
                pnl if pnl >= 1_000.0 => 60,
                pnl if pnl >= 100.0 => 30,
                pnl if pnl >= 0.0 => 10,
                _ => 0,
            }
        } else {
            0
        }
    }

    fn score_frequency(&self, payload: &BinanceProofPayload) -> i32 {
        if let Some(trades) = &payload.spot_trades {
            let trade_count = trades.len() as i32;
            match trade_count {
                c if c >= 100 => 60,
                c if c >= 50 => 40,
                c if c >= 10 => 20,
                _ => 5,
            }
        } else {
            0
        }
    }

    fn liquidation_penalty(&self, payload: &BinanceProofPayload) -> i32 {
        if let Some(income) = &payload.futures_income_history {
            let liquidation_count = income
                .iter()
                .filter(|i| i.amount < -1000.0) // Large negative = likely liquidation
                .count() as i32;

            -liquidation_count * 20 // -20 per liquidation, max -60
        } else {
            0
        }
    }
}