# syntax=docker/dockerfile:1
FROM rust:1.91-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
# Create dummy src to cache dependency compilation when only source changes.
RUN mkdir -p src/bin && \
    printf 'pub fn dummy(){}\n' > src/lib.rs && \
    printf 'fn main(){}\n' > src/bin/api.rs && \
    printf 'fn main(){}\n' > src/bin/worker.rs
RUN cargo build --release 2>/dev/null || true
COPY src ./src
COPY migrations ./migrations
RUN touch src/lib.rs src/bin/api.rs src/bin/worker.rs && cargo build --release --bin api --bin worker

FROM debian:bookworm-slim AS runtime-base
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app

FROM runtime-base AS api
COPY --from=builder /app/target/release/api /usr/local/bin/api
EXPOSE 8080
CMD ["api"]

FROM runtime-base AS worker
COPY --from=builder /app/target/release/worker /usr/local/bin/worker
CMD ["worker"]
