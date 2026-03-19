use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Parser)]
#[command(name = "stress-ws", about = "WebSocket stress test")]
struct Args {
    /// WebSocket server URL
    #[arg(short, long, default_value = "ws://localhost:3003/socket.io/")]
    url: String,

    /// Number of concurrent clients
    #[arg(short, long, default_value_t = 1000)]
    clients: usize,

    /// Total messages per second (distributed across random clients)
    #[arg(short, long, default_value_t = 100)]
    rate: u64,

    /// Test duration in seconds
    #[arg(short, long, default_value_t = 10)]
    duration: u64,

    /// Connection batch size
    #[arg(short, long, default_value_t = 100)]
    batch: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!(
        "\n\
        ╔══════════════════════════════════════════════╗\n\
        ║     WebSocket Stress Test (Rust)              ║\n\
        ╠══════════════════════════════════════════════╣\n\
        ║  Server:       {:<29}║\n\
        ║  Clients:      {:<29}║\n\
        ║  Total rate:   {:<29}║\n\
        ║  Duration:     {:<29}║\n\
        ╚══════════════════════════════════════════════╝\n",
        args.url,
        args.clients,
        format!("{} msg/s (shared)", args.rate),
        format!("{}s", args.duration),
    );

    let total_sent = Arc::new(AtomicU64::new(0));
    let total_received = Arc::new(AtomicU64::new(0));
    let connect_errors = Arc::new(AtomicU64::new(0));
    let send_errors = Arc::new(AtomicU64::new(0));
    let latency_sum = Arc::new(AtomicU64::new(0));
    let latency_count = Arc::new(AtomicU64::new(0));

    // Collect per-client senders for broadcasting
    type WsSender = futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >;
    let senders: Arc<tokio::sync::Mutex<Vec<WsSender>>> =
        Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Connect clients in batches
    println!("⏳ Connecting {} clients in batches of {}...", args.clients, args.batch);
    let connect_start = Instant::now();
    let mut receiver_handles = Vec::new();

    for batch_start in (0..args.clients).step_by(args.batch) {
        let batch_end = (batch_start + args.batch).min(args.clients);
        let mut batch_futures = Vec::new();

        for _id in batch_start..batch_end {
            let url = args.url.clone();
            let recv_count = total_received.clone();
            let lat_sum = latency_sum.clone();
            let lat_cnt = latency_count.clone();
            let conn_errs = connect_errors.clone();
            let senders = senders.clone();

            batch_futures.push(tokio::spawn(async move {
                match tokio::time::timeout(Duration::from_secs(10), connect_async(&url)).await {
                    Ok(Ok((ws_stream, _))) => {
                        let (mut write, read) = ws_stream.split();

                        // Subscribe to room
                        let sub = json!({"type": "subscribe", "room": "userLocation"});
                        let _ = write.send(Message::Text(sub.to_string())).await;

                        senders.lock().await.push(write);

                        // Spawn receiver task
                        let handle = tokio::spawn(async move {
                            let mut read = read;
                            while let Some(Ok(msg)) = read.next().await {
                                if let Message::Text(text) = msg {
                                    recv_count.fetch_add(1, Ordering::Relaxed);
                                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                                        if let Some(sent_at) = parsed
                                            .get("data")
                                            .and_then(|d| d.get("sentAt"))
                                            .and_then(|v| v.as_u64())
                                        {
                                            let now = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap()
                                                .as_millis() as u64;
                                            let lat = now.saturating_sub(sent_at);
                                            lat_sum.fetch_add(lat, Ordering::Relaxed);
                                            lat_cnt.fetch_add(1, Ordering::Relaxed);
                                        }
                                    }
                                }
                            }
                        });
                        Some(handle)
                    }
                    _ => {
                        conn_errs.fetch_add(1, Ordering::Relaxed);
                        None
                    }
                }
            }));
        }

        let results = futures_util::future::join_all(batch_futures).await;
        for r in results {
            if let Ok(Some(handle)) = r {
                receiver_handles.push(handle);
            }
        }

        let connected = args.clients - connect_errors.load(Ordering::Relaxed) as usize;
        eprint!(
            "\r   {}/{} connected — errors: {}   ",
            connected,
            args.clients,
            connect_errors.load(Ordering::Relaxed)
        );

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let connected = args.clients - connect_errors.load(Ordering::Relaxed) as usize;
    let connect_elapsed = connect_start.elapsed().as_secs_f64();
    println!(
        "\n✅ {}/{} connected in {:.1}s ({} errors)\n",
        connected,
        args.clients,
        connect_elapsed,
        connect_errors.load(Ordering::Relaxed)
    );

    let sender_count = senders.lock().await.len();
    if sender_count == 0 {
        eprintln!("❌ No clients connected — aborting");
        return;
    }

