# Phase 07: API CORS and Payload Hardening

## Binding

You are bound to Phase 07 only.

Your job is to reduce browser attack surface and make plaintext-payload rejection honestly described. Do not change crypto. Do not add server-side decryption. Do not add user accounts.

## Goal

Replace permissive CORS with configured origins and clarify that plaintext detection is a heuristic guardrail, not cryptographic proof.

## Source Findings

- API router uses `CorsLayer::permissive()`.
- Plaintext payload rejection checks for JSON-like structures and sensitive field names.
- That heuristic catches obvious mistakes but cannot prove encryption.
- Docs risk implying stronger guarantees than the server can enforce.

## Allowed Files

- `crates/porkpie-api/src/lib.rs`
- `crates/porkpie-api/src/config.rs`
- `crates/porkpie-api/tests/**`
- `infra/compose/.env.example`
- `infra/compose/README.md`
- `docs/SYNC_PROTOCOL.md`
- `docs/SECURITY_INVARIANTS.md`
- `docs/API.md` if present

## Forbidden

- Do not add decrypt endpoints.
- Do not log request bodies.
- Do not accept plaintext item-shaped payloads.
- Do not remove existing plaintext rejection tests.
- Do not make CORS permissive by default.

## Tasks

### 1. Add CORS config

Add optional config:

```env
PORKPIE_CORS_ORIGINS=https://app.porkpie.love,https://sync.porkpie.love
```

Behavior:

- If unset, default to no broad browser origins, or same-origin only.
- If set, parse allowed origins.
- Reject invalid origins at startup.
- Never use permissive CORS by default.

### 2. Update router

Replace `CorsLayer::permissive()` with a configured layer.

Make sure health/status endpoints still work for normal CLI/curl requests.

### 3. Add tests

Tests should cover:

- no CORS env does not create permissive wildcard policy
- configured origin is allowed
- unconfigured origin is not allowed
- invalid CORS origin fails startup/config parsing

### 4. Clarify plaintext heuristic

Update docs to say:

```text
The server rejects obvious plaintext-shaped payloads as a safety guardrail. This is not cryptographic proof that payloads are encrypted. The zero-knowledge guarantee comes from the client-side encryption design and the absence of server-side decryption keys.
```

### 5. Optional envelope improvement

If time allows, add a client-generated encrypted envelope version field to sync payloads. Do not block the phase on this unless already easy.

## Acceptance Criteria

- CORS is no longer permissive by default.
- Allowed origins are configurable.
- Plaintext rejection tests still pass.
- Docs accurately describe heuristic limits.
- Global validation command passes.

## Required Validation

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```
