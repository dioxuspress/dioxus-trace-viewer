use std::collections::{BTreeMap, HashMap};

use serde_json::Value;

use crate::trace::model::{SpanKind, SpanStatus, TraceDataset, TraceFormat, TraceSpan};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineFormat {
  Tracing,
  RieTrain,
  Generic,
}

/// Parses a JSONL string containing Rust `tracing-subscriber`, `rie-train`, or generic span events.
/// Malformed or blank lines are tolerated and counted in `skipped_lines`.
pub fn parse_jsonl(title: &str, input: &str) -> TraceDataset {
  let mut spans = Vec::new();
  let mut total_lines = 0usize;
  let mut skipped_lines = 0usize;
  let mut cursor_ms = 0.0f64;

  let mut saw_tracing = false;
  let mut saw_train = false;
  let mut saw_generic = false;

  for raw_line in input.lines() {
    let trimmed = raw_line.trim();
    if trimmed.is_empty() {
      continue;
    }
    total_lines += 1;

    let Ok(val) = serde_json::from_str::<Value>(trimmed) else {
      skipped_lines += 1;
      continue;
    };
    let Some(obj) = val.as_object() else {
      skipped_lines += 1;
      continue;
    };

    let (line_fmt, mut span) = parse_object(obj, spans.len(), cursor_ms, trimmed);
    match line_fmt {
      LineFormat::Tracing => saw_tracing = true,
      LineFormat::RieTrain => saw_train = true,
      LineFormat::Generic => saw_generic = true,
    }

    // Advance fallback sequential cursor for logs without explicit start_ms.
    cursor_ms = span.end_ms().max(cursor_ms + 0.1);
    span.index = spans.len();
    spans.push(span);
  }

  normalize_timeline(&mut spans);
  recompute_depths(&mut spans);

  let wall_start_ms = spans
    .iter()
    .map(|s| s.start_ms)
    .fold(f64::INFINITY, f64::min);
  let wall_start_ms = if wall_start_ms.is_finite() {
    wall_start_ms
  } else {
    0.0
  };
  let wall_end_ms = spans
    .iter()
    .map(TraceSpan::end_ms)
    .fold(wall_start_ms, f64::max);
  let wall_duration_ms = (wall_end_ms - wall_start_ms).max(0.001);

  let format = match (saw_tracing, saw_train, saw_generic) {
    (true, false, false) => TraceFormat::TracingSubscriber,
    (false, true, false) => TraceFormat::RieTrainBench,
    (false, false, true) => TraceFormat::GenericSpan,
    (false, false, false) => TraceFormat::GenericSpan,
    _ => TraceFormat::Mixed,
  };

  TraceDataset {
    title: title.to_string(),
    format,
    spans,
    total_lines,
    skipped_lines,
    wall_start_ms,
    wall_duration_ms,
  }
}

fn parse_object(
  obj: &serde_json::Map<String, Value>,
  idx: usize,
  fallback_start_ms: f64,
  raw_json: &str,
) -> (LineFormat, TraceSpan) {
  let is_rie_train =
    obj.contains_key("phase") || obj.contains_key("tokens_per_sec") || obj.contains_key("loss");
  let is_tracing = !is_rie_train
    && (obj.contains_key("fields") || obj.contains_key("spans") || obj.contains_key("timestamp"));

  if is_rie_train {
    (
      LineFormat::RieTrain,
      parse_rie_train(obj, idx, fallback_start_ms, raw_json),
    )
  } else if is_tracing {
    (
      LineFormat::Tracing,
      parse_tracing_subscriber(obj, idx, fallback_start_ms, raw_json),
    )
  } else {
    (
      LineFormat::Generic,
      parse_generic_span(obj, idx, fallback_start_ms, raw_json),
    )
  }
}

