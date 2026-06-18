# ABYSS

ABYSS is a privacy-first autonomous blockchain ecosystem with a native coin,
confidential applications, a native DEX, a private wallet, a private social
network, and configurable AI agents for every account.

Public website:

- production: https://abyss-chain.netlify.app/
- local files: `index.html`, `wallet.html`, `invest.html`, `transition.js`

Native coin:

- name: ABYSS Coin
- symbol: AC
- max supply: 55,000,000 AC

## Repository Layout

- `index.html`, `wallet.html`, `invest.html`, `transition.js` - Netlify-ready
  static website.
- `crates/abyss-core` - chain primitives, AC supply rules, transactions,
  blocks, genesis state, and mempool.
- `crates/abyss-crypto` - development identities; production cryptography will
  replace these placeholders.
- `crates/abyss-consensus` - early validator set and quorum certificate model.
- `crates/abyss-wallet` - wallet accounts and AI-agent permission policy.
- `crates/abyss-node` - CLI node and devnet simulation.
- `docs/` - implementation roadmap and protocol planning.
- `tools/static_server.mjs` - local static website server.

## Development Commands

Rust protocol:

```powershell
cargo test --workspace --lib
cargo run -p abyss-node -- devnet
cargo run -p abyss-node -- account new alice
```

Static website:

```powershell
node tools/static_server.mjs . 8080
```

Then open:

```text
http://127.0.0.1:8080
```

## Notes

The current cryptographic identity code is devnet-only. Production signing,
shielded keys, zero-knowledge circuits, and audited primitives must be added
before any real value is secured.

