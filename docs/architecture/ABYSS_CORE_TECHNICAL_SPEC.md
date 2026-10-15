# ABYSS Core Technical Specification

**Status:** Draft for architecture approval
**Version:** 0.1
**Date:** 2026-09-18
**Scope:** Blockchain Core and its contracts with Wallet, ABYSS OS, Social,
Chain Factory, and future interchain components.

---

## 1. Purpose and authority

This document translates the **ABYSS Blockchain — Consolidated Architecture
Specification v0.2** into technical boundaries and implementation gates. It
does not authorise a mainnet launch, a Chain Factory launch, or an AC economic
change.

Accepted ADRs remain authoritative. In particular:

- ADR-0009 defines the future dual execution direction.
- ADR-0010 defines privacy by default as the target, not as a current devnet
  property.
- ADR-0011 and ADR-0014 require separate financial and social authority.
- ADR-0016 fixes the implementation discipline for the base chain.
- ADR-0017 defines the consensus-to-execution lifecycle.
- ADR-0024 prohibits lateral Protocol Layer dependencies.

Where this draft requires a new technical decision, it must be resolved in a
new ADR or RFC. It does not silently amend an accepted ADR.

## 2. Current implementation boundary

The current Rust workspace is a development foundation, not a production
protocol:

- `abyss-core` implements transparent account balances, nonces, atomic staged
  block application, a JSON snapshot, and a development hash.
- `abyss-consensus` implements an in-process BFT round model and quorum
  certificates, but `ConsensusEngine::commit()` does not apply a block to
  `abyss-core::Chain`.
- `abyss-wallet` creates development-key accounts and unsigned transactions;
  its agent limit is local policy logic.
- `abyss-social` is an in-memory data model. A `Shielded` post is not a
  cryptographically encrypted, network-replicated social object.
- `abyss-tokenomics` and the `Coin` limit describe provisional **AC v0** only.
  The separate AC Economy work owns its reconciliation.

No application or design document may claim a stronger current property than
the list above.

## 3. Architectural model

### 3.1 Control plane and data plane

ABYSS has two strictly separate planes:

```text
Control plane, off-chain
  Architect → Blueprint compiler → Shadow Chain → approval → deployment manifest
                                     │
                                     │ signed, hash-addressed artifacts
                                     ▼
Data plane, independently operated
  ChainSpec → consensus → execution → state commitment → persistence → finality
```

The control plane may propose and prepare a network. It must not validate
transactions, participate in quorum, custody end-user keys, or mutate live
chain state outside a chain-authorised upgrade transaction.

### 3.2 Integration rule

Wallet, ABYSS OS, Social, Marketplace, and Chain Factory are clients of
versioned protocol contracts. They must not create special consensus paths or
direct dependencies between the existing Protocol Layer crates.

Before implementation, shared stable identifiers, commitments, proof claims,
capability claims, and `ChainSpec` types belong in a narrow shared interface
package. A proposed name is `abyss-protocol-types`; creating it requires its
own ADR because it becomes a Foundation Layer dependency.

## 4. Canonical identity vocabulary

The following meanings are mandatory across all teams:

```text
Root ABYSS ID       A user-controlled root identity. It is not a public address.
PersonaID           A scoped, unlinkable public persona derived or authorised
                    by a Root ABYSS ID.
WalletAccount       A local wallet container for keys, policies, and recovery
                    material. It is not an on-chain global identity.
Address             A purpose-specific payment or execution endpoint. It is
                    not an identity and must not become a universal social ID.
Key                 Cryptographic material used to authenticate a narrowly
                    defined purpose. Key reuse across purposes is prohibited.
Capability          A bounded, revocable grant of authority.
```

The target hierarchy is:

```text
Root ABYSS ID
  └─ PersonaID
      └─ purpose-specific authentication, payment, encryption, and recovery keys
          └─ one or more addresses or receiver identifiers
```

Social content is authored by a PersonaID and a purpose-specific signing key.
Wallet payment addresses are not social account identifiers. Recovery of a
Root ABYSS ID or PersonaID does not automatically reveal encrypted content or
spending keys; those recovery scopes must be declared independently.

Root ABYSS IDs are self-generated. They do not require a central identity
issuer. An external issuer may instead issue a verifiable credential to a
PersonaID for a narrow claim such as membership, eligibility, or authorization.
A credential is neither a Root ABYSS ID nor a replacement for one.

## 5. Capability contract

All authority is expressed as a capability with this semantic shape:

```text
Capability {
  id,
  namespace,
  subject,
  resource,
  action,
  conditions,
  issued_at,
  expires_at,
  revocation_reference,
  issuer_proof
}
```

