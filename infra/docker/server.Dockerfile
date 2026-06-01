FROM rust:1-bookworm AS builder

WORKDIR /app
COPY . .

# The server binary is defined in the porkpie-api crate, not the porkpie-server package.
RUN cargo build --release -p porkpie-api

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates sqlite3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/porkpie-server /usr/local/bin/porkpie-server

RUN useradd --system --create-home --home-dir /nonexistent --shell /usr/sbin/nologin porkpie \
    && mkdir -p /data \
    && chown -R porkpie:porkpie /data

USER porkpie

EXPOSE 8080

CMD ["/usr/local/bin/porkpie-server"]
