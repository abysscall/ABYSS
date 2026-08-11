# ABYSS OS — ARCHITECTURE

This document describes the target architecture of ABYSS OS.
It distinguishes between what is already built, what is being designed,
and what is planned for future phases.

The rule: do not break what works. Build upward.

---

## The Architectural Model

```
                        ABYSS OS
                           │
          ┌────────────────┼────────────────┐
          │                │                │
       NATIVE           RUNTIME          PORTABLE
          │                │                │
    Bare Metal     Windows / macOS     External SSD
                       Linux
```

All three modes share the same ABYSS OS core.
Only the hosting and boot mechanism differs.

---

## System Layers

```
                        USER
                          │
                          ▼
                   ABYSS ACCOUNT
                          │
                          ▼
                  ABYSS NATIVE AI
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
          MEMORY        POLICY       WALLET
             │            │            │
             └────────────┼────────────┘
                          ▼
                    ABYSS KERNEL
                          │
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
      CONSENSUS        STORAGE         NETWORK
          │               │               │
          └───────────────┼───────────────┘
                          ▼
                      ABYSS OS
```

---

## Layer Descriptions

### ABYSS Account

The fundamental system entity. Every interaction with ABYSS OS is
mediated by an Account.

Components:

- **Identity** — cryptographic root identity (ABYSS ID)
- **Wallet** — asset management and transaction authority
- **Permissions** — what the account can do and delegate
- **View Keys** — selective disclosure grants to external parties
- **Policy** — spending limits, allowed contracts, automation rules
- **Private Storage** — encrypted data belonging to this account
- **Social Identity** — optional unlinkable personas for social layer
- **Native AI** — the account's system-level AI component
- **Reputation** — cryptographically accumulated trust score

### ABYSS Native AI

A system-level AI component assigned to every Account.
Not an optional add-on. A core process.

Properties:

- operates within a sandboxed execution environment
- has its own resource budget (compute, memory, gas)
- can be given delegated authority over specific operations
- cannot bypass the Policy Engine
- maintains layered memory (working / short-term / long-term / persistent)
- accumulates execution reputation
- can communicate with other AI agents through permitted protocols

The AI hierarchy from most to least privileged:
```
Account Owner (human)
      ↓
Policy Engine
      ↓
Native AI
      ↓
Delegated Agents
      ↓
External AI requests
```

### Policy Engine

The enforcement layer between intent and execution.

Every action passes through it:

```
Human Intent
     ↓
Native AI interpretation
     ↓
Policy Engine check
     ↓
System Call
     ↓
Consensus / Execution
```

Policy Engine checks:

- spending limits (per transaction, per day, per period)
- allowed contract addresses and modules
- allowed recipients
- AI action permissions
- storage access rights
- social publication permissions
- multisig requirements
- delegation validity and expiry

### ABYSS Kernel

The core runtime that coordinates all subsystems.

```
ABYSS Kernel
│
├── Consensus Interface
├── State Machine
├── Security Manager
├── Identity Registry
├── Permission Engine (Policy Engine implementation)
├── Storage Coordinator
├── Network Interface
├── Resource Scheduler (Abyssal Scheduler)
└── AI Runtime Host
```

### Consensus

The existing consensus engine (abyss-consensus crate) becomes the
distributed state synchronisation layer of ABYSS OS.

Validators are distributed execution nodes, not only consensus participants.

### Storage

Three-layer storage model:

```
On-Chain State
    └── commitments, hashes, ownership, metadata, state roots

Distributed Storage
    └── large data: AI models, media, documents, social content

AI Memory
    └── working memory, short-term, long-term, persistent account memory
```

### ABYSS Shell

The user-facing interface layer. Supports multiple interaction modes:

- GUI (graphical desktop or web)
- CLI (command line)
- Natural language (via Native AI)
- Voice (planned)
- API (for developers and external systems)

---

## ABYSS ID

ABYSS ID is the cryptographic identity at the root of every Account.

It is not a public address. It is a structured identity that supports:

- multiple unlinkable personas from one root identity
- selective attribute disclosure via ZK proofs
- view-key grants with expiry
- reputation accumulation without identity exposure

```
Root ABYSS ID
│
├── Payment Identity
├── Social Persona A
├── Social Persona B
├── Trading Persona
├── Agent Delegation Identity
└── Credential Proofs
```

