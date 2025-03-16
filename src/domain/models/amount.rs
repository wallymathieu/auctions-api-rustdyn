use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};
use std::str::FromStr;

use super::currency::CurrencyCode;
use super::errors::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amount {
    value: i64,
    currency: CurrencyCode,
}

impl FromStr for Amount {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let amount_regex = Regex::new(r"^(?<currency>[A-Z]+)(?<value>[0-9]+)$").unwrap();

        // Parse strings like "SEK100"
        let captures = amount_regex
            .captures(s)
            .ok_or(Error::InvalidAmount(format!("Invalid amount value: {}", s)))?;
        let value = captures["value"]
            .parse()
            .map_err(|_| Error::InvalidAmount(format!("Invalid amount value: {}", s)))?;
        let currency = captures["currency"]
            .parse()
            .map_err(|_| Error::InvalidAmount(format!("Invalid currency code: {}", s)))?;
        Ok(Amount::new(value, currency))
    }
}

impl Amount {
    pub fn new(value: i64, currency: CurrencyCode) -> Self {
        Self { value, currency }
    }

    pub fn zero(currency: CurrencyCode) -> Self {
        Self::new(0, currency)
    }

    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn currency(&self) -> CurrencyCode {
        self.currency
    }

    fn assert_same_currency(&self, other: &Self) -> Result<(), Error> {
        if self.currency != other.currency {
            Err(Error::CurrencyMismatch(
                format!("{}", self.currency),
                format!("{}", other.currency),
            ))
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.currency, self.value)
    }
}

impl Add for Amount {
    type Output = Result<Amount, Error>;

    fn add(self, other: Self) -> Self::Output {
        self.assert_same_currency(&other)?;
        Ok(Amount::new(self.value + other.value, self.currency))
    }
}

impl Sub for Amount {
    type Output = Result<Amount, Error>;

    fn sub(self, other: Self) -> Self::Output {
        self.assert_same_currency(&other)?;
        Ok(Amount::new(self.value - other.value, self.currency))
    }
}

impl PartialOrd for Amount {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.currency == other.currency {
            self.value.partial_cmp(&other.value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod amount_tests {
    use super::*;

    #[test]
    fn test_create_amount() {
        let amount = Amount::new(100, CurrencyCode::SEK);
        assert_eq!(amount.value(), 100);
        assert_eq!(amount.currency(), CurrencyCode::SEK);
    }

    #[test]
    fn test_zero_amount() {
        let amount: Amount = Amount::zero(CurrencyCode::VAC);
        assert_eq!(amount.value(), 0);
        assert_eq!(amount.currency(), CurrencyCode::VAC);
    }

    #[test]
    fn test_amount_from_string_valid() {
        let amount: Result<Amount, _> = "SEK100".parse();
        assert!(amount.is_ok());
        let amount = amount.unwrap();
        assert_eq!(amount.value(), 100);
        assert_eq!(amount.currency(), CurrencyCode::SEK);
    }

    #[test]
    fn test_amount_from_string_invalid_currency() {
        let result: Result<Amount, _> = "XYZ100".parse();
        assert!(result.is_err());
        match result {
            Err(Error::InvalidAmount(msg)) => {
                assert!(msg.contains("Invalid currency code"), "{}", msg);
            }
            _ => panic!("Expected InvalidAmount error"),
        }
    }

    #[test]
    fn test_amount_from_string_invalid_value() {
        let result: Result<Amount, _> = "SEKabc".parse();
        assert!(result.is_err());
        match result {
            Err(Error::InvalidAmount(msg)) => {
                assert!(msg.contains("Invalid amount value"), "{}", msg);
            }
            _ => panic!("Expected InvalidAmount error"),
        }
    }

    #[test]
    fn test_amount_display() {
        let amount = Amount::new(100, CurrencyCode::SEK);
        assert_eq!(amount.to_string(), "SEK100");
    }

    #[test]
    fn test_amount_add_same_currency() {
        let a1 = Amount::new(100, CurrencyCode::SEK);
        let a2 = Amount::new(200, CurrencyCode::SEK);
        let sum = (a1 + a2).unwrap();
        assert_eq!(sum.value(), 300);
        assert_eq!(sum.currency(), CurrencyCode::SEK);
    }

    #[test]
    fn test_amount_add_different_currency() {
        let a1 = Amount::new(100, CurrencyCode::SEK);
        let a2 = Amount::new(200, CurrencyCode::VAC);
        let result = a1 + a2;
        assert!(result.is_err());
        match result {
            Err(Error::CurrencyMismatch(c1, c2)) => {
                assert_eq!(c1, "SEK");
                assert_eq!(c2, "VAC");
            }
            _ => panic!("Expected CurrencyMismatch error"),
        }
    }

    #[test]
    fn test_amount_subtract_same_currency() {
        let a1 = Amount::new(300, CurrencyCode::SEK);
        let a2 = Amount::new(100, CurrencyCode::SEK);
        let diff = (a1 - a2).unwrap();
        assert_eq!(diff.value(), 200);
        assert_eq!(diff.currency(), CurrencyCode::SEK);
    }

    #[test]
    fn test_amount_compare() {
        let a1 = Amount::new(100, CurrencyCode::SEK);
        let a2 = Amount::new(200, CurrencyCode::SEK);
        let a3 = Amount::new(100, CurrencyCode::VAC);

        assert!(a1 < a2);
        assert!(a2 > a1);
        assert!(a1.partial_cmp(&a3).is_none()); // Different currencies can't be compared
    }
}
