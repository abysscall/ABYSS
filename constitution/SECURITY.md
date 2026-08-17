# ABYSS OS — SECURITY

Security in ABYSS is not a layer added on top of the system.
It is a property of the architecture itself.

This document defines the security model, threat assumptions,
and non-negotiable security principles of ABYSS OS.

---

## Core Security Principle

> **AI is powerful but never sovereign.**

The user is the owner. The AI is the instrument.
The Policy Engine is the law.

No component, performance optimisation, or external request can bypass
this hierarchy.

---

## Trust Hierarchy

```
Account Owner (human, cryptographic key)
        │
        ▼ (sets rules)
  Policy Engine (enforces rules)
        │
        ▼ (operates within rules)
  Native AI (executes tasks)
        │
        ▼ (receives delegated authority)
  Sub-Agents / External AI
        │
        ▼ (lowest trust)
  External Requests
```

Trust flows downward. Privilege cannot be escalated upward.

An AI agent cannot grant itself more permissions than the account owner
has granted to it. A sub-agent cannot exceed the permissions of the
agent that delegated to it.

---

## Threat Model

### Threats ABYSS Is Designed to Resist

**Identity theft**
An attacker cannot impersonate a user's ABYSS Account without the
user's private key. View keys are scoped and time-limited.

**Data surveillance**
All account state, transactions, AI memory, and social data are private
by default. An observer learns nothing about a user's activity unless
the user has explicitly issued a view-key grant.

**AI takeover**
A Native AI cannot exceed its policy-defined permissions regardless of
what instructions it receives. External commands to an AI that would
violate the Policy Engine are rejected at the enforcement layer.

**Sybil attacks**
The ABYSS Human Proof mechanism (ZK-based proof of unique humanity)
provides anti-Sybil protection without requiring identity disclosure.

**Supply manipulation**
The 55,000,000 AC hard cap is a protocol constant, not a governance
parameter. No vote can change it.

**Validator collusion**
The consensus layer requires a qualified supermajority. A minority of
malicious validators cannot alter state or forge transactions.

**Physical device loss (Portable Mode)**
Account data on external storage is encrypted. Loss of the device does
not mean loss of the account. Recovery is possible through the account's
cryptographic recovery mechanism.

### Threats Outside ABYSS's Scope

**Compromised private key**
If the user's root private key is stolen, the attacker controls the
account. ABYSS cannot protect against this. Users must protect their
keys using hardware wallets (Trezor Model T recommended for treasury).

**Compromised host OS (Runtime Mode)**
If the host operating system (Windows, macOS, Linux) is fully compromised
at kernel level, the isolation of the ABYSS Runtime is degraded. TEE
and hardware-backed security reduce this risk but do not eliminate it.

**Zero-day vulnerabilities in cryptographic libraries**
ABYSS uses audited cryptographic primitives. Unknown vulnerabilities
in those libraries are outside the control of the ABYSS architecture.

---

## Policy Engine Security Properties

The Policy Engine is the security boundary between user intent and
system execution. Its properties must hold at all times:

1. **Completeness** — every system call passes through the Policy Engine.
   There are no bypass paths.

2. **Tamper resistance** — the Policy Engine cannot be modified by the
   Native AI or by external requests. Only the account owner can change
   their policy, and only with their private key.

3. **Atomicity** — a policy check either passes entirely or fails entirely.
   Partial enforcement is not acceptable.

4. **Auditability** — every Policy Engine decision is logged in a way the
   account owner can inspect.

---

## AI Sandbox Security Properties

Every Native AI operates inside a sandbox with the following guarantees:

- **Memory isolation** — the AI's memory space is isolated from other
  accounts and from the host system.
- **Capability confinement** — the AI can only call system functions
  explicitly listed in its permission set.
- **Resource limits** — compute, memory, and gas budgets are enforced
  at the runtime level. An AI cannot consume resources beyond its budget.
- **No self-modification of permissions** — an AI cannot modify its own
  permission set or budget.
- **No access to other accounts** — an AI cannot read or write state
  belonging to another account without that account's explicit grant.

---

## Cryptographic Foundations

**Current (development):**
- Ed25519 for signing (via `abyss-crypto-adapter` with ed25519-dalek)
- SHA-256 for hashing (via `abyss-core`)
- Development-only key generation in `abyss-crypto`

**Production requirements (before mainnet):**
- Production signing keys must use hardware wallet integration or
  audited HSM-backed generation
- ZK proof system must be selected and audited (target: Groth16 or PLONK
  for efficiency; exact choice via ADR process)
- All cryptographic primitives must pass independent security audit
- `abyss-crypto` dev placeholders must be fully replaced by production
  implementations in `abyss-crypto-adapter`

---

## Hardware Security

**Treasury wallets:**
All project treasury funds are held in Trezor Model T hardware wallets
on dedicated seeds used exclusively for ABYSS. Treasury addresses are:

- ETH / USDT (ERC-20) / USDC: `_Withheld pending legal review (EU/MiCA). Will be published once compliance review is complete._`
- BTC: `_Withheld pending legal review (EU/MiCA). Will be published once compliance review is complete._`

Before Sale Stage 2 opens, treasury moves to multisig (M-of-N threshold).

**TEE usage:**
Hardware Trusted Execution Environments (Intel SGX, AMD SEV, ARM TrustZone)
are used where available to harden AI sandbox isolation and key operations.
ABYSS architecture does not assume TEE availability — all security
properties must hold in software-only mode. TEE is an enhancement layer.

---

## Key Management Principles

1. Private keys never leave the hardware wallet or secure enclave.
2. The ABYSS team will never ask users for private keys or seed phrases.
3. ABYSS will never DM users with wallet addresses. All official addresses
   are published on the website and verified through official channels.
4. Seed phrases must be stored offline (paper or steel backup). Never
   photograph, never store digitally, never share.

---

## Audit Requirements

Before any sale stage opens:
- [ ] Smart contract audit (when applicable)
- [ ] Cryptographic primitive review

Before testnet:
- [ ] abyss-consensus security review
- [ ] abyss-crypto production implementation audit
- [ ] Policy Engine correctness review
- [ ] AI sandbox isolation audit

Before mainnet:
- [ ] Full protocol security audit by independent firm
- [ ] ZK circuit audit (when ZK components are implemented)
- [ ] Penetration testing of Runtime Mode isolation
- [ ] Bug bounty program launched

---

## Compliance Principles

ABYSS is not a surveillance tool.
ABYSS does not build backdoors.
ABYSS does not implement mechanisms that allow third parties to access
user data without the user's cryptographic authorisation.

No legal, commercial, or political pressure changes this.

The user's sovereignty over their account is inviolable by design.
