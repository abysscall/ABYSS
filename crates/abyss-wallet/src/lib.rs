//! Wallet-domain primitives for ABYSS.

pub mod account;
pub mod agent_policy;

pub use account::{WalletAccount, WalletError};
pub use agent_policy::{AgentPermission, AgentPolicy};
