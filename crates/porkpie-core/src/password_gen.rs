use crate::errors::{CoreError, Result};
use rand::{rngs::OsRng, seq::SliceRandom};

const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 128;
const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";
const AMBIGUOUS: &[char] = &['0', 'O', 'o', '1', 'l', 'I', '|'];

/// Password generator options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordOptions {
    /// Include uppercase ASCII letters.
    pub uppercase: bool,
    /// Include lowercase ASCII letters.
    pub lowercase: bool,
    /// Include ASCII digits.
    pub numbers: bool,
    /// Include common ASCII symbols.
    pub symbols: bool,
    /// Exclude visually ambiguous characters such as `0`, `O`, `1`, and `l`.
    pub exclude_ambiguous: bool,
    /// Additional caller-provided characters to include in the generator pool.
    pub custom_chars: Option<String>,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            exclude_ambiguous: false,
            custom_chars: None,
        }
    }
}

/// Generate a cryptographically random password with the requested length and options.
pub fn generate_password(length: usize, options: &PasswordOptions) -> Result<String> {
    if !(MIN_PASSWORD_LENGTH..=MAX_PASSWORD_LENGTH).contains(&length) {
        return Err(CoreError::InvalidPasswordLength(length));
    }

    let required_sets = required_sets(options);
    if required_sets.is_empty() {
        return Err(CoreError::NoCharacterSetsSelected);
    }

    let mut required_chars = Vec::with_capacity(required_sets.len());
    let mut pool = Vec::new();
    let mut rng = OsRng;

    for set in required_sets {
        let filtered = filter_ambiguous(set, options.exclude_ambiguous);
        if filtered.is_empty() {
            return Err(CoreError::EmptyCharacterSet);
        }
        let selected = filtered
            .choose(&mut rng)
            .copied()
            .ok_or(CoreError::EmptyCharacterSet)?;
        required_chars.push(selected);
        pool.extend(filtered);
    }

    let remaining = length.saturating_sub(required_chars.len());
    let mut password_chars = required_chars;
    for _ in 0..remaining {
        let selected = pool
            .choose(&mut rng)
            .copied()
            .ok_or(CoreError::EmptyCharacterSet)?;
        password_chars.push(selected);
    }

    password_chars.shuffle(&mut rng);
    Ok(password_chars.into_iter().collect())
}

fn required_sets(options: &PasswordOptions) -> Vec<Vec<char>> {
    let mut sets = Vec::new();
    if options.uppercase {
        sets.push(bytes_to_chars(UPPERCASE));
    }
    if options.lowercase {
        sets.push(bytes_to_chars(LOWERCASE));
    }
    if options.numbers {
        sets.push(bytes_to_chars(NUMBERS));
    }
    if options.symbols {
        sets.push(bytes_to_chars(SYMBOLS));
    }
    if let Some(custom_chars) = &options.custom_chars {
        sets.push(custom_chars.chars().collect());
    }
    sets
}

fn bytes_to_chars(bytes: &[u8]) -> Vec<char> {
    bytes.iter().copied().map(char::from).collect()
}

fn filter_ambiguous(chars: Vec<char>, exclude_ambiguous: bool) -> Vec<char> {
    if exclude_ambiguous {
        chars
            .into_iter()
            .filter(|candidate| !AMBIGUOUS.contains(candidate))
            .collect()
    } else {
        chars
    }
}
