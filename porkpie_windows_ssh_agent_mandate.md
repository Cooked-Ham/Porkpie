# Porkpie Windows SSH Agent Implementation Mandate

Generated: 2026-06-01T16:22:15+00:00

Porkpie must support Windows SSH agent workflows.

Reason: 1Password and Bitwarden support Windows SSH agent usage. Porkpie is aimed at developers and homelab users. A password/secrets manager that claims SSH-agent usefulness but punts on Windows is not acceptable for this product.

This is not a "future limitation." This is an implementation task.

---

## Source Reality

- 1Password documents Windows compatibility for Microsoft OpenSSH, Git for Windows, GitHub Desktop, GitKraken, JetBrains IDEs, Sourcetree, Sublime Merge, Tower, and Visual Studio Code.
- Bitwarden documents Windows SSH Agent setup and tells users to disable the built-in Windows OpenSSH Authentication Agent service before enabling Bitwarden’s agent.
- Porkpie currently documents Windows SSH agent as unsupported. That must change.

---

## Global Binding Rules

Repository: `Cooked-Ham/Porkpie`

Hard rules:

1. Do not remove Unix SSH agent support.
2. Do not weaken vault crypto.
3. Do not write private keys to disk.
4. Do not log private keys.
5. Do not print private keys.
6. Do not fake Windows support by only compiling on Windows.
7. Do not mark Windows SSH support complete until `ssh-add -L` and `ssh -T git@github.com` work with Microsoft OpenSSH on Windows.
8. Do not introduce Electron/React/TypeScript/Vite.
9. Keep Rust-first architecture.
10. Add tests and manual QA docs.

Required validation:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

Windows validation must also pass on an actual Windows environment:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

Manual Windows SSH validation:

```powershell
ssh -V
ssh-add -L
ssh -T git@github.com
```

---

# Goal

Implement a real Windows-compatible Porkpie SSH agent.

The acceptable end state:

```text
Porkpie SSH agent supports:
- Unix domain sockets on macOS/Linux.
- Windows OpenSSH-compatible named pipe transport.
- Standard unencrypted OpenSSH Ed25519 private keys.
- Raw Ed25519 seed format as advanced fallback.
- `ssh-add -L` key listing.
- `ssh -T git@github.com` authentication.
```

Encrypted OpenSSH private keys may remain unsupported only if clearly documented, because the vault itself encrypts at rest. But standard unencrypted OpenSSH private key import must work.

---

# Phase 01: Confirm Windows OpenSSH Transport

## Goal

Determine and document the exact Windows transport Porkpie must bind.

## Expected Direction

Windows OpenSSH uses an agent service/pipe model. The built-in Windows OpenSSH Authentication Agent service may occupy the default pipe. Bitwarden’s docs tell users to disable the built-in Windows OpenSSH Authentication Agent service before enabling Bitwarden SSH Agent, so Porkpie likely needs to either:

1. bind the default OpenSSH agent pipe after disabling the service, or
2. bind a Porkpie-specific pipe and configure Windows OpenSSH/Git to use it, if supported.

Do not guess. Verify.

## Tasks

1. On Windows, inspect current OpenSSH behavior:
   - installed `ssh.exe` path,
   - `ssh -V`,
   - whether OpenSSH Authentication Agent service is running,
   - default named pipe used by the service,
   - whether `IdentityAgent` supports named pipe paths on Windows,
   - whether `SSH_AUTH_SOCK` is honored by Microsoft OpenSSH on Windows.

2. Document the result in:

```text
docs/SSH_AGENT_WINDOWS.md
```

3. Pick the supported transport strategy.

## Acceptance Criteria

- Docs state the exact Windows transport.
- Docs say whether Porkpie uses:
  - default OpenSSH pipe,
  - custom Porkpie pipe,
  - IdentityAgent,
  - SSH_AUTH_SOCK,
  - service replacement.
- No speculative language remains.

---

# Phase 02: Implement Windows Named Pipe Agent Transport

## Goal

Add Windows named-pipe support to `porkpie-agent`.

