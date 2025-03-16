use async_trait::async_trait;
use dyn_clone::DynClone;

use crate::domain::commands::CreateBidCommand;
use crate::domain::models::{BidData, Error, Errors, UserId};
use crate::domain::services::SystemClock;
use crate::infrastructure::data::AuctionRepository;

#[async_trait]
pub trait CreateBidCommandHandler: Send + Sync + DynClone {
    async fn handle(&self, user_id: Option<UserId>, command: CreateBidCommand) -> Result<(), Error>;
}

dyn_clone::clone_trait_object!(CreateBidCommandHandler);

#[derive(Clone)]
pub struct DefaultCreateBidCommandHandler {
    repository: Box<dyn AuctionRepository>,
    system_clock: Box<dyn SystemClock>,
}

impl DefaultCreateBidCommandHandler{
    pub fn new(
        repository: Box<dyn AuctionRepository>,
        system_clock: Box<dyn SystemClock>,
    ) -> Self {
        Self {
            repository,
            system_clock,
        }
    }
}

#[async_trait]
impl CreateBidCommandHandler for DefaultCreateBidCommandHandler {
    async fn handle(&self, user_id: Option<UserId>, command: CreateBidCommand) -> Result<(), Error> {
        // Get the auction
        let mut auction = match self.repository.get_auction(command.auction_id).await? {
            Some(auction) => auction,
            None => return Result::Err(Error::Validation(Errors::UnknownAuction)),
        };
        let user_id = user_id
            .ok_or_else(|| Error::Unauthorized("User must be logged in to place a bid".to_string()))?;


        // Create bid
        let bid = BidData {
            user: user_id.clone(),
            amount: command.amount,
            at: self.system_clock.now(),
        };
        
        // Try to add bid to auction
        let result = match auction.try_add_bid(self.system_clock.now(), bid) {
            Ok(_) => {
                // Save updated auction
                self.repository.update_auction(auction).await?;
                Ok(())
            },
            Err(errors) => Err(errors),
        }.map_err(|e| Error::Validation(e))?;
        
        Ok(result)
    }
}

