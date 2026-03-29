# Collaboration Modes — Accept & Bypass

## Context

Orbit Code currently couples the sandbox (filesystem access scope) with the approval system (permission dialogs). When a user sets `sandbox_mode = "danger-full-access"`, the approval prompts also stop firing — because `render_decision_for_unmatched_command()` in `core/src/exec_policy.rs:538-617` interprets `FileSystemSandboxKind::Unrestricted` as "just run commands without asking."

This plan adds two new collaboration modes to `ModeKind` — **Accept** and **Bypass** — that control approval posture independently of the sandbox. Sandbox and collaboration mode are fully orthogonal:

- **Sandbox** = WHERE (filesystem scope): on → repo only, off → full system
- **Collaboration mode** = HOW (workflow + approval posture): Default, Accept, Bypass, Plan

Any mode works with any sandbox setting. Accept in a sandbox still auto-approves writes — within the sandbox boundary. Bypass in a sandbox runs everything — within the sandbox boundary.

**Reference:** Claude Code implements permission posture via `permission_mode` field (`default`, `acceptEdits`, `plan`, `dontAsk`, `bypassPermissions`) passed to hooks. Orbit Code already has a superior approval mechanism (in-process oneshot channels + TUI overlay) that just needs its decision logic extended for new modes.

**Last verified against codebase:** 2026-03-28

---

## Authoritative Behavior Matrix

This is the single source of truth. All prose, pseudocode, and tests must match this table. "Existing logic" means the current `render_decision_for_unmatched_command()` / `assess_patch_safety()` / `default_exec_approval_requirement()` code runs unchanged.

| Mode | Read tools (`ls`, `grep`, `find`) | Write tools (edit, write, apply_patch) | Non-dangerous bash (`npm test`) | Dangerous bash (`rm -rf`) | MCP / other tools |
|------|----------------------------------|---------------------------------------|--------------------------------|--------------------------|-------------------|
| **Default** | Allow | Prompt | Allow | Prompt | Existing logic |
| **Accept** | Allow | Auto-approve | Auto-approve | Prompt | Auto-approve |
| **Bypass** | Allow | Auto-approve | Auto-approve | Auto-approve | Auto-approve |
| **Plan** | Unchanged (no tools) | Unchanged (no tools) | Unchanged (no tools) | Unchanged (no tools) | Unchanged (no tools) |

Key points:
- **Default** prompts for anything that writes or could be destructive. Read-only operations are free.
- **Accept** auto-approves everything except dangerous bash commands (irreversible operations like `rm -rf`, `git push --force`). Those still prompt.
- **Bypass** auto-approves everything. No prompts. Equivalent to current "Full Access" behavior.
- **Plan** is unchanged — no tool execution, planning only.
- **Sandbox is independent.** Modes control approval posture. Sandbox controls filesystem scope. They compose freely.

---

## Why ModeKind, Not a Separate Abstraction

We intentionally expand `ModeKind` rather than creating a new `PermissionMode` axis:

1. **ModeKind already controls tool execution.** Plan mode prevents tool calls entirely — that's an approval decision, not a workflow decision. Adding Accept/Bypass extends this existing axis.

2. **One UI surface.** Users switch modes via Shift+Tab and `/collab`. Having two independent mode systems (collaboration + permission) creates contradictory UI state and requires new Op transport, session state, and event plumbing for the second axis. ModeKind already has all this infrastructure.

3. **The independent selection buys nothing.** Plan overrides everything, so Plan + Accept = Plan + Default = Plan. The useful combinations reduce to a single cycle: Default → Accept → Bypass → Plan.

4. **Existing infrastructure rides for free.** `CollaborationModeMask`, `set_collaboration_mask()`, Shift+Tab cycling, `collaborationMode/list` API, `TurnStartedEvent.collaboration_mode_kind` — all work with new ModeKind variants without new plumbing.

5. **Hook `permission_mode` already maps from mode state.** The hook system derives `permission_mode` as a projection — it doesn't need a separate source of truth.

**Trade-offs accepted:** `ModeKind` variants now appear in `TurnStartedEvent.collaboration_mode_kind`, `collaborationMode/list` API, and developer-instruction preset resolution. Every consumer of `ModeKind` must handle the new variants. The consumers are enumerated in Phase 2 and Phase 7.

---

## What Exists Today vs What Changes

### What exists today

