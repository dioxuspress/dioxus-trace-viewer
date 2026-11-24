/// Built-in realistic trace sample ready for one-click exploration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplePreset {
  pub id: &'static str,
  pub title_zh: &'static str,
  pub title_en: &'static str,
  pub jsonl: &'static str,
}

pub fn sample_datasets() -> Vec<SamplePreset> {
  vec![
    SamplePreset {
      id: "dioxuspress",
      title_zh: "DioxusPress 全栈请求",
      title_en: "DioxusPress Fullstack",
      jsonl: SAMPLE_DIOXUSPRESS_TRACING,
    },
    SamplePreset {
      id: "rie-train",
      title_zh: "rie-train GPU 训练回滚",
      title_en: "rie-train GPU Rollback",
      jsonl: SAMPLE_RIE_TRAIN_BENCH,
    },
    SamplePreset {
      id: "rie-agent",
      title_zh: "rie-agent 护栏拦截",
      title_en: "rie-agent Guardrails",
      jsonl: SAMPLE_RIE_AGENT_TRACE,
    },
  ]
}

const SAMPLE_DIOXUSPRESS_TRACING: &str = r#"{"id":"req-1","timestamp":"2026-11-24T09:00:00.000Z","level":"INFO","target":"pingora::gateway","span":{"name":"tls_sni_dispatch"},"fields":{"host":"rustineverything.app","upstream":"127.0.0.1:8080","duration_ms":68.4,"message":"TLS 1.3 SNI matched dioxuspress-app"}}
{"id":"req-2","parent_id":"req-1","timestamp":"2026-11-24T09:00:00.001Z","level":"INFO","target":"axum::router","span":{"name":"http_request"},"fields":{"method":"POST","path":"/api/search/query","duration_ms":65.2,"message":"Matched server function route"}}
{"id":"req-3","parent_id":"req-2","timestamp":"2026-11-24T09:00:00.002Z","level":"INFO","target":"app_core::auth","span":{"name":"jwt_verify_middleware"},"fields":{"alg":"HS256","role":"member","duration_ms":0.8,"message":"Verified bearer session claims"}}
{"id":"req-4","parent_id":"req-2","timestamp":"2026-11-24T09:00:00.003Z","level":"INFO","target":"module_search::engine","span":{"name":"tantivy_bm25_search"},"fields":{"q_len":18,"segments":4,"hits":20,"duration_ms":1.9,"message":"MmapDirectory BM25 TopDocs fetched"}}
{"id":"req-5","parent_id":"req-4","timestamp":"2026-11-24T09:00:00.0035Z","level":"INFO","target":"module_search::tokenizer","span":{"name":"jieba_tokenize_query"},"fields":{"tokens":6,"duration_ms":0.4,"message":"Normalized CJK + ASCII query terms"}}
{"id":"req-6","parent_id":"req-4","timestamp":"2026-11-24T09:00:00.004Z","level":"INFO","target":"module_search::snippet","span":{"name":"extract_snippets"},"fields":{"docs":20,"duration_ms":0.7,"message":"Sliced UTF-8 safe preview windows"}}
{"id":"req-7","parent_id":"req-2","timestamp":"2026-11-24T09:00:00.006Z","level":"WARN","target":"sea_orm::pool","span":{"name":"pg_acquire_connection"},"fields":{"pool_size":16,"waiters":3,"duration_ms":21.6,"message":"Connection pool saturation during SIGHUP refresh"}}
{"id":"req-8","parent_id":"req-7","timestamp":"2026-11-24T09:00:00.028Z","level":"ERROR","target":"sea_orm::query","span":{"name":"fetch_forum_titles"},"fields":{"sql_state":"57014","duration_ms":19.4,"message":"Statement timeout exceeded (15ms) -> rollback read tx"}}
{"id":"req-9","parent_id":"req-2","timestamp":"2026-11-24T09:00:00.048Z","level":"INFO","target":"module_cases::server","span":{"name":"fallback_cached_case_titles"},"fields":{"cache_hit":true,"items":20,"duration_ms":0.6,"message":"Served case metadata from in-memory RwLock cache"}}
{"id":"req-10","parent_id":"req-2","timestamp":"2026-11-24T09:00:00.049Z","level":"INFO","target":"dioxus_ssr::render","span":{"name":"ssr_stream_chunk"},"fields":{"bytes":14820,"duration_ms":9.8,"message":"Streamed hydrated SSR payload"}}
{"id":"req-11","parent_id":"req-10","timestamp":"2026-11-24T09:00:00.050Z","level":"INFO","target":"widgets::markdown","span":{"name":"pulldown_render_cards"},"fields":{"cards":12,"duration_ms":4.1,"message":"Rendered highlighted snippet cards"}}
{"id":"req-12","parent_id":"req-10","timestamp":"2026-11-24T09:00:00.055Z","level":"INFO","target":"app::csp","span":{"name":"inject_csp_headers"},"fields":{"policy":"script-src 'self' 'wasm-unsafe-eval'","duration_ms":0.3,"message":"Attached strict CSP and ETag headers"}}
{"id":"req-13","parent_id":"req-1","timestamp":"2026-11-24T09:00:00.066Z","level":"INFO","target":"pingora::compress","span":{"name":"brotli_encode_response"},"fields":{"ratio":"3.8x","duration_ms":1.6,"message":"Compressed response stream"}}"#;

