# codex-rs/utils/elapsed/src/

Source directory for the `codex-utils-elapsed` crate.

## Key files

- `lib.rs` -- single-file implementation containing:
  - `format_elapsed(start_time: Instant) -> String` -- convenience wrapper
  - `format_duration(duration: Duration) -> String` -- public API
  - `format_elapsed_millis(millis: i64) -> String` -- internal formatter with rules: `<1000ms` -> `"Xms"`, `<60000ms` -> `"X.XXs"`, `>=60000ms` -> `"Xm YYs"`
  - Tests covering sub-second, second-range, minute-range, and boundary cases