## Target Files

Likely files:

```text
crates/porkpie-agent/src/agent.rs
crates/porkpie-agent/src/lib.rs
crates/porkpie-cli/src/commands/ssh.rs
crates/porkpie-cli/src/lib.rs
docs/SSH_AGENT_WINDOWS.md
docs/KEYCHAIN.md
README.md
STATUS.md
```

## Implementation Direction

Add platform-specific functions:

```rust
#[cfg(unix)]
pub async fn run_unix_socket(...)

#[cfg(windows)]
pub async fn run_windows_named_pipe(...)
```

Suggested API:

```rust
#[cfg(windows)]
pub async fn run_windows_named_pipe(
    pipe_name: &str,
    agent: Arc<Agent>,
) -> Result<(), AgentError>
```

Use Windows named pipe APIs through a maintained Rust path. Possible options:

- `tokio::net::windows::named_pipe`
- `windows` crate with Win32 named pipe APIs
- another maintained crate if clearly justified

Named pipe handling must:

- accept multiple sequential client connections,
- process OpenSSH agent protocol framing exactly like Unix socket path,
- handle request identities,
- handle sign request,
- return failure for unsupported operations,
- avoid leaking private keys,
- shut down cleanly.

## Acceptance Criteria

- Windows build compiles.
- Pipe server accepts client connections.
- Protocol handler is shared with Unix path.
- No key material is logged.
- Unit tests cover framing logic independent of OS.
- Windows integration test or manual QA proves actual OpenSSH client compatibility.

---

# Phase 03: CLI UX for Windows Agent

## Goal

Add clear commands that work on Windows.

## Required Commands

```powershell
porkpie ssh-agent start
porkpie ssh-agent status
porkpie ssh-agent stop
porkpie ssh-agent env
```

Required behavior:

### `start`

- Starts the Windows named pipe agent.
- Loads keys from the currently unlocked vault.
- If the built-in OpenSSH Authentication Agent service conflicts, prints exact instructions.
- Does not claim success unless the pipe is bound and ready.

### `status`

- Reports whether the Porkpie agent pipe exists.
- Reports whether the built-in OpenSSH Authentication Agent service appears to conflict, if detectable.
- Reports loaded public keys only, never private key material.

### `stop`

- Stops the Porkpie agent process or removes/marks the pipe unavailable.
- If running foreground-only at first, say so honestly and make `stop` a no-op with a clear message.
- Final target should support background operation.

### `env`

- Prints the PowerShell commands needed to configure the shell, if applicable.
- If using the default OpenSSH pipe, explain that no env var is needed but the built-in service must be disabled.
- If using a custom pipe, output the required `IdentityAgent` / env setup.

## Acceptance Criteria

- PowerShell workflow is documented.
- Commands behave consistently on Windows.
- Unix commands still work.
- `porkpie ssh-agent` without subcommand either starts foreground mode or prints help. No ambiguity.

---

# Phase 04: Windows OpenSSH Service Collision Handling

## Goal

Handle the built-in Windows OpenSSH Authentication Agent service cleanly.

## Required Behavior

If Porkpie must bind the default OpenSSH agent pipe:

1. Detect whether Windows `ssh-agent` service is running.
2. If running, print exact fix:

```powershell
Get-Service ssh-agent
Stop-Service ssh-agent
Set-Service ssh-agent -StartupType Disabled
```

3. Do not attempt to fight the service or silently fail.
4. After disabling service, Porkpie should bind the pipe and become the active agent.

If Porkpie can use a custom named pipe:

1. Document the exact config.
2. Do not require disabling the service unless needed.
3. Provide per-host `.ssh/config` example if applicable.

## Acceptance Criteria

- Clear collision error.
- No silent fallback to built-in ssh-agent.
- No fake success while OpenSSH still talks to the Microsoft agent.
- Manual QA proves OpenSSH is talking to Porkpie.

---

# Phase 05: Windows Manual QA Harness

## Goal

Make Windows validation repeatable.

## Required Manual QA Document

