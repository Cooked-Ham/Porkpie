# Phase 08: Make Sync Real

## Binding

You are bound to Phase 08 only.

Your job is to upgrade sync from partial encrypted push to a real bidirectional prototype sync. Do not add team sharing. Do not add passkeys. Do not fake conflict handling by overwriting things and calling it “resolved.” That is not sync, that is data loss with a progress bar.

## Goal

Implement a two-profile encrypted sync flow with conflict preservation.

## Required Context

Read first:

- `crates/porkpie-sync/**`
- `crates/porkpie-api/**`
- `crates/porkpie-cli/**`
- `crates/porkpie-store/**`
- `crates/porkpie-core/**`
- `docs/SYNC_PROTOCOL.md`

## Allowed Areas

- `crates/porkpie-sync/**`
- `crates/porkpie-api/**`
- `crates/porkpie-cli/**`
- `crates/porkpie-store/**`
- `crates/porkpie-core/**` only if needed for applying revisions
- sync docs and tests

## Forbidden

- No plaintext item sync payloads.
- No server-side vault decryption.
- No request-body logs containing item data.
- No silently overwritten conflicts.
- No push-only sync presented as real sync.
- No server metadata that exposes item titles/usernames/URLs unless explicitly documented and justified.

## Tasks

1. Add encrypted vault registration if missing.
2. Push local encrypted revisions.
3. Pull remote encrypted revisions.
4. Apply remote encrypted revisions locally.
5. Maintain local sync cursor.
6. Preserve conflicts instead of overwriting.
7. Ensure server never receives plaintext item fields.
8. Ensure server cannot decrypt vault items.
9. Ensure API logs do not expose payload contents.
10. Update `docs/SYNC_PROTOCOL.md`.

## Required Integration Test

1. Profile A creates vault and item.
2. Profile A syncs.
3. Profile B syncs.
4. Profile B sees item after unlock.
5. Profile A and B both edit same item offline.
6. Both sync.
7. Conflict is preserved.
8. Server DB does not contain fixture plaintext.

## Acceptance Criteria

- Bidirectional sync works.
- Conflict behavior exists.
- Server remains zero-knowledge.
- Server DB plaintext scan passes.
- Global validation command passes.

## Required Final Report

Include:

- Summary
- Files changed
- Commands run
- Test results
- Sync protocol behavior
- Conflict strategy
- Zero-knowledge guarantees
- Remaining sync risks
- Next recommended phase
