FROM rust:1.82-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --shell /usr/sbin/nologin pulsar
WORKDIR /app
COPY --from=builder /app/target/release/pulsar-core /usr/local/bin/pulsar-core
COPY migrations ./migrations
USER pulsar
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/pulsar-core"]
