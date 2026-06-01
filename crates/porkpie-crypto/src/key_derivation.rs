use crate::errors::CryptoError;
use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
use secrecy::Secret;

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
    salt: &[u8; 32],
    params: &Argon2Params,
) -> Result<[u8; 32], CryptoError> {
    use secrecy::ExposeSecret;

    let argon2_params = ParamsBuilder::new()
        .m_cost(params.mem_cost)
        .t_cost(params.time_cost)
        .p_cost(params.parallelism)
        .build()
        .map_err(|e| CryptoError::Argon2Error(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.expose_secret().as_bytes(), salt, &mut key)
        .map_err(|e| CryptoError::Argon2Error(e.to_string()))?;

    Ok(key)
}
