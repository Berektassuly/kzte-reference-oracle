FROM rust:1.84-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Anchor.toml ./
COPY crates ./crates
COPY programs ./programs
COPY tests ./tests
COPY config ./config

RUN cargo build --release -p kzte-feeder -p kzte-cli

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/kzte-feeder /usr/local/bin/kzte-feeder
COPY --from=builder /app/target/release/kzte-cli /usr/local/bin/kzte-cli
COPY config/feeder.example.toml /app/config/feeder.example.toml

ENTRYPOINT ["kzte-feeder"]
CMD ["--config", "/app/config/feeder.example.toml"]
