FROM rust:1.80-bullseye AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && update-ca-certificates

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    chromium \
    chromium-driver \
    libssl1.1 \
    ca-certificates \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 9515

COPY --from=builder /app/target/release/rust-undergrad-thesis /usr/local/bin/app

CMD chromedriver --port=9515 --allowed-origins=* & \
    sleep 1 && \
    app
