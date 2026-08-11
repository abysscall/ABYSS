# ABYSS OS — ROADMAP

This roadmap reflects the ABYSS OS master vision.
Each phase builds on the previous without breaking it.
Phases are sequential. A phase does not begin until the prior phase
is stable and its interfaces are accepted.

---

## Phase 1 — Foundation (Complete)

**Goal:** establish a working blockchain with correct tokenomics,
consensus, cryptography, wallet, and social skeleton.

Delivered:

- `abyss-core`: chain primitives, AC supply, transactions, blocks, genesis, mempool
- `abyss-consensus`: validator set, quorum certificate, view change, slashing
- `abyss-crypto` / `abyss-crypto-api` / `abyss-crypto-adapter`: crypto layer
- `abyss-tokenomics`: 55M AC supply, sale rounds, raise model
- `abyss-wallet`: account model, AI-agent permission policy foundation
- `abyss-social`: social layer skeleton
- `abyss-node`: CLI node and devnet simulation
- Website: index.html, invest.html, wallet.html, Netlify deployment
- Token sale: 7 rounds defined, treasury wallets set up

Status: **stable — do not break**

---

## Phase 2 — ABYSS Account Model

**Goal:** define and implement the ABYSS Account as a structured
system entity that unifies identity, wallet, permissions, and AI slot.

Key deliverables:

- ABYSS Account data model (extends existing abyss-wallet)
- ABYSS ID cryptographic identity structure
- View-key grant mechanism (expiry, scope, revocation)
- Selective disclosure proof stubs (ZK proofs to be filled in Phase 4+)
- Multiple unlinkable persona support
- Account RFC published and accepted

Dependencies: Phase 1 stable

---

## Phase 3 — ABYSS Native AI (Basic Runtime)

**Goal:** attach a sandboxed AI runtime to every ABYSS Account.
Not a full AI model — a controlled execution environment that can
run approved AI tasks within defined resource limits.

Key deliverables:

- `abyss-ai-runtime` crate (new)
- Sandbox isolation (process or container level)
- Resource budget model: compute, memory, gas per AI instance
- Basic task queue and execution loop
- AI state persistence (working memory + short-term memory)
- Integration with existing `abyss-wallet` agent policy

Dependencies: Phase 2 stable

---

## Phase 4 — Policy Engine

**Goal:** implement the full Policy Engine as the enforcement layer
between user intent and system execution.

Key deliverables:

- `abyss-policy` crate (new)
- Spending limits (per transaction, daily, periodic)
- Allowed contracts and recipients list
- AI permission grants and revocations
- Delegation model: human → AI → sub-agent
- Intent parsing: human language intent → policy-checked system call
- Full integration with abyss-node system call path
- RFC-0002 (Policy Engine) published and implemented

Dependencies: Phase 3 stable

---

## Phase 5 — Runtime Mode

**Goal:** make ABYSS OS available to users on Windows, macOS, and Linux
without requiring them to replace their operating system.

Key deliverables:

- ABYSS Runtime installer (Windows primary, Linux secondary, macOS tertiary)
- Isolated execution environment (sandbox or container)
- ABYSS Shell: GUI + CLI
- Local blockchain node integration
- Wallet and identity UI
- AI runtime within the isolated environment
- TEE / secure enclave integration where hardware supports it
- Fallback to cryptographic sandboxing where TEE unavailable

Dependencies: Phase 4 stable

---

## Phase 6 — AI Runtime / Abyssal Scheduler / Persistent Memory

**Goal:** extend the Native AI runtime with full scheduling,
persistent memory layers, and execution reputation.

Key deliverables:

- Abyssal Scheduler: RUNNING / SUSPEND / RESUME state machine
- Multi-layer AI memory: working / short-term / long-term / persistent
- AI execution reputation accumulation
- AI-to-AI communication protocol (within permission model)
- AI Marketplace foundation (specialised AI modules purchasable with AC)
- Distributed storage layer integration for large AI data

Dependencies: Phase 5 stable

---

## Phase 7 — Portable Mode

**Goal:** ABYSS OS bootable from an external SSD or NVMe device.

Key deliverables:

- ABYSS bootloader
- Bootable OS image for x86_64 and ARM64
- Encrypted account storage on device
- Secure boot chain
- Account recovery mechanism independent of physical device
- Optional passphrase protection
- Remote wipe capability

Dependencies: Phase 6 stable

---

## Phase 8 — Native Mode

**Goal:** ABYSS OS as a full operating system on bare-metal hardware.

Key deliverables:

- ABYSS Kernel (native)
- Hardware driver layer (or Linux kernel base with ABYSS OS layer above)
- Full ABYSS Shell as native desktop
- All Phase 1–7 components running natively
- Specialised hardware support roadmap

Dependencies: Phase 7 stable

---

## Long-Term Horizon (Post Phase 8)

These items are on the vision horizon. They require earlier phases
to be stable before design begins.

- **Private DAO Voting** — verifiable results without revealing individual votes
- **ABYSS Human Proof** — ZK proof of unique humanity without identity exposure
- **Shadow Transactions** — privacy-preserving delayed execution scheduling
- **Private Smart Contracts** — encrypted contract state with public commitments
- **ABYSS Safe** — privacy-preserving M-of-N threshold signature wallets
- **Encrypted Social Monetisation** — micro-payment gated content
- **Autonomous AI Economy** — AI-to-AI service market with reputation and payment
- **ABYSS URI Standard** — unified address format: `abyss://account/<id>/...`
- **Treasury DAO** — on-chain governance for ecosystem resource allocation

---

## Token Sale Alignment with Roadmap

| Sale Stage              | AC       | Price | Funds directed toward               |
|-------------------------|----------|-------|-------------------------------------|
| Investors               | 2,000,000| $1.00 | Phase 2 + 3 design and team         |
| Pre-Sale                | 3,000,000| $2.00 | Phase 3 + 4 implementation          |
| Sale Stage 1            | 5,000,000| $3.00 | Phase 4 + 5 Runtime Mode            |
| Sale Stage 2            | 5,000,000| $4.00 | Phase 5 + 6 AI Runtime              |
| Sale Stage 3            |10,000,000| $5.00 | Phase 6 + 7 Portable Mode           |
| Final Sale (DEX)        | variable | market| Ecosystem and Phase 8 preparation   |

---

## Principles That Govern This Roadmap

1. Do not begin a phase until the prior phase is stable.
2. Do not break existing components to build new ones.
3. Define interfaces before writing implementation.
4. Ship working software at each phase boundary.
5. The roadmap changes. The doctrine does not.
