# app-server/tests

Integration test suite for the `orbit-code-app-server` crate. Tests exercise the full server stack by spawning a real app-server process (or using in-process transport) and communicating over JSON-RPC via stdio or WebSocket.
