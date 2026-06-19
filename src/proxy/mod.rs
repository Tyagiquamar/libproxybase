pub mod listener;
pub mod socks5;

use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// Local SOCKS5 proxy server for buyer applications.
pub struct ProxyServer {
    port: Option<u16>,
    handle: Option<JoinHandle<()>>,
}

impl ProxyServer {
    pub fn new() -> Self {
        Self {
            port: None,
            handle: None,
        }
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Start the local SOCKS5 proxy on an available port.
    /// Accepts connections and relays them through the provided relay function.
    pub async fn start(
        &mut self,
        relay_fn: impl Fn(tokio::net::TcpStream) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync + 'static,
    ) -> Result<u16> {
        let port = listener::find_open_port().await?;
        let listener = listener::bind_and_serve(port).await?;
        self.port = Some(port);

        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        tracing::debug!("SOCKS5 connection from {}", addr);
                        let relay = relay_fn(stream);
                        tokio::spawn(async move {
                            if let Err(e) = relay.await {
                                tracing::debug!("SOCKS5 relay error from {}: {:?}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("SOCKS5 accept error: {:?}", e);
                        break;
                    }
                }
            }
        });

        self.handle = Some(handle);
        tracing::info!("Local SOCKS5 proxy started on 127.0.0.1:{}", port);
        Ok(port)
    }

    /// Stop the proxy server.
    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
        self.port = None;
    }
}

impl Default for ProxyServer {
    fn default() -> Self {
        Self::new()
    }
}
