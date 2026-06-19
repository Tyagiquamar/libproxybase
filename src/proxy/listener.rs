use anyhow::Result;
use tokio::net::TcpListener;

/// Find an open port starting from 1080, going upward.
pub async fn find_open_port() -> Result<u16> {
    for port in 1080..1200 {
        if TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .is_ok()
        {
            return Ok(port);
        }
    }
    anyhow::bail!("No open ports found in range 1080–1200")
}

/// Bind the SOCKS5 proxy to the given port and serve connections.
pub async fn bind_and_serve(port: u16) -> Result<TcpListener> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind SOCKS5 listener on port {}: {}", port, e))?;

    tracing::info!("Local SOCKS5 proxy listening on 127.0.0.1:{}", port);
    Ok(listener)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_open_port() {
        let port = find_open_port().await.unwrap();
        assert!(port >= 1080 && port < 1200);
    }
}