`namespace` is mandatory. The initial namespaces are `identity`, `finance`,
`social`, `storage`, `commerce`, `organization`, `infrastructure`,
`governance`, and `system`. A capability can never grant authority outside its
namespace. For example, an agent allowed to publish to `social` cannot
initiate a `finance` transfer. The `system` namespace is reserved for tightly
scoped protocol operations and is never implicitly granted to an application
user.

A role is only a named collection of capabilities. It does not bypass,
replace, or broaden the capability model: every effective role permission must
resolve to a capability with an explicit namespace, resource, action,
conditions, expiration, and revocation reference.

The exact serialisation, signature scheme, revocation state, and privacy proof
format remain open technical decisions. Their semantics must be identical in
Wallet, OS, Social, and policy simulation.

## 6. Network privacy model

Every chain profile declares three independent dimensions:

```text
NetworkAccess     = Open | Permissioned | Restricted
DataVisibility    = Public | Private | Confidential
DisclosurePolicy  = Full | Selective | ProofBased
```

These are independent declarations, not mutually exclusive chain types. A
profile can therefore be `Open + Confidential + ProofBased`.

The mandatory data-placement rule is:

```text
On-chain:       commitments, hashes, proofs, permissions, references, state roots
Encrypted/off-chain: personal data, business documents, large payloads, private content
```

No user interface label such as “private” or “shielded” is a security claim
until the selected cryptographic mechanism, metadata model, and proof
verification path are implemented and audited.

## 7. Transaction and block lifecycle

The present `Transaction` struct is a devnet transfer object, not the canonical
transaction format. The target transaction envelope must be versioned,
chain-bound, expiry-bound, domain-separated, and cryptographically
authenticated before it enters a production mempool.

Its eventual fields must cover:

```text
protocol_version, chain_instance_id, expiry, fee/resource declaration,
action payload or commitment, signer/authorisation proof, and replay domain
```

The action model may later represent transfer, contract call, capability grant,
capability revocation, and governance-authorised upgrade. A Chain Factory
prompt, raw AI output, or unapproved Blueprint is never a transaction action.

The canonical commit sequence remains ADR-0017:

```text
propose → structural verification → deterministic staged execution
→ state-root comparison → quorum votes → QC
→ commit canonical state → persist → finalise height
```

The first engineering gate is to bind a verified quorum certificate to the
exact proposed block and invoke the execution/persistence path only after that
certificate. Validator votes must be authenticated before this becomes a
network protocol.

## 8. Progressive decentralisation

Bootstrap centralisation is permitted only as an explicit and removable chain
profile:

```text
Stage 0  Development: local nodes and test-only tooling.
Stage 1  Bootstrap: an N-of-M Genesis Council and declared controlled validator set.
Stage 2  Distributed: independently operated validators and collateral rules.
Stage 3  Open: participation and Sybil-resistance policy chosen by the consensus ADR.
Stage 4  Federation: independent ABYSS-compatible networks exchange proofs.
```

No bootstrap-only secret, privileged RPC method, hard-coded operator address,
or Chain Factory authority may be required for a later stage. Validator-set,
governance, and upgrade transitions must be represented by authorised,
auditable state transitions.

Governance is separated by scope:

- **Protocol Governance** controls protocol rules, consensus, and core upgrades.
- **Chain Governance** controls one chain's validators, policies, contracts,
  economics, and permitted upgrades.
- **Application Governance** controls application or organization resources and
  cannot obtain protocol authority implicitly.
- **Genesis Governance** is the temporary bootstrap N-of-M council and has no
  automatic permanent governance role.

The values of N and M, validator configuration, thresholds, and transition
criteria are intentionally absent from this specification. Their decision
triggers are maintained in `ABYSS_ARCHITECTURE_PARAMETER_REGISTER.md`; no
production configuration may rely on an undocumented default.

## 9. Chain profiles and promotion

ABYSS does not create a standalone blockchain for every user. The approved
product progression is:

```text
Personal Workspace → Shared Namespace → Dedicated Chain → Federation
```

- **Personal Workspace** uses encrypted local or distributed storage and may
  anchor commitments to an existing chain. It has no independent consensus.
- **Shared Namespace** uses a shared settlement/finality environment with
  namespaced applications or an equivalent audited profile.
- **Dedicated Chain** has an explicit validator/sequencer configuration,
  governance, ChainSpec, resource policy, and finality model.
- **Federation** communicates through verified finality evidence and explicit
  capabilities; it does not grant implicit balance, key, or administrator
  access between chains.

New profiles are audited templates, not arbitrary protocol generation. A
request for custom consensus, security assumptions, or privacy model is a
research/ADR process, not a no-code setting.

## 10. Chain Blueprint, Passport, and upgrade model

A Blueprint is a declaration; a `ChainSpec` is its deterministic,
machine-verifiable compilation output. The manifest binds deployment to exact
approved artifacts.

