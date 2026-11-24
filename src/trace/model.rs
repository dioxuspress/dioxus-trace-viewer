use std::collections::BTreeMap;

/// Auto-detected source log format of the loaded JSONL payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceFormat {
  TracingSubscriber,
  RieTrainBench,
  GenericSpan,
  Mixed,
}

impl TraceFormat {
  pub fn label(self, zh: bool) -> &'static str {
    match (self, zh) {
      (Self::TracingSubscriber, true) => "Rust tracing-subscriber JSONL",
      (Self::TracingSubscriber, false) => "Rust tracing-subscriber JSONL",
      (Self::RieTrainBench, true) => "rie-train GPU / 训练基准 JSONL",
      (Self::RieTrainBench, false) => "rie-train GPU / Training JSONL",
      (Self::GenericSpan, true) => "通用 Span JSONL",
      (Self::GenericSpan, false) => "Generic Span JSONL",
      (Self::Mixed, true) => "混合格式 JSONL",
      (Self::Mixed, false) => "Mixed Format JSONL",
    }
  }
}

/// High-level semantic category used for color coding in the waterfall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
  Http,
  Db,
  Search,
  GpuKernel,
  TrainingStep,
  Guardrail,
  General,
}

/// Health classification of a span or training step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStatus {
  Ok,
  Warn,
  Error,
  Rollback,
}

impl SpanStatus {
  pub fn is_anomaly(self) -> bool {
    matches!(self, Self::Error | Self::Rollback)
  }

  pub fn label(self) -> &'static str {
    match self {
      Self::Ok => "OK",
      Self::Warn => "WARN",
      Self::Error => "ERROR",
      Self::Rollback => "ROLLBACK",
    }
  }
}

/// A single parsed span or benchmark event normalized onto a relative millisecond timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceSpan {
  pub index: usize,
  pub id: String,
  pub parent_id: Option<String>,
  pub depth: usize,
  pub name: String,
  pub target: String,
  pub level: String,
  pub kind: SpanKind,
  pub status: SpanStatus,
  pub start_ms: f64,
  pub duration_ms: f64,
  pub message: String,
  pub fields: BTreeMap<String, String>,
  pub raw_json: String,
}

impl TraceSpan {
  pub fn end_ms(&self) -> f64 {
    self.start_ms + self.duration_ms.max(0.0)
  }
}

/// Complete parsed trace session ready for waterfall, histogram, and anomaly views.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceDataset {
  pub title: String,
  pub format: TraceFormat,
  pub spans: Vec<TraceSpan>,
  pub total_lines: usize,
  pub skipped_lines: usize,
  pub wall_start_ms: f64,
  pub wall_duration_ms: f64,
}
