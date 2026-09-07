pub mod dialer;
pub mod multiplexer;
pub mod pool;
pub mod reconnect;
pub mod seller_protocol;
pub mod transport;

/// Network engine orchestrating transport, multiplexer, and dialer.
pub struct NetworkEngine;

impl NetworkEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NetworkEngine {
    fn default() -> Self {
        Self::new()
    }
}
