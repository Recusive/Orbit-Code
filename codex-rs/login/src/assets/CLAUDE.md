# codex-rs/login/src/assets/

Static HTML templates for the login callback server's browser-facing responses.

## What this folder does

Contains HTML files that are embedded into the binary at compile time via `include_str!()` in `server.rs`. They are served as HTTP responses during the OAuth login callback flow.

## Key files

| File | Purpose |
|------|---------|
| `success.html` | Branded success page displayed after a successful OAuth login. Shows a confirmation message and redirects the user back to the CLI |
| `error.html` | Branded error page displayed when login fails. Contains template placeholders (`__ERROR_TITLE__`, `__ERROR_MESSAGE__`, `__ERROR_CODE__`, `__ERROR_DESCRIPTION__`, `__ERROR_HELP__`) that are replaced at runtime with HTML-escaped error details. Handles both generic errors and specific cases like missing Codex entitlement |

## Where it plugs in

- **Consumed by**: `server.rs` via `include_str!("assets/success.html")` and `include_str!("assets/error.html")`
- These files are compiled into the binary; no runtime file access is needed
