#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod views;

use dioxus::prelude::*;
use dioxus_trace_viewer::trace::{
  TraceDataset, TraceStats, compute_stats, parse_jsonl, sample_datasets,
};

const FAVICON: Asset = asset!("/assets/favicon.svg");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
  Waterfall,
  Latency,
  Anomalies,
  Import,
}

impl ActiveTab {
  const ALL: [Self; 4] = [
    Self::Waterfall,
    Self::Latency,
    Self::Anomalies,
    Self::Import,
  ];

  fn index(self) -> usize {
    match self {
      Self::Waterfall => 0,
      Self::Latency => 1,
      Self::Anomalies => 2,
      Self::Import => 3,
    }
  }

  fn from_index(i: usize) -> Self {
    Self::ALL[i % Self::ALL.len()]
  }
}

fn main() {
  dioxus::launch(App);
}

#[component]
fn App() -> Element {
  let presets = use_memo(sample_datasets);
  let mut zh = use_signal(|| true);
  let mut dark = use_signal(|| false);
  let mut active_preset_id = use_signal(|| "dioxuspress".to_string());
  let mut raw_jsonl = use_signal(|| {
    presets()
      .first()
      .map(|p| p.jsonl.to_string())
      .unwrap_or_default()
  });
  let mut dataset_title = use_signal(|| "DioxusPress Fullstack".to_string());
  let mut active_tab = use_signal(|| ActiveTab::Waterfall);
  let mut selected_span = use_signal(|| 0usize);
  let mut search_query = use_signal(String::new);

  let dataset: Memo<TraceDataset> = use_memo(move || {
    let title = dataset_title();
    let payload = raw_jsonl();
    parse_jsonl(&title, &payload)
  });

  let stats: Memo<TraceStats> = use_memo(move || compute_stats(&dataset()));

  let is_zh = zh();
  let is_dark = dark();
  let ds = dataset();
  let st = stats();
  let anomaly_total = st.error_count + st.rollback_count;

  let on_tab_keydown = move |evt: KeyboardEvent| {
    let cur = active_tab().index();
    let next = match evt.key() {
      Key::ArrowRight => Some((cur + 1) % ActiveTab::ALL.len()),
      Key::ArrowLeft => Some((cur + ActiveTab::ALL.len() - 1) % ActiveTab::ALL.len()),
      Key::Home => Some(0),
      Key::End => Some(ActiveTab::ALL.len() - 1),
      _ => None,
    };
    if let Some(idx) = next {
      evt.prevent_default();
      active_tab.set(ActiveTab::from_index(idx));
    }
  };

  rsx! {
    document::Link { rel: "icon", r#type: "image/svg+xml", href: FAVICON }
    document::Stylesheet { href: MAIN_CSS }

    div {
      class: "trace-root",
      "data-trace-root": "1",
      "data-theme": if is_dark { "dark" } else { "light" },

      header { class: "topbar",
        div { class: "topbar-inner",
          div { class: "brand",
            div { class: "brand-mark", "aria-hidden": "true",
              svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none",
                rect { x: "2", y: "4", width: "12", height: "4", rx: "2", fill: "#10b981" }
                rect { x: "6", y: "10", width: "16", height: "4", rx: "2", fill: "#3b82f6" }
                rect { x: "11", y: "16", width: "11", height: "4", rx: "2", fill: "#f43f5e" }
              }
            }
            div {
              h1 { class: "brand-title", "Dioxus Trace Viewer" }
              p { class: "brand-sub",
                if is_zh {
                  "零后端 Rust tracing JSONL 与 rie-train 性能瀑布流剖析器 · Web WASM + Desktop"
                } else {
                  "Zero-backend Rust tracing JSONL & rie-train waterfall profiler · Web WASM + Desktop"
                }
              }
            }
          }

          div { class: "topbar-actions",
            div { class: "dataset-pills",
              for preset in presets() {
                {
                  let pid = preset.id.to_string();
                  let title_text = if is_zh { preset.title_zh } else { preset.title_en };
                  let is_selected = active_preset_id() == pid;
                  let jsonl_str = preset.jsonl.to_string();
                  let title_en = preset.title_en.to_string();
                  rsx! {
                    button {
                      key: "{pid}",
                      class: if is_selected { "btn btn-active" } else { "btn" },
                      onclick: move |_| {
                        active_preset_id.set(pid.clone());
                        dataset_title.set(title_en.clone());
                        raw_jsonl.set(jsonl_str.clone());
                        selected_span.set(0);
                        search_query.set(String::new());
                      },
                      "{title_text}"
                    }
                  }
                }
              }
            }

            button {
              class: "btn btn-ghost",
              onclick: move |_| zh.set(!is_zh),
              if is_zh { "EN" } else { "中文" }
            }

            button {
              class: "btn btn-ghost",
              onclick: move |_| dark.set(!is_dark),
              if is_dark {
                if is_zh { "亮色岩白" } else { "Light" }
              } else {
                if is_zh { "深炭夜视" } else { "Dark" }
              }
            }
          }
        }
      }

      main { class: "shell",
        section { class: "kpi-grid", "aria-label": if is_zh { "核心延迟指标" } else { "Latency KPIs" },
          div { class: "kpi-card",
            div { class: "kpi-label", if is_zh { "Span 总数" } else { "Total Spans" } }
            div { class: "kpi-value", "{st.span_count}" }
            div { class: "kpi-meta",
              if ds.skipped_lines > 0 {
                if is_zh { "跳过 {ds.skipped_lines} 行非 JSON" } else { "Skipped {ds.skipped_lines} invalid lines" }
              } else {
                if is_zh { "全量解析无损坏行" } else { "All lines parsed cleanly" }
              }
            }
          }

          div { class: "kpi-card",
            div { class: "kpi-label", if is_zh { "时间窗" } else { "Wall Window" } }
            div { class: "kpi-value", "{ds.wall_duration_ms:.1} ms" }
            div { class: "kpi-meta",
              if is_zh { "均值 {st.mean_ms:.2} ms" } else { "Mean {st.mean_ms:.2} ms" }
            }
          }

          div { class: "kpi-card",
            div { class: "kpi-label", "P50" }
            div { class: "kpi-value kpi-ok", "{st.p50_ms:.2} ms" }
            div { class: "kpi-meta", if is_zh { "中位延迟" } else { "Median latency" } }
          }

          div { class: "kpi-card",
            div { class: "kpi-label", "P95" }
            div { class: "kpi-value kpi-slow", "{st.p95_ms:.2} ms" }
            div { class: "kpi-meta",
              if is_zh { "{st.slow_count} 个 Span >= P95" } else { "{st.slow_count} spans >= P95" }
            }
          }

          div { class: "kpi-card",
            div { class: "kpi-label", "P99" }
            div { class: "kpi-value kpi-slow", "{st.p99_ms:.2} ms" }
            div { class: "kpi-meta",
              if is_zh { "峰值 {st.max_ms:.2} ms" } else { "Max {st.max_ms:.2} ms" }
            }
          }

          div { class: "kpi-card",
            div { class: "kpi-label", if is_zh { "异常与回滚" } else { "Errors & Rollbacks" } }
            div {
              class: if anomaly_total > 0 { "kpi-value kpi-err" } else { "kpi-value kpi-ok" },
              "{anomaly_total}"
            }
            div { class: "kpi-meta",
              if is_zh {
                "{st.rollback_count} 回滚 · {st.error_count} 错误"
              } else {
                "{st.rollback_count} rollbacks · {st.error_count} errors"
              }
            }
          }
        }

        div { class: "tablist-wrap",
          div {
            class: "tablist",
            role: "tablist",
            "aria-label": if is_zh { "剖析视图切换" } else { "Profiler views" },
            onkeydown: on_tab_keydown,

            button {
              class: "tab-btn",
              role: "tab",
              "aria-selected": if active_tab() == ActiveTab::Waterfall { "true" } else { "false" },
              tabindex: if active_tab() == ActiveTab::Waterfall { "0" } else { "-1" },
              onclick: move |_| active_tab.set(ActiveTab::Waterfall),
              if is_zh { "Span 瀑布流" } else { "Span Waterfall" }
              span { class: "pill-count", "{st.span_count}" }
            }

            button {
              class: "tab-btn",
              role: "tab",
              "aria-selected": if active_tab() == ActiveTab::Latency { "true" } else { "false" },
              tabindex: if active_tab() == ActiveTab::Latency { "0" } else { "-1" },
              onclick: move |_| active_tab.set(ActiveTab::Latency),
              if is_zh { "P50/P95/P99 与热点" } else { "Latency & Hotspots" }
            }

            button {
              class: "tab-btn",
              role: "tab",
              "aria-selected": if active_tab() == ActiveTab::Anomalies { "true" } else { "false" },
              tabindex: if active_tab() == ActiveTab::Anomalies { "0" } else { "-1" },
              onclick: move |_| active_tab.set(ActiveTab::Anomalies),
              if is_zh { "异常与回滚定位" } else { "Anomalies & Rollbacks" }
              span {
                class: if anomaly_total > 0 { "pill-count pill-count-err" } else { "pill-count" },
                "{st.anomalies.len()}"
              }
            }

            button {
              class: "tab-btn",
              role: "tab",
              "aria-selected": if active_tab() == ActiveTab::Import { "true" } else { "false" },
              tabindex: if active_tab() == ActiveTab::Import { "0" } else { "-1" },
              onclick: move |_| active_tab.set(ActiveTab::Import),
              if is_zh { "导入 / 粘贴 JSONL" } else { "Import / Paste JSONL" }
            }
          }

          span { class: "format-badge", "{ds.format.label(is_zh)}" }
        }

        match active_tab() {
          ActiveTab::Waterfall => rsx! {
            views::waterfall::WaterfallView {
              dataset: ds,
              stats: st,
              selected_span,
              search_query,
              zh: is_zh,
            }
          },
          ActiveTab::Latency => rsx! {
            views::latency::LatencyView {
              stats: st,
              zh: is_zh,
              on_filter_hotspot: move |name: String| {
                search_query.set(name);
                active_tab.set(ActiveTab::Waterfall);
              },
            }
          },
          ActiveTab::Anomalies => rsx! {
            views::anomalies::AnomaliesView {
              stats: st,
              zh: is_zh,
              on_jump_span: move |idx: usize| {
                selected_span.set(idx);
                search_query.set(String::new());
                active_tab.set(ActiveTab::Waterfall);
              },
            }
          },
          ActiveTab::Import => rsx! {
            views::importer::ImportView {
              raw_jsonl,
              dataset_title,
              active_preset_id,
              zh: is_zh,
              on_loaded: move |_| {
                selected_span.set(0);
                search_query.set(String::new());
                active_tab.set(ActiveTab::Waterfall);
              },
            }
          },
        }
      }

      footer { class: "footer",
        span {
          if is_zh {
            "100% 本地浏览器 WASM / 桌面解析 · 零日志上传 · 兼容无 'unsafe-eval' 严格 CSP"
          } else {
            "100% local WASM / Desktop parsing · Zero log upload · Strict no-'unsafe-eval' CSP"
          }
        }
        span {
          a {
            href: "https://github.com/dioxuspress/dioxus-trace-viewer",
            target: "_blank",
            rel: "noopener noreferrer",
            "GitHub · dioxuspress/dioxus-trace-viewer"
          }
          " · "
          a {
            href: "https://rustineverything.app/case/dioxus-trace-viewer",
            target: "_blank",
            rel: "noopener noreferrer",
            "Rust in Everything"
          }
        }
      }
    }
  }
}
