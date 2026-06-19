use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Outbound connection pool with TLS session reuse.
pub struct ConnectionPool {
    max_per_host: u32,
    pools: Arc<Mutex<HashMap<String, Vec<PoolEntry>>>>,
}

struct PoolEntry {
    _created_at: std::time::Instant,
}

impl ConnectionPool {
    pub fn new(max_per_host: u32) -> Self {
        Self {
            max_per_host,
            pools: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn evict_idle(&self) {
        let mut pools = self.pools.lock().await;
        for entries in pools.values_mut() {
            entries.retain(|e| e._created_at.elapsed().as_secs() < 120);
        }
    }
}
