# ABYSS Documentation

Welcome to the ABYSS protocol documentation.

## Structure

```
docs/
├── README.md             — this file
├── PHILOSOPHY.md         — why ABYSS exists and what it refuses to become
├── ARCHITECTURE.md       — high-level system overview
├── ROADMAP.md            — development phases and milestones
│
├── adr/                  — Architecture Decision Records
│   ├── ADR-0000.md       — The architecture is the product
│   ├── ADR-0001.md       — Domain types instead of primitives
│   ├── ADR-0002.md       — Immutable domain objects
│   ├── ADR-0003.md       — Separate block header from body
│   ├── ADR-0004.md       — Atomic block application
│   ├── ADR-0005.md       — State as independent module
│   ├── ADR-0006.md       — Replaceable cryptographic algorithms
│   ├── ADR-0007.md       — Architecture before features
│   ├── ADR-0008.md       — Hybrid PoS + BFT consensus
│   ├── ADR-0009.md       — Hybrid VM (EVM-compatible + ZK circuit)
│   ├── ADR-0010.md       — Investor Secondary Window (P2P, no obligation)
│   └── ADR-0011.md       — AI Agent policy as first-class protocol object
│
├── consensus/            — Consensus layer deep-dives
├── crypto/               — Cryptographic primitives and ZK design
├── networking/           — P2P networking design
├── wallet/               — Wallet and Agent policy model
├── tokenomics/           — Token economics and sale structure
└── social/               — Decentralised social layer design
```

## Reading Order

If you are new to the project, read in this order:

1. `PHILOSOPHY.md` — understand *why* before *what*
2. `ARCHITECTURE.md` — understand the system as a whole
3. `adr/ADR-0000.md` — understand the guiding principle for all decisions
4. Individual ADRs as needed for the subsystem you are working on

## Status Legend (used in all ADRs)

| Status | Meaning |
|---|---|
| Accepted | Decision is in effect and reflected in code |
| Proposed | Under discussion, not yet implemented |
| Deprecated | Superseded but kept for historical record |
| Replaced | See replacement ADR for current decision |
