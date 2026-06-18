use crate::hashing::{dev_hash, Hash256, ZERO_HASH};
use crate::transaction::Transaction;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BlockHeader {
    pub height: u64,
    pub previous_hash: Hash256,
    pub state_root: Hash256,
    pub transactions_root: Hash256,
    pub timestamp_ms: u64,
    pub proposer: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn genesis(timestamp_ms: u64, proposer: impl Into<String>) -> Self {
        Self {
            header: BlockHeader {
                height: 0,
                previous_hash: ZERO_HASH,
                state_root: ZERO_HASH,
                transactions_root: ZERO_HASH,
                timestamp_ms,
                proposer: proposer.into(),
            },
            transactions: Vec::new(),
        }
    }

    pub fn new(
        height: u64,
        previous_hash: Hash256,
        state_root: Hash256,
        timestamp_ms: u64,
        proposer: impl Into<String>,
        transactions: Vec<Transaction>,
    ) -> Self {
        let transactions_root = dev_hash(&transactions);
        Self {
            header: BlockHeader {
                height,
                previous_hash,
                state_root,
                transactions_root,
                timestamp_ms,
                proposer: proposer.into(),
            },
            transactions,
        }
    }

    pub fn hash(&self) -> Hash256 {
        dev_hash(&self.header)
    }
}

