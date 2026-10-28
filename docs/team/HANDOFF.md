# HANDOFF

**Active lock:** none
**Last updated:** 2026-10-27 by [claude]

How to use: read this before you start. When you finish, add an entry at the top of the session log. Claude has no repository access, so the owner pastes Claude's entry here.

## Current state (verified facts only)

- Identity Gateway (`gate.js`, `eligibility.js`, `gateway.html`) is deployed on Netlify. Owner-confirmed in production: `+420` is blocked and the admin key works. The eligible path (`+1` → declaration → access) is **not yet confirmed** in production.
- Claude ran the two functions exactly as pasted by the owner in a local simulation: 29 of 29 checks passed. This is not a production test.
- ADR-0025 was synced by Claude (Implementation Status). Delivered to the owner; push not confirmed. Its header date says 2026-10-17 (sandbox clock) and should be 2026-10-27. Fix next time the file is touched.
- `gateway.html` badge now reads "PREVIEW · PROVISIONAL RULES". Delivered; push not confirmed.
- The gateway uses only the country calling code chosen in a dropdown. Typed phone digits are not sent or stored. No SMS verification exists. The restricted list (`420`, `49`, `33`) is a placeholder.
- Treasury wallet addresses are withheld from site, documents and code (D-004). Litepaper PDFs delivered earlier still contain the old sale model and addresses; they are not in the repository.
- The site still carries pre-pivot sale content (investor round, token sale and secondary window blocks in `index.html`; the investor calculator in `wallet.html`). The text fix is waiting for the owner's go-ahead (P-004).
- No-sale master spec: vesting schedule recomputed, 0 mismatches in 28 rows. The five categories of the 40,000,000 AC add up exactly.

## Not verified

Claude has no repository access. Not verified: any Rust code, `cargo test` or `clippy` results, branch and working-tree state, and the contents of ADR-0026 and later files beyond what the owner pasted.

## Next (proposed order)

1. Owner commits this coordination pack (documents only). Start every audit from a clean working tree.
2. Run `docs/team/PHASE0_AUDIT_PROMPT.md` independently in two agents. Neither sees the other's report until both are saved in `docs/audit/`.
3. Owner pastes both reports to Claude. Claude compares them and lists disagreements for the owner.
4. In parallel and not blocked by the audit: website and document text corrections (P-004).
5. Owner answers the open questions in `DECISION_LOG.md`.

## Session log (newest first)

### [claude] 2026-10-27 — Coordination pack and review of the no-sale spec and unified brief
- Did: reviewed the no-sale master spec and the unified brief; recomputed the vesting schedule; created the coordination pack (draft).
- Files: `AGENTS.md`, `docs/team/*` (delivered to the owner, not yet committed).
- Evidence: recomputation script output (0 mismatches); web search on AGENTS.md support in Codex, Claude Code and Kimi Code.
- Not verified: repository state; whether the owner has pushed earlier deliverables.
- Next: owner reviews and commits the pack; answers O-1 to O-9.
