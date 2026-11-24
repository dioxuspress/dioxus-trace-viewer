use dioxus::prelude::*;
use dioxus_trace_viewer::trace::TraceStats;

#[component]
pub fn LatencyView(
  stats: TraceStats,
  zh: bool,
  on_filter_hotspot: EventHandler<String>,
) -> Element {
  let max_count = stats
    .histogram
    .iter()
    .map(|b| b.count)
    .max()
    .unwrap_or(1)
    .max(1);
  let upper_ms = stats.max_ms.max(1.0);
  let p50_x = 40.0 + (stats.p50_ms / upper_ms).clamp(0.0, 1.0) * 520.0;
  let p95_x = 40.0 + (stats.p95_ms / upper_ms).clamp(0.0, 1.0) * 520.0;
  let p99_x = 40.0 + (stats.p99_ms / upper_ms).clamp(0.0, 1.0) * 520.0;

  rsx! {
    div { class: "chart-grid",
      section { class: "panel",
        div { class: "panel-head",
          h2 { class: "panel-title",
            if zh { "延迟分布直方图（P50 / P95 / P99 分位线）" } else { "Latency Distribution Histogram (P50 / P95 / P99)" }
          }
        }
        div { class: "chart-body",
          svg {
            class: "hist-svg",
            view_box: "0 0 600 240",
            "aria-label": if zh { "延迟分布直方图" } else { "Latency distribution histogram" },
            // Baseline axis
            line { x1: "40", y1: "196", x2: "560", y2: "196", stroke: "var(--border-strong)", "stroke-width": "1.2" }

            for (i, bucket) in stats.histogram.iter().enumerate() {
              {
                let bar_w = 520.0 / (stats.histogram.len().max(1) as f64);
                let x = 40.0 + (i as f64) * bar_w + 3.0;
                let h = (bucket.count as f64 / max_count as f64) * 145.0;
                let y = 196.0 - h;
                let fill = if bucket.has_anomaly { "var(--err)" } else { "var(--gpu)" };
                rsx! {
                  rect {
                    key: "hb-{i}",
                    x: "{x:.1}",
                    y: "{y:.1}",
                    width: "{(bar_w - 6.0).max(4.0):.1}",
                    height: "{h:.1}",
                    rx: "3",
                    fill: "{fill}",
                    opacity: "0.86",
                  }
                  if bucket.count > 0 {
                    text {
                      x: "{(x + (bar_w - 6.0) / 2.0):.1}",
                      y: "{(y - 5.0).max(16.0):.1}",
                      "text-anchor": "middle",
                      fill: "var(--text-subtle)",
                      "font-size": "10",
                      "{bucket.count}"
                    }
                  }
                }
              }
            }

            // Percentile markers
            line { x1: "{p50_x:.1}", y1: "22", x2: "{p50_x:.1}", y2: "196", stroke: "var(--ok)", "stroke-dasharray": "4 3", "stroke-width": "1.5" }
            text { x: "{p50_x:.1}", y: "16", "text-anchor": "middle", fill: "var(--ok)", "font-size": "10.5", "font-weight": "700", "P50" }

            line { x1: "{p95_x:.1}", y1: "36", x2: "{p95_x:.1}", y2: "196", stroke: "var(--slow)", "stroke-dasharray": "4 3", "stroke-width": "1.5" }
            text { x: "{p95_x:.1}", y: "30", "text-anchor": "middle", fill: "var(--slow)", "font-size": "10.5", "font-weight": "700", "P95" }

            line { x1: "{p99_x:.1}", y1: "50", x2: "{p99_x:.1}", y2: "196", stroke: "var(--err)", "stroke-dasharray": "4 3", "stroke-width": "1.5" }
            text { x: "{p99_x:.1}", y: "44", "text-anchor": "middle", fill: "var(--err)", "font-size": "10.5", "font-weight": "700", "P99" }

            text { x: "40", y: "218", fill: "var(--text-subtle)", "font-size": "10.5", "0 ms" }
            text { x: "300", y: "218", "text-anchor": "middle", fill: "var(--text-subtle)", "font-size": "10.5", "{(upper_ms * 0.5):.1} ms" }
            text { x: "560", y: "218", "text-anchor": "end", fill: "var(--text-subtle)", "font-size": "10.5", "{upper_ms:.1} ms" }
          }
        }
      }

      section { class: "panel",
        div { class: "panel-head",
          h2 { class: "panel-title",
            if zh { "Span 累计耗时热点排行（点击筛选瀑布流）" } else { "Hotspot Spans by Cumulative Duration" }
          }
        }
        div { class: "data-table-wrap",
          table { class: "data-table",
            thead {
              tr {
                th { if zh { "操作 / Span" } else { "Span / Phase" } }
                th { class: "num", if zh { "次数" } else { "Cnt" } }
                th { class: "num", if zh { "总耗时" } else { "Total" } }
                th { class: "num", "P95" }
                th { class: "num", if zh { "占比" } else { "Share" } }
              }
            }
            tbody {
              for row in stats.hotspots.iter().take(12) {
                {
                  let name_filter = row.name.clone();
                  rsx! {
                    tr { key: "{row.target}-{row.name}",
                      td {
                        button {
                          class: "btn btn-ghost",
                          onclick: move |_| on_filter_hotspot.call(name_filter.clone()),
                          "{row.name}"
                        }
                      }
                      td { class: "num", "{row.count}" }
                      td { class: "num", "{row.total_ms:.2} ms" }
                      td { class: "num", "{row.p95_ms:.2} ms" }
                      td { class: "num", "{row.share_pct:.1}%" }
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
}
