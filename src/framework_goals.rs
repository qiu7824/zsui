use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ZsuiRustFirstGoal {
    pub goal_name: &'static str,
    pub principle: &'static str,
    pub prefer: &'static str,
    pub avoid: &'static str,
    pub current_surface: &'static str,
    pub next_step: &'static str,
}

impl ZsuiRustFirstGoal {
    pub const fn new(
        goal_name: &'static str,
        principle: &'static str,
        prefer: &'static str,
        avoid: &'static str,
        current_surface: &'static str,
        next_step: &'static str,
    ) -> Self {
        Self {
            goal_name,
            principle,
            prefer,
            avoid,
            current_surface,
            next_step,
        }
    }
}

pub fn zsui_rust_first_goal_names() -> Vec<&'static str> {
    zsui_rust_first_goals()
        .into_iter()
        .map(|goal| goal.goal_name)
        .collect()
}

pub fn zsui_rust_first_goals() -> Vec<ZsuiRustFirstGoal> {
    vec![
        ZsuiRustFirstGoal::new(
            "unified_application_authoring",
            "Require one platform-neutral Rust application source for Windows, macOS and Linux while preserving genuinely native platform composition and backend behavior inside ZSUI.",
            "one State/Msg/view/update/native_window path, semantic components and tokens, framework-owned PlatformExperience composition, and compile-time selected Host/Text/Raster/Presenter/Services profiles",
            "application platform cfg, platform enums, raw handles, renderer selection, duplicated per-platform view trees, or examples that patch platform visuals outside reusable framework rules",
            "src/view, src/native.rs, src/platform, src/style.rs, examples/desktop_native_showcase.rs, examples/component_gallery.rs, examples/zsui_notepad.rs",
            "make the shared application path the only ordinary desktop authoring path, move remaining platform composition and typography choices behind framework contracts, and gate it with one-source three-target proof",
        ),
        ZsuiRustFirstGoal::new(
            "native_proof_ci",
            "Require repeatable target-native runtime evidence to block UI regressions on every desktop platform.",
            "fixed target runners, final platform-view screenshots, versioned semantic reports, calibrated image comparison and reviewed read-only baselines",
            "treating cargo check or shared DrawPlan PNGs as platform proof, using moving runner labels, or automatically accepting changed baselines",
            ".github/workflows/ci.yml, docs/v0.3-native-proof-ci.md, docs/native-host-smoke.md, examples/component_gallery.rs, examples/zsui_notepad.rs",
            "keep the operational AppKit gate blocking regressions and align Win32 and Linux with the same reviewed baseline and comparison policy",
        ),
        ZsuiRustFirstGoal::new(
            "runnable_vertical_slices",
            "Measure progress by standalone input-state-paint loops on real hosts, not by declaration or contract count.",
            "a ZSUI-only control gallery with native interaction, repaint and target smoke proof",
            "raising overall completion from manifests, AI metadata or mobile scaffolds that do not run on a target",
            "examples/navigation_shell_layout.rs, examples/workbench_shell.rs, examples/zsui_notepad.rs, examples/zsui_calculator.rs, src/shell_layout.rs, src/workbench.rs, src/document_shell.rs, src/calculator.rs, src/native.rs",
            "attach the calculator runtime to the generic native builder, turn the notepad's native editor/file-dialog/lifecycle plumbing into reusable services, connect the workbench composer loop, then apply the same runtime gate to AppKit, GTK and Android",
        ),
        ZsuiRustFirstGoal::new(
            "one_line_native_entrypoints",
            "Keep the ordinary native-window entry point short, safe and target-selected by the framework.",
            "zsui::native_window(\"Example\").size(900, 620).run()? for desktop now and mobile once the Activity host is real",
            "forcing users to choose raw HWND/AppKit/GTK/Activity objects for ordinary window creation",
            "src/native.rs, src/native_host_launch.rs",
            "prove Windows, macOS and Linux target smoke for the one-line path, then connect the Android host without changing the public entry shape",
        ),
        ZsuiRustFirstGoal::new(
            "composition_and_traits",
            "Use trait-based View/Component contracts and composition instead of inheritance trees.",
            "trait View<Msg> with composable builders such as button(\"Save\").padding(...).on_click(Msg::Save)",
            "Control base classes with Button/TextBox/ListView inheritance trees",
            "src/component_protocol.rs, src/components.rs, src/view/mod.rs",
            "connect View<Msg> trees to richer native host input and layout passes",
        ),
        ZsuiRustFirstGoal::new(
            "typed_messages",
            "Prefer enum/typed messages over string event names.",
            "enum Msg variants returned by typed handlers such as on_click(Msg::SaveClicked)",
            "string event buses such as button.on(\"click\", callback)",
            "src/command_protocol.rs, src/event_protocol.rs, src/view/mod.rs",
            "expand typed message builders across list, menu, tray and text input surfaces",
        ),
        ZsuiRustFirstGoal::new(
            "raii_native_resources",
            "Own native windows, fonts, bitmaps, tray icons and handles with RAII wrappers.",
            "owned Window/Icon/Tray/Font/Bitmap values and internal Drop-backed HWND/GDI/HDC/HBITMAP objects",
            "public APIs that require DestroyWindow, DeleteObject or Release calls from users",
            "src/native.rs, src/platform/windows/mod.rs, src/windows_gdi_renderer.rs",
            "add required tray/menu popup interaction and cleanup target smoke proof while keeping raw HWNDs out of higher-level APIs",
        ),
        ZsuiRustFirstGoal::new(
            "production_native_foundation",
            "Use proven native host and rendering behavior as the framework baseline while keeping product behavior outside ZSUI.",
            "NativeDrawPlan, buffered no-flicker Win32/GDI painting, reusable document/workbench/calculator shells, status/menu/settings contracts and owned native resources",
            "duplicating established host behavior or placing application storage and sync logic inside ZSUI",
            "src/render_protocol.rs, src/native_host_actions.rs, src/windows_gdi_renderer.rs, src/platform/windows/mod.rs",
            "finish tray/menu/input host routes and keep buffered no-flicker self-draw as the Windows baseline",
        ),
        ZsuiRustFirstGoal::new(
            "typed_units",
            "Use strong UI units such as Px, Dp and Dpi instead of mixing raw numeric sizes.",
            "Px, Dp, Dpi and UiLength conversions at layout/render boundaries",
            "raw i32/f32 values flowing through DPI, spacing and size APIs",
            "src/geometry.rs",
            "convert remaining geometry/layout APIs from raw numeric DPI and spacing values",
        ),
        ZsuiRustFirstGoal::new(
            "compile_time_builders",
            "Push invalid app/window states toward compile-time builder constraints where practical.",
            "typestate builders for required title/content/runtime surfaces where they pay for themselves",
            "runtime-only missing-field failures for states the type system can express cleanly",
            "src/native.rs",
            "keep the opt-in native content typestate stable and add AppBuilder lifecycle typestate only if it prevents a demonstrated invalid state",
        ),
        ZsuiRustFirstGoal::new(
            "explicit_context_no_globals",
            "Avoid global mutable app/theme/window/event registries; pass explicit context objects.",
            "update(&mut state, msg, &mut AppCx) and explicit Event/Layout/Paint contexts",
            "GlobalApp, GlobalTheme, GlobalWindowManager, GlobalEventBus or global widget registries",
            "src/host.rs, src/product_adapter.rs, src/view/mod.rs",
            "wire AppCx/EventCx/PaintCx into native runtime and product adapter examples",
        ),
        ZsuiRustFirstGoal::new(
            "safe_public_api_isolated_unsafe",
            "Keep public APIs safe and isolate unsafe platform calls inside backend modules.",
            "safe builders such as zsui::native_window(...).run()? with unsafe limited to native backends",
            "raw HWND/HDC/HBITMAP/objc/GTK handles in declaration models or user-facing APIs",
            "src/native.rs, src/platform/windows/mod.rs, src/windows_gdi_renderer.rs",
            "audit backend unsafe blocks and keep raw platform handles out of declaration models",
        ),
        ZsuiRustFirstGoal::new(
            "explicit_state_model",
            "Keep control state in explicit application state and derive UI from that state.",
            "view(&AppState) -> impl View<Msg> plus update(msg, &mut AppCx)",
            "controls hiding product state or mutating invisible framework-owned application data",
            "src/view/mod.rs, src/native.rs, examples/rust_first_view.rs",
            "extend the live state and dual command-executor loop to AppKit and GTK, then connect asynchronous product events back into state updates",
        ),
        ZsuiRustFirstGoal::new(
            "theme_tokens",
            "Use theme tokens for color, radius, spacing and typography instead of scattered literals.",
            "theme.color.surface, theme.color.text_primary, theme.radius.medium and theme.spacing.md",
            "hard-coded Color::rgb(...) values scattered through widgets and examples",
            "src/render_protocol.rs, src/style.rs",
            "route all built-in view/widget styles through ZsuiTheme tokens",
        ),
        ZsuiRustFirstGoal::new(
            "declarative_rust_api",
            "Expose declarative Rust builders without XML, reflection or magic property names.",
            "Rust-analyzer friendly view builders such as column((text(...), list(...), button(...)))",
            "XML/XAML-style reflection, string bindings and magic property names",
            "src/components.rs, src/view/mod.rs",
            "add tuple/array ergonomics and product adapter examples for declarative views",
        ),
        ZsuiRustFirstGoal::new(
            "reloadable_ui_documents",
            "Allow visual-only UI changes to reload without invoking Cargo while preserving Rust-first typed state, messages and release trimming.",
            "a versioned semantic UiDocument, explicit typed binding manifest, schema validator, prebuilt native Viewer, stable-ID patches and release-time embedding",
            "a global string event bus, arbitrary reflection, browser pixels presented as native proof, development watchers in release builds or a mandatory two-process application runtime",
            "docs/v0.2-desktop-native.md, src/view/mod.rs, src/native.rs",
            "implement the document schema and zsui-uic validation first, then native auto-reload, compatible state retention, deterministic AI handoff and release embedding; defer the full drag-and-drop editor",
        ),
        ZsuiRustFirstGoal::new(
            "result_error_handling",
            "Return Result<T, ZsuiError> for host/backend failures instead of panicking.",
            "fallible app/window/icon/font/backend construction returning Result<T, ZsuiError>",
            "panic, unwrap or MessageBoxW for recoverable host/backend failures",
            "src/core.rs, src/native.rs",
            "remove remaining unwrap-style backend assumptions before marking runtime complete",
        ),
        ZsuiRustFirstGoal::new(
            "capability_traits",
            "Represent platform differences with traits and capability reports instead of fake uniformity.",
            "TrayBackend, HotkeyBackend, DialogBackend and capability reports with honest unsupported states",
            "one fake uniform API pretending every desktop/mobile platform supports the same native services",
            "src/capability.rs, src/host_protocol.rs, src/native_hosts.rs",
            "split tray, hotkey, dialog, file-picker and shell-open backend traits into real hosts",
        ),
        ZsuiRustFirstGoal::new(
            "mobile_native_host",
            "Treat Android as a native platform target with an explicit Activity host boundary.",
            "Android Activity scaffold, lifecycle bindings, FFI gates and device smoke artifacts",
            "pretending desktop tray/window semantics map exactly to mobile notifications or activities",
            "src/mobile_host.rs, src/android_activity_host.rs",
            "turn the scaffold manifest into a real Activity runtime bridge and require device smoke proof",
        ),
        ZsuiRustFirstGoal::new(
            "feature_gated_platform_capabilities",
            "Use Cargo features for widgets, services, platform backends and heavy dependencies.",
            "default-features = false with selected widget/backend features and optional dependencies",
            "eager global registration that makes every widget and backend enter every build",
            "Cargo.toml, src/feature_manifest.rs, scripts/check-feature-matrix.ps1, .github/workflows/ci.yml",
            "move heavy widgets into feature modules or crates while keeping every new public feature in the matrix gate",
        ),
        ZsuiRustFirstGoal::new(
            "task_scoped_ai_context",
            "Keep AI bootstrap context small and load implementation knowledge by task instead of reading the repository as one prompt.",
            "docs/ai-agent.md plus one validated context pack with required paths, optional paths and focused checks",
            "bulk-loading all source, documentation, examples, generated artifacts and readiness metadata before every task",
            "AGENTS.md, docs/ai-agent.md, docs/ai/context-packs.json, scripts/ai-context.ps1",
            "keep pack paths valid in CI, measure bootstrap size and split any pack that grows beyond one ownership boundary",
        ),
        ZsuiRustFirstGoal::new(
            "crate_split_architecture",
            "Let the framework grow through small feature-gated crates or modules instead of one untrimmed mega-crate.",
            "zsui-core, zsui-shell, zsui-render, zsui-style and widget-family crates such as zsui-widgets-input/list/extra",
            "placing every widget, renderer and platform binding in one always-enabled crate",
            "Cargo.toml, src/feature_manifest.rs",
            "split heavier widget families, renderers and backend integrations once their contracts are stable enough to publish separately",
        ),
        ZsuiRustFirstGoal::new(
            "platform_api_on_demand",
            "Add platform API crates and bindings only when a concrete backend surface requires them.",
            "narrow feature-gated windows-sys/windows-rs/AppKit/GTK/Android bindings inside backend modules",
            "pulling broad native dependency surfaces into core declarations or user-facing APIs before they are needed",
            "Cargo.toml, src/platform/windows/mod.rs, src/windows_gdi_renderer.rs",
            "use windows-rs or wider native APIs only for concrete Direct2D, composition, tray, input or renderer work",
        ),
        ZsuiRustFirstGoal::new(
            "strong_typed_ids",
            "Use strong typed IDs for widgets, windows, commands and resources instead of raw strings.",
            "WidgetId, WindowId, command IDs and typed window marker helpers",
            "raw strings such as focus(\"main_input\") or open_window(\"settings\")",
            "src/core.rs, src/geometry.rs, src/command_protocol.rs, src/view/mod.rs",
            "add resource IDs and typed window-marker helpers where they reduce runtime lookup errors",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_first_goal_manifest_tracks_core_direction() {
        let names = zsui_rust_first_goal_names();

        assert_eq!(names.len(), 24);
        assert!(names.contains(&"unified_application_authoring"));
        assert!(names.contains(&"native_proof_ci"));
        assert!(names.contains(&"runnable_vertical_slices"));
        assert!(names.contains(&"one_line_native_entrypoints"));
        assert!(names.contains(&"composition_and_traits"));
        assert!(names.contains(&"typed_messages"));
        assert!(names.contains(&"raii_native_resources"));
        assert!(names.contains(&"production_native_foundation"));
        assert!(names.contains(&"mobile_native_host"));
        assert!(names.contains(&"feature_gated_platform_capabilities"));
        assert!(names.contains(&"task_scoped_ai_context"));
        assert!(names.contains(&"crate_split_architecture"));
        assert!(names.contains(&"platform_api_on_demand"));
        assert!(names.contains(&"strong_typed_ids"));
        assert!(names.contains(&"reloadable_ui_documents"));

        let goals = zsui_rust_first_goals();
        let unified_authoring = goals
            .iter()
            .find(|goal| goal.goal_name == "unified_application_authoring")
            .expect("unified application authoring goal should exist");
        assert!(unified_authoring.prefer.contains("State/Msg/view/update"));
        assert!(unified_authoring.avoid.contains("application platform cfg"));

        let typed_messages = goals
            .iter()
            .find(|goal| goal.goal_name == "typed_messages")
            .expect("typed message goal should exist");
        assert!(typed_messages.prefer.contains("enum Msg"));
        assert!(typed_messages.avoid.contains("button.on"));

        let reloadable = goals
            .iter()
            .find(|goal| goal.goal_name == "reloadable_ui_documents")
            .expect("reloadable UI document goal should exist");
        assert!(reloadable.prefer.contains("UiDocument"));
        assert!(reloadable.avoid.contains("browser pixels"));
    }
}
