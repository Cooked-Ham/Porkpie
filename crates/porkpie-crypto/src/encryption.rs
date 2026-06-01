use crate::constants::NONCE_LEN;
use crate::errors::CryptoError;
use crate::nonce::generate_nonce;
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use zeroize::Zeroizing;

pub fn encrypt_item<T: serde::Serialize>(
    item: &T,
    key: &[u8; 32],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let plaintext: Zeroizing<Vec<u8>> = Zeroizing::new(serde_json::to_vec(item)?);

    let nonce_bytes = generate_nonce();
    let nonce = XNonce::from_slice(&nonce_bytes);

    let cipher = XChaCha20Poly1305::new(key.into());
    let payload = Payload {
        msg: &plaintext,
        aad,
    };
    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|_| CryptoError::EncryptionFailed)?;

    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt_item<T: serde::de::DeserializeOwned>(
    ciphertext_with_nonce: &[u8],
    key: &[u8; 32],
    aad: &[u8],
) -> Result<T, CryptoError> {
    if ciphertext_with_nonce.len() < NONCE_LEN {
        return Err(CryptoError::InvalidCiphertext);
    }

    let (nonce_bytes, ciphertext) = ciphertext_with_nonce.split_at(NONCE_LEN);
    let nonce = XNonce::from_slice(nonce_bytes);

    let cipher = XChaCha20Poly1305::new(key.into());
    let payload = Payload {
        msg: ciphertext,
        aad,
    };
    let plaintext: Zeroizing<Vec<u8>> = Zeroizing::new(
        cipher
            .decrypt(nonce, payload)
            .map_err(|_| CryptoError::DecryptionFailed)?,
    );

    let item = serde_json::from_slice(&plaintext)?;

    Ok(item)
}