const SAMPLE_RIE_TRAIN_BENCH: &str = r#"{"id":"s10","step":10,"phase":"train_step","backend":"metal","start_ms":0.0,"duration_ms":18.4,"tokens_per_sec":47800,"vram_mb":6380,"loss":1.942,"grad_norm":0.86,"status":"ok"}
{"id":"s10-1","parent_id":"s10","step":10,"phase":"data_prefetch","backend":"metal","start_ms":0.0,"duration_ms":0.9,"vram_mb":6380,"status":"ok","message":"Pinned batch transferred to unified memory"}
{"id":"s10-2","parent_id":"s10","step":10,"phase":"gdn_chunk_forward","backend":"metal","start_ms":0.9,"duration_ms":5.1,"tokens_per_sec":48900,"vram_mb":6410,"status":"ok","message":"Gated DeltaNet fused kernel (1.46x vs PyTorch)"}
{"id":"s10-3","parent_id":"s10","step":10,"phase":"cubecl_rmsnorm","backend":"metal","start_ms":6.0,"duration_ms":1.2,"bandwidth_gbps":250.4,"status":"ok","message":"CubeCL fused RMSNorm achieved 250 GB/s"}
{"id":"s10-4","parent_id":"s10","step":10,"phase":"gdn_chunk_backward","backend":"metal","start_ms":7.2,"duration_ms":9.4,"vram_mb":6490,"status":"ok","message":"Recomputed checkpointed activations"}
{"id":"s10-5","parent_id":"s10","step":10,"phase":"adamw_fused_update","backend":"metal","start_ms":16.6,"duration_ms":1.8,"grad_norm":0.86,"loss":1.942,"status":"ok"}
{"id":"s11","step":11,"phase":"train_step","backend":"metal","start_ms":19.0,"duration_ms":17.9,"tokens_per_sec":48600,"vram_mb":6395,"loss":1.889,"grad_norm":0.81,"status":"ok"}
{"id":"s11-1","parent_id":"s11","step":11,"phase":"gdn_chunk_forward","backend":"metal","start_ms":19.6,"duration_ms":4.9,"status":"ok"}
{"id":"s11-2","parent_id":"s11","step":11,"phase":"gdn_chunk_backward","backend":"metal","start_ms":24.5,"duration_ms":9.1,"status":"ok"}
{"id":"s11-3","parent_id":"s11","step":11,"phase":"adamw_fused_update","backend":"metal","start_ms":33.6,"duration_ms":1.7,"grad_norm":0.81,"loss":1.889,"status":"ok"}
{"id":"s12","step":12,"phase":"train_step","backend":"metal","start_ms":37.5,"duration_ms":29.6,"tokens_per_sec":29400,"vram_mb":7120,"loss":8.741,"grad_norm":142.8,"status":"rollback","message":"AMP grad_overflow detected (norm=142.8) -> rollback weights & halve loss_scale 65536->32768"}
{"id":"s12-1","parent_id":"s12","step":12,"phase":"gdn_chunk_forward","backend":"metal","start_ms":38.1,"duration_ms":5.3,"loss":8.741,"status":"warn","message":"Loss spike 4.6x above EMA baseline"}
{"id":"s12-2","parent_id":"s12","step":12,"phase":"gdn_chunk_backward","backend":"metal","start_ms":43.4,"duration_ms":15.8,"grad_norm":142.8,"status":"error","message":"FP16 gradient Inf/NaN in layer 18 attn_proj"}
{"id":"s12-3","parent_id":"s12","step":12,"phase":"checkpoint_rollback","backend":"metal","start_ms":59.2,"duration_ms":7.9,"loss_scale":32768,"status":"rollback","message":"Skipped optimizer step and restored clean state"}
{"id":"s13","step":13,"phase":"train_step","backend":"metal","start_ms":68.0,"duration_ms":18.1,"tokens_per_sec":48300,"vram_mb":6405,"loss":1.841,"grad_norm":0.79,"status":"ok","message":"Recovered stability after loss_scale backoff"}
{"id":"s13-1","parent_id":"s13","step":13,"phase":"gdn_chunk_forward","backend":"metal","start_ms":68.7,"duration_ms":5.0,"status":"ok"}
{"id":"s13-2","parent_id":"s13","step":13,"phase":"gdn_chunk_backward","backend":"metal","start_ms":73.7,"duration_ms":9.2,"status":"ok"}
{"id":"s13-3","parent_id":"s13","step":13,"phase":"adamw_fused_update","backend":"metal","start_ms":82.9,"duration_ms":1.8,"grad_norm":0.79,"loss":1.841,"status":"ok"}"#;

