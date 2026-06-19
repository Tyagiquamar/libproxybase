pub mod watcher;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub wallet: WalletConfig,
    pub market: MarketConfig,
    pub buyer: BuyerConfig,
    pub seller: SellerConfig,
    pub limits: LimitsConfig,
    pub pool: PoolConfig,
    pub engine: EngineConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConfig {
    pub backend_url: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyerConfig {
    pub default_country: String,
    pub default_proxy_category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerConfig {
    pub accept_trial_pool: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    pub max_bandwidth_bytes_per_day: i64,
    pub max_speed_bytes_per_sec: i64,
    pub max_concurrent_streams: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub max_connections_per_host: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub pin_to_core: bool,
    pub log_level: String,
}

impl Config {
    /// Load config from the default path (~/.proxybase/config.toml) or create default.
    pub async fn load_or_create_default() -> Result<Self> {
        let config_dir = dirs_next();
        let config_path = config_dir.join("config.toml");

        if config_path.exists() {
            Self::load(&config_path)
        } else {
            let config = Config::default_config();
            config.save(&config_path)?;
            Ok(config)
        }
    }

    fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {:?}", path))?;
        toml::from_str(&content).context("Failed to parse config TOML")
    }

    fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(path, content).context("Failed to write config file")?;
        Ok(())
    }

    fn default_config() -> Self {
        Self {
            wallet: WalletConfig {
                data_dir: dirs_next(),
            },
            market: MarketConfig {
                backend_url: "wss://gateway.proxybase.io".to_string(),
                role: "buyer".to_string(),
            },
            buyer: BuyerConfig {
                default_country: "US".to_string(),
                default_proxy_category: "residential".to_string(),
            },
            seller: SellerConfig {
                accept_trial_pool: true,
            },
            limits: LimitsConfig {
                max_bandwidth_bytes_per_day: 2_000_000_000,
                max_speed_bytes_per_sec: 5_000_000,
                max_concurrent_streams: 200,
            },
            pool: PoolConfig {
                max_connections_per_host: 16,
            },
            engine: EngineConfig {
                pin_to_core: false,
                log_level: "info".to_string(),
            },
        }
    }
}

fn dirs_next() -> PathBuf {
    dirs_next_crate()
}

fn dirs_next_crate() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".proxybase")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default_config();
        assert_eq!(config.market.role, "buyer");
        assert_eq!(config.buyer.default_country, "US");
        assert_eq!(config.limits.max_concurrent_streams, 200);
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let config = Config::default_config();
        config.save(&path).unwrap();
        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.market.backend_url, config.market.backend_url);
    }
}
