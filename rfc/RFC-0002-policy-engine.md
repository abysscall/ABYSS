# RFC-0002 — ABYSS Policy Engine

**Status:** Proposed
**Created:** 2026-08-11
**Authors:** ABYSS Core Team
**Depends on:** RFC-0001

---

## Summary

This RFC defines the Policy Engine — the enforcement layer between
user intent and system execution in ABYSS OS.

Every action taken by a user, a Native AI, or an external request
passes through the Policy Engine. No exceptions.

---

## Motivation

Without a formal Policy Engine, the ABYSS AI system has no
cryptographically enforced boundary between what the AI is allowed
to do and what it actually does.

A Native AI without a Policy Engine is:
- a security risk (no spending limit enforcement)
- a trust problem (users cannot verify AI behaviour)
- architecturally incomplete (AI sovereignty instead of user sovereignty)

The Policy Engine is the mechanism that makes the doctrine
"AI is powerful but never sovereign" true at the protocol level.

---

## Core Concept

The Policy Engine evaluates every proposed action against the account
owner's declared policy before the action reaches the execution layer.

```
Action proposed by: user / Native AI / external request
           │
           ▼
    Policy Engine
           │
     ┌─────┴─────┐
     │           │
  PASS         REJECT
     │           │
     ▼           ▼
System Call   Error returned to caller
     │
     ▼
Consensus / Execution
```

---

## Policy Object

Every ABYSS Account has a Policy Object.
The Policy Object is owned by the account holder.
Only the account holder can modify it (requires private key signature).

### Policy Object Structure

```rust
struct AccountPolicy {
    // Spending controls
    spending: SpendingPolicy,

    // Allowed interaction targets
    allowlist: Allowlist,

    // AI-specific controls
    ai: AiPolicy,

    // Delegation controls
    delegations: Vec<Delegation>,

    // View key grants
    view_grants: Vec<ViewGrant>,

    // Automation rules
    automation: Vec<AutomationRule>,

    // Multisig requirements
    multisig: Option<MultisigPolicy>,
}
```

### SpendingPolicy

```rust
struct SpendingPolicy {
    per_transaction_max: Option<u64>,   // max AC per single transaction
    daily_limit: Option<u64>,           // max AC per 24h rolling window
    period_limit: Option<PeriodLimit>,  // max AC per week/month
    asset_allowlist: Option<Vec<AssetId>>, // allowed asset types
}
```

### Allowlist

```rust
struct Allowlist {
    allowed_recipients: Option<Vec<Address>>,   // whitelist of addresses
    blocked_recipients: Vec<Address>,            // blacklist
    allowed_modules: Option<Vec<ModuleId>>,     // allowed system modules
    allowed_contracts: Option<Vec<Address>>,    // allowed contract addresses
}
```

### AiPolicy

```rust
struct AiPolicy {
    // What the Native AI can do autonomously
    autonomous_transfer_limit: Option<u64>,     // per action
    autonomous_daily_limit: Option<u64>,        // per day
    allowed_ai_recipients: Option<Vec<Address>>,
    allowed_ai_modules: Option<Vec<ModuleId>>,
    can_publish_social: bool,
    can_access_storage: bool,
    can_delegate_to_subagents: bool,
    max_subagent_budget: Option<u64>,
    compute_budget_per_day: Option<u64>,        // in compute units
}
```

### Delegation

```rust
struct Delegation {
    agent_id: AgentId,
    permissions: DelegatedPermissions,
    spending_limit: Option<u64>,
    allowed_recipients: Option<Vec<Address>>,
    expires_at: Option<Timestamp>,
    revocable: bool,
}
```

### ViewGrant

```rust
struct ViewGrant {
    grantee: GranteeId,             // who receives the grant
    attributes: Vec<Attribute>,     // which attributes are visible
    expires_at: Option<Timestamp>,
    revocable: bool,
}

// Example attributes:
// Attribute::AgeOver(18)           — ZK proof, age not revealed
// Attribute::UniqueHuman           — proof of humanity
// Attribute::Balance               — reveals actual balance
// Attribute::TransactionHistory    — reveals tx history
// Attribute::ReputationScore       — reveals score without history
// Attribute::Name                  — reveals display name
```

---

## Intent Translation

The Policy Engine includes an intent layer that translates human
(or AI) expressed intentions into policy-evaluated system calls.

```
Human/AI intent:
"Pay Alice 50 AC when she confirms delivery."

Intent Parser output:
{
  action: schedule,
  trigger: { event: "delivery_confirmation", from: alice_address },
  system_call: transfer(alice_address, 50_AC),
  policy_context: {
    initiator: native_ai,
    account: user_account,
  }
}

Policy Engine checks:
✓ 50 AC <= per_transaction_max (100 AC)
✓ alice_address in allowed_recipients
✓ trigger-based scheduling is allowed in AiPolicy
✓ daily_limit not exceeded

Result: system call scheduled
```

---

## AI Permission Evaluation

When the Native AI proposes an action, the Policy Engine evaluates it
against the `AiPolicy` section of the account policy.

