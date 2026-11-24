# Dioxus Trace Viewer

> **Zero-backend Rust `tracing` JSONL & `rie-train` benchmark waterfall profiler** — built with one shared **Dioxus 0.7** codebase compiling cleanly to both **Web WASM** (GitHub Pages, strict no-`'unsafe-eval'` CSP) and **Native Desktop**.

- **Live Web WASM Profiler**: [https://dioxuspress.github.io/dioxus-trace-viewer/](https://dioxuspress.github.io/dioxus-trace-viewer/)
- **Deep Architecture Case Study**: [https://rustineverything.app/case/dioxus-trace-viewer](https://rustineverything.app/case/dioxus-trace-viewer)

## Capabilities

- **Auto-detects 3 JSONL schemas locally** (zero upload, never leaves your machine):
  1. Rust `tracing-subscriber::fmt().json()` logs (supports `duration_ms` / `elapsed_ms` / `busy_ms` / `time.busy` human units like `430µs` / `12.4ms`, ISO-8601 timestamps, and nested `spans` / `parent_id` trees).
  2. `rie-train` GPU / training step benchmark JSONL (`step`, `phase`, `backend`, `tokens_per_sec`, `vram_mb`, `loss`, `grad_norm`, `status: "rollback"`).
  3. Generic OpenTelemetry-style Span JSONL (`id`, `parent_id`, `name`, `start_ms`, `duration_ms`, `level`, `fields`).
- **Interactive Span Waterfall & Inspector**: Nested tree indentation, vector SVG timeline bars, keyword + anomaly + `>= P95` quick filters, below-the-fold `content-visibility: auto`, and full field KV + raw JSON inspector.
- **Latency Distribution & Hotspots**: P50 / P95 / P99 percentile markers over a 12-bucket SVG histogram + cumulative-time hotspot ranking table with one-click waterfall filtering.
- **Anomaly & Rollback Triage**: Surfaces `ROLLBACK`, `ERROR`, `WARN`, and P95 tail outliers with ratio vs. P95 and direct jump-to-span navigation.
- **3 Built-in Realistic Datasets**: DioxusPress fullstack request chain (`Pingora` + `Axum` + `Tantivy` + `SeaORM`), `rie-train` Apple Metal AMP gradient overflow rollback, and `rie-agent` L3A indirect prompt injection guardrail trace.
- **Bilingual (中文 / EN) & Light/Dark Telemetry Themes** + keyboard-navigable ARIA tablist (`ArrowLeft`/`ArrowRight`/`Home`/`End`).

## Running Locally

### Web (WASM)

```bash
dx serve --platform web
```

### Native Desktop

```bash
cargo run --no-default-features --features desktop
# or
dx serve --platform desktop
```

### Tests & Verification

```bash
cargo fmt --all -- --check
cargo test --lib
cargo clippy --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo check --no-default-features --features desktop
```

## Architecture

- [`src/trace/`](src/trace/): Pure Rust, DOM-free parsing and statistics kernel (`model.rs`, `parser.rs`, `stats.rs`, `samples.rs`) tested directly on the host in milliseconds.
- [`src/views/`](src/views/): Shared Dioxus 0.7 RSX views (`waterfall.rs`, `latency.rs`, `anomalies.rs`, `importer.rs`) using Dioxus `FileEngine` so local `.jsonl` file loading works identically in Web WASM and Desktop builds.
- [`assets/main.css`](assets/main.css): `11 KB` standalone semantic CSS with zero `node_modules` dependency.

## License

MIT
