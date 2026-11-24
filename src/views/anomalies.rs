use dioxus::prelude::*;
use dioxus_trace_viewer::trace::{SpanStatus, TraceStats};

#[component]
pub fn AnomaliesView(stats: TraceStats, zh: bool, on_jump_span: EventHandler<usize>) -> Element {
  rsx! {
    section { class: "panel",
      div { class: "panel-head",
        h2 { class: "panel-title",
          if zh {
            "异常、回滚与 P95 长尾诊断清单（共 {stats.anomalies.len()} 项）"
          } else {
            "Errors, Rollbacks & P95 Tail Outliers ({stats.anomalies.len()})"
          }
        }
      }

      if stats.anomalies.is_empty() {
        div { class: "inspector-body",
          if zh {
            "未检测到 ERROR、回滚事件或显著长尾延迟异常。"
          } else {
            "No ERROR spans, rollbacks, or tail-latency outliers detected."
          }
        }
      } else {
        div { class: "anomaly-list",
          for item in stats.anomalies.iter() {
            {
              let idx = item.span_index;
              let card_cls = if item.status.is_anomaly() {
                "anomaly-card"
              } else {
                "anomaly-card anomaly-card-slow"
              };
              let badge_cls = match item.status {
                SpanStatus::Rollback | SpanStatus::Error => "status-badge badge-err",
                _ => "status-badge badge-slow",
              };
              rsx! {
                div { key: "anom-{idx}", class: "{card_cls}",
                  div {
                    div { class: "badge-row",
                      span { class: "{badge_cls}", "{item.status.label()}" }
                      strong { "{item.name}" }
                      span { class: "anomaly-meta", "{item.target}" }
                    }
                    div { class: "anomaly-meta",
                      "{item.duration_ms:.2} ms ({item.vs_p95_ratio:.2}× P95) · {item.reason}"
                    }
                  }
                  button {
                    class: "btn",
                    onclick: move |_| on_jump_span.call(idx),
                    if zh { "定位瀑布流 →" } else { "Inspect in Waterfall →" }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
