# Phase 02: Debug Redaction and Secret State Hardening

## Binding

You are bound to Phase 02 only.

Your job is to remove remaining raw Debug leaks and harden secret-bearing UI/app state. Do not change encryption format. Do not change sync. Do not add OS keychain support here.

## Goal

Ensure no secret-bearing structs leak through `Debug`, especially `RecoveryKit` and generated password state.

## Source Findings

- `RecoveryKit` derives `Debug` and contains `local_secret_key`.
- `PasswordGeneratorState` derives `Debug` and contains `generated_password`.
- Item types have custom redacted `Debug`, but these remaining structs are not fixed.

## Allowed Files

- `crates/porkpie-types/src/secret_key.rs`
- `crates/porkpie-ui/src/state.rs`
- `crates/porkpie-types/tests/**`
- `crates/porkpie-ui/tests/**`
- `docs/SECURITY_INVARIANTS.md`
- `docs/STATUS.md`
- `docs/AUDIT_REPORT.md`

## Forbidden

- Do not remove `Serialize`/`Deserialize` behavior needed for recovery kits.
- Do not stop generating recovery kits.
- Do not print the local secret key in test failure messages.
- Do not delete debug tests.
- Do not add fake redaction that still contains the secret.

## Tasks

### 1. Redact `RecoveryKit` Debug

Remove raw `Debug` derive from `RecoveryKit`.

Implement manual redacted `Debug`:

```rust
impl std::fmt::Debug for RecoveryKit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoveryKit")
            .field("vault_id", &self.vault_id)
            .field("local_secret_key", &"[redacted]")
            .field("created_at", &self.created_at)
            .field("instructions", &self.instructions)
            .field("warning", &self.warning)
            .finish()
    }
}
```

### 2. Add `RecoveryKit` redaction tests

Test that:

- `serde_json::to_string(&recovery_kit)` still includes `local_secret_key` because that is the point of the recovery kit.
- `format!("{:?}", recovery_kit)` does **not** include the local secret key.
- Debug output includes `[redacted]`.

### 3. Redact `PasswordGeneratorState` Debug

Do not derive raw `Debug` for generated passwords.

Either:

- remove `Debug`, or
- implement manual Debug that redacts `generated_password`.

Preferred manual behavior:

```text
PasswordGeneratorState { length: 24, uppercase: true, ..., generated_password: "[redacted]" }
```

### 4. Add generated-password redaction test

Test that a known generated password does not appear in Debug output.

### 5. Inspect for other raw Debug leaks

Search production code for:

```text
derive(Debug
generated_password
local_secret_key
RecoveryKit
SecretKey
PlaintextExport
```

Any struct containing a secret must either not implement Debug or must redact.

## Acceptance Criteria

- `RecoveryKit` Debug does not expose local secret key.
- `PasswordGeneratorState` Debug does not expose generated password.
- Tests prove redaction.
- JSON recovery kit export still contains the local secret key.
- Global validation command passes.

## Required Validation

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```
