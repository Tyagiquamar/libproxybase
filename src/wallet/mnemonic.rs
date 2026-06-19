use anyhow::Result;

/// Generate a BIP-39 mnemonic with the given word count (12 or 24).
pub fn generate_mnemonic(word_count: usize) -> Result<bip39::Mnemonic> {
    if word_count != 12 && word_count != 24 {
        anyhow::bail!("Word count must be 12 or 24, got {}", word_count);
    }
    Ok(bip39::Mnemonic::generate(word_count)
        .map_err(|e| anyhow::anyhow!("Failed to generate mnemonic: {:?}", e))?)
}

/// Derive a seed from a mnemonic phrase and optional passphrase.
/// In bip39 v2, `to_seed()` returns `[u8; 64]` directly.
pub fn mnemonic_to_seed(phrase: &str, passphrase: &str) -> Result<[u8; 64]> {
    let mnemonic = bip39::Mnemonic::parse(phrase)
        .map_err(|e| anyhow::anyhow!("Invalid mnemonic: {:?}", e))?;
    Ok(mnemonic.to_seed(passphrase))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_12_words() {
        let mnemonic = generate_mnemonic(12).unwrap();
        let phrase = mnemonic.to_string();
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 12);
    }

    #[test]
    fn test_seed_derivation_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed1 = mnemonic_to_seed(phrase, "").unwrap();
        let seed2 = mnemonic_to_seed(phrase, "").unwrap();
        assert_eq!(seed1, seed2);
    }
}
