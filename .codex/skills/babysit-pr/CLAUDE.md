# .codex/skills/babysit-pr/ — PR Babysitter Skill

## What This Folder Does

Implements an autonomous PR monitoring skill that watches a GitHub pull request's CI pipeline, review comments, and mergeability status. It continuously polls until the PR reaches a terminal state (merged, closed, ready-to-merge, or blocked requiring human intervention).

## Key Files

| File | Role |
|------|------|
| `SKILL.md` | Skill definition: YAML front matter (`name: babysit-pr`) plus full agent instructions covering the monitoring loop, CI failure classification, review comment handling, git safety rules, polling cadence, and stop conditions. |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `agents/` | Agent configuration (OpenAI agent YAML for the PR Babysitter interface). |
| `references/` | Supporting documentation: CI failure heuristics and GitHub CLI/API usage notes. |
| `scripts/` | The Python watcher script (`gh_pr_watch.py`) that does the actual PR state polling and action recommendation. |

## What It Plugs Into

- **Codex agent**: Activated when a user says "monitor", "watch", or "babysit" a PR. The agent follows the instructions in `SKILL.md`.
- **GitHub CLI (`gh`)**: All PR metadata, CI checks, workflow runs, review comments, and rerun commands are executed via `gh` CLI commands.
- **Git**: The agent makes local code fixes, commits, and pushes to the PR branch when CI failures are branch-related or review feedback is actionable.

## Core Workflow

1. The agent runs `scripts/gh_pr_watch.py` in either `--once` (snapshot) or `--watch` (continuous JSONL) mode.
2. The script emits JSON snapshots containing PR state, CI check summary, failed runs, new review items, and recommended `actions`.
3. The agent inspects `actions` and takes appropriate steps:
   - `diagnose_ci_failure`: Inspect logs, classify as branch-related (fix + push) or flaky (rerun).
   - `retry_failed_checks`: Rerun failed jobs (up to 3 retries per SHA).
   - `process_review_comment`: Address actionable review feedback with code changes.
   - `stop_pr_closed` / `stop_ready_to_merge` / `stop_exhausted_retries`: Terminal conditions.
   - `idle`: No action needed, continue polling.
4. After any push or rerun, the agent resumes polling immediately.

## Polling Cadence

- CI not green: every 30 seconds (configurable via `--poll-seconds`).
- CI green, no changes: exponential backoff (30s, 1m, 2m, 4m, ..., up to 1 hour).
- Any state change resets to 30-second polling.

## State Management

The watcher persists state to a JSON file (`/tmp/codex-babysit-pr-<repo>-pr<number>.json`) tracking:
- Retry counts per SHA
- Seen comment/review IDs (for deduplication)
- Last snapshot timestamp

## Imports / Dependencies

- Python 3 standard library only (no pip dependencies).
- Requires `gh` CLI authenticated and available on PATH.
- References heuristics from `references/heuristics.md` for CI failure classification.
- References GitHub API details from `references/github-api-notes.md`.
