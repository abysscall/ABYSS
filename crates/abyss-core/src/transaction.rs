use crate::address::Address;
use crate::coin::Coin;
use crate::hashing::{dev_hash, Hash256};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TransactionId(pub Hash256);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub amount: Coin,
    pub fee: Coin,
    pub nonce: u64,
    pub memo_commitment: Hash256,
}

impl Transaction {
    pub fn new(from: Address, to: Address, amount: Coin, fee: Coin, nonce: u64) -> Self {
        Self {
            from,
            to,
            amount,
            fee,
            nonce,
            memo_commitment: dev_hash(&("abyss:memo:v0", nonce)),
        }
    }

    pub fn id(&self) -> TransactionId {
        TransactionId(dev_hash(self))
    }

    pub fn total_debit(&self) -> Option<Coin> {
        self.amount.checked_add(self.fee)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_id_is_stable_for_same_contents() {
        let tx = Transaction::new(
            Address::new("alice").unwrap(),
            Address::new("bob").unwrap(),
            Coin::from_ac(1).unwrap(),
            Coin::ZERO,
            7,
        );

        assert_eq!(tx.id(), tx.clone().id());
    }
}
