# codex-rs/network-proxy/

HTTP and SOCKS5 proxy server that enforces domain-based network access policies for sandboxed commands. Built on the `rama` async networking framework.

## Build & Test
```bash
cargo build -p orbit-code-network-proxy
cargo test -p orbit-code-network-proxy
```

## Architecture

The proxy sits between sandboxed shell commands and the network, enforcing allow/deny lists per domain. It operates in two modes: Full (all HTTP methods allowed) and Limited (GET/HEAD/OPTIONS only, with MITM TLS interception to inspect HTTPS method usage). The `NetworkProxyBuilder` constructs a `NetworkProxy` that spawns HTTP and SOCKS5 listeners on reserved TCP ports. Domain matching uses `globset` for wildcard patterns. The proxy supports upstream proxy chaining for corporate environments, Unix socket proxying for container daemons, and dynamic policy reloading at runtime via `ConfigReloader`.

## Key Considerations
- Uses `rama-*` crates at pinned alpha versions (`=0.3.0-alpha.4`) -- these are not stable and version bumps may require API changes
- `rama-unix` is only included on Unix targets (`cfg(target_family = "unix")`)
- MITM TLS interception generates per-domain certificates signed by a local CA -- this only activates in Limited mode for HTTPS
- `#![deny(clippy::print_stdout, clippy::print_stderr)]` is set at crate root -- all output must go through `tracing`
- `BlockedRequestObserver` provides an audit trail of denied requests for logging/UI
- The proxy sets environment variables (`HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY`, etc.) on sandboxed child processes -- see `PROXY_URL_ENV_KEYS` and `ALL_PROXY_ENV_KEYS` constants
- `clamp_bind_addrs()` enforces loopback binding when Unix sockets are enabled for security
