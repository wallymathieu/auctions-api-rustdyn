use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Errors {
    None = 0,
    UnknownAuction = 1 << 0,
    AuctionAlreadyExists = 1 << 1,
    AuctionHasEnded = 1 << 2,
    AuctionHasNotStarted = 1 << 3,
    AuctionNotFound = 1 << 4,
    SellerCannotPlaceBids = 1 << 5,
    BidCurrencyConversion = 1 << 6,
    InvalidUserData = 1 << 7,
    MustPlaceBidOverHighestBid = 1 << 8,
    AlreadyPlacedBid = 1 << 9,
    MustRaiseWithAtLeast = 1 << 10,
    MustSpecifyAmount = 1 << 11,
}

impl Errors {
    pub fn is_none(&self) -> bool {
        *self == Errors::None
    }
}

impl std::ops::BitOr for Errors {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let lhs_val = self as u16;
        let rhs_val = rhs as u16;
        unsafe { std::mem::transmute(lhs_val | rhs_val) }
    }
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Errors::None => write!(f, "No error"),
            Errors::UnknownAuction => write!(f, "Unknown auction"),
            Errors::AuctionAlreadyExists => write!(f, "Auction already exists"),
            Errors::AuctionHasEnded => write!(f, "Auction has ended"),
            Errors::AuctionHasNotStarted => write!(f, "Auction has not started"),
            Errors::AuctionNotFound => write!(f, "Auction not found"),
            Errors::SellerCannotPlaceBids => write!(f, "Seller cannot place bids"),
            Errors::BidCurrencyConversion => write!(f, "Bid currency conversion error"),
            Errors::InvalidUserData => write!(f, "Invalid user data"),
            Errors::MustPlaceBidOverHighestBid => write!(f, "Must place bid over highest bid"),
            Errors::AlreadyPlacedBid => write!(f, "Already placed bid"),
            Errors::MustRaiseWithAtLeast => write!(f, "Must raise with at least minimum raise amount"),
            Errors::MustSpecifyAmount => write!(f, "Must specify amount"),
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Validation error: {0}")]
    Validation(Errors),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Currency mismatch: {0} vs {1}")]
    CurrencyMismatch(String, String),

    #[error("Invalid user: {0}")]
    InvalidUser(String),

    #[error("Domain error: {0}")]
    Domain(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal error: {0}")]
    Internal(String),
}