fn parse_tracing_subscriber(
  obj: &serde_json::Map<String, Value>,
  idx: usize,
  fallback_start_ms: f64,
  raw_json: &str,
) -> TraceSpan {
  let level = str_field(obj, "level")
    .unwrap_or_else(|| "INFO".to_string())
    .to_ascii_uppercase();
  let target = str_field(obj, "target").unwrap_or_else(|| "app".to_string());

  let mut fields = BTreeMap::new();
  if let Some(f_obj) = obj.get("fields").and_then(Value::as_object) {
    for (k, v) in f_obj {
      fields.insert(k.clone(), value_to_compact_string(v));
    }
  }
  if let Some(ts) = str_field(obj, "timestamp") {
    fields.insert("timestamp".to_string(), ts);
  }

  let span_obj = obj.get("span").and_then(Value::as_object);
  let span_name = span_obj
    .and_then(|s| str_field(s, "name"))
    .or_else(|| str_field(obj, "name"))
    .or_else(|| fields.get("message").cloned())
    .unwrap_or_else(|| format!("{target}#{idx}"));

  let spans_chain = obj.get("spans").and_then(Value::as_array);
  let depth = spans_chain.map_or(0, |arr| arr.len().saturating_sub(1));

  let id = str_field(obj, "id")
    .or_else(|| span_obj.and_then(|s| str_field(s, "id")))
    .unwrap_or_else(|| format!("tr-{idx}"));
  let parent_id = str_field(obj, "parent_id").or_else(|| {
    spans_chain.and_then(|arr| {
      if arr.len() >= 2 {
        arr[arr.len() - 2]
          .as_object()
          .and_then(|p| str_field(p, "name"))
      } else {
        None
      }
    })
  });

  let duration_ms = extract_duration_ms(obj, &fields).unwrap_or(0.5);
  let start_ms = num_field(obj, "start_ms")
    .or_else(|| parse_f64_str(fields.get("start_ms")))
    .or_else(|| str_field(obj, "timestamp").and_then(|ts| parse_iso_ms(&ts)))
    .unwrap_or(fallback_start_ms);

  let message = fields.get("message").cloned().unwrap_or_default();
  let status = classify_status(&level, None, &message, &fields);
  let kind = classify_kind(&target, &span_name, &fields);

  TraceSpan {
    index: idx,
    id,
    parent_id,
    depth,
    name: span_name,
    target,
    level,
    kind,
    status,
    start_ms,
    duration_ms,
    message,
    fields,
    raw_json: raw_json.to_string(),
  }
}

fn parse_rie_train(
  obj: &serde_json::Map<String, Value>,
  idx: usize,
  fallback_start_ms: f64,
  raw_json: &str,
) -> TraceSpan {
  let step = obj.get("step").and_then(Value::as_u64);
  let phase = str_field(obj, "phase")
    .or_else(|| str_field(obj, "name"))
    .unwrap_or_else(|| "train_step".to_string());
  let backend = str_field(obj, "backend").unwrap_or_else(|| "metal".to_string());
  let name = match step {
    Some(s) => format!("step {s} · {phase}"),
    None => phase.clone(),
  };

  let mut fields = BTreeMap::new();
  for (k, v) in obj {
    if !matches!(k.as_str(), "start_ms" | "duration_ms" | "message") {
      fields.insert(k.clone(), value_to_compact_string(v));
    }
  }

  let raw_status = str_field(obj, "status");
  let level = str_field(obj, "level")
    .unwrap_or_else(|| match raw_status.as_deref() {
      Some("rollback" | "overflow") => "ERROR".to_string(),
      Some("error" | "failed") => "ERROR".to_string(),
      Some("warn" | "slow") => "WARN".to_string(),
      _ => "INFO".to_string(),
    })
    .to_ascii_uppercase();

  let duration_ms = extract_duration_ms(obj, &fields).unwrap_or(1.0);
  let start_ms = num_field(obj, "start_ms").unwrap_or(fallback_start_ms);
  let message = str_field(obj, "message").unwrap_or_else(|| {
    let mut parts = Vec::new();
    if let Some(loss) = fields.get("loss") {
      parts.push(format!("loss={loss}"));
    }
    if let Some(tps) = fields.get("tokens_per_sec") {
      parts.push(format!("{tps} tok/s"));
    }
    if let Some(vram) = fields.get("vram_mb") {
      parts.push(format!("vram={vram} MiB"));
    }
    parts.join(" · ")
  });

  let status = classify_status(&level, raw_status.as_deref(), &message, &fields);
  let depth = obj.get("depth").and_then(Value::as_u64).unwrap_or(0) as usize;
  let id = str_field(obj, "id").unwrap_or_else(|| format!("rt-{idx}"));
  let parent_id = str_field(obj, "parent_id");

  let kind = if phase.contains("kernel")
    || phase.contains("attn")
    || phase.contains("gdn")
    || phase.contains("rmsnorm")
    || phase.contains("matmul")
  {
    SpanKind::GpuKernel
  } else {
    SpanKind::TrainingStep
  };

  TraceSpan {
    index: idx,
    id,
    parent_id,
    depth,
    name,
    target: format!("rie_train::{backend}"),
    level,
    kind,
    status,
    start_ms,
    duration_ms,
    message,
    fields,
    raw_json: raw_json.to_string(),
  }
}

