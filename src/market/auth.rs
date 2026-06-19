use anyhow::Result;
use k256::ecdsa::signature::Signer;
use k256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};

use crate::wallet::WalletManager;

#[derive(Debug, Serialize, Deserialize)]
pub struct Challenge {
    pub nonce: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub session_token: String,
    pub wallet_address: String,
    pub role: String,
}

/// Request a challenge nonce from the backend for this wallet address.
pub async fn request_challenge(
    http: &reqwest::Client,
    backend_url: &str,
    wallet_address: &str,
) -> Result<Challenge> {
    let url = format!("{}/v2/auth/challenge", backend_url.trim_end_matches('/'));
    let resp = http
        .post(&url)
        .json(&serde_json::json!({"wallet_address": wallet_address}))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Challenge request failed: {}", e))?;

    let challenge: Challenge = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse challenge: {}", e))?;

    Ok(challenge)
}

/// Sign the challenge and verify with the backend to get a session token.
pub async fn sign_and_verify(
    http: &reqwest::Client,
    backend_url: &str,
    wallet: &WalletManager,
    vk: &VerifyingKey,
    challenge: &Challenge,
) -> Result<AuthResponse> {
    let wallet_addr = wallet
        .address()
        .ok_or_else(|| anyhow::anyhow!("Wallet not loaded"))?;

    // Message: wallet_address:nonce:timestamp
    let message = format!("{}:{}:{}", wallet_addr, challenge.nonce, challenge.timestamp);
    let signature = wallet.sign(message.as_bytes())?;

    let pk_hex = hex::encode(vk.to_sec1_bytes());
    let sig_hex = hex::encode(&signature);

    let url = format!("{}/v2/auth/verify", backend_url.trim_end_matches('/'));
    let resp = http
        .post(&url)
        .json(&serde_json::json!({
            "public_key_hex": pk_hex,
            "nonce": challenge.nonce,
            "timestamp": challenge.timestamp,
            "signature_hex": sig_hex,
        }))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Verify request failed: {}", e))?;

    let auth: AuthResponse = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse auth response: {}", e))?;

    Ok(auth)
}
