# syntax=docker/dockerfile:1.7

# This Dockerfile is optimized for an AMD64 build host compiling for amd64 + arm64.

# -------- Builder --------
FROM --platform=linux/amd64 rust:1.80.1-slim AS builder
WORKDIR /app

# Build deps
RUN apt-get update && apt-get install -yqq \
    cmake gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu libpq-dev curl bzip2

# Manually compile an arm64 build of libpq
ENV PGVER=16.4
RUN curl -o postgresql.tar.bz2 https://ftp.postgresql.org/pub/source/v${PGVER}/postgresql-${PGVER}.tar.bz2 && \
    tar xjf postgresql.tar.bz2 && \
    cd postgresql-${PGVER} && \
    ./configure --host=aarch64-linux-gnu --enable-shared --disable-static --without-readline --without-zlib --without-icu && \
    cd src/interfaces/libpq && \
    make

COPY . .
RUN rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

# Add ids to cache mounts (Railway requirement)
# Add `--no-default-features` to cargo commands if you don't want stats collection
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-target,target=/app/target \
    cargo build --release --target=x86_64-unknown-linux-gnu && \
    RUSTFLAGS="-L /app/postgresql-${PGVER}/src/interfaces/libpq -C linker=aarch64-unknown-linux-gnu-gcc" \
    cargo build --release --target=aarch64-unknown-linux-gnu && \
    cp /app/target/x86_64-unknown-linux-gnu/release/spoticord /app/x86_64 && \
    cp /app/target/aarch64-unknown-linux-gnu/release/spoticord /app/aarch64

# -------- Runtime --------
FROM debian:bookworm-slim

ARG TARGETPLATFORM
ENV TARGETPLATFORM=${TARGETPLATFORM}

RUN apt-get update && apt-get install -y ca-certificates libpq-dev && rm -rf /var/lib/apt/lists/*

# Copy binaries built for both arches
COPY --from=builder /app/x86_64 /tmp/x86_64
COPY --from=builder /app/aarch64 /tmp/aarch64

# Choose correct binary at build time
RUN if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then \
      cp /tmp/x86_64 /usr/local/bin/spoticord; \
    elif [ "${TARGETPLATFORM}" = "linux/arm64" ]; then \
      cp /tmp/aarch64 /usr/local/bin/spoticord; \
    else \
      echo "Unsupported TARGETPLATFORM: ${TARGETPLATFORM}" && exit 1; \
    fi && \
    rm -rvf /tmp/x86_64 /tmp/aarch64

ENTRYPOINT ["/usr/local/bin/spoticord"]
