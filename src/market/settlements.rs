use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageReceipt {
    pub event_id: String,
    pub session_id: String,
    pub pricing_version: String,
    pub buyer_wallet: String,
    pub seller_wallet: String,
    pub sequence_no: u64,
    pub bytes_up: u64,
    pub bytes_down: u64,
    pub cumulative_billable_bytes: u64,
    pub observed_at: String,
    pub acknowledged: bool,
}

impl UsageReceipt {
    pub fn new(
        session_id: &str,
        pricing_version: &str,
        buyer_wallet: &str,
        seller_wallet: &str,
        sequence_no: u64,
        bytes_up: u64,
        bytes_down: u64,
        cumulative: u64,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            pricing_version: pricing_version.to_string(),
            buyer_wallet: buyer_wallet.to_string(),
            seller_wallet: seller_wallet.to_string(),
            sequence_no,
            bytes_up,
            bytes_down,
            cumulative_billable_bytes: cumulative,
            observed_at: chrono::Utc::now().to_rfc3339(),
            acknowledged: false,
        }
    }
}

/// Persisted store of unacknowledged usage receipts for replay on reconnect.
pub struct ReceiptStore {
    path: PathBuf,
    receipts: Vec<UsageReceipt>,
}

impl ReceiptStore {
    pub fn new(data_dir: &std::path::Path) -> Self {
        let path = data_dir.join("unacked_receipts.json");
        let receipts = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Self { path, receipts }
    }

    /// Add a new receipt to the unacknowledged store.
    pub fn add(&mut self, receipt: UsageReceipt) {
        self.receipts.push(receipt);
        self.persist();
    }

    /// Replay all unacknowledged receipts to the backend.
    /// Returns the list of receipts that need replaying.
    pub fn unacknowledged(&self) -> &[UsageReceipt] {
        &self.receipts
    }

    /// Mark receipts up to a given sequence_no as acknowledged.
    pub fn acknowledge_up_to(&mut self, session_id: &str, sequence_no: u64) {
        self.receipts.retain(|r| {
            !(r.session_id == session_id && r.sequence_no <= sequence_no)
        });
        self.persist();
    }

    /// Mark a specific receipt as acknowledged by event_id.
    pub fn acknowledge(&mut self, event_id: &str) {
        self.receipts.retain(|r| r.event_id != event_id);
        self.persist();
    }

    fn persist(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string(&self.receipts) {
            let _ = fs::write(&self.path, json);
        }
    }
}

/// Replay unacknowledged usage receipts after reconnect.
pub async fn replay_unacknowledged(
    client: &crate::market::MarketClient,
    store: &ReceiptStore,
) -> Result<()> {
    for receipt in store.unacknowledged() {
        tracing::info!(
            "Replaying receipt {} (session={}, seq={})",
            receipt.event_id,
            receipt.session_id,
            receipt.sequence_no
        );

        // Submit usage to backend. In production this would be a dedicated
        // usage submission endpoint. For now, the session settlement job
        // handles accumulation from the session's cumulative_bytes counter.
        tracing::debug!("Receipt replayed: {} bytes total", receipt.cumulative_billable_bytes);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_receipt_store_persist() {
        let dir = tempdir().unwrap();
        let mut store = ReceiptStore::new(dir.path());

        let receipt = UsageReceipt::new("sess-1", "v1", "buyer", "seller", 1, 1000, 2000, 3000);
        store.add(receipt);
        assert_eq!(store.unacknowledged().len(), 1);

        store.acknowledge_up_to("sess-1", 1);
        assert_eq!(store.unacknowledged().len(), 0);
    }
}
