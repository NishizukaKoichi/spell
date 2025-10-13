# syntax=docker/dockerfile:1.7

############################
# Stage 1: chef-prep
# 依存グラフ抽出（ここは軽量・頻繁に当たる）
############################
FROM rust:latest AS chef-prep
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
WORKDIR /app
RUN cargo install cargo-chef
# 依存関係が決まる最小セットのみコピー（キャッシュ保持のため）
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

############################
# Stage 2: chef-build
# 依存をビルド（ここがキャッシュの本丸）
############################
FROM rust:latest AS chef-build
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef
COPY --from=chef-prep /app/recipe.json recipe.json
# 依存だけ cook（BuildKit のキャッシュマウントで更に高速化）
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

############################
# Stage 3: builder
# アプリ本体を差分ビルド
############################
FROM rust:latest AS builder
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
WORKDIR /app
# 依存キャッシュを引き継ぎつつ、全ソースをコピー
COPY . .
# 依存キャッシュを効かせてリリースビルド
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp /app/target/release/spell-api /app/spell-api

############################
# Stage 4: runtime
# 実行のみ（最小・ビルドツールなし）
############################
FROM debian:bullseye-slim AS runtime
ENV RUST_LOG=info
# SQLx を使うならオフラインビルド運用前提（.sqlx はリポジトリに含める）
ENV SQLX_OFFLINE=true
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates tzdata wget libssl1.1 && \
    rm -rf /var/lib/apt/lists/*
# バイナリのみコピー
COPY --from=builder /app/spell-api /app/spell-api
# ネットワークとヘルス
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD wget -qO- http://127.0.0.1:8080/health || exit 1
# Fly.io の Machines は 0.0.0.0 を期待することが多い
# アプリ側で 0.0.0.0:8080 をバインドしている前提
CMD ["/app/spell-api"]
