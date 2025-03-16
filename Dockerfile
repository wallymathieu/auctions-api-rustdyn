FROM rust:1.85 as builder

WORKDIR /usr/src/auctions-api

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {println!(\"dummy\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/auctions-api/target/release/auctions-api /app/
COPY --from=builder /usr/src/auctions-api/config /app/config
COPY --from=builder /usr/src/auctions-api/migrations /app/migrations

# Set environment variables
ENV RUN_ENV=production

# Set the entrypoint
ENTRYPOINT ["/app/auctions-api"]