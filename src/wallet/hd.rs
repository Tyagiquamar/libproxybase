//! BIP-32 hierarchical deterministic key derivation (BIP-44 secp256k1 paths).
//!
//! Fleet nodes derive distinct wallet identities from one master mnemonic via
//! `m/44'/60'/0'/0/{index}`, so N containers get N distinct addresses with a
//! single 12/24-word backup phrase.

use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use k256::ecdsa::{SigningKey, VerifyingKey};
use k256::elliptic_curve::ff::{Field, PrimeField};
use k256::{FieldBytes, Scalar};
use sha2::Sha512;

type HmacSha512 = Hmac<Sha512>;

/// BIP-44 path components: m/44'/60'/0'/0/{index}
const BIP44_PURPOSE: u32 = 44;
const BIP44_COIN_TYPE: u32 = 60;
const BIP44_ACCOUNT: u32 = 0;
const BIP44_CHANGE: u32 = 0;
const HARDENED_FLAG: u32 = 0x8000_0000;

/// Derive a secp256k1 keypair from a 64-byte BIP-39 seed using the
/// BIP-44 path `m/44'/60'/0'/0/{index}`.
pub fn derive_bip44_keypair(seed: &[u8; 64], index: u32) -> Result<(SigningKey, VerifyingKey)> {
    let path = [
        BIP44_PURPOSE | HARDENED_FLAG, // 44'
        BIP44_COIN_TYPE | HARDENED_FLAG, // 60' (EVM-compatible)
        BIP44_ACCOUNT | HARDENED_FLAG, // 0'
        BIP44_CHANGE,                  // 0 (external chain)
        index,                         // i (node ordinal)
    ];
    derive_path(seed, &path)
}

/// Derive a keypair from a seed along an arbitrary BIP-32 path.
/// Each path element is the child index; bit 31 set means hardened.
pub fn derive_path(seed: &[u8], path: &[u32]) -> Result<(SigningKey, VerifyingKey)> {
    // Master key derivation: HMAC-SHA512(key = "Bitcoin seed", data = seed)
    let mut mac = HmacSha512::new_from_slice(b"Bitcoin seed")
        .map_err(|_| anyhow::anyhow!("HMAC-SHA512 init failed"))?;
    mac.update(seed);
    let master = mac.finalize().into_bytes();

    let mut priv_key = master[..32].to_vec();
    let mut chain_code = master[32..].to_vec();

    for &child_index in path {
        let (next_priv, next_chain) = derive_child_step(&priv_key, &chain_code, child_index)?;
        priv_key = next_priv;
        chain_code = next_chain;
    }

    let sk = SigningKey::from_slice(&priv_key)
        .context("Derived private key is invalid for secp256k1 (zero or >= n); try another index")?;
    let vk = VerifyingKey::from(&sk);
    Ok((sk, vk))
}

