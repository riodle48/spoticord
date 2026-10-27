# Railway-friendly single-target build (amd64 only, no stats/Redis)
FROM rust:1-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends cmake gcc g++ pkg-config libpq-dev libssl-dev ca-certificates git && rm -rf /var/lib/apt/lists/*
COPY . .
# Keep memory in check on Railway's many-core builders
ENV CARGO_BUILD_JOBS=4
ENV CARGO_PROFILE_RELEASE_LTO=thin
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
RUN cargo build --release --no-default-features

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libpq5 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/spoticord /usr/local/bin/spoticord
ENTRYPOINT ["/usr/local/bin/spoticord"]
