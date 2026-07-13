# ZSUI Agent Instructions

## Bootstrap

- Read `docs/ai-agent.md` first and nothing else until the task is classified.
- Select one pack with `scripts/ai-context.ps1 -Pack <id>`.
- After classification, read `docs/ai/project-memory.md` before implementation
  so durable product decisions and user priorities are not lost between tasks.
- Read required paths first. Read optional paths only when blocked by a concrete
  unanswered question.
- Use `rg` to locate symbols and read focused ranges. Do not recursively dump
  `src/`, `docs/`, `examples/` or generated output into context.
- Do not load `docs/ai/reference.md` or `src/agent_context.rs` unless the task
  is a completion/readiness audit.

## Scope

- Modify this repository only. Do not edit sibling product repositories.
- Keep product data, persistence, sync and business behavior outside ZSUI.
- Public documentation describes ZSUI as its own framework and should not carry
  external product history.

## Engineering

- Prefer composition, traits, typed messages, explicit state and strong IDs.
- Keep public APIs safe and raw platform handles inside backend modules.
- Keep optional dependencies and advanced capabilities behind Cargo features.
- Use shared typed units, theme tokens and semantic icons in built-in UI.
- Preserve the buffered no-flicker Windows paint path.
- Do not introduce an unrelated reactive runtime layer.
- Do not add global mutable widget/control registries.
- Do not mark a platform complete without target evidence.

## Verification

- Run the focused checks listed by the selected context pack.
- For shared/public/release changes, run formatting, no-default tests, full
  tests and the locked feature matrix.
- Keep `docs/ai/context-packs.json` valid when files or verification commands
  move.
