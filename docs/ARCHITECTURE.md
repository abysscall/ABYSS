# ABYSS — Architecture Overview

This document describes the high-level structure of the ABYSS protocol.
For the reasoning behind individual decisions, see the ADR index.

---

## Crate Structure

```
ABYSS/
├── crates/
│   ├── abyss-core          — ledger primitives: Coin, Chain, Block, Transaction, Mempool, State
│   ├── abyss-consensus      — BFT consensus engine (ValidatorSet, Round, ViewChange, Slashing, ConsensusEngine)
│   ├── abyss-crypto         — cryptographic primitives, abstracted behind interfaces
│   ├── abyss-crypto-adapter — Ed25519 signing/verification production adapter
│   ├── abyss-crypto-api     — minimal crypto trait definitions
│   ├── abyss-tokenomics     — token economics, sale rounds, vesting, secondary window
│   ├── abyss-wallet         — wallet accounts, Agent policy model
│   ├── abyss-social         — decentralised social layer data model
│   └── abyss-node           — CLI node, devnet runner, command dispatcher
```

## Dependency Graph

```
abyss-node
  ├── abyss-core
  ├── abyss-consensus
  ├── abyss-tokenomics
  ├── abyss-wallet
  └── abyss-social

abyss-consensus
  └── abyss-core

abyss-wallet
  └── abyss-core

abyss-tokenomics
  └── abyss-core

abyss-social
  (no abyss-core dependency by design — social primitives are storage-agnostic)

abyss-crypto-adapter
  └── abyss-crypto-api

abyss-crypto
  (standalone; abyss-crypto-adapter is the production-grade sibling)
```

Per ADR-0024: none of `abyss-consensus`, `abyss-tokenomics`, `abyss-wallet`,
or `abyss-social` may depend on one another. Each depends only downward on
the Foundation Layer. See "Enforced Architecture Invariants" below.

## Three-Layer Model

```
┌─────────────────────────────────────────┐
│  Application Layer                      │
│  abyss-node CLI · future RPC API        │
├─────────────────────────────────────────┤
│  Protocol Layer                         │
│  abyss-consensus · abyss-tokenomics     │
│  abyss-wallet · abyss-social            │
│  (siblings — see ADR-0024: no lateral   │
│   dependencies between these crates)    │
├─────────────────────────────────────────┤
│  Foundation Layer                       │
│  abyss-core · abyss-crypto              │
└─────────────────────────────────────────┘
```

## Execution Environments (target mainnet)

```
┌─────────────────────────────────────────┐
│  Public Execution (EVM-compatible)      │
│  DEX contracts · Governance · ERC-20   │
├─────────────────────────────────────────┤
│  Private Execution (ZK circuit layer)  │
│  Shielded transfers · Private contracts │
│  Agent-authorised private actions       │
└─────────────────────────────────────────┘
```

Both environments share the same account model and ledger state.
They interact only through narrow, auditable proof interfaces.

## Current Devnet vs Target Mainnet

| Aspect | Current devnet | Target mainnet |
|---|---|---|
| Consensus | Multi-validator BFT (Stage 1 complete, tag `v0.2.0-bft-stage1`) | Same engine, wired to real P2P transport (Stage 3) |
| Storage | In-memory / JSON snapshot (`storage.rs`) | Persistent (RocksDB — Stage 2) |
| Execution | Transparent transfers only | EVM-compatible + ZK circuit (Stage 4 / ADR-0009) |
| Privacy | None | ZK-shielded by default |
| API | CLI only | JSON-RPC node API (Stage 5) |
| Validators | Simulated multi-validator (single process) | Permissionless set over real network |

The gap between these two columns is the mainnet build roadmap.
See ROADMAP.md for phasing.

---

## ADR Immutability Rule

Once an ADR's status is `Accepted`, its architectural decision does not
change in place. If a decision needs to change:

1. Write a **new** ADR with the next available number.
2. In its header, declare `Supersedes: ADR-00XX` (full replacement) or
   `Amends: ADR-00XX` (partial refinement, original mostly still stands).
3. Update the superseded/amended ADR's own header to note
   `Status: Superseded by ADR-00YY` or `Status: Amended by ADR-00YY`,
   but leave its original content untouched.

ADR numbers are never reused, even if a document is later abandoned
before merge. The history of *why* a decision changed is as valuable as
the decision itself.

---

## Enforced Architecture Invariants

Short, scannable rules that must never be violated, each backed by an ADR.
This section states *what* must hold; the linked ADR explains *why*.

1. **Protocol Layer crates do not depend on each other.**
   `abyss-consensus`, `abyss-tokenomics`, `abyss-wallet`, and `abyss-social`
   are siblings, not a dependency chain — only `abyss-node` (Application
   Layer) wires them together. Consensus Blindness (`abyss-consensus`
   never knows what a transaction *means*, only whether it reached
   quorum) is the safety-critical instance of this rule.
   See **ADR-0024**.
   *CI enforcement: not yet implemented — tracked as a Stage 2 prerequisite.*

2. **Block application is atomic.**
   A block's transactions execute against a staged copy of state; if any
   transaction fails, the entire block is discarded and canonical state
   is unchanged. No partial application.
   See **ADR-0004**.
   *Enforced today by `Chain::apply_block()` / `produce_block()`'s
   staged-clone pattern (verified by `block_application_is_atomic` and
   related tests in `abyss-core`).*

3. **Execution is deterministic.**
   State transitions are a pure function of `(State, Block)` — no
   wall-clock reads, no external randomness, no non-deterministic
   iteration order (hence `State` uses `BTreeMap`, never `HashMap`).
   See **ADR-0021**.
   *Enforced today by construction (`BTreeMap` usage); cross-platform
   property-based tests proving identical `state_root` for identical
   block sequences are a tracked follow-up, not yet implemented.*

4. **Cryptographic algorithms are never hard-coded into business logic.**
   All primitives sit behind `abyss-crypto`'s interface; a primitive
   change requires a new ADR that supersedes the relevant section of
   the cryptographic foundation, never a silent swap.
   See **ADR-0006**, **ADR-0022**.
   *Enforced today by the existing `abyss-crypto` / `abyss-crypto-adapter`
   split.*

5. **The `dev_hash` placeholder must never reach a production build.**
   Current consensus-critical hashing in `abyss-core::hashing` is an
   explicitly-labelled development placeholder, not a committed
   cryptographic decision.
   See **ADR-0022**.
   *Status: placeholder still in use. Replacement with BLAKE3 is required
   before Stage 2 (Storage) treats any hash as durable/canonical.*

When adding a new invariant to this list, link the ADR that established
it — this list does not introduce new decisions on its own, it only
indexes existing ones for quick reference.
