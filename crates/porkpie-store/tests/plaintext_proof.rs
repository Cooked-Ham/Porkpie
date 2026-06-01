use porkpie_core::{Item, LocalSecretKey, Vault};
use porkpie_store::{store_item, store_vault, EncryptedItemData};
use porkpie_types::{
    APIKeySecret, DatabaseSecret, ItemType, LoginSecret, RecoveryCodesSecret, SSHKeySecret,
    SecureNoteSecret,
};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::Executor;
use std::str::FromStr;
use uuid::Uuid;

const FIXTURE_MARKERS: &[&str] = &[
    "DO_NOT_LEAK_PASSWORD_123",
    "DO_NOT_LEAK_API_KEY_123",
    "DO_NOT_LEAK_PRIVATE_KEY_123",
    "DO_NOT_LEAK_NOTE_123",
    "DO_NOT_LEAK_DATABASE_PASSWORD_123",
    "DO_NOT_LEAK_RECOVERY_CODE_123",
];

struct TestDb {
    path: std::path::PathBuf,
}

impl TestDb {
    async fn new() -> (sqlx::SqlitePool, Self) {
        let mut path = std::env::temp_dir();
        path.push(format!("porkpie-plaintext-proof-{}.db", Uuid::new_v4()));
        let database_url = format!("sqlite://{}", path.display().to_string().replace('\\', "/"));

        let options = SqliteConnectOptions::from_str(&database_url)
            .expect("parse database url")
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect to test database");

        pool.execute("PRAGMA foreign_keys = ON;")
            .await
            .expect("enable foreign keys");

        for statement in MIGRATIONS {
            pool.execute(*statement).await.expect("run migration");
        }

        (pool, Self { path })
    }

    fn read_raw_bytes(&self) -> Vec<u8> {
        let main_bytes = std::fs::read(&self.path).expect("read SQLite file");
        let wal_path = self.path.with_extension("db-wal");
        let shm_path = self.path.with_extension("db-shm");
        let mut all = main_bytes;
        if wal_path.exists() {
            all.extend_from_slice(&std::fs::read(&wal_path).expect("read WAL file"));
        }
        if shm_path.exists() {
            all.extend_from_slice(&std::fs::read(&shm_path).expect("read SHM file"));
        }
        all
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(self.path.with_extension("db-wal"));
        let _ = std::fs::remove_file(self.path.with_extension("db-shm"));
    }
}

