FROM rust:1.75 as builder

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/spell-api /app/spell-api

# Copy WASM modules
COPY modules /opt/spell/modules

# Set environment
ENV WASM_MODULE_PATH=/opt/spell/modules
ENV RUST_LOG=info

EXPOSE 8080

CMD ["/app/spell-api"]
