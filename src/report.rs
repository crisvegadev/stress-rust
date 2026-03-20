use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::metrics::{self, Metrics};

pub fn print_progress(elapsed_secs: f64, metrics: &Arc<Metrics>) {
    let conns = metrics.connections_successful.load(Ordering::Relaxed);
    let sent = metrics.messages_sent.load(Ordering::Relaxed);
    let map_recv = metrics.location_messages_received.load(Ordering::Relaxed);
    let group_recv = metrics.group_messages_received.load(Ordering::Relaxed);
    let errs = metrics.send_errors.load(Ordering::Relaxed)
        + metrics.connections_failed.load(Ordering::Relaxed);

    eprintln!(
        "📊 {:.1}s | Conns: {} | Sent: {} | Map recv: {} | Group recv: {} | Errs: {}",
        elapsed_secs, conns, sent, map_recv, group_recv, errs
    );
}

pub fn print_final_report(
    url: &str,
    duration_secs: f64,
    metrics: &Arc<Metrics>,
) {
    let conns_total = metrics.connections_attempted.load(Ordering::Relaxed);
    let conns_ok = metrics.connections_successful.load(Ordering::Relaxed);
    let conns_fail = metrics.connections_failed.load(Ordering::Relaxed);
    let sent = metrics.messages_sent.load(Ordering::Relaxed);
    let map_recv = metrics.location_messages_received.load(Ordering::Relaxed);
    let group_recv = metrics.group_messages_received.load(Ordering::Relaxed);
    let send_errs = metrics.send_errors.load(Ordering::Relaxed);

    let mut conn_times = metrics.connection_time_ms.lock().unwrap().clone();
    let (ct_p50, _ct_p90, ct_p95, ct_p99) = metrics::percentiles(&mut conn_times);

    let mut latencies = metrics.latencies_ms.lock().unwrap().clone();
    let (l_p50, l_p90, l_p95, l_p99) = metrics::percentiles(&mut latencies);

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

    let delivery_pct = if sent > 0 {
        map_recv as f64 / sent as f64 * 100.0
    } else {
        0.0
    };
    let conn_err_pct = if conns_total > 0 {
        conns_fail as f64 / conns_total as f64 * 100.0
    } else {
        0.0
    };

    let result = if delivery_pct > 99.0 && l_p99 < 100 && conn_err_pct < 1.0 {
        "PASS"
    } else if delivery_pct > 95.0 || l_p99 < 500 {
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
            delivery_pct, l_p99
        ),
        _ => format!(
            "❌ FAIL — delivery {:.2}%, p99 {}ms, conn errors {:.1}%",
            delivery_pct, l_p99, conn_err_pct
        ),
    };

    println!(
        r#"
╔══════════════════════════════════════════════════╗
║           Knotion WebSocket Stress Test           ║
╠══════════════════════════════════════════════════╣
║  Server:          {:<30}║
║  Duration:        {:<30.1}║
╠══════════════════════════════════════════════════╣
║  CONNECTIONS                                      ║
║    Total:         {:<30}║
║    Successful:    {:<30}║
║    Failed:        {:<30}║
║    Connect time:  p50={}ms p95={}ms p99={}ms      ║
╠══════════════════════════════════════════════════╣
║  MESSAGES                                         ║
║    Sent:          {:<30}║
║    Map received:  {} ({:.2}%)                      ║
║    Group received: {:<28}║
║    Send errors:   {:<30}║
╠══════════════════════════════════════════════════╣
║  LATENCY (userLocation)                           ║
║    p50:  {}ms                                     ║
║    p90:  {}ms                                     ║
║    p95:  {}ms                                     ║
║    p99:  {}ms                                     ║
╠══════════════════════════════════════════════════╣
║  THROUGHPUT                                       ║
║    Inbound:       {:.0} msg/s                     ║
║    Map outbound:  {:.0} msg/s                     ║
╚══════════════════════════════════════════════════╝

{result_line}
"#,
        url,
        duration_secs,
        conns_total,
        conns_ok,
        conns_fail,
        ct_p50,
        ct_p95,
        ct_p99,
        sent,
        map_recv,
        delivery_pct,
        group_recv,
        send_errs,
        l_p50,
        l_p90,
        l_p95,
        l_p99,
        inbound_per_sec,
        outbound_per_sec,
    );
}

pub fn write_json_report(
    path: &str,
    url: &str,
    duration_secs: f64,
    config: &serde_json::Value,
    metrics: &Arc<Metrics>,
) -> std::io::Result<()> {
    let conns_total = metrics.connections_attempted.load(Ordering::Relaxed);
    let conns_ok = metrics.connections_successful.load(Ordering::Relaxed);
    let conns_fail = metrics.connections_failed.load(Ordering::Relaxed);
    let sent = metrics.messages_sent.load(Ordering::Relaxed);
    let map_recv = metrics.location_messages_received.load(Ordering::Relaxed);
    let group_recv = metrics.group_messages_received.load(Ordering::Relaxed);

    let mut conn_times = metrics.connection_time_ms.lock().unwrap().clone();
    let (ct_p50, _ct_p90, ct_p95, ct_p99) = metrics::percentiles(&mut conn_times);

    let mut latencies = metrics.latencies_ms.lock().unwrap().clone();
    let (l_p50, l_p90, l_p95, l_p99) = metrics::percentiles(&mut latencies);

    let delivery_pct = if sent > 0 {
        map_recv as f64 / sent as f64 * 100.0
    } else {
        0.0
    };
    let conn_err_pct = if conns_total > 0 {
        conns_fail as f64 / conns_total as f64 * 100.0
    } else {
        0.0
    };

    let result = if delivery_pct > 99.0 && l_p99 < 100 && conn_err_pct < 1.0 {
        "PASS"
    } else if delivery_pct > 95.0 || l_p99 < 500 {
        "WARN"
    } else {
        "FAIL"
    };

    let inbound = (map_recv + group_recv) as f64 / duration_secs;
    let outbound = map_recv as f64 / duration_secs;

    let report = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "server": url,
        "config": config,
        "connections": {
            "total": conns_total,
            "successful": conns_ok,
            "failed": conns_fail
        },
        "messages": {
            "sent": sent,
            "map_received": map_recv,
            "group_received": group_recv
        },
        "latency_ms": {
            "p50": l_p50,
            "p90": l_p90,
            "p95": l_p95,
            "p99": l_p99
        },
        "connection_time_ms": {
            "p50": ct_p50,
            "p95": ct_p95,
            "p99": ct_p99
        },
        "throughput": {
            "inbound_per_sec": inbound as u64,
            "outbound_per_sec": outbound as u64
        },
        "result": result
    });

    std::fs::write(path, serde_json::to_string_pretty(&report).unwrap())
}
