# Crypto Format

Porkpie uses Argon2id for key derivation and XChaCha20Poly1305 for authenticated encryption with associated data (AEAD).

## Key Derivation Model

```text
master password + local secret key + salt -> unlock key
unlock key unwraps vault key
vault key decrypts items
```

### Unlock Key Derivation

The unlock key is derived using Argon2id with the concatenation of the master password bytes and the local secret key bytes as the password input, and a random 32-byte salt:

```text
unlock_key = Argon2id(password || secret_key, salt)
```

- `password`: UTF-8 bytes of the user's master password.
- `secret_key`: 32 random bytes generated during vault creation, stored locally only.
- `salt`: 32 random bytes stored in vault metadata.
- Default Argon2id parameters: time_cost=2, mem_cost=19456 KiB, parallelism=1.

The master password alone cannot derive the correct unlock key. Both the password and the local secret key are required.

## Local Secret Key

- Generated as 32 random bytes during vault creation using the OS CSPRNG.
- Stored locally on the user's machine (session file).
- Never sent to the sync server.
- Represented in hex format in the recovery kit.
- Required alongside the master password to unlock the vault.

## Vault Metadata

- `salt`: 32 random bytes used with Argon2id.
- `master_key_wrapped`: vault key encrypted with the unlock key using XChaCha20Poly1305.
- `sync_revision`: monotonically increasing vault revision.

The master password and local secret key are never stored. Unlock derives a transient unlock key, unwraps the vault key, and drops the password material.

## Item Ciphertext

Each encrypted item is stored as:

```text
24-byte XChaCha20 nonce || ciphertext || 16-byte authentication tag
```

The encrypted plaintext is the serde JSON form of `porkpie_core::Item`.

### Associated Data (AAD) Binding

Every item encryption binds the following metadata as AEAD associated data:

```text
"porkpie-v" || schema_version || "|" || vault_id || "|" || item_id || "|" || item_type
```

- `schema_version`: single byte, currently `0x01`.
- `vault_id`: UUID string of the vault.
- `item_id`: UUID string of the item.
- `item_type`: type name string (e.g., "Login", "APIKey", "SSHKey").

Decryption fails if any associated data field does not match. This prevents:
- Moving ciphertext between vaults (wrong vault ID).
- Swapping item IDs (wrong item ID).
- Changing item types without re-encryption (wrong item type).
- Undetected schema migration (wrong schema version).

### Payload AAD

Backup payloads use a simpler AAD:

```text
"porkpie-payload-v" || schema_version || "|" || vault_id
```

## Recovery Kit

Generated during vault creation, the recovery kit contains:

- `vault_id`: the vault's unique identifier.
- `local_secret_key`: hex-encoded 32-byte secret key.
- `created_at`: Unix millisecond timestamp.
- `instructions`: human-readable recovery steps.
- `warning`: explicit warning about losing recovery material.

The recovery kit does NOT contain:
- The master password.
- Any plaintext vault contents.
- Any decrypted items.

The recovery kit is saved as a JSON file (`porkpie-recovery-kit-{vault_id}.json`) during `porkpie init`.

## Backup Files

Encrypted backups are JSON files with `.json.enc` extension. The file contains encrypted vault metadata and encrypted item rows; it does not contain decrypted item fields or the master password. Import validates the master password and local secret key by decrypting item blobs before merge.
