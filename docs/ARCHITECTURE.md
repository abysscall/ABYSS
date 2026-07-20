# ABYSS — Architecture Overview

This document describes the high-level structure of the ABYSS protocol.
For the reasoning behind individual decisions, see the ADR index.

---

## Crate Structure

```
ABYSS/
├── crates/
│   ├── abyss-core          — ledger primitives: Coin, Chain, Block, Transaction, Mempool
│   ├── abyss-consensus     — consensus engine (PoA devnet → Hybrid PoS+BFT mainnet)
│   ├── abyss-crypto        — cryptographic primitives, abstracted behind interfaces
│   ├── abyss-tokenomics    — token economics, sale rounds, vesting, secondary window
│   ├── abyss-wallet        — wallet accounts, Agent policy model
│   ├── abyss-social        — decentralised social layer data model
│   └── abyss-node          — CLI node, devnet runner, command dispatcher
```

## Dependency Graph

```
abyss-node
  ├── abyss-core
  ├── abyss-consensus
  ├── abyss-tokenomics
  ├── abyss-wallet
  └── abyss-social

abyss-wallet
  └── abyss-core

abyss-tokenomics
  └── abyss-core

abyss-social
  (no abyss-core dependency by design — social primitives are storage-agnostic)
```

## Three-Layer Model

```
┌─────────────────────────────────────────┐
│  Application Layer                      │
│  abyss-node CLI · future RPC API        │
├─────────────────────────────────────────┤
│  Protocol Layer                         │
│  abyss-consensus · abyss-tokenomics     │
│  abyss-wallet · abyss-social            │
├─────────────────────────────────────────┤
│  Foundation Layer                       │
│  abyss-core · abyss-crypto              │
└─────────────────────────────────────────┘
```

## Execution Environments (target mainnet)

```
┌─────────────────────────────────────────┐
│  Public Execution (Scalable apps)       │
│  DEX · Governance · Wrapped tokens      │
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
| Consensus | Single proposer (PoA) | Hybrid PoS + BFT |
| Storage | In-memory | Persistent (to be designed) |
| Execution | None (ledger only) | EVM-compatible + ZK circuit |
| Privacy | None | ZK-shielded by default |
| API | CLI only | JSON-RPC node API |
| Validators | 1 | Permissionless set |

The gap between these two columns is the mainnet build roadmap.
See ROADMAP.md for phasing.
