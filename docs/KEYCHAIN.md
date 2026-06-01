# OS Keychain Platform Matrix

This document defines the keychain behavior for each platform.

## Platform Matrix

| Platform | Backend | Supported | Validation |
|---|---|---:|---|
| Windows | Credential Manager (keyring) | yes | `porkpie keychain test` |
| macOS | Keychain (keyring) | yes | `porkpie keychain test` |
| Linux desktop | Secret Service / libsecret (keyring) | yes | `porkpie keychain test` |
| Linux headless | none by default | no | `porkpie keychain status` reports unavailable |
| Web/WASM | browser storage only | no | `porkpie keychain status` reports unavailable |

## Commands

```bash
# Show keychain backend and availability
porkpie keychain status

# Test keychain storage by writing and reading a test secret
porkpie keychain test

# Delete the local secret key for a vault from the keychain
porkpie keychain forget <vault-id>
```

## Behavior Rules

1. **No silent weak fallback**: If the keychain is unavailable, the CLI prints a warning and does not store the secret key. The user must provide the secret key on each unlock.
2. **No local secret key in session file**: The session file (`.porkpie-session.json`) stores only the vault ID and lock state. The secret key is stored exclusively in the OS keychain.
3. **Explicit no-store mode**: If the keychain is unavailable, the CLI commands `init`, `unlock`, and `recovery restore` print a warning and continue without storing the secret key.
4. **Fake backend tests**: The `FakeKeychain` in-memory backend is used for unit tests. All keychain operations are tested with the fake backend before any OS keychain integration.

## Trust Model

- The local secret key is a high-value target. It is stored in the OS keychain because the OS keychain is designed to protect secrets from other applications and from offline attacks (e.g., file-system theft).
- The session file does not contain the secret key. If the session file is stolen, the attacker learns the vault ID but not the secret key.
- The keychain is not a substitute for a strong master password. The vault key is derived from both the master password and the local secret key.

## Limitations

- **Encrypted OpenSSH keys**: The SSH agent does not support encrypted OpenSSH private keys. The user must decrypt the key before storing it in the vault, or use a raw Ed25519 seed.
- **Windows SSH agent**: The SSH agent uses Unix domain sockets and is not supported on Windows. Windows users can still use the `ssh public-key` command and raw seed format.
- **Headless Linux**: The `keyring` crate requires a Secret Service D-Bus session. Headless servers without a desktop session may not have keychain support. In this case, the secret key must be provided manually on each unlock.
