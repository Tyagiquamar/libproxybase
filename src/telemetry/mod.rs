pub mod heartbeat;
pub mod keepalive;
pub mod metrics;
pub mod qos;

/// Telemetry worker that spawns keepalive, heartbeat, and QoS reporting tasks.
pub struct TelemetryWorker;

impl TelemetryWorker {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TelemetryWorker {
    fn default() -> Self {
        Self::new()
    }
}
