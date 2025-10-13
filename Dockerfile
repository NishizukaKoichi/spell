FROM rust:latest as builder

WORKDIR /app

# Use nightly for edition2024 support
RUN rustup default nightly

# Copy Cargo files
COPY Cargo.toml ./
COPY src ./src

# Build release binary
RUN cargo +nightly build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/spell-api /app/spell-api

# Set environment
ENV RUST_LOG=info

EXPOSE 8080

CMD ["/app/spell-api"]
