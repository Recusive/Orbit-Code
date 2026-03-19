# codex-rs/core/src/guardian/

Guardian review system for automated tool approval decisions.

## What this folder does

The Guardian is an automated review system that decides whether an `on-request` approval should be granted automatically instead of being shown to the user. It uses a dedicated AI review session to assess the risk of proposed tool actions.

High-level flow:
1. Reconstruct a compact transcript preserving user intent plus recent assistant/tool context
2. Ask a dedicated guardian review session to assess the exact planned action and return strict JSON
3. Fail closed on timeout, execution failure, or malformed output
4. Approve only low- and medium-risk actions (risk_score < 80)

Key design decisions:
- Guardian uses `gpt-5.4` as its preferred model
- 90-second timeout for review sessions
- Transcript is truncated to 10K tokens for messages and 10K for tools
- The guardian clones the parent config, inheriting managed network proxy/allowlist

## Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations, constants, `GuardianAssessment` and `GuardianEvidence` structs |
| `approval_request.rs` | `GuardianApprovalRequest` -- formats tool calls and MCP annotations for review |
| `prompt.rs` | Builds guardian prompt items, truncates transcripts, defines output schema |
| `review.rs` | `review_approval_request()` -- runs the guardian review and interprets results |
| `review_session.rs` | `GuardianReviewSessionManager` -- creates and manages review sessions |
| `tests.rs` | Integration tests for guardian review flow |
| `snapshots/` | Insta test snapshots for guardian request layouts |

## Imports from

- `crate::client` -- `ModelClient` for running the review session
- `crate::config` -- `Config` for cloning parent configuration
- `crate::codex` -- `Session`, `TurnContext` for accessing conversation state
- `codex_protocol` -- `GuardianRiskLevel`, `ReviewDecision`

## Exports to

- `crate::tools::orchestrator` -- `routes_approval_to_guardian()`, `review_approval_request()` used during tool approval flow
- `crate::tasks` -- guardian review results feed into approval decisions
