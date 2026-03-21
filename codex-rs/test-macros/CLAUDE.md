# codex-rs/test-macros/

Proc-macro crate providing `#[large_stack_test]` -- runs test bodies on a dedicated thread with a 16 MB stack. Handles both sync and async tests (builds its own Tokio runtime for async).

## Build & Test
```bash
cargo build -p orbit-code-test-macros
cargo test -p orbit-code-test-macros
```

## Key Considerations
- Strips `#[tokio::test]` attributes and constructs its own multi-thread runtime with 2 workers for async tests.
- Automatically adds `#[test]` if neither `#[test]` nor `#[test_case]` is present.
