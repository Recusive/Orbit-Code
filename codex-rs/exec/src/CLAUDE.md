# codex-rs/exec/src/

Source for the `orbit-code-exec` crate (binary + library).

`main.rs` is the binary entry point with arg0 dispatch. `lib.rs` contains `run_main()` (config/OTEL/session setup) and `run_exec_session()` (event loop). `cli.rs` defines the clap `Cli` struct. `event_processor.rs` defines the `EventProcessor` trait and `CodexStatus` enum. Two implementations: `event_processor_with_human_output.rs` (colorized stderr) and `event_processor_with_jsonl_output.rs` (structured JSONL stdout). `exec_events.rs` defines the `ThreadEvent` wire types for JSONL output.
