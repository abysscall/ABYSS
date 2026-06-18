use std::collections::BTreeMap;

use crate::transaction::{Transaction, TransactionId};

#[derive(Clone, Debug, Default)]
pub struct Mempool {
    transactions: BTreeMap<TransactionId, Transaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    pub fn insert(&mut self, tx: Transaction) -> Result<TransactionId, MempoolError> {
        let id = tx.id();
        if self.transactions.contains_key(&id) {
            return Err(MempoolError::Duplicate(id));
        }

        self.transactions.insert(id, tx);
        Ok(id)
    }

    pub fn contains(&self, id: &TransactionId) -> bool {
        self.transactions.contains_key(id)
    }

    pub fn remove(&mut self, id: &TransactionId) -> Option<Transaction> {
        self.transactions.remove(id)
    }

    pub fn drain_for_block(&mut self, limit: usize) -> Vec<Transaction> {
        let ids = self
            .transactions
            .keys()
            .copied()
            .take(limit)
            .collect::<Vec<_>>();

        ids.into_iter()
            .filter_map(|id| self.transactions.remove(&id))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MempoolError {
    Duplicate(TransactionId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Address, Coin};

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
    fn rejects_duplicate_transaction() {
        let mut mempool = Mempool::new();
        let tx = tx(0);
        let id = mempool.insert(tx.clone()).unwrap();

        assert_eq!(mempool.insert(tx), Err(MempoolError::Duplicate(id)));
    }

    #[test]
    fn drains_transactions_for_block() {
        let mut mempool = Mempool::new();
        mempool.insert(tx(0)).unwrap();
        mempool.insert(tx(1)).unwrap();

        let drained = mempool.drain_for_block(1);

        assert_eq!(drained.len(), 1);
        assert_eq!(mempool.len(), 1);
    }
}

