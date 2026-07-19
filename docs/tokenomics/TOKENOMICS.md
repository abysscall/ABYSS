# ABYSS Tokenomics — Technical Reference

See ADR-0012 for the decision rationale.
This document is the authoritative reference for all token economics parameters.
The source of truth in code is `crates/abyss-tokenomics/src/lib.rs`.

---

## Supply

| Parameter | Value |
|---|---|
| Token symbol | AC |
| Maximum supply | 55,000,000 AC (hard cap) |
| Team Reserve | 30,000,000 AC (54.5%) |
| Public Sale Pool | 25,000,000 AC (45.5%) |

The cap is enforced in `Coin::MAX` and validated in `TokenomicsPlan::validate()`.

---

## Team Reserve Allocation

| Category | AC | Basis Points |
|---|---|---|
| Validator rewards & network security | 8,250,000 | 1,500 |
| Ecosystem grants, apps, audits, bug bounties | 5,500,000 | 1,000 |
| Foundation treasury (long vesting) | 5,500,000 | 1,000 |
| Core contributors (long vesting) | 5,500,000 | 1,000 |
| DEX liquidity reserve | 5,250,000 | 1,000 |
| **Total** | **30,000,000** | **5,500** |

---

## Team Reserve Vesting

Two tranches, both linear with no cliff:

| Tranche | Amount | Period | Rate |
|---|---|---|---|
| A | 10,000,000 AC | 12 months | ~833,333 AC/month |
| B | 20,000,000 AC | 48 months | 5,000,000 AC/year (cap) |

**Year-by-year unlock:**

| Year | Unlocked this year | Cumulative |
|---|---|---|
| 1 | 15,000,000 (A complete + B yr1) | 15,000,000 |
| 2 | 5,000,000 (B only) | 20,000,000 |
| 3 | 5,000,000 (B only) | 25,000,000 |
| 4 | 5,000,000 (B complete) | 30,000,000 |
| 5 | 0 | 30,000,000 |

---

## Public Sale — Seven Stages

| # | Stage | Tokens | Price | Raise |
|---|---|---|---|---|
| 1 | Sale to Investors | 2,000,000 | $1.00 | $2,000,000 |
| 2 | Pre-Sale | 3,000,000 | $2.00 | $6,000,000 |
| 3 | Sale Stage 1 | 5,000,000 | $3.00 | $15,000,000 |
| — | Investor Secondary Window | ≤2,000,000 | $3.00 | P2P |
| 4 | Sale Stage 2 | 5,000,000 | $4.00 | $20,000,000 |
| 5 | Sale Stage 3 | 10,000,000 | $5.00 | $50,000,000 |
| — | Final Sale (ABYSS DEX) | Variable | $5.00 | Variable |
| **Total (fixed rounds)** | | **25,000,000** | | **$93,000,000** |

---

## Investor Secondary Window

See ADR-0013 for full rationale.

| Parameter | Value |
|---|---|
| Eligible sellers | Stage I investors only |
| Eligible buyers | Any participant |
| Price | $3.00 (fixed) |
| Minimum listing | 50% of Stage I allocation = 250,000 AC |
| Registration phase | 14 days |
| Sales phase | Until all listed tokens are sold |
| ABYSS obligation | None — facilitated P2P market |

---

## CLI Reference

```bash
# Full tokenomics plan
cargo run -p abyss-node -- tokenomics
cargo run -p abyss-node -- tokenomics --json

# Vesting schedule
cargo run -p abyss-node -- vesting
cargo run -p abyss-node -- vesting --json

# Sale quote
cargo run -p abyss-node -- presale quote --amount=500000 --round=sale-to-investors --kyc-approved --professional

# Secondary window info and quote
cargo run -p abyss-node -- presale secondary-window --info
cargo run -p abyss-node -- presale secondary-window --tokens=300000

# DEX final sale quote
cargo run -p abyss-node -- presale dex-quote --amount=5000
```
