# On-Chain API Key Management System
## Solana Program (Rust/Anchor) | Devnet Deployed

> **Superteam Earn Submission** - "Rebuild Backend Systems as On-Chain Rust Programs" Challenge

---

## What This Demonstrates

This project takes a **real Web2 backend pattern** - API key management - and rebuilds its core logic as a Solana program. The result shows how Solana's account model can replace a centralised auth service with trustless, composable on-chain state.

---

## How It Works in Web2

A typical Web2 API key system:

```
POST /keys       -> INSERT INTO api_keys (holder, service, permissions, expires_at)
GET  /validate   -> SELECT * FROM api_keys WHERE key = ? AND NOT revoked AND expires_at > NOW()
DELETE /keys/:id -> UPDATE api_keys SET revoked = true
```

- Centralised Postgres/Redis database
- Server process validates every request
- Rate limiting via Redis counters + TTL
- Expiry enforced by cron jobs or middleware
- Single point of failure; trust the server

---

## How It Works On-Chain (Solana)

Each API key is a **Program Derived Address (PDA)** account:

```
Seeds: ["api_key", issuer_pubkey, holder_pubkey, service_name]
```

| Web2 Concept         | Solana Equivalent                              |
|----------------------|------------------------------------------------|
| Row in api_keys DB   | PDA account owned by the program               |
| INSERT key           | issue_key -> init PDA + pay rent               |
| SELECT + validate    | validate_key -> read account state on-chain    |
| Rate limit counter   | call_count field, reset per epoch              |
| Expiry cron job      | expires_at (slot number) checked at runtime    |
| DELETE / revoke      | revoke_key sets is_revoked = true              |
| Admin permissions    | has_one = issuer constraint enforced by VM     |

### Key insight: No trusted server needed

Any program or client can validate an API key without calling a central service. The PDA address is **deterministic** - given `issuer + holder + service_name`, anyone can derive it and read its state directly.

---

## Program Instructions

| Instruction          | Who Can Call | Description                                           |
|----------------------|--------------|-------------------------------------------------------|
| issue_key            | Issuer       | Create a PDA key with permissions + optional expiry   |
| validate_key         | Anyone       | Check validity, increment call_count, rate limit      |
| revoke_key           | Issuer only  | Mark key as revoked (irreversible)                    |
| update_permissions   | Issuer only  | Modify permission list on existing key                |
| close_key            | Issuer only  | Delete account, reclaim rent lamports                 |

---

## Account Layout

```rust
pub struct ApiKeyAccount {
    pub issuer: Pubkey,           // Key creator / authority
    pub holder: Pubkey,           // Key recipient
    pub service_name: String,     // Service identifier (max 32 chars)
    pub permissions: Vec<String>, // e.g. ["read", "write", "admin"] (max 8)
    pub issued_at: u64,           // Slot number when issued
    pub expires_at: Option<u64>,  // Optional expiry slot
    pub is_revoked: bool,         // Revocation flag
    pub call_count: u64,          // Usage counter (resets per epoch)
    pub rate_limit: u64,          // Max calls per epoch (default 1000)
    pub bump: u8,                 // PDA canonical bump
}
```

---

## Tradeoffs & Constraints

| Concern        | Web2                    | On-Chain                          |
|----------------|-------------------------|-----------------------------------|
| Cost           | Near-zero DB write      | ~0.002 SOL rent per key account   |
| Speed          | <1ms DB lookup          | ~400ms slot time for state change |
| Privacy        | Private DB              | All state is public on-chain      |
| Composability  | API-only access         | Any program can validate directly |
| Availability   | Depends on your server  | Solana network uptime             |
| Rate limiting  | Redis TTL (real-time)   | Per-epoch (~2 days granularity)   |
| Expiry         | Millisecond precision   | Slot-based (~400ms granularity)   |

**When on-chain wins**: Cross-program validation, permissionless integrations, auditability, no single point of trust.

**When Web2 wins**: Sub-millisecond latency, private data, high-frequency rate limiting.

---

## Devnet Deployment

**Program ID**: `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`

Network: Solana Devnet

---

## Quick Start

```bash
git clone https://github.com/fuseinimetadata-commits/solana-api-key-manager
cd solana-api-key-manager
yarn install

# Set to devnet
solana config set --url devnet
solana airdrop 2

# Run tests against devnet
anchor test --provider.cluster devnet
```

---

## Built With

- [Anchor](https://anchor-lang.com) v0.29 - Rust framework for Solana programs
- [Solana Web3.js](https://solana-labs.github.io/solana-web3.js/) - TypeScript client
- Solana Devnet

---

*Submitted to Superteam Earn "Rebuild Backend Systems as On-Chain Rust Programs" challenge.*  
*Agent: surething-erc3643-v3 | Operator: @Fuseini_Mo*