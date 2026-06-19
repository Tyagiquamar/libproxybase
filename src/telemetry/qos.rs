use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct QosReport {
    pub latency_ms: f64,
    pub throughput_mbps: f64,
    pub error_rate: f64,
    pub timestamp: String,
}

/// Report the local view of backend-initiated QoS probe results.
pub fn report_qos(latency_ms: f64, throughput_mbps: f64, error_rate: f64) -> QosReport {
    QosReport {
        latency_ms,
        throughput_mbps,
        error_rate,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}
