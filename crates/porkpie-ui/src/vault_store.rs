#[cfg(not(target_arch = "wasm32"))]
use porkpie_core::Vault;
use porkpie_core::{Item, LocalSecretKey, RecoveryKit};
use porkpie_types::{ItemId, ItemType, Timestamp, VaultId};
use thiserror::Error;

/// Top-level result type used by the UI vault store.
pub type Result<T> = std::result::Result<T, VaultStoreError>;

/// Errors surfaced by the UI vault store.
///
/// The store never leaks raw SQL strings, decrypted material, or unrelated
/// internal state. Error messages are intentionally user-friendly.
#[derive(Debug, Error)]
pub enum VaultStoreError {
    #[error("Vault backend is not available in this environment")]
    Unavailable,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Vault '{0}' not found")]
    VaultNotFound(String),
    #[error("Vault '{0}' already exists")]
    VaultAlreadyExists(String),
    #[error("Wrong master password or local secret key")]
    WrongPassword,
    #[error("Item not found")]
    ItemNotFound,
    #[error("Invalid item data: {0}")]
    InvalidItem(String),
    #[error("JSON error: {0}")]
    Json(String),
    #[error("Import error: {0}")]
    Import(String),
    #[error("Export error: {0}")]
    Export(String),
}

/// Summary of a vault visible in the unlock / onboarding selectors.
///
/// Only the fields needed to identify the vault are exposed. The salt and
/// wrapped key are loaded on demand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultSummary {
    pub id: VaultId,
    pub name: String,
    pub created_at: Timestamp,
}

