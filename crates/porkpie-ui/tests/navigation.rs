use porkpie_types::Timestamp;
use porkpie_ui::state::{AppState, Screen};

#[test]
fn app_state_locks_and_returns_to_unlock_screen() {
    let mut state = AppState {
        unlocked: true,
        screen: Screen::List,
        ..AppState::default()
    };

    state.lock();

    assert!(!state.unlocked);
    assert_eq!(state.screen, Screen::Unlock);
    assert!(state.vault.is_none());
    assert!(state.items.is_empty());
}

#[test]
fn timeout_only_applies_to_unlocked_sessions() {
    let mut state = AppState {
        unlocked: true,
        last_activity: Timestamp(0),
        ..AppState::default()
    };

    assert!(!state.is_timed_out(Timestamp(30 * 60 * 1000)));
    assert!(state.is_timed_out(Timestamp(31 * 60 * 1000)));

    state.unlocked = false;
    assert!(!state.is_timed_out(Timestamp(31 * 60 * 1000)));
}

#[test]
fn navigation_state_can_target_each_screen() {
    let detail_id = porkpie_types::ItemId::new();
    let screens = [
        Screen::Onboarding,
        Screen::Unlock,
        Screen::List,
        Screen::Detail(detail_id),
        Screen::PasswordGenerator,
        Screen::ImportExport,
        Screen::Settings,
    ];

    for screen in screens {
        let state = AppState {
            screen: screen.clone(),
            ..AppState::default()
        };
        assert_eq!(state.screen, screen);
    }
}
