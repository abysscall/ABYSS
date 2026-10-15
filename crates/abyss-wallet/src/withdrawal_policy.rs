//! Wallet-side evaluation of a proposed transfer-protection policy.
//!
//! This module is intentionally a local, deterministic policy model. It does
//! not delay, settle, cancel, or reverse a blockchain transaction. Those
//! actions need a future versioned protocol action and Core execution rules.

/// The recipient assessment supplied to the Wallet policy evaluator.
///
/// A future risk engine may derive this assessment, but its implementation is
/// deliberately outside the Wallet domain. The Wallet receives an explicit
/// result and applies the account owner's configured policy deterministically.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipientAssessment {
    Trusted,
    Unknown,
    Suspicious,
}

/// A non-settling action the Wallet must require before it creates a transfer.
///
/// `ProtectedHold` means that a future protected-transfer protocol action would
/// need to hold funds *before* final settlement. It is not a promise that the
/// current `Transaction` type can be reversed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferProtectionRequirement {
    /// Normal signing and protocol checks still apply.
    StandardChecks,
    /// The account holder must explicitly approve the proposed transfer.
    ExplicitConfirmation,
    /// A future pre-settlement action must require a confirmation during this
    /// window. It must auto-cancel when no confirmation is received.
    ProtectedHold {
        window_seconds: u32,
        auto_cancel_if_unconfirmed: bool,
    },
}

/// Account-owned transfer-protection choices.
///
/// There is intentionally no `Default` implementation: choosing a delay or
/// hold window is a user/product policy decision, not an implicit Wallet
/// entitlement derived from an AC balance or an AI recommendation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferProtectionPolicy {
    pub trusted_recipient: TransferProtectionRequirement,
    pub unknown_recipient: TransferProtectionRequirement,
    pub suspicious_recipient: TransferProtectionRequirement,
}

impl TransferProtectionPolicy {
    /// The non-binding policy discussed by AC Economy and Wallet:
    /// trusted recipients receive normal checks, unknown recipients require
    /// explicit confirmation, and suspicious recipients enter a five-minute
    /// pre-settlement protection window if/when the protocol supports it.
    pub const fn working_draft() -> Self {
        Self {
            trusted_recipient: TransferProtectionRequirement::StandardChecks,
            unknown_recipient: TransferProtectionRequirement::ExplicitConfirmation,
            suspicious_recipient: TransferProtectionRequirement::ProtectedHold {
                window_seconds: 5 * 60,
                auto_cancel_if_unconfirmed: true,
            },
        }
    }

    pub fn requirement_for(
        &self,
        assessment: RecipientAssessment,
    ) -> TransferProtectionRequirement {
        match assessment {
            RecipientAssessment::Trusted => self.trusted_recipient,
            RecipientAssessment::Unknown => self.unknown_recipient,
            RecipientAssessment::Suspicious => self.suspicious_recipient,
        }
    }

    pub fn validate(&self) -> Result<(), TransferProtectionPolicyError> {
        for requirement in [
            self.trusted_recipient,
            self.unknown_recipient,
            self.suspicious_recipient,
        ] {
            if let TransferProtectionRequirement::ProtectedHold {
                window_seconds,
                auto_cancel_if_unconfirmed,
            } = requirement
            {
                if window_seconds == 0 {
                    return Err(TransferProtectionPolicyError::ProtectedHoldRequiresWindow);
                }
                if !auto_cancel_if_unconfirmed {
                    return Err(TransferProtectionPolicyError::ProtectedHoldRequiresExpiryRule);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferProtectionPolicyError {
    ProtectedHoldRequiresWindow,
    ProtectedHoldRequiresExpiryRule,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn working_draft_protects_unknown_and_suspicious_recipients_differently() {
        let policy = TransferProtectionPolicy::working_draft();

        assert_eq!(
            policy.requirement_for(RecipientAssessment::Trusted),
            TransferProtectionRequirement::StandardChecks
        );
        assert_eq!(
            policy.requirement_for(RecipientAssessment::Unknown),
            TransferProtectionRequirement::ExplicitConfirmation
        );
        assert_eq!(
            policy.requirement_for(RecipientAssessment::Suspicious),
            TransferProtectionRequirement::ProtectedHold {
                window_seconds: 300,
                auto_cancel_if_unconfirmed: true,
            }
        );
        assert_eq!(policy.validate(), Ok(()));
    }

    #[test]
    fn protected_hold_cannot_be_configured_without_an_expiry_rule() {
        let policy = TransferProtectionPolicy {
            trusted_recipient: TransferProtectionRequirement::StandardChecks,
            unknown_recipient: TransferProtectionRequirement::ExplicitConfirmation,
            suspicious_recipient: TransferProtectionRequirement::ProtectedHold {
                window_seconds: 300,
                auto_cancel_if_unconfirmed: false,
            },
        };

        assert_eq!(
            policy.validate(),
            Err(TransferProtectionPolicyError::ProtectedHoldRequiresExpiryRule)
        );
    }
}
