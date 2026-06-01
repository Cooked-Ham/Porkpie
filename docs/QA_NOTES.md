# QA Notes

## Phase 08: Doc Truth

- README.md:
  - `porkpie list` updated to `porkpie item list`
  - `porkpie read pie://...` added to examples
  - `porkpie write` does not mention `--stdin` or `--prompt` in the quick-start (they are documented in `porkpie write --help`)
- SECURITY_INVARIANTS.md:
  - Server item ID integrity is now enforced via composite PK `(vault_id, id)` on both server and client schemas
  - Sync conflict default is now `preserve-conflict` instead of `last-write-wins`
- DATA_MODEL.md:
  - Items schema updated to composite PK `(vault_id, id)` with `sync_revision` column
  - Index updated to `idx_items_vault_revision`
- STATUS.md:
  - Added environment setup section
  - Web storage updated to reflect encrypted localStorage (not unavailable)
  - Security audit section updated to reflect placeholder rejection, CORS hardening, and strategy safety
