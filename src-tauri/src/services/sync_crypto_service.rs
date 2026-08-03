use crate::error::CliplyError;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;

pub const KDF_ALGORITHM: &str = "argon2id";
pub const CIPHER_ALGORITHM: &str = "AES-256-GCM";
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const KEY_LEN: usize = 32;

const ARGON2_MEMORY_KIB: u32 = 19_456;
const ARGON2_ITERATIONS: u32 = 2;
const ARGON2_PARALLELISM: u32 = 1;

// Argon2id with 19 MiB memory takes tens of milliseconds per derivation and
// runs on every encrypt/decrypt. Auto sync encrypts every cycle and imports
// several snapshots per cycle, so derived keys are memoized per
// (password, salt, params). Keys are keyed by a password digest, never the
// password itself, and the cache is bounded.
const KEY_CACHE_MAX_ENTRIES: usize = 32;
static KEY_CACHE: Mutex<Option<HashMap<KeyCacheId, [u8; KEY_LEN]>>> = Mutex::new(None);

// Exports reuse one process-lifetime salt so repeated auto-sync cycles hit the
// key cache. The salt is still random per process and still written into every
// envelope, so the on-disk format and old readers are unaffected.
static EXPORT_SALT: Mutex<Option<[u8; SALT_LEN]>> = Mutex::new(None);

type KeyCacheId = ([u8; 16], Vec<u8>, u32, u32, u32);

fn key_cache_id(
    password: &str,
    salt: &[u8],
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> KeyCacheId {
    let digest = Sha256::digest(password.as_bytes());
    let mut password_digest = [0_u8; 16];
    password_digest.copy_from_slice(&digest[..16]);
    (
        password_digest,
        salt.to_vec(),
        memory_kib,
        iterations,
        parallelism,
    )
}

fn cached_or_derive_key(
    password: &str,
    salt: &[u8],
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<[u8; KEY_LEN], CliplyError> {
    let cache_id = key_cache_id(password, salt, memory_kib, iterations, parallelism);
    if let Ok(cache) = KEY_CACHE.lock() {
        if let Some(key) = cache.as_ref().and_then(|map| map.get(&cache_id)) {
            return Ok(*key);
        }
    }

    let key = derive_key_uncached(password, salt, memory_kib, iterations, parallelism)?;
    if let Ok(mut cache) = KEY_CACHE.lock() {
        let map = cache.get_or_insert_with(HashMap::new);
        if map.len() >= KEY_CACHE_MAX_ENTRIES {
            map.clear();
        }
        map.insert(cache_id, key);
    }
    Ok(key)
}

fn export_salt() -> [u8; SALT_LEN] {
    if let Ok(mut slot) = EXPORT_SALT.lock() {
        if let Some(salt) = *slot {
            return salt;
        }
        let salt = random_bytes::<SALT_LEN>();
        *slot = Some(salt);
        return salt;
    }

    random_bytes::<SALT_LEN>()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncEncryptionMetadata {
    pub cipher: String,
    pub kdf: String,
    pub salt: String,
    pub nonce: String,
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

pub fn encrypt_payload(
    payload: &[u8],
    password: &str,
) -> Result<(SyncEncryptionMetadata, String), CliplyError> {
    validate_password(password)?;

    let salt = export_salt();
    let nonce = random_bytes::<NONCE_LEN>();
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| CliplyError::Sync("failed to initialize sync cipher".to_string()))?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), payload)
        .map_err(|_| CliplyError::Sync("同步包加密失败".to_string()))?;

    Ok((
        SyncEncryptionMetadata {
            cipher: CIPHER_ALGORITHM.to_string(),
            kdf: KDF_ALGORITHM.to_string(),
            salt: BASE64.encode(salt),
            nonce: BASE64.encode(nonce),
            memory_kib: ARGON2_MEMORY_KIB,
            iterations: ARGON2_ITERATIONS,
            parallelism: ARGON2_PARALLELISM,
        },
        BASE64.encode(ciphertext),
    ))
}

pub fn decrypt_payload(
    metadata: &SyncEncryptionMetadata,
    encrypted_payload: &str,
    password: &str,
) -> Result<Vec<u8>, CliplyError> {
    validate_password(password)?;
    validate_metadata(metadata)?;

    let salt = BASE64
        .decode(&metadata.salt)
        .map_err(|_| CliplyError::Sync("同步包格式不正确".to_string()))?;
    let nonce = BASE64
        .decode(&metadata.nonce)
        .map_err(|_| CliplyError::Sync("同步包格式不正确".to_string()))?;
    if salt.len() != SALT_LEN || nonce.len() != NONCE_LEN {
        return Err(CliplyError::Sync("同步包格式不正确".to_string()));
    }

    let ciphertext = BASE64
        .decode(encrypted_payload)
        .map_err(|_| CliplyError::Sync("同步包格式不正确".to_string()))?;
    let key = derive_key_with_params(
        password,
        &salt,
        metadata.memory_kib,
        metadata.iterations,
        metadata.parallelism,
    )?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| CliplyError::Sync("failed to initialize sync cipher".to_string()))?;

    cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| CliplyError::Sync("密码错误或同步包已损坏".to_string()))
}

fn validate_password(password: &str) -> Result<(), CliplyError> {
    if password.trim().is_empty() {
        return Err(CliplyError::Sync("同步密码不能为空".to_string()));
    }

    Ok(())
}

fn validate_metadata(metadata: &SyncEncryptionMetadata) -> Result<(), CliplyError> {
    if metadata.cipher != CIPHER_ALGORITHM || metadata.kdf != KDF_ALGORITHM {
        return Err(CliplyError::Sync("同步包版本或加密方式不兼容".to_string()));
    }

    Ok(())
}

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], CliplyError> {
    derive_key_with_params(
        password,
        salt,
        ARGON2_MEMORY_KIB,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
    )
}

fn derive_key_with_params(
    password: &str,
    salt: &[u8],
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<[u8; KEY_LEN], CliplyError> {
    cached_or_derive_key(password, salt, memory_kib, iterations, parallelism)
}

fn derive_key_uncached(
    password: &str,
    salt: &[u8],
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<[u8; KEY_LEN], CliplyError> {
    let params = Params::new(memory_kib, iterations, parallelism, Some(KEY_LEN))
        .map_err(|_| CliplyError::Sync("同步包 KDF 参数不兼容".to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0_u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|_| CliplyError::Sync("同步密码派生失败".to_string()))?;
    Ok(key)
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0_u8; N];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::{decrypt_payload, encrypt_payload};

    #[test]
    fn encrypt_decrypt_roundtrip_keeps_payload_secret() {
        let payload = br#"{"items":[{"normalizedText":"hello"}]}"#;
        let (metadata, encrypted) =
            encrypt_payload(payload, "sync-password").expect("payload should encrypt");

        assert_eq!(metadata.cipher, "AES-256-GCM");
        assert!(!encrypted.contains("hello"));

        let decrypted = decrypt_payload(&metadata, &encrypted, "sync-password")
            .expect("payload should decrypt");
        assert_eq!(decrypted, payload);
    }

    #[test]
    fn wrong_password_fails() {
        let payload = b"secret";
        let (metadata, encrypted) =
            encrypt_payload(payload, "right-password").expect("payload should encrypt");

        let error = decrypt_payload(&metadata, &encrypted, "wrong-password")
            .expect_err("wrong password should fail");
        assert!(error.to_string().contains("密码错误"));
    }
}
