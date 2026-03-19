# codex-rs/utils/pty/src/win/

Windows-specific ConPTY implementation, vendored and modified from WezTerm (MIT license).

## What this folder does

Provides a Windows ConPTY (Console Pseudo Terminal) backend that implements the `portable-pty` traits (`PtySystem`, `Child`, `ChildKiller`). This allows Codex to spawn interactive processes on Windows with proper terminal emulation.

## Key modifications from upstream WezTerm

- **Bug fix #13945**: Corrected inverted `TerminateProcess` return value check in `WinChild::do_kill` and `WinChildKiller::kill` -- Win32 returns nonzero on success, but the original code treated 0 as success.

## Key types

- `WinChild` -- implements `portable_pty::Child` and `ChildKiller`; wraps a process handle for exit code checking, waiting, and killing
- `WinChildKiller` -- clonable killer that terminates a process via `TerminateProcess`
- `ConPtySystem` -- implements `portable_pty::PtySystem` for Windows ConPTY
- `RawConPty` -- raw ConPTY handle (exported for use in lib.rs)

## Key files

- `mod.rs` -- `WinChild`, `WinChildKiller` implementations with corrected kill semantics; re-exports `ConPtySystem` and `conpty_supported`
- `conpty.rs` -- `ConPtySystem` PTY system implementation using Windows ConPTY API
- `psuedocon.rs` -- lower-level pseudoconsole wrapper; `conpty_supported()` detection
- `procthreadattr.rs` -- Windows process thread attribute list management for ConPTY
