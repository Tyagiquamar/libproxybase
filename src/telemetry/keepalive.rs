use std::time::Duration;
use tokio::time::interval;

/// Send a WebSocket PING every 30 seconds to prevent NAT timeouts.
pub async fn run_keepalive(send_ping: impl Fn() -> bool) {
    let mut tick = interval(Duration::from_secs(30));
    loop {
        tick.tick().await;
        if !send_ping() {
            tracing::debug!("Keepalive ping failed, connection lost");
            break;
        }
    }
}
