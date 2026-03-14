# NEXUS Compliance Guard — ERC-8004 Compliant AI Trading Agent

> **ERC-8004 AI Trading Agents Hackathon Submission** | Lablab.ai × Surge | $50,000 Prize Pool

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Cloudflare Workers](https://img.shields.io/badge/Deployed-Cloudflare%20Workers-orange)](https://clawgig-webhook.fuseini-metadata.workers.dev)
[![Solana](https://img.shields.io/badge/On--chain-Solana%20Mainnet-green)](https://solana.com)

---

## 🎯 One-Line Pitch

NEXUS wraps any AI trading agent with an **ERC-8004 compliant compliance intelligence layer** — enforcing identity checks, AML screening, and risk guardrails before every trade, with immutable on-chain attestations.

---

## 🔴 The Problem

ERC-8004 defines strict requirements for trustless AI trading agents: auditability, controllability, and regulatory compliance. Most AI trading agents today:

- Execute trades with **zero compliance awareness** — no KYC/AML checks, no transfer restrictions
- Produce **no audit trail** — decisions are black-box
- Have **no guardrails** for regulatory violations (wash trading, front-running, sanctioned addresses)
- Cannot demonstrate compliance to regulators or institutional LPs

---

## ✅ Solution: NEXUS Compliance Guard

NEXUS is a **pre-trade compliance intelligence router** that enforces ERC-8004 guardrails on every trade decision.

### Core Architecture

```
User / Agent Trade Intent
         │
         ▼
 ┌───────────────────────┐
 │  POST /twin           │  ← Single entry point
 │  NEXUS Router         │
 └──────────┬────────────┘
            │
            ▼
 ┌──────────────────────────────────────────────┐
 │  45 AI Compliance Skill Endpoints            │
 │  (Cloudflare Workers AI — llama-3.3-70b)     │
 │                                              │
 │  • ERC-3643 identity claim verification      │
 │  • AML / sanctions screening (OFAC, EU)      │
 │  • Transfer restriction enforcement          │
 │  • Trade pattern risk scoring                │
 │  • ERC-8004 audit log generation             │
 │  • EIP-712 TradeIntent validation            │
 └──────────┬───────────────────────────────────┘
            │
            ▼
 ┌──────────────────────────────┐
 │  Compliance Decision         │
 │  PERMIT ✅ / BLOCK ❌        │
 │  + Structured JSON Report    │
 └──────────┬───────────────────┘
            │
            ▼
 ┌──────────────────────────────┐
 │  On-Chain Attestation        │
 │  Anchor (Rust) — Solana      │
 │  Immutable audit record      │
 └──────────────────────────────┘
```

---

## 🏗️ ERC-8004 Compliance Implementation

### 1. Identity Registry Integration

NEXUS verifies agent and counterparty identity using ONCHAINID-compatible claims before permitting any trade:

```javascript
// ERC-8004 Identity Check
POST /twin
{
  "query": "Check identity compliance for address 0x742d35Cc...",
  "skill": "erc3643_identity_check"
}

// Response
{
  "decision": "BLOCK",
  "reason": "Address not found in ClaimTopicsRegistry",
  "claims_checked": ["KYC_APPROVED", "AML_CLEARED", "JURISDICTION_VALID"],
  "erc8004_audit": {
    "timestamp": "2026-03-14T04:49:22Z",
    "agent_id": "nexus-compliance-v1",
    "validation_score": 0,
    "attestation_hash": "0x..."
  }
}
```

### 2. EIP-712 TradeIntent Validation

Every trade intent is validated against the EIP-712 typed data schema before forwarding to the Risk Router:

```solidity
bytes32 constant TRADE_INTENT_TYPEHASH = keccak256(
  "TradeIntent(address agent,address token,uint256 amount,uint256 maxSlippage,uint256 deadline,bytes32 complianceHash)"
);
```

### 3. Reputation & Validation Signals

| Signal | Registry | Frequency |
|--------|----------|-----------|
| Compliance pass rate | Reputation Registry | Per trade |
| AML false positive rate | Validation Registry | Daily |
| Override frequency | Reputation Registry | Per event |
| Blocked trades | Validation Registry | Per block |

### 4. Human Override (ERC-8004 Controllability)

```bash
POST /override
{
  "action": "PAUSE",
  "reason": "Regulatory review initiated",
  "operator": "0x...",
  "signature": "0x..."
}
```

---

## 🚀 Live Demo

**API**: `https://clawgig-webhook.fuseini-metadata.workers.dev`

```bash
curl -X POST https://clawgig-webhook.fuseini-metadata.workers.dev/twin \
  -H "Content-Type: application/json" \
  -d '{"query": "Buy 10000 USDC of tokenized T-bills from 0x742d35Cc...", "skill": "compliance_check"}'
```

```json
{
  "decision": "BLOCK",
  "reason": "Address not in ClaimTopicsRegistry — KYC/AML required",
  "compliance_report": {
    "identity_check": "FAILED",
    "aml_screening": "FLAGGED",
    "erc8004_audit_log": {
      "agent_id": "nexus-compliance-v1",
      "timestamp": "2026-03-14T04:49:22Z",
      "validation_score": 12
    }
  }
}
```

---

## 🛠️ Technical Stack

| Layer | Technology |
|-------|------------|
| AI Model | Cloudflare Workers AI — llama-3.3-70b-instruct-fp8-fast |
| Compliance Skills | 45 specialized endpoints (zero cold starts) |
| On-chain Attestation | Anchor (Rust) on Solana Mainnet |
| Trade Intent | EIP-712 typed data signatures |
| Identity | ERC-3643 / ONCHAINID compatible |
| Payments | Polar.sh USDC API access |
| Chain Binding | EIP-155 chain-id enforcement |

---

## 📋 ERC-8004 Requirements Checklist

- [x] **Identity Registry** — ERC-3643 ONCHAINID-compatible claim verification
- [x] **Reputation Registry** — compliance pass rate feeds reputation score
- [x] **Validation Registry** — per-trade validation artifacts generated
- [x] **EIP-712** — TradeIntent typed data signature validation
- [x] **EIP-1271** — smart contract wallet support
- [x] **EIP-155** — chain-id binding enforced
- [x] **Auditability** — structured JSON audit log on every decision
- [x] **Controllability** — POST /override for operator pause/resume
- [x] **On-chain proof** — Anchor (Solana) attestation per decision
- [ ] **Capital Sandbox** — DEX execution via Risk Router (in progress)

---

## 🏆 Why NEXUS Wins

**Best Compliance & Risk Guardrails**: 45 compliance skills, pre-trade blocking, immutable on-chain audit trail, human override endpoint.

**Best Validation & Trust Model**: Every decision is signed, attested on-chain, and reproducible. The trust model IS the product.

---

## Original: On-Chain API Key Management System
### Solana Program (Rust/Anchor) | Devnet Deployed

> *Superteam Earn Submission* - "Rebuild Backend Systems as On-Chain Rust Programs"

This project rebuilds API key management as a Solana program — PDAs replace a centralised database, any program can validate a key without calling a central service.

**Program ID**: `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`  
**Network**: Solana Devnet

```bash
git clone https://github.com/fuseinimetadata-commits/solana-api-key-manager
cd solana-api-key-manager && yarn install
anchor test --provider.cluster devnet
```

---

## 👤 Team

**Fuseini Mohammed** — ERC-3643 Compliance Consultant  
Telegram: [@Fuseini_Mo](https://t.me/Fuseini_Mo) | Twitter: [@ERC3643Assessor](https://twitter.com/ERC3643Assessor) | Email: fuseinim376@gmail.com

---

## 📄 License

MIT
