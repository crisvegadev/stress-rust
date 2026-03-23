use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::metrics::Metrics;

pub fn print_progress(elapsed_secs: f64, metrics: &Arc<Metrics>) {
    let conns = metrics.connections_successful.load(Ordering::Relaxed);
    let sent = metrics.messages_sent.load(Ordering::Relaxed);
    let map_recv = metrics.location_messages_received.load(Ordering::Relaxed);
    let group_recv = metrics.group_messages_received.load(Ordering::Relaxed);
    let errs = metrics.send_errors.load(Ordering::Relaxed)
        + metrics.connections_failed.load(Ordering::Relaxed);
    let lat = metrics.latency.percentiles();

    eprintln!(
        "📊 {:.1}s | Conns: {} | Sent: {} | Map recv: {} | Group recv: {} | p50: {}ms p99: {}ms | Errs: {}",
        elapsed_secs, conns, sent, map_recv, group_recv, lat.p50, lat.p99, errs
    );
}

pub fn print_final_report(
    url: &str,
    duration_secs: f64,
    metrics: &Arc<Metrics>,
    map_clients: usize,
    ramp_up_secs: u64,
) {
    let conns_total = metrics.connections_attempted.load(Ordering::Relaxed);
    let conns_ok = metrics.connections_successful.load(Ordering::Relaxed);
    let conns_fail = metrics.connections_failed.load(Ordering::Relaxed);
    let sent = metrics.messages_sent.load(Ordering::Relaxed);
    let map_recv = metrics.location_messages_received.load(Ordering::Relaxed);
    let group_recv = metrics.group_messages_received.load(Ordering::Relaxed);
    let send_errs = metrics.send_errors.load(Ordering::Relaxed);

    let ct = metrics.conn_time.percentiles();
    let lat = metrics.latency.percentiles();

    let inbound_per_sec = if duration_secs > 0.0 {
        (map_recv + group_recv) as f64 / duration_secs
    } else {
        0.0
    };
    let outbound_per_sec = if duration_secs > 0.0 {
        map_recv as f64 / duration_secs
    } else {
        0.0
    };

    let delivery_pct = if sent > 0 && map_clients > 0 {
        let expected = sent * map_clients as u64;
        map_recv as f64 / expected as f64 * 100.0
    } else {
        0.0
    };
    let conn_err_pct = if conns_total > 0 {
        conns_fail as f64 / conns_total as f64 * 100.0
    } else {
        0.0
    };

    let result = if delivery_pct > 99.0 && lat.p99 < 100 && conn_err_pct < 1.0 {
        "PASS"
    } else if delivery_pct > 95.0 || lat.p99 < 500 {
        "WARN"
    } else {
        "FAIL"
    };

    let result_line = match result {
        "PASS" => format!(
            "✅ PASS — All connections stable, delivery >{:.0}%, p99 <100ms",
            delivery_pct
        ),
        "WARN" => format!(
            "⚠️  WARN — delivery {:.2}%, p99 {}ms",
            delivery_pct, lat.p99
        ),
        _ => format!(
            "❌ FAIL — delivery {:.2}%, p99 {}ms, conn errors {:.1}%",
            delivery_pct, lat.p99, conn_err_pct
        ),
    };

    // Time-series degradation analysis
    let time_series = metrics.time_series.lock().unwrap();
    let stability = analyze_stability(&time_series, ramp_up_secs);

    println!(
        r#"
╔══════════════════════════════════════════════════╗
║           Knotion WebSocket Stress Test           ║
╠══════════════════════════════════════════════════╣
║  Server:          {:<30}║
║  Duration:        {:<30.1}║
║  Latency samples: {:<30}║
╠══════════════════════════════════════════════════╣
║  CONNECTIONS                                      ║
║    Total:         {:<30}║
║    Successful:    {:<30}║
║    Failed:        {:<30}║
║    Connect time:  p50={:>4}ms  p95={:>4}ms  p99={:>4}ms   ║
╠══════════════════════════════════════════════════╣
║  MESSAGES                                         ║
║    Sent:          {:<30}║
║    Map received:  {} ({:.2}%){}
║    Group received: {:<28}║
║    Send errors:   {:<30}║
╠══════════════════════════════════════════════════╣
║  LATENCY (userLocation)                           ║
║    min:   {:>6}ms                                  ║
║    avg:   {:>6.1}ms                                  ║
║    p50:   {:>6}ms                                  ║
║    p90:   {:>6}ms                                  ║
║    p95:   {:>6}ms                                  ║
║    p99:   {:>6}ms                                  ║
║    max:   {:>6}ms                                  ║
╠══════════════════════════════════════════════════╣
║  THROUGHPUT                                       ║
║    Inbound:       {:.0} msg/s                     ║
║    Map outbound:  {:.0} msg/s                     ║
╠══════════════════════════════════════════════════╣
║  STABILITY                                        ║
{}╚══════════════════════════════════════════════════╝

{result_line}
"#,
        url,
        duration_secs,
        lat.count,
        conns_total,
        conns_ok,
        conns_fail,
        ct.p50, ct.p95, ct.p99,
        sent,
        map_recv, delivery_pct,
        " ".repeat(30 - format!("{} ({:.2}%)", map_recv, delivery_pct).len().min(30)),
        group_recv,
        send_errs,
        lat.min,
        lat.avg,
        lat.p50,
        lat.p90,
        lat.p95,
        lat.p99,
        lat.max,
        inbound_per_sec,
        outbound_per_sec,
        stability,
    );
}

