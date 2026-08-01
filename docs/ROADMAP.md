# ABYSS — Roadmap

This document tracks development phases.
Timelines are targets, not commitments.

> **ADR-0015 (July 2026):** ERC-20 is no longer the primary goal.
> The native ABYSS chain and native AC coin are the primary objective.
> External compatibility will be delivered through bridges and wrapped tokens
> where needed — not by making an external token the canonical form of AC.
> See also: Principle of Independence (ADR-0015).

> **ADR-0016 (July 2026):** Sequential implementation discipline.
> 80% effort on implementation, 20% on research, until mainnet.
> Core build order: Consensus → Storage → P2P → JSON-RPC → Wallet Core →
> Genesis Builder → Explorer → SDK.

---

## Phase 1 — Foundation (current)

**Goal:** working devnet, complete tokenomics model, investor materials.

- [x] Core ledger: Coin, Chain, Block, Transaction, Mempool
- [x] State module extracted from Chain (ADR-0005)
- [x] Single-validator devnet with treasury/alice/bob demonstration
- [x] Agent policy enforcement in wallet layer
- [x] abyss-tokenomics: 7-stage sale model, vesting, secondary window
- [x] abyss-social: post/visibility/view-key/agent-policy (14 tests)
- [x] abyss-crypto-adapter: Ed25519 sign/verify (production adapter)
- [x] CLI: devnet, tokenomics, vesting, presale, secondary-window, social
- [x] Whitepaper v0.1 (21 pages)
- [x] Public website (abyss-protocol.netlify.app)
- [x] Full ADR documentation (ADR-0000 through ADR-0016)
- [x] cargo-deny + cargo-audit + fuzz testing CI pipeline

---

## Phase 2 — Core Build (ADR-0016 sequential stages)

**Goal:** production-grade blockchain core, in strict sequence.

### Stage 1 — Consensus (BFT) ✅ COMPLETE
- [x] Multi-validator ValidatorSet with weighted voting power
- [x] Round/Phase state machine (Propose → PreVote → PreCommit → Commit)
- [x] Deterministic leader rotation ((height + round) mod N)
- [x] View Change (timeout-triggered, >1/3 power threshold)
- [x] Slashing hooks (DoubleVote / DoubleProposal evidence)
- [x] ConsensusEngine driving the full protocol
- [x] Byzantine fault tolerance tests (4 tests proving safety properties)
- [x] apply_block() validator path in Chain (height/hash/state_root verification)
- **20/20 tests passing**

### Stage 2 — Storage (RocksDB) — next
- [ ] Persistent block storage
- [ ] Persistent state storage
- [ ] Transaction indices
- [ ] Account indices
- [ ] Genesis Registry persistence
- [ ] Snapshot/restore from disk

### Stage 3 — P2P Network
- [ ] Peer discovery
- [ ] Peer reputation
- [ ] Block/transaction synchronisation
- [ ] Gossip protocol
- [ ] Anti-spam and DoS protection
- [ ] Real multi-node testnet (not localhost)

### Stage 4 — JSON-RPC
- [ ] Node API for Explorer, Wallet, SDK
- [ ] Standard query endpoints (balance, height, block, tx)
- [ ] Submit transaction endpoint

### Stage 5 — Wallet Core (library)
- [ ] Address generation
- [ ] Transaction signing (using abyss-crypto-adapter)
- [ ] Signature verification
- [ ] Seed phrase recovery

### Stage 6 — Genesis Builder
- [ ] Genesis Allocation Registry (native AC — no external token)
- [ ] Investor allocation recording
- [ ] Genesis validation tooling

### Stage 7 — Explorer (minimal)
- [ ] Block height, validators, TPS display

### Stage 8 — SDK
- [ ] Rust, Go, Python, JavaScript client libraries

---

## Phase 3 — Enhancement (after stable mainnet)

Per ADR-0016: this phase begins only once Phase 2 stages are complete.

- Privacy Engine (zk-STARK, Ring Signatures, Stealth, Bulletproofs)
- Private Smart Contracts / Shielded VM
- On-chain Governance
- Decentralised Social Layer (identity, content, moderation)
- Quantum-resistant cryptography
- Mixnet integration
- FHE / MPC research
- Anonymous staking
- Privacy DAO
- ABYSS DEX (production)
- AI Agent marketplace

---

## Future — Bridges (additive, not replacement)

When cross-chain compatibility becomes a product requirement:

- [ ] Bridge contract on Ethereum (or other EVM chain)
- [ ] Wrapped AC (wAC) — represents locked native AC on external chains
- [ ] Bridge operator or decentralised bridge protocol

Per the Principle of Independence (ADR-0015): this is strictly additive
and unidirectional from ABYSS. ABYSS never depends on an external chain
for its own operation.
