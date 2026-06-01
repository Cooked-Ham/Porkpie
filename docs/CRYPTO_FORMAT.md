# Crypto Format

Porkpie uses Argon2id for master-password key derivation and XChaCha20Poly1305 for authenticated encryption.

## Vault Metadata

- `salt`: 32 random bytes used with Argon2id.
- `master_key_wrapped`: encrypted random vault key.
- `sync_revision`: monotonically increasing vault revision.

The master password is never stored. Unlock derives a transient master key, unwraps the vault key, and drops the password material.

## Item Ciphertext

Each encrypted item is stored as:

```text
24-byte XChaCha20 nonce || ciphertext || 16-byte authentication tag
```

The encrypted plaintext is the serde JSON form of `porkpie_core::Item`. Tampering with any byte causes decryption failure through AEAD authentication.

## Backup Files

Encrypted backups are JSON files with `.json.enc` extension. The file contains encrypted vault metadata and encrypted item rows; it does not contain decrypted item fields or the master password. Import validates the master password by decrypting item blobs before merge.
