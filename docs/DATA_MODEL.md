# Data Model

The data model delineates how Porkpie vaults represent items internally, strictly separating unencrypted business logic representations mapping onto fully encrypted serialization payloads.

## Vault

| Field | Type | Notes |
|-------|------|-------|
| id | VaultId (UUID) | Unique identifier globally mapping vault states |
| created_at | Timestamp | Unix milliseconds epoch denoting initialization |
| master_key_wrapped | Vec<u8> | Encrypted validation key; never decrypted by store |
| sync_revision | u64 | Integer tracking remote replication cycles |

## Item (Base)

| Field | Type | Notes |
|-------|------|-------|
| id | ItemId (UUID) | Distinct unique string mapping to individual elements |
| vault_id | VaultId (UUID) | Strict Foreign Key joining elements directly to native vaults |
| item_type | ItemType | Enum mapping (Login, APIKey, Server, etc.) |
| ciphertext | Vec<u8> | Serialized JSON containing all distinct internal models wrapped within XChaCha20Poly1305 boundaries |
| created_at | Timestamp | Initial creation sequence |
| updated_at | Timestamp | Synchronization resolution flag tracking latest edits |

## Item Types

### 1. Login
```json
{
  "username": "user@example.com",
  "password": "secret_password_123",
  "url": "https://dashboard.example.com",
  "notes": "Work domain account details."
}
```

### 2. APIKey
```json
{
  "name": "Production Database Key",
  "key": "pk_live_d81347ba0182",
  "provider": "Stripe/AWS"
}
```

### 3. SSHKey
```json
{
  "name": "Production Box A",
  "public_key": "ssh-ed25519 AAAAC3...",
  "private_key": "-----BEGIN OPENSSH PRIVATE KEY-----...",
  "passphrase": "super_secret_ssh_phrase"
}
```

### 4. SecureNote
```json
{
  "title": "Crypto Recovery Backup",
  "content": "Word1 Word2 Word3 Word4... WordN"
}
```

### 5. Server
```json
{
  "hostname": "192.168.1.100",
  "port": 22,
  "username": "admin",
  "password": "server_password"
}
```

### 6. Database
```json
{
  "engine": "PostgreSQL",
  "host": "db.internal.network",
  "port": 5432,
  "username": "postgres_admin",
  "password": "database_password",
  "database": "primary_schema"
}
```

### 7. Identity
```json
{
  "name": "John Doe",
  "email": "john.doe@example.com",
  "phone": "+1-555-555-1234",
  "address": "123 Main Street, Appt 4B, City, Country"
}
```

### 8. SoftwareLicense
```json
{
  "product": "Professional Utility Software v5",
  "key": "XXXX-AAAA-BBBB-CCCC",
  "version": "5.1.0",
  "expiry": "2027-10-15T00:00:00Z"
}
```

### 9. RecoveryCodes
```json
{
  "list_of_codes": [
    "A81729B1",
    "J18A891K",
    "M18S8UA1"
  ]
}
```

### 10. Custom
```json
{
  "key_value_pairs": {
    "Vehicle License Plate": "ABC-1234",
    "Insurance Router": "XYZ-91823"
  }
}
```

## Relationships & Constraints
- Maximum lengths apply heavily across item type domains. ciphertext is explicitly limited to bounds. Max lengths enforce secure buffer padding lengths.
- All relationships are purely structurally limited. Items inherently isolate vertically; no secondary linking mappings exist between unique Items.
- Required fields like the ciphertext and distinct UUID fields are strictly non-nullable under standard SQLite rules constraints.

## SQLite Schema Outline

```sql
CREATE TABLE IF NOT EXISTS vaults (
    id TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER NOT NULL,
    master_key_wrapped BLOB NOT NULL,
    sync_revision INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY NOT NULL,
    vault_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    ciphertext BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(vault_id) REFERENCES vaults(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_items_vault_id ON items(vault_id);
```
