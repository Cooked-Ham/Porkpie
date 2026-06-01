# Infra Deployment Scaffold Task

## Binding

You are working only on the Porkpie `infra/` folder and minimal server config/docs needed to support it.

Do not add application features.
Do not change crypto.
Do not change UI.
Do not introduce Electron, React, TypeScript, or Vite.
Do not commit real secrets.
Do not add public default API keys.

## Goal

Fill the existing empty infrastructure folders:

```text
infra/
  caddy/
  compose/
  docker/

with a usable self-host deployment scaffold.

Required Files

Create:

infra/
  caddy/
    Caddyfile

  compose/
    docker-compose.yml
    docker-compose.dev.yml
    .env.example
    README.md

  docker/
    server.Dockerfile
    README.md
Required Behavior
Docker Compose must build and run the Porkpie server.
Compose must mount a persistent data volume.
Compose must not contain real secrets.
Compose must not use public default API keys.
.env.example may contain placeholder values only.
Real .env must not be committed.
Caddy must reverse proxy to the Porkpie server.
Server must fail startup if required secrets/config are missing.
Infra docs must explain how to run the stack.
Expected Layout
infra/compose/docker-compose.yml
infra/compose/docker-compose.dev.yml
infra/compose/.env.example
infra/compose/README.md
infra/docker/server.Dockerfile
infra/docker/README.md
infra/caddy/Caddyfile
Suggested infra/compose/.env.example
PORKPIE_PUBLIC_URL=https://sync.porkpie.love
PORKPIE_SERVER_BIND=0.0.0.0:8080
PORKPIE_DATABASE_URL=sqlite:///data/porkpie-server.db

# Replace before real use. The server must reject this placeholder at runtime.
PORKPIE_API_KEY=replace-with-a-generated-secret
Suggested infra/compose/docker-compose.yml
services:
  porkpie-server:
    build:
      context: ../..
      dockerfile: infra/docker/server.Dockerfile
    container_name: porkpie-server
    restart: unless-stopped
    env_file:
      - .env
    volumes:
      - porkpie-data:/data
    expose:
      - "8080"
    healthcheck:
      test: ["CMD", "/usr/local/bin/porkpie-server", "--healthcheck"]
      interval: 30s
      timeout: 5s
      retries: 5

  caddy:
    image: caddy:2
    container_name: porkpie-caddy
    restart: unless-stopped
    depends_on:
      - porkpie-server
    ports:
      - "80:80"
      - "443:443"
    environment:
      PORKPIE_PUBLIC_HOST: ${PORKPIE_PUBLIC_HOST:-sync.porkpie.love}
    volumes:
      - ../caddy/Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy-data:/data
      - caddy-config:/config

volumes:
  porkpie-data:
  caddy-data:
  caddy-config:
Suggested infra/compose/docker-compose.dev.yml
services:
  porkpie-server:
    build:
      context: ../..
      dockerfile: infra/docker/server.Dockerfile
    container_name: porkpie-server-dev
    restart: unless-stopped
    env_file:
      - .env
    ports:
      - "8080:8080"
    volumes:
      - porkpie-dev-data:/data

volumes:
  porkpie-dev-data:
Suggested infra/caddy/Caddyfile
{
    email admin@porkpie.love
}

{$PORKPIE_PUBLIC_HOST} {
    encode zstd gzip

    header {
        X-Content-Type-Options nosniff
        X-Frame-Options DENY
        Referrer-Policy no-referrer
    }

    reverse_proxy porkpie-server:8080
}
Suggested infra/docker/server.Dockerfile
FROM rust:1-bookworm AS builder

WORKDIR /app
COPY . .

RUN cargo build --release -p porkpie-server

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
Required Validation

Run:

docker compose -f infra/compose/docker-compose.yml --env-file infra/compose/.env.example config

Then run the global Rust validation command:

cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
Acceptance Criteria
infra/caddy/ is no longer empty.
infra/compose/ is no longer empty.
infra/docker/ is no longer empty.
Compose config renders successfully.
No real secrets are committed.
No public default API key is accepted as production config.
Server image builds or failures are documented with exact crate/binary name mismatch.
README explains self-host setup.

## Important caveat

The Dockerfile assumes the server binary/package is called:

```text
porkpie-server

If the actual binary is named differently, the agent must adjust this line:

RUN cargo build --release -p porkpie-server

and this line:

COPY --from=builder /app/target/release/porkpie-server /usr/local/bin/porkpie-server

to match the real package/binary.