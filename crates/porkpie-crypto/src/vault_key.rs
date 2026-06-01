use crate::constants::NONCE_LEN;
use crate::errors::CryptoError;
use crate::nonce::generate_nonce;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use zeroize::Zeroizing;

pub fn wrap_vault_key(master_key: &[u8; 32], vault_key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
    let nonce_bytes = generate_nonce();
    let nonce = XNonce::from_slice(&nonce_bytes);

    let cipher = XChaCha20Poly1305::new(master_key.into());
    let ciphertext = cipher
        .encrypt(nonce, vault_key.as_ref())
        .map_err(|_| CryptoError::EncryptionFailed)?;

    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn unwrap_vault_key(master_key: &[u8; 32], wrapped: &[u8]) -> Result<[u8; 32], CryptoError> {
    if wrapped.len() < NONCE_LEN {
        return Err(CryptoError::InvalidCiphertext);
    }

    let (nonce_bytes, ciphertext) = wrapped.split_at(NONCE_LEN);
    let nonce = XNonce::from_slice(nonce_bytes);

    let cipher = XChaCha20Poly1305::new(master_key.into());
    let plaintext: Zeroizing<Vec<u8>> = Zeroizing::new(
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::WrongPassword)?,
    );

    if plaintext.len() != 32 {
        return Err(CryptoError::InvalidCiphertext);
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&plaintext);

    Ok(key)
}
