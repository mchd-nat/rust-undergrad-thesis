FROM rust:1.79-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    firefox-esr \
    wget \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

ENV GECKODRIVER_VERSION=v0.35.0
RUN wget -O /tmp/geckodriver.tar.gz \
    "https://github.com/mozilla/geckodriver/releases/download/${GECKODRIVER_VERSION}/geckodriver-${GECKODRIVER_VERSION}-linux64.tar.gz" \
    && tar -xzf /tmp/geckodriver.tar.gz -C /usr/local/bin \
    && rm /tmp/geckodriver.tar.gz \
    && chmod +x /usr/local/bin/geckodriver

COPY --from=builder /app/target/release/* /usr/local/bin/app

ENV PORT=8080
EXPOSE 8080

CMD geckodriver --port 4444 --log debug & \
    sleep 1 && \
    app
