use std::collections::BTreeMap;

use crate::address::Address;
use crate::block::Block;
use crate::coin::Coin;
use crate::genesis::{GenesisConfig, GenesisError};
use crate::hashing::{dev_hash, Hash256};
use crate::transaction::{Transaction, TransactionId};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChainConfig {
    pub chain_id: String,
    pub block_time_ms: u64,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            chain_id: "abyss-devnet-1".to_string(),
            block_time_ms: 1_000,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Chain {
    config: ChainConfig,
    blocks: Vec<Block>,
    balances: BTreeMap<Address, Coin>,
    nonces: BTreeMap<Address, u64>,
}

impl Chain {
    pub fn from_genesis(
        config: ChainConfig,
        genesis: GenesisConfig,
        timestamp_ms: u64,
    ) -> Result<Self, ApplyError> {
        genesis.validate()?;

        let mut balances = BTreeMap::new();
        for (address, amount) in genesis.allocations {
            let current = balances.get(&address).copied().unwrap_or(Coin::ZERO);
            let next = current
                .checked_add(amount)
                .ok_or(ApplyError::BalanceOverflow)?;
            balances.insert(address, next);
        }

        Ok(Self {
            config,
            blocks: vec![Block::genesis(timestamp_ms, "genesis")],
            balances,
            nonces: BTreeMap::new(),
        })
    }

    pub fn config(&self) -> &ChainConfig {
        &self.config
    }

    pub fn height(&self) -> u64 {
        self.blocks
            .last()
            .map(|block| block.header.height)
            .unwrap_or(0)
    }

    pub fn tip_hash(&self) -> Hash256 {
        self.blocks
            .last()
            .map(Block::hash)
            .unwrap_or_else(|| dev_hash(&"abyss:empty-chain"))
    }

    pub fn balance_of(&self, address: &Address) -> Coin {
        self.balances.get(address).copied().unwrap_or(Coin::ZERO)
    }

    pub fn next_nonce(&self, address: &Address) -> u64 {
        self.nonces.get(address).copied().unwrap_or(0)
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn snapshot_state(&self) -> (BTreeMap<String, u64>, BTreeMap<String, u64>) {
        let balances = self
            .balances
            .iter()
            .map(|(address, coin)| (address.as_str().to_string(), coin.micro_ac()))
            .collect();
        let nonces = self
            .nonces
            .iter()
            .map(|(address, nonce)| (address.as_str().to_string(), *nonce))
            .collect();
        (balances, nonces)
    }

    pub fn restore_state(
        mut self,
        balances: BTreeMap<String, u64>,
        nonces: BTreeMap<String, u64>,
    ) -> Result<Self, ApplyError> {
        self.balances = BTreeMap::new();
        for (address, micro_ac) in balances {
            let coin = Coin::from_micro_ac(micro_ac).ok_or(ApplyError::BalanceOverflow)?;
            self.balances.insert(
                Address::new(address).map_err(|_| ApplyError::BalanceOverflow)?,
                coin,
            );
        }
        self.nonces = BTreeMap::new();
        for (address, nonce) in nonces {
            self.nonces.insert(
                Address::new(address).map_err(|_| ApplyError::BalanceOverflow)?,
                nonce,
            );
        }
        Ok(self)
    }

    pub fn from_persisted(
        config: ChainConfig,
        blocks: Vec<Block>,
        balances: BTreeMap<String, u64>,
        nonces: BTreeMap<String, u64>,
    ) -> Result<Self, ApplyError> {
        Self {
            config,
            blocks,
            balances: BTreeMap::new(),
            nonces: BTreeMap::new(),
        }
        .restore_state(balances, nonces)
    }

    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<TransactionId, ApplyError> {
        validate_transaction(tx)?;

        let expected_nonce = self.next_nonce(&tx.from);
        if tx.nonce != expected_nonce {
            return Err(ApplyError::InvalidNonce {
                expected: expected_nonce,
                actual: tx.nonce,
            });
        }

        let debit = tx.total_debit().ok_or(ApplyError::BalanceOverflow)?;
        let sender_balance = self.balance_of(&tx.from);
        let new_sender_balance =
            sender_balance
                .checked_sub(debit)
                .ok_or(ApplyError::InsufficientFunds {
                    available: sender_balance,
                    required: debit,
                })?;

        let receiver_balance = self.balance_of(&tx.to);
        let new_receiver_balance = receiver_balance
            .checked_add(tx.amount)
            .ok_or(ApplyError::BalanceOverflow)?;

        self.balances.insert(tx.from.clone(), new_sender_balance);
        self.balances.insert(tx.to.clone(), new_receiver_balance);
        self.nonces.insert(tx.from.clone(), expected_nonce + 1);

        Ok(tx.id())
    }

    pub fn produce_block(
        &mut self,
        proposer: impl Into<String>,
        timestamp_ms: u64,
        transactions: Vec<Transaction>,
    ) -> Result<&Block, ApplyError> {
        let mut staged = self.clone();
        for tx in &transactions {
            staged.apply_transaction(tx)?;
        }

        let state_root = staged.state_root();
        let block = Block::new(
            self.height() + 1,
            self.tip_hash(),
            state_root,
            timestamp_ms,
            proposer,
            transactions,
        );

        staged.blocks.push(block);
        *self = staged;
        Ok(self.blocks.last().expect("block was just pushed"))
    }

    pub fn state_root(&self) -> Hash256 {
        dev_hash(&(&self.balances, &self.nonces))
    }
}

fn validate_transaction(tx: &Transaction) -> Result<(), ApplyError> {
    if tx.amount.is_zero() {
        return Err(ApplyError::ZeroAmount);
    }

    if tx.from == tx.to {
        return Err(ApplyError::SelfTransfer);
    }

    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplyError {
    BalanceOverflow,
    Genesis(GenesisError),
    InsufficientFunds { available: Coin, required: Coin },
    InvalidNonce { expected: u64, actual: u64 },
    SelfTransfer,
    ZeroAmount,
}

impl From<GenesisError> for ApplyError {
    fn from(value: GenesisError) -> Self {
        Self::Genesis(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chain() -> Chain {
        let treasury = Address::new("treasury").unwrap();
        Chain::from_genesis(
            ChainConfig::default(),
            GenesisConfig::single_treasury(treasury),
            0,
        )
        .unwrap()
    }

    #[test]
    fn genesis_starts_at_height_zero() {
        let chain = chain();
        assert_eq!(chain.height(), 0);
    }

    #[test]
    fn transfer_updates_balances_and_nonce() {
        let mut chain = chain();
        let treasury = Address::new("treasury").unwrap();
        let alice = Address::new("alice").unwrap();
        let tx = Transaction::new(
            treasury.clone(),
            alice.clone(),
            Coin::from_ac(10).unwrap(),
            Coin::from_micro_ac(100).unwrap(),
            0,
        );

        chain.produce_block("validator-1", 1_000, vec![tx]).unwrap();

        assert_eq!(chain.height(), 1);
        assert_eq!(chain.next_nonce(&treasury), 1);
        assert_eq!(chain.balance_of(&alice), Coin::from_ac(10).unwrap());
    }

    #[test]
    fn rejects_replay_nonce() {
        let mut chain = chain();
        let treasury = Address::new("treasury").unwrap();
        let alice = Address::new("alice").unwrap();
        let tx = Transaction::new(treasury, alice, Coin::from_ac(1).unwrap(), Coin::ZERO, 1);

        assert!(matches!(
            chain.produce_block("validator-1", 1_000, vec![tx]),
            Err(ApplyError::InvalidNonce {
                expected: 0,
                actual: 1
            })
        ));
    }

    #[test]
    fn block_application_is_atomic() {
        let mut chain = chain();
        let treasury = Address::new("treasury").unwrap();
        let alice = Address::new("alice").unwrap();
        let bob = Address::new("bob").unwrap();
        let valid = Transaction::new(
            treasury.clone(),
            alice.clone(),
            Coin::from_ac(1).unwrap(),
            Coin::ZERO,
            0,
        );
        let invalid = Transaction::new(
            treasury.clone(),
            bob,
            Coin::from_ac(1).unwrap(),
            Coin::ZERO,
            0,
        );

        assert!(chain
            .produce_block("validator-1", 1_000, vec![valid, invalid])
            .is_err());

        assert_eq!(chain.height(), 0);
        assert_eq!(chain.next_nonce(&treasury), 0);
        assert_eq!(chain.balance_of(&alice), Coin::ZERO);
    }
}
