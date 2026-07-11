# ZSUI AI Bootstrap

This is the only document an AI agent should read before it knows the task.
Do not recursively read `src/`, `docs/`, `examples/`, generated artifacts or
the full readiness report during bootstrap.

## Minimal-Context Contract

1. Read this file only.
2. Match the task to one context pack below.
3. Run `scripts/ai-context.ps1 -Pack <id>`.
4. Read the returned `required` files and search inside them with `rg`.
5. Read `optional` files only when a concrete question remains unanswered.
6. Add a second pack only when the task genuinely crosses ownership boundaries.
7. Run the verification commands returned by the selected pack.

The routing source of truth is `docs/ai/context-packs.json`. This file stays
small on purpose; detailed readiness material lives in `docs/ai/reference.md`
and is not part of normal task context.

```powershell
.\scripts\ai-context.ps1 -List
.\scripts\ai-context.ps1 -Pack quickstart
.\scripts\ai-context.ps1 -Pack calculator -Format Json
.\scripts\ai-context.ps1 -Pack windows-renderer -IncludeOptional
```

## Baseline Facts

- ZSUI is a Rust-first native UI framework, not a browser shell.
- Composition, traits, typed messages and explicit state are preferred over
  inheritance, reflection and string event buses.
- Public APIs stay safe; raw platform handles and `unsafe` stay in backends.
- Product data, persistence, sync and business behavior stay in the product.
- Cargo defaults remain small; advanced controls, services and backends are
  explicit features or optional dependencies.
- Windows is the strongest real runtime today. macOS/Linux are first-pass
  desktop paths. Android/Harmony still require real runtime and device proof.
- The component catalog currently tracks 48 families: 20 first-pass runtime,
  8 contract-only and 20 not started. Composite shells do not change that count.

## Task Router

| Pack | Use it for |
| --- | --- |
| `quickstart` | Public entry points, one-line windows, basic examples |
| `features` | Cargo features, optional dependencies, compile trimming |
| `view-widgets` | `View<Msg>`, state/update, basic controls, input routing |
| `navigation-shell` | Left navigation, grouped cards, settings rows, scroll |
| `workbench` | Conversation/task shell, composer, inspector, message blocks |
| `document-shell` | Document chrome, native editor inset, notepad example |
| `calculator` | Decimal engine, keypad shell, calculator example/measurement |
| `windows-renderer` | Win32/GDI+, no-flicker paint, DPI, icons, pointer input |
| `desktop-hosts` | Windows/macOS/Linux host and capability boundaries |
| `mobile-hosts` | Android Activity and Harmony Ability contracts/proof |
| `product-adapter` | Product boundary, runtime harness, command executors |
| `completion-audit` | Full progress, component count, platform evidence |
| `release` | Formatting, tests, feature matrix, docs and release checks |

Do not load `completion-audit` just to implement a control. Do not load
platform backends for a pure layout or engine task.

## Non-Negotiable Engineering Rules

- Reuse existing module patterns and keep edits within the selected ownership
  boundary.
- Do not add a global mutable widget/control registry.
- Keep every heavy dependency optional and every new public feature in
  `scripts/check-feature-matrix.ps1`.
- Use `Dp`/`Px`/`Dpi`, theme tokens and semantic `ZsIcon` values in built-in UI.
- Do not place private icon-font code points or local palettes in components.
- Preserve buffered no-flicker Windows painting and background-erase
  suppression when changing self-drawn surfaces.
- Do not introduce an unrelated reactive runtime layer. Use platform APIs only
  for a concrete backend need.
- Return `Result<T, ZsuiError>` for recoverable failures; do not expose raw
  handle cleanup to users.
- Treat code-level, target-smoke and system-complete as different statuses.
  Never claim a platform complete from declarations or scaffolds alone.
- Do not rewrite a stable native solution inside an example when a reusable
  framework service already owns it.

## Reading Discipline

Use `rg` before opening a large source file:

```powershell
rg -n "NativeWindowBuilder|stateful_view" src\native.rs
rg -n "ZsCalculatorEngine|ZsCalculatorShellSpec" src\calculator.rs
```

Read focused ranges around matches rather than dumping entire files. Skip
`target/`, package output and comparison build directories unless a measurement
pack explicitly names an artifact. Prefer public exports in `src/lib.rs` over
reading every implementation module.

When a task is complete, update only the context pack whose required paths or
verification commands changed. Do not grow this bootstrap with implementation
history.

## Verification Rule

Run the selected pack's focused checks first. Run the full gate only for shared
protocols, public feature changes, renderer/host changes or release work:

```powershell
cargo fmt --all -- --check
cargo test --no-default-features --quiet
cargo test --features full --quiet
.\scripts\check-feature-matrix.ps1 -Locked
```

Target-specific completion additionally requires the artifact rules in
`docs/native-host-smoke.md`.

## Optional Deep References

Load these only when the selected pack requests them:

- `docs/ai/reference.md`: full readiness and public-surface reference
- `src/agent_context.rs`: machine-readable readiness model
- `docs/skills/zsui-native-ui/`: detailed native-host workflow
- `docs/architecture.md`: ownership and layering
- `docs/framework-goals.md`: long-range Rust-first direction

For ordinary feature work, this bootstrap plus one context pack is sufficient.
