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
            "one_line_native_entrypoints",
            "Keep the ordinary native-window entry point short, safe and target-selected by the framework.",
            "zsui::native_window(\"Example\").size(900, 620).run()? for desktop now and mobile once Activity/Ability hosts are real",
            "forcing users to choose raw HWND/AppKit/GTK/Activity/Ability objects for ordinary window creation",
            "src/native.rs, src/native_host_launch.rs",
            "prove Windows, macOS and Linux target smoke for the one-line path, then connect Android and Harmony hosts without changing the public entry shape",
        ),
        ZsuiRustFirstGoal::new(
            "composition_and_traits",
            "Use trait-based View/Component contracts and composition instead of inheritance trees.",
            "trait View<Msg> with composable builders such as button(\"Save\").padding(...).on_click(Msg::Save)",
            "Control base classes with Button/TextBox/ListView inheritance trees",
            "src/component_protocol.rs, src/components.rs, src/view.rs",
            "connect View<Msg> trees to richer native host input and layout passes",
        ),
        ZsuiRustFirstGoal::new(
            "typed_messages",
            "Prefer enum/typed messages over string event names.",
            "enum Msg variants returned by typed handlers such as on_click(Msg::SaveClicked)",
            "string event buses such as button.on(\"click\", callback)",
            "src/command_protocol.rs, src/event_protocol.rs, src/view.rs",
            "expand typed message builders across list, menu, tray and text input surfaces",
        ),
        ZsuiRustFirstGoal::new(
            "raii_native_resources",
            "Own native windows, fonts, bitmaps, tray icons and handles with RAII wrappers.",
            "owned Window/Icon/Tray/Font/Bitmap values and internal Drop-backed HWND/GDI/HDC/HBITMAP objects",
            "public APIs that require DestroyWindow, DeleteObject or Release calls from users",
            "src/native.rs, src/windows_win32_host.rs, src/windows_gdi_renderer.rs",
            "add required tray/menu popup interaction and cleanup target smoke proof while keeping raw HWNDs out of higher-level APIs",
        ),
        ZsuiRustFirstGoal::new(
            "zsclip_extraction_foundation",
            "Use reusable ZSClip native UI code as the extraction baseline while leaving product behavior behind.",
            "NativeDrawPlan, buffered no-flicker Win32/GDI painting, status/menu/settings contracts and owned native resources",
            "rewriting equivalent reusable host behavior from scratch or copying clipboard-product storage/sync logic into ZSUI",
            "src/render_protocol.rs, src/native_host_actions.rs, src/windows_gdi_renderer.rs, src/windows_win32_host.rs",
            "finish extracting reusable tray/menu/input host routes and keep the latest no-flicker self-draw path as the Windows baseline",
        ),
        ZsuiRustFirstGoal::new(
            "typed_units",
            "Use strong UI units such as Px, Dp and Dpi instead of mixing raw numeric sizes.",
            "Px, Dp, Dpi and UiLength conversions at layout/render boundaries",
            "raw i32/f32 values flowing through DPI, spacing and size APIs",
            "src/geometry.rs",
            "migrate remaining geometry/layout APIs from raw numeric DPI and spacing values",
        ),
        ZsuiRustFirstGoal::new(
            "compile_time_builders",
            "Push invalid app/window states toward compile-time builder constraints where practical.",
            "typestate builders for required title/content/runtime surfaces where they pay for themselves",
            "runtime-only missing-field failures for states the type system can express cleanly",
            "src/window.rs, src/app.rs",
            "introduce typestate builders for required title/content surfaces without breaking simple APIs",
        ),
        ZsuiRustFirstGoal::new(
            "explicit_context_no_globals",
            "Avoid global mutable app/theme/window/event registries; pass explicit context objects.",
            "update(&mut state, msg, &mut AppCx) and explicit Event/Layout/Paint contexts",
            "GlobalApp, GlobalTheme, GlobalWindowManager, GlobalEventBus or global widget registries",
            "src/host.rs, src/product_adapter.rs, src/view.rs",
            "wire AppCx/EventCx/PaintCx into native runtime and product adapter examples",
        ),
        ZsuiRustFirstGoal::new(
            "safe_public_api_isolated_unsafe",
            "Keep public APIs safe and isolate unsafe platform calls inside backend modules.",
            "safe builders such as zsui::native_window(...).run()? with unsafe limited to native backends",
            "raw HWND/HDC/HBITMAP/objc/GTK handles in declaration models or user-facing APIs",
            "src/native.rs, src/windows_win32_host.rs, src/windows_gdi_renderer.rs",
            "audit backend unsafe blocks and keep raw platform handles out of declaration models",
        ),
        ZsuiRustFirstGoal::new(
            "explicit_state_model",
            "Keep control state in explicit application state and derive UI from that state.",
            "view(&AppState) -> impl View<Msg> plus update(msg, &mut AppCx)",
            "controls hiding product state or mutating invisible framework-owned application data",
            "src/product_adapter.rs",
            "add examples for view(state) -> impl View<Msg> and update(msg, cx)",
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
            "src/components.rs, src/view.rs",
            "add tuple/array ergonomics and product adapter examples for declarative views",
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
            "mobile_native_hosts",
            "Treat Android and Harmony as native platform targets with explicit Activity/Ability host boundaries.",
            "Android Activity and Harmony Ability scaffolds, lifecycle bindings, FFI gates and device smoke artifacts",
            "pretending desktop tray/window semantics map exactly to mobile notifications, abilities or activities",
            "src/mobile_host.rs, src/android_activity_host.rs, src/harmony_ability_host.rs",
            "turn scaffold manifests into real Activity/Ability runtime bridges and require device smoke proof",
        ),
        ZsuiRustFirstGoal::new(
            "feature_gated_platform_capabilities",
            "Use Cargo features for widgets, services, platform backends and heavy dependencies.",
            "default-features = false with selected widget/backend features and optional dependencies",
            "eager global registration that makes every widget and backend enter every build",
            "Cargo.toml, src/feature_manifest.rs",
            "add feature-matrix CI and move heavy widgets into feature modules or crates",
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
            "narrow feature-gated windows-sys/windows-rs/AppKit/GTK/Android/Harmony bindings inside backend modules",
            "pulling broad native dependency surfaces into core declarations or user-facing APIs before they are needed",
            "Cargo.toml, src/windows_win32_host.rs, src/windows_gdi_renderer.rs",
            "use windows-rs or wider native APIs only for concrete Direct2D, composition, tray, input or renderer work",
        ),
        ZsuiRustFirstGoal::new(
            "strong_typed_ids",
            "Use strong typed IDs for widgets, windows, commands and resources instead of raw strings.",
            "WidgetId, WindowId, command IDs and typed window marker helpers",
            "raw strings such as focus(\"main_input\") or open_window(\"settings\")",
            "src/core.rs, src/geometry.rs, src/command_protocol.rs, src/view.rs",
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

        assert_eq!(names.len(), 19);
        assert!(names.contains(&"one_line_native_entrypoints"));
        assert!(names.contains(&"composition_and_traits"));
        assert!(names.contains(&"typed_messages"));
        assert!(names.contains(&"raii_native_resources"));
        assert!(names.contains(&"zsclip_extraction_foundation"));
        assert!(names.contains(&"mobile_native_hosts"));
        assert!(names.contains(&"feature_gated_platform_capabilities"));
        assert!(names.contains(&"crate_split_architecture"));
        assert!(names.contains(&"platform_api_on_demand"));
        assert!(names.contains(&"strong_typed_ids"));

        let goals = zsui_rust_first_goals();
        let typed_messages = goals
            .iter()
            .find(|goal| goal.goal_name == "typed_messages")
            .expect("typed message goal should exist");
        assert!(typed_messages.prefer.contains("enum Msg"));
        assert!(typed_messages.avoid.contains("button.on"));
    }
}