/// One BIP-32 CKD step: (parent_key, parent_chain, index) → (child_key, child_chain).
fn derive_child_step(
    parent_key: &[u8],
    parent_chain: &[u8],
    index: u32,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut mac = HmacSha512::new_from_slice(parent_chain)
        .map_err(|_| anyhow::anyhow!("HMAC-SHA512 init failed"))?;

    if index & HARDENED_FLAG != 0 {
        // Hardened: data = 0x00 || ser256(parent_key) || ser32(index)
        mac.update(&[0x00]);
        mac.update(parent_key);
    } else {
        // Normal: data = serP(point(parent_key)) || ser32(index) (compressed point)
        let parent_sk = SigningKey::from_slice(parent_key)
            .context("Invalid parent key in BIP-32 derivation")?;
        let point = parent_sk.verifying_key().to_encoded_point(true);
        mac.update(point.as_bytes());
    }
    mac.update(&index.to_be_bytes());

    let result = mac.finalize().into_bytes();
    let il = &result[..32];
    let ir = &result[32..];

    // Child key = (IL + parent_key) mod n. IL >= n or a zero result are
    // astronomically rare per BIP-32; surface them instead of silently
    // producing a broken key.
    let il_scalar: Scalar = Option::<Scalar>::from(Scalar::from_repr(*FieldBytes::from_slice(il)))
        .ok_or_else(|| anyhow::anyhow!("BIP-32: IL >= n — invalid child, increment the index"))?;
    let parent_scalar: Scalar =
        Option::<Scalar>::from(Scalar::from_repr(*FieldBytes::from_slice(parent_key)))
            .ok_or_else(|| anyhow::anyhow!("BIP-32: parent key >= n — invalid"))?;
    let child_scalar = il_scalar.add(&parent_scalar);
    if child_scalar.is_zero().unwrap_u8() == 1 {
        anyhow::bail!("BIP-32: derived zero child key — invalid, increment the index");
    }

    Ok((child_scalar.to_bytes().to_vec(), ir.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::{keypair, mnemonic};

    fn sk_hex(sk: &SigningKey) -> String {
        hex::encode(sk.to_bytes())
    }

    /// BIP-32 spec test vector 1: seed 000102030405060708090a0b0c0d0e0f
    /// The spec's 128-bit seed, used verbatim (not zero-padded).
    fn vector1_seed() -> Vec<u8> {
        (0x00..=0x0f).collect()
    }

    #[test]
    fn test_bip32_vector1_master() {
        let (sk, _vk) = derive_path(&vector1_seed(), &[]).unwrap();
        assert_eq!(
            sk_hex(&sk),
            "e8f32e723decf4051aefac8e2c93c9c5b214313817cdb01a1494b917c8436b35"
        );
    }

    #[test]
    fn test_bip32_vector1_hardened() {
        let (sk, _vk) = derive_path(&vector1_seed(), &[0 | HARDENED_FLAG]).unwrap();
        assert_eq!(
            sk_hex(&sk),
            "edb2e14f9ee77d26dd93b4ecede8d16ed408ce149b6cd80b0715a2d911a0afea"
        );
    }

    #[test]
    fn test_bip32_vector1_mixed_path() {
        // m/0'/1/2'/2
        let path = [0 | HARDENED_FLAG, 1, 2 | HARDENED_FLAG, 2];
        let (sk, _vk) = derive_path(&vector1_seed(), &path).unwrap();
        assert_eq!(
            sk_hex(&sk),
            "0f479245fb19a38a1954c5c7c0ebab2f9bdfd96a17563ef28a6a4b1a2a764ef4"
        );
    }

    #[test]
    fn test_bip32_vector1_deep_path() {
        // m/0'/1/2'/2/1000000000
        let path = [
            0 | HARDENED_FLAG,
            1,
            2 | HARDENED_FLAG,
            2,
            1_000_000_000,
        ];
        let (sk, _vk) = derive_path(&vector1_seed(), &path).unwrap();
        assert_eq!(
            sk_hex(&sk),
            "471b76e389e528d6de6d816857e012c5455051cad6660850e58372a6c3e6e7c8"
        );
    }

    #[test]
    fn test_bip44_abandon_about_known_addresses() {
        // Canonical "abandon ... about" mnemonic, standard Ethereum BIP-44 accounts.
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = mnemonic::mnemonic_to_seed(phrase, "").unwrap();

        let expected = [
            (0u32, "0x9858effd232b4033e47d90003d41ec34ecaeda94"),
            (1, "0x6fac4d18c912343bf86fa7049364dd4e424ab9c0"),
            (2, "0xb6716976a3ebe8d39aceb04372f22ff8e6802d7a"),
        ];
        for (index, addr) in expected {
            let (_sk, vk) = derive_bip44_keypair(&seed, index).unwrap();
            assert_eq!(keypair::public_key_to_address(&vk).unwrap(), addr);
        }
    }

    #[test]
    fn test_bip44_same_index_is_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = mnemonic::mnemonic_to_seed(phrase, "").unwrap();

        let (sk1, _vk1) = derive_bip44_keypair(&seed, 7).unwrap();
        let (sk2, _vk2) = derive_bip44_keypair(&seed, 7).unwrap();
        assert_eq!(sk_hex(&sk1), sk_hex(&sk2));
    }

    #[test]
    fn test_bip44_distinct_indices_are_distinct() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = mnemonic::mnemonic_to_seed(phrase, "").unwrap();

        let mut seen = std::collections::HashSet::new();
        for index in 0..16u32 {
            let (sk, _vk) = derive_bip44_keypair(&seed, index).unwrap();
            assert!(seen.insert(sk_hex(&sk)), "index {} collided", index);
        }
    }
}
