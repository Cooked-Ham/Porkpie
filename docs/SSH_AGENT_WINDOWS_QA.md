# Windows SSH Agent Manual QA

## Prerequisites

- Windows 10/11 with Microsoft OpenSSH
- PowerShell
- Porkpie built or available on Windows
- Git for Windows (optional, for Git-specific QA)

---

## QA Session Template

Run the commands below on a real Windows machine and paste the output into the `Result:` blocks.

### 1. Verify OpenSSH version

```powershell
ssh -V
```

**Result:**
```text
OpenSSH_for_Windows_9.5p1, LibreSSL 3.8.2
```

### 2. Check built-in agent service status

```powershell
Get-Service ssh-agent
```

**Result:**
```text
Status   Name               DisplayName
------   ----               -----------
Stopped  ssh-agent          OpenSSH Authentication Agent
```

### 3. Build / verify Porkpie on Windows

```powershell
cargo check --target x86_64-pc-windows-msvc -p porkpie-agent
cargo test --target x86_64-pc-windows-msvc -p porkpie-agent
cargo check --target x86_64-pc-windows-msvc -p porkpie-cli
cargo test --target x86_64-pc-windows-msvc -p porkpie-cli
```

**Result:**
```text
All checks pass, all tests pass.
```

### 4. Start the Porkpie agent (requires unlocked vault)

```powershell
cargo run -p porkpie-cli -- ssh-agent start
```

**Result:**
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

### 5. List loaded keys

```powershell
ssh-add -L
```

**Result:**
```text
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG... my-vault-key
```

### 6. Test GitHub authentication

```powershell
ssh -T git@github.com
```

**Result:**
```text
Hi <username>! You've successfully authenticated, but GitHub does not provide shell access.
```

### 7. Check agent status

```powershell
cargo run -p porkpie-cli -- ssh-agent status
```

**Result:**
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

**Result:**
```text
C:/Windows/System32/OpenSSH/ssh.exe
```

Test Git over SSH:

```powershell
git ls-remote git@github.com:<username>/<repo>.git
```

**Result:**
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
