# Windows SSH Agent Manual QA

## Verification Status

> **UNVERIFIED — TEMPLATE ONLY**
>
> This document contains the exact commands and expected output format for a real Windows manual QA session. The code compiles and passes CI on `windows-latest`, but the runtime end-to-end handshake (`ssh-add -L`, `ssh -T git@github.com`) has **not been verified on a real Windows machine** yet.
>
> To mark this document verified, run the commands below on a Windows 10/11 machine with Microsoft OpenSSH, paste the real output into the `Result:` blocks, and commit the updated file.

## Prerequisites

- Windows 10/11 with Microsoft OpenSSH
- PowerShell
- Porkpie built or available on Windows
- Git for Windows (optional, for Git-specific QA)
- An unlocked Porkpie vault containing at least one valid Ed25519 SSH key item

---

## QA Checklist

### 1. Locate the Microsoft OpenSSH binary

```powershell
where.exe ssh
```

**Result (template):**
```text
C:\Windows\System32\OpenSSH\ssh.exe
```

### 2. Verify OpenSSH version

```powershell
ssh -V
```

**Result (template):**
```text
OpenSSH_for_Windows_9.5p1, LibreSSL 3.8.2
```

### 3. Check built-in agent service status

```powershell
Get-Service ssh-agent
```

**Result (template):**
```text
Status   Name               DisplayName
------   ----               -----------
Stopped  ssh-agent          OpenSSH Authentication Agent
```

### 4. Build / verify Porkpie on Windows

```powershell
cargo check --target x86_64-pc-windows-msvc -p porkpie-agent
cargo test --target x86_64-pc-windows-msvc -p porkpie-agent
cargo check --target x86_64-pc-windows-msvc -p porkpie-cli
cargo test --target x86_64-pc-windows-msvc -p porkpie-cli
```

**Result (template):**
```text
All checks pass, all tests pass.
```

### 5. Start the Porkpie agent (requires unlocked vault)

```powershell
cargo run -p porkpie-cli -- ssh-agent start
```

**Result (template):**
```text
Porkpie SSH agent will bind the named pipe:
  \\.\pipe\openssh-ssh-agent

This replaces the Windows OpenSSH Authentication Agent service.
If the built-in service is running, disable it first:
  Stop-Service ssh-agent
  Set-Service ssh-agent -StartupType Disabled

Then test with: ssh -T git@github.com

Press Ctrl+C to stop the agent.
```

### 6. List loaded keys

```powershell
ssh-add -L
```

**Result (template):**
```text
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG... my-vault-key
```

### 7. Test GitHub authentication

```powershell
ssh -T git@github.com
```

**Result (template):**
```text
Hi <username>! You've successfully authenticated, but GitHub does not provide shell access.
```

### 8. Check agent status

```powershell
cargo run -p porkpie-cli -- ssh-agent status
```

**Result (template):**
```text
Agent pipe is active (connect probe succeeded): \\.\pipe\openssh-ssh-agent
Try: ssh-add -L
Windows OpenSSH Authentication Agent service is not running.
```

---

## If Built-in Agent Service Conflicts

```powershell
Stop-Service ssh-agent
Set-Service ssh-agent -StartupType Disabled
```

---

## Git for Windows Configuration

Force Git for Windows to use Microsoft OpenSSH instead of its bundled ssh:

```powershell
git config --global core.sshCommand "C:/Windows/System32/OpenSSH/ssh.exe"
```

Verify:

```powershell
git config --global core.sshCommand
```

**Result (template):**
```text
C:/Windows/System32/OpenSSH/ssh.exe
```

Test Git over SSH:

```powershell
git ls-remote git@github.com:<username>/<repo>.git
```

**Result (template):**
```text
<ref>        HEAD
<ref>        refs/heads/main
```

---

## Notes

- The agent runs in foreground mode. Press Ctrl+C to stop it.
- `porkpie ssh-agent stop` is not available because the agent has no background/service mode yet.
- If the agent is not running, `ssh-add -L` will report `Error connecting to agent: No such file or directory`.
- The `status` command performs a real connect probe: it attempts to open the client side of the named pipe. If a server is listening, the open succeeds.
- Do not claim Bitwarden/1Password parity until the above checklist is verified with real output on a Windows machine.
