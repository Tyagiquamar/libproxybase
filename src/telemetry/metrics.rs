use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub struct MetricsSnapshot {
    pub active_streams: u32,
    pub bytes_today: u64,
    pub uptime_seconds: u64,
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub bandwidth_mbps: f64,
}

impl MetricsSnapshot {
    pub fn collect() -> Self {
        use sysinfo::System;
        let sys = System::new_all();

        let cpu = sys.global_cpu_usage() as f64;
        let mem_mb = sys.used_memory() as f64 / 1_048_576.0;

        Self {
            cpu_percent: cpu,
            memory_mb: mem_mb,
            ..Default::default()
        }
    }
}
