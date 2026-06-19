pub mod bandwidth;
pub mod throttle;

use bandwidth::BandwidthCounter;
use throttle::RateLimiter;

/// Resource governor — bandwidth tracking, daily caps, and rate limiting.
pub struct ResourceGovernor {
    pub bandwidth: BandwidthCounter,
    pub throttle: RateLimiter,
    max_concurrent_streams: u32,
}

impl ResourceGovernor {
    pub fn new(
        max_bandwidth_bytes_per_day: i64,
        max_speed_bytes_per_sec: i64,
        max_concurrent_streams: u32,
    ) -> Self {
        Self {
            bandwidth: BandwidthCounter::new(max_bandwidth_bytes_per_day),
            throttle: RateLimiter::new(max_speed_bytes_per_sec),
            max_concurrent_streams,
        }
    }

    pub fn max_concurrent_streams(&self) -> u32 {
        self.max_concurrent_streams
    }
}