const SAMPLE_RIE_AGENT_TRACE: &str = r#"{"id":"ag-1","start_ms":0.0,"duration_ms":142.5,"level":"INFO","target":"rie_agent::runtime","name":"agent_turn","message":"Completed user query with guardrail protection","fields":{"route":"hybrid_small_first","cost_saved_pct":"57.5%"}}
{"id":"ag-2","parent_id":"ag-1","start_ms":0.5,"duration_ms":4.2,"level":"INFO","target":"rie_agent::router","name":"complexity_classifier","message":"Routed initial retrieval to fast local model","fields":{"model":"qwen2.5-7b-q4","confidence":0.91}}
{"id":"ag-3","parent_id":"ag-1","start_ms":5.0,"duration_ms":12.8,"level":"INFO","target":"rie_grep::hybrid","name":"tool_code_search","message":"Hybrid BM25 + Candle RRF fetched 8 chunks","fields":{"bm25_ms":0.14,"dense_ms":0.31,"hits":8}}
{"id":"ag-4","parent_id":"ag-1","start_ms":18.5,"duration_ms":3.1,"level":"INFO","target":"rie_eval::guardrail","name":"l1_regex_screen","message":"Input prompt passed L1 static rules","fields":{"layer":"L1"}}
{"id":"ag-5","parent_id":"ag-1","start_ms":22.0,"duration_ms":19.6,"level":"ERROR","target":"rie_eval::guardrail","name":"l3a_jev_indirect_injection","status":"blocked","message":"Blocked indirect prompt injection inside fetched web snippet","fields":{"layer":"L3A","score":0.97,"policy":"deny_tool_escalation"}}
{"id":"ag-6","parent_id":"ag-5","start_ms":41.8,"duration_ms":2.4,"level":"WARN","target":"rie_agent::sandbox","name":"tool_call_rollback","status":"rollback","message":"Rolled back unauthorized shell tool invocation","fields":{"blocked_tool":"exec_shell"}}
{"id":"ag-7","parent_id":"ag-1","start_ms":45.0,"duration_ms":8.4,"level":"INFO","target":"rie_agent::context","name":"sanitize_untrusted_context","message":"Stripped injected directives and rebuilt safe prompt","fields":{"dropped_tokens":148}}
{"id":"ag-8","parent_id":"ag-1","start_ms":54.0,"duration_ms":78.2,"level":"INFO","target":"rie_agent::llm","name":"generate_final_answer","message":"Streamed grounded answer with citations","fields":{"ttft_ms":16.4,"output_tokens":312}}
{"id":"ag-9","parent_id":"ag-8","start_ms":54.5,"duration_ms":16.4,"level":"INFO","target":"rie_agent::llm","name":"prefill_kv_cache","message":"KV cache prefix hit 74%","fields":{"cached_tokens":1240}}
{"id":"ag-10","parent_id":"ag-8","start_ms":71.2,"duration_ms":60.8,"level":"INFO","target":"rie_agent::llm","name":"decode_stream","message":"Generated 312 tokens at 51.3 tok/s","fields":{"tok_per_sec":51.3}}
{"id":"ag-11","parent_id":"ag-1","start_ms":133.0,"duration_ms":6.8,"level":"INFO","target":"rie_eval::online","name":"citation_faithfulness_check","message":"Verified 4/4 file citations match tool outputs","fields":{"faithfulness":1.0}}
{"id":"ag-12","parent_id":"ag-1","start_ms":140.0,"duration_ms":2.1,"level":"INFO","target":"rie_agent::telemetry","name":"flush_jsonl_sink","message":"Persisted 12 spans to local trace log","fields":{"bytes":3420}}"#;
