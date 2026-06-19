use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Lock-free byte counter with daily cap enforcement and at-capacity signaling.
pub struct BandwidthCounter {
    bytes_today: AtomicU64,
    daily_cap: i64,
    last_reset_day: AtomicU64,
    /// Set to true when the daily cap is reached — backend should stop routing.
    at_capacity: Arc<AtomicBool>,
}

impl BandwidthCounter {
    pub fn new(daily_cap: i64) -> Self {
        Self {
            bytes_today: AtomicU64::new(0),
            daily_cap,
            last_reset_day: AtomicU64::new(Self::day_now()),
            at_capacity: Arc::new(AtomicBool::new(false)),
        }
    }

    fn day_now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 86400
    }

    /// Reset counter if it's a new day.
    fn maybe_reset(&self) {
        let today = Self::day_now();
        let last = self.last_reset_day.load(Ordering::Relaxed);
        if today != last {
            self.last_reset_day.store(today, Ordering::Relaxed);
            self.bytes_today.store(0, Ordering::Relaxed);
            self.at_capacity.store(false, Ordering::Relaxed);
        }
    }

    /// Record bytes transferred. Returns true if cap was reached.
    pub fn record(&self, bytes: u64) -> bool {
        self.maybe_reset();
        let total = self.bytes_today.fetch_add(bytes, Ordering::Relaxed) + bytes;

        if total as i64 >= self.daily_cap && !self.at_capacity.load(Ordering::Relaxed) {
            self.at_capacity.store(true, Ordering::Relaxed);
            tracing::warn!(
                "Daily bandwidth cap reached: {} / {} bytes",
                total, self.daily_cap
            );
            return true;
        }
        false
    }

    /// Check if the seller has reached daily capacity.
    /// The backend should stop routing new sessions when this is true.
    pub fn is_at_capacity(&self) -> bool {
        self.maybe_reset();
        self.at_capacity.load(Ordering::Relaxed)
            || (self.bytes_today.load(Ordering::Relaxed) as i64 >= self.daily_cap)
    }

    /// Get bytes used today.
    pub fn bytes_used_today(&self) -> u64 {
        self.maybe_reset();
        self.bytes_today.load(Ordering::Relaxed)
    }

    /// Check if within 80% of daily cap — should send early warning.
    pub fn is_near_capacity(&self) -> bool {
        let used = self.bytes_used_today() as f64;
        let cap = self.daily_cap as f64;
        cap > 0.0 && used / cap >= 0.8
    }

    /// Get the at_capacity flag as an Arc<AtomicBool> for external monitoring.
    pub fn capacity_flag(&self) -> Arc<AtomicBool> {
        self.at_capacity.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording() {
        let c = BandwidthCounter::new(1_000_000);
        c.record(500_000);
        assert!(!c.is_at_capacity());
        assert_eq!(c.bytes_used_today(), 500_000);

        let capped = c.record(500_000);
        assert!(capped);
        assert!(c.is_at_capacity());
    }

    #[test]
    fn test_near_capacity() {
        let c = BandwidthCounter::new(1_000_000);
        c.record(800_000);
        assert!(c.is_near_capacity());
        assert!(!c.is_at_capacity());
    }
}