```
AI proposes: transfer(recipient, amount)

Policy Engine checks:
1. Is recipient in allowed_ai_recipients (or no allowlist)?
2. Is amount <= autonomous_transfer_limit?
3. Is cumulative today <= autonomous_daily_limit?
4. Is asset in allowed asset types?

All pass → system call proceeds
Any fail → action rejected, AI receives error with reason
```

The AI cannot retry a rejected action by rephrasing it.
The policy check is deterministic for a given policy state.

---

## Delegation Model

A user can delegate authority to their Native AI, or the Native AI
can delegate to a sub-agent.

Delegation is always bounded:

```
Account Owner grants to Native AI:
  max 200 AC / day
  allowed recipients: [merchant_list]
  expires: 30 days

Native AI grants to Sub-Agent:
  max 50 AC / day          ← must be ≤ AI's own limit
  allowed recipients: [subset of merchant_list]
  expires: 7 days          ← must be ≤ AI's own expiry
```

A delegation cannot grant more authority than the delegating party holds.
This is enforced by the Policy Engine at delegation creation time.

---

## Automation Rules

The Policy Engine supports time-based and event-based automation:

```rust
struct AutomationRule {
    trigger: Trigger,
    action: SystemCall,
    constraints: PolicyConstraints,
    max_executions: Option<u32>,
    expires_at: Option<Timestamp>,
}

enum Trigger {
    Schedule(CronExpression),
    Event(EventFilter),
    Condition(ConditionExpression),
    AfterDelay(Duration),       // Shadow Transaction model
}
```

Shadow Transactions (privacy-preserving delayed execution) are
implemented as `Trigger::AfterDelay` with randomised execution
within a declared window. This provides temporal privacy without
claiming absolute untraceability.

---

## Multisig Policy

For accounts requiring multiple signers (ABYSS Safe):

```rust
struct MultisigPolicy {
    required_signers: u32,          // M
    total_signers: u32,             // N
    signer_set_commitment: Hash,    // commitment to signer set (not public)
    applies_to: MultisigScope,      // which actions require multisig
}
```

The signer set is not public. External observers see a single account
identity with a commitment proving M-of-N structure exists.
This is the "Invisible Threshold Wallet" model.

---

## Private DAO Voting

The Policy Engine supports a private voting primitive:

```
Proposal: P42
Eligible voters: committed set of ABYSS IDs
Each voter submits: ZK proof of eligibility + encrypted vote

Policy Engine verification:
✓ voter is in eligible set (ZK, voter identity not revealed)
✓ voter has not voted before (nullifier check)
✓ vote is valid

Tally: homomorphic or reveal-on-close
Result: YES/NO/ABSTAIN percentages published
Individual votes: permanently private
```

---

## Error Model

When the Policy Engine rejects an action:

```rust
enum PolicyError {
    SpendingLimitExceeded { limit: u64, requested: u64 },
    RecipientNotAllowed { recipient: Address },
    ModuleNotAllowed { module: ModuleId },
    AiLimitExceeded { limit: u64, requested: u64 },
    DelegationExpired { expired_at: Timestamp },
    DelegationExceedsGrantor { requested: u64, grantor_limit: u64 },
    MultisigRequired { required: u32, provided: u32 },
    ViewGrantExpired,
    ViewGrantScopeInsufficient { requested: Attribute, granted: Vec<Attribute> },
    PolicyNotFound,
    Unauthorised,
}
```

All errors are returned to the caller. No silent failures.

---

## Implementation Notes

### Phase 4 Minimum Viable Policy Engine

For Phase 4, the minimum implementation must provide:

1. SpendingPolicy enforcement (per-transaction and daily limits)
2. Allowlist enforcement (allowed recipients)
3. AiPolicy enforcement (autonomous transfer limits)
4. Delegation creation and validation
5. ViewGrant creation, validation, and revocation

ZK-based selective disclosure and Private DAO Voting are longer-term
additions that require the ZK primitive ADR to be resolved first.

### Rust Implementation Target

```
crates/abyss-policy/
├── src/
│   ├── lib.rs
│   ├── policy.rs         — AccountPolicy and sub-structs
│   ├── engine.rs         — evaluation logic
│   ├── intent.rs         — intent → system call translation
│   ├── delegation.rs     — delegation model
│   ├── view_key.rs       — view grant management
│   ├── automation.rs     — automation rules
│   └── error.rs          — PolicyError enum
└── Cargo.toml
```

---

## Open Questions

1. Should the Policy Object be stored on-chain (accessible to validators)
   or encrypted off-chain with only commitments on-chain?

2. What is the correct granularity for compute budget accounting in AiPolicy?

3. How does the intent parser interact with the Native AI — is it part of
   the AI runtime or the Policy Engine?

4. What is the revocation model for ViewGrants — instant on-chain or
   time-bounded with commitment?

---

## Acceptance Criteria

This RFC is accepted when:

- [ ] AccountPolicy structure is agreed and matches abyss-wallet conventions
- [ ] Policy Engine evaluation logic is specified unambiguously
- [ ] Delegation model correctly prevents privilege escalation
- [ ] Error model is complete
- [ ] Phase 4 minimum implementation scope is confirmed
- [ ] Integration path with existing abyss-node system call path is clear
