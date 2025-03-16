use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::models::{Amount, CurrencyCode};

use crate::api::models::BidModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionModel {
    pub id: i64,
    #[serde(rename = "startsAt")]
    pub starts_at: DateTime<Utc>,
    pub title: String,
    #[serde(rename = "expiry")]
    pub expiry: DateTime<Utc>,
    pub seller: Option<String>,
    pub currency: CurrencyCode,
    pub bids: Vec<BidModel>,
    pub price: Option<Amount>,
    pub winner: Option<String>,
    #[serde(rename = "hasEnded")]
    pub has_ended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuctionModel {
    pub title: String,
    pub currency: CurrencyCode,
    #[serde(rename = "startsAt")]
    pub starts_at: DateTime<Utc>,
    #[serde(rename = "endsAt")]
    pub ends_at: DateTime<Utc>,
    #[serde(rename = "minRaise")]
    pub min_raise: Option<i64>,
    #[serde(rename = "reservePrice")]
    pub reserve_price: Option<i64>,
    #[serde(rename = "timeFrame")]
    pub time_frame: Option<i64>, // in seconds
    #[serde(rename = "singleSealedBidOptions")]
    pub single_sealed_bid_options: Option<String>,
    #[serde(default,rename = "openBidders")]
    pub open_bidders: bool,
}
