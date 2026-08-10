//! Chain — block sequencing and consensus integration.
//!
//! Per ADR-0005: Chain owns consensus and block sequencing.
//! All account state is delegated to State (state.rs).
//!
//! Per ADR-0016: This file is being hardened toward BFT consensus.
//! New method apply_block() separates the proposer path (produce_block)
//! from the validator path (receive and verify a block from the network).

use crate::address::Address;
use crate::block::Block;
use crate::coin::Coin;
use crate::genesis::{GenesisConfig, GenesisError};
use crate::hashing::{dev_hash, Hash256};
use crate::state::{State, StateError};
use crate::transaction::{Transaction, TransactionId};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChainConfig {
    pub chain_id: String,
    pub block_time_ms: u64,
    /// Maximum number of validators (enforced at genesis, extended by governance later).
    pub max_validators: u32,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            chain_id: "abyss-devnet-1".to_string(),
            block_time_ms: 1_000,
            max_validators: 21,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Chain {
    config: ChainConfig,
    blocks: Vec<Block>,
    /// All account state lives here, not directly on Chain.
    state: State,
}

impl Chain {
    // ── Construction ──────────────────────────────────────────────────────────

    pub fn from_genesis(
        config: ChainConfig,
        genesis: GenesisConfig,
        timestamp_ms: u64,
    ) -> Result<Self, ApplyError> {
        genesis.validate()?;

        let mut state = State::new();
        for (address, amount) in genesis.allocations {
            state.credit(&address, amount)?;
        }

        Ok(Self {
            config,
            blocks: vec![Block::genesis(timestamp_ms, "genesis")],
            state,
        })
    }

    /// Restore a chain from persisted snapshot (called by storage.rs).
    pub fn from_persisted(
        config: ChainConfig,
        blocks: Vec<Block>,
        balances: std::collections::BTreeMap<String, u64>,
        nonces: std::collections::BTreeMap<String, u64>,
    ) -> Result<Self, ApplyError> {
        let state = State::from_raw(balances, nonces)?;
        Ok(Self {
            config,
            blocks,
            state,
        })
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn config(&self) -> &ChainConfig {
        &self.config
    }

    pub fn height(&self) -> u64 {
        self.blocks.last().map(|b| b.header.height).unwrap_or(0)
    }

    pub fn tip_hash(&self) -> Hash256 {
        self.blocks
            .last()
            .map(Block::hash)
            .unwrap_or_else(|| dev_hash(&"abyss:empty-chain"))
    }

    pub fn balance_of(&self, address: &Address) -> Coin {
        self.state.balance_of(address)
    }

    pub fn next_nonce(&self, address: &Address) -> u64 {
        self.state.nonce_of(address)
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn state_root(&self) -> Hash256 {
        self.state.root()
    }

    // ── State snapshot (for storage.rs compatibility) ─────────────────────────

    pub fn snapshot_state(
        &self,
    ) -> (
        std::collections::BTreeMap<String, u64>,
        std::collections::BTreeMap<String, u64>,
    ) {
        self.state.to_raw()
    }

    pub fn restore_state(
        mut self,
        balances: std::collections::BTreeMap<String, u64>,
        nonces: std::collections::BTreeMap<String, u64>,
    ) -> Result<Self, ApplyError> {
        self.state = State::from_raw(balances, nonces)?;
        Ok(self)
    }

    // ── Transaction execution ─────────────────────────────────────────────────

    /// Execute a single transaction against state. Called from both
    /// produce_block (proposer) and apply_block (validator).
    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<TransactionId, ApplyError> {
        // Basic validation
        if tx.amount.is_zero() {
            return Err(ApplyError::ZeroAmount);
        }
        if tx.from == tx.to {
            return Err(ApplyError::SelfTransfer);
        }

        // Nonce check
        let expected = self.state.nonce_of(&tx.from);
        if tx.nonce != expected {
            return Err(ApplyError::InvalidNonce {
                expected,
                actual: tx.nonce,
            });
        }

        // Debit sender (amount + fee)
        let debit = tx.total_debit().ok_or(ApplyError::BalanceOverflow)?;
        self.state.debit(&tx.from, debit)?;

        // Credit receiver
        self.state.credit(&tx.to, tx.amount)?;

        // Advance sender nonce
        self.state.increment_nonce(&tx.from);

        Ok(tx.id())
    }

    // ── Block production (proposer path) ──────────────────────────────────────

    /// Propose and immediately apply a new block.
    /// Used by the current single-validator devnet.
    /// In BFT mode, the proposer calls this; validators call apply_block().
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

        let block = Block::new(
            self.height() + 1,
            self.tip_hash(),
            staged.state_root(),
            timestamp_ms,
            proposer,
            transactions,
        );

        staged.blocks.push(block);
        *self = staged;
        Ok(self.blocks.last().expect("block was just pushed"))
    }

    // ── Block application (validator path — BFT Stage 1 prerequisite) ─────────

    /// Apply a block received from the network (validator path).
    ///
    /// Verifies:
    ///   - block height is exactly tip + 1
    ///   - previous_hash matches current tip
    ///   - all transactions execute successfully
    ///   - resulting state_root matches block header
    ///
    /// This is the method that BFT consensus will call after a block
    /// receives 2/3+ PreCommit votes. Currently used for single-node
    /// testing; the signature verification step is a placeholder.
    pub fn apply_block(&mut self, block: Block) -> Result<(), ApplyError> {
        // Height continuity
        let expected_height = self.height() + 1;
        if block.header.height != expected_height {
            return Err(ApplyError::UnexpectedBlockHeight {
                expected: expected_height,
                actual: block.header.height,
            });
        }

        // Chain linkage
        if block.header.previous_hash != self.tip_hash() {
            return Err(ApplyError::PreviousHashMismatch);
        }

        // Execute transactions on a staged copy (atomic application — ADR-0004)
        let mut staged = self.clone();
        for tx in &block.transactions {
            staged.apply_transaction(tx)?;
        }

        // State root verification
        let computed_root = staged.state_root();
        if computed_root != block.header.state_root {
            return Err(ApplyError::StateRootMismatch {
                expected: block.header.state_root,
                computed: computed_root,
            });
        }

        // Commit
        staged.blocks.push(block);
        *self = staged;
        Ok(())
    }
}

// ── Error types ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplyError {
    BalanceOverflow,
    Genesis(GenesisError),
    InsufficientFunds {
        available: Coin,
        required: Coin,
    },
    InvalidNonce {
        expected: u64,
        actual: u64,
    },
    PreviousHashMismatch,
    SelfTransfer,
    StateRootMismatch {
        expected: Hash256,
        computed: Hash256,
    },
    UnexpectedBlockHeight {
        expected: u64,
        actual: u64,
    },
    ZeroAmount,
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BalanceOverflow => write!(f, "balance overflow"),
            Self::Genesis(e) => write!(f, "genesis error: {e:?}"),
            Self::InsufficientFunds {
                available,
                required,
            } => write!(f, "insufficient funds: have {available}, need {required}"),
            Self::InvalidNonce { expected, actual } => {
                write!(f, "invalid nonce: expected {expected}, got {actual}")
            }
            Self::PreviousHashMismatch => write!(f, "previous_hash does not match tip"),
            Self::SelfTransfer => write!(f, "sender and receiver are the same address"),
            Self::StateRootMismatch { .. } => {
                write!(f, "state root mismatch after block execution")
            }
            Self::UnexpectedBlockHeight { expected, actual } => write!(
                f,
                "unexpected block height: expected {expected}, got {actual}"
            ),
            Self::ZeroAmount => write!(f, "transaction amount is zero"),
        }
    }
}