| Concept | Current Location | Current Behavior |
|---|---|---|
| `ModeKind` | `protocol/src/config_types.rs:314` | Enum: `Plan`, `Default`, `PairProgramming` (hidden), `Execute` (hidden) |
| `TUI_VISIBLE_COLLABORATION_MODES` | `protocol/src/config_types.rs` | `[ModeKind::Default, ModeKind::Plan]` — 2-element array |
| `CollaborationMode` | `protocol/src/config_types.rs:356` | Struct: `mode: ModeKind`, `settings: Settings` |
| `CollaborationModeMask` | `protocol/src/config_types.rs:434` | Preset mask applied at runtime via Shift+Tab / `set_collaboration_mask()` |
| Collaboration presets | `core/src/models_manager/collaboration_mode_presets.rs` | Two presets: `plan_preset()`, `default_preset()` |
| Collaboration templates | `core/templates/collaboration_mode/` | `default.md`, `plan.md`, `pair_programming.md`, `execute.md` |
| `ApprovalPreset` / `/permissions` | `utils/approval-presets/src/lib.rs`, `tui/src/chatwidget.rs` | Three presets: "Read Only", "Default", "Full Access". Popup applies `AskForApproval` + `SandboxPolicy`. |
| `render_decision_for_unmatched_command()` | `core/src/exec_policy.rs:538-617` | Decides based on `AskForApproval` + `SandboxPolicy` — no mode awareness |
| `default_exec_approval_requirement()` | `core/src/tools/sandboxing.rs:167` | Decides based on `AskForApproval` + `FileSystemSandboxPolicy` — no mode awareness |
| `ExecApprovalRequest` | `core/src/exec_policy.rs:196` | No `mode_kind` field |
| Hook `permission_mode` | `core/src/hook_runtime.rs:267` | Derived from `AskForApproval` — `Never` → `"bypassPermissions"`, else → `"default"` |
| Stop hook `permission_mode` | `core/src/codex.rs:5760` | Inline match on `AskForApproval` — duplicates hook_runtime logic |
| Model instructions | `protocol/src/models.rs:475-663` | `DeveloperInstructions::from()` selects approval_policy templates; `from_collaboration_mode()` wraps developer_instructions |
| App-server API | `app-server-protocol/src/protocol/common.rs:409` | `collaborationMode/list` (experimental) |
| `TurnStartedEvent` | `protocol/src/protocol.rs:1772` | Contains `collaboration_mode_kind: ModeKind` |
| `TurnStartedNotification` | `app-server-protocol/src/protocol/v2.rs` | Manually mapped from `TurnStartedEvent` in `bespoke_event_handling.rs` |
| Shift+Tab cycling | `tui/src/collaboration_modes.rs` | Cycles `TUI_VISIBLE_COLLABORATION_MODES` via `next_mask()` |

### What this plan changes

| Concept | Change |
|---|---|
| `ModeKind` | **Add Accept, Bypass variants.** Update `display_name()`, `is_tui_visible()`, exhaustive matches. |
| `TUI_VISIBLE_COLLABORATION_MODES` | **Expand to 4 elements:** `[Default, Accept, Bypass, Plan]` |
| Collaboration presets | **Add `accept_preset()`, `bypass_preset()`** with their own `developer_instructions` |
| Collaboration templates | **New files:** `accept.md`, `bypass.md` under `core/templates/collaboration_mode/` |
| `ExecApprovalRequest` | **New field:** `mode_kind: ModeKind` |
| `render_decision_for_unmatched_command()` | **New parameter:** `mode_kind`. Branch on Accept/Bypass. |
| `default_exec_approval_requirement()` | **New parameter:** `mode_kind`. Bypass short-circuits. |
| Orchestrator | **Mode override:** Accept auto-approves write tools, Bypass auto-approves all. |
| Model instructions | **Accept/Bypass suppress approval_policy prompt.** Developer_instructions from preset handle permission guidance. Sandbox prompt remains. |
| Hook `permission_mode` | **Derived from `ModeKind`**, not `AskForApproval`. |
| `/permissions` popup | **Becomes sandbox toggle.** No longer drives approval posture — that's the collaboration mode's job. |
| Plan mode | **Unchanged.** Existing behavior preserved exactly. |
| All existing transport | **Unchanged.** `Op::UserTurn`, `SessionSettingsUpdate`, `AppEvent`, `AppCommand`, `TurnStartedEvent.collaboration_mode_kind` — all carry ModeKind already. |

---

## Sandbox Independence

### Current coupling problem

Today, `render_decision_for_unmatched_command()` checks `FileSystemSandboxKind::Unrestricted` and interprets it as "don't ask for approval." This means setting `DangerFullAccess` sandbox also disables approval prompts — the two concerns are fused.

### Fix

The decision logic must check `mode_kind` for approval posture and the sandbox for filesystem scope. They are independent:

```
Approval decision = f(mode_kind, command_danger_level)
Filesystem scope  = f(sandbox_policy)
```

In Default mode, the existing `approval_policy` + `sandbox_policy` logic continues to run as-is (preserving current behavior). In Accept/Bypass mode, `mode_kind` determines approval posture directly — the sandbox only affects what paths the tool can reach.

### `/permissions` popup rework

The `/permissions` popup currently conflates sandbox scope with approval posture (the "Full Access" preset sets both `DangerFullAccess` and `Never`). After this change:

