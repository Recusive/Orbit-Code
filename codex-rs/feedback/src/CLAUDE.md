# codex-rs/feedback/src/

Source for the `orbit-code-feedback` crate -- ring-buffer log capture with Sentry upload.

## Module Layout
- **Core** (`lib.rs`): `CodexFeedback` collector with ring buffer and tag map; `FeedbackSnapshot` with `upload_feedback()` and `save_to_temp_file()`; `FeedbackMakeWriter`/`FeedbackWriter` implementing `tracing_subscriber::fmt::MakeWriter`; `FeedbackMetadataLayer` for tag extraction from tracing events
- **Diagnostics** (`feedback_diagnostics.rs`): `FeedbackDiagnostics` collecting connectivity info from environment variables; `FeedbackDiagnostic` formatting for Sentry attachments
