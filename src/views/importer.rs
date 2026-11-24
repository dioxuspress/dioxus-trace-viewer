use dioxus::prelude::*;

const JSONL_PLACEHOLDER: &str = r#"{"timestamp":"...","level":"INFO","target":"axum","span":{"name":"req"},"fields":{"duration_ms":4.2}}"#;

#[component]
pub fn ImportView(
  mut raw_jsonl: Signal<String>,
  mut dataset_title: Signal<String>,
  mut active_preset_id: Signal<String>,
  zh: bool,
  on_loaded: EventHandler<()>,
) -> Element {
  let mut draft = use_signal(move || raw_jsonl.peek().clone());
  let mut status_msg = use_signal(String::new);

  let on_file_change = move |evt: FormEvent| {
    let files = evt.files();
    if let Some(file) = files.into_iter().next() {
      let file_name = file.name();
      spawn(async move {
        if let Ok(text) = file.read_string().await {
          draft.set(text.clone());
          raw_jsonl.set(text);
          dataset_title.set(file_name.clone());
          active_preset_id.set("custom".to_string());
          status_msg.set(format!("Loaded local file: {file_name}"));
          on_loaded.call(());
        } else {
          status_msg.set("Failed to read file as UTF-8 text.".to_string());
        }
      });
    }
  };

  rsx! {
    section { class: "panel",
      div { class: "panel-head",
        h2 { class: "panel-title",
          if zh {
            "本地加载或粘贴 JSONL（纯本地 WASM / Desktop 解析，绝不上传服务器）"
          } else {
            "Load Local File or Paste JSONL (100% Local Parsing, Never Uploaded)"
          }
        }
      }
      div { class: "import-grid",
        div { class: "drop-box",
          div {
            strong {
              if zh {
                "选择本地 .jsonl / .log 文件"
              } else {
                "Select a local .jsonl / .log file"
              }
            }
            p { class: "brand-sub",
              if zh {
                "自动识别 Rust tracing-subscriber::fmt().json()、rie-train 基准报告与通用 Span JSONL。"
              } else {
                "Auto-detects Rust tracing-subscriber JSON, rie-train benchmarks, and generic Span JSONL."
              }
            }
          }
          input {
            r#type: "file",
            accept: ".jsonl,.json,.log,.txt",
            onchange: on_file_change,
          }
        }

        textarea {
          class: "jsonl-area",
          value: "{draft}",
          placeholder: JSONL_PLACEHOLDER,
          oninput: move |evt| draft.set(evt.value()),
        }

        div { class: "filter-bar",
          button {
            class: "btn btn-active",
            onclick: move |_| {
              raw_jsonl.set(draft());
              dataset_title.set(if zh { "自定义粘贴日志".to_string() } else { "Custom Pasted JSONL".to_string() });
              active_preset_id.set("custom".to_string());
              status_msg.set(if zh { "已解析自定义 JSONL" } else { "Parsed custom JSONL" }.to_string());
              on_loaded.call(());
            },
            if zh { "解析并打开瀑布流" } else { "Parse & Open Waterfall" }
          }
          if !status_msg().is_empty() {
            span { class: "brand-sub", "{status_msg}" }
          }
        }
      }
    }
  }
}
