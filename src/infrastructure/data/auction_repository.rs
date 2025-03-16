use async_trait::async_trait;
use dyn_clone::DynClone;
use sqlx::PgPool;
use std::collections::HashSet;

use crate::domain::models::{Auction, AuctionId, Error};

dyn_clone::clone_trait_object!(AuctionRepository);

#[async_trait]
pub trait AuctionRepository: Send + Sync + DynClone {
    async fn get_auction(&self, auction_id: AuctionId) -> Result<Option<Auction>, Error>;
    async fn get_auctions(&self) -> Result<Vec<Auction>, Error>;
    async fn create_auction(&self, auction: Auction) -> Result<Auction, Error>;
    async fn update_auction(&self, auction: Auction) -> Result<Auction, Error>;
}

#[derive(Clone)]
pub struct PgAuctionRepository {
    pool: PgPool,
}

impl PgAuctionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
fn build_auction_json_query() -> &'static str {
    r#"
        json_build_object(
            'auction_id', a.id,
            'title', a.title,
            'starts_at', a.starts_at,
            'expiry', a.expiry,
            'user', a.user_id,
            'currency', a.currency,
            'auction_type', a.auction_type,
            'options', a.options,
            'expiry', a.expiry,
            'open_bidders', a.open_bidders,
            'bids', coalesce( (
                SELECT json_agg(
                    json_build_object(
                        'id', b.id,
                        'user', b.user_id,
                        'amount', json_build_object(
                            'value', b.amount_value,
                            'currency', b.amount_currency
                        ),
                        'at', b.at
                    )
                )
                FROM bids b
                WHERE b.auction_id = a.id
            ), '[]'::json)
        )
    "#
}
#[async_trait]
impl AuctionRepository for PgAuctionRepository {
    async fn get_auction(&self, auction_id: AuctionId) -> Result<Option<Auction>, Error> {
        let query = format!(
            r#"
            SELECT {} as auction
            FROM auctions a
            WHERE a.id = $1
        "#,
            build_auction_json_query()
        );

        // Note: In a real implementation, we'd handle the complex JSON deserialization
        // This is just a skeleton - real implementation would use proper row mapping
        let result = sqlx::query_scalar::<_, serde_json::Value>(&query)
            .bind(auction_id.value())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Repository(e.to_string()))?;

        match result {
            Some(json) => {
                log::info!("Auction from db {}", json);
                let auction = serde_json::from_value(json).map_err(|e| {
                    Error::Repository(format!("get_auction: Failed to deserialize auction: {}", e))
                })?;
                Ok(Some(auction))
            }
            None => Ok(None),
        }
    }

    async fn get_auctions(&self) -> Result<Vec<Auction>, Error> {
        let query = format!(
            r#"
            SELECT json_agg(
                {}
            ) as auctions
            FROM auctions a
        "#,
            build_auction_json_query()
        );

        let result = sqlx::query_scalar::<_, Option<serde_json::Value>>(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Repository(e.to_string()))?;

        match result {
            Some(json) => {
                let auctions = serde_json::from_value(json).map_err(|e| {
                    Error::Repository(format!(
                        "get_auctions: Failed to deserialize auctions: {}",
                        e
                    ))
                })?;
                Ok(auctions)
            }
            None => Ok(Vec::new()),
        }
    }

    async fn create_auction(&self, auction: Auction) -> Result<Auction, Error> {
        // Start a transaction
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Repository(e.to_string()))?;

        // Insert the auction
        let auction_json = serde_json::to_value(&auction) // TODO: there must be a better way
            .map_err(|e| {
                Error::Repository(format!(
                    "create_auction: Failed to serialize auction: {}",
                    e
                ))
            })?;

        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO auctions (
                title, starts_at, expiry, user_id, currency, 
                auction_type, options, ends_at, open_bidders
            ) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
        "#,
        )
        .bind(auction.title())
        .bind(auction.starts_at())
        .bind(auction.expiry())
        .bind(auction.user().value())
        .bind(auction.currency().to_string())
        .bind(auction.auction_type().to_string())
        .bind(
            auction_json
                .get("options")
                .unwrap_or(&serde_json::Value::Null),
        )
        .bind(match &auction {
            Auction::TimedAscending { ends_at, .. } => ends_at.clone(),
            _ => None,
        })
        .bind(auction.open_bidders())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| Error::Repository(e.to_string()))?;

        // Commit the transaction
        tx.commit()
            .await
            .map_err(|e| Error::Repository(e.to_string()))?;

        // Return the auction with the assigned ID
        let mut new_auction = auction;
        new_auction.set_auction_id(AuctionId::new(id));

        Ok(new_auction)
    }

    async fn update_auction(&self, auction: Auction) -> Result<Auction, Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Repository(e.to_string()))?;
        fn not_found(auction_id: AuctionId) -> Error {
            Error::NotFound(format!("Auction with ID {} not found", auction_id))
        }
        let auction_from_db = self
            .get_auction(auction.auction_id())
            .await?
            .ok_or(not_found(auction.auction_id()))?;
        let updated = sqlx::query(
            r#"
            UPDATE auctions
            SET expiry = $2
            WHERE id = $1
        "#,
        )
        .bind(auction.auction_id().value())
        .bind(auction.expiry())
        .execute(&mut *tx)
        .await
        .map_err(|e| Error::Repository(e.to_string()))?;

        // Check if the auction was updated
        if updated.rows_affected() == 0 {
            return Err(not_found(auction.auction_id()));
        }
        let existing_ids: HashSet<_> = auction_from_db.bids().iter().map(|b| b.id).collect();
        let incoming_ids: HashSet<_> = auction.bids().iter().map(|b| b.id).collect();
        let to_delete: Vec<_> = existing_ids.difference(&incoming_ids).collect();
        log::info!("to_delete {:#?}", to_delete);
        let to_add: Vec<_> = incoming_ids.difference(&existing_ids).collect();
        log::info!("to_add {:#?}", to_add);
        if !to_delete.is_empty() {
            return Err(Error::Internal(
                "Should not be able to delete bids".to_string(),
            ));
        }
        for &bid_id in to_add {
            let bid = auction.bids().iter().find(|b| b.id == bid_id).unwrap();
            sqlx::query(
                r#"
            INSERT INTO bids (
                auction_id, id, at, amount_value, amount_currency, user_id
            )
            VALUES ($1, $2, $3, $4, $5, $6)
        "#,
            )
            .bind(auction.auction_id().value())
            .bind(bid.id)
            .bind(bid.at())
            .bind(bid.amount().value())
            .bind(bid.amount().currency().to_string())
            .bind(bid.user().value())
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Repository(e.to_string()))?;
        }

        // Commit the transaction
        tx.commit()
            .await
            .map_err(|e| Error::Repository(e.to_string()))?;

        Ok(auction)
    }
}

