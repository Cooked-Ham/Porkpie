# Phase 03: CLI Secret Input Hardening

## Binding

You are bound to Phase 03 only.

Your job is to stop showing secrets visibly during interactive CLI entry. Do not redesign the CLI. Do not change `pie://`. Do not change crypto. Do not change UI.

## Goal

Use hidden prompts or safer input flows for secret fields instead of visible `Input<String>` prompts.

## Source Findings

The current CLI uses visible prompts for many secrets through `prompt_string()`:

- login password
- API key
- SSH private key
- SSH passphrase
- database password
- software license key
- recovery codes
- custom secret values

This exposes secrets in terminal scrollback and screen shares, because apparently the terminal needed gossip.

## Allowed Files

- `crates/porkpie-cli/src/interactive.rs`
- `crates/porkpie-cli/src/commands/add.rs`
- `crates/porkpie-cli/src/commands/edit.rs`
- `crates/porkpie-cli/src/commands/write.rs`
- `crates/porkpie-cli/tests/**`
- CLI docs in `README.md` or `docs/**`

## Forbidden

- Do not change command names.
- Do not remove `pie://`.
- Do not print entered secrets.
- Do not add plaintext temp files.
- Do not make private key entry impossible.
- Do not weaken password length checks.

## Tasks

### 1. Add hidden prompt helper

Create a helper:

```rust
fn prompt_secret(prompt: &str, default_present: bool) -> Result<String>
```

Rules:

- Use `dialoguer::Password`.
- Do not show the existing secret as a default.
- During edit, allow blank input to mean “keep existing secret” where appropriate.
- Do not echo secret values.

### 2. Use hidden prompts for secret fields

Replace visible prompts with secret prompts for:

- `LoginSecret.password`
- `APIKeySecret.key`
- `SSHKeySecret.private_key`
- `SSHKeySecret.passphrase`
- `ServerSecret.password`
- `DatabaseSecret.password`
- `SoftwareLicenseSecret.key`
- `RecoveryCodesSecret.codes`
- any `CustomSecret` value

Non-secret labels like username, host, provider, public key, and product may remain visible.

### 3. Add multiline private key support

Private keys are commonly multiline. Add one of these safe flows:

Option A, preferred:

```text
Private key source:
1. Paste interactively
2. Read from file
3. Read from stdin
```

Rules:

- If reading from file, do not log contents.
- If reading from stdin, do not echo.
- Validate that the private key is non-empty.

Option B:

- Keep single-prompt support for now but document the limitation and add a follow-up issue.

### 4. Harden `porkpie write`

`porkpie write <pie-uri> <value>` currently takes the value as a visible CLI argument. Keep it for automation, but add safer alternatives:

```bash
porkpie write <pie-uri> --stdin
porkpie write <pie-uri> --prompt
```

Rules:

- CLI args are visible in shell history and process lists. Docs must warn about this.
- `--prompt` uses hidden input.
- `--stdin` reads from stdin and does not print value.

### 5. Tests

Add tests for CLI parsing:

- `write <uri> --stdin`
- `write <uri> --prompt`
- error if both literal value and `--stdin`/`--prompt` are provided
- help text warns about literal value exposure

Manual tests may be needed for interactive hidden prompt behavior. Document them.

## Acceptance Criteria

- Secret fields are no longer entered through visible prompts.
- `porkpie write` has `--stdin` and `--prompt` safe modes.
- Docs warn that literal CLI values may leak into shell history/process lists.
- Existing CLI commands still parse.
- Global validation command passes.

## Required Validation

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```
