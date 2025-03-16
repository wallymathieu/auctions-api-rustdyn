use async_trait::async_trait;
use dyn_clone::DynClone;

use crate::domain::commands::CreateAuctionCommand;
use crate::domain::models::{Auction, Error, UserId};
use crate::domain::models::auction::AuctionFactory;
use crate::infrastructure::data::AuctionRepository;

#[async_trait]
pub trait CreateAuctionCommandHandler: Send + Sync + DynClone {
    async fn handle(&self, user_id: Option<UserId>, command: CreateAuctionCommand) -> Result<Auction, Error>;
}

dyn_clone::clone_trait_object!(CreateAuctionCommandHandler);

#[derive(Clone)]
pub struct DefaultCreateAuctionCommandHandler {
    repository: Box<dyn AuctionRepository>,
}

impl DefaultCreateAuctionCommandHandler {
    pub fn new(
        repository: Box<dyn AuctionRepository>,
    ) -> Self {
        Self {
            repository,
        }
    }
}

#[async_trait]
impl CreateAuctionCommandHandler for DefaultCreateAuctionCommandHandler {
    async fn handle(&self, user_id: Option<UserId>, command: CreateAuctionCommand) -> Result<Auction, Error> {
        let user_id = user_id
            .ok_or_else(|| Error::Unauthorized("User must be logged in to create an auction".to_string()))?;

        // Create the auction using the factory
        let auction = AuctionFactory::create_auction(command, user_id)
            .map_err(|e| Error::Domain(e.to_string()))?;
            
        // Save to repository
        let saved_auction = self.repository.create_auction(auction).await?;
        
        Ok(saved_auction)
    }
}
