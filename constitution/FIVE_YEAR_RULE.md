# ABYSS OS — FIVE YEAR RULE

This document defines what ABYSS must look like in five years
and the constraints that govern how we get there.

It is not a marketing projection. It is an architectural commitment.

---

## The Question

Every decision made today must be tested against one question:

> **Does this decision make ABYSS stronger or weaker in five years?**

If the answer is "weaker" or "unknown", the decision requires deeper
analysis before it proceeds.

---

## Where ABYSS Must Be in Five Years

### 1. A Real Operating Environment

In five years, ABYSS OS Runtime must be available and stable on:

- Windows (primary)
- Linux (primary)
- macOS (secondary)

And ABYSS OS Portable must work from external SSD on standard x86_64
and ARM64 hardware.

ABYSS Native Mode must be in active development with a clear roadmap.

A person who has never heard of blockchain must be able to install ABYSS
Runtime, create an Account, and have a working Native AI within 15 minutes.

### 2. A Functioning Economy

In five years:

- All seven AC sale rounds must be complete or clearly on schedule
- AC must be tradeable on at least the ABYSS DEX (testnet → mainnet)
- The AI-to-AI micro-payment economy must be live in at least prototype form
- Treasury must operate under multisig with published addresses and reports
- Circulating supply reports must be published after every sale stage

### 3. A Trustworthy Protocol

In five years:

- The consensus layer must have passed independent security audit
- The cryptographic layer must be production-grade (no dev placeholders)
- The Policy Engine must be audited and formally specified
- The ZK primitive selection must be decided via ADR and implemented
- A live bug bounty program must be running

### 4. An Identity Layer That Works

In five years:

- ABYSS ID must be a working cryptographic identity used by real accounts
- Selective disclosure via view keys must be live and usable by developers
- At least one ZK-based proof (age, uniqueness, or reputation) must be
  available in the protocol

### 5. A Native AI That Respects Its Owner

In five years:

- Every ABYSS Account must have a Native AI runtime attached
- The AI must be sandboxed, budgeted, and policy-controlled
- The AI must support at least: intent interpretation, scheduled tasks,
  and basic AI-to-AI delegation
- The Abyssal Scheduler must be live and managing AI workloads

---

## What Must Not Change in Five Years

These are constants. They do not evolve. They do not get reconsidered.

**1. The 55,000,000 AC hard cap.**
It will not change. No governance proposal can change it.
Any implementation that makes this possible is wrong.

**2. Privacy by default.**
Every new component added in the next five years must have
privacy as a first-class property. Not an option. Not a setting.
Architecture.

**3. AI is never sovereign.**
No matter how capable the Native AI becomes in five years,
the Policy Engine remains the enforcement layer.
The user remains the owner.

**4. The user owns their account.**
No ABYSS team update, no governance vote, no legal request can
override a user's cryptographic control of their own account.

---

## What Must Improve in Five Years

**Performance.**
The current devnet is a prototype. In five years, the network must
support real throughput for AI-to-AI transactions, social layer activity,
and wallet operations simultaneously.

**Developer experience.**
In five years, building on ABYSS OS must have:
- documented SDK
- published ABI for System Calls
- working testnet with faucet
- developer documentation that does not require reading source code

**Cryptography.**
The dev-only signing and hashing must be fully replaced.
The ZK components must be live.
All of it must have been independently audited.

**Community.**
In five years, the ABYSS team must not be the only contributors.
There must be external validators, external developers building on
the platform, and a clear path for community governance.

---

## Decisions That Shorten the Five-Year Arc

The following decisions would damage ABYSS's five-year position.
They must be avoided.

**Rewriting the foundation.**
The existing consensus, core, crypto, wallet and tokenomics represent
significant proven work. Rewriting any of it without a concrete,
documented reason is destruction of value, not improvement.

**Adding features without architecture.**
Every new capability must have an ADR, an accepted interface, and a
defined relationship to existing components. Features without
architecture accumulate as debt.

**Promising more than the cryptography can deliver.**
ABYSS must never claim "unbreakable privacy" or "impossible to trace".
These are marketing claims that damage credibility when the edge cases
emerge. Describe what the system actually provides and what it does not.

**Accepting funds without legal readiness.**
Taking investor funds before legal structure, terms, and KYC/AML
policy are in place creates liability that can stop the project.
This applies to all sale stages.

**Centralising control.**
Any mechanism that gives the ABYSS team special administrative access
to user accounts or funds must be rejected. Not because the team is
untrustworthy but because the architecture must not depend on trust.

---

## The Five-Year Test

Before any major decision, ask:

1. Does this make the protocol stronger in five years?
2. Does this respect the doctrine?
3. Does this build on what exists, or does it require breaking it?
4. If the team disappeared tomorrow, could the community continue?
5. Would we be proud to explain this decision publicly?

If the answer to any of these is "no" or "uncertain", the decision
requires more work before it proceeds.

---

## The Single Most Important Five-Year Outcome

If ABYSS achieves only one thing in five years, it must be this:

> A person can install ABYSS, create an Account, have a Native AI
> working for them, make private transactions, and control their own
> digital identity — without trusting any company, any platform,
> or the ABYSS team itself.

Everything else is in service of this.
