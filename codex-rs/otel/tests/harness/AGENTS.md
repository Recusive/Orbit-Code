# codex-rs/otel/tests/harness/

This file applies to `codex-rs/otel/tests/harness/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-otel` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-otel`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Shared test helpers for `codex-otel` integration tests.

### What this folder does

Provides utility functions used across multiple test suites to build in-memory metrics clients and inspect exported metric data.

### Key files

- `mod.rs` -- helper functions:
  - `build_metrics_with_defaults(default_tags)` -- creates a `MetricsClient` backed by `InMemoryMetricExporter` with optional default tags
  - `latest_metrics(exporter)` -- extracts the last `ResourceMetrics` from an in-memory exporter
  - `find_metric(resource_metrics, name)` -- locates a specific `Metric` by name in exported data
  - `attributes_to_map(attributes)` -- converts `KeyValue` iterator to `BTreeMap<String, String>` for assertions
  - `histogram_data(resource_metrics, name)` -- extracts bounds, bucket counts, sum, and count from a histogram metric

### Imports from

- `codex_otel::metrics` -- `MetricsClient`, `MetricsConfig`, `Result`
- `opentelemetry_sdk::metrics` -- `InMemoryMetricExporter`, metric data types

### Exports to

All functions are `pub(crate)` and used by test modules in `suite/`.
