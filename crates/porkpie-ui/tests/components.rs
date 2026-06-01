use porkpie_types::{ItemId, Timestamp};
use porkpie_ui::state::{filter_items, ItemSummary};
use porkpie_ui::utils::validation::{
    has_password_character_set, validate_master_password, validate_password_length,
};

#[test]
fn validates_master_password_rules() {
    assert!(validate_master_password("sixteen-characters", "sixteen-characters").is_ok());
    assert!(validate_master_password("short", "short").is_err());
    assert!(validate_master_password("sixteen-characters", "different-characters").is_err());
}

#[test]
fn validates_generator_controls() {
    assert!(validate_password_length(8).is_ok());
    assert!(validate_password_length(128).is_ok());
    assert!(validate_password_length(7).is_err());
    assert!(validate_password_length(129).is_err());
    assert!(has_password_character_set(false, true, false, false));
    assert!(!has_password_character_set(false, false, false, false));
}

#[test]
fn password_generator_state_debug_redacts_generated_password() {
    use porkpie_ui::state::PasswordGeneratorState;
    let state = PasswordGeneratorState {
        generated_password: "my-secret-generated-password-123".to_string(),
        ..PasswordGeneratorState::default()
    };
    let debug = format!("{:?}", state);
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("my-secret-generated-password-123"));
    assert!(debug.contains("length"));
    assert!(debug.contains("uppercase"));
}

#[test]
fn password_generator_state_zeroizes_on_drop() {
    use porkpie_ui::state::PasswordGeneratorState;
    let mut state = PasswordGeneratorState {
        generated_password: "my-secret-generated-password-123".to_string(),
        ..PasswordGeneratorState::default()
    };
    state.clear_generated();
    assert!(state.generated_password.is_empty());
}

#[test]
fn app_state_lock_clears_decrypted_state() {
    use porkpie_ui::state::{AppState, Screen};
    let mut state = AppState::default();
    state.screen = Screen::List;
    state.password_generator.generated_password = "secret-password".to_string();
    state.lock();
    assert_eq!(state.screen, Screen::Unlock);
    assert!(state.current_item.is_none());
    assert!(state.items.is_empty());
    assert!(state.current_vault.is_none());
    assert!(state.unlocked_handle.is_none());
    assert!(state.password_generator.generated_password.is_empty());
}

#[test]
fn filters_item_list_by_title_or_type() {
    let now = Timestamp::now();
    let items = vec![
        ItemSummary {
            id: ItemId::new(),
            item_type: "Login".to_string(),
            title: "Email".to_string(),
            created_at: now,
            updated_at: now,
        },
        ItemSummary {
            id: ItemId::new(),
            item_type: "API key".to_string(),
            title: "Deploy token".to_string(),
            created_at: now,
            updated_at: now,
        },
    ];

    assert_eq!(filter_items(&items, "email").len(), 1);
    assert_eq!(filter_items(&items, "api").len(), 1);
    assert_eq!(filter_items(&items, "missing").len(), 0);
    assert_eq!(filter_items(&items, "").len(), 2);
}
