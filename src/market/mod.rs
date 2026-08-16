pub mod auth;
pub mod balances;
pub mod catalog;
pub mod deposits;
pub mod reservations;
pub mod seller_state;
pub mod sessions;
pub mod settlements;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Client for the ProxyBase Markets v2 backend.
pub struct MarketClient {
    pub backend_url: String,
    http: reqwest::Client,
    token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToken {
    pub token: String,
    pub wallet_address: String,
    pub role: String,
}

impl MarketClient {
    pub fn new(backend_url: &str) -> Self {
        Self {
            backend_url: backend_url.to_string(),
            http: reqwest::Client::new(),
            token: None,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    fn bearer(&self) -> String {
        format!("Bearer {}", self.token.as_deref().unwrap_or(""))
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.backend_url.trim_end_matches('/'), path)
    }

    // --- Auth flow ---

    /// Request a challenge nonce from the backend.
    pub async fn request_challenge(&self, wallet_address: &str) -> Result<auth::Challenge> {
        let resp = self
            .http
            .post(self.api_url("/v2/auth/challenge"))
            .json(&serde_json::json!({"wallet_address": wallet_address}))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Challenge request failed: {}", e))?;

        resp.json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse challenge: {}", e))
    }

    /// Verify a signed challenge and get a session token.
    pub async fn verify_challenge(
        &mut self,
        public_key_hex: &str,
        nonce: &str,
        timestamp: &str,
        signature_hex: &str,
    ) -> Result<AuthResult> {
        let resp = self
            .http
            .post(self.api_url("/v2/auth/verify"))
            .json(&serde_json::json!({
                "public_key_hex": public_key_hex,
                "nonce": nonce,
                "timestamp": timestamp,
                "signature_hex": signature_hex,
            }))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Verify request failed: {}", e))?;

        let auth: AuthResult = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse auth response: {}", e))?;

        self.token = Some(auth.session_token.clone());
        Ok(auth)
    }

    // --- Authenticated API calls ---

    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        let resp = self
            .http
            .get(self.api_url(path))
            .header("Authorization", self.bearer())
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    async fn post(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let resp = self
            .http
            .post(self.api_url(path))
            .header("Authorization", self.bearer())
            .json(&body)
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    /// Get wallet balance.
    pub async fn get_balance(&self) -> Result<balances::BalanceState> {
        let v = self.get("/v2/wallet/balance").await?;
        Ok(serde_json::from_value(v)?)
    }

    /// Transfer seller → buyer.
    pub async fn transfer(&self, amount: i64) -> Result<balances::BalanceState> {
        let v = self
            .post(
                "/v2/wallet/transfer",
                serde_json::json!({"amount_microcredits": amount}),
            )
            .await?;
        Ok(serde_json::from_value(v)?)
    }

    /// Create a deposit invoice.
    pub async fn create_deposit(
        &self,
        amount: i64,
        currency: &str,
    ) -> Result<deposits::DepositInvoice> {
        let v = self
            .post(
                "/v2/deposits",
                serde_json::json!({"amount_microcredits": amount, "pay_currency": currency}),
            )
            .await?;
        Ok(serde_json::from_value(v)?)
    }

    /// Get deposit status.
    pub async fn get_deposit(&self, deposit_id: &str) -> Result<serde_json::Value> {
        self.get(&format!("/v2/deposits/{}", deposit_id)).await
    }

    /// List available countries.
    pub async fn list_countries(&self) -> Result<Vec<catalog::CountryInfo>> {
        let v = self.get("/v2/catalog/countries").await?;
        let countries: Vec<catalog::CountryInfo> =
            serde_json::from_value(v["countries"].clone()).unwrap_or_default();
        Ok(countries)
    }

    /// List pricing.
    pub async fn list_pricing(&self) -> Result<Vec<catalog::PricingSnapshot>> {
        let v = self.get("/v2/catalog/pricing").await?;
        let pricing: Vec<catalog::PricingSnapshot> =
            serde_json::from_value(v["pricing"].clone()).unwrap_or_default();
        Ok(pricing)
    }

    /// Open a buyer proxy session.
    pub async fn open_session(
        &self,
        country: &str,
        network_type: &str,
        session_type: &str,
        spend_cap: Option<i64>,
    ) -> Result<sessions::SessionHandle> {
        let v = self
            .post(
                "/v2/sessions",
                serde_json::json!({
                    "country": country,
                    "network_type": network_type,
                    "session_type": session_type,
                    "spend_cap_microcredits": spend_cap,
                }),
            )
            .await?;
        Ok(serde_json::from_value(v)?)
    }

    /// Close a session.
    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        let url = self.api_url(&format!("/v2/sessions/{}", session_id));
        self.http
            .delete(&url)
            .header("Authorization", self.bearer())
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Close session failed: {}", e))?;
        Ok(())
    }

    /// Register as a seller (standard compensation mode).
    pub async fn register_seller(&self) -> Result<serde_json::Value> {
        self.register_seller_with_node_type(None).await
    }

    /// Register as a seller with an explicit node type:
    /// "standard" (default) or "volunteer" (donate bandwidth, no earnings).
    pub async fn register_seller_with_node_type(
        &self,
        node_type: Option<&str>,
    ) -> Result<serde_json::Value> {
        self.post(
            "/v2/seller/register",
            serde_json::json!({ "node_type": node_type.unwrap_or("standard") }),
        )
        .await
    }

    /// Get seller status.
    pub async fn seller_status(&self) -> Result<seller_state::SellerStatus> {
        let v = self.get("/v2/seller/status").await?;
        Ok(serde_json::from_value(v)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub session_token: String,
    pub wallet_address: String,
    pub role: String,
    pub buyer_available: i64,
    pub buyer_reserved: i64,
    pub buyer_spent: i64,
    pub seller_pending: i64,
    pub seller_available: i64,
    pub seller_payout_locked: i64,
    pub spendable_balance: i64,
}
