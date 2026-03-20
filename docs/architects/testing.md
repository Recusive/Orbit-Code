# agent-desktop: Desktop Automation for AI Agents

> **Status:** Design spec
> **Author:** Brainstorm session, 2026-03-20
> **Scope:** macOS first, Windows abstraction ready

## Problem

AI agents can control web apps (Playwright, agent-browser) and CLIs (tmux, shell). They cannot control desktop applications. When an agent builds a feature in a native app — VS Code, Electron, SwiftUI — it has no way to launch the app, interact with the UI, and verify the result. The agent must ask a human to test, breaking the build-verify loop.

## Solution

A standalone MCP server + CLI that lets AI agents control any desktop application through the OS accessibility API. Same philosophy as agent-browser (ref-based, compact text, agent-first), but targeting native apps instead of Chrome.

```
┌──────────────────────────────────────────────┐
│  AI Agent (Claude Code, Cursor, Codex, etc.) │
│                                              │
│  "click the Run button"                      │
│  "what's in the sidebar?"                    │
│  "capture frames while I click Save"         │
└──────────────┬───────────────────────────────┘
               │ MCP / CLI
               ▼
┌──────────────────────────────────────────────┐
│  agent-desktop                               │
│                                              │
│  ┌─────────────────────────────────────────┐ │
│  │  Platform Adapter (trait)               │ │
│  │                                         │ │
│  │  ┌─────────────┐  ┌──────────────────┐  │ │
│  │  │ macOS       │  │ Windows (future) │  │ │
│  │  │ Accessibility│  │ UI Automation    │  │ │
│  │  │ API         │  │ API              │  │ │
│  │  └─────────────┘  └──────────────────┘  │ │
│  └─────────────────────────────────────────┘ │
│                                              │
│  ┌────────────┐ ┌───────────┐ ┌───────────┐ │
│  │ Ref Engine │ │ Screenshot│ │ Frame     │ │
│  │            │ │ Capture   │ │ Capture   │ │
│  └────────────┘ └───────────┘ └───────────┘ │
└──────────────────────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────┐
│  Target Application (any native app)         │
│  Finder, VS Code, Orbit, Terminal, etc.      │
└──────────────────────────────────────────────┘
```

## Design Decisions

### Why accessibility API (not screen scraping)

| Approach | Pros | Cons |
|----------|------|------|
| Accessibility API | Structured tree, reliable element IDs, fast, works with hidden elements | Requires app to implement a11y (most do), platform-specific |
| Screen scraping (OCR) | Works with anything visible | Slow, brittle, can't find hidden elements, no semantic info |
| App-specific protocol | Deepest integration | Only works with one app, not general purpose |

Accessibility API is the right choice. Every major framework (AppKit, SwiftUI, Electron, Qt, GTK) exposes accessibility data. It gives us the same structured tree that agent-browser gets from the DOM, but for native apps.

### Why ref-based (not coordinate-based)

Refs like `@b1`, `@e2` point to accessibility elements by identity, not screen position. This means:
- Window can be any size or position
- Works even if layout shifts between commands
- Deterministic — same ref always means same element
- Agent doesn't need to reason about pixel coordinates

### Why timed PNG frames (not GIF/video)

The AI agent processes images as static visuals — it cannot "play" animations. Individual PNGs with timing metadata give the agent:
- Full resolution per frame
- Exact timestamps for mathematical reasoning about timing
- Ability to compare specific frames side-by-side
- No encoding artifacts or dependencies (no ffmpeg)

## Platform Abstraction

