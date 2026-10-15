# ABYSS Governance, Identity & Capability Architecture

**Status:** Architecture Decision Baseline
**Version:** 0.1
**Date:** 2026-09-18
**Scope:** Governance, Identity, Persona, Capability, decentralisation, and
privacy boundaries.
**AC Economy:** Not changed by this record.

---

## 1. Purpose and authority

This record fixes the architecture direction described in the ABYSS Blockchain
Consolidated Architecture Specification v0.2. It is the governance, identity,
and capability companion to `ABYSS_CORE_TECHNICAL_SPEC.md`.

It does not select numeric parameters, a final cryptographic proof system, or
the AC Economy. Those are explicitly tracked in
`ABYSS_ARCHITECTURE_PARAMETER_REGISTER.md` and require separate ADRs or
genesis/configuration decisions at their implementation gates.

## 2. Governing principle

> **Decentralized by Design, Progressively Decentralized in Deployment.**

ABYSS may deploy through a controlled bootstrap stage, but its Core Protocol
must not require an enduring central operator, Chain Factory authority, or
private bootstrap secret to transition later to independent validators and
distributed governance.

## 3. Bootstrap governance

Bootstrap governance uses a recorded **N-of-M Genesis Council**, never a
single permanent operator. `N` and `M` are genesis parameters, not values
embedded in this record or assumed in code.

The council may authorize genesis configuration, bootstrap validator
configuration, controlled upgrades, and explicitly scoped emergency bootstrap
procedures. It must not become the automatic permanent governing authority.

```text
Genesis Council → Bootstrap Network → Distributed Validators
→ Decentralized Governance → Federation
```

## 4. Separation of governance domains

Each governance authority has a different scope:

- **Protocol Governance** owns protocol rules, consensus, core upgrades, and
  protocol-level security mechanisms.
- **Chain Governance** owns one chain's validator configuration, policies,
  contracts, chain economics, and permitted upgrades.
- **Application Governance** owns application, organization, commerce, DAO, or
  community resources. It has no automatic control over a chain or the Core
  Protocol.
- **Genesis Governance** is a temporary bootstrap authority, with no implicit
  right to remain permanent after a declared transition.

No governance domain may gain authority in another domain through a UI role,
an AI instruction, a Chain Factory deployment, or an unscoped administrator
key.

## 5. Root identity and credentials

The canonical rule is:

```text
Identity ≠ Wallet ≠ Address ≠ Key
```

Every person or organization can self-generate a **Root ABYSS ID**. This
permissionless root does not depend on a central identity issuer.

External issuers may provide scoped, verifiable credentials to a PersonaID.
Examples include organization membership, employee status, authorization,
eligibility, or an age-related property. A credential proves a specific claim;
it is not an identity and does not replace or expose the Root ABYSS ID.

```text
Root ABYSS ID
  └─ PersonaID
      └─ purpose-specific keys
          └─ purpose-specific addresses or receiver identifiers

External issuer ── scoped credential ──► PersonaID
```

## 6. Persona and recovery

A Root ABYSS ID may authorize multiple independently scoped PersonaIDs, such
as personal, creator, business, or organization personas. Each PersonaID may
have distinct keys, addresses, capabilities, credentials, recovery policy, and
privacy settings.

Persona separation is a privacy and compartmentalisation feature. Linking two
personas requires an explicit authorization, credential, or proof path; it is
never inferred from a shared wallet-address model.

Recovery is explicit, auditable, and scoped. Guardians, hardware keys,
secondary identities, threshold recovery, and time locks may restore only the
authority the policy grants. Identity recovery never automatically discloses
historic private content or spending keys.

## 7. Capability model

All authority follows one capability model:

```text
Capability {
  namespace,
  subject,
  resource,
  action,
  conditions,
  expiration,
  revocation
}
```

The initial namespaces are:

```text
identity
finance
social
storage
commerce
organization
infrastructure
governance
system
```

Every capability is least-privilege. Granting `commerce:store/orders:approve`
does not grant `commerce:*`, finance authority, or protocol authority.

Conditions may constrain a capability by time, resource, credential,
transaction state, multi-party approval, or another explicitly supported
property. Their exact serialisation and proof mechanism remain future technical
decisions.

Capabilities have an explicit lifetime and revocation reference. Permanent,
unbounded permissions are not the default.

`system` is a special namespace for consensus, validators, protocol upgrades,
genesis, and declared emergency mechanisms. Ordinary application users and AI
agents never receive it automatically.

## 8. Roles are convenience, not authority

A role is a named grouping of independently evaluated capabilities. It is not
a global privilege level or an alternative authorization mechanism.

```text
StoreManager
  ├─ commerce:store/orders:read
  ├─ commerce:store/orders:approve [conditions apply]
  └─ commerce:store/inventory:manage
```

A direct capability grant remains valid without a role. A role cannot bypass a
capability's namespace, conditions, expiry, or revocation.

## 9. Privacy and disclosure

ABYSS keeps three independent network declarations:

```text
Network Access:    Open | Permissioned | Restricted
Data Visibility:   Public | Private | Confidential
Disclosure:        Full | Selective | Proof-based
```

Persona and credential designs must support selective disclosure: a party can
prove a necessary property without automatically revealing all identity data.
Specific ZK or other proof systems are not fixed by this record.

## 10. Chain Factory and AI boundaries

Chain Factory is an off-chain control plane. It can compile Blueprints, run
simulations, generate manifests, and present evidence. It cannot validate
transactions, hold user private keys, bypass governance/capabilities, or alter
production state by itself.

ABYSS Architect AI may analyze requirements, propose a Blueprint, identify
conflicts, and run simulations. It has no implicit protocol authority:

```text
AI proposes → Human or authorised governance approves → controlled deployment
```

## 11. Progressive decentralisation

```text
Bootstrap → Distributed → Decentralized → Federated
```

The bootstrap stage may use the Genesis Council, a controlled validator set,
and scoped controlled upgrades. The architecture must subsequently support
independent node operators, distributed validator authority, documented
governance, and federation between independent chains without a Core rewrite.

## 12. Parameters deliberately left open

This record fixes no value for Genesis Council N or M, validator count,
validator admission, staking/collateral, governance thresholds, stage
transition criteria, emergency governance, credential issuer policy, proof
security parameters, recovery time windows, or AC Economy. These are listed
with decision triggers in `ABYSS_ARCHITECTURE_PARAMETER_REGISTER.md`.

## 13. Team handoff

**Blockchain Core:** implement no privileged single-operator path. Keep
protocol, chain, application, and genesis governance distinct.

**Wallet:** treat Root ABYSS ID, PersonaID, keys, and addresses as distinct;
evaluate signed capability claims rather than one global role.

**ABYSS OS:** render human approval and policy simulation; treat AI requests as
untrusted proposals with no bypass route.

**Social:** author through PersonaID and purpose-specific keys, not payment
addresses; consume only `social` capabilities.

**Chain Factory:** remain off-chain and produce attestable Blueprint,
simulation, approval, and deployment artifacts.

**AC Economy:** keeps ownership of AC v0 reconciliation and final economic
decisions.
