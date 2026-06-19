use anyhow::Result;
use serde::Deserialize;

/// Compile-time embedded secp256k1 release public key for binary verification.
const RELEASE_PUBLIC_KEY: &str = match option_env!("PROXYBASE_RELEASE_PUBKEY") {
    Some(key) => key,
    None => "000000000000000000000000000000000000000000000000000000000000000000",
};

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub prerelease: bool,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

/// Poll GitHub Releases for the latest version.
pub async fn check_latest(owner: &str, repo: &str) -> Result<GitHubRelease> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );

    let client = reqwest::Client::builder()
        .user_agent("libproxybase-auto-updater/0.1")
        .build()?;

    let release: GitHubRelease = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch releases: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse release: {}", e))?;

    Ok(release)
}

/// Download a binary, verify its secp256k1 signature, and return the data.
pub async fn download_and_verify(url: &str, signature_url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .user_agent("libproxybase-auto-updater/0.1")
        .build()?;

    // Download binary
    let binary = client
        .get(url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Download failed: {}", e))?
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("Download read failed: {}", e))?;

    // Download signature
    let signature_hex = client
        .get(signature_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Signature download failed: {}", e))?
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("Signature read failed: {}", e))?;

    let signature_hex = signature_hex.trim();

    // Verify secp256k1 signature
    let pk_bytes = hex::decode(RELEASE_PUBLIC_KEY.trim_start_matches("0x"))
        .map_err(|e| anyhow::anyhow!("Invalid release public key: {}", e))?;

    let verifying_key = k256::ecdsa::VerifyingKey::from_sec1_bytes(&pk_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid release public key: {}", e))?;

    let sig_bytes = hex::decode(signature_hex)
        .map_err(|e| anyhow::anyhow!("Invalid signature hex: {}", e))?;

    let signature = k256::ecdsa::Signature::from_slice(&sig_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid ECDSA signature: {}", e))?;

    use k256::ecdsa::signature::Verifier;
    verifying_key
        .verify(&binary, &signature)
        .map_err(|e| anyhow::anyhow!("Binary signature verification FAILED: {}", e))?;

    tracing::info!("Binary signature verified successfully ({} bytes)", binary.len());
    Ok(binary.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_key_embedded() {
        let key = RELEASE_PUBLIC_KEY;
        assert!(!key.is_empty());
    }
}
