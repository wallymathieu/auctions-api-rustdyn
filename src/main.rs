// src/main.rs
use actix_web::{App, HttpServer, middleware::Logger, web};
use dotenv::dotenv;

use auctions_api::{
    domain::services::{RealSystemClock, SystemClock}, infrastructure::{
        data::{create_pg_pool, migrations::run_migrations, PgAuctionRepository},
        services::{
            CreateAuctionCommandHandler, CreateBidCommandHandler, 
            DefaultCreateAuctionCommandHandler,
            DefaultCreateBidCommandHandler,
        },
        AuctionRepository, Settings,
    }, 
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();
    
    // Configure logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Load configuration
    let config = Settings::new().expect("Failed to load configuration");
    log::info!("Starting server in {} environment", config.environment);
    
    // Create database connection pool
    let db_pool = create_pg_pool(&config.database.url).await
        .expect("Failed to create database pool");
    
    // Run database migrations
    log::info!("Running database migrations");
    if let Err(e) = run_migrations(&db_pool).await {
        log::error!("Failed to run migrations: {}", e);
        std::process::exit(1);
    }
    
    // Create system clock
    let system_clock: Box<dyn SystemClock> = Box::new(RealSystemClock);
    
    // Create repositories and queries
    let auction_repository: Box<dyn AuctionRepository> = Box::new(PgAuctionRepository::new(db_pool.clone()));
    
    // Create command handlers
    let create_auction_handler: Box<dyn CreateAuctionCommandHandler> = Box::new(DefaultCreateAuctionCommandHandler::new(
        auction_repository.clone(),
    ));

    
    let create_bid_handler: Box<dyn CreateBidCommandHandler> = Box::new(DefaultCreateBidCommandHandler::new(
        auction_repository.clone(),
        system_clock.clone(),
    ));
    
    // Start HTTP server
    log::info!("Starting HTTP server on {}:{}", config.server.host, config.server.port);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(create_auction_handler.clone()))
            .app_data(web::Data::new(create_bid_handler.clone()))
            .app_data(web::Data::new(system_clock.clone()))
            .app_data(web::Data::new(auction_repository.clone()))
            .service(auctions_api::api::handlers::auctions::get_scope())
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}

