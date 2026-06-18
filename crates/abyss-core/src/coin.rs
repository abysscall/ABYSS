use core::fmt;
use core::ops::{Add, Sub};

pub const COIN: u64 = 100_000_000;
pub const MAX_SUPPLY: u64 = 55_000_000 * COIN;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Coin(u64);

impl Coin {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(MAX_SUPPLY);

    pub const fn from_micro_ac(value: u64) -> Option<Self> {
        if value <= MAX_SUPPLY {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn from_ac(value: u64) -> Option<Self> {
        match value.checked_mul(COIN) {
            Some(v) if v <= MAX_SUPPLY => Some(Self(v)),
            _ => None,
        }
    }

    pub const fn micro_ac(self) -> u64 {
        self.0
    }

    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0
            .checked_add(rhs.0)
            .filter(|value| *value <= MAX_SUPPLY)
            .map(Self)
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl Add for Coin {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Sub for Coin {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl fmt::Display for Coin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.0 / COIN;
        let fraction = self.0 % COIN;
        write!(f, "{whole}.{fraction:08} AC")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supply_is_capped_at_55m_ac() {
        assert_eq!(Coin::MAX.micro_ac(), 5_500_000_000_000_000);
        assert!(Coin::from_ac(55_000_000).is_some());
        assert!(Coin::from_ac(55_000_001).is_none());
    }

    #[test]
    fn display_formats_micro_units() {
        assert_eq!(
            Coin::from_micro_ac(123_456_789).unwrap().to_string(),
            "1.23456789 AC"
        );
    }
}
