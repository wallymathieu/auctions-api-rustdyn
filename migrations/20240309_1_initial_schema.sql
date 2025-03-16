CREATE TABLE auctions (
    id BIGSERIAL PRIMARY KEY,
    title VARCHAR(200) NOT NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    expiry TIMESTAMPTZ NOT NULL,
    user_id VARCHAR(2000) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    auction_type VARCHAR(50) NOT NULL,
    options JSONB,
    ends_at TIMESTAMPTZ,
    open_bidders BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE bids (
    id BIGINT,
    auction_id BIGINT NOT NULL REFERENCES auctions(id),
    user_id VARCHAR(2000) NOT NULL,
    amount_value BIGINT NOT NULL,
    amount_currency VARCHAR(3) NOT NULL,
    at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(id, auction_id),
    CONSTRAINT fk_auction FOREIGN KEY (auction_id) REFERENCES auctions(id) ON DELETE CASCADE
);

CREATE INDEX idx_bids_auction_id ON bids(auction_id);
CREATE INDEX idx_bids_user_id ON bids(user_id);
CREATE INDEX idx_auctions_user_id ON auctions(user_id);

-- Create a function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create a trigger to automatically update the updated_at column
CREATE TRIGGER update_auctions_updated_at
BEFORE UPDATE ON auctions
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();