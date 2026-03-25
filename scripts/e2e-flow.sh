#!/bin/bash
#
# e2e-flow.sh — Run a multi-step TUI flow in tmux, capturing screen at each step.
#
# Usage:
#   ./scripts/e2e-flow.sh <flow-script>
#   ./scripts/e2e-flow.sh scripts/flows/check-auth.flow
#   ./scripts/e2e-flow.sh scripts/flows/send-prompt.flow
#
# Flow script format (one instruction per line):
#   WAIT <seconds>          — Wait for UI to settle
#   TYPE <text>             — Type text into the TUI
#   ENTER                   — Press Enter
#   ESCAPE                  — Press Escape
#   KEY <key>               — Send a tmux key (e.g. C-c, Up, Down, Tab)
#   CAPTURE <label>         — Capture screen and save as <label>.txt
#   EXPECT <text>           — Capture screen and check if <text> appears (PASS/FAIL)
#   SELECT <n>              — Press Down n times then Enter (pick item from list)
#   ---                     — Visual separator (ignored)
#
# All captures saved to /tmp/e2e-flow/<label>.txt
# Final summary printed at end: which steps passed/failed.
#

set -euo pipefail

FLOW_FILE="${1:?Usage: e2e-flow.sh <flow-script>}"
SESSION="e2e-flow-$$"
BINARY="./codex-rs/target/debug/orbit-code"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUTPUT_DIR="/tmp/e2e-flow"
PASS_COUNT=0
FAIL_COUNT=0
STEP=0

cd "$REPO_ROOT"

# Build if needed
if [ ! -f "$BINARY" ]; then
    echo "Building orbit-code..." >&2
    cd codex-rs && cargo build --bin orbit-code 2>&1 | tail -3 >&2
    cd ..
fi

# Clean output dir
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

# Kill any leftover session
tmux kill-session -t "$SESSION" 2>/dev/null || true

# Launch orbit-code in tmux
echo "Launching orbit-code..." >&2
tmux new-session -d -s "$SESSION" -x 120 -y 50 "$BINARY"

# Wait for startup
echo "Waiting for startup..." >&2
sleep 5

# Capture helper
capture() {
    local label="$1"
    tmux capture-pane -t "$SESSION" -p > "$OUTPUT_DIR/${label}.txt"
    cat "$OUTPUT_DIR/${label}.txt"
}

# Process flow file
while IFS= read -r line || [ -n "$line" ]; do
    # Skip empty lines and comments
    [[ -z "$line" || "$line" == \#* || "$line" == "---" ]] && continue

    STEP=$((STEP + 1))
    CMD=$(echo "$line" | awk '{print $1}')
    ARG=$(echo "$line" | cut -d' ' -f2-)

    case "$CMD" in
        WAIT)
            echo "[$STEP] Waiting ${ARG}s..." >&2
            sleep "$ARG"
            ;;
        TYPE)
            echo "[$STEP] Typing: $ARG" >&2
            tmux send-keys -t "$SESSION" "$ARG"
            sleep 0.5
            ;;
        ENTER)
            echo "[$STEP] Pressing Enter" >&2
            tmux send-keys -t "$SESSION" Enter
            sleep 1
            ;;
        ESCAPE)
            echo "[$STEP] Pressing Escape" >&2
            tmux send-keys -t "$SESSION" Escape
            sleep 0.5
            ;;
        KEY)
            echo "[$STEP] Sending key: $ARG" >&2
            tmux send-keys -t "$SESSION" "$ARG"
            sleep 0.5
            ;;
        SELECT)
            echo "[$STEP] Selecting item $ARG" >&2
            for ((i=1; i<ARG; i++)); do
                tmux send-keys -t "$SESSION" Down
                sleep 0.2
            done
            tmux send-keys -t "$SESSION" Enter
            sleep 1
            ;;
        CAPTURE)
            echo "[$STEP] Capturing: $ARG" >&2
            echo "=== CAPTURE: $ARG ==="
            capture "$ARG"
            echo ""
            ;;
        EXPECT)
            echo -n "[$STEP] Expecting: $ARG ... " >&2
            SCREEN=$(tmux capture-pane -t "$SESSION" -p)
            echo "$SCREEN" > "$OUTPUT_DIR/expect-step-${STEP}.txt"
            if echo "$SCREEN" | grep -q "$ARG"; then
                echo "PASS" >&2
                PASS_COUNT=$((PASS_COUNT + 1))
            else
                echo "FAIL" >&2
                FAIL_COUNT=$((FAIL_COUNT + 1))
                echo "=== EXPECTED: $ARG ==="
                echo "=== ACTUAL SCREEN ==="
                echo "$SCREEN"
                echo ""
            fi
            ;;
        *)
            echo "[$STEP] Unknown command: $CMD" >&2
            ;;
    esac
done < "$FLOW_FILE"

# Cleanup
echo "" >&2
echo "Cleaning up..." >&2
tmux send-keys -t "$SESSION" C-c 2>/dev/null || true
sleep 1
tmux kill-session -t "$SESSION" 2>/dev/null || true

# Summary
echo "" >&2
echo "================================" >&2
echo "  PASS: $PASS_COUNT  |  FAIL: $FAIL_COUNT" >&2
echo "  Captures saved to: $OUTPUT_DIR/" >&2
echo "================================" >&2

[ "$FAIL_COUNT" -eq 0 ] && exit 0 || exit 1
