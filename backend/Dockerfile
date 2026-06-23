FROM debian:bookworm-slim AS build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    protobuf-compiler \
    libprotobuf-dev \
    musl-tools \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none && \
    export PATH="/root/.cargo/bin:${PATH}" && \
    rustup set profile minimal && \
    rustup toolchain install nightly-x86_64-unknown-linux-gnu && \
    rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu && \
    rustup target add x86_64-unknown-linux-musl --toolchain nightly-x86_64-unknown-linux-gnu

WORKDIR /app

COPY backend .
COPY proto proto

ARG CARGO_FLAGS="--release"
ENV CC_x86_64_unknown_linux_musl=musl-gcc
ENV RUSTFLAGS="-C target-cpu=x86-64"

RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --bin grpc --features proto-in-root --target x86_64-unknown-linux-musl ${CARGO_FLAGS} && \
    PROFILE=$(echo "${CARGO_FLAGS}" | grep -q "\-\-release" && echo "release" || echo "debug") && \
    cp /app/target/x86_64-unknown-linux-musl/${PROFILE}/grpc /app/grpc

FROM alpine:latest AS runtime

RUN apk add --no-cache \
    gcompat \
    ca-certificates

RUN addgroup -g 1001 backendgroup && \
    adduser -u 1001 -G backendgroup -D -s /sbin/nologin backenduser

COPY --from=build --chown=backenduser:backendgroup \
    /app/grpc \
    /usr/local/bin/backend

USER backenduser
ENTRYPOINT ["/usr/local/bin/backend"]