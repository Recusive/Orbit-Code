# .codex/skills/babysit-pr/scripts/ — Watcher Script

## What This Folder Does

Contains the executable Python script that powers the PR babysitter's polling loop. This is the primary tool the Codex agent invokes during PR monitoring.

## Key Files

| File | Role |
|------|------|
| `gh_pr_watch.py` | Python 3 script (~800 lines) that snapshots PR state (CI checks, workflow runs, review comments, mergeability) and emits structured JSON with recommended actions. Supports one-shot, continuous watch, and retry-failed modes. |

## What It Plugs Into

- **Codex agent**: Called directly via `python3 .codex/skills/babysit-pr/scripts/gh_pr_watch.py` with various flags.
- **GitHub CLI (`gh`)**: All data fetching and actions (PR view, checks, run reruns, API calls for comments/reviews) go through the `gh` command.
- **State file**: Persists to `/tmp/codex-babysit-pr-<repo>-pr<number>.json` for deduplication and retry tracking across invocations.

## `gh_pr_watch.py` — Architecture

### Entry Points (CLI Modes)

| Flag | Behavior |
|------|----------|
| `--once` (default) | Emit a single JSON snapshot and exit. |
| `--watch` | Emit JSONL snapshots continuously, polling at `--poll-seconds` intervals. Stops on terminal actions (`stop_pr_closed`, `stop_ready_to_merge`, `stop_exhausted_retries`). |
| `--retry-failed-now` | Collect a snapshot, then rerun all failed workflow runs if the retry budget allows. Emits a JSON result with rerun details. |

### Key Arguments

| Argument | Default | Purpose |
|----------|---------|---------|
| `--pr` | `auto` | PR number, URL, or `auto` (infer from current branch). |
| `--repo` | (inferred) | `OWNER/REPO` override. |
| `--poll-seconds` | `30` | Polling interval for `--watch` mode. |
| `--max-flaky-retries` | `3` | Max rerun cycles per head SHA before recommending stop. |
| `--state-file` | (auto-generated) | Path to persistent state JSON. |

### Core Functions

| Function | Purpose |
|----------|---------|
| `resolve_pr()` | Resolves PR spec (auto/number/URL) to full PR metadata via `gh pr view`. |
| `get_pr_checks()` | Fetches CI check results via `gh pr checks`. |
| `summarize_checks()` | Counts pending/failed/passed checks and determines if all are terminal. |
| `get_workflow_runs_for_sha()` | Fetches workflow runs for a specific SHA via GitHub Actions API. |
| `failed_runs_from_workflow_runs()` | Filters to failed runs with rerunnable IDs. |
| `fetch_new_review_items()` | Fetches and deduplicates PR comments, inline review comments, and review submissions. Filters to trusted authors and known review bots. |
| `recommend_actions()` | Produces the `actions` list based on current state (idle, diagnose_ci_failure, retry_failed_checks, process_review_comment, stop_*). |
| `collect_snapshot()` | Orchestrates all data fetching and produces a complete snapshot dict. |
| `retry_failed_now()` | Executes `gh run rerun <id> --failed` for each failed run and updates retry count in state. |
| `run_watch()` | Continuous polling loop with adaptive backoff (exponential when CI is green and stable, reset on any change). |

### Output JSON Schema (Snapshot)

```json
{
  "pr": { "number": int, "url": str, "repo": str, "head_sha": str, "state": str, "merged": bool, "closed": bool, "mergeable": str, "merge_state_status": str, "review_decision": str },
  "checks": { "pending_count": int, "failed_count": int, "passed_count": int, "all_terminal": bool },
  "failed_runs": [{ "run_id": int, "workflow_name": str, "status": str, "conclusion": str, "html_url": str }],
  "new_review_items": [{ "kind": str, "id": str, "author": str, "body": str, "path": str|null, "line": int|null, "url": str }],
  "actions": ["idle" | "diagnose_ci_failure" | "retry_failed_checks" | "process_review_comment" | "stop_pr_closed" | "stop_ready_to_merge" | "stop_exhausted_retries"],
  "retry_state": { "current_sha_retries_used": int, "max_flaky_retries": int }
}
```

### Review Comment Filtering

Only surfaces comments from:
- Trusted humans: repo OWNER, MEMBER, COLLABORATOR, or the authenticated `gh` user.
- Approved review bots: logins containing "codex" that end with `[bot]`.

All other bot noise is filtered out.

## Dependencies

- Python 3 standard library only (argparse, json, subprocess, etc.).
- Requires `gh` CLI installed and authenticated.
- No pip packages needed.
