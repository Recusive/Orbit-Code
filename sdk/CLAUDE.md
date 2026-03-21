# sdk/

Client SDKs for embedding the Orbit Code agent into external applications. Contains TypeScript, Python, and Python runtime sub-packages.

## Architecture

All SDKs follow the same pattern: locate the `codex` CLI binary, spawn it as a child process (`codex exec` for TypeScript, `codex app-server` for Python), exchange structured messages over stdin/stdout, and expose typed Thread/Turn abstractions to the consumer.

## Module Layout

- **typescript/** -- `@orbit.build/orbit-code-sdk`: Node.js SDK using JSONL over stdio
- **python/** -- `orbit-code-app-server-sdk`: Python SDK using JSON-RPC v2 over stdio
- **python-runtime/** -- `orbit-code-cli-bin`: wheel-only package that bundles the platform-specific `codex` binary for Python distribution
