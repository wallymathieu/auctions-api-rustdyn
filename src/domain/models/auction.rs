use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::amount::Amount;
use super::bid::Bid;
use super::currency::CurrencyCode;
use super::errors::Errors;
use super::user::UserId;
use std::fmt;
use crate::domain::commands::CreateAuctionCommand;
use crate::domain::models::BidData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuctionId(i64);

impl AuctionId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn value(&self) -> i64 {
        self.0
    }
}

impl fmt::Display for AuctionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuctionType {
    SingleSealedBid,
    TimedAscending,
}
impl fmt::Display for AuctionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

// Common traits that both auction types will implement
pub trait AuctionState {
    fn try_add_bid(&mut self, time: DateTime<Utc>, bid: Bid) -> Result<bool, Errors>;
    fn get_bids(&self, time: DateTime<Utc>) -> Vec<Bid>;
    fn try_get_amount_and_winner(&self, time: DateTime<Utc>) -> Option<(Amount, UserId)>;
    fn has_ended(&self, time: DateTime<Utc>) -> bool;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SingleSealedBidOptions {
    Blind,
    Vickrey,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimedAscendingOptions {
    pub reserve_price: i64,
    pub min_raise: i64,
    pub time_frame: chrono::Duration,
}

impl Default for TimedAscendingOptions {
    fn default() -> Self {
        Self {
            reserve_price: 0,
            min_raise: 0,
            time_frame: chrono::Duration::seconds(0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "auction_type")]
pub enum Auction {
    //
    SingleSealedBid {
        #[serde(flatten)]
        base: AuctionBase,
        options: SingleSealedBidOptions,
    },
    TimedAscending {
        #[serde(flatten)]
        base: AuctionBase,
        options: TimedAscendingOptions,
        ends_at: Option<DateTime<Utc>>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuctionBase {
    pub auction_id: AuctionId,
    pub title: String,
    pub starts_at: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub user: UserId,
    pub currency: CurrencyCode,
    pub bids: Vec<Bid>,
    pub open_bidders: bool,
}

impl Auction {
    pub fn auction_id(&self) -> AuctionId {
        match self {
            Auction::SingleSealedBid { base, .. } => base.auction_id,
            Auction::TimedAscending { base, .. } => base.auction_id,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Auction::SingleSealedBid { base, .. } => &base.title,
            Auction::TimedAscending { base, .. } => &base.title,
        }
    }

    pub fn starts_at(&self) -> DateTime<Utc> {
        match self {
            Auction::SingleSealedBid { base, .. } => base.starts_at,
            Auction::TimedAscending { base, .. } => base.starts_at,
        }
    }

    pub fn expiry(&self) -> DateTime<Utc> {
        match self {
            Auction::SingleSealedBid { base, .. } => base.expiry,
            Auction::TimedAscending { base, .. } => base.expiry,
        }
    }

    pub fn user(&self) -> &UserId {
        match self {
            Auction::SingleSealedBid { base, .. } => &base.user,
            Auction::TimedAscending { base, .. } => &base.user,
        }
    }

    pub fn currency(&self) -> CurrencyCode {
        match self {
            Auction::SingleSealedBid { base, .. } => base.currency,
            Auction::TimedAscending { base, .. } => base.currency,
        }
    }

    pub fn bids(&self) -> &[Bid] {
        match self {
            Auction::SingleSealedBid { base, .. } => &base.bids,
            Auction::TimedAscending { base, .. } => &base.bids,
        }
    }

    pub fn bids_mut(&mut self) -> &mut Vec<Bid> {
        match self {
            Auction::SingleSealedBid { base, .. } => &mut base.bids,
            Auction::TimedAscending { base, .. } => &mut base.bids,
        }
    }

    pub fn open_bidders(&self) -> bool {
        match self {
            Auction::SingleSealedBid { base, .. } => base.open_bidders,
            Auction::TimedAscending { base, .. } => base.open_bidders,
        }
    }

    pub fn set_open_bidders(&mut self, open: bool) {
        match self {
            Auction::SingleSealedBid { base, .. } => base.open_bidders = open,
            Auction::TimedAscending { base, .. } => base.open_bidders = open,
        }
    }

    pub fn set_auction_id(&mut self, id: AuctionId) {
        match self {
            Auction::SingleSealedBid { base, .. } => base.auction_id = id,
            Auction::TimedAscending { base, .. } => base.auction_id = id,
        }
    }

    pub fn auction_type(&self) -> AuctionType {
        match self {
            Auction::SingleSealedBid { .. } => AuctionType::SingleSealedBid,
            Auction::TimedAscending { .. } => AuctionType::TimedAscending,
        }
    }

    // Implementation of validation for bid
    fn validate_bid(&self, bid: &BidData) -> Errors {
        let mut errors = Errors::None;

        // Check if seller is bidding on their own auction
        if bid.user == *self.user() {
            errors = errors | Errors::SellerCannotPlaceBids;
        }

        // Check currency match
        if bid.amount.currency() != self.currency() {
            errors = errors | Errors::BidCurrencyConversion;
        }

        // Check auction timing
        if bid.at < self.starts_at() {
            errors = errors | Errors::AuctionHasNotStarted;
        }
        if bid.at > self.expiry() {
            errors = errors | Errors::AuctionHasEnded;
        }

        errors
    }

    // Implement the state pattern for auction states
    pub fn try_add_bid(&mut self, time: DateTime<Utc>, bid: BidData) -> Result<bool, Errors> {
        let errors = self.validate_bid(&bid);
        if errors != Errors::None {
            return Err(errors);
        }

        match self {
            Auction::SingleSealedBid { base, options: _ } => {
                // Single sealed bid auction logic
                if time > base.expiry {
                    return Err(Errors::AuctionHasEnded);
                }
                
                if time < base.starts_at {
                    return Err(Errors::AuctionHasNotStarted);
                }

                // Check if bidder already placed a bid
                let user_already_bid = base.bids.iter().any(|b| b.user() == bid.user);
                if user_already_bid {
                    return Err(Errors::AlreadyPlacedBid);
                }

                // Add bid
                let next_id = base.bids.len() as i64 + 1;
                let bid_entity = Bid::new(
                    next_id,
                    bid.user.clone(),
                    bid.amount.clone(),
                    bid.at,
                );
                base.bids.push(bid_entity);
                
                Ok(true)
            },
            Auction::TimedAscending { base, options, ends_at } => {
                // Timed ascending auction logic
                if time > base.expiry {
                    return Err(Errors::AuctionHasEnded);
                }
                
                if time < base.starts_at {
                    return Err(Errors::AuctionHasNotStarted);
                }

                // Check if bid is higher than current highest bid
                if !base.bids.is_empty() {
                    let highest_bid = base.bids
                        .iter()
                        .max_by_key(|b| b.amount().value())
                        .unwrap();
                    
                    if bid.amount.value() <= highest_bid.amount().value() {
                        return Err(Errors::MustPlaceBidOverHighestBid);
                    }
                    
                    if bid.amount.value() < highest_bid.amount().value() + options.min_raise {
                        return Err(Errors::MustRaiseWithAtLeast);
                    }
                }

                // Update the auction end time
                let time_extended = time + options.time_frame;
                let current_end = *ends_at.as_ref().unwrap_or(&base.expiry);
                let new_end = if time_extended > current_end {
                    time_extended
                } else {
                    current_end
                };
                *ends_at = Some(new_end);

                // Add bid
                let next_id = base.bids.len() as i64 + 1;
                let bid_entity = Bid::new(
                    next_id,
                    bid.user.clone(),
                    bid.amount.clone(),
                    bid.at,
                );
                base.bids.push(bid_entity);
                
                Ok(true)
            },
        }
    }

    pub fn get_bids(&self, time: DateTime<Utc>) -> Option<&Vec<Bid>> {
        match self {
            Auction::SingleSealedBid { base, .. } => {
                if time < base.starts_at || time > base.expiry {
                    return None;
                }
                
                Some(&base.bids)
            },
            Auction::TimedAscending { base, .. } => {
                if time < base.starts_at {
                    return None;
                }
                
                Some(&base.bids)
            },
        }
    }

    pub fn try_get_amount_and_winner(&self, time: DateTime<Utc>) -> Option<(Amount, UserId)> {
        match self {
            Auction::SingleSealedBid { base, options } => {
                // Only return winner after auction has ended
                if time <= base.expiry || base.bids.is_empty() {
                    return None;
                }
                
                match options {
                    SingleSealedBidOptions::Blind => {
                        // First price sealed bid - highest bidder wins and pays their bid
                        base.bids
                            .iter()
                            .max_by_key(|b| b.amount().value())
                            .map(|b| (b.amount(), b.user()))
                    },
                    SingleSealedBidOptions::Vickrey => {
                        // Second price sealed bid - highest bidder wins but pays second highest bid
                        if base.bids.len() == 1 {
                            let bid = &base.bids[0];
                            return Some((bid.amount(), bid.user()));
                        }
                        
                        let mut bids: Vec<_> = base.bids.iter().collect();
                        bids.sort_by(|a, b| b.amount().value().cmp(&a.amount().value()));
                        
                        // Highest bidder wins but pays second-highest price
                        Some((bids[1].amount(), bids[0].user()))
                    },
                }
            },
            Auction::TimedAscending { base, options, .. } => {
                // Only return winner after auction has ended
                if time <= base.expiry || base.bids.is_empty() {
                    return None;
                }
                
                // Find highest bid
                let highest_bid = base.bids
                    .iter()
                    .max_by_key(|b| b.amount().value())
                    .unwrap();
                
                // Check reserve price
                if highest_bid.amount().value() >= options.reserve_price {
                    Some((highest_bid.amount(), highest_bid.user()))
                } else {
                    None
                }
            },
        }
    }

    pub fn has_ended(&self, time: DateTime<Utc>) -> bool {
        match self {
            Auction::SingleSealedBid { base, .. } => time > base.expiry,
            Auction::TimedAscending { base, ends_at, .. } => {
                time > ends_at.unwrap_or(base.expiry)
            },
        }
    }
}

pub struct AuctionFactory;

impl AuctionFactory {
    pub fn create_auction(
        cmd: CreateAuctionCommand,
        user_id: UserId,
    ) -> Result<Auction, &'static str> {
        let base = AuctionBase {
            auction_id: AuctionId::new(0),
            title: cmd.title,
            starts_at: cmd.starts_at,
            expiry: cmd.ends_at,
            user: user_id.clone(),
            currency: cmd.currency,
            bids: Vec::new(),
            open_bidders: cmd.open_bidders,
        };

        if let Some(options) = cmd.single_sealed_bid_options {
            // Create a single sealed bid auction
            Ok(Auction::SingleSealedBid {
                base,
                options,
            })
        } else {
            // Create a timed ascending auction
            let options = TimedAscendingOptions {
                min_raise: cmd.min_raise.unwrap_or(0),
                reserve_price: cmd.reserve_price.unwrap_or(0),
                time_frame: cmd.time_frame.unwrap_or_else(|| chrono::Duration::seconds(0)),
            };
            
            Ok(Auction::TimedAscending {
                base,
                options,
                ends_at: None,
            })
        }
    }
}