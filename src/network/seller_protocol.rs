use serde::{Deserialize, Serialize};

/// Messages sent from the seller client (CLI or GUI) to the backend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum SellerClientMessage {
    /// Legacy Phase 1 single-path registration message.
    #[serde(rename = "path_info")]
    PathInfo { path_id: String },

    /// Phase 2 multiplexed batch path announcement.
    #[serde(rename = "register_multiplex")]
    RegisterMultiplex {
        tunnel_id: String,
        paths: Vec<String>,
    },

    /// Periodic heartbeat across all multiplexed paths on a tunnel.
    #[serde(rename = "multiplex_heartbeat")]
    MultiplexHeartbeat { active_streams: u32 },

    /// Data received from target socket relayed back to the buyer stream.
    #[serde(rename = "relay_response")]
    RelayResponse {
        session_id: String,
        data: String,
    },

    /// Command acknowledgment for sequenced commands (e.g. stream_open).
    #[serde(rename = "cmd_ack")]
    CmdAck { seq: u64 },
}

/// Messages sent from the backend to the seller client (CLI or GUI).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum SellerServerMessage {
    /// Acknowledgment of successful batch registration in multiplexed mode.
    #[serde(rename = "register_multiplex_ack")]
    RegisterMultiplexAck { count: usize },

    /// Request to open a connection to the target host through a specific path.
    #[serde(rename = "stream_open")]
    StreamOpen {
        session_id: String,
        #[serde(default)]
        path_id: Option<String>,
        target_ip: String,
        target_port: u16,
        #[serde(default)]
        target_host: Option<String>,
        #[serde(default)]
        seq: Option<u64>,
    },

    /// Incoming buyer data to be written to the target socket.
    #[serde(rename = "relay_data")]
    RelayData {
        session_id: String,
        data: String,
    },

    /// Instruction to close the target socket for a stream.
    #[serde(rename = "stream_close")]
    StreamClose { session_id: String },
}

/// Partition a collection of items (such as upstream proxy paths) evenly
/// across a target number of shards (tunnels).
pub fn shard_paths<T: Clone>(paths: &[T], max_shards: usize) -> Vec<Vec<T>> {
    if paths.is_empty() {
        return Vec::new();
    }
    let shard_count = max_shards.min(paths.len()).max(1);
    let mut shards: Vec<Vec<T>> = (0..shard_count).map(|_| Vec::new()).collect();
    for (i, p) in paths.iter().enumerate() {
        shards[i % shard_count].push(p.clone());
    }
    shards
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_paths_empty() {
        let empty: Vec<String> = vec![];
        let shards = shard_paths(&empty, 16);
        assert!(shards.is_empty());
    }

    #[test]
    fn test_shard_paths_single() {
        let single = vec!["path_0".to_string()];
        let shards = shard_paths(&single, 16);
        assert_eq!(shards.len(), 1);
        assert_eq!(shards[0], vec!["path_0"]);
    }

    #[test]
    fn test_shard_paths_balanced() {
        let paths: Vec<String> = (0..100).map(|i| format!("path_{}", i)).collect();
        let shards = shard_paths(&paths, 4);
        assert_eq!(shards.len(), 4);
        for s in &shards {
            assert_eq!(s.len(), 25);
        }
        assert_eq!(shards[0][0], "path_0");
        assert_eq!(shards[1][0], "path_1");
        assert_eq!(shards[2][0], "path_2");
        assert_eq!(shards[3][0], "path_3");
    }

    #[test]
    fn test_shard_paths_more_shards_than_items() {
        let paths: Vec<String> = (0..3).map(|i| format!("path_{}", i)).collect();
        let shards = shard_paths(&paths, 16);
        assert_eq!(shards.len(), 3);
        assert_eq!(shards[0], vec!["path_0"]);
        assert_eq!(shards[1], vec!["path_1"]);
        assert_eq!(shards[2], vec!["path_2"]);
    }

    #[test]
    fn test_register_multiplex_serialization_roundtrip() {
        let msg = SellerClientMessage::RegisterMultiplex {
            tunnel_id: "tun_01".to_string(),
            paths: vec!["upstream_0".to_string(), "upstream_1".to_string()],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"register_multiplex""#));
        assert!(json.contains(r#""tunnel_id":"tun_01""#));

        let parsed: SellerClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_stream_open_deserialization() {
        let json = r#"{
            "type": "stream_open",
            "session_id": "sess_42",
            "path_id": "upstream_99",
            "target_ip": "1.2.3.4",
            "target_port": 443,
            "target_host": "example.com"
        }"#;
        let msg: SellerServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            SellerServerMessage::StreamOpen {
                session_id,
                path_id,
                target_ip,
                target_port,
                target_host,
                seq,
            } => {
                assert_eq!(session_id, "sess_42");
                assert_eq!(path_id.as_deref(), Some("upstream_99"));
                assert_eq!(target_ip, "1.2.3.4");
                assert_eq!(target_port, 443);
                assert_eq!(target_host.as_deref(), Some("example.com"));
                assert_eq!(seq, None);
            }
            _ => panic!("Unexpected message type"),
        }
    }
}
