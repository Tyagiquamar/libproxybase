pub mod hd;
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

    fn keyfile_path(&self) -> PathBuf {
        self.data_dir.join("wallet").join("keyfile.enc")
    }

    /// True when an encrypted keyfile exists on disk, regardless of whether
    /// it can be decrypted without the user's password.
    pub fn exists(&self) -> bool {
        self.keyfile_path().exists()
    }

    /// Generate a new wallet (12-word mnemonic), encrypt keyfile, return mnemonic.
    pub fn create(&mut self, password: &str) -> Result<String> {
        let mnemonic = mnemonic::generate_mnemonic(12)?;
        let phrase = mnemonic.to_string();
        let seed = mnemonic::mnemonic_to_seed(&phrase, "")?;
        let (sk, pk) = keypair::seed_to_keypair(&seed)?;

        keystore::encrypt_and_save(&sk, password, &self.keyfile_path())?;

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

        keystore::encrypt_and_save(&sk, password, &self.keyfile_path())?;

        self.signing_key = Some(sk);
        self.wallet_address = Some(keypair::public_key_to_address(&pk)?);

        Ok(())
    }

    /// Import from a master mnemonic with a specific BIP-44 HD child index:
    /// `m/44'/60'/0'/0/{index}`. Each index yields a distinct wallet address,
    /// so a fleet of nodes can share one master phrase without collisions.
    ///
    /// NOTE: this derives keys differently from [`Self::import`]/[`Self::create`]
    /// (which use the raw seed prefix), so the same phrase produces different
    /// addresses across the two methods. Both are intentional; `import_hd` is
    /// the standard for new fleet deployments.
    pub fn import_hd(&mut self, mnemonic_phrase: &str, index: u32, password: &str) -> Result<()> {
        let seed = mnemonic::mnemonic_to_seed(mnemonic_phrase, "")?;
        let (sk, pk) = hd::derive_bip44_keypair(&seed, index)?;

        keystore::encrypt_and_save(&sk, password, &self.keyfile_path())?;

        self.signing_key = Some(sk);
        self.wallet_address = Some(keypair::public_key_to_address(&pk)?);

        Ok(())
    }

    /// Load wallet from encrypted keyfile.
    pub fn load(&mut self, password: &str) -> Result<()> {
        let (sk, pk) = keystore::load_and_decrypt(password, &self.keyfile_path())?;

        self.signing_key = Some(sk);
        self.wallet_address = Some(keypair::public_key_to_address(&pk)?);

        Ok(())
    }

    /// Try each candidate password in order; the first one that decrypts the
    /// keyfile wins. Fails only when none of them work.
    pub fn try_load(&mut self, passwords: &[&str]) -> Result<()> {
        for pw in passwords {
            if self.load(pw).is_ok() {
                return Ok(());
            }
        }
        anyhow::bail!("Wallet could not be decrypted with any of the provided passwords")
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

    #[test]
    fn test_import_hd_matches_direct_derivation() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let dir = tempdir().unwrap();

        let mut wm = WalletManager::new(dir.path().to_path_buf()).unwrap();
        wm.import_hd(phrase, 3, "pw").unwrap();

        // Cross-check against the raw hd/keypair modules.
        let seed = mnemonic::mnemonic_to_seed(phrase, "").unwrap();
        let (_sk, vk) = hd::derive_bip44_keypair(&seed, 3).unwrap();
        let expected = keypair::public_key_to_address(&vk).unwrap();
        assert_eq!(wm.address().unwrap(), expected);

        // Reload from the encrypted keyfile yields the same identity.
        let mut wm2 = WalletManager::new(dir.path().to_path_buf()).unwrap();
        wm2.load("pw").unwrap();
        assert_eq!(wm2.address().unwrap(), expected);
    }

    #[test]
    fn test_import_hd_distinct_indices() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mut wm0 = WalletManager::new(tempdir().unwrap().path().to_path_buf()).unwrap();
        wm0.import_hd(phrase, 0, "pw").unwrap();
        let mut wm1 = WalletManager::new(tempdir().unwrap().path().to_path_buf()).unwrap();
        wm1.import_hd(phrase, 1, "pw").unwrap();
        assert_ne!(wm0.address(), wm1.address());
    }

    #[test]
    fn test_import_hd_differs_from_legacy_import() {
        // Legacy import uses the raw seed prefix; HD import uses BIP-44.
        // Same phrase, intentionally different addresses.
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mut legacy = WalletManager::new(tempdir().unwrap().path().to_path_buf()).unwrap();
        legacy.import(phrase, "pw").unwrap();
        let mut hd = WalletManager::new(tempdir().unwrap().path().to_path_buf()).unwrap();
        hd.import_hd(phrase, 0, "pw").unwrap();
        assert_ne!(legacy.address(), hd.address());
    }

    #[test]
    fn test_exists_flags_wallet_presence_without_decrypting() {
        let dir = tempdir().unwrap();
        let mut wm = WalletManager::new(dir.path().to_path_buf()).unwrap();
        assert!(!wm.exists(), "no keyfile yet");

        // Password-protected wallet must still be reported as present —
        // callers (GUI wallet_info) must not need the password to know
        // a wallet exists.
        wm.create("secret").unwrap();
        assert!(wm.exists());

        let wm2 = WalletManager::new(dir.path().to_path_buf()).unwrap();
        assert!(wm2.exists(), "fresh manager must see the keyfile on disk");
    }

    #[test]
    fn test_try_load_password_wallet() {
        let dir = tempdir().unwrap();
        let mut wm = WalletManager::new(dir.path().to_path_buf()).unwrap();
        wm.create("secret").unwrap();
        let addr = wm.address().unwrap().to_string();

        // Wrong-only candidates fail
        let mut wm2 = WalletManager::new(dir.path().to_path_buf()).unwrap();
        assert!(wm2.try_load(&["wrong", "nope"]).is_err());

        // Correct candidate anywhere in the list succeeds
        let mut wm3 = WalletManager::new(dir.path().to_path_buf()).unwrap();
        wm3.try_load(&["wrong", "secret"]).unwrap();
        assert_eq!(wm3.address().unwrap(), addr);
    }

    #[test]
    fn test_try_load_empty_password_wallet() {
        let dir = tempdir().unwrap();
        let mut wm = WalletManager::new(dir.path().to_path_buf()).unwrap();
        wm.create("").unwrap();

        let mut wm2 = WalletManager::new(dir.path().to_path_buf()).unwrap();
        wm2.try_load(&[""]).unwrap();
        assert!(wm2.is_loaded());
    }
}
