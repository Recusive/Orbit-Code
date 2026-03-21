# codex-rs/utils/sleep-inhibitor/

Prevents system idle sleep during active agent turns using native platform APIs (macOS IOKit, Linux systemd-inhibit/gnome-session-inhibit, Windows PowerCreateRequest). Falls back to a no-op on unsupported platforms.

## Build & Test
```bash
cargo build -p orbit-code-utils-sleep-inhibitor
cargo test -p orbit-code-utils-sleep-inhibitor
```

## Key Considerations
- Platform backends are selected via conditional compilation; each lives in its own module (`macos.rs`, `linux_inhibitor.rs`, `windows_inhibitor.rs`, `dummy.rs`).
- `iokit_bindings.rs` contains generated FFI bindings -- do not hand-edit.