fn parse_generic_span(
  obj: &serde_json::Map<String, Value>,
  idx: usize,
  fallback_start_ms: f64,
  raw_json: &str,
) -> TraceSpan {
  let name = str_field(obj, "name")
    .or_else(|| str_field(obj, "span"))
    .unwrap_or_else(|| format!("span_{idx}"));
  let target = str_field(obj, "target")
    .or_else(|| str_field(obj, "service"))
    .unwrap_or_else(|| "trace".to_string());
  let level = str_field(obj, "level")
    .unwrap_or_else(|| "INFO".to_string())
    .to_ascii_uppercase();
  let raw_status = str_field(obj, "status");
  let message = str_field(obj, "message").unwrap_or_default();

  let mut fields = BTreeMap::new();
  if let Some(f_obj) = obj.get("fields").and_then(Value::as_object) {
    for (k, v) in f_obj {
      fields.insert(k.clone(), value_to_compact_string(v));
    }
  }
  for (k, v) in obj {
    if !matches!(
      k.as_str(),
      "id"
        | "parent_id"
        | "name"
        | "target"
        | "level"
        | "status"
        | "start_ms"
        | "duration_ms"
        | "message"
        | "fields"
        | "depth"
    ) {
      fields.insert(k.clone(), value_to_compact_string(v));
    }
  }

  let duration_ms = extract_duration_ms(obj, &fields).unwrap_or(1.0);
  let start_ms = num_field(obj, "start_ms").unwrap_or(fallback_start_ms);
  let depth = obj.get("depth").and_then(Value::as_u64).unwrap_or(0) as usize;
  let id = str_field(obj, "id").unwrap_or_else(|| format!("sp-{idx}"));
  let parent_id = str_field(obj, "parent_id");

  let status = classify_status(&level, raw_status.as_deref(), &message, &fields);
  let kind = classify_kind(&target, &name, &fields);

  TraceSpan {
    index: idx,
    id,
    parent_id,
    depth,
    name,
    target,
    level,
    kind,
    status,
    start_ms,
    duration_ms,
    message,
    fields,
    raw_json: raw_json.to_string(),
  }
}

fn normalize_timeline(spans: &mut [TraceSpan]) {
  let Some(min_start) = spans.iter().map(|s| s.start_ms).reduce(f64::min) else {
    return;
  };
  if min_start > 0.0 {
    for span in spans {
      span.start_ms = (span.start_ms - min_start).max(0.0);
    }
  }
}

