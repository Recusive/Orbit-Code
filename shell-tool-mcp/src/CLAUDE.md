# shell-tool-mcp/src/

TypeScript source for the `@orbit.build/orbit-code-shell-tool-mcp` package. Handles platform detection and selection of the correct patched Bash binary.

## Module Layout

- **Entry point**: `index.ts` -- resolves platform, reads OS info, selects bash binary, prints path
- **Selection logic**: `bashSelection.ts` -- distro/version matching for Linux and Darwin kernel version matching for macOS
- **Platform detection**: `platform.ts` (target triple mapping), `osRelease.ts` (`/etc/os-release` parser)
- **Configuration**: `constants.ts` (Linux and Darwin bash variant definitions)
- **Types**: `types.ts` (LinuxBashVariant, DarwinBashVariant, OsReleaseInfo, BashSelection)
