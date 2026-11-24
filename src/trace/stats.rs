use std::collections::BTreeMap;

use crate::trace::model::{SpanStatus, TraceDataset, TraceSpan};

/// A single bucket in the latency distribution histogram.
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramBucket {
  pub min_ms: f64,
  pub max_ms: f64,
  pub count: usize,
  pub has_anomaly: bool,
}

/// Aggregated latency and error breakdown grouped by span name + target.
#[derive(Debug, Clone, PartialEq)]
pub struct HotspotRow {
  pub name: String,
  pub target: String,
  pub count: usize,
  pub total_ms: f64,
  pub share_pct: f64,
  pub p95_ms: f64,
  pub max_ms: f64,
  pub anomalies: usize,
}

/// Diagnostic card entry for errors, rollbacks, or slow P95+ tail outliers.
#[derive(Debug, Clone, PartialEq)]
pub struct AnomalyItem {
  pub span_index: usize,
  pub name: String,
  pub target: String,
  pub status: SpanStatus,
  pub duration_ms: f64,
  pub vs_p95_ratio: f64,
  pub reason: String,
}

/// Complete statistical summary for a loaded [`TraceDataset`].
#[derive(Debug, Clone, PartialEq)]
pub struct TraceStats {
  pub span_count: usize,
  pub p50_ms: f64,
  pub p95_ms: f64,
  pub p99_ms: f64,
  pub max_ms: f64,
  pub mean_ms: f64,
  pub error_count: usize,
  pub rollback_count: usize,
  pub slow_count: usize,
  pub histogram: Vec<HistogramBucket>,
  pub hotspots: Vec<HotspotRow>,
  pub anomalies: Vec<AnomalyItem>,
}

/// Computes P50/P95/P99 latency metrics, histogram buckets, hotspot table, and anomaly list.
pub fn compute_stats(dataset: &TraceDataset) -> TraceStats {
  let spans = &dataset.spans;
  if spans.is_empty() {
    return TraceStats {
      span_count: 0,
      p50_ms: 0.0,
      p95_ms: 0.0,
      p99_ms: 0.0,
      max_ms: 0.0,
      mean_ms: 0.0,
      error_count: 0,
      rollback_count: 0,
      slow_count: 0,
      histogram: Vec::new(),
      hotspots: Vec::new(),
      anomalies: Vec::new(),
    };
  }

  let mut durations: Vec<f64> = spans.iter().map(|s| s.duration_ms).collect();
  durations.sort_by(f64::total_cmp);

  let p50_ms = percentile(&durations, 0.50);
  let p95_ms = percentile(&durations, 0.95);
  let p99_ms = percentile(&durations, 0.99);
  let max_ms = durations.last().copied().unwrap_or(0.0);
  let sum_ms: f64 = durations.iter().sum();
  let mean_ms = sum_ms / (durations.len() as f64);

  let error_count = spans
    .iter()
    .filter(|s| s.status == SpanStatus::Error)
    .count();
  let rollback_count = spans
    .iter()
    .filter(|s| s.status == SpanStatus::Rollback)
    .count();
  let slow_threshold = p95_ms.max(0.01);
  let slow_count = spans
    .iter()
    .filter(|s| s.duration_ms >= slow_threshold)
    .count();

  let histogram = build_histogram(spans, max_ms);
  let hotspots = build_hotspots(spans, sum_ms);
  let anomalies = build_anomalies(spans, p95_ms);

  TraceStats {
    span_count: spans.len(),
    p50_ms,
    p95_ms,
    p99_ms,
    max_ms,
    mean_ms,
    error_count,
    rollback_count,
    slow_count,
    histogram,
    hotspots,
    anomalies,
  }
}

pub fn percentile(sorted: &[f64], p: f64) -> f64 {
  if sorted.is_empty() {
    return 0.0;
  }
  let idx = ((sorted.len().saturating_sub(1)) as f64 * p).round() as usize;
  sorted[idx.min(sorted.len() - 1)]
}

