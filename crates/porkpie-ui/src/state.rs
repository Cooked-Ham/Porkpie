use porkpie_core::{PasswordOptions, Vault};
use porkpie_types::{ItemId, Timestamp, VaultId};
use serde::{Deserialize, Serialize};

/// Top-level page shown by the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    Onboarding,
    Unlock,
    List,
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

/// Minimal item summary rendered by the list page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemSummary {
    pub id: ItemId,
    pub item_type: String,
    pub title: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Password generator form state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordGeneratorState {
    pub length: usize,
    pub options: PasswordOptions,
    pub generated_password: String,
}

impl Default for PasswordGeneratorState {
    fn default() -> Self {
        Self {
            length: 24,
            options: PasswordOptions::default(),
            generated_password: String::new(),
        }
    }
}

/// Application state owned by the root component.
pub struct AppState {
    pub screen: Screen,
    pub current_vault_id: Option<VaultId>,
    pub vault: Option<Vault>,
    pub unlocked: bool,
    pub items: Vec<ItemSummary>,
    pub search_query: String,
    pub last_activity: Timestamp,
    pub settings: SettingsState,
    pub password_generator: PasswordGeneratorState,
    pub toast: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Onboarding,
            current_vault_id: None,
            vault: None,
            unlocked: false,
            items: Vec::new(),
            search_query: String::new(),
            last_activity: Timestamp::now(),
            settings: SettingsState::default(),
            password_generator: PasswordGeneratorState::default(),
            toast: None,
        }
    }
}

impl AppState {
    /// Mark the state as locked and purge in-memory decrypted vault state.
    pub fn lock(&mut self) {
        self.vault = None;
        self.unlocked = false;
        self.screen = Screen::Unlock;
        self.items.clear();
        self.last_activity = Timestamp::now();
    }

    /// Returns true when the configured inactivity timeout has elapsed.
    pub fn is_timed_out(&self, now: Timestamp) -> bool {
        if !self.unlocked {
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
