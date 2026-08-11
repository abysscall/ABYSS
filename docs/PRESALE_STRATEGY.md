# ABYSS Presale Strategy

This document is the authoritative working strategy for attracting investment
and running the ABYSS Coin presale. It is not legal, tax, accounting, or
investment advice. No funds should be accepted until counsel reviews the
structure in each target jurisdiction.

---

## Objective

Raise capital for ABYSS development while keeping the project credible,
auditable, and resilient:

- fund protocol engineering and cryptography research;
- fund security audits (zk-SNARK circuits, consensus, smart contracts);
- fund wallet, DEX, private social app, and AI agent development;
- fund legal/compliance setup and entity formation;
- bootstrap ecosystem liquidity without damaging long-term token economics.

---

## Native Asset

- name: ABYSS Coin
- symbol: AC
- max supply: 55,000,000 AC (hard cap, no inflation)
- default unit: 1 AC = 100,000,000 micro-AC
- team reserve: 30,000,000 AC (locked, long vesting)
- public sale total: 25,000,000 AC

---

## Token Allocation

| Bucket                              | AC           |
| ----------------------------------- | -----------: |
| Team & founding contributors        | 30,000,000   |
| Public sale (all rounds combined)   | 25,000,000   |
| **Total**                           | **55,000,000** |

Team allocation is subject to a vesting schedule to be published before
the investor round opens. No team tokens are liquid at sale launch.

---

## Sale Rounds

All rounds are sequential. A round closes when its token cap is sold out
or the project formally closes it. Rounds do not overlap.

| # | Round                        | AC Cap     | Price / AC | Min. Ticket   | Notes                              |
|---|------------------------------|------------|------------|---------------|------------------------------------|
| 1 | Sale to Investors            | 2,000,000  | $1.00      | $500,000      | Max 4 slots; accredited investors  |
| 2 | Pre-Sale                     | 3,000,000  | $2.00      | none          | Open to registered participants    |
| 3 | Sale Stage 1                 | 5,000,000  | $3.00      | none          | Public                             |
| — | Investor Secondary Window    | —          | $3.00      | 250,000 AC    | 14-day opt-in; P2P only; ABYSS has no buyback obligation |
| 4 | Sale Stage 2                 | 5,000,000  | $4.00      | none          | Public                             |
| 5 | Sale Stage 3                 | 10,000,000 | $5.00      | none          | Public                             |
| 6 | Final Sale — DEX Order Book  | variable   | market     | none          | Buyback tokens relisted on ABYSS testnet DEX |

Maximum modeled raise (if all rounds sell out): ~$93,000,000.

### Investor Secondary Window — details

After Sale Stage 1 closes, a two-phase opt-in window opens for Round 1
investors only:

- **Phase A (14 days):** investors submit listing intent; minimum lot
  size is 250,000 AC (50% of a full slot).
- **Phase B:** listed tokens are matched with new buyers at $3.00/AC
  via P2P facilitation. ABYSS has **no obligation to purchase** any tokens.
  This is not a guarantee of exit.

### Final Sale — DEX Order Book

Tokens sourced from the Investor Secondary Window (if any) will be relisted
as sell orders on the ABYSS DEX in testnet mode. This is the first live
demonstration of the ABYSS private decentralised exchange. Buyers set orders;
price discovery is open. Supply is variable depending on secondary window
participation.

---

## Maximum Raise Breakdown

| Round          | AC       | Price | Max Raise    |
|----------------|----------|-------|--------------|
| Investors      | 2,000,000| $1.00 | $2,000,000   |
| Pre-Sale       | 3,000,000| $2.00 | $6,000,000   |
| Stage 1        | 5,000,000| $3.00 | $15,000,000  |
| Stage 2        | 5,000,000| $4.00 | $20,000,000  |
| Stage 3        |10,000,000| $5.00 | $50,000,000  |
| **Total**      |**25,000,000**|   | **$93,000,000** |

