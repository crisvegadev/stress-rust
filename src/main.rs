mod client;
mod metrics;
mod report;
mod scenarios;

use std::sync::Arc;

use clap::Parser;
use tokio::time::{Duration, Instant};

use metrics::Metrics;

#[derive(Parser)]
#[command(name = "stress-ws", about = "Knotion WebSocket stress test")]
struct Args {
    /// WebSocket server URL
    #[arg(short, long, default_value = "ws://localhost:3003/socket.io/")]
    url: String,

    /// Number of device clients (send userLocation)
    #[arg(short = 'd', long, default_value_t = 1000)]
    devices: usize,

    /// Number of group rooms to simulate
    #[arg(short = 'g', long, default_value_t = 20)]
    groups: usize,

    /// Devices per group
    #[arg(long, default_value_t = 30)]
    devices_per_group: usize,

    /// Location ping interval in seconds per device
    #[arg(short = 'i', long, default_value_t = 10)]
    ping_interval: u64,

    /// Test duration in seconds
    #[arg(short = 't', long, default_value_t = 60)]
    duration: u64,

    /// Connection batch size (concurrent connects)
    #[arg(short, long, default_value_t = 200)]
    batch: usize,

    /// Connection ramp-up delay between batches (ms)
    #[arg(long, default_value_t = 100)]
    ramp_delay_ms: u64,

    /// Number of coach broadcast events per group per minute
    #[arg(long, default_value_t = 2)]
    coach_events_per_min: u64,

    /// Output JSON report to file
    #[arg(long)]
    json_output: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let metrics = Arc::new(Metrics::new());
    let test_duration = Duration::from_secs(args.duration);

    // Total clients: 1 map + devices + groups (coaches)
    let total_group_devices = (args.groups * args.devices_per_group).min(args.devices);

    eprintln!("🚀 Knotion WebSocket Stress Test");
    eprintln!("   Server: {}", args.url);
    eprintln!("   Devices: {}, Groups: {}, Devices/group: {}", args.devices, args.groups, args.devices_per_group);
    eprintln!("   Duration: {}s, Ping interval: {}s", args.duration, args.ping_interval);
    eprintln!();

    let start = Instant::now();

    // Spawn map client first
    let map_url = args.url.clone();
    let map_metrics = metrics.clone();
    let map_handle = tokio::spawn(async move {
        scenarios::run_map_client(map_url, test_duration, map_metrics).await;
    });

    // Small delay to let map client connect first
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Spawn coach clients
    let mut coach_handles = Vec::new();
    for g in 1..=args.groups {
        let url = args.url.clone();
        let m = metrics.clone();
        let epm = args.coach_events_per_min;
        coach_handles.push(tokio::spawn(async move {
            scenarios::run_coach(url, g, epm, test_duration, m).await;
        }));
    }

    // Spawn device clients in batches
    let mut device_handles = Vec::new();
    for batch_start in (0..args.devices).step_by(args.batch) {
        let batch_end = (batch_start + args.batch).min(args.devices);
        let mut batch_tasks = Vec::new();

        for device_id in batch_start..batch_end {
            let url = args.url.clone();
            let m = metrics.clone();
            let pi = args.ping_interval;

            // Assign first N devices to groups
            let group = if device_id < total_group_devices {
                Some((device_id % args.groups) + 1)
            } else {
                None
            };

            batch_tasks.push(tokio::spawn(async move {
                scenarios::run_device(url, device_id, group, pi, test_duration, m).await;
            }));
        }

        // Wait for this batch to at least start connecting
        for task in batch_tasks {
            device_handles.push(task);
        }

        let connected = metrics
            .connections_successful
            .load(std::sync::atomic::Ordering::Relaxed);
        let failed = metrics
            .connections_failed
            .load(std::sync::atomic::Ordering::Relaxed);
        eprintln!(
            "🔗 Batch {}-{} spawned ({} connected, {} failed)",
            batch_start, batch_end, connected, failed
        );

        tokio::time::sleep(Duration::from_millis(args.ramp_delay_ms)).await;
    }

    // Progress reporter
    let progress_metrics = metrics.clone();
    let progress_handle = tokio::spawn(async move {
        let progress_start = Instant::now();
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let elapsed = progress_start.elapsed().as_secs_f64();
            report::print_progress(elapsed, &progress_metrics);
            if elapsed >= test_duration.as_secs_f64() + 5.0 {
                break;
            }
        }
    });

    // Wait for all tasks
    for h in device_handles {
        let _ = h.await;
    }
    for h in coach_handles {
        let _ = h.await;
    }
    let _ = map_handle.await;
    progress_handle.abort();

    let actual_duration = start.elapsed().as_secs_f64();

    // Final report
    report::print_final_report(&args.url, actual_duration, &metrics);

    // JSON output
    if let Some(ref path) = args.json_output {
        let config = serde_json::json!({
            "devices": args.devices,
            "groups": args.groups,
            "devices_per_group": args.devices_per_group,
            "ping_interval": args.ping_interval,
            "duration": args.duration,
            "batch": args.batch,
            "coach_events_per_min": args.coach_events_per_min
        });
        if let Err(e) = report::write_json_report(path, &args.url, actual_duration, &config, &metrics) {
            eprintln!("❌ Failed to write JSON report: {e}");
        } else {
            eprintln!("📄 JSON report written to {path}");
        }
    }
}