- `/permissions` becomes a **sandbox toggle**: sandbox on (repo-scoped) vs sandbox off (full system access)
- Approval posture is controlled by collaboration mode (Shift+Tab: Default → Accept → Bypass → Plan)
- The existing Guardian Approvals, full-access confirmation warnings, and Windows sandbox setup flows remain on the `/permissions` popup since they are sandbox concerns, not approval concerns

---

## Model-Facing Instructions

### Current architecture

Model-facing permission instructions are assembled in `protocol/src/models.rs` via:
- `DeveloperInstructions::from()` / `from_policy()` — selects from `include_str!` templates in `protocol/src/prompts/permissions/approval_policy/` and `protocol/src/prompts/permissions/sandbox_mode/`
- `DeveloperInstructions::from_collaboration_mode()` — wraps collaboration mode `developer_instructions` in `<collaboration_mode>` tags

These run based on `approval_policy` and `sandbox_policy` values in `TurnContext`, independent of `ModeKind`.

### Problem

The existing `AskForApproval` prompt templates do NOT describe Accept/Bypass behavior:
- `OnRequest` template (`on_request_rule.md`) describes sandbox escalation, prefix rules, and `sandbox_permissions` parameters — none of which apply in Accept mode
- `Never` template (`never.md`) says "commands will be rejected" — the opposite of Bypass

### Solution

In Accept and Bypass mode, **suppress the approval_policy prompt entirely**. The collaboration mode `developer_instructions` (from `core/templates/collaboration_mode/accept.md` / `bypass.md`) handle all permission-related guidance instead. The `sandbox_policy` prompt remains — it accurately describes the filesystem scope regardless of mode.

This means:
- **Accept mode** → model sees: sandbox prompt + Accept developer_instructions (file ops auto, dangerous prompts)
- **Bypass mode** → model sees: sandbox prompt + Bypass developer_instructions (everything auto, no prompts)
- **Default mode** → existing approval + sandbox prompts unchanged
- **Plan mode** → unchanged

The collaboration mode templates in Phase 2.1 (`accept.md`, `bypass.md`) must cover the permission guidance that the suppressed approval prompt would have provided.

---

## System Design — Data Flow

```
TUI / App-server
     │
     ├── User presses Shift+Tab or types /collab or /accept or /bypass
     │   └── collaboration_modes::next_mask() cycles Default → Accept → Bypass → Plan
     │       └── set_collaboration_mask() applies the mask (existing infrastructure)
     │
     ▼
CollaborationMode.mode = ModeKind::Accept (runtime session state, existing field)
     │
     ├── Flows through existing Op::UserTurn / Op::OverrideTurnContext / SessionSettingsUpdate
     │   (no new transport needed — ModeKind is already carried on these paths)
     │
     ▼
Turn starts → TurnContext built (core/src/codex.rs)
     │
     ├── TurnContext.collaboration_mode.mode = ModeKind::Accept (existing field)
     │
     ├── Model instructions: mode selects developer_instructions from preset
     │   → Accept/Bypass: suppress approval_policy prompt, use mode developer_instructions
     │   → Default/Plan: existing prompts unchanged
     │
     ├── Hook permission_mode: derived from ModeKind
     │
     ▼
Tool call arrives
     │
     ├── exec_policy .rules evaluation (always first)
     │   → if matched: use rule decision
     │   → if unmatched: fall through
     │
     ├── render_decision_for_unmatched_command() with mode_kind
     │   (3 callsites: codex.rs, shell handler, unix_escalation)
     │
     ├── default_exec_approval_requirement() with mode_kind
     │   (2 callsites in orchestrator.rs: initial + retry)
     │
     └── orchestrator mode override (ToolCategory-based)
         (Accept auto-approves write tools, Bypass auto-approves all)
```

---

## Phase 1: Protocol Types

### 1.1 Extend `ModeKind` enum

**File:** `protocol/src/config_types.rs:314`

```rust
pub enum ModeKind {
    Plan,
    #[default]
    Default,
    #[serde(alias = "accept-edits", alias = "acceptEdits")]
    Accept,
    #[serde(alias = "bypass-permissions", alias = "bypassPermissions")]
    Bypass,
    #[doc(hidden)] PairProgramming,
    #[doc(hidden)] Execute,
}
```

Update all match arms exhaustively (Rule 33):
- `display_name()` — Accept → `"Accept"`, Bypass → `"Bypass"`
- `is_tui_visible()` — include Accept and Bypass → `true`
- `TUI_VISIBLE_COLLABORATION_MODES` — 4-element array: `[Default, Accept, Bypass, Plan]`

### 1.2 Add helper methods

```rust
impl ModeKind {
    pub const fn auto_approves_writes(self) -> bool {
        matches!(self, Self::Accept | Self::Bypass)
    }
    pub const fn auto_approves_dangerous(self) -> bool {
        matches!(self, Self::Bypass)
    }
}
```

