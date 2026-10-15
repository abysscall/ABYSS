# ABYSS AC Economy — Working Draft

**Status:** Proposed working draft — not active protocol, sale, or genesis configuration  
**Draft ID:** `ac-economy-working-draft-2026-09`  
**Owners:** AC Economy and Wallet  
**Related:** ADR-0027 (proposed), ADR-0024, ADR-0026, RFC-0001, RFC-0002

---

## Scope and authority

This document records the current direction for the AC economy and Wallet
utility model. It is deliberately separate from the AC v0 devnet plan in
`docs/tokenomics/TOKENOMICS.md` and `TokenomicsPlan::abyss_default()`.

Until ADR-0027 is accepted and a versioned genesis configuration is approved:

- no contribution may be accepted, quoted, or settled from this draft;
- no token can be issued, released, or claimed from this draft;
- `data/tokenomics.json`, public sale pages, and existing v0 CLI commands keep
  their current semantics; and
- Core, Consensus, Wallet, and ABYSS OS must not treat a draft balance, tier,
  or release policy as an entitlement.

The planning model is available only for reconciliation and review through
`AcEconomyWorkingDraft::current()`.

## Fixed direction under review

AC remains a hard-capped native asset with a maximum supply of **55,000,000
AC**. The working allocation is:

| Bucket | Exact AC | Informational share |
|---|---:|---:|
| Sponsorship Rewards | 25,000,000 | 45.4545…% |
| Team / Core Development | 15,000,000 | 27.2727…% |
| Ecosystem / Protocol Reserve | 15,000,000 | 27.2727…% |
| **Total** | **55,000,000** | **100%** |

Exact AC amounts, rather than rounded basis points, are authoritative for this
draft. The 25M / 15M / 15M split cannot be expressed exactly in whole basis
points of a 55M cap.

## Sponsorship Rewards

The 25M Sponsorship Rewards allocation is provisionally divided into five
stages. `USD per AC` expresses the ratio used by the discussion; it is not a
price, offer, legal term, or authorization to accept funds.

| Stage | Reward pool | USD per AC | Implied maximum contribution |
|---|---:|---:|---:|
| 1 | 1,000,000 AC | $1 | $1,000,000 |
| 2 | 10,000,000 AC | $2 | $20,000,000 |
| 3 | 5,000,000 AC | $3 | $15,000,000 |
| 4 | 5,000,000 AC | $4 | $20,000,000 |
| 5 | 4,000,000 AC | $5 | $20,000,000 |
| **Total** | **25,000,000 AC** | — | **$76,000,000** |

Before this can become an implementation, AC Economy and legal review must
define eligibility, contribution handling, reward rights, vesting, claiming,
cancellation/refund treatment, stage transition criteria, reporting, and
jurisdiction-specific restrictions. A reward entitlement, a credited AC
balance, an available AC balance, and circulating supply are four distinct
states.

## Team / Core Development release policy

The team allocation is not an automatic unlock. A permitted release is always
calculated from the **current remaining Team / Core Development balance**, not
from the original 15M allocation. Capacity unused in one period never carries
forward into a later period.

| Period | Maximum release from current balance | Status |
|---|---:|---|
| Year 1 | 0% | Drafted |
| Years 2–3 | 12% per year | Drafted |
| Year 4 | 25% | Drafted |
| Years 5–6 | 50% per year | Drafted |
| Year 7+ | Not selected | Must remain limited; formula open |

`TeamReleasePolicy` deliberately represents Year 7+ as `PendingDecision`.
It must not be replaced by a full-unlock default. The future ledger/contract
also needs a precise definition of what counts as a release: direct sale,
third-party transfer, liquidity provision, or other movement into free
circulation. Transfers between controlled team custody addresses require a
separate auditable rule.

## Ecosystem / Protocol Reserve

The 15M reserve is a distinct allocation, not a disguised Team balance.
Potential uses include development grants, infrastructure, audits and bug
bounties, AI and Wallet development, ecosystem rewards, community initiatives,
and strategic reserve. Any Reserve → Team movement requires its own published,
auditable governance rule; this draft creates none.

## Wallet utility and security boundary

AC is intended to support payments, staking, protocol participation, and
access to additional Wallet functions. It does **not** sell basic safety.

The current access proposal is L = 0–999 AC, F = 1,000–9,999 AC, and Pro =
10,000+ AC. These thresholds and the balance-evaluation method (instant,
average, or hybrid with grace period) remain open. They must not gate the
baseline security controls available to every account.

L, F, and Pro all receive high baseline protection. Pro may add more explicit
control: stricter approval policies, higher-sensitivity anomaly detection,
advanced privacy controls, validator/staking operations, and user-selected
protection for tier-impacting or high-value transfers. Possessing 10,000 AC
does not itself make an account a validator or authorize infrastructure access.

## Protected-transfer compatibility rule

The proposed `Trusted → Instant`, `Unknown → Delay`, and `Suspicious →
Protected Buffer` experience must never claim to reverse a final blockchain
transfer. A cancellation is possible only before settlement inside a future,
explicitly designed protected-transfer/escrow action. Once a transfer reaches
`SETTLED`, it is final.

```text
INITIATED → ANALYZING → PROTECTED → CONFIRMED → SETTLED
                             └──────────────→ CANCELLED
```

This is not implemented by the current `Transaction` type. It requires a
separate approved transaction/action specification, deterministic execution
rules, recipient-visible pending semantics, timeout handling, and adversarial
testing. Neither a Wallet UI nor an AI agent may pretend that an ordinary
on-chain transfer can be recalled.

## Product integration contract

- **ABYSS Core:** owns `Coin::MAX`, canonical transaction semantics, state
  transition, and final settlement. It must not import Wallet or tokenomics
  policy code.
- **Consensus:** remains blind to balances, Wallet tiers, risk scores, and
  transaction meaning; it finalizes only the exact approved block.
- **AC Economy:** owns allocation/release/reward reconciliation. It publishes
  versioned, reviewable parameters but grants no execution authority.
- **Wallet:** simulates a transfer's tier and balance impact locally, collects
  signatures, and creates only protocol-defined actions. It cannot declare a
  transfer reversible after final settlement.
- **ABYSS OS / Policy Engine:** treats human, Claude, Kimi, and every other AI
  request as untrusted intent. Every proposed action is evaluated by the same
  account policy before Wallet signing or execution. An AI may analyze, warn,
  delay, or apply an owner-approved policy; it has no implicit right to settle
  or release funds.
- **Application integration:** is the only layer permitted to compose Core,
  Wallet, and AC Economy outputs. It must bind inputs to explicit version IDs
  and surface `draft`, `active`, or `superseded` status to users.

No product may copy threshold, release, or risk logic into a private fork. A
change is introduced through this document/ADR, a versioned shared interface,
test vectors, and a coordinated migration plan.

## Decisions still required

1. Acceptance or amendment of ADR-0027.
2. Legal and economic specification for sponsorship rewards and any public
   contribution flow.
3. Year 7+ team release formula and the definition of free circulation.
4. Reserve governance, disclosure, and audit rules.
5. Wallet tier measurement, grace-period, validator/staking requirements, and
   privacy mechanism selection.
6. Versioned protocol action for protected transfers, including adversarial
   timeout, double-spend, and recipient-notification cases.
7. Shared serialization and test vectors before Wallet, ABYSS OS, or an AI
   runtime consumes the model.