fn build_histogram(spans: &[TraceSpan], max_ms: f64) -> Vec<HistogramBucket> {
  const BUCKETS: usize = 12;
  let upper = max_ms.max(1.0);
  let step = upper / (BUCKETS as f64);
  let mut buckets: Vec<HistogramBucket> = (0..BUCKETS)
    .map(|i| HistogramBucket {
      min_ms: i as f64 * step,
      max_ms: (i + 1) as f64 * step,
      count: 0,
      has_anomaly: false,
    })
    .collect();

  for span in spans {
    let raw_idx = (span.duration_ms / step).floor() as usize;
    let idx = raw_idx.min(BUCKETS - 1);
    buckets[idx].count += 1;
    if span.status.is_anomaly() {
      buckets[idx].has_anomaly = true;
    }
  }
  buckets
}

fn build_hotspots(spans: &[TraceSpan], total_sum_ms: f64) -> Vec<HotspotRow> {
  let mut grouped: BTreeMap<(String, String), Vec<&TraceSpan>> = BTreeMap::new();
  for span in spans {
    // Strip step numbers for rie-train so repeated phases aggregate cleanly.
    let key_name = span
      .name
      .split(" · ")
      .nth(1)
      .unwrap_or(&span.name)
      .to_string();
    grouped
      .entry((key_name, span.target.clone()))
      .or_default()
      .push(span);
  }

  let denom = total_sum_ms.max(1e-6);
  let mut rows: Vec<HotspotRow> = grouped
    .into_iter()
    .map(|((name, target), group)| {
      let mut durs: Vec<f64> = group.iter().map(|s| s.duration_ms).collect();
      durs.sort_by(f64::total_cmp);
      let total_ms: f64 = durs.iter().sum();
      let anomalies = group.iter().filter(|s| s.status.is_anomaly()).count();
      HotspotRow {
        name,
        target,
        count: group.len(),
        total_ms,
        share_pct: (total_ms / denom) * 100.0,
        p95_ms: percentile(&durs, 0.95),
        max_ms: durs.last().copied().unwrap_or(0.0),
        anomalies,
      }
    })
    .collect();

  rows.sort_by(|a, b| b.total_ms.total_cmp(&a.total_ms));
  rows
}

fn build_anomalies(spans: &[TraceSpan], p95_ms: f64) -> Vec<AnomalyItem> {
  let baseline = p95_ms.max(0.1);
  let mut items = Vec::new();
  for span in spans {
    let is_slow_tail = span.duration_ms >= baseline && span.duration_ms > 5.0;
    if span.status.is_anomaly() || span.status == SpanStatus::Warn || is_slow_tail {
      let reason = if !span.message.is_empty() {
        span.message.clone()
      } else if span.status == SpanStatus::Rollback {
        "Rollback triggered during execution".to_string()
      } else if span.status == SpanStatus::Error {
        "Span exited with ERROR status".to_string()
      } else {
        format!(
          "Tail latency {:.2} ms (>= P95 {:.2} ms)",
          span.duration_ms, p95_ms
        )
      };

      items.push(AnomalyItem {
        span_index: span.index,
        name: span.name.clone(),
        target: span.target.clone(),
        status: span.status,
        duration_ms: span.duration_ms,
        vs_p95_ratio: span.duration_ms / baseline,
        reason,
      });
    }
  }

  // Sort rollbacks & errors ahead of pure latency outliers.
  items.sort_by(|a, b| {
    severity_rank(b.status)
      .cmp(&severity_rank(a.status))
      .then_with(|| b.duration_ms.total_cmp(&a.duration_ms))
  });
  items
}

fn severity_rank(status: SpanStatus) -> u8 {
  match status {
    SpanStatus::Rollback => 4,
    SpanStatus::Error => 3,
    SpanStatus::Warn => 2,
    SpanStatus::Ok => 1,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::trace::samples::sample_datasets;

  #[test]
  fn computes_percentiles_and_detects_rollbacks_on_all_samples() {
    let samples = sample_datasets();
    assert_eq!(samples.len(), 3);

    for preset in samples {
      let ds = crate::trace::parse_jsonl(preset.title_en, preset.jsonl);
      let st = compute_stats(&ds);
      assert!(st.span_count >= 12);
      assert!(st.p50_ms <= st.p95_ms);
      assert!(st.p95_ms <= st.p99_ms);
      assert!(st.p99_ms <= st.max_ms);
      assert!(!st.hotspots.is_empty());
      assert!(!st.anomalies.is_empty());
    }
  }
}
