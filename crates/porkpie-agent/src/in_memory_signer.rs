//! In-memory Ed25519 signer for testing and future agent integration.

use crate::signer::{SignerError, SshSigner};
use ed25519_dalek::{Signer as Ed25519SignerTrait, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

/// An in-memory Ed25519 signer that holds a raw signing key.
///
/// This is intended for tests and for the future agent path where the
/// private key is decrypted into unlocked memory.  It does **not**
/// parse OpenSSH key formats; callers must supply the 32-byte raw seed.
#[derive(Debug, Clone)]
pub struct Ed25519Signer {
    keypair: SigningKey,
    public_key_bytes: [u8; 32],
}

impl Ed25519Signer {
    /// Create a new random Ed25519 signer.
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let keypair = SigningKey::generate(&mut csprng);
        let public_key_bytes = *keypair.verifying_key().as_bytes();
        Self {
            keypair,
            public_key_bytes,
        }
    }

    /// Create a signer from a 32-byte raw seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let keypair = SigningKey::from_bytes(seed);
        let public_key_bytes = *keypair.verifying_key().as_bytes();
        Self {
            keypair,
            public_key_bytes,
        }
    }

    /// Return the 32-byte verifying (public) key.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.keypair.verifying_key()
    }
}

impl SshSigner for Ed25519Signer {
    fn algorithm(&self) -> &'static str {
        "ssh-ed25519"
    }

    fn public_key_bytes(&self) -> &[u8] {
        &self.public_key_bytes
    }

    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SignerError> {
        let signature = self.keypair.sign(data);
        Ok(signature.to_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Verifier;

    #[test]
    fn generated_signer_produces_valid_signature() {
        let signer = Ed25519Signer::generate();
        let data = b"hello ssh agent";
        let signature = signer.sign(data).expect("sign should succeed");

        let verifying_key = signer.verifying_key();
        let sig = ed25519_dalek::Signature::from_bytes(
            &signature.try_into().expect("signature is 64 bytes"),
        );
        verifying_key
            .verify(data, &sig)
            .expect("signature should verify");
    }

    #[test]
    fn signer_from_seed_is_deterministic() {
        let seed = [42u8; 32];
        let signer_a = Ed25519Signer::from_seed(&seed);
        let signer_b = Ed25519Signer::from_seed(&seed);
        assert_eq!(signer_a.public_key_bytes(), signer_b.public_key_bytes());
    }

    #[test]
    fn signer_trait_algorithm_is_ssh_ed25519() {
        let signer = Ed25519Signer::generate();
        assert_eq!(signer.algorithm(), "ssh-ed25519");
    }

    #[test]
    fn signer_trait_public_key_matches_verifying_key() {
        let signer = Ed25519Signer::generate();
        let trait_bytes = signer.public_key_bytes();
        let vk = signer.verifying_key();
        let vk_bytes = vk.as_bytes();
        assert_eq!(trait_bytes, vk_bytes);
    }

    #[test]
    fn different_signers_have_different_public_keys() {
        let signer_a = Ed25519Signer::generate();
        let signer_b = Ed25519Signer::generate();
        assert_ne!(signer_a.public_key_bytes(), signer_b.public_key_bytes());
    }
}
