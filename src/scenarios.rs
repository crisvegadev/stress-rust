use std::sync::atomic::Ordering;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use tokio::time::{Duration, Instant};
use tokio_tungstenite::tungstenite::Message;

use crate::client;
use crate::metrics::Metrics;

/// Device client: subscribes to userLocation (and optionally a group), sends location pings.
pub async fn run_device(
    url: String,
    device_id: usize,
    group: Option<usize>,
    ping_interval: u64,
    duration: Duration,
    metrics: Arc<Metrics>,
) {
    let ws = match client::connect(&url, &metrics).await {
        Ok(c) => c,
        Err(_) => return,
    };
    let (mut sink, mut stream) = (ws.sink, ws.stream);

    // Devices only subscribe to their group room (if any), NOT to userLocation.
    // In production, devices send location but only the map listens.
    if let Some(g) = group {
        let room = format!("group-{g}");
        if client::subscribe(&mut sink, &room).await.is_err() {
            return;
        }
    }

    let deadline = Instant::now() + duration;
    let metrics_recv = metrics.clone();

    // Receiver task — only listens for group broadcasts (coach commands)
    let recv_handle = tokio::spawn(async move {
        while Instant::now() < deadline {
            match tokio::time::timeout(Duration::from_millis(100), stream.next()).await {
                Ok(Some(Ok(Message::Text(txt)))) => {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                        let event_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        match event_type {
                            "openResource" | "openSession" | "openSection" | "openActivity" => {
                                metrics_recv
                                    .group_messages_received
                                    .fetch_add(1, Ordering::Relaxed);
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Some(Ok(Message::Ping(_)))) => {}
                Ok(Some(Err(_))) | Ok(None) => break,
                _ => {} // timeout, continue
            }
        }
    });

    // Sender loop
    let base_lat = 19.4326 + (device_id as f64 * 0.0001) % 0.5;
    let base_lng = -99.1332 + (device_id as f64 * 0.00007) % 0.5;

    while Instant::now() < deadline {
        let (interval, lat, lng) = {
            let mut rng = rand::thread_rng();
            let jitter_ms = rng.gen_range(0..4000) as i64 - 2000;
            let interval =
                Duration::from_millis((ping_interval as i64 * 1000 + jitter_ms).max(1000) as u64);
            let lat = base_lat + rng.gen_range(-0.001..0.001);
            let lng = base_lng + rng.gen_range(-0.001..0.001);
            (interval, lat, lng)
        };
        tokio::time::sleep(interval).await;

        if Instant::now() >= deadline {
            break;
        }
        let now_ms = chrono::Utc::now().timestamp_millis();

        let msg = serde_json::json!({
            "type": "userLocation",
            "room": "userLocation",
            "userId": format!("user-{device_id}"),
            "location": { "lat": lat, "lng": lng },
            "sentAt": now_ms
        });

        match sink
            .send(Message::Text(msg.to_string()))
            .await
        {
            Ok(_) => {
                metrics.messages_sent.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {
                metrics.send_errors.fetch_add(1, Ordering::Relaxed);
                break;
            }
        }
    }

    let _ = recv_handle.await;
}

/// Map client: subscribes to userLocation, counts received messages and tracks latency.
pub async fn run_map_client(url: String, duration: Duration, metrics: Arc<Metrics>) {
    let ws = match client::connect(&url, &metrics).await {
        Ok(c) => c,
        Err(_) => return,
    };
    let (mut sink, mut stream) = (ws.sink, ws.stream);

    if client::subscribe(&mut sink, "userLocation").await.is_err() {
        return;
    }

    let deadline = Instant::now() + duration;

    while Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(100), stream.next()).await {
            Ok(Some(Ok(Message::Text(txt)))) => {
                metrics
                    .location_messages_received
                    .fetch_add(1, Ordering::Relaxed);

                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                    let sent_at = v.get("data")
                        .and_then(|d| d.get("sentAt"))
                        .or_else(|| v.get("sentAt"))
                        .and_then(|s| s.as_i64());
                    if let Some(sent_at) = sent_at {
                        let now = chrono::Utc::now().timestamp_millis();
                        let latency = (now - sent_at).max(0) as u64;
                        metrics.record_latency(latency);
                    }
                }
            }
            Ok(Some(Ok(Message::Ping(_)))) => {}
            Ok(Some(Err(_))) | Ok(None) => break,
            _ => {}
        }
    }
}

/// Coach client: subscribes to a group room and sends broadcast commands at configured rate.
pub async fn run_coach(
    url: String,
    group_id: usize,
    events_per_min: u64,
    duration: Duration,
    metrics: Arc<Metrics>,
) {
    let ws = match client::connect(&url, &metrics).await {
        Ok(c) => c,
        Err(_) => return,
    };
    let (mut sink, _stream) = (ws.sink, ws.stream);

    let room = format!("group-{group_id}");
    if client::subscribe(&mut sink, &room).await.is_err() {
        return;
    }

    let interval = if events_per_min > 0 {
        Duration::from_secs(60 / events_per_min)
    } else {
        Duration::from_secs(60)
    };

    let event_types = ["openResource", "openSession", "openSection", "openActivity"];
    let deadline = Instant::now() + duration;
    let mut counter = 0usize;

    while Instant::now() < deadline {
        tokio::time::sleep(interval).await;
        if Instant::now() >= deadline {
            break;
        }

        let event_type = event_types[counter % event_types.len()];
        let resource_id = format!("res-{group_id}-{counter}");
        let msg = serde_json::json!({
            "type": event_type,
            "room": room,
            "resourceId": resource_id,
            "data": {
                "type": event_type,
                "resourceId": resource_id
            }
        });

        match sink
            .send(Message::Text(msg.to_string()))
            .await
        {
            Ok(_) => {
                metrics.messages_sent.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {
                metrics.send_errors.fetch_add(1, Ordering::Relaxed);
                break;
            }
        }
        counter += 1;
    }
}