#[cfg(test)]
mod repository_tests {
    use super::*;
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use crate::domain::commands::CreateAuctionCommand;
    use crate::domain::models::{Amount, AuctionFactory, BidData, CurrencyCode, UserId};
    use crate::infrastructure::run_migrations;

    fn starts_at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2016, 1, 1, 0, 0, 0).unwrap()
    }
    fn ends_at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2016, 2, 1, 0, 0, 0).unwrap()
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_with_postgres() {
        env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

        let container = Postgres::default().start().await.unwrap();
        let host_ip = container.get_host().await.unwrap();
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();

        fn match_auction(auction: &Auction) {
            assert_eq!(starts_at(), auction.starts_at(), "starts_at should match");
            assert_eq!(ends_at(), auction.expiry(), "ends_at should match");
            assert_eq!("title", auction.title(), "title should match");
        }

        async fn test_auction_repository(host_ip: String, host_port: u16) -> Result<(), Error> {
            let url = &format!(
                "postgresql://postgres:postgres@{}:{}/postgres",
                host_ip, host_port
            );
            log::info!("Connecting to {}", url);
            let pool = PgPool::connect(url)
                .await
                .map_err(|e| Error::Repository(e.to_string()))?;
            run_migrations(&pool)
                .await
                .map_err(|e| Error::Repository(e.to_string()))?;
            let repo = PgAuctionRepository::new(pool);
            let mut auction = repo
                .create_auction(
                    AuctionFactory::create_auction(
                        CreateAuctionCommand {
                            title: "title".to_string(),
                            starts_at: starts_at(),
                            ends_at: ends_at(),
                            currency: CurrencyCode::SEK,
                            min_raise: Some(10),
                            reserve_price: Some(100),
                            time_frame: None,
                            single_sealed_bid_options: None,
                            open_bidders: true,
                        },
                        UserId::new("seller"),
                    )
                    .unwrap(),
                )
                .await?;
            match_auction(&auction);
            let fetched_auction = repo.get_auction(auction.auction_id()).await?;
            assert!(
                fetched_auction.is_some(),
                "we should be able to find the created auction"
            );
            match_auction(&fetched_auction.unwrap());
            let now = auction.starts_at() + Duration::hours(1);
            let res = auction
                .try_add_bid(
                    now,
                    BidData {
                        user: UserId::new("buyer1"),
                        amount: Amount::new(10, CurrencyCode::SEK),
                        at: now,
                    },
                )
                .map_err(|e| Error::Validation(e))?;
            assert_eq!(true, res, "we should be able to add a bid");
            let updated_auction = repo.update_auction(auction.clone()).await?;
            assert_eq!(
                updated_auction.bids().len(),
                1,
                "we should be able to update the auction"
            );
            let fetched_auction_2 = repo.get_auction(auction.auction_id()).await?.unwrap();
            assert_eq!(
                fetched_auction_2.bids().len(),
                1,
                "we should still be able to get the bids"
            );

            let auctions = repo.get_auctions().await?;
            let find_auction_among_auctions = auctions
                .iter()
                .find(|a| a.auction_id() == auction.auction_id());
            assert!(
                find_auction_among_auctions.is_some(),
                "we should be able to find the auction among the auctions"
            );
            match_auction(&find_auction_among_auctions.unwrap());
            Ok(())
        }
        test_auction_repository(host_ip.to_string(), host_port)
            .await
            .unwrap();
    }
}