fn recompute_depths(spans: &mut [TraceSpan]) {
  let mut id_to_depth: HashMap<String, usize> = HashMap::new();
  for span in spans.iter_mut() {
    if let Some(ref pid) = span.parent_id
      && let Some(&parent_depth) = id_to_depth.get(pid)
    {
      span.depth = (parent_depth + 1).min(4);
    }
    id_to_depth.insert(span.id.clone(), span.depth);
  }
}

fn extract_duration_ms(
  obj: &serde_json::Map<String, Value>,
  fields: &BTreeMap<String, String>,
) -> Option<f64> {
  for key in ["duration_ms", "elapsed_ms", "busy_ms", "latency_ms"] {
    if let Some(v) = num_field(obj, key) {
      return Some(v.max(0.0));
    }
    if let Some(v) = parse_f64_str(fields.get(key)) {
      return Some(v.max(0.0));
    }
  }
  if let Some(us) =
    num_field(obj, "duration_us").or_else(|| parse_f64_str(fields.get("duration_us")))
  {
    return Some((us / 1000.0).max(0.0));
  }
  if let Some(raw) = fields.get("time.busy").or_else(|| fields.get("busy")) {
    return parse_human_duration_ms(raw);
  }
  None
}

/// Parses `tracing-subscriber` formatted durations such as `12.4ms`, `430µs`, `1.2s`.
fn parse_human_duration_ms(raw: &str) -> Option<f64> {
  let s = raw.trim();
  if let Some(num) = s.strip_suffix("ms") {
    num.trim().parse::<f64>().ok()
  } else if let Some(num) = s.strip_suffix("µs").or_else(|| s.strip_suffix("us")) {
    num.trim().parse::<f64>().ok().map(|v| v / 1000.0)
  } else if let Some(num) = s.strip_suffix("ns") {
    num.trim().parse::<f64>().ok().map(|v| v / 1_000_000.0)
  } else if let Some(num) = s.strip_suffix('s') {
    num.trim().parse::<f64>().ok().map(|v| v * 1000.0)
  } else {
    s.parse::<f64>().ok()
  }
}

/// Lightweight RFC3339 / ISO-8601 time-of-day millisecond extractor (`HH:MM:SS.sss`).
fn parse_iso_ms(ts: &str) -> Option<f64> {
  let time_part = ts.split('T').nth(1).unwrap_or(ts);
  let clean = time_part.trim_end_matches('Z').split(['+', '-']).next()?;
  let mut parts = clean.split(':');
  let h: f64 = parts.next()?.parse().ok()?;
  let m: f64 = parts.next()?.parse().ok()?;
  let s: f64 = parts.next()?.parse().ok()?;
  Some(((h * 3600.0) + (m * 60.0) + s) * 1000.0)
}

fn classify_status(
  level: &str,
  raw_status: Option<&str>,
  message: &str,
  fields: &BTreeMap<String, String>,
) -> SpanStatus {
  let lower_status = raw_status.unwrap_or("").to_ascii_lowercase();
  let lower_msg = message.to_ascii_lowercase();
  let has_rollback = lower_status.contains("rollback")
    || lower_status.contains("overflow")
    || lower_msg.contains("rollback")
    || lower_msg.contains("grad_overflow")
    || lower_msg.contains("loss_spike")
    || fields.get("rollback").is_some_and(|v| v == "true")
    || fields
      .get("action")
      .is_some_and(|v| v.to_ascii_lowercase().contains("rollback"));

  if has_rollback {
    SpanStatus::Rollback
  } else if level == "ERROR"
    || lower_status == "error"
    || lower_status == "failed"
    || lower_status == "blocked"
  {
    SpanStatus::Error
  } else if level == "WARN" || lower_status == "warn" || lower_status == "retry" {
    SpanStatus::Warn
  } else {
    SpanStatus::Ok
  }
}

