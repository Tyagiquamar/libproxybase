use anyhow::Result;
use std::sync::atomic::{AtomicU32, Ordering};

/// Stream multiplexer with capacity enforcement and yamux integration.
///
/// Manages concurrent stream count over a yamux-multiplexed WebSocket connection.
/// The yamux Connection is created elsewhere; the Multiplexer tracks active stream
/// count and enforces the SellerBusy cap.
pub struct Multiplexer {
    active_streams: AtomicU32,
    max_streams: u32,
}

impl Multiplexer {
    pub fn new(max_streams: u32) -> Self {
        Self {
            active_streams: AtomicU32::new(0),
            max_streams,
        }
    }

    pub fn is_at_capacity(&self) -> bool {
        self.active_streams.load(Ordering::Relaxed) >= self.max_streams
    }

    /// Try to reserve a stream slot. Returns OK if under the cap.
    pub fn try_open_stream(&self) -> Result<()> {
        let current = self.active_streams.fetch_add(1, Ordering::Relaxed);
        if current >= self.max_streams {
            self.active_streams.fetch_sub(1, Ordering::Relaxed);
            anyhow::bail!(
                "SellerBusy: at capacity ({}/{})",
                current,
                self.max_streams
            );
        }
        Ok(())
    }

    /// Close a stream, releasing its slot.
    pub fn close_stream(&self) {
        self.active_streams.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn active_count(&self) -> u32 {
        self.active_streams.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capacity_enforcement() {
        let mux = Multiplexer::new(2);
        assert!(!mux.is_at_capacity());

        mux.try_open_stream().unwrap();
        mux.try_open_stream().unwrap();
        assert!(mux.is_at_capacity());

        assert!(mux.try_open_stream().is_err());

        mux.close_stream();
        assert!(!mux.is_at_capacity());
    }
}
