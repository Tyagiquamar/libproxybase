use rand::Rng;
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Serialize)]
pub struct HeartbeatPayload {
    pub cpu_percent: f64,
    pub available_bandwidth_mbps: f64,
    pub active_streams: u32,
    pub bytes_today: u64,
    pub wallet_balance_microcredits: i64,
    pub seller_status: String,
    pub assigned_country: Option<String>,
    pub assigned_category: Option<String>,
    pub uptime_seconds: u64,
    pub version: String,
}

/// Adaptive heartbeat — send only when idle for 60s, with ±20% jitter.
pub async fn run_heartbeat(
    mut get_payload: impl FnMut() -> HeartbeatPayload,
    mut send: impl FnMut(&HeartbeatPayload),
) {
    loop {
        // Wait 60s ± 20% jitter
        let mut rng = rand::thread_rng();
        let jitter: f64 = rng.gen_range(-0.2..0.2);
        let wait_secs = 60.0 * (1.0 + jitter);
        sleep(Duration::from_secs_f64(wait_secs.max(1.0))).await;

        let payload = get_payload();
        send(&payload);
    }
}
