# libproxybase

`libproxybase` is the core Rust library for ProxyBase.

It provides reusable modules for:
- configuration loading and watching
- wallet and key management (BIP-39 mnemonics, encrypted keystores,
  BIP-32/BIP-44 HD child derivation at `m/44'/60'/0'/0/{index}` via
  `wallet::hd` / `WalletManager::import_hd`)
- market authentication and session APIs
- network transport, pooling, and reconnect logic
- SOCKS5/local proxy components
- telemetry, QoS, and heartbeat reporting
- resource governance and throttling

## Installation

```toml
[dependencies]
libproxybase = "0.4.0"
```

## Notes

This crate is designed to be embedded by ProxyBase applications such as CLI, backend, and GUI/Tauri integrations.
