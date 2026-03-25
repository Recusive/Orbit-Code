# codex-rs/core/src/rollout/

This file applies to `codex-rs/core/src/rollout/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Session rollout persistence, discovery, listing, and indexing.

### What this folder does

Manages the durable persistence of session data (rollouts). Every event in a Codex session is recorded to rollout files, enabling session resume, forking, and historical browsing.

Key responsibilities:
- **Recorder** (`recorder.rs`): `RolloutRecorder` -- writes rollout items (events, response items, metadata) to append-only files organized by date.
- **Listing** (`list.rs`): Discovers and lists saved sessions with pagination, sorting, and filtering. Supports both active (`sessions/`) and archived (`archived_sessions/`) directories.
- **Session index** (`session_index.rs`): Name-based session indexing for fast lookup. Maps thread names/IDs to rollout file paths.
- **Metadata** (`metadata.rs`): Reads and writes session metadata (source, model, timestamp) from rollout file headers.
- **Policy** (`policy.rs`): `EventPersistenceMode` -- controls which events are persisted (all, none, metadata-only).
- **Truncation** (`truncation.rs`): Handles rollout file size management and truncation.
- **Error handling** (`error.rs`): Specialized error types for session initialization failures.

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations, constants (`SESSIONS_SUBDIR`, `ARCHIVED_SESSIONS_SUBDIR`), re-exports |
| `recorder.rs` | `RolloutRecorder` -- append-only session event persistence |
| `list.rs` | Session discovery, listing, pagination, `find_thread_path_by_id_str()` |
| `session_index.rs` | Name-based indexing, `append_thread_name()`, `find_thread_name_by_id()` |
| `metadata.rs` | `SessionMeta` reading/writing from rollout headers |
| `policy.rs` | Event persistence mode configuration |
| `truncation.rs` | Rollout file truncation management |
| `error.rs` | Session initialization error mapping |
| `tests.rs` | Integration tests |

### Directory layout

```
$ORBIT_HOME/
  sessions/
    YYYY/MM/DD/
      <thread_id>.jsonl    # Active rollout files
  archived_sessions/
    YYYY/MM/DD/
      <thread_id>.jsonl    # Archived rollout files
```

### Imports from

- `codex_protocol` -- `SessionSource`, `SessionMeta`, `RolloutItem`
- `crate::config` -- Session persistence settings

### Exports to

- `crate::codex` -- `RolloutRecorder` used for event persistence during sessions
- `crate::codex::rollout_reconstruction` -- Reads rollouts for resume/fork
- `crate::memories` -- Reads rollouts for memory extraction
- Public API: `RolloutRecorder`, `SessionMeta`, listing functions, thread name utilities
