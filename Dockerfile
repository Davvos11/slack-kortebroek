FROM rust:1.99-trixie AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:trixie-20261005-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3t64 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/kortebroek /usr/local/bin/kortebroek
USER 65534:65534
ENTRYPOINT ["kortebroek"]