```rust
/// Core trait that each platform implements.
trait DesktopPlatform: Send + Sync {
    /// List all running applications.
    fn list_apps(&self) -> Result<Vec<AppInfo>>;

    /// Launch an application by name or bundle ID.
    fn launch_app(&self, app: &str) -> Result<AppHandle>;

    /// Focus an application window.
    fn focus_window(&self, handle: &AppHandle) -> Result<()>;

    /// Get the accessibility tree for the focused window.
    /// Returns a tree of elements, each with a unique ref.
    fn snapshot(&self) -> Result<AccessibilityTree>;

    /// Perform a click on the element identified by ref.
    fn click(&self, element_ref: &str) -> Result<()>;

    /// Type text into the focused element or a specific ref.
    fn type_text(&self, element_ref: Option<&str>, text: &str) -> Result<()>;

    /// Press a key combination (e.g., "cmd+s", "enter").
    fn press_key(&self, key: &str) -> Result<()>;

    /// Capture a screenshot of the focused window.
    fn screenshot(&self, path: &Path) -> Result<()>;

    /// Capture N frames over a duration around an action.
    fn capture_frames(
        &self,
        action: Box<dyn FnOnce() -> Result<()>>,
        duration_ms: u64,
        interval_ms: u64,
    ) -> Result<FrameSequence>;

    /// Get the value/text content of an element.
    fn get_value(&self, element_ref: &str) -> Result<String>;

    /// Wait for an element matching a condition to appear.
    fn wait_for(
        &self,
        condition: WaitCondition,
        timeout_ms: u64,
    ) -> Result<()>;
}
```

### macOS Implementation

Uses Apple's Accessibility API (`AXUIElement`):

- `AXUIElementCreateApplication(pid)` — connect to app
- `AXUIElementCopyAttributeValue` — read element properties (role, title, value, children)
- `AXUIElementPerformAction` — click, press, etc.
- `AXUIElementSetAttributeValue` — set text values
- `CGWindowListCreateImage` — screenshots
- `CGDisplayStream` or timed `CGWindowListCreateImage` — frame capture

Requires: macOS Accessibility permission (System Settings → Privacy → Accessibility). The MCP server prompts on first use.

### Windows Implementation (future)

Uses Windows UI Automation:
- `IUIAutomation` — connect to app
- `IUIAutomationElement` — traverse element tree
- `Invoke`, `SetValue`, `Select` patterns — interact
- `PrintWindow` or `BitBlt` — screenshots

Same trait, different backend. The ref engine and frame capture logic are shared.

## Accessibility Tree & Ref Engine

The snapshot command returns a compact text representation of the UI tree:

```
- window "Orbit" [ref=w1]
  - toolbar [ref=t1]
    - button "Run" [ref=b1]
    - button "Debug" [ref=b2]
    - button "Stop" [ref=b3] (disabled)
  - splitview [ref=s1]
    - sidebar [ref=s2]
      - treeitem "src" [ref=i1] (expanded)
        - treeitem "main.rs" [ref=i2]
        - treeitem "lib.rs" [ref=i3]
    - editor [ref=e1]
      - textarea [ref=e2] value="fn main() {\n    println!(\"hello\");\n}"
  - panel "Terminal" [ref=p1]
    - text "$ cargo run" [ref=x1]
    - text "hello" [ref=x2]
```

**Ref lifecycle:**
- Refs are generated per snapshot (monotonically incrementing: `e1`, `e2`, ...)
- Refs are stable within a snapshot — calling `click @b1` uses the ref from the last snapshot
- A new `snapshot` command invalidates old refs and generates fresh ones
- If the UI changes between snapshot and action, the ref engine validates the element still exists

**Filtering:**
```bash
agent-desktop snapshot                  # full tree
agent-desktop snapshot --depth 2        # only 2 levels deep
agent-desktop snapshot --role button    # only buttons
agent-desktop snapshot --text "Save"    # elements containing "Save"
```

## Frame Capture System

For verifying animations, transitions, and temporal UI behavior:

```bash
# Capture frames around an action
agent-desktop frames click @b1 --duration 500 --interval 50
```

