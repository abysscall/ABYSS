# AGENTS.md — ABYSS

You are one of several AI agents (Codex, Kimi, Claude, others) serving one human owner.
**The owner decides. Agents propose, verify and report.**

Read this file first, then:

1. `docs/team/HANDOFF.md` — current state, active lock, notes from the last session
2. `docs/team/DECISION_LOG.md` — what is accepted, proposed and open
3. `docs/team/PROTOCOL.md` — workflow details (owner-facing, in Russian)

## Hard rules

1. **Evidence before claims.** Never say something is implemented, working or passing unless you read the code or ran the command in this session. Cite `path:function` and line. Label status as `Implemented`, `Partial`, `Concept only` or `Unknown`. If you did not run the tests, say so.
2. **Read-only until a scope is approved.** Audits change nothing. A change needs an approved scope: files, goal, acceptance check.
3. **One writer at a time.** Check `Active lock` in HANDOFF.md. If another agent holds it, work read-only. A lock older than 24 hours is stale: ask the owner.
4. **Stay in your lane** (table below). Do not edit outside it without the owner's approval.
5. **Small, verifiable steps.** Run checks after each step. If you are close to your usage limit, stop at a clean state and write the handoff. Never leave files half-edited.
6. **No destructive git.** Do not run `git push`, `reset`, `rebase`, `clean`, `stash`, force options, or switch branches unless the owner asks. Run `git status` before and after your work. If you commit locally on request, start the message with your tag: `[codex]`, `[kimi]` or `[claude]`.
7. **No secrets.** Never print, request or store keys, seed phrases, tokens or Netlify variables (`ABYSS_GATE_SECRET`, `ABYSS_ADMIN_SECRET`, `ABYSS_ADMIN_KEY`). This repository is public.
8. **Do not invent parameters.** Open numbers live in `docs/architecture/ABYSS_ARCHITECTURE_PARAMETER_REGISTER.md`. The supply cap is 55,000,000 AC. Do not silently choose economic, cryptographic or governance values: present options and ask.
9. **Public wording.** No promises of price, profit, returns, liquidity or listings. Do not describe AC as sold or purchasable. Do not claim privacy, throughput or security that is not proven. Do not call the launch "fair launch".
10. **Protect the gateway.** Do not delete or rename `gateway.html`, `netlify/edge-functions/gate.js` or `netlify/functions/eligibility.js` without the owner's explicit approval. They are the Identity Gateway (ADR-0025), not presale code.
11. **Record, do not remember.** End every session with an entry in `docs/team/HANDOFF.md`. Put decisions in `DECISION_LOG.md` as `PROPOSED`. Only the owner marks `ACCEPTED`.
12. **Disagree with evidence, not confidence.** If another agent's claim looks wrong, quote the code or output that shows it and report it. Do not silently "fix" it.

## Lanes (proposed; the owner may reassign)

| Area | Default writer | Notes |
|---|---|---|
| `crates/**`, `Cargo.toml`, `Cargo.lock`, `.github/**`, `tools/**` | Codex | **Frozen during the Phase 0 audit.** Later changes need an approved scope. |
| `data/tokenomics.json`, supply and allocation numbers | Owner decides, Codex implements | Frozen until the tokenomics decision is `ACCEPTED`. |
| `*.html`, `assets/**`, `netlify/**`, `netlify.toml` | Claude | Others read-only. |
| `docs/legal/**`, `docs/investor/**`, public wording | Claude | Others read-only. |
| `docs/adr/**`, `docs/architecture/**`, `constitution/**`, `rfc/**` | Any agent drafts | Reserve the ADR number in HANDOFF.md first. One independent reviewer. Owner accepts. |
| `docs/audit/**` | Each auditor writes only its own file | Never edit another agent's report. |
| `docs/team/**` | All agents append | Only the owner changes PROTOCOL.md and marks decisions. |

Kimi's default role is independent auditor and reviewer (read-only) until the owner grants a lane.

## Stop and ask the owner when

- code, ADRs, tokenomics and documents contradict each other;
- you find a security or fund-safety defect;
- a task would touch a frozen area;
- an instruction conflicts with `DECISION_LOG.md`.
