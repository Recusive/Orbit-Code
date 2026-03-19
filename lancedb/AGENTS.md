# lancedb/

This file applies to `lancedb/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Read the files listed below before changing behavior in this subtree; keep neighboring docs and call sites consistent with any structural change.

## Validate
- Run the nearest package or crate tests that exercise this subtree.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Placeholder directory for LanceDB-related files. LanceDB is a vector database used for semantic search capabilities in the Codex CLI (e.g., file search and code indexing).

### Current State

This directory is currently empty. The actual LanceDB integration code lives in `codex-rs/lancedb/` (which contains the Rust bindings and Bazel build configuration for the LanceDB dependency).

### Relationship to Other Directories

- `codex-rs/lancedb/` contains the Rust-side LanceDB integration
- `codex-rs/file-search/` uses LanceDB for vector-based file search
