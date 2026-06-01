use crate::vault_store::UnlockedVaultHandle;
pub use crate::vault_store::{DecryptedItem, ItemSummary, VaultSummary};
use porkpie_types::{ItemId, Timestamp};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Top-level page shown by the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    Onboarding,
    Unlock,
    List,
    NewItem,
    Detail(ItemId),
    PasswordGenerator,
    ImportExport,
    Settings,
}

/// UI theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

/// User-configurable app settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsState {
    pub lock_timeout_minutes: u16,
    pub theme: Theme,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            lock_timeout_minutes: 30,
            theme: Theme::Dark,
        }
    }
}

/// Password generator form state.
///
/// The generated password is zeroized when the generator is reset or the
/// vault is locked.
#[derive(Clone, PartialEq, Eq)]
pub struct PasswordGeneratorState {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub numbers: bool,
    pub symbols: bool,
    pub exclude_ambiguous: bool,
    pub generated_password: String,
}

impl std::fmt::Debug for PasswordGeneratorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasswordGeneratorState")
            .field("length", &self.length)
            .field("uppercase", &self.uppercase)
            .field("lowercase", &self.lowercase)
            .field("numbers", &self.numbers)
            .field("symbols", &self.symbols)
            .field("exclude_ambiguous", &self.exclude_ambiguous)
            .field("generated_password", &"[redacted]")
            .finish()
    }
}

impl Default for PasswordGeneratorState {
    fn default() -> Self {
        Self {
            length: 24,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            exclude_ambiguous: false,
            generated_password: String::new(),
        }
    }
}

impl PasswordGeneratorState {
    pub fn to_options(&self) -> porkpie_core::PasswordOptions {
        porkpie_core::PasswordOptions {
            uppercase: self.uppercase,
            lowercase: self.lowercase,
            numbers: self.numbers,
            symbols: self.symbols,
            exclude_ambiguous: self.exclude_ambiguous,
            custom_chars: None,
        }
    }

    /// Zeroize the generated password and reset it to empty.
    pub fn clear_generated(&mut self) {
        self.generated_password.zeroize();
        self.generated_password.clear();
    }
}

impl Drop for PasswordGeneratorState {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.generated_password.zeroize();
    }
}

/// Application state owned by the root component.
///
/// Holds only metadata and non-secret data. Decrypted items are held in
/// short-lived form via [`AppState::current_item`] and the on-disk vault
/// store. No decrypted secret material is persisted to localStorage,
/// sessionStorage, or any client-side cache.
pub struct AppState {
    pub screen: Screen,
    pub vaults: Vec<VaultSummary>,
    pub current_vault: Option<VaultSummary>,
    pub items: Vec<ItemSummary>,
    pub current_item: Option<DecryptedItem>,
    pub unlocked_handle: Option<UnlockedVaultHandle>,
    pub search_query: String,
    pub last_activity: Timestamp,
    pub settings: SettingsState,
    pub password_generator: PasswordGeneratorState,
    pub toast: Option<String>,
    pub error: Option<String>,
    pub status: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Onboarding,
            vaults: Vec::new(),
            current_vault: None,
            items: Vec::new(),
            current_item: None,
            unlocked_handle: None,
            search_query: String::new(),
            last_activity: Timestamp::now(),
            settings: SettingsState::default(),
            password_generator: PasswordGeneratorState::default(),
            toast: None,
            error: None,
            status: None,
        }
    }
}

impl AppState {
    /// Mark the state as locked and purge in-memory decrypted vault state.
    ///
    /// Zeroizes any owned secret material before dropping the references:
    /// - clears the generated password from the password generator,
    /// - drops the unlocked vault handle (which zeroizes the vault key),
    /// - clears the current item detail.
    pub fn lock(&mut self) {
        self.current_vault = None;
        self.items.clear();
        self.current_item = None;
        // Zeroize generated password before dropping the handle.
        self.password_generator.clear_generated();
        // Drop the unlocked handle so the in-memory vault key is zeroized.
        self.unlocked_handle = None;
        self.screen = Screen::Unlock;
        self.last_activity = Timestamp::now();
    }

    /// Returns true when the configured inactivity timeout has elapsed.
    pub fn is_timed_out(&self, now: Timestamp) -> bool {
        if self.current_vault.is_none() {
            return false;
        }
        let elapsed = now.to_millis() - self.last_activity.to_millis();
        elapsed > i64::from(self.settings.lock_timeout_minutes) * 60 * 1000
    }

    /// Update the last activity timestamp.
    pub fn touch(&mut self) {
        self.last_activity = Timestamp::now();
    }

    /// Return list rows filtered by the current search query.
    pub fn filtered_items(&self) -> Vec<ItemSummary> {
        filter_items(&self.items, &self.search_query)
    }
}

/// Filter item summaries by type or title.
pub fn filter_items(items: &[ItemSummary], query: &str) -> Vec<ItemSummary> {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return items.to_vec();
    }

    items
        .iter()
        .filter(|item| {
            item.title.to_ascii_lowercase().contains(&query)
                || item.item_type.to_ascii_lowercase().contains(&query)
        })
        .cloned()
        .collect()
}
