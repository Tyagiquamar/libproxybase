use anyhow::{Context, Result};
use k256::ecdsa::{SigningKey, VerifyingKey};

/// Generate a new secp256k1 keypair.
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let sk = SigningKey::random(&mut rand::thread_rng());
    let vk = VerifyingKey::from(&sk);
    (sk, vk)
}

/// Derive a keypair from a seed. Uses the first 32 bytes as the private key.
pub fn seed_to_keypair(seed: &[u8]) -> Result<(SigningKey, VerifyingKey)> {
    let key_bytes = &seed[..32];
    let sk = SigningKey::from_slice(key_bytes)
        .context("Invalid seed for secp256k1 keypair")?;
    let vk = VerifyingKey::from(&sk);
    Ok((sk, vk))
}

/// Derive an Ethereum-style wallet address from a public key.
///
/// Uses keccak256 of the uncompressed public key (without the 0x04 prefix),
/// taking the last 20 bytes.
pub fn public_key_to_address(vk: &VerifyingKey) -> Result<String> {
    use sha3::{Digest, Keccak256};

    let uncompressed = vk.to_encoded_point(false);
    let pub_key_bytes = uncompressed.as_bytes();

    // keccak256(x || y) — skip the 0x04 prefix
    let hash = Keccak256::digest(&pub_key_bytes[1..]);

    // Last 20 bytes
    let address_bytes = &hash[hash.len() - 20..];

    Ok(format!("0x{}", hex::encode(address_bytes)))
}

/// Sign a message with the given signing key.
pub fn sign(sk: &SigningKey, message: &[u8]) -> Result<Vec<u8>> {
    use k256::ecdsa::signature::Signer;
    let signature: k256::ecdsa::Signature = sk.sign(message);
    Ok(signature.to_vec())
}

/// Verify a signature against a public key.
pub fn verify(vk: &VerifyingKey, message: &[u8], signature: &[u8]) -> Result<bool> {
    use k256::ecdsa::signature::Verifier;
    use k256::ecdsa::Signature;
    let sig = Signature::from_slice(signature)
        .context("Invalid ECDSA signature")?;
    Ok(vk.verify(message, &sig).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_roundtrip() {
        let (sk, vk) = generate_keypair();
        let message = b"proxybase market auth";
        let sig = sign(&sk, message).unwrap();
        assert!(verify(&vk, message, &sig).unwrap());
    }

    #[test]
    fn test_address_determinism() {
        let (sk, vk) = generate_keypair();
        let addr1 = public_key_to_address(&vk).unwrap();
        let addr2 = public_key_to_address(&vk).unwrap();
        assert_eq!(addr1, addr2);
    }
}
