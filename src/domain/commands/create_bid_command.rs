use serde::{Deserialize, Serialize};

use crate::domain::models::{Amount, AuctionId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBidCommand {
    pub amount: Amount,
    pub auction_id: AuctionId,
}
