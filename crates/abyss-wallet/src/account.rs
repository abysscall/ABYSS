use abyss_core::{Address, Coin, Transaction};
use abyss_crypto::{DevKeypair, PublicKey};

use crate::agent_policy::AgentPolicy;

#[derive(Clone, Debug)]
pub struct WalletAccount {
    label: String,
    keypair: DevKeypair,
    agent_policy: AgentPolicy,
}

impl WalletAccount {
    pub fn generate(label: impl Into<String>) -> Self {
        let label = label.into();
        Self {
            keypair: DevKeypair::generate(&label),
            agent_policy: AgentPolicy::default(),
            label,
        }
    }

    pub fn from_dev_seed(label: impl Into<String>, seed: impl AsRef<[u8]>) -> Self {
        Self {
            label: label.into(),
            keypair: DevKeypair::from_seed(seed),
            agent_policy: AgentPolicy::default(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn address(&self) -> Address {
        self.keypair.dev_address()
    }

    pub fn public_key(&self) -> &PublicKey {
        self.keypair.public()
    }

    pub fn agent_policy(&self) -> &AgentPolicy {
        &self.agent_policy
    }

    pub fn agent_policy_mut(&mut self) -> &mut AgentPolicy {
        &mut self.agent_policy
    }

    pub fn create_payment(&self, to: Address, amount: Coin, fee: Coin, nonce: u64) -> Transaction {
        Transaction::new(self.address(), to, amount, fee, nonce)
    }

    pub fn create_agent_payment(
        &self,
        to: Address,
        amount: Coin,
        fee: Coin,
        nonce: u64,
    ) -> Result<Transaction, WalletError> {
        if !self.agent_policy.transaction_allowed(amount) {
            return Err(WalletError::PolicyRejected);
        }

        Ok(Transaction::new(self.address(), to, amount, fee, nonce))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WalletError {
    PolicyRejected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_can_create_payment() {
        let account = WalletAccount::from_dev_seed("alice", "alice");
        let tx = account.create_payment(
            Address::new("bob").unwrap(),
            Coin::from_ac(1).unwrap(),
            Coin::ZERO,
            0,
        );

        assert_eq!(tx.from, account.address());
    }

    #[test]
    fn policy_can_reject_large_agent_payment() {
        let mut account = WalletAccount::from_dev_seed("alice", "alice");
        account
            .agent_policy_mut()
            .grant(crate::AgentPermission::ExecuteLimitedTrades);
        account
            .agent_policy_mut()
            .set_agent_trade_limit(Coin::from_ac(10).unwrap());

        let rejected = account.create_agent_payment(
            Address::new("bob").unwrap(),
            Coin::from_ac(11).unwrap(),
            Coin::ZERO,
            0,
        );

        assert_eq!(rejected, Err(WalletError::PolicyRejected));
    }
}
