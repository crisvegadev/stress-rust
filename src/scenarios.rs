use std::sync::atomic::Ordering;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use tokio::time::{Duration, Instant};
use tokio_tungstenite::tungstenite::Message;

use crate::client;
use crate::metrics::Metrics;

const CAMPUSES: &[&str] = &[
    "Colegio Colón de Ocotlán",
    "Colegio Lomas del Valle (Campus Acueducto)",
    "Instituto Cumbres Villahermosa",
    "Colegio Miraflores Guadalajara",
    "Instituto Alpes San Javier",
    "Colegio Británico de Monterrey",
    "Liceo Europeo de México",
    "Colegio Williams Campus Mixcoac",
    "Instituto Thomas Jefferson CDMX",
    "Colegio Americano de Puebla",
    "Instituto Irlandés Monterrey",
    "Colegio Suizo de México",
    "Academia Juárez de Chihuahua",
    "Colegio Cervantes Primavera GDL",
    "Instituto Oriente de Puebla",
    "Colegio La Salle Cancún",
    "Instituto Cumbres Oaxaca",
    "Colegio Marista Mérida",
    "Liceo Anglo Francés de Tijuana",
    "Colegio del Bosque Querétaro",
    "Instituto Bilingüe Canadiense León",
    "Colegio Americano de Bogotá",
    "Liceo Francés de Buenos Aires",
    "Instituto Cervantes São Paulo",
    "Colegio Lincoln Santiago Chile",
];

const USER_TYPES: &[&str] = &[
    "Student", "Student", "Student", "Student", "Student",
    "Student", "Student", "Student", "Student", "Student",
    "Coach", "Coordinator", "Parent",
];

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

    // Engine.IO + Socket.IO handshake
    if client::handshake(&mut sink, &mut stream).await.is_err() {
        return;
    }

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
    // Also handles Engine.IO pings
    let recv_handle = tokio::spawn(async move {
        while Instant::now() < deadline {
            match tokio::time::timeout(Duration::from_millis(100), stream.next()).await {
                Ok(Some(Ok(Message::Text(txt)))) => {
                    // Engine.IO ping/pong
                    if txt == "2" || txt == "3" { continue; }
                    if !txt.starts_with("42") { continue; }

                    // Parse Socket.IO format: 42["event", {data}]
                    let json_str = &txt[2..];
                    if let Ok(arr) = serde_json::from_str::<serde_json::Value>(json_str) {
                        if let Some(arr) = arr.as_array() {
                            let event_type = arr.first().and_then(|v| v.as_str()).unwrap_or("");
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
                }
                Ok(Some(Ok(Message::Ping(_)))) => {}
                Ok(Some(Err(_))) | Ok(None) => break,
                _ => {} // timeout, continue
            }
        }
    });

    // Sender loop — random positions across Mexico
    // Lat: 14.5 (Chiapas) to 32.7 (Tijuana)
    // Lng: -117.1 (Tijuana) to -86.7 (Cancún)
    let base_lat: f64 = {
        let mut rng = rand::thread_rng();
        // Weighted toward major cities
        let cities: &[(f64, f64)] = &[
            (19.4326, -99.1332),   // CDMX
            (20.6597, -103.3496),  // Guadalajara
            (25.6866, -100.3161),  // Monterrey
            (21.1619, -86.8515),   // Cancún
            (20.9674, -89.5926),   // Mérida
            (19.1738, -96.1342),   // Veracruz
            (32.5149, -117.0382),  // Tijuana
            (20.5888, -100.3899),  // Querétaro
            (21.8818, -102.2916),  // Aguascalientes
            (22.1565, -100.9855),  // San Luis Potosí
            (28.6353, -106.0889),  // Chihuahua
            (24.0277, -104.6532),  // Durango
            (17.0732, -96.7266),   // Oaxaca
            (16.7370, -93.1296),   // Tuxtla Gutiérrez
            (19.0414, -98.2063),   // Puebla
            (21.0190, -101.2574),  // León
            (20.1011, -98.7591),   // Pachuca
            (18.9242, -99.2216),   // Cuernavaca
            (23.6345, -102.5528),  // Zacatecas
            (31.6904, -106.4245),  // Ciudad Juárez
        ];
        let city = cities[device_id % cities.len()];
        city.0 + rng.gen_range(-0.05..0.05)
    };
    let base_lng: f64 = {
        let mut rng = rand::thread_rng();
        let cities: &[(f64, f64)] = &[
            (19.4326, -99.1332),
            (20.6597, -103.3496),
            (25.6866, -100.3161),
            (21.1619, -86.8515),
            (20.9674, -89.5926),
            (19.1738, -96.1342),
            (32.5149, -117.0382),
            (20.5888, -100.3899),
            (21.8818, -102.2916),
            (22.1565, -100.9855),
            (28.6353, -106.0889),
            (24.0277, -104.6532),
            (17.0732, -96.7266),
            (16.7370, -93.1296),
            (19.0414, -98.2063),
            (21.0190, -101.2574),
            (20.1011, -98.7591),
            (18.9242, -99.2216),
            (23.6345, -102.5528),
            (31.6904, -106.4245),
        ];
        let city = cities[device_id % cities.len()];
        city.1 + rng.gen_range(-0.05..0.05)
    };

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

        let msg = format!(
            "42{}",
            serde_json::json!([
                "userLocation",
                {
                    "isDevice": true,
                    "latitude": lat,
                    "longitude": lng,
                    "idUserInt": device_id,
                    "campus": CAMPUSES[device_id % CAMPUSES.len()],
                    "userType": USER_TYPES[device_id % USER_TYPES.len()],
                    "sentAt": now_ms
                }
            ])
        );

        match sink
            .send(Message::Text(msg))
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

    if client::handshake(&mut sink, &mut stream).await.is_err() {
        return;
    }

    if client::subscribe(&mut sink, "roomMap").await.is_err() {
        return;
    }

    let deadline = Instant::now() + duration;

    while Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(100), stream.next()).await {
            Ok(Some(Ok(Message::Text(txt)))) => {
                // Engine.IO ping — respond with pong
                if txt == "2" {
                    let _ = sink.send(Message::Text("3".to_string())).await;
                    continue;
                }
                // Engine.IO pong — ignore
                if txt == "3" { continue; }
                // Skip non-42 messages
                if !txt.starts_with("42") { continue; }

                metrics
                    .location_messages_received
                    .fetch_add(1, Ordering::Relaxed);

                let json_str = &txt[2..];
                if let Ok(arr) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(arr) = arr.as_array() {
                        let data = arr.get(1).unwrap_or(&serde_json::Value::Null);
                        if let Some(sent_at) = data.get("sentAt").and_then(|s| s.as_i64()) {
                            let now = chrono::Utc::now().timestamp_millis();
                            let latency = (now - sent_at).max(0) as u64;
                            metrics.record_latency(latency);
                        }
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
    let (mut sink, mut stream) = (ws.sink, ws.stream);

    if client::handshake(&mut sink, &mut stream).await.is_err() {
        return;
    }

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
        let msg = format!(
            "42{}",
            serde_json::json!([
                event_type,
                {
                    "room": room,
                    "resourceId": resource_id,
                    "type": event_type
                }
            ])
        );

        match sink
            .send(Message::Text(msg))
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
