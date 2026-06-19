use serde::{Deserialize, Serialize};

/// Client-side view of active session reserves.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReservationState {
    pub active_sessions: u32,
    pub total_reserved_microcredits: i64,
    pub sessions: Vec<ReservationInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationInfo {
    pub session_id: String,
    pub country: String,
    pub network_type: String,
    pub reserve_microcredits: i64,
    pub cumulative_bytes: i64,
}
