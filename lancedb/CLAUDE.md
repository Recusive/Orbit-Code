# lancedb/

## Purpose

Placeholder directory for LanceDB-related files. LanceDB is a vector database used for semantic search capabilities in the Codex CLI (e.g., file search and code indexing).

## Current State

This directory is currently empty. The actual LanceDB integration code lives in `codex-rs/lancedb/` (which contains the Rust bindings and Bazel build configuration for the LanceDB dependency).

## Relationship to Other Directories

- `codex-rs/lancedb/` contains the Rust-side LanceDB integration
- `codex-rs/file-search/` uses LanceDB for vector-based file search