fn classify_kind(target: &str, name: &str, fields: &BTreeMap<String, String>) -> SpanKind {
  let combined = format!("{target} {name}").to_ascii_lowercase();
  if combined.contains("guard") || combined.contains("inject") || combined.contains("policy") {
    SpanKind::Guardrail
  } else if combined.contains("metal")
    || combined.contains("cuda")
    || combined.contains("cubecl")
    || combined.contains("kernel")
    || combined.contains("attn")
    || combined.contains("gdn")
  {
    SpanKind::GpuKernel
  } else if combined.contains("train") || fields.contains_key("step") {
    SpanKind::TrainingStep
  } else if combined.contains("sql")
    || combined.contains("sea_orm")
    || combined.contains("postgres")
    || combined.contains("db")
  {
    SpanKind::Db
  } else if combined.contains("tantivy")
    || combined.contains("search")
    || combined.contains("bm25")
    || combined.contains("vector")
  {
    SpanKind::Search
  } else if combined.contains("http")
    || combined.contains("axum")
    || combined.contains("pingora")
    || combined.contains("gateway")
  {
    SpanKind::Http
  } else {
    SpanKind::General
  }
}

fn str_field(obj: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
  obj.get(key).and_then(|v| match v {
    Value::String(s) => Some(s.clone()),
    Value::Number(n) => Some(n.to_string()),
    _ => None,
  })
}

fn num_field(obj: &serde_json::Map<String, Value>, key: &str) -> Option<f64> {
  obj.get(key).and_then(|v| match v {
    Value::Number(n) => n.as_f64(),
    Value::String(s) => s.parse::<f64>().ok(),
    _ => None,
  })
}

fn parse_f64_str(opt: Option<&String>) -> Option<f64> {
  opt.and_then(|s| s.parse::<f64>().ok())
}

fn value_to_compact_string(v: &Value) -> String {
  match v {
    Value::String(s) => s.clone(),
    other => other.to_string(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_tracing_subscriber_jsonl_and_human_durations() {
    let input = r#"
{"timestamp":"2026-11-24T08:00:00.100Z","level":"INFO","target":"pingora::proxy","fields":{"message":"tls handshake","time.busy":"1.5ms"},"span":{"name":"sni_route","id":"s1"}}
{"timestamp":"2026-11-24T08:00:00.102Z","level":"ERROR","target":"sea_orm::db","parent_id":"s1","fields":{"message":"pool timeout rollback","time.busy":"48.2ms"},"span":{"name":"query_orders","id":"s2"}}
not valid json
"#;
    let ds = parse_jsonl("test", input);
    assert_eq!(ds.format, TraceFormat::TracingSubscriber);
    assert_eq!(ds.spans.len(), 2);
    assert_eq!(ds.skipped_lines, 1);
    assert_eq!(ds.spans[0].depth, 0);
    assert_eq!(ds.spans[1].depth, 1);
    assert_eq!(ds.spans[1].status, SpanStatus::Rollback);
    assert!((ds.spans[1].start_ms - 2.0).abs() < 1e-3);
    assert!((ds.spans[1].duration_ms - 48.2).abs() < 1e-3);
  }

  #[test]
  fn parses_rie_train_jsonl_and_flags_grad_overflow_rollback() {
    let input = r#"
{"step":13,"phase":"gdn_chunk_forward","backend":"metal","start_ms":0.0,"duration_ms":3.4,"tokens_per_sec":48200,"loss":1.81,"status":"ok"}
{"step":14,"phase":"amp_step_update","backend":"metal","start_ms":3.4,"duration_ms":14.8,"loss":9.42,"grad_norm":128.4,"status":"rollback","message":"AMP grad overflow -> rollback loss scale 65536->32768"}
"#;
    let ds = parse_jsonl("train", input);
    assert_eq!(ds.format, TraceFormat::RieTrainBench);
    assert_eq!(ds.spans.len(), 2);
    assert_eq!(ds.spans[0].kind, SpanKind::GpuKernel);
    assert_eq!(ds.spans[1].status, SpanStatus::Rollback);
  }
}
