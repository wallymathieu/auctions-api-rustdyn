use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::amount::Amount;
use super::user::UserId;
use super::{Auction, Errors};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BidData {
    pub user: UserId,
    pub amount: Amount,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bid {
    pub id: i64,
    #[serde(flatten)]
    pub data: BidData
}

impl Bid {
    pub fn new(id: i64, user: UserId, amount: Amount, at: DateTime<Utc>) -> Self {
        Self {
            id,
            data: BidData {user,amount,at}
        }
    }

    pub fn at(&self) -> DateTime<Utc> { self.data.at }
    pub fn user(&self) -> UserId { self.data.user.clone() }
    pub fn amount(&self) -> Amount { self.data.amount.clone() }

    pub fn validate(&self, auction: &Auction) -> Errors {
        let mut errors = Errors::None;
        if self.user() == *auction.user() {
            errors = errors | Errors::SellerCannotPlaceBids;
        }
        if self.amount().currency() != auction.currency() {
            errors = errors | Errors::BidCurrencyConversion;
        }
        if self.at() < auction.starts_at() {
            errors = errors | Errors::AuctionHasNotStarted;
        }
        if self.at() > auction.expiry() {
            errors = errors | Errors::AuctionHasEnded;
        }

        errors
    }
}
