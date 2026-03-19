# .codex/

This file applies to `.codex/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- These directories define Codex skills and their support files. Keep the runnable instructions, helper scripts, and references aligned so the skill still works end-to-end when invoked.
- Prefer editing the smallest scope that owns the behavior: skill-level docs in the skill root, reusable snippets in `references/`, worker prompts in `agents/`, and executable helpers in `scripts/`.

## Validate
- Validate by reading the skill from the top-level `SKILL.md` or directory doc and checking that referenced relative paths still exist.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### What This Folder Does

This is the configuration root for Codex skills — reusable, declarative automation modules that extend the Codex CLI agent with domain-specific capabilities. Skills are invoked by name when a user asks Codex to perform a specialized task (e.g., "babysit this PR" or "test the TUI").

### Structure

```
.codex/
└── skills/
    ├── babysit-pr/   # Automated PR monitoring, CI triage, and review handling
    └── test-tui/     # Interactive TUI testing guidance
```

### What It Plugs Into

- **Codex CLI agent** (`codex-rs/`): The agent discovers skills from this directory at runtime. When a user's request matches a skill's `name` or `description` (defined in each skill's `SKILL.md` front matter), the agent loads the skill's instructions and tools.
- **Codex skills system** (`codex-rs/skills/`): The Rust crate that handles skill discovery, loading, and registration reads from `.codex/skills/`.

### How Skills Work

Each skill is a subdirectory under `.codex/skills/` containing:

1. **`SKILL.md`** (required): YAML front matter with `name` and `description`, followed by Markdown instructions the agent follows when the skill is activated.
2. **`agents/`** (optional): Agent configuration YAML files (e.g., OpenAI agent definitions).
3. **`references/`** (optional): Supporting documentation the agent can consult during execution.
4. **`scripts/`** (optional): Executable scripts the skill invokes as tools.

### Key Conventions

- Skills are self-contained: all scripts, references, and agent configs live within the skill's directory.
- The `description` field in `SKILL.md` front matter is used for skill matching — it should be detailed enough for the agent to identify when to activate the skill.
- Scripts are called via their relative path from the repo root (e.g., `python3 .codex/skills/babysit-pr/scripts/gh_pr_watch.py`).
