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
- Let the brief or reference design determine visual structure. Native Rust
  execution is mandatory; platform-styled component composition is an opt-in
  authoring route, not the default design template. Reuse control behavior
  where it fits, and use native drawing for deliberate custom visuals.
- For HTML design references, select `html-to-rust`: extract design facts,
  generate Rust layout/paint/state code, and compare native output. The shipped
  application must not depend on HTML/CSS interpretation, a DOM engine or WebView.
- Keep agent entry points small and feature-selectable; do not add a second UI
  runtime, global registry or heavy dependency merely to simplify authoring.
- Preserve the buffered no-flicker Windows paint path.
- Do not introduce an unrelated reactive runtime layer.
- Do not add global mutable widget/control registries.
- Do not mark a platform complete without target evidence.

## Verification

- Run the focused checks listed by the selected context pack for code changes.
  Documentation-only authoring changes require context-manifest validation,
  local-link checks and `git diff --check`; do not rebuild unrelated runtime code.
- For shared/public/release changes, run formatting, no-default tests, full
  tests and the locked feature matrix.
- Keep `docs/ai/context-packs.json` valid when files or verification commands
  move.
