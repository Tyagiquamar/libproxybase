pub mod keystore;
pub mod keypair;
pub mod mnemonic;

use anyhow::Result;
use k256::ecdsa::SigningKey;
use std::path::PathBuf;

/// Manages wallet identity — secp256k1 keypair, BIP-39 mnemonic, encrypted storage.
pub struct WalletManager {
    pub data_dir: PathBuf,
    signing_key: Option<SigningKey>,
    wallet_address: Option<String>,
}

impl WalletManager {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            data_dir,
            signing_key: None,
            wallet_address: None,
        })
    }

    /// Generate a new wallet (12-word mnemonic), encrypt keyfile, return mnemonic.
    pub fn create(&mut self, password: &str) -> Result<String> {
        let mnemonic = mnemonic::generate_mnemonic(12)?;
        let phrase = mnemonic.to_string();
        let seed = mnemonic::mnemonic_to_seed(&phrase, "")?;
        let (sk, pk) = keypair::seed_to_keypair(&seed)?;

        let keyfile_path = self.data_dir.join("wallet").join("keyfile.enc");
        keystore::encrypt_and_save(&sk, password, &keyfile_path)?;

        self.signing_key = Some(sk);
        let address = keypair::public_key_to_address(&pk)?;
        self.wallet_address = Some(address.clone());

        Ok(mnemonic.to_string())
    }

    /// Import from an existing mnemonic. `password` is for keystore encryption,
    /// not the BIP-39 passphrase (which defaults to empty).
    pub fn import(&mut self, phrase: &str, password: &str) -> Result<()> {
        let seed = mnemonic::mnemonic_to_seed(phrase, "")?;
        let (sk, pk) = keypair::seed_to_keypair(&seed)?;

        let keyfile_path = self.data_dir.join("wallet").join("keyfile.enc");
        keystore::encrypt_and_save(&sk, password, &keyfile_path)?;

        self.signing_key = Some(sk);
        self.wallet_address = Some(keypair::public_key_to_address(&pk)?);

        Ok(())
    }

    /// Load wallet from encrypted keyfile.
    pub fn load(&mut self, password: &str) -> Result<()> {
        let keyfile_path = self.data_dir.join("wallet").join("keyfile.enc");
        let (sk, pk) = keystore::load_and_decrypt(password, &keyfile_path)?;

        self.signing_key = Some(sk);
        self.wallet_address = Some(keypair::public_key_to_address(&pk)?);

        Ok(())
    }

    /// Sign a message with the loaded wallet key.
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        let sk = self
            .signing_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Wallet not loaded"))?;
        use k256::ecdsa::signature::Signer;
        let sig: k256::ecdsa::Signature = sk.sign(message);
        Ok(sig.to_vec())
    }

    /// Get the wallet address (keccak256-derived from public key).
    pub fn address(&self) -> Option<&str> {
        self.wallet_address.as_deref()
    }

    /// Get the SEC1-encoded public key hex (used for backend auth verification).
    pub fn public_key_hex(&self) -> Option<String> {
        self.signing_key.as_ref().map(|sk| {
            let vk = sk.verifying_key();
            hex::encode(vk.to_sec1_bytes())
        })
    }

    pub fn is_loaded(&self) -> bool {
        self.signing_key.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_load_wallet() {
        let dir = tempdir().unwrap();

        let mut wm = WalletManager::new(dir.path().to_path_buf()).unwrap();
        let mnemonic = wm.create("test-password").unwrap();
        assert!(wm.is_loaded());
        assert!(wm.address().is_some());

        // Sign a message
        let sig = wm.sign(b"hello").unwrap();
        assert!(!sig.is_empty());

        // Reload
        let mut wm2 = WalletManager::new(dir.path().to_path_buf()).unwrap();
        wm2.load("test-password").unwrap();
        assert_eq!(wm.address(), wm2.address());
    }

    #[test]
    fn test_import_mnemonic() {
        let dir = tempdir().unwrap();
        let mut wm = WalletManager::new(dir.path().to_path_buf()).unwrap();
        let mnemonic = wm.create("pw").unwrap();

        let mut wm2 = WalletManager::new(tempdir().unwrap().path().to_path_buf()).unwrap();
        wm2.import(&mnemonic, "new-pw").unwrap();
        // Different passwords, same mnemonic → same address
        assert_eq!(wm.address(), wm2.address());
    }
}
