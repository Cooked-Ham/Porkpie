# Hostile QA Issue Matrix

| ID | Finding | Severity | Phase |
|---|---|---:|---|
| HQA-001 | Placeholder API key accepted despite docs saying it is rejected | Critical | Phase 01 |
| HQA-002 | Generated secret artifacts not gitignored | High | Phase 01 |
| HQA-003 | RecoveryKit Debug leaks local secret key | High | Phase 02 |
| HQA-004 | PasswordGeneratorState Debug can expose generated password | Medium | Phase 02 |
| HQA-005 | CLI uses visible prompts for secrets | High | Phase 03 |
| HQA-006 | `porkpie write <uri> <value>` exposes secrets in shell history/process args | Medium | Phase 03 |
| HQA-007 | Web storage docs/code contradict | Medium | Phase 04 |
| HQA-008 | WASM localStorage backend exists but app startup does not wire it | Medium | Phase 04 |
| HQA-009 | Server item primary key is global, not vault-scoped | High | Phase 05 |
| HQA-010 | Invalid sync strategy silently falls back to LastWriteWins | Medium | Phase 06 |
| HQA-011 | CORS is permissive by default | Medium | Phase 07 |
| HQA-012 | Plaintext detection is heuristic but docs imply stronger guarantees | Medium | Phase 07 |
| HQA-013 | README CLI commands are stale | Medium | Phase 08 |
| HQA-014 | Audit/status/completion docs contradict each other | Medium | Phase 08 |
| HQA-015 | Missing regression tests for the hostile-QA fixes | Medium | Phase 09 |
| HQA-016 | Final audit required after fixes land | Required | Phase 10 |
