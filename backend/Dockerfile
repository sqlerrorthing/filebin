FROM rustlang/rust:nightly-slim AS build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential \
    protobuf-compiler \
    libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

WORKDIR /app

COPY backend .
COPY proto proto

ARG CARGO_FLAGS="--release"

RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --bin grpc --features proto-in-root ${CARGO_FLAGS} && \
    if echo "${CARGO_FLAGS}" | grep -q "\-\-release"; then \
        PROFILE="release"; \
    else \
        PROFILE="debug"; \
    fi && \
    cp /app/target/${PROFILE}/grpc /app/grpc

FROM debian:stable-slim AS runtime
RUN groupadd -g 1001 backendgroup && \
    useradd -u 1001 -g backendgroup -m -d /home/backenduser -s /bin/bash backenduser

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

ARG CARGO_FLAGS="--release"
COPY --from=build --chown=backenduser:backendgroup \
    /app/grpc \
    /usr/local/bin/backend

USER backenduser
ENTRYPOINT ["/usr/local/bin/backend"]