fn analyze_stability(snapshots: &[crate::metrics::Snapshot], ramp_up_secs: u64) -> String {
    let skip = (ramp_up_secs as usize).max(3);

    if snapshots.len() <= skip {
        return "║    Not enough data points for analysis           ║\n".to_string();
    }

    // Skip ramp-up period
    let stable: Vec<_> = snapshots.iter().skip(skip).collect();
    if stable.is_empty() {
        return "║    Not enough data after ramp-up                 ║\n".to_string();
    }

    let p99_values: Vec<u64> = stable.iter().map(|s| s.lat_p99).collect();
    let p99_min = *p99_values.iter().min().unwrap_or(&0);
    let p99_max = *p99_values.iter().max().unwrap_or(&0);
    let p99_first = p99_values.first().copied().unwrap_or(0);
    let p99_last = p99_values.last().copied().unwrap_or(0);

    let rate_values: Vec<u64> = stable.iter().map(|s| s.msgs_sent_per_sec).collect();
    let rate_avg = if rate_values.is_empty() {
        0
    } else {
        rate_values.iter().sum::<u64>() / rate_values.len() as u64
    };

    let degraded = p99_last > p99_first * 3 && p99_last > 100;
    let rate_dropped = rate_values.last().copied().unwrap_or(0) < rate_avg / 2 && rate_avg > 0;

    let mut lines = String::new();

    lines.push_str(&format!(
        "║    p99 range:     {}ms — {}ms                    \n",
        p99_min, p99_max
    ));
    lines.push_str(&format!(
        "║    p99 trend:     {}ms → {}ms                    \n",
        p99_first, p99_last
    ));
    lines.push_str(&format!(
        "║    Avg send rate: {} msg/s                       \n",
        rate_avg
    ));

    if degraded {
        lines.push_str(
            "║    ⚠️  DEGRADATION DETECTED — p99 increased 3x+   \n",
        );
    } else if rate_dropped {
        lines.push_str(
            "║    ⚠️  THROUGHPUT DROP — send rate fell >50%        \n",
        );
    } else {
        lines.push_str(
            "║    ✅ Latency stable throughout test               \n",
        );
    }

    lines
}

pub fn write_json_report(
    path: &str,
    url: &str,
    duration_secs: f64,
    config: &serde_json::Value,
    metrics: &Arc<Metrics>,
    map_clients: usize,
    ramp_up_secs: u64,
) -> std::io::Result<()> {
    let conns_total = metrics.connections_attempted.load(Ordering::Relaxed);
    let conns_ok = metrics.connections_successful.load(Ordering::Relaxed);
    let conns_fail = metrics.connections_failed.load(Ordering::Relaxed);
    let sent = metrics.messages_sent.load(Ordering::Relaxed);
    let map_recv = metrics.location_messages_received.load(Ordering::Relaxed);
    let group_recv = metrics.group_messages_received.load(Ordering::Relaxed);

    let ct = metrics.conn_time.percentiles();
    let lat = metrics.latency.percentiles();

    let delivery_pct = if sent > 0 && map_clients > 0 {
        let expected = sent * map_clients as u64;
        map_recv as f64 / expected as f64 * 100.0
    } else {
        0.0
    };
    let conn_err_pct = if conns_total > 0 {
        conns_fail as f64 / conns_total as f64 * 100.0
    } else {
        0.0
    };

    let result = if delivery_pct > 99.0 && lat.p99 < 100 && conn_err_pct < 1.0 {
        "PASS"
    } else if delivery_pct > 95.0 || lat.p99 < 500 {
        "WARN"
    } else {
        "FAIL"
    };

    let inbound = (map_recv + group_recv) as f64 / duration_secs;
    let outbound = map_recv as f64 / duration_secs;

    let time_series = metrics.time_series.lock().unwrap();

    let report = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "server": url,
        "config": config,
        "result": result,
        "connections": {
            "total": conns_total,
            "successful": conns_ok,
            "failed": conns_fail
        },
        "messages": {
            "sent": sent,
            "map_received": map_recv,
            "group_received": group_recv,
            "delivery_pct": delivery_pct
        },
        "latency_ms": {
            "min": lat.min,
            "avg": lat.avg,
            "p50": lat.p50,
            "p90": lat.p90,
            "p95": lat.p95,
            "p99": lat.p99,
            "max": lat.max,
            "samples": lat.count
        },
        "connection_time_ms": {
            "p50": ct.p50,
            "p90": ct.p90,
            "p95": ct.p95,
            "p99": ct.p99
        },
        "throughput": {
            "inbound_per_sec": inbound as u64,
            "outbound_per_sec": outbound as u64
        },
        "ramp_up_secs": ramp_up_secs,
        "time_series": *time_series
    });

    std::fs::write(path, serde_json::to_string_pretty(&report).unwrap())
}
