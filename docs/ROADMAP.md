# ABYSS — Roadmap

This document tracks development phases. Timelines are targets, not commitments.
See the whitepaper (docs/whitepaper/) for the full investor-facing roadmap.

**Primary goal (ADR-0015):** launch the independent ABYSS network with native
coin AC. ERC-20 is not the product; cross-ecosystem compatibility, if needed, is
added later via bridges and wrapped assets — never by replacing native
economics.

---

## Phase 1 — Foundation (current)

**Goal:** working devnet, complete tokenomics model, investor materials.

- [x] Core ledger: Coin, Chain, Block, Transaction, Mempool
- [x] Single-validator devnet with treasury/alice/bob demonstration
- [x] Agent policy enforcement in wallet layer
- [x] abyss-tokenomics: 7-stage sale model, vesting, secondary window, DEX final sale
- [x] abyss-social: post/visibility/view-key/agent-policy data model (14 tests)
- [x] CLI: devnet, tokenomics, vesting, presale, secondary-window, dex-quote, social
- [x] Whitepaper v0.1 (21 pages)
- [x] Public website (abyss-protocol.netlify.app)
- [x] Off-chain allocation receipts (ContributionReceipt), honoured 1:1 at genesis

---

## Phase 2 — Investment & Team

**Goal:** close investor round, assemble core team. No ERC-20 issuance (ADR-0015).

- [ ] Pre-genesis allocation instrument finalised (receipts/escrow, legal-reviewed)
- [ ] Token sale platform live (invest.html → allocation receipts, not a token contract)
- [ ] Investment round completed
- [ ] Core team: protocol engineers, ZK cryptographers, security researchers
- [ ] Independent architecture review

---

## Phase 3 — Testnet

**Goal:** public multi-validator network demonstrating real consensus.

- [ ] Persistent storage backend (RocksDB or similar)
- [ ] JSON-RPC node API
- [ ] Multi-validator BFT consensus (replacing single-proposer devnet)
- [ ] Public testnet (3–5 validator nodes)
- [ ] Block explorer

---

## Phase 4 — Protocol Build

**Goal:** complete the two execution environments and privacy layer.

- [ ] EVM-compatible execution environment
- [ ] ZK circuit layer for shielded transactions (Groth16 + Plonk)
- [ ] Shielded-by-default transfer model
- [ ] Stealth addresses
- [ ] Private mempool
- [ ] Multi-prover verification
- [ ] Independent security audit

---

## Phase 5 — Mainnet

**Goal:** ABYSS chain genesis on the independent network.

- [ ] Mainnet genesis block
- [ ] Pre-genesis allocation receipts redeemed 1:1 for native AC
- [ ] Optional interop: audited bridge + wrapped-AC contracts (post-genesis)
- [ ] AI Agent marketplace launch
- [ ] ABYSS DEX (production)
- [ ] Production wallet application

---

## Phase 6 — Social Layer

**Goal:** censorship-resistant social network on ABYSS chain.

- [ ] Content-addressed, replicated storage backend
- [ ] On-chain identity (wallet address as social identity)
- [ ] Post/reply/repost with shielded authorship option
- [ ] View-key-gated selective disclosure
- [ ] On-chain governance for moderation
- [ ] Agent-curated feeds
- [ ] Creator monetisation via AC and DEX
