//! Decentralised social layer primitives for ABYSS.
//!
//! This module defines the data model for the social layer described in
//! the ABYSS Technical Whitepaper, Section 9. It is a planning/data model
//! only — it does not implement content-addressed storage, P2P
//! replication, or on-chain governance voting. Those are tracked as
//! Phase 6 roadmap items.
//!
//! Design constraints carried over deliberately from the whitepaper:
//!   - Identity reuses the wallet address (no separate social account).
//!   - Visibility is controlled by the same view-key model used for
//!     financial transactions (Attributed vs Shielded authorship).
//!   - An AI Agent may act as a delegated author under a policy object
//!     that is scoped to social actions only, never financial ones.

use std::collections::HashMap;

/// Maximum length of a post body, in bytes. Kept conservative for an
/// early planning model; real storage limits depend on the
/// content-addressed storage backend chosen in Phase 6.
pub const MAX_POST_BYTES: usize = 2_000;

/// How a post's authorship is disclosed to the network.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    /// Authorship is publicly attributable to the posting address.
    Attributed,
    /// Authorship is shielded; only holders of the relevant view key
    /// (see `ViewKeyGrant`) can attribute the post to its author.
    Shielded,
}

/// A decentralised social post.
///
/// `author` is an ABYSS wallet address — there is no separate social
/// account identifier, consistent with the whitepaper's identity model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Post {
    pub id: PostId,
    pub author: String,
    pub body: String,
    pub visibility: Visibility,
    pub created_at_ms: u64,
    /// Optional reference to a post this one replies to.
    pub in_reply_to: Option<PostId>,
    pub authored_by_agent: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PostId(pub u64);

impl Post {
    pub fn new(
        id: PostId,
        author: impl Into<String>,
        body: impl Into<String>,
        visibility: Visibility,
        created_at_ms: u64,
    ) -> Result<Self, SocialError> {
        let body = body.into();
        if body.is_empty() {
            return Err(SocialError::EmptyPost);
        }
        if body.len() > MAX_POST_BYTES {
            return Err(SocialError::PostTooLong {
                max_bytes: MAX_POST_BYTES,
                actual_bytes: body.len(),
            });
        }

        Ok(Self {
            id,
            author: author.into(),
            body,
            visibility,
            created_at_ms,
            in_reply_to: None,
            authored_by_agent: false,
        })
    }

    pub fn as_reply_to(mut self, parent: PostId) -> Self {
        self.in_reply_to = Some(parent);
        self
    }

    pub fn as_agent_authored(mut self) -> Self {
        self.authored_by_agent = true;
        self
    }

    /// Whether `viewer` is permitted to see who authored this post.
    /// Attributed posts are visible to everyone; shielded posts require
    /// either being the author or holding a valid view-key grant for it.
    pub fn author_visible_to(&self, viewer: &str, grants: &ViewKeyRegistry) -> bool {
        match self.visibility {
            Visibility::Attributed => true,
            Visibility::Shielded => viewer == self.author || grants.has_grant(&self.id, viewer),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SocialError {
    EmptyPost,
    PostTooLong {
        max_bytes: usize,
        actual_bytes: usize,
    },
    AgentLacksSocialPermission,
    AgentExceedsPostingRate {
        limit_per_window: u32,
        window_seconds: u64,
    },
    UnknownPost(PostId),
    UnauthorisedDisclosure,
}

/// Registry of view-key grants for shielded posts: who, besides the
/// author, has been given the ability to see the true author of a
/// specific shielded post. Mirrors the financial view-key model
/// (whitepaper Section 5.3) but scoped to the social layer.
#[derive(Clone, Debug, Default)]
pub struct ViewKeyRegistry {
    grants: HashMap<PostId, Vec<String>>,
}

impl ViewKeyRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Author grants `viewer` the ability to see the true authorship of
    /// `post_id`. Only meaningful for shielded posts; calling this for
    /// an attributed post is harmless but redundant.
    pub fn grant(&mut self, post_id: PostId, viewer: impl Into<String>) {
        self.grants.entry(post_id).or_default().push(viewer.into());
    }

    pub fn has_grant(&self, post_id: &PostId, viewer: &str) -> bool {
        self.grants
            .get(post_id)
            .map(|viewers| viewers.iter().any(|v| v == viewer))
            .unwrap_or(false)
    }
}

/// Scoped permission policy for an AI Agent acting on the social layer.
///
/// This is intentionally a *separate* policy object from any financial
/// Agent permission (e.g. abyss-wallet's AgentPermission /
/// agent_trade_limit). An Agent authorised to post or curate content
/// must not be able to leverage that authorisation to move funds, and
/// vice versa — see whitepaper Section 9.3.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentSocialPolicy {
    pub can_post: bool,
    pub can_reply: bool,
    pub can_repost: bool,
    /// Maximum number of posts (including replies/reposts) the Agent
    /// may create within `rate_window_seconds`.
    pub max_posts_per_window: u32,
    pub rate_window_seconds: u64,
}

impl AgentSocialPolicy {
    pub fn none() -> Self {
        Self {
            can_post: false,
            can_reply: false,
            can_repost: false,
            max_posts_per_window: 0,
            rate_window_seconds: 0,
        }
    }

