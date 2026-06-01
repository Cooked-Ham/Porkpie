# Porkpie SSH Agent on Windows

Porkpie supports Windows OpenSSH-compatible named pipe transport.

Microsoft OpenSSH (the one shipping with Windows) uses the named pipe
`\\.\pipe\openssh-ssh-agent` for its authentication agent.  Porkpie binds
this pipe so that `ssh-add`, `ssh`, Git for Windows, and other tools
talk to the Porkpie agent directly.

---

## Requirements

- Windows 10/11 with Microsoft OpenSSH installed.
- The built-in Windows OpenSSH Authentication Agent service must be
disabled (or not running) so that Porkpie can bind the pipe.

---

## Commands

```powershell
# Start the agent (foreground)
porkpie ssh-agent start

# Check whether the agent pipe is active
porkpie ssh-agent status

# Print the environment / configuration needed
porkpie ssh-agent env

# Stop guidance (foreground only on Windows)
porkpie ssh-agent stop
```

---

## Disabling the Built-in Windows OpenSSH Agent

If `porkpie ssh-agent start` reports that the Windows OpenSSH
Authentication Agent service is running, disable it:

```powershell
Stop-Service ssh-agent
Set-Service ssh-agent -StartupType Disabled
```

After disabling, run `porkpie ssh-agent start` again.

---

## Git for Windows

To force Git for Windows to use Microsoft OpenSSH instead of its bundled
OpenSSH:

```powershell
git config --global core.sshCommand "C:/Windows/System32/OpenSSH/ssh.exe"
```

---

## SSH Commit Signing (Optional)

If you want to use SSH commit signing with Porkpie-held keys:

```powershell
git config --global gpg.format ssh
git config --global gpg.ssh.program "C:/Windows/System32/OpenSSH/ssh-keygen.exe"
```

---

## Supported Key Formats

- Standard unencrypted OpenSSH Ed25519 private keys
- Raw 64-character hex Ed25519 seeds
- Base64-encoded 32-byte Ed25519 seeds

Encrypted OpenSSH private keys are not imported directly.  Decrypt the
key first, then import it into Porkpie.  The vault encrypts it at rest.

---

## Limitations

- The agent runs in foreground mode on Windows.  Background service mode
is not yet implemented.
- Only Ed25519 keys are supported.
