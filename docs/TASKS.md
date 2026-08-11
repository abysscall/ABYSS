# ABYSS Tasks

Last updated: 2026-08-11

---

## Backlog

### Rust / Protocol

- Wire `abyss-crypto-adapter` (production ed25519 signing) into `abyss-node`
  and `abyss-wallet`, replacing the `abyss-crypto` dev-keys path. The adapter
  crate builds and has passing tests but nothing in the workspace depends on it
  yet — see AUDIT_REPORT_2026_08_02.md CRITICAL-1.
- `abyss-crypto-api::Keypair::secret()` returns the raw secret type by
  reference (no wrapper), which undercuts the "kept private" intent in its own
  doc comment. Revisit trait shape once the adapter is wired in — callers
  should only get a `Signer`, never the raw secret.
- Re-check AUDIT_REPORT_2026_08_02.md ADR citations against current numbering
  — several have drifted. Treat ADR numbers in that report as approximate.
- Add vesting schedule logic on-chain.
- Start `abyss-crypto` production primitive selection (libsodium or RustCrypto).
- Implement Consensus-to-Execution finality interface (AUDIT CRITICAL-2).
- Add transaction encryption layer (AUDIT HIGH-1).
- Create `docs/threat_models/` (consensus, crypto, privacy, economic).

### Website / Frontend

- Insert treasury wallet addresses into `invest.html` when Trezor wallets
  are ready (ETH/USDT/USDC address + BTC address).
- Verify Netlify Forms are receiving submissions after first deploy
  (check Netlify dashboard → Forms).
- Add newsletter form to `index.html` Netlify Forms hidden field.
- Test `presale-quote.js` against all 5 sale rounds, not just
  `sale-to-investors`.
- Fill in `constitution/` documents (VISION, DOCTRINE, ARCHITECTURE,
  ROADMAP, SECURITY, FIVE_YEAR_RULE).
- Fill in `rfc/RFC-0001-platform.md` and `rfc/RFC-0002-policy-engine.md`.
- Update Netlify domain from `abyss-chain.netlify.app` to custom domain
  when ready.

### Legal / Business

- Review presale strategy with crypto/securities counsel before accepting
  funds.
- Set up multisig treasury wallet before Sale Stage 2 opens.
- Prepare investor data room (litepaper, whitepaper, tokenomics paper,
  risk disclosure, vesting terms).
- Book legal review for token sale structure and jurisdiction.

---

## In Progress

- Netlify Forms for newsletter and investor intents — deploy and verify
  in Netlify dashboard after next push.
- Treasury hardware wallet setup (Trezor Model T, dedicated seed for ABYSS).
- Wallet addresses for invest.html (pending Trezor generation).

---

## Review

- Rust devnet skeleton (consensus, mempool, chain, wallet policy).
- Static website integration with tokenomics.json.
- Presale quote engine on invest.html.

---

## Done

- Establish `C:\ABYSS` as the main monorepo on Windows after recovery.
- Repository published at https://github.com/abysscall/ABYSS
- Rust workspace added (9 crates).
- Core AC supply model: 55,000,000 hard cap, 30M team, 25M public sale.
- Devnet chain, mempool, wallet policy, and consensus primitives added.
- Static website kept at repository root for Netlify.
- Replace simple buyback with Investor Secondary Window model.
- Chain persistence, wallet CLI, presale quote on invest.html.
- Fix Cargo.toml repository URL to https://github.com/abysscall/ABYSS
- Sync PRESALE_STRATEGY.md with actual tokenomics.json sale rounds.
- Unify token symbol to AC everywhere in docs.
