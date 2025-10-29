use dotenvy::dotenv;
use std::env;

pub struct Config {
    pub rpc_url: String,
    pub keypair_path: String,
    pub port: u16,
    pub credit_score_program_id: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        Self {
            rpc_url: env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
            keypair_path: env::var("SOLANA_KEYPAIR").unwrap_or_else(|_| "./id.json".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
            credit_score_program_id: env::var("CREDIT_SCORE_PROGRAM_ID").unwrap_or_else(|_| "5M7DFNUwLpR3eFvCMuKpukVNx6W6fQXz9hHYB8SxG7uy".to_string()),
        }
    }
}
