use anyhow::Result;

pub struct HistoryScorer;

impl HistoryScorer {
    pub fn new() -> Self {
        Self
    }

    pub fn score(&self, _wallet: &str) -> Result<u16> {
        // PRODUCTION TODO: Query ProtocolHistoryProgram or DB
        // - Successful loan repayments
        // - On-time payment history
        // - Previous liquidations
        
        tracing::info!("Scoring protocol history (placeholder)");
        
        // Default: no history = baseline score
        Ok(50) // 0-100 range
    }
}