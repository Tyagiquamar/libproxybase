use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryInfo {
    pub country: String,
    pub seller_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingSnapshot {
    pub country: String,
    pub proxy_category: String,
    pub buyer_price_microcredits_per_gb: i64,
    pub seller_credit_microcredits_per_gb: i64,
    pub version: String,
}

/// Fetch available countries from the backend.
pub async fn fetch_countries(
    http: &reqwest::Client,
    backend_url: &str,
    token: &str,
) -> anyhow::Result<Vec<CountryInfo>> {
    let url = format!("{}/v2/catalog/countries", backend_url.trim_end_matches('/'));
    let resp = http
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch countries: {}", e))?;

    let body: serde_json::Value = resp.json().await?;
    let countries: Vec<CountryInfo> = serde_json::from_value(body["countries"].clone())
        .unwrap_or_default();

    Ok(countries)
}

/// Fetch pricing for all buckets.
pub async fn fetch_pricing(
    http: &reqwest::Client,
    backend_url: &str,
    token: &str,
) -> anyhow::Result<Vec<PricingSnapshot>> {
    let url = format!("{}/v2/catalog/pricing", backend_url.trim_end_matches('/'));
    let resp = http
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch pricing: {}", e))?;

    let body: serde_json::Value = resp.json().await?;
    let pricing: Vec<PricingSnapshot> = serde_json::from_value(body["pricing"].clone())
        .unwrap_or_default();

    Ok(pricing)
}
