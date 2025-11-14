# ============================
# 1. BUILD STAGE
# ============================
FROM rust:1.77-slim AS builder

# Install dependencies required to build + Chromium libs
RUN apt-get update && apt-get install -y \
    clang \
    pkg-config \
    libssl-dev \
    libx11-dev \
    libxkbcommon-dev \
    libgtk-3-dev \
    libxcomposite-dev \
    libxdamage-dev \
    libxrandr-dev \
    libgbm-dev \
    libpango1.0-dev \
    libcups2-dev \
    libnss3 \
    libasound2 \
    chromium \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY . .
RUN cargo build --release

# ============================
# 2. RUNTIME STAGE
# ============================
FROM debian:bookworm-slim

# Install Chromium + runtime libs
RUN apt-get update && apt-get install -y \
    chromium \
    libssl3 \
    libx11-6 \
    libxkbcommon0 \
    libgtk-3-0 \
    libxcomposite1 \
    libxdamage1 \
    libxrandr2 \
    libgbm1 \
    libpango-1.0-0 \
    libcups2 \
    libnss3 \
    libasound2 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/* /app/

# Render exposes PORT env var
ENV PORT=10000

# Expose port for Axum
EXPOSE 10000

# Start your server
CMD ["./rust-undergrad-thesis"]