/// A redacted summary of an item used by the list view.
///
/// The UI never decrypts fields for the list view. Titles are decrypted on
/// the fly when an unlocked vault is open, and field values are only ever
/// materialised inside [`DecryptedItem`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSummary {
    pub id: ItemId,
    pub item_type: String,
    pub title: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A fully-decrypted item, held only while the user is viewing or editing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecryptedItem {
    pub id: ItemId,
    pub data: ItemType,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl DecryptedItem {
    pub fn into_core_item(self) -> Item {
        Item {
            id: self.id,
            data: self.data,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl From<&Item> for DecryptedItem {
    fn from(item: &Item) -> Self {
        Self {
            id: item.id,
            data: item.data.clone(),
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

/// Owned, plaintext export of a vault. Used only for explicit plaintext
/// export. The user must opt in via a confirmation flag.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlaintextExport {
    pub vault_name: String,
    pub exported_at: Timestamp,
    pub items: Vec<(String, ItemType)>,
}

/// Result of an encrypted backup export.
#[derive(Debug, Clone)]
pub struct EncryptedBackupExport {
    pub json: String,
    pub suggested_filename: String,
}

/// Summary of an encrypted import operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncryptedImportSummary {
    pub imported: usize,
    pub skipped: usize,
}

/// Backend that performs all vault I/O.
///
/// In a desktop or server-side web build this is backed by SQLite. In a
/// pure WASM build the backend is unavailable and every method returns
/// `VaultStoreError::Unavailable`.
#[derive(Clone)]
pub enum VaultBackend {
    #[cfg(not(target_arch = "wasm32"))]
    Sqlite(std::sync::Arc<tokio::sync::Mutex<SqliteState>>),
    #[cfg(target_arch = "wasm32")]
    LocalStorage(std::sync::Arc<std::sync::Mutex<LocalStorageState>>),
    Unavailable,
}

impl std::fmt::Debug for VaultBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Sqlite(_) => f.write_str("Sqlite(<state>)"),
            #[cfg(target_arch = "wasm32")]
            Self::LocalStorage(_) => f.write_str("LocalStorage(<state>)"),
            Self::Unavailable => f.write_str("Unavailable"),
        }
    }
}

impl VaultBackend {
    /// Create a new vault and persist it. Returns the summary and the
    /// recovery kit. Falls back to [`VaultStoreError::Unavailable`] when
    /// the backend is not configured (for example on WASM).
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn create_vault(
        &self,
        name: &str,
        password: &str,
        secret_key: &LocalSecretKey,
    ) -> Result<(VaultSummary, RecoveryKit)> {
        match self {
            Self::Sqlite(state) => {
                let pool = {
                    let guard = state.lock().await;
                    guard.pool.clone()
                };
                if name.trim().is_empty() {
                    return Err(VaultStoreError::InvalidItem(
                        "vault name is empty".to_string(),
                    ));
                }
                let (vault, recovery_kit) =
                    Vault::create(name, password, secret_key).map_err(VaultStoreError::from)?;
                let existing: Option<(String,)> =
                    sqlx::query_as("SELECT id FROM vaults WHERE name = ?")
                        .bind(name)
                        .fetch_optional(&pool)
                        .await
                        .map_err(|error| VaultStoreError::Database(error.to_string()))?;
                if existing.is_some() {
                    return Err(VaultStoreError::VaultAlreadyExists(name.to_string()));
                }
                porkpie_store::store_vault(&pool, &vault)
                    .await
                    .map_err(|error| VaultStoreError::Database(error.to_string()))?;
                Ok((
                    VaultSummary {
                        id: vault.id,
                        name: vault.name.clone(),
                        created_at: vault.created_at,
                    },
                    recovery_kit,
                ))
            }
            Self::Unavailable => Err(VaultStoreError::Unavailable),
        }
    }

    /// WASM implementation. The web shell uses localStorage when available.
    #[cfg(target_arch = "wasm32")]
    pub async fn create_vault(
        &self,
        name: &str,
        password: &str,
        secret_key: &LocalSecretKey,
    ) -> Result<(VaultSummary, RecoveryKit)> {
        match self {
            Self::LocalStorage(state) => {
                UnlockedVaultHandle::create(state.clone(), name, password, secret_key).await
            }
            Self::Unavailable => Err(VaultStoreError::Unavailable),
        }
    }

    /// Unlock an existing vault, load and decrypt all items, and return a
    /// handle for subsequent operations.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn unlock_vault(
        &self,
        name: &str,
        password: &str,
        secret_key: &LocalSecretKey,
    ) -> Result<UnlockedVaultHandle> {
        match self {
            Self::Sqlite(state) => {
                UnlockedVaultHandle::open(state.clone(), name, password, secret_key).await
            }
            Self::Unavailable => Err(VaultStoreError::Unavailable),
        }
    }

    /// WASM implementation. The web shell uses localStorage when available.
    #[cfg(target_arch = "wasm32")]
    pub async fn unlock_vault(
        &self,
        name: &str,
        password: &str,
        secret_key: &LocalSecretKey,
    ) -> Result<UnlockedVaultHandle> {
        match self {
            Self::LocalStorage(state) => {
                UnlockedVaultHandle::open(state.clone(), name, password, secret_key).await
            }
            Self::Unavailable => Err(VaultStoreError::Unavailable),
        }
    }

    /// Load the list of vault summaries. Used by the unlock page to show
    /// available vaults.
    pub async fn list_vault_summaries(&self) -> Result<Vec<VaultSummary>> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Sqlite(state) => {
                let pool = {
                    let guard = state.lock().await;
                    guard.pool.clone()
                };
                let rows: Vec<(String, String, i64)> = sqlx::query_as(
                    "SELECT id, name, created_at FROM vaults ORDER BY created_at, name",
                )
                .fetch_all(&pool)
                .await
                .map_err(|error| VaultStoreError::Database(error.to_string()))?;
                rows.into_iter()
                    .map(|(id, name, created_at)| {
                        let id = VaultId::from_string(&id).map_err(|_| {
                            VaultStoreError::Database("invalid vault id".to_string())
                        })?;
                        Ok(VaultSummary {
                            id,
                            name,
                            created_at: Timestamp(created_at),
                        })
                    })
                    .collect()
            }
            #[cfg(target_arch = "wasm32")]
            Self::LocalStorage(state) => {
                let guard = state.lock().unwrap();
                let vaults = guard.load_vaults()?;
                Ok(vaults
                    .into_iter()
                    .map(|v| VaultSummary {
                        id: v.id,
                        name: v.name,
                        created_at: v.created_at,
                    })
                    .collect())
            }
            Self::Unavailable => Err(VaultStoreError::Unavailable),
        }
    }

    /// Open a SQLite database at the given URL and return the backend.
    pub async fn connect_sqlite(database_url: &str) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let pool = porkpie_store::connect_database(database_url)
                .await
                .map_err(|error| VaultStoreError::Database(error.to_string()))?;
            Ok(Self::Sqlite(std::sync::Arc::new(tokio::sync::Mutex::new(
                SqliteState { pool },
            ))))
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = database_url;
            Err(VaultStoreError::Unavailable)
        }
    }

    /// Connect to the browser's localStorage and return the backend.
    /// Only available on the `wasm32` target.
    #[cfg(target_arch = "wasm32")]
    pub async fn connect_local_storage() -> Result<Self> {
        let window = web_sys::window().ok_or(VaultStoreError::Unavailable)?;
        let storage = window
            .local_storage()
            .map_err(|_| VaultStoreError::Unavailable)?
            .ok_or(VaultStoreError::Unavailable)?;
        Ok(Self::LocalStorage(std::sync::Arc::new(
            std::sync::Mutex::new(LocalStorageState { storage }),
        )))
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod sqlite_impl {
    use super::{
        item_title, item_type_label, DecryptedItem, EncryptedBackupExport, EncryptedImportSummary,
        Item, ItemId, ItemSummary, ItemType, LocalSecretKey, Result, Timestamp, Vault,
        VaultStoreError, VaultSummary,
    };
    use porkpie_store::{self, EncryptedItemData, EncryptedVaultData};
    use std::collections::HashSet;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Debug)]
    pub struct SqliteState {
        pub pool: sqlx::SqlitePool,
    }

    /// An unlocked vault with its items loaded in memory. Mutations are
    /// persisted to SQLite immediately.
    #[derive(Clone)]
    pub struct UnlockedVaultHandle {
        pub summary: VaultSummary,
        state: Arc<Mutex<SqliteState>>,
        vault: Arc<Mutex<Vault>>,
    }

    impl std::fmt::Debug for UnlockedVaultHandle {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("UnlockedVaultHandle")
                .field("summary", &self.summary)
                .field("vault", &"<unlocked>")
                .finish()
        }
    }

    impl UnlockedVaultHandle {
        pub async fn open(
            state: Arc<Mutex<SqliteState>>,
            name: &str,
            password: &str,
            secret_key: &LocalSecretKey,
        ) -> Result<Self> {
            let pool = {
                let guard = state.lock().await;
                guard.pool.clone()
            };
            let stored: EncryptedVaultData = porkpie_store::load_vault_by_name(&pool, name)
                .await
                .map_err(|error| match error {
                porkpie_store::StoreError::VaultNotFoundByName(name) => {
                    VaultStoreError::VaultNotFound(name)
                }
                other => VaultStoreError::Database(other.to_string()),
            })?;
            let summary = VaultSummary {
                id: stored.id,
                name: stored.name.clone(),
                created_at: stored.created_at,
            };
            let mut vault = stored.into_locked_vault();
            vault
                .unlock(password, secret_key)
                .map_err(VaultStoreError::from)?;
            let rows = porkpie_store::load_items_with_type(&pool, &summary.id)
                .await
                .map_err(|error| VaultStoreError::Database(error.to_string()))?;
            for (item_id, item_type, ciphertext) in rows {
                let item = vault
                    .decrypt_item(&ciphertext, &item_id, &item_type)
                    .map_err(VaultStoreError::from)?;
                vault.items_mut().insert(item.id, item);
            }
            Ok(Self {
                summary,
                state,
                vault: Arc::new(Mutex::new(vault)),
            })
        }

        pub async fn list_items(&self) -> Result<Vec<ItemSummary>> {
            let vault = self.vault.lock().await;
            let mut out: Vec<ItemSummary> = vault.items().values().map(ItemSummary::from).collect();
            out.sort_by(|a, b| {
                a.title
                    .cmp(&b.title)
                    .then(a.id.to_string().cmp(&b.id.to_string()))
            });
            Ok(out)
        }

        pub async fn get_item(&self, id: ItemId) -> Result<DecryptedItem> {
            let vault = self.vault.lock().await;
            vault
                .items()
                .get(&id)
                .map(DecryptedItem::from)
                .ok_or(VaultStoreError::ItemNotFound)
        }

        pub async fn create_item(&self, data: ItemType) -> Result<DecryptedItem> {
            let item = Item::new(data);
            let mut vault = self.vault.lock().await;
            let id = vault.create_item(item).map_err(VaultStoreError::from)?;
            let stored = vault
                .items()
                .get(&id)
                .cloned()
                .ok_or(VaultStoreError::ItemNotFound)?;
            let ciphertext = vault.encrypt_item(&stored).map_err(VaultStoreError::from)?;
            let item_type = item_type_label(&stored.data).to_string();
            let record = EncryptedItemData::new(
                stored.id,
                self.summary.id,
                item_type,
                ciphertext,
                stored.created_at,
                stored.updated_at,
                vault.sync_revision(),
            );
            let pool = {
                let guard = self.state.lock().await;
                guard.pool.clone()
            };
            porkpie_store::store_item(&pool, &record)
                .await
                .map_err(|error| VaultStoreError::Database(error.to_string()))?;
            Ok(DecryptedItem::from(&stored))
        }

        pub async fn update_item(&self, id: ItemId, data: ItemType) -> Result<DecryptedItem> {
            let mut vault = self.vault.lock().await;
            let created_at = vault
                .items()
                .get(&id)
                .map(|existing| existing.created_at)
                .ok_or(VaultStoreError::ItemNotFound)?;
            let updated_at = Timestamp::now();
            let item = Item {
                id,
                data,
                created_at,
                updated_at,
            };
            vault
                .update_item(id, item.clone())
                .map_err(VaultStoreError::from)?;
            let ciphertext = vault.encrypt_item(&item).map_err(VaultStoreError::from)?;
            let item_type = item_type_label(&item.data).to_string();
            let record = EncryptedItemData::new(
                item.id,
                self.summary.id,
                item_type,
                ciphertext,
                item.created_at,
                item.updated_at,
                vault.sync_revision(),
            );
            let pool = {
                let guard = self.state.lock().await;
                guard.pool.clone()
            };
            porkpie_store::update_item(&pool, &record.id, &record.ciphertext)
                .await
                .map_err(|error| VaultStoreError::Database(error.to_string()))?;
            Ok(DecryptedItem::from(&item))
        }

        pub async fn delete_item(&self, id: ItemId) -> Result<()> {
            {
                let mut vault = self.vault.lock().await;
                vault.delete_item(id).map_err(VaultStoreError::from)?;
            }
            let pool = {
                let guard = self.state.lock().await;
                guard.pool.clone()
            };
            porkpie_store::delete_item(&pool, &id)
                .await
                .map_err(|error| VaultStoreError::Database(error.to_string()))?;
            Ok(())
        }

        pub async fn lock(self) -> Result<()> {
            let mut vault = self.vault.lock().await;
            let _ = vault.lock();
            Ok(())
        }

        pub async fn export_encrypted(&self) -> Result<EncryptedBackupExport> {
            // Hold the vault lock for the duration of the export so that the
            // payload and the item list are snapshotted consistently.
            let vault = self.vault.lock().await;
            let pool = {
                let guard = self.state.lock().await;
                guard.pool.clone()
            };
            let encrypted_vault = porkpie_store::load_vault(&pool, &self.summary.id)
                .await
                .map_err(|error| VaultStoreError::Database(error.to_string()))?;
            let mut items: Vec<EncryptedItemData> = Vec::new();
            for item in vault.items().values() {
                let ciphertext = vault.encrypt_item(item).map_err(VaultStoreError::from)?;
                let item_type = item_type_label(&item.data).to_string();
                items.push(EncryptedItemData::new(
                    item.id,
                    self.summary.id,
                    item_type,
                    ciphertext,
                    item.created_at,
                    item.updated_at,
                    vault.sync_revision(),
                ));
            }
            let payload = porkpie_import::export_backup_file(&vault, encrypted_vault, items)
                .map_err(|error| VaultStoreError::Export(error.to_string()))?;
            Ok(EncryptedBackupExport {
                json: serde_json::to_string_pretty(&payload)
                    .map_err(|error| VaultStoreError::Json(error.to_string()))?,
                suggested_filename: porkpie_import::backup_file_name(Timestamp::now().to_millis()),
            })
        }

        pub async fn export_plaintext(&self, confirm: bool) -> Result<super::PlaintextExport> {
            if !confirm {
                return Err(VaultStoreError::Export(
                    "plaintext export requires explicit confirmation".to_string(),
                ));
            }
            let vault = self.vault.lock().await;
            let items = vault
                .items()
                .values()
                .map(|item| (item_title(&item.data).to_string(), item.data.clone()))
                .collect();
            Ok(super::PlaintextExport {
                vault_name: self.summary.name.clone(),
                exported_at: Timestamp::now(),
                items,
            })
        }

        pub async fn import_encrypted_with_keys(
            &self,
            password: &str,
            secret_key: &LocalSecretKey,
            json: &str,
            mode: porkpie_import::BackupImportMode,
        ) -> Result<EncryptedImportSummary> {
            let backup: porkpie_import::BackupFile = serde_json::from_str(json)
                .map_err(|error| VaultStoreError::Json(error.to_string()))?;
            let pool = {
                let guard = self.state.lock().await;
                guard.pool.clone()
            };
            let existing: HashSet<String> =
                sqlx::query_as::<_, (String,)>("SELECT id FROM items WHERE vault_id = ?")
                    .bind(self.summary.id.to_string())
                    .fetch_all(&pool)
                    .await
                    .map_err(|error| VaultStoreError::Database(error.to_string()))?
                    .into_iter()
                    .map(|(id,)| id)
                    .collect();
            let result =
                porkpie_import::import_backup(backup, password, secret_key, &existing, mode)
                    .map_err(|error| VaultStoreError::Import(error.to_string()))?;
            let mut vault = self.vault.lock().await;
            for item in &result.items {
                porkpie_store::store_item(&pool, item)
                    .await
                    .map_err(|error| VaultStoreError::Database(error.to_string()))?;
                let decrypted = vault
                    .decrypt_item(&item.ciphertext, &item.id, &item.item_type)
                    .map_err(VaultStoreError::from)?;
                vault.items_mut().insert(decrypted.id, decrypted);
            }
            Ok(EncryptedImportSummary {
                imported: result.imported,
                skipped: result.skipped,
            })
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use sqlite_impl::{SqliteState, UnlockedVaultHandle};

#[cfg(target_arch = "wasm32")]
pub use local_storage_impl::{LocalStorageState, UnlockedVaultHandle};

impl From<porkpie_core::CoreError> for VaultStoreError {
    fn from(error: porkpie_core::CoreError) -> Self {
        match error {
            porkpie_core::CoreError::WrongPassword => VaultStoreError::WrongPassword,
            porkpie_core::CoreError::VaultLocked => VaultStoreError::WrongPassword,
            porkpie_core::CoreError::ItemNotFound => VaultStoreError::ItemNotFound,
            other => VaultStoreError::Database(other.to_string()),
        }
    }
}

impl From<&Item> for ItemSummary {
    fn from(item: &Item) -> Self {
        Self {
            id: item.id,
            item_type: item_type_label(&item.data).to_string(),
            title: item_title(&item.data),
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod local_storage_impl {
    use super::{
        item_title, item_type_label, DecryptedItem, EncryptedBackupExport, EncryptedImportSummary,
        Item, ItemId, ItemSummary, ItemType, LocalSecretKey, Result, Timestamp, Vault,
        VaultStoreError, VaultSummary,
    };
    use porkpie_store::{EncryptedItemData, EncryptedVaultData};
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::sync::Mutex;
    use web_sys::Storage;

    const VAULTS_KEY: &str = "porkpie_vaults";
    fn items_key(vault_id: &str) -> String {
        format!("porkpie_items:{}", vault_id)
    }

    #[derive(Debug)]
    pub struct LocalStorageState {
        pub storage: Storage,
    }

    impl LocalStorageState {
        fn load_vaults(&self) -> Result<Vec<EncryptedVaultData>> {
            let json = self
                .storage
                .get_item(VAULTS_KEY)
                .map_err(|_| VaultStoreError::Database("localStorage read failed".to_string()))?
                .unwrap_or_else(|| "[]".to_string());
            serde_json::from_str(&json).map_err(|e| VaultStoreError::Json(e.to_string()))
        }

        fn save_vaults(&self, vaults: &[EncryptedVaultData]) -> Result<()> {
            let json =
                serde_json::to_string(vaults).map_err(|e| VaultStoreError::Json(e.to_string()))?;
            self.storage
                .set_item(VAULTS_KEY, &json)
                .map_err(|_| VaultStoreError::Database("localStorage write failed".to_string()))
        }

        fn load_items(&self, vault_id: &str) -> Result<Vec<EncryptedItemData>> {
            let json = self
                .storage
                .get_item(&items_key(vault_id))
                .map_err(|_| VaultStoreError::Database("localStorage read failed".to_string()))?
                .unwrap_or_else(|| "[]".to_string());
            serde_json::from_str(&json).map_err(|e| VaultStoreError::Json(e.to_string()))
        }

        fn save_items(&self, vault_id: &str, items: &[EncryptedItemData]) -> Result<()> {
            let json =
                serde_json::to_string(items).map_err(|e| VaultStoreError::Json(e.to_string()))?;
            self.storage
                .set_item(&items_key(vault_id), &json)
                .map_err(|_| VaultStoreError::Database("localStorage write failed".to_string()))
        }
    }

    /// An unlocked vault backed by localStorage.
    #[derive(Clone)]
    pub struct UnlockedVaultHandle {
        pub summary: VaultSummary,
        state: Arc<Mutex<LocalStorageState>>,
        vault: Arc<Mutex<Vault>>,
    }

    impl std::fmt::Debug for UnlockedVaultHandle {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("UnlockedVaultHandle")
                .field("summary", &self.summary)
                .field("vault", &"<unlocked>")
                .finish()
        }
    }

    impl UnlockedVaultHandle {
        pub async fn create(
            state: Arc<Mutex<LocalStorageState>>,
            name: &str,
            password: &str,
            secret_key: &LocalSecretKey,
        ) -> Result<(VaultSummary, porkpie_types::RecoveryKit)> {
            if name.trim().is_empty() {
                return Err(VaultStoreError::InvalidItem(
                    "vault name is empty".to_string(),
                ));
            }
            let (vault, recovery_kit) =
                Vault::create(name, password, secret_key).map_err(VaultStoreError::from)?;
            let guard = state.lock().unwrap();
            let mut vaults = guard.load_vaults()?;
            if vaults.iter().any(|v| v.name == name) {
                return Err(VaultStoreError::VaultAlreadyExists(name.to_string()));
            }
            vaults.push(EncryptedVaultData {
                id: vault.id,
                name: vault.name.clone(),
                created_at: vault.created_at,
                salt: vault.salt,
                master_key_wrapped: vault.master_key_wrapped().clone(),
                sync_revision: vault.sync_revision(),
            });
            guard.save_vaults(&vaults)?;
            guard.save_items(&vault.id.to_string(), &[])?;
            Ok((
                VaultSummary {
                    id: vault.id,
                    name: vault.name.clone(),
                    created_at: vault.created_at,
                },
                recovery_kit,
            ))
        }

        pub async fn open(
            state: Arc<Mutex<LocalStorageState>>,
            name: &str,
            password: &str,
            secret_key: &LocalSecretKey,
        ) -> Result<Self> {
            let guard = state.lock().unwrap();
            let vaults = guard.load_vaults()?;
            let stored = vaults
                .into_iter()
                .find(|v| v.name == name)
                .ok_or_else(|| VaultStoreError::VaultNotFound(name.to_string()))?;
            let summary = VaultSummary {
                id: stored.id,
                name: stored.name.clone(),
                created_at: stored.created_at,
            };
            let mut vault = stored.into_locked_vault();
            vault
                .unlock(password, secret_key)
                .map_err(VaultStoreError::from)?;
            let items = guard.load_items(&summary.id.to_string())?;
            for item in items {
                let decrypted = vault
                    .decrypt_item(&item.ciphertext, &item.id, &item.item_type)
                    .map_err(VaultStoreError::from)?;
                vault.items_mut().insert(decrypted.id, decrypted);
            }
            Ok(Self {
                summary,
                state,
                vault: Arc::new(Mutex::new(vault)),
            })
        }

        pub async fn list_items(&self) -> Result<Vec<ItemSummary>> {
            let vault = self.vault.lock().unwrap();
            let mut out: Vec<ItemSummary> = vault.items().values().map(ItemSummary::from).collect();
            out.sort_by(|a, b| {
                a.title
                    .cmp(&b.title)
                    .then(a.id.to_string().cmp(&b.id.to_string()))
            });
            Ok(out)
        }

        pub async fn get_item(&self, id: ItemId) -> Result<DecryptedItem> {
            let vault = self.vault.lock().unwrap();
            vault
                .items()
                .get(&id)
                .map(DecryptedItem::from)
                .ok_or(VaultStoreError::ItemNotFound)
        }

        pub async fn create_item(&self, data: ItemType) -> Result<DecryptedItem> {
            let item = Item::new(data);
            let mut vault = self.vault.lock().unwrap();
            let id = vault.create_item(item).map_err(VaultStoreError::from)?;
            let stored = vault
                .items()
                .get(&id)
                .cloned()
                .ok_or(VaultStoreError::ItemNotFound)?;
            let ciphertext = vault.encrypt_item(&stored).map_err(VaultStoreError::from)?;
            let item_type = item_type_label(&stored.data).to_string();
            let record = EncryptedItemData::new(
                stored.id,
                self.summary.id,
                item_type,
                ciphertext,
                stored.created_at,
                stored.updated_at,
                vault.sync_revision(),
            );
            let guard = self.state.lock().unwrap();
            let mut items = guard.load_items(&self.summary.id.to_string())?;
            items.push(record);
            guard.save_items(&self.summary.id.to_string(), &items)?;
            Ok(DecryptedItem::from(&stored))
        }

        pub async fn update_item(&self, id: ItemId, data: ItemType) -> Result<DecryptedItem> {
            let mut vault = self.vault.lock().unwrap();
            let created_at = vault
                .items()
                .get(&id)
                .map(|existing| existing.created_at)
                .ok_or(VaultStoreError::ItemNotFound)?;
            let updated_at = Timestamp::now();
            let item = Item {
                id,
                data,
                created_at,
                updated_at,
            };
            vault
                .update_item(id, item.clone())
                .map_err(VaultStoreError::from)?;
            let ciphertext = vault.encrypt_item(&item).map_err(VaultStoreError::from)?;
            let item_type = item_type_label(&item.data).to_string();
            let record = EncryptedItemData::new(
                item.id,
                self.summary.id,
                item_type,
                ciphertext,
                item.created_at,
                item.updated_at,
                vault.sync_revision(),
            );
            let guard = self.state.lock().unwrap();
            let mut items = guard.load_items(&self.summary.id.to_string())?;
            if let Some(existing) = items.iter_mut().find(|i| i.id == id) {
                *existing = record;
            } else {
                return Err(VaultStoreError::ItemNotFound);
            }
            guard.save_items(&self.summary.id.to_string(), &items)?;
            Ok(DecryptedItem::from(&item))
        }

        pub async fn delete_item(&self, id: ItemId) -> Result<()> {
            {
                let mut vault = self.vault.lock().unwrap();
                vault.delete_item(id).map_err(VaultStoreError::from)?;
            }
            let guard = self.state.lock().unwrap();
            let mut items = guard.load_items(&self.summary.id.to_string())?;
            items.retain(|i| i.id != id);
            guard.save_items(&self.summary.id.to_string(), &items)?;
            Ok(())
        }

        pub async fn lock(self) -> Result<()> {
            let mut vault = self.vault.lock().unwrap();
            let _ = vault.lock();
            Ok(())
        }

        pub async fn export_encrypted(&self) -> Result<EncryptedBackupExport> {
            let vault = self.vault.lock().unwrap();
            let guard = self.state.lock().unwrap();
            let encrypted_vault = guard
                .load_vaults()?
                .into_iter()
                .find(|v| v.id == self.summary.id)
                .ok_or(VaultStoreError::VaultNotFound(self.summary.name.clone()))?;
            let mut items: Vec<EncryptedItemData> = Vec::new();
            for item in vault.items().values() {
                let ciphertext = vault.encrypt_item(item).map_err(VaultStoreError::from)?;
                let item_type = item_type_label(&item.data).to_string();
                items.push(EncryptedItemData::new(
                    item.id,
                    self.summary.id,
                    item_type,
                    ciphertext,
                    item.created_at,
                    item.updated_at,
                    vault.sync_revision(),
                ));
            }
            let payload = porkpie_import::export_backup_file(&vault, encrypted_vault, items)
                .map_err(|error| VaultStoreError::Export(error.to_string()))?;
            Ok(EncryptedBackupExport {
                json: serde_json::to_string_pretty(&payload)
                    .map_err(|error| VaultStoreError::Json(error.to_string()))?,
                suggested_filename: porkpie_import::backup_file_name(Timestamp::now().to_millis()),
            })
        }

        pub async fn export_plaintext(&self, confirm: bool) -> Result<super::PlaintextExport> {
            if !confirm {
                return Err(VaultStoreError::Export(
                    "plaintext export requires explicit confirmation".to_string(),
                ));
            }
            let vault = self.vault.lock().unwrap();
            let items = vault
                .items()
                .values()
                .map(|item| (item_title(&item.data).to_string(), item.data.clone()))
                .collect();
            Ok(super::PlaintextExport {
                vault_name: self.summary.name.clone(),
                exported_at: Timestamp::now(),
                items,
            })
        }

        pub async fn import_encrypted_with_keys(
            &self,
            password: &str,
            secret_key: &LocalSecretKey,
            json: &str,
            mode: porkpie_import::BackupImportMode,
        ) -> Result<EncryptedImportSummary> {
            let backup: porkpie_import::BackupFile = serde_json::from_str(json)
                .map_err(|error| VaultStoreError::Json(error.to_string()))?;
            let guard = self.state.lock().unwrap();
            let existing: HashSet<String> = guard
                .load_items(&self.summary.id.to_string())?
                .into_iter()
                .map(|item| item.id.to_string())
                .collect();
            let result =
                porkpie_import::import_backup(backup, password, secret_key, &existing, mode)
                    .map_err(|error| VaultStoreError::Import(error.to_string()))?;
            let mut vault = self.vault.lock().unwrap();
            let mut items = guard.load_items(&self.summary.id.to_string())?;
            for item in &result.items {
                items.push(item.clone());
                let decrypted = vault
                    .decrypt_item(&item.ciphertext, &item.id, &item.item_type)
                    .map_err(VaultStoreError::from)?;
                vault.items_mut().insert(decrypted.id, decrypted);
            }
            guard.save_items(&self.summary.id.to_string(), &items)?;
            Ok(EncryptedImportSummary {
                imported: result.imported,
                skipped: result.skipped,
            })
        }
    }
}

pub(super) fn item_type_label(item_type: &ItemType) -> &'static str {
    match item_type {
        ItemType::Login(_) => "Login",
        ItemType::APIKey(_) => "APIKey",
        ItemType::SSHKey(_) => "SSHKey",
        ItemType::SecureNote(_) => "SecureNote",
        ItemType::Server(_) => "Server",
        ItemType::Database(_) => "Database",
        ItemType::Identity(_) => "Identity",
        ItemType::SoftwareLicense(_) => "SoftwareLicense",
        ItemType::RecoveryCodes(_) => "RecoveryCodes",
        ItemType::Custom(_) => "Custom",
    }
}

pub(super) fn item_title(item_type: &ItemType) -> String {
    match item_type {
        ItemType::Login(secret) => secret.username.clone(),
        ItemType::APIKey(secret) => secret.name.clone(),
        ItemType::SSHKey(secret) => secret.name.clone(),
        ItemType::SecureNote(secret) => secret.title.clone(),
        ItemType::Server(secret) => secret.hostname.clone(),
        ItemType::Database(secret) => format!("{}/{}", secret.engine, secret.database),
        ItemType::Identity(secret) => secret.name.clone(),
        ItemType::SoftwareLicense(secret) => secret.product.clone(),
        ItemType::RecoveryCodes(secret) => {
            format!("{} recovery codes", secret.codes.len())
        }
        ItemType::Custom(secret) => secret
            .fields
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "Custom item".to_string()),
    }
}
