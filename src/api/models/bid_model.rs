use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::domain::models::Amount;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidModel {
    pub amount: Amount,
    pub bidder: Option<String>,
    pub at: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBidModel {
    pub amount: Amount,
}
