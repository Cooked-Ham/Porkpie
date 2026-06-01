use crate::errors::CryptoError;
use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
use secrecy::{ExposeSecret, Secret};
use zeroize::Zeroizing;

pub struct Argon2Params {
    pub time_cost: u32,
    pub mem_cost: u32,
    pub parallelism: u32,
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
    password: &Secret<String>,
    secret_key: &[u8; 32],
    salt: &[u8; 32],
    params: &Argon2Params,
) -> Result<[u8; 32], CryptoError> {
    let argon2_params = ParamsBuilder::new()
        .m_cost(params.mem_cost)
        .t_cost(params.time_cost)
        .p_cost(params.parallelism)
        .build()
        .map_err(|e| CryptoError::Argon2Error(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);

    let combined =
        Zeroizing::new([password.expose_secret().as_bytes(), secret_key.as_ref()].concat());

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(combined.as_slice(), salt, &mut key)
        .map_err(|e| CryptoError::Argon2Error(e.to_string()))?;

    Ok(key)
}