**Output:**
```json
{
  "action": "click @b1",
  "target": "button \"Run\" [ref=b1]",
  "frames": [
    { "path": "/tmp/agent-desktop/frames/001.png", "ms": 0, "label": "before" },
    { "path": "/tmp/agent-desktop/frames/002.png", "ms": 50 },
    { "path": "/tmp/agent-desktop/frames/003.png", "ms": 100 },
    { "path": "/tmp/agent-desktop/frames/004.png", "ms": 150 },
    { "path": "/tmp/agent-desktop/frames/005.png", "ms": 200 },
    { "path": "/tmp/agent-desktop/frames/006.png", "ms": 250 },
    { "path": "/tmp/agent-desktop/frames/007.png", "ms": 300 },
    { "path": "/tmp/agent-desktop/frames/008.png", "ms": 350 },
    { "path": "/tmp/agent-desktop/frames/009.png", "ms": 400 },
    { "path": "/tmp/agent-desktop/frames/010.png", "ms": 500, "label": "after" }
  ],
  "duration_ms": 500,
  "frame_count": 10
}
```

**How the agent uses this:**
1. Receives the frame list with paths and timestamps
2. Views each PNG as a separate image
3. Compares frame-to-frame: "sidebar width goes 0% → 10% → 30% → 90% → 100%"
4. Checks timing: "the jump from 30% to 90% happened in one 50ms interval — that's a stutter"
5. Reports: "animation has a dropped frame between 150ms-200ms, should be smooth ease-out"

**What this catches:**
- Missing animations (state teleports in one frame)
- Flicker (element disappears in a middle frame, reappears later)
- Stutter (uneven progress between frames)
- Wrong timing (all motion in first 100ms of a 300ms animation)
- Layout glitches (overlapping elements mid-transition)
- Loading state issues (spinner never appears, or appears too briefly)

## CLI Commands

```bash
# Application management
agent-desktop list                          # list running apps
agent-desktop open "Orbit"                  # launch app by name
agent-desktop open --bundle com.recursive.orbit  # launch by bundle ID
agent-desktop focus "Orbit"                 # bring window to front
agent-desktop close                         # close focused window

# Accessibility tree
agent-desktop snapshot                      # full accessibility tree
agent-desktop snapshot --depth 3            # limit tree depth
agent-desktop snapshot --role button        # filter by role
agent-desktop snapshot --text "Save"        # filter by text content
agent-desktop snapshot --ref @s1            # subtree from a ref

# Interaction
agent-desktop click @b1                     # click element
agent-desktop double-click @b1              # double-click
agent-desktop right-click @b1               # right-click / context menu
agent-desktop type @e1 "hello world"        # type into element
agent-desktop clear @e1                     # clear element value
agent-desktop press "cmd+s"                 # key combination
agent-desktop press "enter"                 # single key
agent-desktop scroll @s1 down 3             # scroll element
agent-desktop drag @i1 @s2                  # drag and drop
agent-desktop hover @b1                     # hover over element
agent-desktop select @d1 "Option A"         # select dropdown value

# Reading state
agent-desktop value @e1                     # get element's text/value
agent-desktop attr @b1 enabled              # get element attribute
agent-desktop bounds @b1                    # get element position/size
agent-desktop focused                       # which element has focus

# Visual capture
agent-desktop screenshot                    # screenshot focused window
agent-desktop screenshot --path /tmp/s.png  # save to specific path
agent-desktop screenshot --ref @p1          # screenshot specific element
agent-desktop frames click @b1 --duration 500 --interval 50

# Waiting
agent-desktop wait --text "Build succeeded" --timeout 30000
agent-desktop wait --ref @b1 --attr enabled --timeout 5000
agent-desktop wait --gone @spinner1 --timeout 10000

# Session management
agent-desktop status                        # show daemon status
agent-desktop pid "Orbit"                   # get PID of app
```

## MCP Server Interface

The same functionality exposed as MCP tools for AI agents:

```json
{
  "tools": [
    {
      "name": "desktop_open",
      "description": "Launch a desktop application",
      "input_schema": {
        "type": "object",
        "properties": {
          "app": { "type": "string", "description": "App name or bundle ID" }
        },
        "required": ["app"]
      }
    },
    {
      "name": "desktop_snapshot",
      "description": "Get accessibility tree of focused window. Returns compact text with refs for each element.",
      "input_schema": {
        "type": "object",
        "properties": {
          "depth": { "type": "integer", "description": "Max tree depth" },
          "role": { "type": "string", "description": "Filter by element role" },
          "text": { "type": "string", "description": "Filter by text content" },
          "ref": { "type": "string", "description": "Subtree from this ref" }
        }
      }
    },
    {
      "name": "desktop_click",
      "description": "Click an element by ref from last snapshot",
      "input_schema": {
        "type": "object",
        "properties": {
          "ref": { "type": "string", "description": "Element ref like @b1" }
        },
        "required": ["ref"]
      }
    },
    {
      "name": "desktop_type",
      "description": "Type text into an element",
      "input_schema": {
        "type": "object",
        "properties": {
          "ref": { "type": "string", "description": "Element ref" },
          "text": { "type": "string", "description": "Text to type" }
        },
        "required": ["text"]
      }
    },
    {
      "name": "desktop_screenshot",
      "description": "Capture screenshot of focused window or specific element",
      "input_schema": {
        "type": "object",
        "properties": {
          "ref": { "type": "string", "description": "Element ref (optional, defaults to window)" }
        }
      }
    },
    {
      "name": "desktop_frames",
      "description": "Capture timed PNG frame sequence around an action for animation/transition verification",
      "input_schema": {
        "type": "object",
        "properties": {
          "action": { "type": "string", "description": "Action to perform: 'click @b1', 'press enter', etc." },
          "duration_ms": { "type": "integer", "description": "Total capture duration", "default": 500 },
          "interval_ms": { "type": "integer", "description": "Time between frames", "default": 50 }
        },
        "required": ["action"]
      }
    },
    {
      "name": "desktop_wait",
      "description": "Wait for a condition (element appears, text matches, element disappears)",
      "input_schema": {
        "type": "object",
        "properties": {
          "text": { "type": "string", "description": "Wait for text to appear" },
          "ref": { "type": "string", "description": "Wait for element state change" },
          "gone": { "type": "string", "description": "Wait for element to disappear" },
          "timeout_ms": { "type": "integer", "default": 10000 }
        }
      }
    },
    {
      "name": "desktop_press",
      "description": "Press a key or key combination",
      "input_schema": {
        "type": "object",
        "properties": {
          "key": { "type": "string", "description": "Key combo like 'cmd+s', 'enter', 'tab'" }
        },
        "required": ["key"]
      }
    },
    {
      "name": "desktop_value",
      "description": "Read the text/value of an element",
      "input_schema": {
        "type": "object",
        "properties": {
          "ref": { "type": "string", "description": "Element ref" }
        },
        "required": ["ref"]
      }
    }
  ]
}
```

## Architecture: Client-Daemon

Same pattern as agent-browser:

```
agent-desktop CLI ──(IPC)──▶ agent-desktop daemon ──(Accessibility API)──▶ App
     │                              │
     │ Fast command parsing         │ Persistent connection to apps
     │ Thin client                  │ Manages ref state
     │                              │ Frame capture scheduler
     │                              │ Screenshot buffer
```

**Why a daemon:**
- Accessibility API connections are stateful — creating/destroying per command is slow
- Ref engine needs to persist between commands (snapshot → click uses same refs)
- Frame capture needs a background thread already running to grab frames at precise intervals
- Multiple agent commands can share one daemon instance

**Daemon lifecycle:**
- Auto-starts on first CLI command
- Persists between commands (like agent-browser)
- Auto-shuts down after 5 minutes of inactivity
- Health check via `agent-desktop status`