impl From<GenesisError> for ApplyError {
    fn from(e: GenesisError) -> Self {
        Self::Genesis(e)
    }
}

impl From<StateError> for ApplyError {
    fn from(e: StateError) -> Self {
        match e {
            StateError::BalanceOverflow => Self::BalanceOverflow,
            StateError::InsufficientFunds {
                available,
                required,
            } => Self::InsufficientFunds {
                available,
                required,
            },
            StateError::Address(_) => Self::BalanceOverflow,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

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
        assert_eq!(chain().height(), 0);
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
            chain.produce_block("v1", 1_000, vec![tx]),
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
        // invalid: same nonce as valid (would be a replay)
        let invalid = Transaction::new(
            treasury.clone(),
            bob,
            Coin::from_ac(1).unwrap(),
            Coin::ZERO,
            0,
        );
        assert!(chain
            .produce_block("v1", 1_000, vec![valid, invalid])
            .is_err());
        assert_eq!(chain.height(), 0);
        assert_eq!(chain.next_nonce(&treasury), 0);
        assert_eq!(chain.balance_of(&alice), Coin::ZERO);
    }

    #[test]
    fn apply_block_validates_height_and_previous_hash() {
        let mut proposer_chain = chain();
        let treasury = Address::new("treasury").unwrap();
        let alice = Address::new("alice").unwrap();
        let tx = Transaction::new(treasury, alice, Coin::from_ac(5).unwrap(), Coin::ZERO, 0);
        // Proposer produces block
        proposer_chain
            .produce_block("proposer", 1_000, vec![tx.clone()])
            .unwrap();
        let block = proposer_chain.blocks().last().unwrap().clone();

        // Validator applies the same block
        let mut validator_chain = chain();
        validator_chain.apply_block(block).unwrap();

        assert_eq!(validator_chain.height(), 1);
        assert_eq!(
            validator_chain.balance_of(&Address::new("alice").unwrap()),
            Coin::from_ac(5).unwrap()
        );
        // Tips must match after same sequence
        assert_eq!(proposer_chain.tip_hash(), validator_chain.tip_hash());
    }

    #[test]
    fn apply_block_rejects_wrong_height() {
        let mut chain = chain();
        let treasury = Address::new("treasury").unwrap();
        let alice = Address::new("alice").unwrap();
        let tx = Transaction::new(treasury, alice, Coin::from_ac(1).unwrap(), Coin::ZERO, 0);
        let mut other = chain.clone();
        other.produce_block("proposer", 1_000, vec![tx]).unwrap();
        let block_h1 = other.blocks().last().unwrap().clone();

        // Try to apply block at height 1 to a chain already at height 1
        chain.apply_block(block_h1.clone()).unwrap();
        let result = chain.apply_block(block_h1);
        assert!(matches!(
            result,
            Err(ApplyError::UnexpectedBlockHeight { .. })
        ));
    }

    #[test]
    fn state_root_is_consistent_between_proposer_and_validator() {
        let mut proposer = chain();
        let mut validator = chain();
        let treasury = Address::new("treasury").unwrap();
        let alice = Address::new("alice").unwrap();
        let tx = Transaction::new(treasury, alice, Coin::from_ac(10).unwrap(), Coin::ZERO, 0);

        proposer
            .produce_block("proposer", 1_000, vec![tx.clone()])
            .unwrap();
        let block = proposer.blocks().last().unwrap().clone();
        validator.apply_block(block).unwrap();

        assert_eq!(proposer.state_root(), validator.state_root());
    }
}
