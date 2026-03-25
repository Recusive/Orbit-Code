# codex-rs/core/src/memories/

This file applies to `codex-rs/core/src/memories/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Memory extraction and consolidation pipeline for learning from past sessions.

### What this folder does

Implements a two-phase memory pipeline that runs at session startup to extract and consolidate learnings from past conversation sessions:

#### Phase 1: Startup Extraction
- Selects recent rollout files for processing
- Runs stage-1 extraction using `gpt-5.1-codex-mini` (low reasoning effort) to produce raw memories from each rollout
- Persists raw memories as markdown files
- Enqueues consolidation work
- Concurrency-limited to 8 parallel extraction jobs

#### Phase 2: Consolidation
- Claims a global consolidation lock
- Materializes consolidation inputs from raw stage-1 memories
- Dispatches a single consolidation agent using `gpt-5.3-codex` (medium reasoning effort)
- Produces a consolidated `memory_summary.md`

#### Key design decisions
- Jobs use lease-based ownership (1 hour leases) with retry backoff
- Phase-2 uses heartbeat-based liveness (90-second intervals)
- Memory artifacts stored under `$ORBIT_HOME/memories/`
- Thread scan limited to 5,000 threads
- Rollout truncation based on model context window (70% utilization target)

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations, constants for both phases, directory layout helpers |
| `start.rs` | `start_memories_startup_task()` -- single entry point for the memory pipeline |
| `phase1.rs` | Phase 1 extraction: rollout selection, raw memory extraction, job management |
| `phase2.rs` | Phase 2 consolidation: lock acquisition, input materialization, agent dispatch |
| `storage.rs` | Persistent storage for memory artifacts and job state |
| `prompts.rs` | System prompts for extraction and consolidation agents |
| `citations.rs` | Citation handling for memory sources |
| `control.rs` | `clear_memory_root_contents()` for resetting memory state |
| `usage.rs` | Token usage tracking for memory operations |
| `tests.rs` | Integration tests for the memory pipeline |

### Imports from

- `crate::rollout` -- Session rollout file access for extraction
- `crate::client` -- `ModelClient` for running extraction/consolidation agents
- `crate::config` -- Memory configuration settings
- `crate::truncate` -- Token estimation and rollout truncation
- `codex_protocol` -- `ReasoningEffort` for model configuration

### Exports to

- `crate::codex` -- `start_memories_startup_task()` called during session initialization
- `crate::config` -- `memory_root()` used for memory directory resolution
