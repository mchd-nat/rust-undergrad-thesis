# -------------------------------------------------
#  Builder stage – compile the Rust binary
# -------------------------------------------------
FROM rust:1.79-slim AS builder

WORKDIR /app

# ---- System build‑tools & libs needed by Chromiumoxide ----
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    # ---- Chromiumoxide native deps ----
    libgtk-3-dev \
    libx11-dev \
    libxcb1-dev \
    libxrandr-dev \
    libasound2-dev \
    libatk-bridge2.0-dev \
    libdrm-dev \
    libgbm-dev \
    libwayland-client0 \
    libwayland-cursor0 \
    libwayland-egl0 \
    libxcomposite-dev \
    libxdamage-dev \
    libxfixes-dev \
    libxkbcommon-dev \
    libxshmfence-dev \
    
    curl \
    && rm -rf /var/lib/apt/lists/*

# ---- Cache dependencies (Cargo.lock + Cargo.toml) ----
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch   # download crates without compiling yet

# ---- Copy the rest of the source and build ----
COPY . .
RUN cargo build --release

# -------------------------------------------------
#  Runtime stage – smallest possible image
# -------------------------------------------------
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    libgtk-3-0 \
    libx11-6 \
    libxcb1 \
    libxrandr2 \
    libasound2 \
    libatk-bridge2.0-0 \
    libdrm2 \
    libgbm1 \
    libwayland-client0 \
    libwayland-cursor0 \
    libwayland-egl0 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxkbcommon0 \
    libxshmfence1 \
    ca-certificates \
    firefox-esr \
    wget \
    && rm -rf /var/lib/apt/lists/*

# ---- Geckodriver ----
ENV GECKODRIVER_VERSION=v0.35.0
RUN wget -O /tmp/geckodriver.tar.gz \
    "https://github.com/mozilla/geckodriver/releases/download/${GECKODRIVER_VERSION}/geckodriver-${GECKODRIVER_VERSION}-linux64.tar.gz" \
    && tar -xzf /tmp/geckodriver.tar.gz -C /usr/local/bin \
    && rm /tmp/geckodriver.tar.gz \
    && chmod +x /usr/local/bin/geckodriver

# ---- Copy the compiled binary from the builder ----
COPY --from=builder /app/target/release/rust-undergrad-thesis /usr/local/bin/app

# ---- Small entrypoint that starts geckodriver then the app ----
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

ENV PORT=8080
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]