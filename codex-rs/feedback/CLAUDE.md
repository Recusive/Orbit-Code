# codex-rs/feedback/

User feedback collection and Sentry-based upload for the Codex CLI. Captures logs in a ring buffer and packages them with metadata tags for bug reports and experience ratings.

## Build & Test
```bash
cargo build -p orbit-code-feedback
cargo test -p orbit-code-feedback
```

## Architecture

`CodexFeedback` is a thread-safe feedback collector that provides two `tracing_subscriber` layers: a `logger_layer()` capturing full-fidelity TRACE-level logs into a 4 MB ring buffer, and a `metadata_layer()` collecting key/value tags from tracing events with target `"feedback_tags"`. When a user submits feedback, `FeedbackSnapshot::upload_feedback()` packages the buffered logs, tags, and connectivity diagnostics into a Sentry envelope and uploads it with a 10-second flush timeout.

## Key Considerations
- The Sentry DSN is hardcoded in the source -- do not change without coordinating with the backend team
- Ring buffer is capped at 4 MB (`DEFAULT_MAX_BYTES`) with oldest-byte eviction -- no unbounded growth
- Tags are limited to 64 entries (`MAX_FEEDBACK_TAGS`) -- tags emitted after the limit are silently dropped
- `FeedbackDiagnostics` collects environment-based connectivity info (OPENAI_BASE_URL, proxy vars) for debugging
- Classifications are fixed strings: "bug", "bad_result", "good_result", "safety_check"
- The `feedback_tags` tracing target is the contract between emitters (core, tui) and this crate's metadata layer
