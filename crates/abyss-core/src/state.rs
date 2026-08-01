//! Ledger state — separated from Chain per ADR-0005.
//!
//! `Chain` owns consensus and block sequencing.
//! `State` owns all persistent account data.
//!
//! Current scope: Accounts (balances + nonces).
//! Future scope: Validators, Treasury, Contracts, Governance, Storage anchors.

use std::collections::BTreeMap;

use crate::address::{Address, AddressError};
use crate::coin::Coin;
use crate::hashing::{dev_hash, Hash256};

/// All persistent ledger state for the ABYSS chain.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct State {
    pub accounts: AccountState,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    /// Deterministic hash of the entire state — used as block state_root.
    pub fn root(&self) -> Hash256 {
        dev_hash(&(&self.accounts.balances, &self.accounts.nonces))
    }

    pub fn balance_of(&self, address: &Address) -> Coin {
        self.accounts.balances.get(address).copied().unwrap_or(Coin::ZERO)
    }

    pub fn nonce_of(&self, address: &Address) -> u64 {
        self.accounts.nonces.get(address).copied().unwrap_or(0)
    }

    /// Credit amount to address. Creates account if it does not exist.
    pub fn credit(&mut self, address: &Address, amount: Coin) -> Result<(), StateError> {
        let next = self.balance_of(address)
            .checked_add(amount)
            .ok_or(StateError::BalanceOverflow)?;
        self.accounts.balances.insert(address.clone(), next);
        Ok(())
    }

    /// Debit amount from address. Fails if balance is insufficient.
    pub fn debit(&mut self, address: &Address, amount: Coin) -> Result<(), StateError> {
        let current = self.balance_of(address);
        let next = current.checked_sub(amount)
            .ok_or(StateError::InsufficientFunds { available: current, required: amount })?;
        self.accounts.balances.insert(address.clone(), next);
        Ok(())
    }

    /// Advance nonce for address by one.
    pub fn increment_nonce(&mut self, address: &Address) {
        let next = self.nonce_of(address) + 1;
        self.accounts.nonces.insert(address.clone(), next);
    }

    /// Export to string-keyed map for JSON serialisation (compatible with
    /// existing storage.rs snapshot format).
    pub fn to_raw(&self) -> (BTreeMap<String, u64>, BTreeMap<String, u64>) {
        let balances = self.accounts.balances.iter()
            .map(|(a, c)| (a.as_str().to_string(), c.micro_ac()))
            .collect();
        let nonces = self.accounts.nonces.iter()
            .map(|(a, n)| (a.as_str().to_string(), *n))
            .collect();
        (balances, nonces)
    }

    /// Restore from string-keyed maps (compatible with existing storage.rs).
    pub fn from_raw(
        balances: BTreeMap<String, u64>,
        nonces: BTreeMap<String, u64>,
    ) -> Result<Self, StateError> {
        let mut state = State::new();
        for (addr_str, micro_ac) in balances {
            let addr = Address::new(addr_str)?;
            let coin = Coin::from_micro_ac(micro_ac).ok_or(StateError::BalanceOverflow)?;
            state.accounts.balances.insert(addr, coin);
        }
        for (addr_str, nonce) in nonces {
            let addr = Address::new(addr_str)?;
            state.accounts.nonces.insert(addr, nonce);
        }
        Ok(state)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AccountState {
    pub balances: BTreeMap<Address, Coin>,
    pub nonces: BTreeMap<Address, u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StateError {
    Address(AddressError),
    BalanceOverflow,
    InsufficientFunds { available: Coin, required: Coin },
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Address(e) => write!(f, "invalid address: {e}"),
            Self::BalanceOverflow => write!(f, "balance overflow"),
            Self::InsufficientFunds { available, required } =>
                write!(f, "insufficient funds: have {available}, need {required}"),
        }
    }
}

impl From<AddressError> for StateError {
    fn from(e: AddressError) -> Self { Self::Address(e) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(s: &str) -> Address { Address::new(s).unwrap() }

    #[test]
    fn credit_and_debit_update_balance() {
        let mut s = State::new();
        let alice = addr("alice");
        s.credit(&alice, Coin::from_ac(100).unwrap()).unwrap();
        assert_eq!(s.balance_of(&alice), Coin::from_ac(100).unwrap());
        s.debit(&alice, Coin::from_ac(30).unwrap()).unwrap();
        assert_eq!(s.balance_of(&alice), Coin::from_ac(70).unwrap());
    }

    #[test]
    fn debit_fails_on_insufficient_funds_and_leaves_balance_unchanged() {
        let mut s = State::new();
        let alice = addr("alice");
        s.credit(&alice, Coin::from_ac(10).unwrap()).unwrap();
        assert!(matches!(
            s.debit(&alice, Coin::from_ac(20).unwrap()),
            Err(StateError::InsufficientFunds { .. })
        ));
        assert_eq!(s.balance_of(&alice), Coin::from_ac(10).unwrap());
    }

    #[test]
    fn nonce_starts_at_zero_and_increments() {
        let mut s = State::new();
        let alice = addr("alice");
        assert_eq!(s.nonce_of(&alice), 0);
        s.increment_nonce(&alice);
        assert_eq!(s.nonce_of(&alice), 1);
    }

    #[test]
    fn root_changes_after_mutation() {
        let mut s = State::new();
        let root_before = s.root();
        s.credit(&addr("alice"), Coin::from_ac(1).unwrap()).unwrap();
        assert_ne!(s.root(), root_before);
    }

    #[test]
    fn round_trips_through_raw() {
        let mut s = State::new();
        s.credit(&addr("alice"), Coin::from_ac(500).unwrap()).unwrap();
        s.credit(&addr("bob"), Coin::from_ac(250).unwrap()).unwrap();
        s.increment_nonce(&addr("alice"));
        s.increment_nonce(&addr("alice"));

        let (balances, nonces) = s.to_raw();
        let restored = State::from_raw(balances, nonces).unwrap();

        assert_eq!(restored.balance_of(&addr("alice")), Coin::from_ac(500).unwrap());
        assert_eq!(restored.nonce_of(&addr("alice")), 2);
        assert_eq!(restored.root(), s.root());
    }

    #[test]
    fn unknown_address_returns_zero() {
        let s = State::new();
        assert_eq!(s.balance_of(&addr("nobody")), Coin::ZERO);
        assert_eq!(s.nonce_of(&addr("nobody")), 0);
    }
}
