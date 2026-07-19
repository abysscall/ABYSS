# ABYSS Decentralised Social Layer — Design Reference

See ADR-0014 for the decision rationale.
Current implementation: `crates/abyss-social/src/lib.rs` (data model, 14 tests).
Production storage and P2P replication: Phase 6 on the roadmap.

---

## Core Design Principles

**Identity reuses the wallet address.**
There is no separate social account to register, link, or have breached.
Your ABYSS wallet address is your social identity.

**Content lives on content-addressed, replicated storage.**
No single company server. No single point of censorship.
Once published, content cannot be unilaterally deleted by any central party.

**Visibility uses the same view-key model as transactions.**
Post attributed (public authorship) or shielded (anonymous authorship).
Grant selective disclosure via view keys — same mechanism, same privacy guarantees.

**Moderation is on-chain and transparent.**
Rules are public. Enforcement is verifiable. No opaque shadow-banning.

---

## Data Model

### Post

```rust
pub struct Post {
    pub id: PostId,
    pub author: String,          // ABYSS wallet address
    pub body: String,            // max 2,000 bytes
    pub visibility: Visibility,  // Attributed | Shielded
    pub created_at_ms: u64,
    pub in_reply_to: Option<PostId>,
    pub authored_by_agent: bool,
}
```

### Visibility

```rust
pub enum Visibility {
    Attributed,  // author is public to all
    Shielded,    // author visible only to self + view-key grantees
}
```

### ViewKeyRegistry

Maps PostId → list of addresses granted authorship visibility.
Mirrors the financial view-key model (ADR-0010, whitepaper Section 5.3).

### AgentSocialPolicy

Separate from financial Agent policy (ADR-0011). An Agent authorised only
to curate content can never spend funds, and vice versa.

```rust
pub struct AgentSocialPolicy {
    pub can_post: bool,
    pub can_reply: bool,
    pub can_repost: bool,
    pub max_posts_per_window: u32,
    pub rate_window_seconds: u64,
}
```

Presets:
- `AgentSocialPolicy::none()` — no social permissions
- `AgentSocialPolicy::curator_default()` — reposts only, 10/hour
- `AgentSocialPolicy::full_posting_default()` — post/reply/repost, 5/hour

---

## CLI Reference

```bash
# Full devnet demonstration
cargo run -p abyss-node -- social demo

# Single attributed post
cargo run -p abyss-node -- social post --author=abyss1myaddress --body="Hello ABYSS"

# Single shielded post
cargo run -p abyss-node -- social post --author=abyss1myaddress --body="Anonymous thought" --shielded
```

---

## Current vs Production

| Aspect | Current (devnet) | Phase 6 (production) |
|---|---|---|
| Storage | In-memory DevFeed | Content-addressed, replicated |
| Identity | Any string | ABYSS wallet address with signature |
| Authorship proof | Trust | Cryptographic signature |
| Moderation | None | On-chain governance |
| Monetisation | None | AC tips and creator economy via DEX |
