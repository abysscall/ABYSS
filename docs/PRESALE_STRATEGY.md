# ABYSS Presale Strategy

This document is a working strategy for attracting investment and running a
future ABYSS Coin presale. It is not legal, tax, accounting, or investment
advice. No funds should be accepted until counsel reviews the structure in each
target jurisdiction.

## Objective

Raise capital for ABYSS development while keeping the project credible,
auditable, and resilient:

- fund protocol engineering;
- fund cryptography and security audits;
- fund wallet, DEX, and private social app development;
- fund legal/compliance setup;
- bootstrap liquidity without damaging long-term token economics.

## Native Asset

- name: ABYSS Coin
- symbol: AC
- max supply: 55,000,000 AC
- default unit: 1 AC = 100,000,000 micro-AC

## Proposed Allocation

| Bucket | Share | AC |
| --- | ---: | ---: |
| Validator rewards and network security | 25% | 13,750,000 |
| Ecosystem grants, apps, audits, bug bounties | 20% | 11,000,000 |
| Public sale and liquidity formation | 20% | 11,000,000 |
| Foundation treasury with long vesting | 15% | 8,250,000 |
| Core contributors with long vesting | 10% | 5,500,000 |
| DEX liquidity reserve | 10% | 5,500,000 |

## Proposed Sale Rounds

| Round | AC Cap | Price | Minimum Ticket | Lockup |
| --- | ---: | ---: | ---: | ---: |
| Strategic round | 2,000,000 AC | $1.00 | $100,000 | 24 months |
| Private presale | 3,000,000 AC | $2.00 | $250 | 18 months |
| Public presale stage I | 4,000,000 AC | $3.00 | $50 | 12 months |
| Launch liquidity round | 2,000,000 AC | $5.00 | $25 | 6 months |

Maximum modeled raise: $30,000,000.

These numbers are initial planning values. They should be adjusted after legal,
market, treasury, and security review.

## Investment Materials Needed

Before accepting funds:

- protocol litepaper;
- full technical whitepaper;
- tokenomics paper;
- risk disclosure;
- lockup and vesting terms;
- use-of-proceeds document;
- legal entity and jurisdiction plan;
- AML/KYC policy where required;
- investor FAQ;
- wallet custody warning;
- security/audit roadmap;
- treasury multisig policy.

## Compliance Guardrails

ABYSS should avoid public language that creates avoidable legal risk:

- do not promise guaranteed returns;
- do not advertise "risk-free" upside;
- do not imply exchange listings are guaranteed;
- do not imply the team will pump token price;
- do not take funds before terms are final;
- do not accept sanctioned users or prohibited jurisdictions;
- do not skip KYC/AML where required;
- do not sell to U.S. persons without specialized legal advice;
- do not call the token "utility" unless counsel confirms the structure.

## Investor Readiness Checklist

Minimum readiness before serious investor outreach:

- clean GitHub repository;
- public roadmap;
- working devnet demo;
- clear Netlify website;
- founder/team presentation;
- security-first narrative;
- presale terms draft;
- legal review booked;
- treasury wallet design;
- signed contributor vesting model;
- investor data room.

## Technical Implementation Plan

Short term:

- keep tokenomics as deterministic Rust code in `abyss-tokenomics`;
- expose token plan through `abyss-node tokenomics`;
- keep docs synced with code;
- add JSON export for tokenomics later;
- maintain investor whitelist/KYC-ready data model;
- maintain contribution receipt model;
- maintain vesting schedule model.
- expose presale quote simulation through `abyss-node presale quote`.

Example quote:

```powershell
cargo run -p abyss-node -- presale quote --amount=900 --round=public-stage-1 --kyc-approved
```

Long term:

- implement sale contract only after legal review;
- require multisig treasury;
- require audited smart contracts;
- publish sale reports;
- publish vesting proofs;
- publish circulating supply reports.

## Messaging

Strong positioning:

- privacy-first autonomous blockchain;
- ABYSS Coin powers private transactions, DEX fees, AI-agent execution, and
  network security;
- zero-knowledge privacy is core protocol design, not an add-on;
- AI agents are permissioned by wallet policies, not uncontrolled wallet owners;
- security and audits are explicit budget items.

Avoid:

- "guaranteed profit";
- "next Bitcoin";
- "risk free";
- "military-grade" without concrete controls;
- "fully compliant" before legal sign-off.