External parties see only what the user explicitly grants them.

---

## Selective Disclosure

The standard model for identity interaction in ABYSS:

```
Alice → Service B:
  ✓ age > 18         (ZK proof, age not revealed)
  ✓ unique human      (proof, identity not revealed)
  ✓ reputation > 4.5  (proof, history not revealed)
  ✗ name
  ✗ balance
  ✗ transaction history
  expires: 7 days
```

---

## Intent-Based Execution

Users express intentions. The system resolves them into system calls.

```
User: "Pay supplier 500 AC when delivery is confirmed."

Native AI
   ↓
Policy Engine check
   ↓
Execution Plan:
  - monitor: delivery_confirmation event
  - on trigger: transfer 500 AC to supplier_address
  - within limits: daily_limit check, allowed_recipients check
   ↓
System Calls scheduled
   ↓
Consensus execution on trigger
```

---

## AI-to-AI Economy

In the extended model, Native AI agents interact with each other:

```
Alice's Native AI
      ↓
requests service (e.g. flight search)
      ↓
Bob's Specialist AI
      ↓
executes task
      ↓
micro-payment in AC (auto-approved within policy budget)
      ↓
result returned to Alice's AI
      ↓
Alice's AI presents result to Alice
```

All interactions are bounded by the Policy Engines of both accounts.

---

## AI Reputation

Every Native AI accumulates an execution reputation:

```
Reliability:      successful task completion rate
Security:         policy violation rate (lower is better)
Efficiency:       resource utilisation vs task outcome
Accuracy:         result quality score
Policy Compliance: adherence to declared permissions
```

Reputation affects execution priority and marketplace visibility.

---

## Abyssal Scheduler

The resource scheduler for AI workloads:

```
States:

RUNNING
  ↓ (resource pressure)
SUSPEND
  ↓ (resources available)
RESUME

or

RUNNING
  ↓ (task complete)
IDLE
  ↓ (new task assigned)
RUNNING
```

AI processes are not killed on resource pressure. They are suspended
with state preserved and resumed when resources are available.

---

## Three Deployment Modes — Detail

### Native Mode

```
Hardware
    ↓
ABYSS Boot / UEFI
    ↓
ABYSS Kernel
    ↓
Consensus / Storage / AI Runtime / Security
    ↓
ABYSS Shell
    ↓
Applications / Agents / Social / Wallet
```

### Runtime Mode

```
Windows / macOS / Linux (host)
    └── ABYSS Runtime Environment
             ├── ABYSS Kernel Runtime
             ├── AI Runtime
             ├── Wallet
             ├── Identity
             ├── Agent Runtime
             ├── Blockchain Node
             └── ABYSS Shell
```

Runtime isolation: sandboxing, containers, or hardware-backed security
where available (TEE / secure enclave). Not dependent on TEE — TEE
is an enhancement, not a requirement.

### Portable Mode

```
External SSD / NVMe
    └── ABYSS OS
         ├── ABYSS Bootloader
         ├── ABYSS Kernel
         └── Full ABYSS Environment
```

Account data is encrypted on the device. Loss of the device does not
mean loss of account — recovery mechanisms exist independent of the
physical storage.

---

## What Is Already Built (Phase 1 Foundation)

The following exists and must not be broken:

- `abyss-core` — chain primitives, AC supply rules, transactions, blocks, genesis, mempool
- `abyss-consensus` — validator set and quorum certificate model
- `abyss-crypto` / `abyss-crypto-api` / `abyss-crypto-adapter` — cryptographic layer
- `abyss-tokenomics` — AC allocation, presale, raise planning
- `abyss-wallet` — wallet accounts and AI-agent permission policy (foundation)
- `abyss-social` — social layer skeleton
- `abyss-node` — CLI node and devnet simulation

All future components build on top of these. They extend interfaces.
They do not rewrite foundations.

---

## Build Sequence

```
Phase 1 (done)     Blockchain foundation
Phase 2            ABYSS Account model
Phase 3            ABYSS Native AI (basic runtime)
Phase 4            Policy Engine (full enforcement layer)
Phase 5            Runtime Mode (Windows / Linux / macOS)
Phase 6            AI Runtime / Abyssal Scheduler / Persistent Memory
Phase 7            Portable Mode
Phase 8            Native Mode (bare metal)
```
