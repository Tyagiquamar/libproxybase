use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

/// Exponential backoff with jitter: 1s → 2s → 4s → ... → 60s max, ±20% jitter.
pub struct Backoff {
    current: u64,
    max: u64,
}

impl Backoff {
    pub fn new() -> Self {
        Self {
            current: 1,
            max: 60,
        }
    }

    /// Wait for the next backoff period and return the duration waited.
    pub async fn wait(&mut self) -> Duration {
        let mut rng = rand::thread_rng();
        let jitter: f64 = rng.gen_range(-0.2..0.2);
        let duration_secs = (self.current as f64 * (1.0 + jitter)).max(0.5);
        let duration = Duration::from_secs_f64(duration_secs);

        sleep(duration).await;

        self.current = (self.current * 2).min(self.max);
        duration
    }

    /// Reset backoff to initial value (after a successful connection).
    pub fn reset(&mut self) {
        self.current = 1;
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_progression() {
        let mut b = Backoff::new();
        assert_eq!(b.current, 1);
        b.current = 2;
        b.current = (b.current * 2).min(b.max);
        assert_eq!(b.current, 4);
        b.current = (b.current * 2).min(b.max);
        assert_eq!(b.current, 8);
        b.current = 30;
        b.current = (b.current * 2).min(b.max);
        assert_eq!(b.current, 60); // clamped to max
    }
}
