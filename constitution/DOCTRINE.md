# ABYSS OS — DOCTRINE

These are the principles that govern every decision in ABYSS.
They are not guidelines. They are constraints.
When a proposed feature or implementation conflicts with doctrine,
the doctrine wins.

---

## Doctrine 1 — Privacy Is Architecture

Privacy is not a feature that can be added later.
Privacy is not a setting the user enables.
Privacy is not a compliance checkbox.

Privacy is part of the base design of every ABYSS system component.

If a component cannot be built with privacy as a first-class property,
the design of that component must change.

**Corollary:** any component that leaks data by default is architecturally
wrong, regardless of how convenient it is.

---

## Doctrine 2 — The User Is Sovereign

The user owns their ABYSS Account.
The user owns their identity.
The user owns their data.
The user owns their AI.
The user owns their computation.

No external party — not a company, not a government, not the ABYSS team —
can access, modify, suspend or delete a user's account or data without the
user's cryptographic authorisation.

**Corollary:** any design that creates a backdoor, a recovery mechanism
controlled by a third party, or an override path outside the user's control
is a violation of this doctrine.

---

## Doctrine 3 — AI Is Powerful But Never Sovereign

The Native AI is a system-level component of every ABYSS Account.
It has significant capabilities.
It can act autonomously within defined limits.

But the AI is always the instrument. The user is always the owner.

The AI cannot:

- bypass the Policy Engine
- access data not explicitly permitted by the user
- spend funds beyond its allocated budget
- modify its own permissions
- acquire capabilities the user has not granted
- act on behalf of any party other than the account owner

**Corollary:** a Native AI that can override the Policy Engine is not an
ABYSS Native AI. It is a security vulnerability.

---

## Doctrine 4 — Policy Is Law

Every action in ABYSS — whether initiated by the user, by a Native AI, or by
an external request — must pass through the Policy Engine.

The Policy Engine is not advisory. It is the final enforcement layer.

No component, no shortcut, no performance optimisation justifies bypassing
the Policy Engine.

**Corollary:** a system call that does not pass through the Policy Engine is
an architectural defect.

---

## Doctrine 5 — Do Not Break What Works

The existing ABYSS foundation is working:

- consensus engine
- core chain and state
- cryptographic layer
- tokenomics
- wallet primitives
- social layer skeleton

New capabilities must be built on top of this foundation.
They must not require dismantling it.

Every new component must define a clear interface with the existing layer.
If building a new component requires breaking an existing one, the design is
wrong. Find a different path.

**Corollary:** premature refactoring of working systems is a form of
architectural debt, not a form of improvement.

---

## Doctrine 6 — Interfaces Before Implementation

Before writing implementation code for any new ABYSS OS component, the
interface must be defined and accepted.

This applies to:

- ABYSS Account
- Native AI Runtime
- Policy Engine
- System Call model
- Storage layer
- Runtime Mode
- Portable Mode

An interface is the contract. Implementation is the fulfilment.
Contracts must exist before fulfilment begins.

**Corollary:** code that implements an undefined interface is a guess, not
an engineering decision.

---

## Doctrine 7 — Selective Disclosure Is the Default Identity Model

An ABYSS identity does not reveal everything or nothing.

The user controls which attributes of their identity are visible to which
parties, under which conditions, and for how long.

Examples of valid disclosure policies:

```
Alice → Service A:
  ✓ age > 18 (proven by ZK, not revealed)
  ✓ unique human (proven, not identified)
  ✗ name
  ✗ balance
  ✗ transaction history
  expires: 30 days
```

No system in ABYSS should require full identity disclosure when partial
disclosure is sufficient.

**Corollary:** any authentication mechanism that requires exposing more
identity information than the minimum necessary is architecturally wrong.

---

## Doctrine 8 — Hard Supply Cap Is Inviolable

ABYSS Coin (AC) has a hard cap of 55,000,000.

This is not a parameter. It is a constant.

No governance vote, no team decision, no market condition changes this.
The supply is fixed. The economy is built around scarcity.

**Corollary:** any protocol upgrade that modifies the supply cap is invalid
by definition, regardless of the majority that approved it.

---

## Doctrine 9 — Nodes Are Execution Environment, Not Just Validators

In ABYSS OS, a validator node is not merely a consensus participant.

It is a distributed execution node of ABYSS OS.

Validators participate in:

- consensus and state validation
- AI execution verification
- resource accounting
- security enforcement

The network of nodes is the distributed kernel of ABYSS OS.

**Corollary:** validator design must account for execution responsibility, not
only consensus participation.

---

## Doctrine 10 — Build for the Long Arch

ABYSS is building toward a ten-year architectural vision.

Short-term decisions must not compromise long-term integrity.

This means:

- prefer clean interfaces over convenient shortcuts
- prefer correct design over fast implementation
- prefer documented trade-offs over undocumented hacks
- prefer composable components over monolithic systems

Every architecture decision record (ADR) must state its long-term
compatibility assumptions explicitly.

**Corollary:** a decision that solves a problem today but creates a migration
wall in 18 months is not a solution. It is a deferred problem.

---

## What Doctrine Is Not

Doctrine is not a list of preferences.
Doctrine is not a starting point for negotiation.
Doctrine is not something we revisit when it becomes inconvenient.

Doctrine defines what ABYSS is.

If an implementation contradicts doctrine, the implementation changes.
If a roadmap item contradicts doctrine, the roadmap item changes.
If a business decision contradicts doctrine, the business decision changes.

The doctrine does not change to accommodate the implementation.
