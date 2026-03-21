# codex-rs/network-proxy/src/

Source for the `orbit-code-network-proxy` crate -- HTTP/SOCKS5 proxy with domain-based policy enforcement.

## Module Layout
- **Proxy lifecycle** (`proxy.rs`): `NetworkProxy`, `NetworkProxyBuilder`, `NetworkProxyHandle`; TCP listener reservation, task spawning, proxy environment variable constants
- **HTTP handling** (`http_proxy.rs`, `mitm.rs`, `certs.rs`): HTTP proxy with CONNECT tunneling and plain forwarding; MITM TLS interception for Limited mode HTTPS; dynamic per-domain certificate generation
- **SOCKS5** (`socks5.rs`): SOCKS5 proxy with optional UDP relay
- **Policy** (`config.rs`, `policy.rs`, `network_policy.rs`, `state.rs`): `NetworkProxyConfig`/`NetworkMode`; globset-based domain matching; `NetworkPolicyDecider` trait; config validation and runtime state
- **Runtime** (`runtime.rs`): `ConfigReloader` for hot-reload; `BlockedRequestObserver` for audit; runtime state management
- **Plumbing** (`upstream.rs`, `responses.rs`, `reasons.rs`): Upstream proxy chaining; HTTP error responses; human-readable block reason formatting
