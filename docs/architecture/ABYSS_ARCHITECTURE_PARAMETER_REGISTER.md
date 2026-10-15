# ABYSS Architecture Parameter Register

**Status:** Open decisions — no numeric values are fixed here.
**Purpose:** Prevent hidden defaults and ensure the project owner is notified
before architecture requires a concrete configuration value.

---

## Operating rule

An open parameter must never be silently selected in application code, a
genesis artifact, a testnet profile, a deployment manifest, or production
documentation. When an implementation reaches a trigger below, the owning
team must present the owner with the parameter's purpose, security and
operational constraints, alternatives, and recommendation before assigning a
value.

Development-only constants in the existing repository are not genesis or
production defaults. For example, the current `ChainConfig::max_validators`
value exists only in the devnet skeleton and cannot be copied into a launch
configuration without an explicit decision.

## Governance and validator parameters

### Genesis Council threshold and membership size

**Status:** Open genesis parameters.

**Decision trigger:** Creation of a signed genesis configuration, council key
ceremony, bootstrap upgrade authorization, or a testnet intended to rehearse
bootstrap governance.

**Decision must consider:** fault model, signer independence, key custody,
availability, recovery, emergency scope, and the planned transition away from
Genesis Governance.

### Bootstrap validator configuration

**Status:** Open configuration parameters.

**Decision trigger:** A real multi-node bootstrap testnet, validator
onboarding, validator-power model, or genesis deployment.

**Decision must consider:** consensus fault tolerance, independent operation,
network geography, hardware/network requirements, key rotation, slashing
evidence, and admission/revocation process.

### Validator admission, collateral, and rewards

**Status:** Open protocol and AC Economy parameters.

**Decision trigger:** Start of Distributed Validation, staking implementation,
or any validator reward/slashing code.

**Decision must consider:** Sybil resistance, economic security, validator
concentration, withdrawal rules, liveness, legal review, and the approved AC
Economy.

### Governance thresholds and stage transitions

**Status:** Open protocol parameters.

**Decision trigger:** Implementation of protocol/chain governance, upgrades,
emergency procedures, or a declared Bootstrap → Distributed transition.

**Decision must consider:** quorum, proposal eligibility, voting period,
delegation, timelocks, veto/emergency scope, auditability, and whether the
change is protocol-, chain-, or application-governance.

## Identity, capability, and recovery parameters

### Credential issuer policy

**Status:** Open trust-policy parameters.

**Decision trigger:** Acceptance of external credentials in a production
policy, credential registry, or a proof-based access flow.

**Decision must consider:** issuer onboarding/removal, credential schemas,
revocation, verification freshness, privacy, jurisdictional limits, and
dispute handling.

### Capability conditions and lifetimes

**Status:** Open per-capability configuration.

**Decision trigger:** A capability template, wallet/OS policy UI, agent
delegation, organization feature, or chain policy becomes executable.

**Decision must consider:** least privilege, maximum authority, expiry,
revocation latency, multi-party conditions, audit visibility, and the
difference between a human, agent, and service subject.

### Recovery and emergency delays

**Status:** Open per-Persona and governance configuration.

**Decision trigger:** A recovery policy, guardian flow, dead-man-switch rule,
or emergency upgrade path is implemented.

**Decision must consider:** account takeover risk, loss recovery, guardian
independence, time-lock requirements, disclosure scope, cancellation, and
appeal/reversal rules.

## Privacy and cryptography parameters

### Proof-system security and disclosure policy

**Status:** Open technical/security parameters.

**Decision trigger:** Selection of a cryptographic proof system, implementation
of confidential operations, or release of proof-based disclosure.

**Decision must consider:** security assumptions, trusted setup, verifier cost,
prover cost, hardware requirements, circuit versioning, metadata resistance,
audit scope, and migration path.

### Data retention and disclosure budget

**Status:** Open per-application/chain policy.

**Decision trigger:** Storage, credential disclosure, social visibility, or
confidential business-data features become persistent.

**Decision must consider:** purpose limitation, expiry, revocation, encryption
key lifecycle, off-chain availability, legal obligations, and privacy impact.

## Economic parameters

### AC Economy

**Status:** Owned by the separate AC Economy work; AC v0 remains provisional.

**Decision trigger:** Any code or public document that sets final supply,
allocation, issuance, fees, staking, rewards, burns, resource pricing, or
privacy-operation pricing.

**Decision must consider:** the approved AC Economy specification, security
requirements, sustainability, market/legal review, and the ADR that supersedes
or amends current AC v0 assumptions.

## Required notification format

When a trigger is reached, report:

1. the exact parameter(s) now required;
2. the affected full paths, network/profile, and decision deadline;
3. the constraints and consequences of each viable option;
4. the recommended option and why; and
5. the ADR, genesis document, or configuration artifact that will record the
   final value.

No value is selected until the project owner approves it in the appropriate
decision artifact.
