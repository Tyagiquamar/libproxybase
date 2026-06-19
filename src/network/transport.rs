use anyhow::Result;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

/// Connect to the backend WebSocket endpoint via TLS 1.3 (rustls).
pub async fn connect_ws(url_str: &str) -> Result<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>> {
    let (ws, _response) = connect_async(url_str)
        .await
        .map_err(|e| anyhow::anyhow!("WebSocket connection failed: {}", e))?;

    Ok(ws)
}
