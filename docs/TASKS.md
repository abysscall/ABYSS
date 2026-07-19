# ABYSS Tasks

**Priority (ADR-0015):** independent ABYSS network + native AC. ERC-20 is not the
product; interop later via bridges/wrapped assets. Prioritise native-network work
(consensus, storage, RPC, shielded execution) over any token-contract work.

## Backlog

- Native network track (in priority order): (1) Multi-validator BFT consensus, (2) persistent storage (RocksDB), (3) JSON-RPC node API.
- Genesis Allocation Registry module (ADR-0016): Contribution Receipts → Genesis Root → Genesis Builder → first AC balances.
- Rework investor site (index/invest/wallet.html) to the Genesis Allocation concept ("Contribute to ABYSS Genesis", "Reserve Native ABYSS Allocation", "Genesis Distribution"); keep USDT/USDC/BTC payment rails.
- Legal review before accepting any contributions.
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

