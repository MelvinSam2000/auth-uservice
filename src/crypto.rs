use anyhow::Result;
use argon2::Config;
use argon2::ThreadMode;
use argon2::Variant;
use argon2::Version;
use rand::Rng;

pub struct PasswordHasher(Config<'static>);

impl PasswordHasher {
    pub fn new() -> Self {
        Self(Config {
            variant: Variant::Argon2id,
            version: Version::Version13,
            mem_cost: 1024,
            time_cost: 3,
            lanes: 4,
            thread_mode: ThreadMode::Sequential,
            secret: &[],
            ad: &[],
            hash_length: 32,
        })
    }

    pub fn hash_password(&self, password_raw: &str) -> Result<String> {
        let salt = rand::thread_rng().gen::<[u8; 8]>();
        Ok(argon2::hash_encoded(
            password_raw.as_bytes(),
            &salt,
            &self.0,
        )?)
    }

    pub fn verify_password(&self, password_raw: &str, password_hash: &str) -> Result<bool> {
        Ok(argon2::verify_encoded(
            password_hash,
            password_raw.as_bytes(),
        )?)
    }
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}