Create:

```text
docs/SSH_AGENT_WINDOWS_QA.md
```

Must include one-line commands only.

Required checklist:

```powershell
where.exe ssh
ssh -V
Get-Service ssh-agent
cargo run -p porkpie-cli -- ssh-agent status
cargo run -p porkpie-cli -- ssh-agent start
ssh-add -L
ssh -T git@github.com
```

If built-in agent must be disabled:

```powershell
Stop-Service ssh-agent
Set-Service ssh-agent -StartupType Disabled
```

If Git for Windows must use Microsoft OpenSSH:

```powershell
git config --global core.sshCommand "C:/Windows/System32/OpenSSH/ssh.exe"
```

If commit signing is supported:

```powershell
git config --global gpg.format ssh
git config --global gpg.ssh.program "C:/Windows/System32/OpenSSH/ssh-keygen.exe"
```

## Required Proof

The agent must report:

- `ssh -V` output,
- `ssh-add -L` output redacted to public key prefix/comment only,
- `ssh -T git@github.com` result,
- whether Windows service was disabled,
- whether Git for Windows was configured,
- exact Porkpie command used.

## Acceptance Criteria

- QA doc exists.
- QA was performed on Windows.
- Results are pasted into `docs/SSH_AGENT_WINDOWS_QA.md`.
- Failure is not accepted as done.

---

# Phase 06: Tests

## Unit Tests

Add transport-independent tests:

- agent request identities response encodes public keys correctly,
- sign request returns valid Ed25519 signature,
- unsupported message returns failure,
- malformed length prefix fails safely,
- private key never appears in debug/log output.

## Windows-Specific Tests

Where possible:

- named pipe server starts,
- named pipe client can connect,
- protocol request roundtrip works,
- multiple sequential requests work,
- pipe conflict is detected.

Feature-gate tests if CI cannot run them locally.

## CI Matrix

Add GitHub Actions Windows job.

Required:

```yaml
runs-on: windows-latest
```

Commands:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

If integration test can run on GitHub Windows runner, include it.

If not, include manual QA docs and mark integration as manual only.

## Acceptance Criteria

- Windows CI exists.
- Cross-platform tests pass.
- Manual Windows OpenSSH QA exists.
- No Windows-only code breaks Linux/macOS.

---

# Phase 07: Documentation and Product Claims

## Required Docs Updates

Update:

```text
README.md
STATUS.md
docs/KEYCHAIN.md
docs/SSH_AGENT_WINDOWS.md
docs/SSH_AGENT_WINDOWS_QA.md
docs/TRUST_GATE.md
docs/AUDIT_REPORT.md
docs/COMPLETION_GATE.md
```

Remove language saying Windows SSH agent is unsupported once it works.

Allowed final language:

```text
Porkpie SSH agent supports Unix domain sockets on macOS/Linux and Windows OpenSSH-compatible named pipe transport on Windows.
```

If encrypted OpenSSH private keys remain unsupported:

```text
Encrypted OpenSSH private keys are not imported directly. Import an unencrypted OpenSSH Ed25519 private key into Porkpie; Porkpie encrypts it at rest inside the vault.
```

## Acceptance Criteria

- Docs match code.
- No stale unsupported Windows claim remains.
- No "complete" claim without Windows QA.
- Trust gate remains honest.

---

# Definition of Done

Windows SSH agent support is done only when all are true:

- Windows named pipe transport implemented.
- Microsoft OpenSSH can list Porkpie-held public keys with `ssh-add -L`.
- Microsoft OpenSSH can authenticate to GitHub with `ssh -T git@github.com`.
- Standard unencrypted OpenSSH Ed25519 private key import works.
- Built-in OpenSSH Authentication Agent service conflict is handled and documented.
- Git for Windows path is documented.
- Windows CI build/test job exists.
- Manual Windows QA results are recorded.
- Unix SSH agent still works.
- No private key is written to disk or logged.
- Docs no longer exclude Windows SSH agent support.

If any of the above cannot be met, do not claim Windows SSH agent support.
