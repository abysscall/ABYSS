# ABYSS Wallet & AI Agent Model — Reference

See ADR-0011 for the decision rationale.
Implementation: `crates/abyss-wallet/src/`

---

## Account Model

Every ABYSS account consists of:
- A keypair (public key → address)
- An AgentPolicy (financial permissions)
- An AgentSocialPolicy (social permissions, see docs/social/SOCIAL.md)

Financial and social policies are intentionally separate objects.
An Agent authorised for social actions cannot leverage that to move funds.

---

## Agent Financial Policy

```rust
// Permissions an Agent may be granted
pub enum AgentPermission {
    ExecuteLimitedTrades,
    // future: SendPayments, StakeOnBehalf, VoteOnBehalf
}

// Per-agent spending cap
agent_policy.set_agent_trade_limit(Coin::from_ac(250));
```

When `create_agent_payment()` is called:
1. Policy is checked: does the Agent have `ExecuteLimitedTrades`?
2. Amount is checked: does it exceed `agent_trade_limit`?
3. If either check fails, the transaction is rejected before entering the mempool.

---

## Devnet Demonstration

```bash
cargo run -p abyss-node -- devnet
```

This demonstrates:
1. Treasury → Alice transfer (standard transaction)
2. Alice grants Agent `ExecuteLimitedTrades` with a 250 AC limit
3. Agent-authorised payment of 125 AC (within limit) → accepted
4. (If tested) payment of 300 AC (exceeds limit) → rejected by policy

---

## CLI: Account Creation

```bash
cargo run -p abyss-node -- account new my-wallet
```

Output: label, address, public_key, agent_permissions.
Warning: development keypairs only. Production key management
(hardware wallet integration, encrypted storage) is a Phase 5 item.
