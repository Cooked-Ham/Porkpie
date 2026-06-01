---
task_id: 04-crypto
task_name: Cryptographic Operations
sequence: 4
dependencies_complete: [01-workspace, 02-docs, 03-types]
estimated_duration: 4-5 hours
difficulty: High
blockers_resolved: none
can_parallelize: false
security_critical: true
---

# Task 4: Cryptographic Operations

## 🎯 Objective

Implement `porkpie-crypto` crate with production-grade encryption. **This is security-critical.** Every function must be bulletproof.

## ⚠️ SECURITY INVARIANTS (CRITICAL)

**Non-negotiable rules for this task:**

1. ✓ No mock crypto in production code
2. ✓ No static keys or hardcoded values
3. ✓ No nonce reuse (fail if duplicate)
4. ✓ No plaintext output except decrypt() result
5. ✓ No logging of secrets
6. ✓ Wrong password → error (never decrypt)
7. ✓ Tampered ciphertext → error (MAC fail)
8. ✓ Use `zeroize` for sensitive memory
9. ✓ Use Argon2id for key derivation
10. ✓ Use XChaCha20Poly1305 for AEAD

**If you violate any rule, stop and escalate.**

## ✅ Acceptance Criteria

**Key Derivation**
- [ ] `derive_key(password: &str, salt: &[u8; 32]) -> Result<[u8; 32]>`
  - [ ] Uses Argon2id (not Argon2i, not Argon2d)
  - [ ] Salt must be 32 bytes
  - [ ] Output must be 32 bytes
  - [ ] Configurable time/memory/parallelism
- [ ] Key derivation is deterministic (same input → same key)
- [ ] Different salt → different key
- [ ] Test: Wrong password produces error

**Vault Key Wrapping**
- [ ] `wrap_vault_key(master_key: &[u8; 32], vault_key: &[u8; 32]) -> Result<EncryptedVaultKey>`
- [ ] `unwrap_vault_key(master_key: &[u8; 32], wrapped: &EncryptedVaultKey) -> Result<[u8; 32]>`
- [ ] Wrapping is deterministic
- [ ] Unwrapping with wrong key fails
- [ ] Tampered wrapped key fails

**Item Encryption**
- [ ] `encrypt_item<T: Serialize>(item: &T, key: &[u8; 32]) -> Result<Vec<u8>>`
- [ ] `decrypt_item<T: DeserializeOwned>(ciphertext: &[u8], key: &[u8; 32]) -> Result<T>`
- [ ] Uses XChaCha20Poly1305
- [ ] Each encryption produces different ciphertext (nonce randomness)
- [ ] Tampered ciphertext → error
- [ ] Wrong key → error

**Nonce Management**
- [ ] `generate_nonce() -> [u8; 24]`
- [ ] Cryptographically secure random
- [ ] Never reuses nonce with same key (fail if duplicate)
- [ ] Nonce stored in output (not ephemeral)

**MAC Validation**
- [ ] All decrypt operations verify authentication tag
- [ ] Truncated ciphertext → error
- [ ] Modified ciphertext → error
- [ ] Constant-time comparison for MAC

**Memory Safety**
- [ ] Plaintext keys zeroized after use
- [ ] Plaintext passwords zeroized after use
- [ ] No plaintext copied unnecessarily
- [ ] Use `secrecy::Secret<T>` for sensitive types

**Error Handling**
- [ ] Wrong password detected (specific error)
- [ ] Corruption detected (specific error)
- [ ] Invalid parameters caught
- [ ] No panics on bad input (return errors)

**Tests**
- [ ] Encrypt → decrypt round-trip (plaintext matches)
- [ ] Different nonces for different encryptions
- [ ] Tamper detection (modify 1 byte → fail)
- [ ] Truncate ciphertext → fail
- [ ] Wrong key → fail
- [ ] Wrong password → fail
- [ ] Argon2id params verify expected
- [ ] NIST test vectors (if available)

**Code Quality**
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes (zero warnings)
- [ ] No `todo!()` or `unwrap()` in prod code
- [ ] All functions documented
- [ ] Security reasoning documented

## 📋 Output Specification

### File Structure

```
crates/porkpie-crypto/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Module declarations + public API
│   ├── key_derivation.rs       # Argon2id operations
│   ├── encryption.rs           # XChaCha20Poly1305 operations
│   ├── nonce.rs                # Nonce generation + tracking
│   ├── vault_key.rs            # Vault key wrapping
│   ├── errors.rs               # Crypto-specific errors
│   └── constants.rs            # Security parameters
└── tests/
    ├── encryption.rs           # Encrypt/decrypt tests
    ├── tampering.rs            # Tamper detection tests
    ├── key_derivation.rs       # Argon2id tests
    └── security.rs             # Security invariant tests
```

### Cargo.toml

```toml
[package]
name = "porkpie-crypto"
version = "0.1.0"
edition = "2021"

[dependencies]
porkpie-types = { path = "../porkpie-types" }
chacha20poly1305 = "0.10"
argon2 = "0.5"
rand = "0.8"
zeroize = "1.6"
secrecy = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"

[dev-dependencies]
hex = "0.4"
```

### Example: Key Derivation (`src/key_derivation.rs`)