### 1.3 Add `ToolCategory` enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolCategory { ReadOnly, WriteMutation, Shell, Other }
```

### 1.4 Schema regeneration

```bash
just write-config-schema
just write-app-server-schema
just write-app-server-schema --experimental
```

---

## Phase 2: Collaboration Mode Presets + Templates

### 2.1 Add template files

**New files:** `core/templates/collaboration_mode/accept.md`, `core/templates/collaboration_mode/bypass.md`

Follow the existing pattern from `default.md`: include `<collaboration_mode>` marker, `{{KNOWN_MODE_NAMES}}` placeholder, and mode-specific behavior guidance.

Content guidelines:
- `accept.md` — "Accept mode is active. File edits and writes are auto-approved. Non-dangerous shell commands are auto-approved. Dangerous commands (rm -rf, git push --force, etc.) require user approval — do not attempt to bypass this. Read-only operations are unrestricted."
- `bypass.md` — "Bypass mode is active. All tools and commands are auto-approved. Proceed autonomously without requesting approval. The user has granted full trust for this session."

### 2.2 Update `core/BUILD.bazel`

Currently exports `default.md` and `plan.md`. Add:
```python
"templates/collaboration_mode/accept.md",
"templates/collaboration_mode/bypass.md",
```

### 2.3 Add presets with `developer_instructions`

**File:** `core/src/models_manager/collaboration_mode_presets.rs`

Presets MUST populate `developer_instructions: Some(Some(...))` — setting `None` breaks `app-server/src/orbit_code_message_processor.rs:595-611` `normalize_turn_start_collaboration_mode()`.

```rust
pub(crate) fn builtin_collaboration_mode_presets() -> Vec<CollaborationModeMask> {
    vec![default_preset(), accept_preset(), bypass_preset(), plan_preset()]
}

fn accept_preset() -> CollaborationModeMask {
    CollaborationModeMask {
        name: ModeKind::Accept.display_name().to_string(),
        mode: Some(ModeKind::Accept),
        model: None,
        reasoning_effort: None,
        developer_instructions: Some(Some(accept_mode_instructions())),
    }
}

