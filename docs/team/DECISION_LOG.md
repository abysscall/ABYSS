# DECISION LOG

Status: `ACCEPTED` (owner confirmed) · `PROPOSED` (suggested, owner has not confirmed) · `OPEN` (question) · `REJECTED`.
Only the owner moves an item to `ACCEPTED`. Agents add items as `PROPOSED` or `OPEN` with evidence.
This repository is public: no secrets, no personal data, no legal speculation.

## Accepted

| ID | Decision | Source |
|---|---|---|
| D-001 | Native coin: ABYSS Coin, symbol AC, hard supply cap 55,000,000 AC. The distribution is under review (P-002). | Owner |
| D-002 | The public site is behind the Identity Gateway. The owner keeps an admin bypass. | Owner, 2026-10 |
| D-003 | Gateway policy: Verify → Declare → Forget. It is Eligibility Verification, not KYC. ABYSS does not collect identity data it does not need. People who may not participate are not admitted. Deliberate circumvention is the user's responsibility to the extent the law allows. | Owner |
| D-004 | Treasury wallet addresses stay withheld from site, documents and code until the owner restores them. | Owner |
| D-005 | The legal-counsel topic is parked by the owner (cost and effort). | Owner, 2026-10-27 |
| D-006 | The Genesis Miner Passport direction is approved. Format, storage and recovery are not specified. | Owner, via unified brief of 2026-10-26 |

## Proposed

| ID | Proposal | Notes |
|---|---|---|
| P-001 | Core first (Approach A) and Balanced-by-Design: privacy, security and performance each have a mandatory minimum. A read-only Phase 0 audit comes before any Core change. | Relayed by the owner from an external AI chat. Claude agrees. |
| P-002 | No-sale model: founder allocation 15,000,000 AC with 6-year vesting, 40,000,000 AC by contribution, sponsorship is a donation. | Claude review: vesting arithmetic verified (0 mismatches in 28 rows). Do not remove `eligibility.js`. Remove "token warrant". Replace "sale is impossible by design" with "the project does not sell AC". Answer the "premine" question honestly. Rename "mining" where it means validator rewards. Define who approves grants. Nothing public may assert this model until it is `ACCEPTED`. |
| P-003 | Neural Mining categories (Uptime, Privacy, Defense, Performance) and early stages 5 / 10 / 35 participants (ceilings 50,000 / 100,000 / 175,000 AC, total 325,000 AC). | Ceilings are limits, not promises. Naming collision: "Founder Stage" vs "Founder Allocation". |
| P-004 | Freeze scope: `crates/**`, consensus, crypto, tokenomics logic and `data/tokenomics.json` are frozen during Phase 0. Website and document text corrections are not blocked by the Core audit. | Needed because the live site still carries pre-pivot sale wording. |
| P-005 | The gateway is utility, not investor-only. Keep `gateway.html`, `gate.js`, `eligibility.js`. | Conflicts with the deletion list in the no-sale master spec. |
| P-006 | This coordination pack (AGENTS.md, PROTOCOL, DECISION_LOG, HANDOFF, Phase 0 prompt). | |
| P-007 | Passport prototype as a signed, verifiable credential. Native protocol record only at mainnet. | Claude suggestion. |
| P-008 | Early-stage contributions tracked in a public, signed ledger ("points") before any AC balance exists. | Claude suggestion. Not a legal conclusion. |
| P-009 | Community stage: published eligibility criteria, then deterministic selection from a public randomness beacon. | Claude suggestion. |
| P-010 | Write the reward invariants as failing tests before any reward code: (1) total supply never changes; (2) issued rewards never exceed the stage budget; (3) a claim is processed at most once; (4) a closed miner authorization cannot be reopened; (5) one Passport per participant record per stage; (6) an emergency stop never changes balances. | Claude suggestion. |

## Open questions

| ID | Question |
|---|---|
| O-1 | Were old investor materials (litepapers, PDFs, links) shared with anyone, or were intent forms submitted on `invest.html`? Netlify Forms keeps submitted data. |
| O-2 | Who holds the founder allocation: a person or a future legal entity? |
| O-3 | Who approves contribution grants in the first period, and how are conflicts of interest handled? |
| O-4 | What can Codex and Kimi do here: read the repo, edit, run cargo, commit, push? |
| O-5 | Which two agents run the Phase 0 audit independently? |
| O-6 | Where does the suspicion "chain height advances but state is not applied" come from? It must be proved or refuted by the audit. |
| O-7 | Which of the 40,000,000 AC buckets funds the 325,000 AC early program? |
| O-8 | Is "Sponsor Stage" eligibility independent of donations? It must be. |
| O-9 | What is the emission schedule of the rewards pool, and is it validator rewards or contribution rewards? |

## Rejected

None yet.
