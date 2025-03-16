use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Scope};
use chrono::{DateTime, Utc};
use log::error;

use crate::api::models::{AuctionModel, CreateAuctionModel, CreateBidModel};
use crate::domain::commands::{CreateAuctionCommand, CreateBidCommand};
use crate::domain::models::{Auction, AuctionId, Error, Errors, SingleSealedBidOptions};
use crate::domain::services::SystemClock;
use crate::infrastructure::{jwt_payload_handling, AuctionRepository};
use crate::infrastructure::services::{CreateAuctionCommandHandler, CreateBidCommandHandler};

pub fn map_auction_to_model (auction:&Auction, now:DateTime<Utc>) -> AuctionModel {
    let has_ended = auction.has_ended(now);
    let winner_info = auction.try_get_amount_and_winner(now);
    
    AuctionModel {
        id: auction.auction_id().value(),
        starts_at: auction.starts_at(),
        title: auction.title().to_string(),
        expiry: auction.expiry(),
        seller: Some(auction.user().to_string()),
        currency: auction.currency(),
        bids: auction.get_bids(now).map_or_else(|| {Vec::new()},|bids| {bids.iter().map(|bid| {
            // In a real application, we'd use a proper mapper service
            // that takes care of bidder representation based on open_bidders setting
            crate::api::models::BidModel {
                amount: bid.amount(),
                bidder: Some(bid.user().to_string()),
                at: bid.at() - auction.starts_at(),
            }
        }).collect()}),
        price: winner_info.as_ref().map(|(amount, _)| amount.clone()),
        winner: winner_info.as_ref().map(|(_, user)| user.to_string()),
        has_ended,
    }
}

// Get all auctions
#[get("/auctions")]
pub async fn get_auctions(
    query: web::Data<Box<dyn AuctionRepository>>,
    clock: web::Data<Box<dyn SystemClock>>,
) -> impl Responder {
    match query.get_auctions().await {
        Ok(auctions) => {
            let now = clock.now();
            
            // Map domain auctions to API models
           
            let models: Vec<AuctionModel> = auctions.iter().map(|auction| { 
                return map_auction_to_model(auction,now)
            }).collect();
            HttpResponse::Ok().json(models)
        },
        Err(e) => {
            log::error!("Error getting auctions: {:?}", e);
            HttpResponse::InternalServerError().json(format!("Internal server error: {}", e))
        }
    }
}

// Get a single auction
#[get("/auctions/{auction_id}")]
pub async fn get_auction(
    auction_id: web::Path<i64>,
    query: web::Data<Box<dyn AuctionRepository>>,
    clock: web::Data<Box<dyn SystemClock>>,
) -> impl Responder {
    let id = AuctionId::new(*auction_id);
    
    match query.get_auction(id).await {
        Ok(Some(auction)) => {
            let now = clock.now();
            let model= map_auction_to_model(&auction,now);            
            HttpResponse::Ok().json(model)
        },
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Error getting auction {}: {:?}", auction_id, e);
            HttpResponse::InternalServerError().json(format!("Internal server error: {}", e))
        }
    }
}

// Create an auction
#[post("/auction")]
pub async fn create_auction(
    req: HttpRequest,
    model: web::Json<CreateAuctionModel>,
    clock: web::Data<Box<dyn SystemClock>>,
    handler: web::Data<Box<dyn CreateAuctionCommandHandler>>,
) -> impl Responder {
    // TODO: Move to configurable middleware
    let user = jwt_payload_handling::from_request(&req);
    // Convert API model to domain command
    let single_sealed_bid_options = match model.single_sealed_bid_options.as_deref() {
        Some("Blind") => Some(SingleSealedBidOptions::Blind),
        Some("Vickrey") => Some(SingleSealedBidOptions::Vickrey),
        _ => None,
    };
    
    let time_frame = model.time_frame.map(|seconds| chrono::Duration::seconds(seconds));
    
    let command = CreateAuctionCommand {
        title: model.title.clone(),
        currency: model.currency,
        starts_at: model.starts_at,
        ends_at: model.ends_at,
        min_raise: model.min_raise,
        reserve_price: model.reserve_price,
        time_frame,
        single_sealed_bid_options,
        open_bidders: model.open_bidders,
    };

    match handler.handle(user, command).await {
        Ok(auction) => {
            let now = clock.now();
            // Return the created auction
            HttpResponse::Created().json(map_auction_to_model(&auction, now))
        },
        Err(Error::Unauthorized(msg)) => {
            HttpResponse::Unauthorized().json(msg)
        },
        Err(e) => {
            error!("Error creating auction: {:?}", e);
            HttpResponse::InternalServerError().json(format!("Internal server error: {}", e))
        }
    }
}

// Create a bid
#[post("/auctions/{auction_id}/bids")]
pub async fn create_bid(
    req: HttpRequest,
    auction_id: web::Path<i64>,
    model: web::Json<CreateBidModel>,
    handler: web::Data<Box<dyn CreateBidCommandHandler>>,
) -> impl Responder {
    // TODO: Move to configurable middleware
    let user = jwt_payload_handling::from_request(&req);

    let id = AuctionId::new(*auction_id);
    
    // Convert API model to domain command
    let command = CreateBidCommand {
        amount: model.amount.clone(),
        auction_id: id,
    };
    
    match handler.handle(user, command).await {
        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(Error::Validation(Errors::UnknownAuction)) => HttpResponse::NotFound().finish(),
        Err(Error::Validation(errors)) => HttpResponse::BadRequest().json(errors.to_string()),

        Err(Error::Unauthorized(msg)) => {
            HttpResponse::Unauthorized().json(msg)
        },
        Err(e) => {
            error!("Error creating bid: {:?}", e);
            HttpResponse::InternalServerError().json(format!("Internal server error: {}", e))
        }
    }
}

// Configure routes
pub fn get_scope() -> Scope {
    web::scope("")
            .service(get_auctions)
            .service(create_auction)
            .service(get_auction)
            .service(create_bid)
}