## Implementation: Rust

Native Rust for performance and cross-platform compilation:

```
agent-desktop/
├── Cargo.toml
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── daemon.rs               # Daemon process + IPC server
│   ├── ipc.rs                  # Client-daemon communication
│   ├── ref_engine.rs           # Ref generation and lookup
│   ├── frame_capture.rs        # Timed PNG frame sequences
│   ├── screenshot.rs           # Single screenshot capture
│   ├── mcp_server.rs           # MCP tool interface
│   ├── platform/
│   │   ├── mod.rs              # DesktopPlatform trait
│   │   ├── macos.rs            # macOS Accessibility API
│   │   └── windows.rs          # Windows UI Automation (future)
│   └── output/
│       ├── text.rs             # Compact text format (default)
│       └── json.rs             # JSON format (--json flag)
├── tests/
│   ├── snapshot_tests.rs
│   ├── interaction_tests.rs
│   └── frame_capture_tests.rs
└── README.md
```

**Dependencies (macOS):**
- `core-foundation` — macOS framework bindings
- `core-graphics` — screenshot capture
- `accessibility` (or raw `AXUIElement` FFI) — accessibility tree
- `png` — frame encoding
- `tokio` — async daemon runtime
- `serde` / `serde_json` — serialization

## Example: Agent Self-Testing Orbit Code TUI

The motivating use case — an AI agent builds a feature and tests it:

```
Agent: I just added a new /model command. Let me test it.

> desktop_open "Terminal"
> desktop_type "cargo run --bin orbit-code\n"
> desktop_wait --text "Orbit Code" --timeout 30000

# Verify the TUI started
> desktop_screenshot
[Agent sees: TUI welcome screen with model name in header]

# Open model picker
> desktop_snapshot --depth 1
- window "Terminal" [ref=w1]
  - ... terminal content ...

# Actually, for TUI apps in terminal, use tmux integration
# For native GUI apps (Orbit IDE), use accessibility directly:

> desktop_open "Orbit"
> desktop_wait --text "Orbit" --timeout 10000
> desktop_snapshot --depth 3

- window "Orbit" [ref=w1]
  - sidebar [ref=s1]
    - treeitem "src" [ref=i1]
  - editor [ref=e1]
  - panel "Chat" [ref=p1]
    - textfield "Ask anything..." [ref=t1]

# Type a message in the chat panel
> desktop_click @t1
> desktop_type "Hello, can you help me?"
> desktop_press "enter"

# Wait for response
> desktop_wait --text "Sure" --timeout 15000
> desktop_snapshot --ref @p1

# Verify the response appeared in the chat panel
- panel "Chat" [ref=p1]
  - text "Hello, can you help me?" [ref=m1]
  - text "Sure, I'd be happy to help..." [ref=m2]

# Test animation: click a collapsible section
> desktop_frames click @i1 --duration 400 --interval 40
[Agent receives 10 PNGs, verifies the tree collapse animation is smooth]
```

## Security

- **macOS Accessibility permission required** — the daemon needs to be in System Settings → Privacy → Accessibility. This is an OS-level gate that the user must explicitly grant.
- **No network access** — the daemon only talks to local apps via accessibility API and the agent via local IPC.
- **Ref scoping** — refs are scoped to the focused window. An agent can't accidentally interact with a different app's UI unless it explicitly focuses it.
- **Read-before-write** — the daemon always validates an element still exists before performing an action on it.

## Success Criteria

The system is done when an AI agent can:

1. Launch any macOS app by name
2. Read the full accessibility tree as compact text with refs
3. Click, type, press keys on any element by ref
4. Take screenshots of windows or specific elements
5. Capture timed frame sequences to verify animations
6. Wait for conditions (text appears, element gone, attribute changes)
7. Do all of the above through both CLI and MCP server
8. Self-test a feature it just built in a native app end-to-end without human intervention
