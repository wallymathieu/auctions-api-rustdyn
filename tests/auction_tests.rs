use auctions_api::domain::models::{
    Amount, Auction, AuctionBase, AuctionId, Bid, BidData, CurrencyCode, Errors,
    SingleSealedBidOptions, TimedAscendingOptions, UserId,
};
use chrono::Duration;
use chrono::{DateTime, TimeZone, Utc};

pub fn auction_id() -> AuctionId {
    AuctionId::new(1)
}

pub fn title() -> &'static str {
    "auction"
}

pub fn initial_now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2015, 12, 4, 0, 0, 0).unwrap()
}

pub fn starts_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2016, 1, 1, 0, 0, 0).unwrap()
}

pub fn ends_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2016, 2, 1, 0, 0, 0).unwrap()
}

pub fn seller() -> UserId {
    UserId::new("x1".to_string())
}

pub fn buyer() -> UserId {
    UserId::new("x2".to_string())
}

pub fn get_english_auction() -> Auction {
    Auction::TimedAscending {
        base: AuctionBase {
            auction_id: auction_id(),
            title: title().to_string(),
            starts_at: starts_at(),
            expiry: ends_at(),
            user: seller(),
            currency: CurrencyCode::SEK,
            bids: Vec::new(),
            open_bidders: true,
        },
        options: TimedAscendingOptions {
            min_raise: 10,
            time_frame: Duration::minutes(1),
            reserve_price: 150,
        },
        ends_at: None,
    }
}

pub fn vickrey_auction() -> Auction {
    Auction::SingleSealedBid {
        base: AuctionBase {
            auction_id: auction_id(),
            title: title().to_string(),
            starts_at: starts_at(),
            expiry: ends_at(),
            user: seller(),
            currency: CurrencyCode::SEK,
            open_bidders: true,
            bids: Vec::new(),
        },
        options: SingleSealedBidOptions::Vickrey,
    }
}

pub fn blind_auction() -> Auction {
    Auction::SingleSealedBid {
        base: AuctionBase {
            auction_id: auction_id(),
            title: title().to_string(),
            starts_at: starts_at(),
            expiry: ends_at(),
            user: seller(),
            currency: CurrencyCode::SEK,
            open_bidders: true,
            bids: Vec::new(),
        },
        options: SingleSealedBidOptions::Blind,
    }
}

pub fn sek(a: i64) -> Amount {
    Amount::new(a, CurrencyCode::SEK)
}

pub fn buyer1() -> UserId {
    UserId::new("x2".to_string())
}

pub fn buyer2() -> UserId {
    UserId::new("x3".to_string())
}

pub fn bid1() -> BidData {
    BidData {
        user: buyer1(),
        amount: sek(10),
        at: starts_at() + Duration::hours(2),
    }
}

pub fn bid2() -> BidData {
    BidData {
        user: buyer2(),
        amount: sek(12),
        at: starts_at() + Duration::hours(2),
    }
}

fn create_sample_bid(user_id: &str, amount: i64, hours_after_start: i64) -> BidData {
    BidData {
        user: UserId::new(user_id),
        amount: sek(amount),
        at: starts_at() + Duration::hours(hours_after_start),
    }
}

#[test]
fn test_timed_ascending_auction_add_bid() {
    let mut auction = get_english_auction();

    // Create a valid bid
    let now = auction.starts_at() + Duration::hours(1);
    let bid = create_sample_bid("buyer1", 150, 1);

    // Add the bid
    let result = auction.try_add_bid(now, bid);
    assert!(result.is_ok());

    // Verify the bid was added
    assert_eq!(auction.bids().len(), 1);
}

#[test]
fn test_timed_ascending_auction_add_bid_before_start() {
    let mut auction = get_english_auction();

    // Create a bid before the auction starts
    let now = auction.starts_at() - Duration::hours(1);
    let bid = create_sample_bid("buyer1", 150, -1);

    // Try to add the bid
    let result = auction.try_add_bid(now, bid);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Errors::AuctionHasNotStarted);
}

#[test]
fn test_timed_ascending_auction_add_bid_after_end() {
    let mut auction = get_english_auction();

    // Create a bid after the auction ends
    let now = auction.expiry() + Duration::hours(1);
    let bid = create_sample_bid("buyer1", 150, 31 * 24 + 1); // 1 hour after expiry

    // Try to add the bid
    let result = auction.try_add_bid(now, bid);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Errors::AuctionHasEnded);
}

#[test]
fn test_timed_ascending_auction_add_bid_min_raise() {
    let mut auction = get_english_auction();

    // First bid is always valid if it meets other criteria
    let now = auction.starts_at() + Duration::hours(1);
    let bid1 = create_sample_bid("buyer1", 50, 1);
    let result1 = auction.try_add_bid(now, bid1);
    assert!(result1.is_ok(), "Expected success");

    // Second bid must be at least min_raise higher
    let now = auction.starts_at() + Duration::hours(2);
    let bid2 = create_sample_bid("buyer2", 51, 2); // Only 1 higher but min_raise is 10
    let result2 = auction.try_add_bid(now, bid2);
    assert!(result2.is_err(), "Expected error");
    assert_eq!(result2.unwrap_err(), Errors::MustRaiseWithAtLeast);

    // A valid second bid
    let bid3 = create_sample_bid("buyer2", 60, 2); // 20 higher, which exceeds min_raise of 10
    let result3 = auction.try_add_bid(now, bid3);
    assert!(result3.is_ok(), "Expected success");
}