    // Send phase
    println!(
        "🚀 Sending {} msg/s across {} clients for {}s...\n",
        args.rate, sender_count, args.duration
    );

    let interval_us = 1_000_000 / args.rate;
    let test_start = Instant::now();
    let ts = total_sent.clone();
    let se = send_errors.clone();
    let dur = args.duration;

    // Progress printer
    let prog_sent = total_sent.clone();
    let prog_recv = total_received.clone();
    let prog_errs = send_errors.clone();
    let prog_start = test_start;
    let progress_handle = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(500));
        loop {
            interval.tick().await;
            let elapsed = prog_start.elapsed().as_secs_f64();
            let sent = prog_sent.load(Ordering::Relaxed);
            let recv = prog_recv.load(Ordering::Relaxed);
            let rate = if elapsed > 0.0 { sent as f64 / elapsed } else { 0.0 };
            eprint!(
                "\r📊 {:.1}s | Sent: {} | Received: {} | Rate: {:.0} msg/s | Errors: {}   ",
                elapsed,
                sent,
                recv,
                rate,
                prog_errs.load(Ordering::Relaxed)
            );
            if elapsed >= dur as f64 + 3.0 {
                break;
            }
        }
    });

    // Sender loop
    let senders_ref = senders.clone();
    let sender_handle = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_micros(interval_us));

        loop {
            interval.tick().await;
            if test_start.elapsed() >= Duration::from_secs(dur) {
                break;
            }

            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let user_id: usize = rand::random::<usize>() % sender_count;
            let lat: f64 = 19.4326 + (rand::random::<f64>() * 0.1);
            let lng: f64 = -99.1332 + (rand::random::<f64>() * 0.1);

            let payload = json!({
                "type": "userLocation",
                "room": "userLocation",
                "userId": format!("user-{}", user_id),
                "sentAt": now_ms,
                "location": {
                    "lat": lat,
                    "lng": lng,
                }
            });

            let idx = rand::random::<usize>() % sender_count;
            let mut lock = senders_ref.lock().await;
            if let Some(writer) = lock.get_mut(idx) {
                match writer.send(Message::Text(payload.to_string())).await {
                    Ok(_) => {
                        ts.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        se.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
    });

    sender_handle.await.unwrap();
    tokio::time::sleep(Duration::from_secs(3)).await;
    progress_handle.abort();

    // Results
    let elapsed = test_start.elapsed().as_secs_f64() - 3.0;
    let sent = total_sent.load(Ordering::Relaxed);
    let received = total_received.load(Ordering::Relaxed);
    let expected = sent * connected as u64;
    let delivery = if expected > 0 {
        (received as f64 / expected as f64) * 100.0
    } else {
        0.0
    };
    let actual_rate = sent as f64 / elapsed;

    let lat_c = latency_count.load(Ordering::Relaxed);
    let lat_s = latency_sum.load(Ordering::Relaxed);
    let avg_lat = if lat_c > 0 { lat_s as f64 / lat_c as f64 } else { 0.0 };

    println!(
        "\n\n\
        ╔══════════════════════════════════════════════╗\n\
        ║              Results                         ║\n\
        ╠══════════════════════════════════════════════╣\n\
        ║  Clients:        {:<27}║\n\
        ║  Duration:       {:<27}║\n\
        ║  Sent:           {:<27}║\n\
        ║  Received:       {:<27}║\n\
        ║  Delivery:       {:<27}║\n\
        ║  Actual rate:    {:<27}║\n\
        ║  Connect errs:   {:<27}║\n\
        ║  Send errs:      {:<27}║\n\
        ╠══════════════════════════════════════════════╣\n\
        ║  Latency (avg):  {:<27}║\n\
        ╚══════════════════════════════════════════════╝\n",
        format!("{} connected", connected),
        format!("{:.1}s", elapsed),
        format!("{} messages", sent),
        format!("{} / {}", received, expected),
        format!("{:.1}%", delivery),
        format!("{:.1} msg/s", actual_rate),
        connect_errors.load(Ordering::Relaxed),
        send_errors.load(Ordering::Relaxed),
        format!("{:.1} ms", avg_lat),
    );

    if delivery >= 99.0
        && connect_errors.load(Ordering::Relaxed) == 0
        && send_errors.load(Ordering::Relaxed) == 0
    {
        println!("✅ PASS — Server handled the load successfully");
    } else if delivery >= 95.0 {
        println!("⚠️  WARN — Minor issues detected");
    } else {
        println!("❌ FAIL — Significant issues under load");
    }

    // Cleanup
    println!("\n🧹 Closing connections...");
    for handle in receiver_handles {
        handle.abort();
    }
}
