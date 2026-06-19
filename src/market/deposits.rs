use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositInvoice {
    pub deposit_id: String,
    pub payment_id: Option<String>,
    pub pay_address: Option<String>,
    pub pay_currency: Option<String>,
    pub pay_amount: Option<f64>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositStatus {
    pub deposit_id: String,
    pub wallet_address: String,
    pub payment_id: Option<String>,
    pub amount_microcredits: i64,
    pub status: String,
}

/// Create a deposit invoice via the backend.
pub async fn create_deposit(
    http: &reqwest::Client,
    backend_url: &str,
    token: &str,
    amount_microcredits: i64,
    pay_currency: Option<&str>,
) -> anyhow::Result<DepositInvoice> {
    let url = format!("{}/v2/deposits", backend_url.trim_end_matches('/'));
    let resp = http
        .post(&url)
        .bearer_auth(token)
        .json(&serde_json::json!({
            "amount_microcredits": amount_microcredits,
            "pay_currency": pay_currency,
        }))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create deposit: {}", e))?;

    let invoice: DepositInvoice = resp.json().await?;
    Ok(invoice)
}

/// Get the status of a deposit.
pub async fn get_deposit_status(
    http: &reqwest::Client,
    backend_url: &str,
    token: &str,
    deposit_id: &str,
) -> anyhow::Result<DepositStatus> {
    let url = format!("{}/v2/deposits/{}", backend_url.trim_end_matches('/'), deposit_id);
    let resp = http
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get deposit status: {}", e))?;

    let status: DepositStatus = resp.json().await?;
    Ok(status)
}
