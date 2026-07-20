use crate::hashing::{dev_hash, merkle_root, Hash256, ZERO_HASH};
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
        let transactions_root = Self::compute_transactions_root(&transactions);
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

    /// Merkle root over the transaction IDs, in block order. Empty blocks yield
    /// [`ZERO_HASH`], matching the genesis header.
    pub fn compute_transactions_root(transactions: &[Transaction]) -> Hash256 {
        let leaves: Vec<Hash256> = transactions.iter().map(|tx| tx.id().0).collect();
        merkle_root(&leaves)
    }

    /// Returns true when the header's `transactions_root` matches the Merkle
    /// root recomputed from the block body.
    pub fn transactions_root_is_valid(&self) -> bool {
        self.header.transactions_root == Self::compute_transactions_root(&self.transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::Address;
    use crate::coin::Coin;

    fn tx(nonce: u64) -> Transaction {
        Transaction::new(
            Address::new("alice").unwrap(),
            Address::new("bob").unwrap(),
            Coin::from_ac(1).unwrap(),
            Coin::ZERO,
            nonce,
        )
    }

    #[test]
    fn empty_block_root_is_zero_and_matches_genesis() {
        let block = Block::new(1, ZERO_HASH, ZERO_HASH, 0, "proposer", Vec::new());
        assert_eq!(block.header.transactions_root, ZERO_HASH);
        assert_eq!(
            block.header.transactions_root,
            Block::genesis(0, "proposer").header.transactions_root
        );
        assert!(block.transactions_root_is_valid());
    }

    #[test]
    fn root_uses_merkle_tree_not_flat_hash() {
        let txs = vec![tx(1), tx(2), tx(3)];
        let block = Block::new(1, ZERO_HASH, ZERO_HASH, 0, "proposer", txs.clone());

        let leaves: Vec<Hash256> = txs.iter().map(|t| t.id().0).collect();
        assert_eq!(block.header.transactions_root, merkle_root(&leaves));
        // Regression guard: it must not be the old flat hash of the vector.
        assert_ne!(block.header.transactions_root, dev_hash(&txs));
    }

    #[test]
    fn reordering_transactions_changes_root() {
        let a = Block::new(1, ZERO_HASH, ZERO_HASH, 0, "p", vec![tx(1), tx(2)]);
        let b = Block::new(1, ZERO_HASH, ZERO_HASH, 0, "p", vec![tx(2), tx(1)]);
        assert_ne!(a.header.transactions_root, b.header.transactions_root);
    }

    #[test]
    fn tampered_body_fails_validation() {
        let mut block = Block::new(1, ZERO_HASH, ZERO_HASH, 0, "p", vec![tx(1), tx(2)]);
        assert!(block.transactions_root_is_valid());
        block.transactions.push(tx(3));
        assert!(!block.transactions_root_is_valid());
    }
}

