# shell-tool-mcp/src/

This file applies to `shell-tool-mcp/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `@openai/codex-shell-tool-mcp` package. Keep `package.json` entry points, exports, and scripts aligned with source changes.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/shell-tool-mcp && pnpm build`
- `cd /Users/no9labs/Developer/Recursive/codex/shell-tool-mcp && pnpm test`
- `cd /Users/no9labs/Developer/Recursive/codex/shell-tool-mcp && pnpm format`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

TypeScript source code for the `@openai/codex-shell-tool-mcp` package. Handles platform detection and selection of the correct patched Bash binary from the bundled vendor directory.

### Key Files

| File | Role |
|------|------|
| `index.ts` | Entry point. Calls `resolveTargetTriple()` to get the platform, `readOsRelease()` for Linux distro info, and `resolveBashPath()` to select the correct Bash binary. Prints the resolved path. |
| `bashSelection.ts` | Core selection logic. `selectLinuxBash()` matches OS release info against known Linux variants (Ubuntu, Debian, CentOS/RHEL/Rocky/AlmaLinux) with version-aware fallback. `selectDarwinBash()` matches the Darwin kernel major version against macOS builds (15/14/13). `resolveBashPath()` is the unified entry point. |
| `constants.ts` | Defines the available Bash variant configurations: `LINUX_BASH_VARIANTS` (6 variants with distro ID and version matching) and `DARWIN_BASH_VARIANTS` (3 variants with minimum Darwin kernel version). |
| `osRelease.ts` | Parses `/etc/os-release` files to extract `ID`, `ID_LIKE`, and `VERSION_ID` fields. Returns an `OsReleaseInfo` object. Falls back to empty values if the file is not readable. |
| `platform.ts` | Maps Node.js `process.platform` and `process.arch` to Rust target triples (e.g., `linux` + `arm64` becomes `aarch64-unknown-linux-musl`). Supports linux and darwin, x64 and arm64. |
| `types.ts` | TypeScript type definitions: `LinuxBashVariant`, `DarwinBashVariant`, `OsReleaseInfo`, `BashSelection` |

### Data Flow

```
index.ts
  -> platform.ts (resolveTargetTriple)
  -> osRelease.ts (readOsRelease, Linux only)
  -> bashSelection.ts (resolveBashPath)
     -> constants.ts (variant definitions)
     -> types.ts (type definitions)
```

### Build Output

Compiled by `tsup` to `bin/mcp-server.js` (CommonJS format, targeting Node.js 18+).

### Test Coverage

Tests in `../tests/` validate:
- `bashSelection.test.ts`: Linux distro matching and Darwin version selection
- `osRelease.test.ts`: Parsing of `/etc/os-release` content
