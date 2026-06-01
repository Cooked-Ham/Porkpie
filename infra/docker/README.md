# Porkpie Server Dockerfile

Multi-stage build for the Porkpie sync server.

## Build

```bash
docker build -f infra/docker/server.Dockerfile -t porkpie-server:latest .
```

## Context

The Dockerfile must be built from the **workspace root** so it can access all crates:

```bash
docker build -f infra/docker/server.Dockerfile -t porkpie-server:latest .
```

## Stages

1. **Builder** (`rust:1-bookworm`) — Compiles the `porkpie-api` crate which defines the `porkpie-server` binary.
2. **Runtime** (`debian:bookworm-slim`) — Minimal image with the compiled binary, `ca-certificates`, and `sqlite3`.

## Notes

- The server binary is produced by the `porkpie-api` crate, not the `porkpie-server` package.
- The binary is copied to `/usr/local/bin/porkpie-server`.
- Runs as the unprivileged `porkpie` user.
- Persists data to `/data`.
