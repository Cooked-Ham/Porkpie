# Windows SSH Agent Manual QA

## Prerequisites

- Windows 10/11 with Microsoft OpenSSH
- PowerShell
- Porkpie built or available on Windows

## Checklist

```powershell
# 1. Verify OpenSSH version
ssh -V

# 2. Check built-in agent service status
Get-Service ssh-agent

# 3. Build / verify Porkpie on Windows
cargo check --target x86_64-pc-windows-msvc -p porkpie-agent
cargo test --target x86_64-pc-windows-msvc -p porkpie-agent

# 4. Start the Porkpie agent (requires unlocked vault)
cargo run -p porkpie-cli -- ssh-agent start

# 5. List loaded keys
ssh-add -L

# 6. Test GitHub authentication
ssh -T git@github.com

# 7. Check agent status
cargo run -p porkpie-cli -- ssh-agent status
```

## If Built-in Agent Service Conflicts

```powershell
Stop-Service ssh-agent
Set-Service ssh-agent -StartupType Disabled
```

## Git for Windows Configuration

```powershell
git config --global core.sshCommand "C:/Windows/System32/OpenSSH/ssh.exe"
```

## Expected Results

- `ssh-add -L` should print the public key comment loaded from the vault.
- `ssh -T git@github.com` should return `Hi <username>! You've successfully authenticated...`.
- `porkpie ssh-agent status` should report the pipe as active.

## Notes

- The agent runs in foreground mode.  Press Ctrl+C to stop it.
- If the agent is not running, `ssh-add -L` will report `Error connecting to agent: No such file or directory`.