#[test]
fn test_timed_ascending_auction_has_ended() {
    let auction = get_english_auction();

    // Before expiry
    let before = auction.expiry() - Duration::hours(1);
    assert!(!auction.has_ended(before));

    // After expiry
    let after = auction.expiry() + Duration::hours(1);
    assert!(auction.has_ended(after));
}

#[test]
fn test_single_sealed_bid_auction_add_bid() {
    let mut auction = blind_auction();

    // Create a valid bid
    let now = auction.starts_at() + Duration::hours(1);
    let bid = create_sample_bid("buyer1", 150, 1);

    // Add the bid
    let result = auction.try_add_bid(now, bid);
    assert!(result.is_ok());

    // Verify the bid was added
    assert_eq!(auction.bids().len(), 1);

    // Try to add another bid from the same user
    let now = auction.starts_at() + Duration::hours(2);
    let bid2 = create_sample_bid("buyer1", 200, 2);
    let result2 = auction.try_add_bid(now, bid2);
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err(), Errors::AlreadyPlacedBid);
}

#[test]
fn test_single_sealed_bid_auction_winner_blind() {
    let mut auction = blind_auction();

    // Add bids
    let now = auction.starts_at() + Duration::hours(1);
    let bid1 = create_sample_bid("buyer1", 150, 1);
    let bid2 = create_sample_bid("buyer2", 200, 2);

    assert!(auction.try_add_bid(now, bid1).is_ok());
    assert!(auction.try_add_bid(now, bid2).is_ok());

    // Before auction ends, no winner
    let before_end = auction.expiry() - Duration::hours(1);
    assert!(auction.try_get_amount_and_winner(before_end).is_none());

    // After auction ends, highest bidder wins and pays their bid
    let after_end = auction.expiry() + Duration::hours(1);
    let winner_info = auction.try_get_amount_and_winner(after_end);
    assert!(winner_info.is_some());

    let (amount, winner) = winner_info.unwrap();
    assert_eq!(amount.value(), 200); // Highest bid amount
    assert_eq!(winner.value(), "buyer2"); // Highest bidder
}

#[test]
fn test_single_sealed_bid_auction_winner_vickrey() {
    let mut auction = vickrey_auction();

    // Add bids
    let now = auction.starts_at() + Duration::hours(1);
    let bid1 = create_sample_bid("buyer1", 150, 1);
    let bid2 = create_sample_bid("buyer2", 200, 2);

    assert!(auction.try_add_bid(now, bid1).is_ok());
    assert!(auction.try_add_bid(now, bid2).is_ok());

    // After auction ends, highest bidder wins but pays second highest bid
    let after_end = auction.expiry() + Duration::hours(1);
    let winner_info = auction.try_get_amount_and_winner(after_end);
    assert!(winner_info.is_some());

    let (amount, winner) = winner_info.unwrap();
    assert_eq!(amount.value(), 150); // Second-highest bid amount
    assert_eq!(winner.value(), "buyer2"); // Highest bidder
}

#[test]
fn test_bid_validation_seller_cannot_bid() {
    // Create an auction
    let auction = blind_auction();

    // Create a bid from the seller
    let bid = Bid::new(
        1,
        seller(),
        sek(100),
        starts_at()
            .checked_add_signed(Duration::seconds(1))
            .unwrap(),
    );

    // Validate the bid
    let errors = bid.validate(&auction);
    assert_eq!(errors, Errors::SellerCannotPlaceBids);
}

#[test]
fn test_bid_validation_currency_mismatch() {
    // Create an auction
    let auction = blind_auction();

    // Create a bid with different currency
    let bid = Bid::new(
        2,
        buyer(),
        Amount::new(100, CurrencyCode::VAC),
        starts_at()
            .checked_add_signed(Duration::seconds(1))
            .unwrap(),
    );

    // Validate the bid
    let errors = bid.validate(&auction);
    assert_eq!(errors, Errors::BidCurrencyConversion);
}

#[test]
fn test_bid_validation_auction_timing() {
    // Create an auction
    let auction = blind_auction();

    // Create a bid before auction starts
    let before_bid = Bid::new(
        3,
        buyer(),
        Amount::new(100, CurrencyCode::SEK),
        starts_at()
            .checked_add_signed(Duration::seconds(-1))
            .unwrap(),
    );

    // Validate the bid
    let errors = before_bid.validate(&auction);
    assert_eq!(errors, Errors::AuctionHasNotStarted);

    // Create a bid after auction ends
    let after_bid = Bid::new(
        4,
        UserId::new("buyer1"),
        Amount::new(100, CurrencyCode::SEK),
        ends_at().checked_add_signed(Duration::seconds(1)).unwrap(),
    );

    // Validate the bid
    let errors = after_bid.validate(&auction);
    assert_eq!(errors, Errors::AuctionHasEnded);
}
