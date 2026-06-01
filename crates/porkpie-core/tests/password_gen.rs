use porkpie_core::{generate_password, CoreError, PasswordOptions};

#[test]
fn password_generation_respects_length_and_enabled_sets() {
    let options = PasswordOptions::default();

    let password = generate_password(32, &options).expect("password should generate");

    assert_eq!(password.len(), 32);
    assert!(password.chars().any(|c| c.is_ascii_uppercase()));
    assert!(password.chars().any(|c| c.is_ascii_lowercase()));
    assert!(password.chars().any(|c| c.is_ascii_digit()));
    assert!(password.chars().any(|c| !c.is_ascii_alphanumeric()));
}

#[test]
fn password_generation_rejects_invalid_lengths() {
    let options = PasswordOptions::default();

    assert!(matches!(
        generate_password(7, &options),
        Err(CoreError::InvalidPasswordLength(7))
    ));
    assert!(matches!(
        generate_password(129, &options),
        Err(CoreError::InvalidPasswordLength(129))
    ));
}

#[test]
fn password_generation_rejects_empty_pool() {
    let options = PasswordOptions {
        uppercase: false,
        lowercase: false,
        numbers: false,
        symbols: false,
        exclude_ambiguous: false,
        custom_chars: None,
    };

    assert!(matches!(
        generate_password(16, &options),
        Err(CoreError::NoCharacterSetsSelected)
    ));
}

#[test]
fn password_generation_excludes_ambiguous_characters() {
    let options = PasswordOptions {
        uppercase: true,
        lowercase: true,
        numbers: true,
        symbols: false,
        exclude_ambiguous: true,
        custom_chars: None,
    };

    let password = generate_password(64, &options).expect("password should generate");

    for ambiguous in ['0', 'O', 'o', '1', 'l', 'I', '|'] {
        assert!(!password.contains(ambiguous));
    }
}

#[test]
fn password_generation_supports_custom_characters() {
    let options = PasswordOptions {
        uppercase: false,
        lowercase: false,
        numbers: false,
        symbols: false,
        exclude_ambiguous: false,
        custom_chars: Some("abcXYZ".to_string()),
    };

    let password = generate_password(16, &options).expect("password should generate");

    assert!(password.chars().all(|c| "abcXYZ".contains(c)));
}
