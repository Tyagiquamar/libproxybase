use anyhow::{Context, Result};
use k256::ecdsa::{SigningKey, VerifyingKey};
use std::path::Path;

/// Encrypt a signing key with a password and save to disk.
///
/// File format: [32-byte salt][12-byte nonce][ciphertext][16-byte tag]
pub fn encrypt_and_save(sk: &SigningKey, password: &str, path: &Path) -> Result<()> {
    use aes_gcm::aead::{Aead, KeyInit, OsRng};
    use aes_gcm::{Aes256Gcm, Nonce};
    use argon2::Argon2;
    use rand::Rng;

    let key_bytes = sk.to_bytes();
    let key_be_bytes: &[u8] = key_bytes.as_slice();

    // Derive encryption key via Argon2id
    let salt: [u8; 32] = rand::thread_rng().gen();
    let mut enc_key = [0u8; 32];
    let argon2 = Argon2::default();
    argon2
        .hash_password_into(password.as_bytes(), &salt, &mut enc_key)
        .map_err(|e| anyhow::anyhow!("Argon2 error: {}", e))?;

    // Encrypt with AES-256-GCM
    let cipher = Aes256Gcm::new_from_slice(&enc_key)
        .map_err(|e| anyhow::anyhow!("AES init error: {}", e))?;
    let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, key_be_bytes)
        .map_err(|e| anyhow::anyhow!("Encryption error: {}", e))?;

    // Write: salt || nonce || ciphertext
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut output = Vec::new();
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    std::fs::write(path, output).context("Failed to write encrypted keyfile")?;
    Ok(())
}

/// Load and decrypt a signing key from an encrypted keyfile.
pub fn load_and_decrypt(password: &str, path: &Path) -> Result<(SigningKey, VerifyingKey)> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    use argon2::Argon2;

    let data = std::fs::read(path)
        .with_context(|| format!("Failed to read keyfile at {:?}", path))?;

    if data.len() < 32 + 12 + 16 {
        anyhow::bail!("Keyfile too short (corrupted?)");
    }

    let salt = &data[..32];
    let nonce_bytes = &data[32..44];
    let ciphertext = &data[44..];

    // Re-derive key via Argon2id
    let mut enc_key = [0u8; 32];
    let argon2 = Argon2::default();
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut enc_key)
        .map_err(|_| anyhow::anyhow!("Decryption failed — wrong password or corrupted file"))?;

    // Decrypt
    let cipher = Aes256Gcm::new_from_slice(&enc_key)
        .map_err(|e| anyhow::anyhow!("AES init error: {}", e))?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Decryption failed — wrong password or corrupted file"))?;

    let sk = SigningKey::from_slice(&plaintext)
        .context("Invalid private key bytes")?;
    let vk = VerifyingKey::from(&sk);

    Ok((sk, vk))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::keypair;
    use tempfile::tempdir;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (sk, _vk) = keypair::generate_keypair();
        let dir = tempdir().unwrap();
        let path = dir.path().join("wallet").join("keyfile.enc");

        encrypt_and_save(&sk, "correct-horse-battery-staple", &path).unwrap();

        let (loaded_sk, _loaded_vk) =
            load_and_decrypt("correct-horse-battery-staple", &path).unwrap();

        assert_eq!(
            sk.to_bytes().as_slice(),
            loaded_sk.to_bytes().as_slice()
        );

        // Wrong password
        let result = load_and_decrypt("wrong-password", &path);
        assert!(result.is_err());
    }
}
