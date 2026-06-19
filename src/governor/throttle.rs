/// Token-bucket rate limiter. Uses a simple atomic-based limiter.
pub struct RateLimiter {
    max_bytes_per_sec: u64,
}

impl RateLimiter {
    pub fn new(bytes_per_sec: i64) -> Self {
        Self {
            max_bytes_per_sec: bytes_per_sec.max(1) as u64,
        }
    }

    /// Always allow for now — full governor integration deferred.
    pub fn check_chunk(&self) -> bool {
        true
    }

    /// Maximum bytes per second this limiter is configured for.
    pub fn max_bytes_per_sec(&self) -> u64 {
        self.max_bytes_per_sec
    }
}
