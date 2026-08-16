use serde::{Deserialize, Serialize};

fn default_node_type() -> String {
    "standard".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerStatus {
    pub wallet_address: String,
    /// Compensation mode: "standard" or "volunteer".
    /// Defaulted so old backends that don't report it still deserialize.
    #[serde(default = "default_node_type")]
    pub node_type: String,
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

/// Register as a seller. `node_type` selects the compensation mode:
/// "standard" (default) or "volunteer" (donate bandwidth, no earnings).
/// `None` keeps the historical no-payload behavior.
pub async fn register_as_seller(
    http: &reqwest::Client,
    backend_url: &str,
    token: &str,
    node_type: Option<&str>,
) -> anyhow::Result<()> {
    let url = format!("{}/v2/seller/register", backend_url.trim_end_matches('/'));
    let payload = serde_json::json!({ "node_type": node_type.unwrap_or("standard") });
    http.post(&url)
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to register as seller: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seller_status_node_type_defaults_when_missing() {
        // Old backends omit node_type — deserialization must not fail.
        let json = serde_json::json!({
            "wallet_address": "0xabc",
            "pool": "production",
            "country": "US",
            "proxy_category": "residential",
            "active_streams": 0,
            "max_streams": 200,
            "qos_uptime": 0.99,
            "qos_latency_ms": 50.0,
            "qos_throughput_mbps": 100.0,
        });
        let status: SellerStatus = serde_json::from_value(json).expect("legacy payload must parse");
        assert_eq!(status.node_type, "standard");

        let json = serde_json::json!({
            "wallet_address": "0xabc",
            "node_type": "volunteer",
            "pool": "production",
            "country": null,
            "proxy_category": null,
            "active_streams": 0,
            "max_streams": 200,
            "qos_uptime": null,
            "qos_latency_ms": null,
            "qos_throughput_mbps": null,
        });
        let status: SellerStatus = serde_json::from_value(json).expect("volunteer payload must parse");
        assert_eq!(status.node_type, "volunteer");
    }
}