const MIGRATIONS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS vaults (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT NOT NULL UNIQUE,
        created_at INTEGER NOT NULL,
        salt BLOB NOT NULL,
        master_key_wrapped BLOB NOT NULL,
        sync_revision INTEGER NOT NULL DEFAULT 0,
        created_at_ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS items (
        id TEXT PRIMARY KEY NOT NULL,
        vault_id TEXT NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
        item_type TEXT NOT NULL,
        ciphertext BLOB NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        sync_revision INTEGER NOT NULL DEFAULT 0,
        created_at_ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS sync_state (
        vault_id TEXT PRIMARY KEY NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
        last_synced_revision INTEGER,
        last_synced_at INTEGER
    );
    "#,
    "CREATE INDEX IF NOT EXISTS idx_items_vault_id ON items(vault_id);",
    "CREATE INDEX IF NOT EXISTS idx_items_type ON items(item_type);",
    "CREATE INDEX IF NOT EXISTS idx_vaults_name ON vaults(name);",
];

#[tokio::test]
async fn raw_sqlite_does_not_contain_fixture_secrets() {
    let secret_key = LocalSecretKey::generate();
    let password = "correct horse battery staple";

    let (mut vault, _recovery) =
        Vault::create("TestVault", password, &secret_key).expect("vault should be created");

    let login_item = Item::new(ItemType::Login(LoginSecret {
        username: "admin@example.com".to_string(),
        password: "DO_NOT_LEAK_PASSWORD_123".to_string(),
        url: Some("https://example.com".to_string()),
        notes: None,
    }));

    let api_key_item = Item::new(ItemType::APIKey(APIKeySecret {
        name: "Production API".to_string(),
        key: "DO_NOT_LEAK_API_KEY_123".to_string(),
        provider: "example".to_string(),
    }));

    let ssh_key_item = Item::new(ItemType::SSHKey(SSHKeySecret {
        name: "prod-server".to_string(),
        public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5".to_string(),
        private_key: "DO_NOT_LEAK_PRIVATE_KEY_123".to_string(),
        passphrase: None,
        comment: Some("production server key".to_string()),
        allowed_hosts: vec!["prod.example.com".to_string()],
    }));

    let note_item = Item::new(ItemType::SecureNote(SecureNoteSecret {
        title: "Secret Note".to_string(),
        content: "DO_NOT_LEAK_NOTE_123".to_string(),
    }));

    let db_item = Item::new(ItemType::Database(DatabaseSecret {
        engine: "PostgreSQL".to_string(),
        host: "db.internal".to_string(),
        port: 5432,
        username: "pg_admin".to_string(),
        password: "DO_NOT_LEAK_DATABASE_PASSWORD_123".to_string(),
        database: "production".to_string(),
    }));

    let recovery_item = Item::new(ItemType::RecoveryCodes(RecoveryCodesSecret {
        codes: vec![
            "DO_NOT_LEAK_RECOVERY_CODE_123".to_string(),
            "SECOND_CODE_456".to_string(),
        ],
    }));

    let items_to_store: Vec<(Item, &str)> = vec![
        (login_item, "Login"),
        (api_key_item, "APIKey"),
        (ssh_key_item, "SSHKey"),
        (note_item, "SecureNote"),
        (db_item, "Database"),
        (recovery_item, "RecoveryCodes"),
    ];

    let (pool, test_db) = TestDb::new().await;

    store_vault(&pool, &vault)
        .await
        .expect("vault should persist");

    for (item, type_name) in items_to_store {
        let item_id = vault.create_item(item).expect("item should be created");
        let stored = vault.get_item(item_id).expect("item should exist").clone();
        let ciphertext = vault.encrypt_item(&stored).expect("item should encrypt");

        let encrypted = EncryptedItemData::new(
            item_id,
            vault.id,
            type_name,
            ciphertext,
            stored.created_at,
            stored.updated_at,
            vault.sync_revision(),
        );
        store_item(&pool, &encrypted)
            .await
            .expect("item should persist");
    }

    pool.close().await;

    let raw_bytes = test_db.read_raw_bytes();
    assert!(
        !raw_bytes.is_empty(),
        "SQLite file must have content after persisting vault and items"
    );

    for marker in FIXTURE_MARKERS {
        let marker_bytes = marker.as_bytes();
        let found = raw_bytes
            .windows(marker_bytes.len())
            .any(|window| window == marker_bytes);
        assert!(
            !found,
            "FIXTURE SECRET LEAKED: raw SQLite contains '{marker}'"
        );
    }
}

#[tokio::test]
async fn raw_sqlite_does_not_contain_vault_metadata_plaintext() {
    let secret_key = LocalSecretKey::generate();
    let password = "correct horse battery staple";

    let (vault, _recovery) =
        Vault::create("TestVault", password, &secret_key).expect("vault should be created");

    let (pool, test_db) = TestDb::new().await;

    store_vault(&pool, &vault)
        .await
        .expect("vault should persist");

    pool.close().await;

    let raw_bytes = test_db.read_raw_bytes();

    let vault_id_str = vault.id.to_string();
    let vault_id_bytes = vault_id_str.as_bytes();
    let found_vault_id = raw_bytes
        .windows(vault_id_bytes.len())
        .any(|window| window == vault_id_bytes);
    assert!(
        found_vault_id,
        "vault ID (non-secret metadata) should be present in raw SQLite"
    );

    assert!(
        !raw_bytes
            .windows(password.len())
            .any(|window| window == password.as_bytes()),
        "master password must not appear in raw SQLite"
    );
}
