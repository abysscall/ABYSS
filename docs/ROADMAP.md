# ABYSS — Roadmap

This document tracks development phases.
Timelines are targets, not commitments.

> **Status labels used below:** MVP (functionally works, tested, scope
> intentionally narrow) → Beta (feature-complete for its stage, needs
> hardening) → Production Ready (audited, economically complete, safe
> for real value). A stage marked MVP is genuinely useful and tested —
> it is not "fake done." It means the next layer can safely build on it,
> while some sub-parts (e.g. slashing economics) remain open.

> **ADR-0015 (July 2026):** ERC-20 is no longer the primary goal.
> The native ABYSS chain and native AC coin are the primary objective.
> See also: Principle of Independence (ADR-0015).

> **ADR-0016 (July 2026):** Sequential implementation discipline.
> 80% effort on implementation, 20% on research, until mainnet.

---

## Phase 1 — Foundation (current)

- [x] Core ledger: Coin, Chain, Block, Transaction, Mempool — **MVP**
- [x] State module extracted from Chain (ADR-0005) — **MVP**
- [x] Single-validator devnet demonstration — **MVP**
- [x] Agent policy enforcement in wallet layer — **MVP**
- [x] abyss-tokenomics: 7-stage sale model, vesting, secondary window — **MVP**
- [x] abyss-social: post/visibility/view-key/agent-policy — **MVP**
- [x] abyss-crypto-adapter: Ed25519 sign/verify — **MVP**
- [x] Whitepaper v0.1 (21 pages)
- [x] Public website
- [x] Full ADR documentation
- [x] cargo-deny + cargo-audit + fuzz testing CI pipeline

---

## Phase 2 — Core Build (ADR-0016 sequential stages)

### Stage 1 — Consensus (BFT) — **MVP, not Production Ready**
- [x] Multi-validator ValidatorSet with weighted voting power
- [x] Round/Phase state machine
- [x] Deterministic leader rotation (round-robin; VRF-based rotation is future work)
- [x] View Change (>1/3 power timeout threshold)
- [x] Slashing **API** (evidence submission, vote rejection for jailed validators)
  — **NOT** economically complete: no bond/unbond/jail/penalty/reward yet.
  See ADR-0019 (State Machine) for the proposed `ValidatorState` shape
  that will carry this economics.
- [x] ConsensusEngine driving the full protocol
- [x] Byzantine fault tolerance tests (4 tests proving core safety properties)
- [x] `Chain::apply_block()` validator-path verification
- **20/20 consensus tests passing**
- **Immediate follow-up (before Stage 2 storage work is considered complete):**
  - [ ] ADR-0017: Consensus ↔ Execution Interface (defines the full
        propose→verify→execute→commit→persist→finalize pipeline)
  - [ ] ADR-0018: Block Header format (adds `version`, `validator_root`,
        `consensus_proof` fields)
  - [ ] ADR-0019: State Machine scope (adds `ValidatorState`,
        `TreasuryState`, placeholder `ContractState`/`GovernanceState`)
  - [ ] Wire `ConsensusEngine::commit()` to actually call
        `Chain::apply_block()` (currently these are separate, untied systems)

### Stage 2 — Storage (RocksDB) — blocked on the three ADRs above
- [ ] Persistent block storage (against the ADR-0018 header format)
- [ ] Persistent state storage (against the ADR-0019 state shape)
- [ ] Transaction indices
- [ ] Account indices
- [ ] Genesis Registry persistence
- [ ] Snapshot/restore from disk

### Stage 3 — Execution Engine
- [ ] `execute_block()` as a named, tested entry point (currently exists
      only as an internal staging step inside `apply_block()`)
- [ ] Fee/gas semantics beyond flat per-tx fee
- [ ] Validator bonding/unbonding/jailing operations (closes the
      Slashing economics gap flagged above)

### Stage 4 — P2P Network
- [ ] Peer discovery, reputation, sync, gossip, anti-spam
- [ ] Real multi-node testnet

### Stage 5 — JSON-RPC
- [ ] Built only after Stages 2–4 have real data to serve —
      not before, to avoid a well-documented API returning empty results.

### Stage 6 — Wallet Core (library)
- [ ] Address generation, signing (via abyss-crypto-adapter), seed recovery

### Stage 7 — Genesis Builder
- [ ] Genesis Allocation Registry (native AC only, per ADR-0015)

### Stage 8 — Explorer (minimal)

### Stage 9 — SDK (Rust, Go, Python, JavaScript)

---

## Phase 3 — Enhancement (after stable mainnet)

Privacy Engine, Shielded VM, On-chain Governance, Decentralised Social
Layer, Quantum-resistant cryptography, Mixnet, FHE/MPC research,
Anonymous staking, Privacy DAO, ABYSS DEX, AI Agent marketplace.

---

## Future — Bridges (additive, not replacement)

Per the Principle of Independence (ADR-0015): bridges and wrapped tokens
are strictly additive and unidirectional from ABYSS. ABYSS never depends
on an external chain for its own operation.