fn bypass_preset() -> CollaborationModeMask {
    CollaborationModeMask {
        name: ModeKind::Bypass.display_name().to_string(),
        mode: Some(ModeKind::Bypass),
        model: None,
        reasoning_effort: None,
        developer_instructions: Some(Some(bypass_mode_instructions())),
    }
}
```

Load via `include_str!("../../templates/collaboration_mode/accept.md")` and `include_str!("../../templates/collaboration_mode/bypass.md")`.

### 2.4 Mirror in `tui_app_server/src/model_catalog.rs`

Add identical presets with identical developer_instructions.

---

## Phase 3: ModeKind Threading

### 3.1 Add `mode_kind` to `ExecApprovalRequest`

**File:** `core/src/exec_policy.rs:196`

```rust
pub(crate) struct ExecApprovalRequest<'a> {
    pub(crate) command: &'a [String],
    pub(crate) mode_kind: ModeKind,  // NEW
    pub(crate) approval_policy: AskForApproval,
    pub(crate) sandbox_policy: &'a SandboxPolicy,
    pub(crate) file_system_sandbox_policy: &'a FileSystemSandboxPolicy,
    pub(crate) sandbox_permissions: SandboxPermissions,
    pub(crate) prefix_rule: Option<Vec<String>>,
}
```

### 3.2 Update ALL 3 construction sites

| Site | File | Value |
|------|------|-------|
| 1 | `core/src/codex.rs` | `turn_context.collaboration_mode.mode` |
| 2 | `core/src/unified_exec/process_manager.rs:599` | `context.turn.collaboration_mode.mode` |
| 3 | `core/src/tools/handlers/shell.rs:431` | `turn.collaboration_mode.mode` |

### 3.3 Update `unix_escalation.rs` direct fallback

**File:** `core/src/tools/runtimes/shell/unix_escalation.rs:784-792`

Thread `mode_kind` to the function's parameter list and pass to `render_decision_for_unmatched_command()`.

### 3.4 Add `ToolCategory` to `ToolRuntime` trait

**File:** `core/src/tools/sandboxing.rs`

```rust
fn tool_category(&self) -> ToolCategory { ToolCategory::Other }
```

Implement: `shell.rs` → Shell, `unified_exec.rs` → Shell, `apply_patch.rs` → WriteMutation.

---

## Phase 4: Decision Logic

### 4.1 Rewrite `render_decision_for_unmatched_command()`

**File:** `core/src/exec_policy.rs:538-617`

Must match the Authoritative Behavior Matrix exactly:

```rust
pub fn render_decision_for_unmatched_command(
    mode_kind: ModeKind,
    approval_policy: AskForApproval,
    sandbox_policy: &SandboxPolicy,
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    command: &[String],
    sandbox_permissions: SandboxPermissions,
    used_complex_parsing: bool,
) -> Decision {
    // Safe read commands: always allow (all modes except Plan)
    if is_known_safe_command(command) && !used_complex_parsing {
        return Decision::Allow;
    }

    // Dangerous commands
    if command_might_be_dangerous(command) {
        return match mode_kind {
            // Bypass: auto-approve everything including dangerous
            ModeKind::Bypass => Decision::Allow,
            // Accept: dangerous commands still prompt
            ModeKind::Accept => Decision::Prompt,
            // Default + hidden variants: existing behavior
            ModeKind::Default | ModeKind::PairProgramming | ModeKind::Execute => {
                existing_dangerous_command_logic(approval_policy)
            }
            // Plan: should not reach here (tools blocked upstream)
            ModeKind::Plan => Decision::Forbidden,
        };
    }

    // Non-dangerous commands
    match mode_kind {
        // Bypass: auto-approve
        ModeKind::Bypass => Decision::Allow,
        // Accept: auto-approve non-dangerous
        ModeKind::Accept => Decision::Allow,
        // Default + hidden: existing logic
        ModeKind::Default | ModeKind::PairProgramming | ModeKind::Execute => {
            render_decision_for_default_mode(
                approval_policy, sandbox_policy, file_system_sandbox_policy,
                command, sandbox_permissions, used_complex_parsing,
            )
        }
        ModeKind::Plan => Decision::Forbidden,
    }
}
```

Extract `render_decision_for_default_mode()` — exact copy of current lines 550-616. The Windows `runtime_sandbox_provides_safety` check is preserved.

### 4.2 Update `default_exec_approval_requirement()` — BOTH callsites

**File:** `core/src/tools/sandboxing.rs:167`

Add `mode_kind: ModeKind` parameter. Bypass short-circuits to skip all approval. Accept falls through — its write-tool override happens in the orchestrator (Phase 4.3).

```rust
pub(crate) fn default_exec_approval_requirement(
    mode_kind: ModeKind,
    policy: AskForApproval,
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
) -> ExecApprovalRequirement {
    match mode_kind {
        ModeKind::Bypass => {
            return ExecApprovalRequirement::Skip {
                bypass_sandbox: false,
                proposed_execpolicy_amendment: None,
            };
        }
        ModeKind::Accept | ModeKind::Default | ModeKind::Plan
        | ModeKind::PairProgramming | ModeKind::Execute => {}
    }
    // ... existing logic unchanged ...
}
```

Note: `bypass_sandbox: false` — sandbox scope is independent of mode. The sandbox continues to enforce filesystem boundaries regardless of approval posture.

**Callsite 1:** `orchestrator.rs:121` (initial approval)
**Callsite 2:** `orchestrator.rs:252` (retry after sandbox denial)

Both receive `turn_ctx` and can access `turn_ctx.collaboration_mode.mode`.

### 4.3 Orchestrator mode override

**File:** `core/src/tools/orchestrator.rs` — after requirement resolution at line ~123

```rust
let requirement = match (turn_ctx.collaboration_mode.mode, tool.tool_category()) {
    // Bypass: auto-approve everything
    (ModeKind::Bypass, _) => ExecApprovalRequirement::Skip {
        bypass_sandbox: false, proposed_execpolicy_amendment: None,
    },
    // Accept: auto-approve write tools (edit, write, apply_patch)
    (ModeKind::Accept, ToolCategory::WriteMutation) => ExecApprovalRequirement::Skip {
        bypass_sandbox: false, proposed_execpolicy_amendment: None,
    },
    // Accept: auto-approve non-dangerous shell (dangerous handled by exec_policy)
    (ModeKind::Accept, ToolCategory::Shell) => ExecApprovalRequirement::Skip {
        bypass_sandbox: false, proposed_execpolicy_amendment: None,
    },
    // Accept: auto-approve other tools (MCP, etc.)
    (ModeKind::Accept, ToolCategory::Other) => ExecApprovalRequirement::Skip {
        bypass_sandbox: false, proposed_execpolicy_amendment: None,
    },
    _ => requirement,
};
```

Note: Accept + Shell skips approval here, but `render_decision_for_unmatched_command()` (Phase 4.1) already returned `Decision::Prompt` for dangerous commands before the orchestrator runs. The orchestrator override only applies to commands that passed the exec_policy check.

### 4.4 Model-facing instruction override

**File:** `core/src/codex.rs` — where model prompt is assembled

In Accept and Bypass mode, **suppress** the `approval_policy` prompt entirely. The collaboration mode developer_instructions handle permission guidance.

```rust
let mode_kind = turn_context.collaboration_mode.mode;

// Approval prompt: suppress for Accept/Bypass (developer_instructions handle it)
let approval_prompt = match mode_kind {
    ModeKind::Accept | ModeKind::Bypass => None,
    ModeKind::Default | ModeKind::Plan
    | ModeKind::PairProgramming | ModeKind::Execute => {
        Some(DeveloperInstructions::from_policy(
            // ... existing params ...
        ))
    }
};

