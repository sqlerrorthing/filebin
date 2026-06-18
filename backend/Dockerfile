FROM rust:latest AS planner
WORKDIR /backend
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust AS cacher
WORKDIR /backend
RUN rustup default nightly && rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
RUN cargo install cargo-chef
COPY --from=planner /backend/recipe.json recipe.json

ARG CARGO_FLAGS="--release"
RUN cargo chef cook ${CARGO_FLAGS} --recipe-path recipe.json

FROM rust AS build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /backend

COPY . .
COPY --from=cacher /backend/target target

ARG CARGO_FLAGS="--release"
RUN rustup default nightly && rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
RUN cargo build ${CARGO_FLAGS} --bin grpc

FROM debian:stable-slim AS runtime
RUN groupadd -g 1001 backendgroup && \
    useradd -u 1001 -g backendgroup -m -d /home/backenduser -s /bin/bash backenduser

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*
ARG CARGO_FLAGS="--release"
COPY --from=build --chown=backenduser:backendgroup \
    /backend/target/${CARGO_FLAGS:+release}${CARGO_FLAGS:-debug}/grpc \
    /usr/local/bin/backend

USER backenduser
ENTRYPOINT ["/usr/local/bin/backend"]