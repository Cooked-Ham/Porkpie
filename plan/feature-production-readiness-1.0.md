---
goal: Achieve production readiness and complete sell-worthy features
version: 1.0
date_created: 2026-05-31
last_updated: 2026-05-31
owner: AI Assistant
status: 'Planned'
tags: [feature, architecture, product-readiness]
---

# Introduction

![Status: Planned](https://img.shields.io/badge/status-Planned-blue)

This plan outlines the steps required to take the Porkpie password manager from its current foundational prototype state to a fully "sell-worthy" production product. The current codebase has implemented core backend requirements including local SQLite vaults, CLI, HTTP Sync API, and zero-knowledge cryptography. The Dioxus UI is a static mockup without real interactivity. Desktop and web app shells are empty stubs. The `pie://` URI scheme is not yet implemented. Before commercial viability, the project requires honest completion of all foundation gates, deeper OS integration, browser extensions, expanded importers, and recovery features.

## 1. Requirements & Constraints

- **REQ-001**: Implement browser extension with autofill capabilities.
- **REQ-002**: Implement full desktop shell integration (system tray, hotkeys, clipboard auto-clearing).
- **REQ-003**: Implement native importers for major competitors (1Password, Bitwarden, LastPass).
- **REQ-004**: Implement plaintext export capabilities behind an explicit dangerous flag.
- **REQ-005**: Add recovery workflows (e.g., recovery codes) for lost master passwords.
- **REQ-006**: Create visual conflict resolution and duplicate management tools.
- **REQ-007**: Implement hardware-backed key support (FIDO2/WebAuthn/YubiKey).
- **CON-001**: Must maintain zero-knowledge encryption invariants.
- **SEC-001**: Third-party security audit and penetration testing must be scheduled before commercial launch.

## 2. Implementation Steps

### Implementation Phase 1: Interoperability & Data Portability

- GOAL-001: Implement advanced importing and exporting strategies so users can easily migrate to and from Porkpie.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-001 | Implement 1Password native JSON importer in `porkpie-import` | |  |
| TASK-002 | Implement Bitwarden native JSON/CSV importer in `porkpie-import` | |  |
| TASK-003 | Implement LastPass CSV importer in `porkpie-import` | |  |
| TASK-004 | Add plaintext export support to `porkpie-cli` behind an `--unsafe-export-plaintext` flag | |  |

### Implementation Phase 2: OS Integration & Browser Support

- GOAL-002: Seamlessly integrate the app with desktop environments and web browsers for auto-filling and background execution.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-005 | scaffolding browser extension structure under `apps/extension/` | |  |
| TASK-006 | Integrate browser extension with local desktop app via Native Messaging / local API | |  |
| TASK-007 | Add system tray support and background running to `apps/desktop` | |  |
| TASK-008 | Implement clipboard timed auto-clearing after copy actions in UI | |  |
| TASK-009 | Add global OS hotkeys for global auto-type functionality | |  |

### Implementation Phase 3: Recovery & Advanced Features

- GOAL-003: Implement account recovery options, conflict resolution tooling, and hardware security.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-010 | Implement recovery code generation during vault setup in `porkpie-core` | |  |
| TASK-011 | Implement master password reset using verified recovery codes | |  |
| TASK-012 | Build visual diff UI for sync conflict resolution in `porkpie-ui` | |  |
| TASK-013 | Add FIDO2 / WebAuthn hardware key support for 2FA/vault unlocking | |  |
| TASK-014 | Develop initial UI and endpoints for team sharing / Public-Key Recipient wrapping | |  |

## 3. Alternatives

- **ALT-001**: Relying strictly on CSV for all competitors instead of custom native importers. Rejected because competitor CSV exports often lose granular data attachments compared to their JSON exports.
- **ALT-002**: Skipping desktop app integration in favor of a purely web/browser-based app. Rejected because local native performance and strict offline availability are key selling points for this product.

## 4. Dependencies

- **DEP-001**: External libraries for FIDO2/WebAuthn integration (e.g., `webauthn-rs`).
- **DEP-002**: Rust crates for system tray and global hotkeys (`tray-icon`, `global-hotkey`).

## 5. Files

- **FILE-001**: `crates/porkpie-import/src/lib.rs` (expanding importer logic)
- **FILE-002**: `crates/porkpie-cli/src/main.rs` (adding the unsafe export flag)
- **FILE-003**: `apps/desktop/src/main.rs` (adding system tray and hotkeys)
- **FILE-004**: `apps/extension/` (new directory for browser extension)

## 6. Testing

- **TEST-001**: Verify that 1Password and Bitwarden mock exports are correctly parsed into `porkpie-types::Item` structures.
- **TEST-002**: Verify that recovery codes correctly bypass a forgotten master password locally.
- **TEST-003**: Test clipboard auto-clearing functionality across Windows, macOS, and Linux runners.

## 7. Risks & Assumptions

- **RISK-001**: Cross-platform system tray and global hotkey API inconsistencies might cause desktop app regressions.
- **ASSUMPTION-001**: Browser native messaging will be stable enough to bridge the extension and the local Rust desktop binary.

## 8. Related Specifications / Further Reading

- [ROADMAP.md](../docs/ROADMAP.md)
- [PRODUCT_SPEC.md](../docs/PRODUCT_SPEC.md)