```text
Blueprint vN
  → schema validation
  → deterministic ChainSpec compilation
  → Shadow Chain evidence
  → declared audit scope/status
  → approver signatures
  → Deployment Manifest
  → deployed chain and Chain Passport
```

The minimum Blueprint/ChainSpec fields are:

- `instance_id`, parent/settlement chain, template and protocol versions;
- genesis commitments and validator/sequencer profile;
- access, visibility, disclosure, identity, capability, and recovery policies;
- execution modules, contract versions, resource and token registry policies;
- interchain endpoints and permitted capabilities;
- upgrade, emergency, and governance authority;
- hashes of simulation evidence, audit scope/status, and approver signatures.

The Chain Passport publishes only intentionally discoverable facts: chain ID,
version, Blueprint hash, declared security/audit status, interoperability
endpoints, dependencies, and governance metadata. It must never expose private
keys, private user data, or undisclosed validator/network metadata.

Every production change follows `Proposal → Simulation → Approval →
Deployment`. A deployment is rejected when its manifest does not bind to the
approved Blueprint and evidence hashes. Security simulation is evidence, not
an audit conclusion.

## 11. Interchain, recovery, oracle, and directory boundaries

Interchain begins with finality/light-client evidence and explicit capability
messages. A trusted bridge key, shared administrator key, or implicit asset
access does not meet this specification.

Recovery is a per-Persona policy with explicit scope. Guardians, hardware
keys, threshold recovery, and time locks can recover only the authority they
are granted; they do not receive automatic access to historic private data.

Oracles are untrusted external inputs until an application validates their
provenance, freshness, quorum, and failure behaviour. Oracle output cannot
alter base consensus rules.

The Network Directory is a discovery index, not a source of consensus truth.
Directory entries refer to signed Chain Passports and may disclose only the
information a chain makes public.

## 12. Delivery gates

### Gate A — secure core before Factory

1. Define canonical transaction authentication and wire QC to execution,
   persistence, and finality.
2. Replace the development consensus-critical hash and development-only keys.
3. Implement durable storage, authenticated P2P, sync, and RPC under the
   ADR-0017 lifecycle.
4. Add tests that prove deterministic state roots and reject unauthorised,
   replayed, malformed, and mismatched-QC transitions.

### Gate B — shared contracts

1. Approve the identity vocabulary and the capability semantics in this draft.
2. Define versioned serialization and migration policy for protocol contracts.
3. Make Wallet, OS, and Social consume the same capability semantics without
   sibling crate dependencies.
4. Specify recovery scopes and disclosure revocation semantics.

### Gate C — Blueprint and Shadow Chain

1. Define Blueprint schema, deterministic compiler, and ChainSpec validation.
2. Implement at most one or two audited profile templates.
3. Produce reproducible simulation artifacts and a signed approval trail.
4. Only then build the user-facing Factory workflow and AI Architect.

### Gate D — privacy, interchain, and economics

Privacy proof selection, interchain transport, marketplace modules, and AC
Economy each require their own approved technical specification. They do not
block Core design, but they must not be represented as delivered beforehand.

## 13. Owner input and parameter gates

The following architecture decisions are now fixed by
`ABYSS_GOVERNANCE_IDENTITY_CAPABILITY_DECISION_RECORD.md`:

1. Bootstrap governance is an N-of-M Genesis Council, with N and M held as
   explicit genesis parameters rather than architecture constants.
2. Root ABYSS IDs are self-generated; external issuers may provide scoped
   credentials to PersonaIDs.
3. The capability namespaces are `identity`, `finance`, `social`, `storage`,
   `commerce`, `organization`, `infrastructure`, `governance`, and `system`.
   Roles are convenience groupings, never an authority bypass.

Before a Gate implementation needs an unresolved value, consult
`ABYSS_ARCHITECTURE_PARAMETER_REGISTER.md` and present its trigger, security
constraints, and alternatives to the project owner. No arbitrary placeholder
may reach a testnet or production configuration.

After AC Economy is complete, the project owner must nominate the ADR that
formally supersedes or amends the current AC v0 supply/sale assumptions.

## 14. Team handoff

**Blockchain Core team:** complete Gate A; do not add Factory business logic to
consensus.

**Wallet team:** migrate from `WalletAccount = identity` thinking to scoped
keys under PersonaID; prepare to sign versioned transaction envelopes and
consume capability claims.

**ABYSS OS team:** treat human/AI intent as untrusted input; render policy
simulation and approval flows, but do not bypass chain or wallet authority.

**Social team:** use PersonaID plus content manifests and encrypted payload
references; remove the assumption that a payment address is a social account.

**Chain Factory team:** remain off-chain; compile approved Blueprints into
verifiable artifacts and never take custody of user or validator keys.

**AC Economy team:** owns the reconciliation of current AC v0 values with the
future economic model. No Core change should pre-commit this outcome.
