import path from "node:path";

export function codexPathOverride() {
  return (
    process.env.ORBIT_EXECUTABLE ??
    path.join(process.cwd(), "..", "..", "codex-rs", "target", "debug", "codex")
  );
}
