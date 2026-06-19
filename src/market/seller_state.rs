use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerStatus {
    pub wallet_address: String,
    pub pool: String,
    pub country: Option<String>,
    pub proxy_category: Option<String>,
    pub active_streams: u32,
    pub max_streams: u32,
    pub qos_uptime: Option<f64>,
    pub qos_latency_ms: Option<f64>,
    pub qos_throughput_mbps: Option<f64>,
}

pub async fn get_seller_status(
    http: &reqwest::Client,
    backend_url: &str,
    token: &str,
) -> anyhow::Result<SellerStatus> {
    let url = format!("{}/v2/seller/status", backend_url.trim_end_matches('/'));
    let resp = http
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get seller status: {}", e))?;

    Ok(resp.json().await?)
}

pub async fn register_as_seller(
    http: &reqwest::Client,
    backend_url: &str,
    token: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/v2/seller/register", backend_url.trim_end_matches('/'));
    http.post(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to register as seller: {}", e))?;

    Ok(())
}
