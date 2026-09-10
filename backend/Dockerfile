FROM clux/muslrust:nightly AS chef

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    protobuf-compiler \
    libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add rust-src \
    --toolchain nightly-x86_64-unknown-linux-gnu

RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh \
    | bash

RUN cargo binstall cargo-chef --locked \
    && cargo binstall sccache --locked

WORKDIR /app

ENV RUSTFLAGS="-C target-cpu=x86-64"
ENV RUSTC_WRAPPER="/opt/cargo/bin/sccache"

FROM chef AS planner

COPY backend .
COPY proto proto

RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS deps

ARG CARGO_FLAGS="--release"

COPY --from=planner /app/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    --mount=type=cache,target=/root/.cache/sccache,sharing=locked \
    cargo chef cook \
        ${CARGO_FLAGS} \
        --recipe-path recipe.json


FROM deps AS builder

ARG CARGO_FLAGS="--release"

COPY backend .
COPY proto proto

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    --mount=type=cache,target=/root/.cache/sccache,sharing=locked \
    cargo build \
        --bin grpc \
        --features proto-in-root \
        ${CARGO_FLAGS} \
    && \
    PROFILE=$( \
        echo "${CARGO_FLAGS}" | \
        grep -q -- "--release" && \
        echo release || \
        echo debug \
    ) \
    && \
    cp \
        "/app/target/x86_64-unknown-linux-musl/${PROFILE}/grpc" \
        /app/grpc


FROM gcr.io/distroless/static:latest AS runtime

COPY --from=builder /app/grpc /usr/local/bin/backend

ENTRYPOINT ["/usr/local/bin/backend"]
