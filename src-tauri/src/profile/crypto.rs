use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sha2::{Sha256, Digest};
use rand::RngCore;
use crate::error::AppError;

const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

/// Derive a key from a password using SHA-256
fn derive_key_from_password(password: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.finalize().to_vec()
}

/// Get or create the encryption key for profiles
pub fn get_or_create_key() -> Result<Vec<u8>, AppError> {
    // For production, this should be stored securely in the OS keyring
    // For now, we'll use a derived key from a fixed passphrase
    // TODO: Store this in the keyring properly
    let master_password = "dataforge_profile_encryption_key_v1";
    Ok(derive_key_from_password(master_password))
}

/// Encrypt data using AES-256-GCM
pub fn encrypt(data: &[u8]) -> Result<String, AppError> {
    let key_bytes = get_or_create_key()?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..KEY_SIZE]);
    let cipher = Aes256Gcm::new(&key);

    // Generate a random nonce
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Encrypt the data
    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| AppError::Encryption(format!("Failed to encrypt data: {}", e)))?;

    // Combine nonce and ciphertext
    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);

    // Encode as base64
    Ok(BASE64.encode(combined))
}

/// Decrypt data using AES-256-GCM
pub fn decrypt(encrypted_data: &str) -> Result<Vec<u8>, AppError> {
    // Decode from base64
    let combined = BASE64
        .decode(encrypted_data)
        .map_err(|e| AppError::Encryption(format!("Failed to decode base64: {}", e)))?;

    if combined.len() < NONCE_SIZE {
        return Err(AppError::Encryption("Invalid encrypted data".to_string()));
    }

    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let key_bytes = get_or_create_key()?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..KEY_SIZE]);
    let cipher = Aes256Gcm::new(&key);

    // Decrypt the data
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Encryption(format!("Failed to decrypt data: {}", e)))
}

/// Generate a random salt
pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let original = b"Hello, World! This is a secret message.";

        let encrypted = encrypt(original).expect("Failed to encrypt");
        assert_ne!(BASE64.encode(original), encrypted);

        let decrypted = decrypt(&encrypted).expect("Failed to decrypt");
        assert_eq!(original.to_vec(), decrypted);
    }

    #[test]
    fn test_encrypt_different_nonces() {
        let data = b"Test data";

        let encrypted1 = encrypt(data).expect("Failed to encrypt");
        let encrypted2 = encrypt(data).expect("Failed to encrypt");

        // Same data encrypted twice should produce different results due to random nonces
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same original data
        let decrypted1 = decrypt(&encrypted1).expect("Failed to decrypt");
        let decrypted2 = decrypt(&encrypted2).expect("Failed to decrypt");

        assert_eq!(decrypted1, decrypted2);
        assert_eq!(decrypted1, data.to_vec());
    }

    #[test]
    fn test_invalid_encrypted_data() {
        let result = decrypt("invalid_base64!");
        assert!(result.is_err());

        let result = decrypt("dG9vc2hvcnQ="); // "tooshort" in base64
        assert!(result.is_err());
    }
}