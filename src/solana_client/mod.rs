use anchor_lang::prelude::*;
declare_program!(core_router);

use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_program,
    },
    Client, Cluster, Program,
};
use anyhow::{anyhow, Result};
use std::{str::FromStr, sync::Arc};

// Auto-generate the client bindings using the IDL at /idls/credit_score.json
use core_router::client::{accounts, args};

pub struct SolanaClient {
    client: Client<Arc<Keypair>>,
    authority_keypair: Arc<Keypair>,
    credit_score_program_id: Pubkey,
}

impl SolanaClient {
    pub fn new(
        rpc_url: &str,
        authority_keypair: Arc<Keypair>,
        credit_score_program_id: &str,
    ) -> Result<Self> {
        let client = Client::new_with_options(
            Cluster::Custom(rpc_url.to_string(), rpc_url.to_string()),
            Arc::clone(&authority_keypair),
            CommitmentConfig::confirmed(),
        );

        let program_id = Pubkey::from_str(credit_score_program_id)
            .map_err(|e| anyhow!("Invalid program ID: {}", e))?;

        Ok(Self {
            client,
            authority_keypair,
            credit_score_program_id: program_id,
        })
    }

    /// Submit UpdateCreditScore transaction to Solana
    pub async fn update_credit_score(
        &self,
        user_pubkey: &str,
        new_score: u16,
        proof_hash: &str,
        expiry: i64,
    ) -> Result<String> {
        let user_pk = Pubkey::from_str(user_pubkey)
            .map_err(|e| anyhow!("Invalid user pubkey: {}", e))?;

        // Convert proof_hash hex string to [u8; 32]
        let proof_hash_bytes = hex::decode(proof_hash)
            .map_err(|e| anyhow!("Invalid proof hash: {}", e))?;

        if proof_hash_bytes.len() != 32 {
            return Err(anyhow!("Proof hash must be 32 bytes"));
        }

        let mut proof_hash_array = [0u8; 32];
        proof_hash_array.copy_from_slice(&proof_hash_bytes);

        // Derive CreditScoreAccount PDA
        let (credit_score_account, _bump) = Pubkey::find_program_address(
            &[b"credit_score", user_pk.as_ref()],
            &self.credit_score_program_id,
        );

        tracing::info!(
            "Updating credit score for {} to {} (proof: {})",
            user_pubkey,
            new_score,
            &proof_hash[..8]
        );

        // Get Anchor program instance
        let program: Program<Arc<Keypair>> = self.client.program(self.credit_score_program_id)?;

        // === Build and send transaction ===
        let signature = program
            .request()
            .accounts(accounts::UpdateCreditScore {
                credit_score_account,
                authority: self.authority_keypair.pubkey(),
                system_program: system_program::ID,
            })
            .args(args::UpdateCreditScore {
                user_pubkey: user_pk,
                new_score,
                proof_hash: proof_hash_array,
                expiry,
            })
            .signer(&*self.authority_keypair)
            .send()?;
        
        tracing::info!("Credit score updated, tx: {}", signature);
        Ok(signature.to_string())
    }

    pub fn authority_pubkey(&self) -> Pubkey {
        self.authority_keypair.pubkey()
    }
}
