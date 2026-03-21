# codex-cli/bin/

Executable entry points shipped inside the `@orbit.build/orbit-code` npm package.

## Module Layout

- **codex.js** -- ESM launcher that detects OS/arch, resolves the native binary from vendor or optional dependency packages, and spawns it with signal forwarding. This is the `"bin"` entry in package.json.
- **rg** -- DotSlash manifest (JSON) for fetching ripgrep 15.1.0 binaries per platform. Consumed by the build system, not executed directly.
