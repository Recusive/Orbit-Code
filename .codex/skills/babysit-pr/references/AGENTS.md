# .codex/skills/babysit-pr/references/

This file applies to `.codex/skills/babysit-pr/references/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- These directories define Codex skills and their support files. Keep the runnable instructions, helper scripts, and references aligned so the skill still works end-to-end when invoked.
- Prefer editing the smallest scope that owns the behavior: skill-level docs in the skill root, reusable snippets in `references/`, worker prompts in `agents/`, and executable helpers in `scripts/`.
- Reference material should stay concise and task-focused. Update it when the workflow changes so the skill does not drift away from the codebase it describes.

## Validate
- Validate by reading the skill from the top-level `SKILL.md` or directory doc and checking that referenced relative paths still exist.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### What This Folder Does

Contains reference documentation that the Codex agent consults during PR babysitting to make classification and triage decisions. These are not executable — they are knowledge artifacts the agent reads when it needs to decide how to handle a CI failure or review comment.

### Key Files

| File | Role |
|------|------|
| `heuristics.md` | CI failure classification checklist and decision tree. Defines when to treat a failure as branch-related (fix locally) vs. flaky/unrelated (rerun). Also covers review comment agreement criteria and stop-and-ask conditions. |
| `github-api-notes.md` | Documents the specific `gh` CLI commands and GitHub API endpoints used by the watcher script, including the JSON fields consumed from each response. |

### What They Plug Into

- **SKILL.md**: References these files explicitly (e.g., "Read `.codex/skills/babysit-pr/references/heuristics.md` for a concise checklist").
- **Agent runtime**: The Codex agent reads these files during execution to inform its CI failure diagnosis and review comment handling logic.
- **`scripts/gh_pr_watch.py`**: The watcher script implements the API calls documented in `github-api-notes.md`.

### `heuristics.md` Summary

Defines two classification buckets for CI failures:
- **Branch-related**: compile/typecheck/lint failures in touched files, deterministic test failures in changed areas, snapshot changes, static analysis violations, build config failures.
- **Flaky/unrelated**: DNS/network timeouts, runner provisioning failures, GitHub Actions infra outages, rate limits, non-deterministic failures in unrelated tests.

Decision tree: merged/closed -> stop; failed checks -> diagnose -> fix or rerun; flaky retries exhausted (default 3) -> stop; process review comments independently.

Review comment criteria: address if technically correct, actionable, consistent with user intent, and safe. Stop and ask if ambiguous, conflicting, or requires product decisions.

### `github-api-notes.md` Summary

Documents these `gh` commands and their consumed fields:
- `gh pr view --json ...` — PR metadata (number, URL, state, SHA, branch, mergeable, reviewDecision)
- `gh pr checks --json ...` — CI check buckets (pass/fail/pending/skipping)
- `gh api repos/.../actions/runs` — Workflow run discovery for a head SHA
- `gh run view <id> --json ...` and `--log-failed` — Failed run log inspection
- `gh run rerun <id> --failed` — Retry only failed jobs
- `gh api repos/.../issues/<n>/comments` — PR issue comments
- `gh api repos/.../pulls/<n>/comments` — Inline review comments
- `gh api repos/.../pulls/<n>/reviews` — Review submissions
