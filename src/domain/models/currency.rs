use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurrencyCode {
    None,
    VAC = 1001,
    SEK = 752,
    DKK = 208,
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CurrencyCode::None => write!(f, "NONE"),
            CurrencyCode::VAC => write!(f, "VAC"),
            CurrencyCode::SEK => write!(f, "SEK"),
            CurrencyCode::DKK => write!(f, "DKK"),
        }
    }
}

impl FromStr for CurrencyCode {
    type Err=();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "VAC" => Ok(CurrencyCode::VAC),
            "SEK" => Ok(CurrencyCode::SEK),
            "DKK" => Ok(CurrencyCode::DKK),
            _ => Err(()),
        }
    }
}

impl Default for CurrencyCode {
    fn default() -> Self {
        CurrencyCode::None
    }
}