---

## Investment Materials Needed Before Accepting Funds

- [ ] Protocol litepaper;
- [ ] Full technical whitepaper;
- [ ] Tokenomics paper (this doc + data/tokenomics.json);
- [ ] Risk disclosure (drafted in invest.html);
- [ ] Lockup and vesting terms (published per round);
- [ ] Use-of-proceeds document;
- [ ] Legal entity and jurisdiction plan;
- [ ] AML/KYC policy where required;
- [ ] Investor FAQ;
- [ ] Hardware wallet custody policy for treasury;
- [ ] Security and audit roadmap;
- [ ] Treasury multisig policy.

---

## Treasury Wallet Policy

Project funds are held in hardware wallets (Trezor Model T) on dedicated
seeds used exclusively for ABYSS. Separate addresses are maintained per
accepted currency:

- USDT (ERC-20) and USDC (ERC-20) and ETH → Ethereum address (Trezor)
- BTC → Bitcoin address (Trezor)

All treasury addresses are published on the official website and verified
via official Telegram and Twitter before any round opens. ABYSS will never
DM investors asking for funds. Investors must independently verify the address
before every transfer.

Multisig treasury (M-of-N) will be implemented before Stage 2 opens.

---

## Compliance Guardrails

ABYSS avoids public language that creates avoidable legal risk:

- do not promise guaranteed returns;
- do not advertise "risk-free" upside;
- do not imply exchange listings are guaranteed;
- do not imply the team will pump token price;
- do not take funds before terms are final;
- do not accept sanctioned users or prohibited jurisdictions;
- do not skip KYC/AML where required;
- do not sell to U.S. persons without specialised legal advice;
- do not call the token "utility" unless counsel confirms the structure;
- do not describe the Investor Secondary Window as a guaranteed buyback.

---

## Investor Readiness Checklist

Minimum readiness before serious investor outreach:

- [x] Clean GitHub repository (https://github.com/abysscall/ABYSS)
- [x] Public roadmap (docs/ROADMAP.md)
- [x] Working devnet demo (cargo run -p abyss-node -- devnet)
- [x] Clear Netlify website
- [ ] Founder/team presentation
- [ ] Security-first narrative (litepaper)
- [ ] Presale terms draft (legal review)
- [ ] Legal review booked
- [ ] Treasury hardware wallet setup (Trezor, dedicated seed)
- [ ] Signed contributor vesting model
- [ ] Investor data room

---

## Technical Implementation Plan

### Short term

- tokenomics kept as deterministic Rust code in `abyss-tokenomics`;
- `data/tokenomics.json` is the single source of truth for website and CLI;
- docs stay in sync with `tokenomics.json` and `lib.rs`;
- investor intent forms processed via Netlify Forms;
- presale quote simulation via CLI:

```powershell
cargo run -p abyss-node -- presale quote --amount=500000 --round=sale-to-investors --kyc-approved
cargo run -p abyss-node -- presale quote --amount=1000 --round=pre-sale
cargo run -p abyss-node -- presale quote --amount=500 --round=public-stage-1
```

### Long term

- sale contract only after legal review;
- multisig treasury before Stage 2;
- audited smart contracts;
- publish sale reports after each round closes;
- publish vesting proofs;
- publish circulating supply reports;
- DEX order book live for Final Sale.

---

## Messaging

### Strong positioning

- privacy-first autonomous blockchain — not a fork, built from scratch;
- ABYSS Coin (AC) powers private transactions, DEX fees, AI agent
  execution, and network security;
- zero-knowledge privacy is core protocol design, not an add-on;
- every wallet account has its own AI agent, permissioned by the user;
- security and audits are explicit budget line items.

### Avoid

- "guaranteed profit";
- "next Bitcoin";
- "risk free";
- "military-grade" without concrete controls;
- "fully compliant" before legal sign-off;
- "guaranteed buyback" in relation to the Investor Secondary Window.
