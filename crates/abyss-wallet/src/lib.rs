//! Wallet-domain primitives for ABYSS.

pub mod account;
pub mod agent_policy;
pub mod withdrawal_policy;

pub use account::{WalletAccount, WalletError};
pub use agent_policy::{AgentPermission, AgentPolicy};
pub use withdrawal_policy::{
    RecipientAssessment, TransferProtectionPolicy, TransferProtectionPolicyError,
    TransferProtectionRequirement,
};
