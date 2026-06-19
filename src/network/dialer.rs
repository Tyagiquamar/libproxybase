use anyhow::Result;
use std::time::Duration;
use tokio::net::TcpStream;

/// Dial a target IP and port with a 10-second timeout.
pub async fn dial(target_ip: &str, target_port: u16) -> Result<TcpStream> {
    let addr = format!("{}:{}", target_ip, target_port);
    let stream = tokio::time::timeout(
        Duration::from_secs(10),
        TcpStream::connect(&addr),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Connection timeout to {}", addr))?
    .map_err(|e| anyhow::anyhow!("Failed to connect to {}: {}", addr, e))?;

    Ok(stream)
}
