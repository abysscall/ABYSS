use core::fmt;
use core::str::FromStr;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Address(String);

impl Address {
    pub fn new(value: impl Into<String>) -> Result<Self, AddressError> {
        let value = value.into();
        validate_address(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AddressError {
    Empty,
    TooLong,
    InvalidCharacter(char),
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "address is empty"),
            Self::TooLong => write!(f, "address is too long"),
            Self::InvalidCharacter(ch) => write!(f, "address contains invalid character '{ch}'"),
        }
    }
}

impl FromStr for Address {
    type Err = AddressError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_address(value: &str) -> Result<(), AddressError> {
    if value.is_empty() {
        return Err(AddressError::Empty);
    }

    if value.len() > 96 {
        return Err(AddressError::TooLong);
    }

    for ch in value.chars() {
        if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == ':') {
            return Err(AddressError::InvalidCharacter(ch));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_accepts_devnet_names() {
        let address = Address::new("abyss:dev:treasury").unwrap();
        assert_eq!(address.as_str(), "abyss:dev:treasury");
    }

    #[test]
    fn address_rejects_spaces() {
        assert!(matches!(
            Address::new("bad address"),
            Err(AddressError::InvalidCharacter(' '))
        ));
    }
}

