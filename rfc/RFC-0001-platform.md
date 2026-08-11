# RFC-0001 — ABYSS OS Platform Architecture

**Status:** Proposed
**Created:** 2026-08-11
**Authors:** ABYSS Core Team

---

## Summary

This RFC defines the platform architecture of ABYSS OS — the layered
system that connects the existing blockchain foundation to the ABYSS
Account model, Native AI runtime, Policy Engine, and the three
deployment modes (Native, Runtime, Portable).

---

## Motivation

The existing ABYSS foundation (Phase 1) delivers a working blockchain
with consensus, tokenomics, wallet primitives, and a social skeleton.

To evolve toward ABYSS OS, we need a documented platform architecture
that specifies:

- how existing crates map to OS-layer concepts
- what new components must be introduced
- what the interface boundaries are between layers
- what the build sequence is

Without this specification, Phase 2+ development risks:
- inconsistent interface assumptions between crates
- duplicated logic across layers
- breaking changes to working foundation code
- inability to onboard new contributors

---

## The Platform Stack

```
User
 │
 ▼
ABYSS Shell (CLI / GUI / Natural Language)
 │
 ▼
ABYSS Account
 ├── ABYSS ID (identity)
 ├── Wallet (asset management)
 ├── Permissions (view keys, grants)
 ├── Policy (spending rules, AI limits)
 ├── Private Storage
 └── Native AI slot
 │
 ▼
Policy Engine
 │
 ▼
ABYSS Kernel
 ├── Consensus Interface → abyss-consensus
 ├── State Machine → abyss-core
 ├── Identity Registry → (new: abyss-identity)
 ├── Permission Engine → (new: abyss-policy)
 ├── Storage Coordinator → (new: abyss-storage)
 ├── Network Interface → (existing + new)
 ├── Resource Scheduler → (new: abyss-scheduler)
 └── AI Runtime Host → (new: abyss-ai-runtime)
 │
 ▼
Consensus / Storage / Network
(existing abyss-consensus, abyss-core, abyss-node)
```

---

## Existing Crates — Platform Mapping

| Crate | Current role | OS layer role |
|---|---|---|
| `abyss-core` | Chain primitives, AC supply, mempool | State Machine, Transaction layer |
| `abyss-consensus` | Validator set, quorum, view change | Consensus Interface, Distributed Kernel |
| `abyss-crypto` | Dev signing keys | Cryptographic substrate (to be replaced by adapter) |
| `abyss-crypto-api` | Signing trait interface | Crypto API contract |
| `abyss-crypto-adapter` | Production signing | Production Crypto layer |
| `abyss-tokenomics` | AC allocation, sale rounds | Economic kernel constants |
| `abyss-wallet` | Account model, agent policy | ABYSS Account foundation (to be extended) |
| `abyss-social` | Social layer skeleton | Social Interface layer |
| `abyss-node` | CLI, devnet, presale quote | ABYSS Shell (CLI) foundation |

---

## New Crates Required

### `abyss-identity`

Implements ABYSS ID:
- root identity structure
- view-key generation and verification
- selective disclosure proof stubs
- multiple unlinkable persona support

Interface boundary: `abyss-wallet` (extends Account with identity),
`abyss-policy` (identity used in policy evaluation)

### `abyss-policy`

Implements the Policy Engine:
- spending limit enforcement
- allowed recipients and contracts
- AI permission grants and revocations
- delegation model
- intent → system call translation

Interface boundary: all system call paths must pass through this crate.

### `abyss-ai-runtime`

Implements Native AI runtime:
- sandbox environment per account
- resource budget tracking (compute, memory, gas)
- task queue and execution loop
- AI state persistence (memory layers)
- reputation accumulation

Interface boundary: `abyss-policy` (all AI actions pass through Policy),
`abyss-wallet` (AI operates on behalf of Account)

### `abyss-storage`

Implements the three-layer storage model:
- on-chain state coordination (via abyss-core)
- distributed storage interface
- AI memory persistence

Interface boundary: `abyss-ai-runtime` (AI memory), `abyss-wallet` (account storage)

### `abyss-scheduler`

Implements the Abyssal Scheduler:
- AI process state machine (RUNNING / SUSPEND / RESUME / IDLE)
- resource accounting
- priority management

Interface boundary: `abyss-ai-runtime` (manages AI processes)

---

## Deployment Mode Architecture

### Runtime Mode (Phase 5 priority)

The ABYSS OS Runtime packages the following into an installable
application for Windows, macOS and Linux:

```
ABYSS Runtime Package
├── ABYSS Kernel (runtime subset)
├── abyss-node (local or light node)
├── abyss-wallet + abyss-identity + abyss-policy
├── abyss-ai-runtime + abyss-scheduler
├── ABYSS Shell (GUI + CLI)
└── Isolation layer (sandbox / container / TEE where available)
```

The host OS provides hardware access. ABYSS Runtime runs in an isolated
environment above the host OS.

### Portable Mode (Phase 7)

ABYSS OS image on external storage:

```
External SSD / NVMe
└── ABYSS Bootloader
     └── ABYSS Kernel (full)
          └── All OS components
```

Portable Mode uses the same component set as Runtime Mode but
boots directly rather than running within a host OS.

### Native Mode (Phase 8)

ABYSS Kernel runs directly on hardware. Defines its own hardware
abstraction layer. Full OS environment.

---

## System Call Model

ABYSS OS defines a set of system calls as the interface between
the ABYSS Account layer and the Kernel layer.

Initial system call set:

```
transfer(recipient, amount, asset)      — asset transfer
store(key, value, visibility)           — storage write
publish(content, visibility, price)     — social publication
execute(module, params)                 — module execution
delegate(agent_id, permissions, expiry) — AI delegation
grant(party, attributes, expiry)        — view-key grant
revoke(grant_id)                        — revoke a grant
schedule(intent, trigger, params)       — scheduled/delayed execution
query(target, params)                   — read-only query
```

All system calls must pass through the Policy Engine before execution.

---

## ABYSS URI Standard (Proposed)

Unified addressing format for ABYSS OS resources:

```
abyss://account/<id>
abyss://account/<id>/wallet
abyss://account/<id>/social/posts
abyss://account/<id>/storage/<key>
abyss://agent/<id>/state
abyss://agent/<id>/memory/long-term
abyss://module/<id>
```

This standard must be formalised in a separate RFC before implementation.

---

## What This RFC Does Not Define

- Specific ZK proof system (separate ADR required)
- AI model selection or training details
- Specific consensus changes (existing consensus is used as-is)
- Governance model (separate RFC)
- Cross-chain bridge design (long-term horizon)

---

## Open Questions

1. Should `abyss-identity` extend `abyss-wallet` or be a peer crate?
2. What is the minimal Policy Engine implementation for Phase 4?
3. What isolation technology is used for Runtime Mode on Windows?
4. How does AI memory persist across Runtime restarts?

These questions require ADR resolution before Phase 2 implementation begins.

---

## Acceptance Criteria

This RFC is accepted when:

- [ ] Interface boundaries between all crates are documented and agreed
- [ ] New crate names and responsibility boundaries are confirmed
- [ ] System call set is reviewed and approved
- [ ] Phase 2 implementation can begin without this RFC being modified