// Sandbox prompt: always included (sandbox is independent of mode)
let sandbox_prompt = build_sandbox_policy_prompt(turn_context.sandbox_mode());
```

---

## Phase 5: Hook Permission Mode

### 5.1 Centralize `hook_permission_mode()`

**File:** `core/src/hook_runtime.rs:267-276`

Replace AskForApproval-based mapping with ModeKind-based:

```rust
pub(crate) fn hook_permission_mode(turn_context: &TurnContext) -> String {
    match turn_context.collaboration_mode.mode {
        ModeKind::Plan => "plan",
        ModeKind::Default | ModeKind::PairProgramming | ModeKind::Execute => "default",
        ModeKind::Accept => "acceptEdits",
        ModeKind::Bypass => "bypassPermissions",
    }
    .to_string()
}
```

### 5.2 Update Stop hook inline mapping

**File:** `core/src/codex.rs:5760-5766`

Replace inline match with call to centralized function:
```rust
let stop_hook_permission_mode = crate::hook_runtime::hook_permission_mode(&turn_context);
```

### 5.3 No hook schema changes needed

`hooks/src/schema.rs:306-314` already enumerates all required values: `"default"`, `"acceptEdits"`, `"plan"`, `"dontAsk"`, `"bypassPermissions"`.

---

## Phase 6: `/permissions` Popup Rework

### 6.1 Rework into sandbox toggle

**Files:** `tui/src/chatwidget.rs`, `tui_app_server/src/chatwidget.rs`

The `/permissions` popup stops driving approval posture (that's the collaboration mode's job now). It becomes a sandbox scope selector:

| Option | Sandbox | Description |
|--------|---------|-------------|
| **Sandbox On** | Repo-scoped (WorkspaceWrite or ReadOnly) | Agent can only access files in the current repo |
| **Sandbox Off** | DangerFullAccess | Agent has full system access |

The existing Guardian Approvals flow, full-access confirmation warnings, Windows sandbox setup, and world-writable path warnings **remain** on this popup — they are sandbox concerns, not approval concerns.

### 6.2 Status card shows mode + sandbox independently

**Files:** `tui/src/status/card.rs`, `tui_app_server/src/status/card.rs`

Status card renders two independent lines:
- **Mode:** Default / Accept / Bypass / Plan (from `collaboration_mode_kind`)
- **Sandbox:** On (repo-scoped) / Off (full access) (from `sandbox_policy`)

### 6.3 Mode indicator exhaustive match

**Files:** `tui/src/chatwidget.rs`, `tui_app_server/src/chatwidget.rs`

`CollaborationModeIndicator` and `collaboration_mode_indicator()` use exhaustive matches. Add arms for Accept and Bypass.

---

## Phase 7: TUI Integration

### 7.1 Mode cycling

Existing `collaboration_modes::next_mask()` auto-includes new modes via `is_tui_visible()` filter + new presets. Cycle order: Default → Accept → Bypass → Plan → Default.

No code change needed in `collaboration_modes.rs` — it reads from `TUI_VISIBLE_COLLABORATION_MODES` and the preset list, both updated in Phase 1 and Phase 2.

### 7.2 Slash commands

**File:** `tui/src/slash_command.rs` — add `Accept`, `Bypass` variants
**File:** `tui/src/chatwidget.rs` — route via `mask_for_kind()` → `set_collaboration_mask()`

```rust
SlashCommand::Accept => {
    if let Some(mask) = collaboration_modes::mask_for_kind(models_manager, ModeKind::Accept) {
        self.set_collaboration_mask(mask);
    }
}
SlashCommand::Bypass => {
    if let Some(mask) = collaboration_modes::mask_for_kind(models_manager, ModeKind::Bypass) {
        self.set_collaboration_mask(mask);
    }
}
```

Uses the existing `set_collaboration_mask()` — no new state plumbing needed.

### 7.3 Mirror ALL TUI changes in `tui_app_server/`

| tui/ file | tui_app_server/ equivalent | Change |
|---|---|---|
| `src/slash_command.rs` | `src/slash_command.rs` | Add Accept, Bypass variants |
| `src/chatwidget.rs` | `src/chatwidget.rs` | Dispatch, mode indicator, /permissions rework |
| `src/status/card.rs` | `src/status/card.rs` | Mode + sandbox rendering |
| Snapshots | Snapshots | Accept new snapshots |

---

## Phase 8: App-Server Integration

### 8.1 `collaborationMode/list` — automatic

The existing `collaborationMode/list` handler returns presets from `thread_manager.list_collaboration_modes()`. Since we add Accept and Bypass presets in Phase 2.3, they automatically appear in the list. No handler changes needed.

### 8.2 `turn/start` collaboration_mode — automatic

The existing `collaboration_mode` field on `TurnStartParams` accepts a `CollaborationMode` with `mode: ModeKind`. Clients can already set `mode: "accept"` or `mode: "bypass"` once the ModeKind variants exist. No new field needed.

### 8.3 `TurnStartedEvent` → `TurnStartedNotification` mapping

**File:** `app-server/src/bespoke_event_handling.rs`

The `TurnStartedEvent.collaboration_mode_kind` field (which already carries `ModeKind`) must be mapped to `TurnStartedNotification`. Verify that the bespoke mapper passes through the new variants correctly.

### 8.4 Schema and SDK regeneration

```bash
just write-app-server-schema
just write-app-server-schema --experimental
cargo test -p orbit-code-app-server-protocol
cd sdk/python && python scripts/update_sdk_artifacts.py generate-types && pytest -q
```

### 8.5 Update `app-server/README.md`

Document Accept and Bypass as new collaboration mode values in the API reference.

---

## Phase 9: Testing

### 9.1 Unit tests — exec_policy

**File:** `core/src/exec_policy_tests.rs` (existing)

Test every cell of the Authoritative Behavior Matrix:
- Default mode: preserves all existing behavior
- Accept mode: read allows, write allows (non-dangerous shell), dangerous prompts
- Bypass mode: everything allows including dangerous
- Plan mode: forbidden (if reached)

### 9.2 Unit tests — sandboxing

**File:** `core/src/tools/sandboxing_tests.rs` (existing)

Test `default_exec_approval_requirement` with each ModeKind.

### 9.3 Unit tests — unix escalation

Test `render_decision_for_unmatched_command` direct call respects mode_kind.

### 9.4 Integration tests — hooks

**File:** `core/tests/suite/hooks.rs` (existing)

Assert SessionStart, UserPromptSubmit, and Stop hooks receive correct `permission_mode`:
- Default → `"default"`
- Accept → `"acceptEdits"`
- Bypass → `"bypassPermissions"`
- Plan → `"plan"`

### 9.5 Integration tests — approval flows

**File:** `core/tests/suite/permission_modes.rs` (new, add to `tests/suite/mod.rs`)

TestCodexBuilder end-to-end for each mode.

### 9.6 TUI tests

**Files:** `tui/src/chatwidget/tests.rs`, `tui_app_server/src/chatwidget/tests.rs`

Mode indicator, slash command dispatch, /permissions popup rework.

### 9.7 Snapshot tests

```bash
cargo test -p orbit-code-tui
cargo insta accept -p orbit-code-tui
cargo test -p orbit-code-tui-app-server
cargo insta accept -p orbit-code-tui-app-server
```

### 9.8 `TurnStartedEvent` constructor sites

The new ModeKind variants must be handled at all `TurnStartedEvent` construction sites:
- `core/src/tasks/regular.rs`
- `core/src/tasks/user_shell.rs`
- `core/src/compact.rs`
- `core/src/compact_remote.rs`
- `app-server-protocol/src/protocol/thread_history.rs` (tests)

These sites already pass `collaboration_mode_kind` from `TurnContext` — no code change needed, but they must be verified to handle new variants without panicking.

---

## Files Changed (Complete List)

| File | Change |
|------|--------|
| `protocol/src/config_types.rs` | Add Accept, Bypass to ModeKind; ToolCategory enum; helpers |
| `core/templates/collaboration_mode/accept.md` | New template |
| `core/templates/collaboration_mode/bypass.md` | New template |
| `core/BUILD.bazel` | Export accept.md and bypass.md |
| `core/src/models_manager/collaboration_mode_presets.rs` | Add presets with developer_instructions |
| `core/src/exec_policy.rs` | mode_kind in ExecApprovalRequest; rewrite render_decision; extract default_mode |
| `core/src/tools/sandboxing.rs` | mode_kind in default_exec_approval_requirement; tool_category() trait method |
| `core/src/tools/orchestrator.rs` | Mode override; thread mode_kind to both default_exec callsites |
| `core/src/tools/handlers/shell.rs` | mode_kind in ExecApprovalRequest construction |
| `core/src/tools/runtimes/shell/unix_escalation.rs` | Thread mode_kind to direct fallback |
| `core/src/tools/runtimes/shell.rs` | tool_category() → Shell |
| `core/src/tools/runtimes/unified_exec.rs` | tool_category() → Shell |
| `core/src/tools/runtimes/apply_patch.rs` | tool_category() → WriteMutation |
| `core/src/codex.rs` | ExecApprovalRequest mode_kind; approval prompt suppression for Accept/Bypass; stop hook centralization |
| `core/src/unified_exec/process_manager.rs` | ExecApprovalRequest mode_kind |
| `core/src/hook_runtime.rs` | Rewrite hook_permission_mode from ModeKind |
| `app-server/src/bespoke_event_handling.rs` | Verify TurnStartedNotification mapping handles new variants |
| `app-server/README.md` | Document Accept, Bypass as collaboration mode values |
| `tui/src/slash_command.rs` | Add Accept, Bypass |
| `tui/src/chatwidget.rs` | Dispatch, indicator, /permissions rework to sandbox toggle |
| `tui/src/status/card.rs` | Mode + sandbox independent rendering |
| `tui_app_server/src/model_catalog.rs` | Mirror presets |
| `tui_app_server/src/slash_command.rs` | Mirror slash commands |
| `tui_app_server/src/chatwidget.rs` | Mirror dispatch, indicator, /permissions rework |
| `tui_app_server/src/status/card.rs` | Mirror status card |
| `core/src/exec_policy_tests.rs` | Behavior matrix tests |
| `core/src/tools/sandboxing_tests.rs` | Mode override tests |
| `core/tests/suite/permission_modes.rs` | Integration tests (new) |
| `core/tests/suite/hooks.rs` | Hook permission_mode assertions |

---

## Edge Cases & Decisions

| Edge Case | Decision |
|---|---|
| No config.toml `mode` field | Modes are runtime session state. No config field in v1. |
| No `--mode` CLI flag | Defer to follow-up. Use `--dangerously-bypass-approvals-and-sandbox` for CLI bypass. |
| Mode persistence across resume | Modes do NOT persist. Reset to Default on resume. |
| Mode switch mid-turn | Takes effect on next turn (existing collaboration mode behavior). |
| `.rules` deny vs Bypass | `.rules` evaluated first — denials always override any mode. |
| Sandbox + mode independence | Sandbox controls filesystem scope. Mode controls approval posture. They compose freely. |
| `/permissions` popup | Becomes sandbox toggle. Guardian, warnings, Windows setup stay — they are sandbox concerns. |
| Status card | Shows mode and sandbox as two separate fields. |
| App-server `collaborationMode/list` | Automatically includes Accept/Bypass presets (no handler change). |
| App-server `turn/start` | Existing `collaboration_mode.mode` field accepts new variants (no new field). |
| `TurnStartedNotification` | Verify bespoke mapper in `bespoke_event_handling.rs` passes through new ModeKind variants. |
| Python SDK | Regenerate generated models after schema change. Run `pytest -q`. |
| `--full-auto` + `/accept` in-session | `--full-auto` sets approval/sandbox at startup. `/accept` overrides at mode layer. |
| `--dangerously-bypass-approvals-and-sandbox` + `/bypass` | Functionally equivalent but independent layers. |
| Bypass dangerous commands | Bypass auto-approves dangerous commands. This matches current "Full Access" behavior. Users choosing Bypass accept full risk. |
| Accept + MCP tool | Auto-approve (MCP tools are "other tools" — auto-approved in Accept). |
| Default semantics for apply_patch | Existing behavior preserved (safe patches auto-approve via existing logic). |
| Hook `permission_mode` after Plan selected | Reports `"plan"` — matches Claude Code behavior. |
| Plan mode behavior | Completely unchanged. Do not touch. |
| Op transport | No new transport needed. ModeKind is already carried on Op::UserTurn, Op::OverrideTurnContext, SessionSettingsUpdate, AppEvent, AppCommand. |
| `bypass_sandbox` in ExecApprovalRequirement | Always `false`. Mode controls approval, not sandbox scope. Sandbox scope is determined by sandbox_policy independently. |

---

## Verification

Per-phase:
```bash
just fmt
just fix -p orbit-code-protocol
just fix -p orbit-code-core
just fix -p orbit-code-tui
just fix -p orbit-code-tui-app-server
just fix -p orbit-code-app-server-protocol
just fix -p orbit-code-app-server
cargo test -p orbit-code-protocol
cargo test -p orbit-code-core
cargo test -p orbit-code-tui
cargo test -p orbit-code-tui-app-server
cargo test -p orbit-code-app-server-protocol
cargo test -p orbit-code-app-server
```

Targeted:
```bash
cargo test -p orbit-code-core -- exec_policy
cargo test -p orbit-code-core -- sandboxing
cargo test -p orbit-code-core -- unix_escalation
cargo test -p orbit-code-core -- suite::hooks
cargo test -p orbit-code-core -- suite::permission_modes
cargo test -p orbit-code-tui -- chatwidget
cargo test -p orbit-code-tui-app-server -- chatwidget
```

Schema + SDK:
```bash
just write-config-schema
just write-app-server-schema
just write-app-server-schema --experimental
cargo test -p orbit-code-app-server-protocol
cd sdk/python && python scripts/update_sdk_artifacts.py generate-types && pytest -q
```

Snapshots:
```bash
cargo insta pending-snapshots -p orbit-code-tui
cargo insta pending-snapshots -p orbit-code-tui-app-server
cargo insta accept -p orbit-code-tui
cargo insta accept -p orbit-code-tui-app-server
```

End-to-end:
```bash
just codex
# Shift+Tab → cycles Default → Accept → Bypass → Plan → Default
# /accept → write tools auto-approve, dangerous bash prompts
# /bypass → everything auto-approved, no prompts
# /permissions → sandbox toggle (repo-scoped vs full system)
# /default → back to standard behavior
```
