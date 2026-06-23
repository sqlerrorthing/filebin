FROM clux/muslrust:nightly AS build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    protobuf-compiler \
    libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

WORKDIR /app

COPY backend .
COPY proto proto

ARG CARGO_FLAGS="--release"
ENV RUSTFLAGS="-C target-cpu=x86-64"

RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --bin grpc --features proto-in-root ${CARGO_FLAGS} && \
    PROFILE=$(echo "${CARGO_FLAGS}" | grep -q "\-\-release" && echo "release" || echo "debug") && \
    cp /app/target/x86_64-unknown-linux-musl/${PROFILE}/grpc /app/grpc

FROM gcr.io/distroless/static:latest AS runtime

COPY --from=build \
    /app/grpc \
    /usr/local/bin/backend

ENTRYPOINT ["/usr/local/bin/backend"]