use std::sync::Arc;
use std::time::Instant;

use futures_util::{SinkExt, StreamExt};
use tokio::time::{timeout, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::metrics::Metrics;

pub struct WsClient {
    pub sink: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    pub stream: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

pub async fn connect(url: &str, metrics: &Arc<Metrics>) -> Result<WsClient, String> {
    metrics
        .connections_attempted
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let start = Instant::now();

    let result = timeout(Duration::from_secs(10), connect_async(url)).await;

    match result {
        Ok(Ok((ws_stream, _))) => {
            let elapsed = start.elapsed().as_millis() as u64;
            metrics.record_connection_time(elapsed);
            metrics
                .connections_successful
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let (sink, stream) = ws_stream.split();
            Ok(WsClient { sink, stream })
        }
        Ok(Err(e)) => {
            metrics
                .connections_failed
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Err(format!("WebSocket error: {e}"))
        }
        Err(_) => {
            metrics
                .connections_failed
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Err("Connection timeout (10s)".to_string())
        }
    }
}

pub async fn subscribe(
    sink: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    room: &str,
) -> Result<(), String> {
    let msg = serde_json::json!({"type": "subscribe", "room": room});
    sink.send(Message::Text(msg.to_string()))
        .await
        .map_err(|e| format!("Subscribe error: {e}"))
}
