# Porkpie Threat Model v2.0

**Date:** 2026-06-01
**Status:** Post-implementation review

## Attack Surface

### 1. Local Machine (Primary)
- **Threat:** Malware with user-level access
- **Mitigation:** OS keychain for local secret key; vault key zeroized on lock; clipboard auto-clear
- **Residual risk:** Memory dump during unlocked session; keylogger during password entry

### 2. Sync Server
- **Threat:** Server compromise, insider threat
- **Mitigation:** API key hashing (SHA-256 + constant-time comparison); CORS origin allowlist; encrypted payloads only
- **Residual risk:** Metadata analysis (vault count, item counts, sync frequency); timing side-channels

### 3. Network Transit
- **Threat:** MITM, replay attacks
- **Mitigation:** TLS for sync; encrypted payload means plaintext never transits
- **Residual risk:** Certificate pinning not implemented; downgrade attacks if TLS misconfigured

### 4. Backup/Recovery
- **Threat:** Recovery kit theft, backup file exposure
- **Mitigation:** Recovery kit requires both password AND local secret key; encrypted backups are opaque blobs
- **Residual risk:** User stores recovery kit in plaintext; backup file not encrypted with strong passphrase

### 5. Clipboard
- **Threat:** Clipboard snooping, history retention
- **Mitigation:** `--clear-after` flag; TTY warning on `read`
- **Residual risk:** User copies password and forgets to clear; clipboard manager retains history

## Trust Boundaries

| Component | Trust Level | Notes |
|-----------|-------------|-------|
| User brain | High | Must remember master password |
| OS keychain | High | Platform-provided, DPAPI/macOS Keychain/libsecret |
| Local SQLite | Medium | File-level encryption not implemented |
| Sync server | Low | Never sees decrypted data |
| Recovery kit | High | Offline storage required |

## Security Roadmap

### Completed (2026-06-01)
- [x] Memory zeroization for vault keys, item secrets, generated passwords
- [x] OS keychain storage for local secret key
- [x] Vault key rotation (re-encrypts all items)
- [x] Master password change (re-wraps vault key)
- [x] Argon2id calibration and KDF profiles
- [x] Safer secret output (`--no-newline`, TTY warnings, clipboard clear)
- [x] SSH agent protocol (Ed25519 signing)
- [x] API key rotation endpoints (admin add/revoke)
- [x] Property-based fuzzing (crypto roundtrip, ID parsing, nonce uniqueness)
- [x] Startup self-check for DB path validation
- [x] Composite primary key (vault_id, id) for item integrity
- [x] CORS origin allowlist with URL validation

### Short Term (Next 30 days)
- [ ] Memory zeroization test harness (probe memory after lock)
- [ ] SSH agent socket integration (Unix domain socket / Windows named pipe)
- [ ] Browser extension for web vault autofill
- [ ] System tray / hotkey for quick lock
- [ ] TOTP generation and storage
- [ ] Import from Bitwarden, 1Password, KeePass
- [ ] Audit logging for all vault operations
- [ ] Rate limiting on sync endpoints
- [ ] Certificate pinning for sync server
- [ ] Automatic backup before key rotation

### Medium Term (3-6 months)
- [ ] Hardware security key (YubiKey) support for local secret key
- [ ] Biometric unlock (Touch ID / Windows Hello) as convenience layer
- [ ] Multi-device sync with peer-to-peer option
- [ ] Encrypted file attachments
- [ ] Shared vaults (multi-user) with ACL
- [ ] Formal security audit by external firm
- [ ] FIPS 140-2 compliance evaluation
- [ ] Bug bounty program

### Long Term (12 months)
- [ ] Post-quantum cryptography evaluation (ML-KEM, ML-DSA)
- [ ] Self-hosted sync server with end-to-end encrypted replication
- [ ] Mobile apps (iOS, Android) with secure enclave integration
- [ ] Enterprise features (SSO, SCIM, admin dashboard)
- [ ] Third-party security certification (SOC 2, ISO 27001)

## Assumptions

1. The user's OS keychain is not compromised.
2. The user does not store the recovery kit in plaintext on a networked device.
3. The sync server TLS certificate is valid and not MITM'd.
4. The `porkpie-crypto` dependencies (XChaCha20-Poly1305, Argon2id) are cryptographically sound.
5. The user chooses a strong master password (>= 16 characters).

## Out of Scope

- Protection against nation-state actors with physical access to the running machine
- Protection against hardware-level attacks (cold boot, RAM freezing)
- Protection against compromised operating system kernel
- Protection against side-channel attacks (power analysis, EM emissions)

## Incident Response

If a vault is compromised:
1. Rotate the local secret key immediately (`porkpie vault rotate-local-secret`)
2. Rotate the vault key (`porkpie vault rotate-key`)
3. Revoke all sync server API keys (`curl -X POST /api/v1/admin/api-key/revoke`)
4. Review audit logs for unauthorized access
5. Notify users of shared vaults (if applicable)

## Compliance Notes

- **GDPR:** Vault data is encrypted; server sees only opaque blobs. User is data controller.
- **PCI-DSS:** Not applicable (no payment card data).
- **HIPAA:** Requires BAA and additional logging for healthcare use.
