use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHandle {
    pub session_id: String,
    pub country: String,
    pub network_type: String,
    pub session_type: String,
    pub seller_wallet: Option<String>,
    pub reserve_microcredits: i64,
    pub status: String,
}

/// Open a buyer proxy session.
pub async fn open_session(
    http: &reqwest::Client,
    backend_url: &str,
    token: &str,
    country: &str,
    network_type: &str,
    session_type: &str,
    spend_cap: Option<i64>,
) -> anyhow::Result<SessionHandle> {
    let url = format!("{}/v2/sessions", backend_url.trim_end_matches('/'));
    let resp = http
        .post(&url)
        .bearer_auth(token)
        .json(&serde_json::json!({
            "country": country,
            "network_type": network_type,
            "session_type": session_type,
            "spend_cap_microcredits": spend_cap,
        }))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to open session: {}", e))?;

    let handle: SessionHandle = resp.json().await?;
    Ok(handle)
}

/// Close a session and release the reserve.
pub async fn close_session(
    http: &reqwest::Client,
    backend_url: &str,
    token: &str,
    session_id: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/v2/sessions/{}", backend_url.trim_end_matches('/'), session_id);
    http.delete(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to close session: {}", e))?;

    Ok(())
}
