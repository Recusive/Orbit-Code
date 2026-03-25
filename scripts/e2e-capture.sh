#!/bin/bash
#
# e2e-capture.sh — Launch orbit-code in tmux, send keystrokes, capture screen output.
#
# Usage:
#   ./scripts/e2e-capture.sh "command" [wait_seconds] [output_file]
#   ./scripts/e2e-capture.sh "/model" 5
#   ./scripts/e2e-capture.sh "/auth" 3 /tmp/auth-screen.txt
#   ./scripts/e2e-capture.sh ""  # just capture the startup screen
#
# Examples:
#   ./scripts/e2e-capture.sh "/model" 8          # Open model picker, wait 8s for API
#   ./scripts/e2e-capture.sh "/auth" 3            # Open auth status
#   ./scripts/e2e-capture.sh "/skills" 3          # Check skills list
#   ./scripts/e2e-capture.sh "" 5                 # Just capture startup screen
#   ./scripts/e2e-capture.sh "hello world" 10     # Type a message and wait for response
#
# The script:
#   1. Builds the binary (if needed)
#   2. Launches orbit-code in a tmux session
#   3. Waits for startup
#   4. Sends the command + Enter
#   5. Waits for the UI to settle
#   6. Captures the full screen to a file
#   7. Prints it to stdout
#   8. Kills the session
#

set -euo pipefail

COMMAND="${1:-}"
WAIT="${2:-5}"
OUTPUT="${3:-/tmp/orbit-e2e-capture.txt}"
SESSION="orbit-e2e-$$"
BINARY="./codex-rs/target/debug/orbit-code"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

cd "$REPO_ROOT"

# Build if binary doesn't exist
if [ ! -f "$BINARY" ]; then
    echo "Building orbit-code..." >&2
    cd codex-rs && cargo build --bin orbit-code 2>&1 | tail -3 >&2
    cd ..
fi

# Kill any leftover session
tmux kill-session -t "$SESSION" 2>/dev/null || true

# Launch in tmux
tmux new-session -d -s "$SESSION" -x 120 -y 50 "$BINARY"

# Wait for startup
echo "Waiting for startup..." >&2
sleep 5

# Send command if provided
if [ -n "$COMMAND" ]; then
    # Check if it's a slash command (needs Enter after)
    tmux send-keys -t "$SESSION" "$COMMAND"
    sleep 1
    tmux send-keys -t "$SESSION" Enter
    echo "Sent: $COMMAND" >&2
fi

# Wait for UI to settle
echo "Waiting ${WAIT}s for UI..." >&2
sleep "$WAIT"

# Capture screen
tmux capture-pane -t "$SESSION" -p > "$OUTPUT"

# Print to stdout
cat "$OUTPUT"

# Cleanup
tmux send-keys -t "$SESSION" C-c 2>/dev/null || true
sleep 1
tmux kill-session -t "$SESSION" 2>/dev/null || true

echo "" >&2
echo "Screen captured to: $OUTPUT" >&2
