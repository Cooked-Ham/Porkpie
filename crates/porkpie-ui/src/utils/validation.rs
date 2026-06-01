/// Validate onboarding password fields.
pub fn validate_master_password(password: &str, confirmation: &str) -> Result<(), String> {
    if password.len() < 16 {
        return Err("Master password must be at least 16 characters".to_string());
    }

    if password != confirmation {
        return Err("Master password confirmation does not match".to_string());
    }

    Ok(())
}

/// Validate a password generator length.
pub fn validate_password_length(length: usize) -> Result<(), String> {
    if !(8..=128).contains(&length) {
        return Err("Password length must be between 8 and 128".to_string());
    }

    Ok(())
}

/// Return true when at least one character set is enabled.
pub fn has_password_character_set(
    uppercase: bool,
    lowercase: bool,
    numbers: bool,
    symbols: bool,
) -> bool {
    uppercase || lowercase || numbers || symbols
}
