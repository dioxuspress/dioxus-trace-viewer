use dioxus::prelude::*;
use dioxus_trace_viewer::trace::{SpanKind, SpanStatus, TraceDataset, TraceSpan, TraceStats};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuickFilter {
  All,
  Anomalies,
  Slow,
}

#[component]
pub fn WaterfallView(
  dataset: TraceDataset,
  stats: TraceStats,
  mut selected_span: Signal<usize>,
  mut search_query: Signal<String>,
  zh: bool,
) -> Element {
  let mut filter = use_signal(|| QuickFilter::All);
  let query_lower = search_query().trim().to_ascii_lowercase();
  let p95 = stats.p95_ms.max(0.01);
  let wall = dataset.wall_duration_ms.max(0.001);

  let filtered: Vec<&TraceSpan> = dataset
    .spans
    .iter()
    .filter(|span| {
      let pass_filter = match filter() {
        QuickFilter::All => true,
        QuickFilter::Anomalies => span.status.is_anomaly() || span.status == SpanStatus::Warn,
        QuickFilter::Slow => span.duration_ms >= p95,
      };
      if !pass_filter {
        return false;
      }
      if query_lower.is_empty() {
        return true;
      }
      span.name.to_ascii_lowercase().contains(&query_lower)
        || span.target.to_ascii_lowercase().contains(&query_lower)
        || span.message.to_ascii_lowercase().contains(&query_lower)
    })
    .collect();

  let active_span = dataset
    .spans
    .get(selected_span())
    .or_else(|| filtered.first().copied())
    .or_else(|| dataset.spans.first());

  let tick_25 = wall * 0.25;
  let tick_50 = wall * 0.50;
  let tick_75 = wall * 0.75;

  rsx! {
    div { class: "wf-split",
      section { class: "panel",
        div { class: "panel-head",
          h2 { class: "panel-title",
            if zh {
              "Span 时间轴瀑布流（显示 {filtered.len()} / {dataset.spans.len()}）"
            } else {
              "Span Timeline Waterfall ({filtered.len()} / {dataset.spans.len()})"
            }
          }
          div { class: "filter-bar",
            input {
              class: "search-input",
              r#type: "search",
              value: "{search_query}",
              placeholder: if zh { "按名称 / target / 消息筛选..." } else { "Filter by name, target, message..." },
              oninput: move |evt| search_query.set(evt.value()),
            }
            button {
              class: if filter() == QuickFilter::All { "btn btn-active" } else { "btn" },
              onclick: move |_| filter.set(QuickFilter::All),
              if zh { "全部" } else { "All" }
            }
            button {
              class: if filter() == QuickFilter::Anomalies { "btn btn-active" } else { "btn" },
              onclick: move |_| filter.set(QuickFilter::Anomalies),
              if zh { "仅异常" } else { "Anomalies" }
            }
            button {
              class: if filter() == QuickFilter::Slow { "btn btn-active" } else { "btn" },
              onclick: move |_| filter.set(QuickFilter::Slow),
              if zh { ">= P95" } else { ">= P95" }
            }
          }
        }

        div { class: "wf-ruler",
          span { if zh { "Span / 层级" } else { "Span / Hierarchy" } }
          div { class: "wf-ticks",
            span { "0 ms" }
            span { class: "wf-tick-mid", "{tick_25:.1}" }
            span { class: "wf-tick-mid", "{tick_50:.1}" }
            span { class: "wf-tick-mid", "{tick_75:.1}" }
            span { "{wall:.1} ms" }
          }
          span { class: "wf-dur", if zh { "耗时" } else { "Dur" } }
        }

        div { class: "wf-list",
          for (pos, span) in filtered.iter().enumerate() {
            {
              let span_idx = span.index;
              let is_selected = active_span.is_some_and(|s| s.index == span_idx);
              let depth_cls = match span.depth.min(4) {
                0 => "wf-name-cell wf-depth-0",
                1 => "wf-name-cell wf-depth-1",
                2 => "wf-name-cell wf-depth-2",
                3 => "wf-name-cell wf-depth-3",
                _ => "wf-name-cell wf-depth-4",
              };
              let dot_cls = dot_class(span, p95);
              let bar_fill = bar_color(span, p95);
              let x_pct = ((span.start_ms / wall) * 1000.0).clamp(0.0, 994.0);
              let w_pct = ((span.duration_ms / wall) * 1000.0).max(6.0).min(1000.0 - x_pct);
              let dur_cls = if span.status.is_anomaly() {
                "wf-dur wf-dur-err"
              } else if span.duration_ms >= p95 {
                "wf-dur wf-dur-slow"
              } else {
                "wf-dur"
              };
              // Apply content-visibility: auto only below the initial fold (pos >= 18).
              let row_cls = match (is_selected, pos >= 18) {
                (true, true) => "wf-row wf-row-selected wf-row-deferred",
                (true, false) => "wf-row wf-row-selected",
                (false, true) => "wf-row wf-row-deferred",
                (false, false) => "wf-row",
              };

              rsx! {
                button {
                  key: "{span.id}-{span_idx}",
                  class: "{row_cls}",
                  onclick: move |_| selected_span.set(span_idx),
                  div { class: "{depth_cls}",
                    span { class: "{dot_cls}", "aria-hidden": "true" }
                    span { class: "wf-name-text", title: "{span.name} ({span.target})", "{span.name}" }
                  }
                  svg {
                    class: "wf-bar-svg",
                    view_box: "0 0 1000 18",
                    preserve_aspect_ratio: "none",
                    "aria-label": "{span.name}: {span.duration_ms:.2} ms",
                    line { x1: "0", y1: "9", x2: "1000", y2: "9", stroke: "var(--border)", "stroke-width": "1" }
                    rect {
                      x: "{x_pct:.1}",
                      y: "3",
                      width: "{w_pct:.1}",
                      height: "12",
                      rx: "3",
                      fill: "{bar_fill}",
                    }
                  }
                  span { class: "{dur_cls}", "{span.duration_ms:.2} ms" }
                }
              }
            }
          }
        }
      }

      aside { class: "panel",
        div { class: "panel-head",
          h2 { class: "panel-title", if zh { "Span 详情检查器" } else { "Span Inspector" } }
        }
        if let Some(span) = active_span {
          {
            let ratio = span.duration_ms / p95;
            let wall_share = (span.duration_ms / wall) * 100.0;
            let badge_cls = match span.status {
              SpanStatus::Rollback | SpanStatus::Error => "status-badge badge-err",
              SpanStatus::Warn => "status-badge badge-slow",
              SpanStatus::Ok if span.duration_ms >= p95 => "status-badge badge-slow",
              SpanStatus::Ok => "status-badge badge-ok",
            };
            rsx! {
              div { class: "inspector-body",
                div { class: "badge-row",
                  span { class: "{badge_cls}", "{span.status.label()}" }
                  span { class: "status-badge badge-gpu", "{span.level}" }
                  span { class: "status-badge", "{wall_share:.1}% wall" }
                }
                table { class: "kv-table",
                  tbody {
                    tr {
                      th { "name" }
                      td { "{span.name}" }
                    }
                    tr {
                      th { "target" }
                      td { "{span.target}" }
                    }
                    tr {
                      th { if zh { "起止时间" } else { "window" } }
                      td { "{span.start_ms:.2} ms → {span.end_ms():.2} ms ({span.duration_ms:.2} ms)" }
                    }
                    tr {
                      th { if zh { "相对 P95" } else { "vs P95" } }
                      td { "{ratio:.2}× (P95 = {stats.p95_ms:.2} ms)" }
                    }
                    if !span.message.is_empty() {
                      tr {
                        th { "message" }
                        td { "{span.message}" }
                      }
                    }
                    for (k, v) in span.fields.iter().filter(|(k, _)| k.as_str() != "message") {
                      tr { key: "{k}",
                        th { "{k}" }
                        td { "{v}" }
                      }
                    }
                  }
                }
                pre { class: "raw-pre", "{span.raw_json}" }
              }
            }
          }
        } else {
          div { class: "inspector-body",
            if zh { "当前无可选 Span，请清空过滤条件或导入 JSONL 日志。" } else { "No span selected. Clear filters or import JSONL logs." }
          }
        }
      }
    }
  }
}

fn dot_class(span: &TraceSpan, p95: f64) -> &'static str {
  if span.status.is_anomaly() {
    "wf-dot dot-err"
  } else if span.status == SpanStatus::Warn || span.duration_ms >= p95 {
    "wf-dot dot-slow"
  } else {
    match span.kind {
      SpanKind::GpuKernel | SpanKind::TrainingStep => "wf-dot dot-gpu",
      SpanKind::Db => "wf-dot dot-db",
      _ => "wf-dot dot-ok",
    }
  }
}

fn bar_color(span: &TraceSpan, p95: f64) -> &'static str {
  if span.status.is_anomaly() {
    "var(--err)"
  } else if span.status == SpanStatus::Warn || span.duration_ms >= p95 {
    "var(--slow)"
  } else {
    match span.kind {
      SpanKind::GpuKernel | SpanKind::TrainingStep => "var(--gpu)",
      SpanKind::Db => "var(--db)",
      _ => "var(--ok)",
    }
  }
}
