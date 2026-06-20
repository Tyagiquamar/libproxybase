# libproxybase

`libproxybase` is the core Rust library for ProxyBase.

It provides reusable modules for:
- configuration loading and watching
- wallet and key management
- market authentication and session APIs
- network transport, pooling, and reconnect logic
- SOCKS5/local proxy components
- telemetry, QoS, and heartbeat reporting
- resource governance and throttling

## Installation

```toml
[dependencies]
libproxybase = "0.1"
```

## Notes

This crate is designed to be embedded by ProxyBase applications such as CLI, backend, and GUI/Tauri integrations.
