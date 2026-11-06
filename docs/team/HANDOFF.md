# HANDOFF

**Active lock:** none
**Last updated:** 2026-10-28 by [claude]

How to use: read this before you start. When you finish, add an entry at the top of the session log. Claude has no repository access, so the owner pastes Claude's entry here.

## Current state (verified facts only)

- `main` on GitHub is at `cf7bb9b` (owner's push output, 2026-10-28). It contains the coordination pack, the ADR-0025 update and the gateway badge change (`61600f0`).
- Identity Gateway (`gate.js`, `eligibility.js`, `gateway.html`) is deployed on Netlify. Owner-confirmed in production: `+420` is blocked and the admin key works. The eligible path (`+1` → declaration → access) is **not yet confirmed** in production. Claude's local simulation of the two functions as pasted by the owner: 29 of 29 checks passed (not a production test).
- The gateway uses only the country calling code chosen in a dropdown. Typed phone digits are not sent or stored. No SMS verification exists. The restricted list (`420`, `49`, `33`) is a placeholder.
- ADR-0025 header says `Updated: 2026-10-17`; the commit was made on 2026-10-18. Cosmetic; fix next time the file is touched.
- **Website text fix (D-008) is prepared, not yet pushed:** `index.html`, `wallet.html`, `invest.html` delivered by Claude. They remove the investor round, secondary window, price table, allocation calculator, receive-address and fake QR from the public pages. `invest.html` becomes a "page retired" notice. Claude checked: tags balanced, inline scripts parse, internal anchors resolve, no links to `invest.html`, no wallet addresses, embedded logos untouched. Not checked: rendering in a browser.
- Treasury wallet addresses are withheld from site, documents and code (D-004). Litepaper PDFs delivered earlier still contain the old sale model; they are not in the repository.
- Documents still describing the old sale model (not yet updated): `docs/investor/*`, `docs/LITEPAPER_EN.md`, `docs/PRESALE_STRATEGY.md`, the sale table in `constitution/ROADMAP.md`, `constitution/FIVE_YEAR_RULE.md`, README, `data/tokenomics.json` and the tokenomics crate. Their update waits for the tokenomics decision (P-002).
- `netlify.toml` was removed in commit `e595cd2` ("remove conflicting netlify.toml to restore root deployment"). Author and reason are not recorded here. After the next deploy, re-check that the gate still works.
- Branch `kimi/crypto-wiring` (local, not on GitHub) holds commit `dcaed14`, an **unreviewed work in progress** on `abyss-consensus`. Claude read the diff as text (not compiled):
  - adds a private `chain: Option<Chain>` field, `attach_chain()` and a `chain()` accessor to `ConsensusEngine`; adds four new `ConsensusError` variants;
  - the doc comments refer to `apply_committed_block`, which is **not in the diff** — the wiring is unfinished;
  - none of the new error variants is produced anywhere in the diff;
  - `use abyss_core::block::Block;` is added but unused, which would raise an unused-import warning and fail `clippy -D warnings`;
  - `BlockExecutionFailed(abyss_core::ApplyError)` needs `ApplyError` to implement `Clone`, `Eq`, `PartialEq`, `Debug` and `Display` — unverified;
  - `attach_chain` reuses `HeightAlreadyFinalized` for the case where the chain is *behind* the engine, which the name does not describe.
  - It appears aimed at the consensus-to-execution wiring of ADR-0017 steps 6–10, which suggests the "height advances but state is not applied" suspicion (O-6) has a real basis. The Phase 0 audit must still prove or refute it on `main`.
  - Decision: do not merge. Keep parked.
- Local-only leftovers: branch `backup-mixed-commit` (can be deleted later); untracked `PROJECT_MAP.md`, `repo-inventory.txt`, `docs/architecture/ABYSS_BALANCED_ARCHITECTURE_NEURAL_MINING.md` (an older version of the unified brief). Do not commit them without reading them.

## Not verified

Claude has no repository access. Not verified: any Rust code beyond the diff above, `cargo test` or `clippy` results, and whether `main` has the consensus-to-execution defect.

## Next (proposed order)

1. Owner pushes the website text fix (explicit file names only) and re-tests the gate in a private window, including the eligible path.
2. Owner checks Netlify → Forms for old submissions (O-1) and reports only how many exist.
3. Phase 0 audit (`docs/team/PHASE0_AUDIT_PROMPT.md`) from a clean `main`, never from `kimi/crypto-wiring`. Kimi is paused (D-007). Codex's usage limit resets on 2026-11-01.
4. Owner pastes the audit report to Claude for review.
5. After the tokenomics decision: update the documents listed above. Until then, add an "under revision" banner to each instead of rewriting.

## Session log (newest first)

### [claude] 2026-10-28 — Website text fix, branch review, handoff update
- Did: rebuilt `index.html` and `wallet.html` without sale content, created the `invest.html` retirement notice, read the `kimi/crypto-wiring` diff, diagnosed the mixed commit, updated this file and the decision log.
- Files: `index.html`, `wallet.html`, `invest.html`, `docs/team/HANDOFF.md`, `docs/team/DECISION_LOG.md` (delivered to the owner, not yet committed).
- Evidence: verification script output (tag balance, script syntax, anchors, forbidden-word scan); owner's git output for the push and the diff.
- Not verified: rendering in a browser; compilation of any Rust code.
- Next: see the list above.

### [claude] 2026-10-27 — Coordination pack and review of the no-sale spec and unified brief
- Did: reviewed the no-sale master spec and the unified brief; recomputed the vesting schedule (0 mismatches in 28 rows); created the coordination pack.
- Files: `AGENTS.md`, `docs/team/*`. Pushed to `main` by the owner on 2026-10-28 as `cf7bb9b`.
