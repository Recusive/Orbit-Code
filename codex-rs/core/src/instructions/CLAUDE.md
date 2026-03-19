# codex-rs/core/src/instructions/

User instruction loading and injection for AGENTS.md files and skill instructions.

## What this folder does

Manages the loading and serialization of user-provided instructions that get injected into the model context. Two instruction types are handled:

- **UserInstructions**: Content from `AGENTS.md` (or similar) files associated with a directory. Serialized with XML markers (`AGENTS_MD_FRAGMENT`) and injected as user-role messages into the conversation.
- **SkillInstructions**: Content from skill definition files, wrapped with skill-specific XML markers (`SKILL_FRAGMENT`) including the skill name and path.

## Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declaration, re-exports `UserInstructions`, `SkillInstructions`, `USER_INSTRUCTIONS_PREFIX` |
| `user_instructions.rs` | `UserInstructions` and `SkillInstructions` structs with serialization to `ResponseItem` |
| `user_instructions_tests.rs` | Tests for instruction serialization |

## Imports from

- `codex_protocol` -- `ResponseItem` for conversion
- `crate::contextual_user_message` -- `AGENTS_MD_FRAGMENT`, `SKILL_FRAGMENT` XML tag helpers

## Exports to

- `crate::codex` -- injected into conversation history during turn preparation
- `crate::skills` -- `SkillInstructions` used when skills are invoked
- `crate::custom_prompts` -- coordinates with user instruction loading
