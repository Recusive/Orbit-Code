# codex-rs/execpolicy/examples/

Example policy files demonstrating the `.codexpolicy` Starlark DSL syntax.

## What this folder does

Contains example exec policy files that illustrate the syntax and capabilities of the policy language. These are reference examples, not recommended for production use.

## Key files and their roles

- `example.codexpolicy` -- Demonstrates `prefix_rule()` with various configurations: forbidden rules with justification (`git reset --hard`), allow rules (`ls`, `cat`, `pwd`, `which`, `printenv`), prompt rules (`cp`), and match/not_match example annotations for self-testing rules.