    pub fn curator_default() -> Self {
        Self {
            can_post: false,
            can_reply: false,
            can_repost: true,
            max_posts_per_window: 10,
            rate_window_seconds: 3_600,
        }
    }

    pub fn full_posting_default() -> Self {
        Self {
            can_post: true,
            can_reply: true,
            can_repost: true,
            max_posts_per_window: 5,
            rate_window_seconds: 3_600,
        }
    }
}

/// Tracks how many posts an Agent has made within the current rate
/// window, to enforce `AgentSocialPolicy::max_posts_per_window`.
#[derive(Clone, Debug, Default)]
pub struct AgentActivityWindow {
    window_start_ms: u64,
    posts_in_window: u32,
}

impl AgentActivityWindow {
    pub fn new(window_start_ms: u64) -> Self {
        Self {
            window_start_ms,
            posts_in_window: 0,
        }
    }

    /// Checks the policy, and if allowed, records one post against the
    /// window. `now_ms` is used to roll the window over once
    /// `rate_window_seconds` has elapsed.
    pub fn record_post(
        &mut self,
        policy: &AgentSocialPolicy,
        action: AgentSocialAction,
        now_ms: u64,
    ) -> Result<(), SocialError> {
        let permitted = match action {
            AgentSocialAction::Post => policy.can_post,
            AgentSocialAction::Reply => policy.can_reply,
            AgentSocialAction::Repost => policy.can_repost,
        };
        if !permitted {
            return Err(SocialError::AgentLacksSocialPermission);
        }

        let window_elapsed_ms = now_ms.saturating_sub(self.window_start_ms);
        if window_elapsed_ms >= policy.rate_window_seconds.saturating_mul(1_000) {
            self.window_start_ms = now_ms;
            self.posts_in_window = 0;
        }

        if self.posts_in_window >= policy.max_posts_per_window {
            return Err(SocialError::AgentExceedsPostingRate {
                limit_per_window: policy.max_posts_per_window,
                window_seconds: policy.rate_window_seconds,
            });
        }

        self.posts_in_window += 1;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentSocialAction {
    Post,
    Reply,
    Repost,
}

/// An in-memory feed used for devnet demonstration purposes only.
/// Production storage is content-addressed and replicated (Phase 6);
/// this is a planning-stage stand-in, mirroring how `abyss-core`'s
/// devnet Chain is an in-memory stand-in for persistent ledger storage.
#[derive(Clone, Debug, Default)]
pub struct DevFeed {
    posts: Vec<Post>,
    next_id: u64,
}

impl DevFeed {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn publish(
        &mut self,
        author: impl Into<String>,
        body: impl Into<String>,
        visibility: Visibility,
        created_at_ms: u64,
    ) -> Result<PostId, SocialError> {
        let id = PostId(self.next_id);
        let post = Post::new(id, author, body, visibility, created_at_ms)?;
        self.posts.push(post);
        self.next_id += 1;
        Ok(id)
    }

    pub fn reply(
        &mut self,
        author: impl Into<String>,
        body: impl Into<String>,
        visibility: Visibility,
        created_at_ms: u64,
        parent: PostId,
    ) -> Result<PostId, SocialError> {
        if !self.posts.iter().any(|p| p.id == parent) {
            return Err(SocialError::UnknownPost(parent));
        }
        let id = PostId(self.next_id);
        let post = Post::new(id, author, body, visibility, created_at_ms)?.as_reply_to(parent);
        self.posts.push(post);
        self.next_id += 1;
        Ok(id)
    }

    pub fn get(&self, id: PostId) -> Option<&Post> {
        self.posts.iter().find(|p| p.id == id)
    }

    pub fn by_author(&self, author: &str) -> Vec<&Post> {
        self.posts.iter().filter(|p| p.author == author).collect()
    }

    pub fn replies_to(&self, parent: PostId) -> Vec<&Post> {
        self.posts
            .iter()
            .filter(|p| p.in_reply_to == Some(parent))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.posts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.posts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publishes_an_attributed_post() {
        let mut feed = DevFeed::new();
        let id = feed
            .publish("abyss1alice", "hello abyss", Visibility::Attributed, 1_000)
            .unwrap();

        let post = feed.get(id).unwrap();
        assert_eq!(post.author, "abyss1alice");
        assert_eq!(post.visibility, Visibility::Attributed);
    }

    #[test]
    fn rejects_empty_post() {
        let mut feed = DevFeed::new();
        let result = feed.publish("abyss1alice", "", Visibility::Attributed, 1_000);
        assert_eq!(result, Err(SocialError::EmptyPost));
    }

    #[test]
    fn rejects_post_exceeding_max_length() {
        let mut feed = DevFeed::new();
        let too_long = "a".repeat(MAX_POST_BYTES + 1);
        let result = feed.publish("abyss1alice", too_long, Visibility::Attributed, 1_000);
        assert_eq!(
            result,
            Err(SocialError::PostTooLong {
                max_bytes: MAX_POST_BYTES,
                actual_bytes: MAX_POST_BYTES + 1
            })
        );
    }

    #[test]
    fn shielded_post_hides_author_from_strangers() {
        let mut feed = DevFeed::new();
        let id = feed
            .publish(
                "abyss1alice",
                "secret thoughts",
                Visibility::Shielded,
                1_000,
            )
            .unwrap();
        let post = feed.get(id).unwrap();
        let grants = ViewKeyRegistry::new();

        assert!(!post.author_visible_to("abyss1bob", &grants));
        assert!(post.author_visible_to("abyss1alice", &grants)); // author always sees self
    }

    #[test]
    fn view_key_grant_discloses_shielded_author() {
        let mut feed = DevFeed::new();
        let id = feed
            .publish(
                "abyss1alice",
                "secret thoughts",
                Visibility::Shielded,
                1_000,
            )
            .unwrap();
        let post = feed.get(id).unwrap();

        let mut grants = ViewKeyRegistry::new();
        assert!(!post.author_visible_to("abyss1auditor", &grants));

        grants.grant(id, "abyss1auditor");
        assert!(post.author_visible_to("abyss1auditor", &grants));
        // Granting one viewer does not disclose to others.
        assert!(!post.author_visible_to("abyss1bob", &grants));
    }

    #[test]
    fn attributed_post_is_visible_to_everyone_without_a_grant() {
        let mut feed = DevFeed::new();
        let id = feed
            .publish("abyss1alice", "public post", Visibility::Attributed, 1_000)
            .unwrap();
        let post = feed.get(id).unwrap();
        let grants = ViewKeyRegistry::new();

        assert!(post.author_visible_to("abyss1bob", &grants));
        assert!(post.author_visible_to("anyone-at-all", &grants));
    }

    #[test]
    fn replies_link_to_their_parent() {
        let mut feed = DevFeed::new();
        let root = feed
            .publish("abyss1alice", "root post", Visibility::Attributed, 1_000)
            .unwrap();
        let reply = feed
            .reply("abyss1bob", "a reply", Visibility::Attributed, 2_000, root)
            .unwrap();

        let replies = feed.replies_to(root);
        assert_eq!(replies.len(), 1);
        assert_eq!(replies[0].id, reply);
    }

    #[test]
    fn reply_to_unknown_post_is_rejected() {
        let mut feed = DevFeed::new();
        let result = feed.reply(
            "abyss1bob",
            "a reply",
            Visibility::Attributed,
            2_000,
            PostId(999),
        );
        assert_eq!(result, Err(SocialError::UnknownPost(PostId(999))));
    }

    #[test]
    fn by_author_filters_correctly() {
        let mut feed = DevFeed::new();
        feed.publish("abyss1alice", "post 1", Visibility::Attributed, 1_000)
            .unwrap();
        feed.publish("abyss1bob", "post 2", Visibility::Attributed, 2_000)
            .unwrap();
        feed.publish("abyss1alice", "post 3", Visibility::Attributed, 3_000)
            .unwrap();

        assert_eq!(feed.by_author("abyss1alice").len(), 2);
        assert_eq!(feed.by_author("abyss1bob").len(), 1);
    }

    // ── Agent social policy tests ──

    #[test]
    fn agent_with_no_policy_cannot_post() {
        let policy = AgentSocialPolicy::none();
        let mut window = AgentActivityWindow::new(0);

        let result = window.record_post(&policy, AgentSocialAction::Post, 0);
        assert_eq!(result, Err(SocialError::AgentLacksSocialPermission));
    }

    #[test]
    fn curator_policy_allows_repost_but_not_post() {
        let policy = AgentSocialPolicy::curator_default();
        let mut window = AgentActivityWindow::new(0);

        assert_eq!(
            window.record_post(&policy, AgentSocialAction::Post, 0),
            Err(SocialError::AgentLacksSocialPermission)
        );

        let mut window2 = AgentActivityWindow::new(0);
        assert_eq!(
            window2.record_post(&policy, AgentSocialAction::Repost, 0),
            Ok(())
        );
    }

    #[test]
    fn agent_posting_rate_limit_is_enforced() {
        let policy = AgentSocialPolicy {
            can_post: true,
            can_reply: true,
            can_repost: true,
            max_posts_per_window: 2,
            rate_window_seconds: 3_600,
        };
        let mut window = AgentActivityWindow::new(0);

        assert_eq!(
            window.record_post(&policy, AgentSocialAction::Post, 0),
            Ok(())
        );
        assert_eq!(
            window.record_post(&policy, AgentSocialAction::Post, 100),
            Ok(())
        );
        // third post within the same window exceeds the cap
        assert_eq!(
            window.record_post(&policy, AgentSocialAction::Post, 200),
            Err(SocialError::AgentExceedsPostingRate {
                limit_per_window: 2,
                window_seconds: 3_600
            })
        );
    }

    #[test]
    fn agent_posting_rate_resets_after_window_elapses() {
        let policy = AgentSocialPolicy {
            can_post: true,
            can_reply: true,
            can_repost: true,
            max_posts_per_window: 1,
            rate_window_seconds: 60, // 60 seconds
        };
        let mut window = AgentActivityWindow::new(0);

        assert_eq!(
            window.record_post(&policy, AgentSocialAction::Post, 0),
            Ok(())
        );
        // still within the 60s window -> rejected
        assert_eq!(
            window.record_post(&policy, AgentSocialAction::Post, 30_000),
            Err(SocialError::AgentExceedsPostingRate {
                limit_per_window: 1,
                window_seconds: 60
            })
        );
        // 61s later, window has rolled over -> allowed again
        assert_eq!(
            window.record_post(&policy, AgentSocialAction::Post, 61_000),
            Ok(())
        );
    }

    #[test]
    fn agent_authored_flag_is_tracked_on_the_post() {
        let post = Post::new(
            PostId(0),
            "abyss1alice",
            "agent generated this",
            Visibility::Attributed,
            1_000,
        )
        .unwrap()
        .as_agent_authored();

        assert!(post.authored_by_agent);
    }
}
