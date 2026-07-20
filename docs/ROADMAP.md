# ABYSS — Roadmap

This document tracks development phases.
Timelines are targets, not commitments.

> **ADR-0015 (July 2026):** Native chain is the primary goal.
> The native ABYSS chain and native AC coin are the primary objective.
> External compatibility will be delivered through bridges and wrapped tokens
> where needed — not as the canonical form of AC.

---

## Phase 1 — Foundation (current)

**Goal:** working devnet, complete tokenomics model, investor materials.

- [x] Core ledger: Coin, Chain, Block, Transaction, Mempool
- [x] Single-validator devnet with treasury/alice/bob demonstration
- [x] Agent policy enforcement in wallet layer
- [x] abyss-tokenomics: 7-stage sale model, vesting, secondary window
- [x] abyss-social: post/visibility/view-key/agent-policy (14 tests)
- [x] CLI: devnet, tokenomics, vesting, presale, secondary-window, social
- [x] Whitepaper v0.1 (21 pages)
- [x] Public website (abyss-protocol.netlify.app)
- [x] Full ADR documentation (ADR-0000 through ADR-0015)

---

## Phase 2 — Investment & Team

**Goal:** close investor round, assemble core team.

- [ ] Investment round completed (off-chain signed agreements + registry)
- [ ] Core team: protocol engineers, ZK cryptographers, security researchers
- [ ] Independent architecture review
- [ ] Whitepaper v1.0 (updated to reflect native-first model per ADR-0015)

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
- [ ] Multi-prover verification (ADR-0009)
- [ ] Independent security audit

---

## Phase 5 — Mainnet

**Goal:** ABYSS chain genesis. Native AC is the canonical asset.

- [ ] Mainnet genesis block
- [ ] Native AC coin live — no migration needed
- [ ] Investor allocations fulfilled in native AC at genesis
- [ ] AI Agent marketplace launch
- [ ] ABYSS DEX (production, native AC pairs)
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
- [ ] Creator monetisation via native AC

---

## Future — Bridges (additive, not replacement)

When cross-chain compatibility becomes a product requirement:

- [ ] Wrapped AC (wAC) — represents locked native AC on external chains
- [ ] Bridge operator or decentralised bridge protocol
- [ ] Bridge contracts on EVM chains (Ethereum, BSC, etc.) — optional, additive only

This is strictly additive. wAC extends ABYSS's reach
without replacing its native economy.
