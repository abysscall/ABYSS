use crate::address::Address;
use crate::coin::{Coin, MAX_SUPPLY};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenesisConfig {
    pub allocations: Vec<(Address, Coin)>,
}

impl GenesisConfig {
    pub fn single_treasury(address: Address) -> Self {
        Self {
            allocations: vec![(address, Coin::MAX)],
        }
    }

    pub fn total_allocated(&self) -> Option<Coin> {
        self.allocations
            .iter()
            .try_fold(Coin::ZERO, |acc, (_, amount)| acc.checked_add(*amount))
    }

    pub fn validate(&self) -> Result<(), GenesisError> {
        let total = self.total_allocated().ok_or(GenesisError::SupplyOverflow)?;
        if total.micro_ac() > MAX_SUPPLY {
            return Err(GenesisError::SupplyOverflow);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenesisError {
    SupplyOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_treasury_allocates_full_supply() {
        let config = GenesisConfig::single_treasury(Address::new("treasury").unwrap());
        assert_eq!(config.total_allocated(), Some(Coin::MAX));
        assert_eq!(config.validate(), Ok(()));
    }
}
