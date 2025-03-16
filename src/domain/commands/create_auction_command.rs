use chrono::{DateTime, Utc};
use crate::domain::models::{CurrencyCode, SingleSealedBidOptions};

#[derive(Debug, Clone)]
pub struct CreateAuctionCommand {
    pub title: String,
    pub currency: CurrencyCode,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub min_raise: Option<i64>,
    pub reserve_price: Option<i64>,
    pub time_frame: Option<chrono::Duration>,
    pub single_sealed_bid_options: Option<SingleSealedBidOptions>,
    pub open_bidders: bool,
}
