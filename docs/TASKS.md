# ABYSS Tasks

## Backlog

- Wire `abyss-crypto-adapter` (production ed25519 signing) into `abyss-node` / `abyss-wallet`, replacing the `abyss-crypto` dev-keys path. Currently the adapter crate builds and has passing tests but nothing in the workspace depends on it — see AUDIT_REPORT_2026_08_02.md CRITICAL-1.
- `abyss-crypto-api::Keypair::secret()` returns the raw secret type by reference (no wrapper), which undercuts the "kept private" intent in its own doc comment. Revisit trait shape once the adapter is wired in — may want callers to only get a `Signer`, never the raw secret.
- Re-check `AUDIT_REPORT_2026_08_02.md`'s ADR citations against the current numbering — several have drifted (its "ADR-0017" for privacy/crypto principles is now ADR-0023; its "ADR-0021" for slashing is now closer to ADR-0022's cryptographic-foundation content). The report itself is a dated snapshot and wasn't rewritten; treat its ADR numbers as approximate.
- Compare `C:\Users\z-mir\abyss` with the current website and extract cold/multisig wallet UX ideas.
- Migrate old newsletter subscribers from ABYSS-website backups if `subscribers.csv` / `abyss.db` are found.
- Decide whether old folders should be archived or deleted after comparison.
- Review presale strategy with crypto/securities counsel before accepting funds.
- Add vesting schedule logic on-chain.
- Start `abyss-crypto` production primitive selection.

## In Progress

- Netlify Forms for newsletter and investor intents (deploy + verify in dashboard).

## Review

- Rust devnet skeleton.
- Static website integration.
- Chain persistence, wallet CLI, presale quote on invest page.

## Done

- Establish `C:\ABYSS` as the main monorepo.
- Rust workspace added.
- Core AC supply model added.
- Devnet chain, mempool, wallet policy, and consensus primitives added.
- Static website kept at repository root for Netlify.
- Replace buyback with investor secondary window model.
- Chain persistence, wallet CLI, presale quote on invest page.