```rust
use argon2::{Argon2, ParamsBuilder, VariantParamBuilder, PasswordHash, PasswordHasher};
use rand::Rng;
use zeroize::Zeroizing;

pub struct Argon2Params {
    pub time_cost: u32,      // 2-3 recommended
    pub mem_cost: u32,       // 19 MiB recommended
    pub parallelism: u32,    // 1 recommended
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            time_cost: 2,
            mem_cost: 19456,
            parallelism: 1,
        }
    }
}

pub fn derive_key(
    password: &str,
    salt: &[u8; 32],
    params: &Argon2Params,
) -> Result<[u8; 32], CryptoError> {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Argon2::default().params().clone(),
    );
    
    // Derive using Argon2id
    // Return [u8; 32] key
    
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_password_same_salt_produces_same_key() {
        let password = "correct horse battery staple";
        let salt = [0u8; 32];
        let key1 = derive_key(password, &salt, &Default::default()).unwrap();
        let key2 = derive_key(password, &salt, &Default::default()).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn different_salt_produces_different_key() {
        let password = "correct horse battery staple";
        let salt1 = [0u8; 32];
        let salt2 = [1u8; 32];
        let key1 = derive_key(password, &salt1, &Default::default()).unwrap();
        let key2 = derive_key(password, &salt2, &Default::default()).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn wrong_password_produces_different_key() {
        let salt = [0u8; 32];
        let key1 = derive_key("correct horse battery staple", &salt, &Default::default()).unwrap();
        let key2 = derive_key("wrong password", &salt, &Default::default()).unwrap();
        assert_ne!(key1, key2);
    }
}
```

### Example: Encryption (`src/encryption.rs`)

```rust
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};

pub fn encrypt_item<T: serde::Serialize>(
    item: &T,
    key: &[u8; 32],
) -> Result<Vec<u8>, CryptoError> {
    // Serialize item to JSON
    let plaintext = serde_json::to_vec(item)?;
    
    // Generate random nonce (24 bytes)
    let nonce = generate_nonce();
    
    // Encrypt with AEAD
    let cipher = ChaCha20Poly1305::new(key.into());
    let ciphertext = cipher.encrypt(nonce.into(), plaintext.as_ref())?;
    
    // Return: [nonce (24 bytes) || ciphertext]
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_item<T: serde::de::DeserializeOwned>(
    data: &[u8],
    key: &[u8; 32],
) -> Result<T, CryptoError> {
    if data.len() < 24 {
        return Err(CryptoError::DecryptionFailed);
    }
    
    // Extract nonce
    let nonce = &data[0..24];
    let ciphertext = &data[24..];
    
    // Decrypt with AEAD
    let cipher = ChaCha20Poly1305::new(key.into());
    let plaintext = cipher.decrypt(nonce.into(), ciphertext)?;
    
    // Deserialize
    let item: T = serde_json::from_slice(&plaintext)?;
    
    Ok(item)
}
```

## 🔗 References

- **Security Invariants:** Task 2 (SECURITY_INVARIANTS.md)
- **Crypto Stack:** Porkpie Architecture and Coding Plan — Section 3
- **Crate Responsibilities:** Porkpie Architecture and Coding Plan — Section 5

## ✔️ Success Verification

```bash
# Build
cargo build --package porkpie-crypto

# Format
cargo fmt --package porkpie-crypto --check

# Lint (ZERO warnings tolerated)
cargo clippy --package porkpie-crypto -- -D warnings

# All tests pass
cargo test --package porkpie-crypto -- --nocapture

# Security checks
cargo test --package porkpie-crypto tampering
cargo test --package porkpie-crypto wrong_password
cargo test --package porkpie-crypto key_derivation
```

## 🚨 If Blocked...

| Problem | Solution |
|---------|----------|
| "Argon2id is complex" | Use `argon2` crate. Call `Argon2::new(Algorithm::Argon2id, ...)`. Docs are clear. |
| "XChaCha20Poly1305 confuses me" | Use `chacha20poly1305` crate. `.encrypt(key, nonce, plaintext)` returns ciphertext + tag. |
| "How do I test encryption?" | Encrypt plaintext → decrypt ciphertext → verify plaintext matches. Try tampering with 1 ciphertext byte, verify decrypt fails. |
| "Nonce reuse causes crashes" | Track used nonces in memory. On duplicate, return error (don't panic). Or use counter-based nonces. |
| "Zeroize breaks my code" | Use `Zeroizing<Vec<u8>>` instead of `Vec<u8>`. Auto-zeros on drop. No behavior change. |

## 🔒 STRICT TYPECHECK REQUIREMENTS

**Type safety is non-negotiable.** Rust's type system is your first line of defense.

- ✓ **All type errors must compile** — `cargo build` must succeed with zero type errors
- ✓ **No `unsafe` blocks without justification** — Document why in code comment
- ✓ **No unchecked casts** — Use `as` only where necessary (document reasoning)
- ✓ **No `unwrap()` on external input** — Use `.map_err()` or `?` operator
- ✓ **No `todo!()` or `unimplemented!()` in production code** — Only in stubs
- ✓ **Compiler warnings are failures** — `cargo clippy` must have zero warnings
- ✓ **Type inference must be clear** — Add explicit types where ambiguous
- ✓ **Trait bounds must be explicit** — Don't hide requirements

**Verification command:**
```bash
cargo check --workspace
cargo build --workspace
```

**If ANY type error appears, stop and fix it. Type errors = broken code.**

## ⚠️ CRITICAL REVIEW CHECKPOINT

**Before marking complete, verify:**

- [ ] No `panic!()` or `unwrap()` on ciphertext input
- [ ] All test vectors pass (if available)
- [ ] Tamper detection actually fails on modified ciphertext
- [ ] Wrong password produces specific error (not generic)
- [ ] No secrets logged anywhere
- [ ] Memory zeroized for sensitive data
- [ ] All clippy warnings resolved

**If ANY security invariant violated, DO NOT complete. Escalate.**

## 📌 What Comes Next

**Task 5: Vault Core Logic**

Next agent will implement porkpie-core. They'll use encrypt_item/decrypt_item APIs to manage vault lifecycle.

---

**Status:** Ready for agent assignment
