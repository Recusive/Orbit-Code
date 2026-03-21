# codex-rs/process-hardening/src/

Source for the `orbit-code-process-hardening` crate -- single-file platform-conditional security hardening.

## Module Layout
- **Single file** (`lib.rs`): `pre_main_hardening()` dispatching to platform-specific functions (`pre_main_hardening_linux`, `pre_main_hardening_macos`, `pre_main_hardening_bsd`, `pre_main_hardening_windows`); `set_core_file_size_limit_to_zero()` helper; `env_keys_with_prefix()` for environment variable filtering
