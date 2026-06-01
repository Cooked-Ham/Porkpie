# Phase 10: Harden API

## Binding

You are bound to Phase 10 only.

Your job is to make the API resistant to unsafe payloads and auth mistakes. Do not add product features. Do not add UI. Do not make the server smarter by decrypting things. A zero-knowledge server that decrypts is just a regular breach with branding.

## Goal

Reject plaintext item-shaped payloads, harden auth paths, and keep the server zero-knowledge.

## Required Context

Read first:

- `crates/porkpie-api/**`
- `crates/porkpie-sync/**`
- `crates/porkpie-types/**`
- `docs/SYNC_PROTOCOL.md`
- `docs/SECURITY_INVARIANTS.md`

## Allowed Areas

- `crates/porkpie-api/**`
- `crates/porkpie-sync/**`
- tests/docs related to API behavior
- config handling if auth config needs correction

## Forbidden

- No server-side vault decryption.
- No decrypt endpoint.
- No plaintext item payload acceptance.
- No request-body logging.
- No casual encrypted blob logging unless justified.
- No default public auth key.
- No storing raw API keys.

## Tasks

1. Reject plaintext item-shaped payloads.
2. Add tests for payloads containing:
   - username
   - password
   - private_key
   - api_key
   - totp
   - notes
3. Add auth tests:
   - missing auth
   - wrong auth
   - revoked key
   - missing required API key config
4. Ensure request bodies are not logged.
5. Ensure encrypted blobs are not casually logged.
6. Ensure there is no decrypt endpoint.
7. Ensure server-side code cannot decrypt vault items.
8. Update API/security docs as needed.

## Acceptance Criteria

- Plaintext item payloads are rejected.
- Auth failures behave correctly.
- Missing API key config fails safely.
- Logs do not expose payloads.
- Server remains zero-knowledge.
- Global validation command passes.

## Required Final Report

Include:

- Summary
- Files changed
- Commands run
- Test results
- Payload rejection behavior
- Auth behavior
- Logging behavior
- Remaining API risks
- Next recommended phase
