# Porkpie Self-Host Deployment

This directory contains the production Docker Compose stack for Porkpie.

## Quick Start

1. Copy `.env.example` to `.env` and edit the values:
   ```bash
   cp .env.example .env
   ```
   - Set `PORKPIE_API_KEY` to a long random secret.
   - Set `PORKPIE_PUBLIC_URL` to your domain.
   - Set `PORKPIE_PUBLIC_HOST` to the same domain (used by Caddy).

2. Start the stack:
   ```bash
   docker compose up -d
   ```

3. Verify:
   ```bash
   docker compose logs -f porkpie-server
   ```

## Services

- `porkpie-server` — The Porkpie sync API server (port 8080 internally).
- `caddy` — Reverse proxy with automatic HTTPS (ports 80 and 443).

## Data Persistence

- `porkpie-data` — SQLite database and server state.
- `caddy-data` — TLS certificates and Caddy state.
- `caddy-config` — Caddy configuration storage.

## Security

- The server will **refuse to start** if `PORKPIE_API_KEY` is missing or set to the placeholder value.
- No real secrets should be committed. `.env` is ignored by git.
- API keys are hashed with SHA-256 before storage.

## Development

Use `docker-compose.dev.yml` for local development without Caddy:

```bash
docker compose -f docker-compose.dev.yml up -d
```

The server will be exposed directly on `http://localhost:8080`.
