## Things We're Keeping for Later

- Database layer (Postgres) - storing proofs, scores, positions
- Redis caching - locks, rate limits, stale flags
- Liquidation & health monitoring - scheduled CheckHealth, AutoDelever, Liquidate calls
- Alerting & monitoring - Slack/PagerDuty, dashboards
- Oracle authority separation - HSM/KMS key management, dedicated oracle service
- Protocol History Program - immutable ledger of events
- Advanced security - encryption at rest, envelope encryption for proofs
- WebSocket real-time updates - WS endpoint for live score updates
- Multi-oracle redundancy - multiple oracle keys for verification
- Rate limiting & DDoS protection - per-wallet/IP throttling
- Admin endpoints - manual review, proof revocation
- Sophisticated behavior scoring - NFT history, futures liquidation detection
- Scheduled bulk refresh - 7-day score refresh jobs

## Required Solana programs (names, accounts, and exact params)

You will deploy three program binaries (kept minimal and permissioned):

## Program A — `LendingPoolProgram`

Purpose: core lending logic, positions, liquidation, deposits/withdraws.
**Accounts (per invocation)**:

* `PoolAccount (PDA)` — holds `ProtocolRiskParams`, total deposits/borrows, admin.
* `PositionAccount (PDA)` — per-user position (owner pubkey).
* `UserTokenAccount` — user’s collateral SPL token account.
* `PoolVault` — pool's SPL token vault for borrowed asset (USDC mint).
* `PriceFeedAccount` — oracle feed (Pyth/Switchboard) for asset price.

**Key instructions & exact parameters (what backend calls / expects):**

* `InitializePool(pool_bump: u8, risk_params_hash: [u8;32])`
  -> set pool parameters (admin-only).
* `InitializePosition(owner: Pubkey, collateral_mint: Pubkey, collateral_amount: u64)`
  -> creates PositionAccount with `max_leverage = base_leverage`.
* `Borrow(position_pubkey: Pubkey, amount: u64)`
  -> requires `PositionAccount`, `PoolVault`, `UserTokenAccount`, `PriceFeed`.
* `Repay(position_pubkey: Pubkey, amount: u64)`
* `CheckHealth(position_pubkey: Pubkey)` — idempotent health recalculation (anyone can call, but backend does scheduled calls).
* `AutoDelever(position_pubkey: Pubkey, percent: u8)` — reduces borrow by percent (authorized by program logic).
* `Liquidate(position_pubkey: Pubkey, liquidator: Pubkey)` — partial liquidation logic.

> Note: these instructions are the on-chain surface; the backend calls them by producing signed transactions (either user-signed for `Borrow`/`Repay` or backend-signed for `CheckHealth`/`AutoDelever` if authorized by program).

---

## Program B — `CreditScoreProgram`

Purpose: store `CreditScoreAccount` PDAs and allow authorized oracle(s) to update them.
**Accounts:**

* `CreditScoreAccount (PDA derived from user pubkey + program id)`
* `PoolAccount` or `AdminAccount` (to verify oracle authority)

**Key instructions & exact parameters:**

* `CreateCreditAccount(user_pubkey: Pubkey)` — creates the account with empty score and expiry.
* `UpdateCreditScore(user_pubkey: Pubkey, new_score: u16, proof_hash: [u8;32], last_updated: i64, expiry: i64)`
  -> must be signed by `OracleKey` (the oracle authority pubkey set in PoolAccount). Backend uses oracle keypair to sign this tx.
* `RevokeProof(user_pubkey: Pubkey, proof_hash: [u8;32])` — optional to invalidate suspected proofs.

Important: `UpdateCreditScore` must enforce the “never decrease leverage while position exists” policy in the lending program by emitting events and letting LendingPool read CreditScoreAccount during borrow checks.

---

## Program C — `ProtocolHistoryProgram` (optional but recommended)

Purpose: immutable ledger of borrow/repay/liquidation events to compute protocol history cheaply onchain.
**Accounts:**

* `ProtocolHistoryAccount (PDA)` per user — counters and aggregates.

**Key instructions & exact parameters:**

* `RecordLoan(user_pubkey: Pubkey, amount: u64, timestamp: i64)`
* `RecordRepayment(user_pubkey: Pubkey, amount: u64, timestamp: i64)`
* `RecordLiquidation(user_pubkey: Pubkey, penalty: u64, timestamp: i64)`

This program can be merged into LendingPool, but separating improves security/upgradeability.