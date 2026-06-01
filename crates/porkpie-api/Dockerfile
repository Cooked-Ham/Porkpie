FROM rust:latest AS builder
WORKDIR /workspace
COPY . .
RUN cargo build --release --package porkpie-api --bin porkpie-server

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /app/data
COPY --from=builder /workspace/target/release/porkpie-server /usr/local/bin/porkpie-server
ENV DATABASE_URL=sqlite:/app/data/porkpie.db
ENV API_PORT=8000
EXPOSE 8000
CMD ["porkpie-server"]
