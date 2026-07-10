use crate::{
    capability::{CapabilityStatus, HostCapabilities},
    components::{UiNode, UiNodeKind},
    core::{Command, TrayId, WindowId, ZsuiError, ZsuiResult},
    host::{PlatformHost, ZsuiHost},
    hotkey::HotkeySpec,
    menu::{MenuItemSpec, MenuSpec},
    settings::{SettingsItemKind, SettingsPageSpec, SettingsValue},
    tray::TraySpec,
    window::WindowSpec,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub fn app(name: impl Into<String>) -> AppBuilder {
    AppBuilder::new(name)
}

pub const ZSUI_DECLARATION_AUDIT_SURFACES: &[&str] = &[
    "app_name",
    "windows",
    "window_content_tree",
    "tray",
    "menus",
    "hotkeys",
    "settings_pages",
    "commands",
    "host_capability_degradation",
];

pub fn zsui_declaration_audit_surface_names() -> Vec<&'static str> {
    ZSUI_DECLARATION_AUDIT_SURFACES.to_vec()
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppBuilder {
    name: String,
    windows: Vec<WindowSpec>,
    tray: Option<TraySpec>,
    hotkeys: Vec<HotkeySpec>,
    settings_pages: Vec<SettingsPageSpec>,
}

impl AppBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            windows: Vec::new(),
            tray: None,
            hotkeys: Vec::new(),
            settings_pages: Vec::new(),
        }
    }

    pub fn window(mut self, spec: WindowSpec) -> Self {
        self.windows.push(spec);
        self
    }

    pub fn tray(mut self, spec: TraySpec) -> Self {
        self.tray = Some(spec);
        self
    }

    pub fn global_hotkey(mut self, accelerator: impl Into<String>, command: Command) -> Self {
        self.hotkeys.push(HotkeySpec::new(accelerator, command));
        self
    }

    pub fn hotkey(mut self, spec: HotkeySpec) -> Self {
        self.hotkeys.push(spec);
        self
    }

    pub fn settings_page(mut self, spec: SettingsPageSpec) -> Self {
        self.settings_pages.push(spec);
        self
    }

    pub fn build(self) -> ZsuiResult<ZsuiApp> {
        self.declaration_report().ensure_valid()?;
        Ok(ZsuiApp {
            name: self.name,
            windows: self.windows,
            tray: self.tray,
            hotkeys: self.hotkeys,
            settings_pages: self.settings_pages,
        })
    }

    pub fn declaration_report(&self) -> ZsuiAppDeclarationReport {
        self.declaration_report_for(&HostCapabilities::all_supported(
            crate::capability::PlatformName::Unknown,
        ))
    }

    pub fn declaration_report_for(
        &self,
        capabilities: &HostCapabilities,
    ) -> ZsuiAppDeclarationReport {
        declaration_report_for_parts(
            &self.name,
            &self.windows,
            self.tray.as_ref(),
            &self.hotkeys,
            &self.settings_pages,
            capabilities,
        )
    }

    pub fn run(self) -> ZsuiResult<ZsuiAppRuntime> {
        let mut host = PlatformHost::new();
        self.run_with_host(&mut host)
    }

    pub fn run_with_host<H: ZsuiHost>(self, host: &mut H) -> ZsuiResult<ZsuiAppRuntime> {
        self.build()?.run_with_host(host)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZsuiApp {
    pub name: String,
    pub windows: Vec<WindowSpec>,
    pub tray: Option<TraySpec>,
    pub hotkeys: Vec<HotkeySpec>,
    pub settings_pages: Vec<SettingsPageSpec>,
}

impl ZsuiApp {
    pub fn declaration_report(&self) -> ZsuiAppDeclarationReport {
        self.declaration_report_for(&HostCapabilities::all_supported(
            crate::capability::PlatformName::Unknown,
        ))
    }

    pub fn declaration_report_for(
        &self,
        capabilities: &HostCapabilities,
    ) -> ZsuiAppDeclarationReport {
        audit_app_declaration(self, capabilities)
    }

    pub fn run_with_host<H: ZsuiHost>(&self, host: &mut H) -> ZsuiResult<ZsuiAppRuntime> {
        let capabilities = host.capabilities();
        let mut window_ids = Vec::new();
        let mut hotkey_ids = Vec::new();

        for window in &self.windows {
            window_ids.push(host.create_main_window(window)?);
        }

        let tray_id = if let Some(tray) = &self.tray {
            Some(host.create_tray(tray)?)
        } else {
            None
        };

        for hotkey in &self.hotkeys {
            if hotkey.enabled {
                hotkey_ids.push(host.register_global_hotkey(hotkey)?);
            }
        }

        host.run_event_loop()?;

        Ok(ZsuiAppRuntime {
            app_name: self.name.clone(),
            windows: window_ids,
            tray: tray_id,
            hotkeys: hotkey_ids,
            settings_pages: self.settings_pages.len(),
            capabilities: capabilities.clone(),
            degraded_capabilities: used_degraded_capabilities(self, &capabilities),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZsuiAppRuntime {
    pub app_name: String,
    pub windows: Vec<WindowId>,
    pub tray: Option<TrayId>,
    pub hotkeys: Vec<crate::core::HotkeyId>,
    pub settings_pages: usize,
    pub capabilities: HostCapabilities,
    pub degraded_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsuiDeclarationIssueLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsuiDeclarationIssue {
    pub level: ZsuiDeclarationIssueLevel,
    pub path: String,
    pub message: String,
}

impl ZsuiDeclarationIssue {
    pub fn error(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: ZsuiDeclarationIssueLevel::Error,
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn warning(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: ZsuiDeclarationIssueLevel::Warning,
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn is_error(&self) -> bool {
        self.level == ZsuiDeclarationIssueLevel::Error
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsuiAppDeclarationReport {
    pub app_name: String,
    pub platform: crate::capability::PlatformName,
    pub window_count: usize,
    pub tray_declared: bool,
    pub hotkey_count: usize,
    pub settings_page_count: usize,
    pub issue_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub degraded_capabilities: Vec<String>,
    pub issues: Vec<ZsuiDeclarationIssue>,
}

impl ZsuiAppDeclarationReport {
    pub fn is_valid(&self) -> bool {
        self.error_count == 0
    }

    pub fn first_error(&self) -> Option<&ZsuiDeclarationIssue> {
        self.issues.iter().find(|issue| issue.is_error())
    }

    pub fn ensure_valid(&self) -> ZsuiResult<()> {
        if let Some(issue) = self.first_error() {
            Err(ZsuiError::invalid_spec(
                issue.path.clone(),
                issue.message.clone(),
            ))
        } else {
            Ok(())
        }
    }
}

pub fn audit_app_declaration(
    app: &ZsuiApp,
    capabilities: &HostCapabilities,
) -> ZsuiAppDeclarationReport {
    declaration_report_for_parts(
        &app.name,
        &app.windows,
        app.tray.as_ref(),
        &app.hotkeys,
        &app.settings_pages,
        capabilities,
    )
}

fn declaration_report_for_parts(
    name: &str,
    windows: &[WindowSpec],
    tray: Option<&TraySpec>,
    hotkeys: &[HotkeySpec],
    settings_pages: &[SettingsPageSpec],
    capabilities: &HostCapabilities,
) -> ZsuiAppDeclarationReport {
    let mut issues = Vec::new();

    validate_app_name(name, &mut issues);
    validate_windows(windows, &mut issues);
    if let Some(tray) = tray {
        validate_tray(tray, &mut issues);
    }
    validate_hotkeys(hotkeys, &mut issues);
    validate_settings_pages(settings_pages, &mut issues);

    let degraded_capabilities = used_degraded_capabilities_for_parts(
        windows,
        tray.is_some(),
        hotkeys,
        settings_pages,
        capabilities,
    );
    for detail in &degraded_capabilities {
        issues.push(ZsuiDeclarationIssue::warning(
            "capabilities",
            detail.clone(),
        ));
    }

    let error_count = issues.iter().filter(|issue| issue.is_error()).count();
    let issue_count = issues.len();
    ZsuiAppDeclarationReport {
        app_name: name.to_string(),
        platform: capabilities.platform.clone(),
        window_count: windows.len(),
        tray_declared: tray.is_some(),
        hotkey_count: hotkeys.len(),
        settings_page_count: settings_pages.len(),
        issue_count,
        error_count,
        warning_count: issue_count - error_count,
        degraded_capabilities,
        issues,
    }
}

fn validate_app_name(name: &str, issues: &mut Vec<ZsuiDeclarationIssue>) {
    if name.trim().is_empty() {
        issues.push(ZsuiDeclarationIssue::error(
            "app.name",
            "app name cannot be empty",
        ));
    }
}

fn validate_windows(windows: &[WindowSpec], issues: &mut Vec<ZsuiDeclarationIssue>) {
    for (index, window) in windows.iter().enumerate() {
        validate_window(window, &format!("windows[{index}]"), issues);
    }
}

fn validate_window(window: &WindowSpec, path: &str, issues: &mut Vec<ZsuiDeclarationIssue>) {
    if window.title.trim().is_empty() {
        issues.push(ZsuiDeclarationIssue::error(
            format!("{path}.title"),
            "window title cannot be empty",
        ));
    }
    if window.width == 0 || window.height == 0 {
        issues.push(ZsuiDeclarationIssue::error(
            format!("{path}.size"),
            "window size must be greater than zero",
        ));
    }
    if window.min_width == Some(0) || window.min_height == Some(0) {
        issues.push(ZsuiDeclarationIssue::error(
            format!("{path}.min_size"),
            "window minimum size must be greater than zero",
        ));
    }
    if let Some(icon_path) = &window.icon_path {
        if icon_path.trim().is_empty() {
            issues.push(ZsuiDeclarationIssue::error(
                format!("{path}.icon_path"),
                "window icon path cannot be empty",
            ));
        }
    }
    if let Some(min_width) = window.min_width {
        if min_width > window.width {
            issues.push(ZsuiDeclarationIssue::warning(
                format!("{path}.min_width"),
                "window minimum width is greater than requested width",
            ));
        }
    }
    if let Some(min_height) = window.min_height {
        if min_height > window.height {
            issues.push(ZsuiDeclarationIssue::warning(
                format!("{path}.min_height"),
                "window minimum height is greater than requested height",
            ));
        }
    }
    if let Some(content) = &window.content {
        let mut ids = HashSet::new();
        validate_ui_node(content, &format!("{path}.content"), &mut ids, issues);
    }
}

fn validate_ui_node(
    node: &UiNode,
    path: &str,
    ids: &mut HashSet<String>,
    issues: &mut Vec<ZsuiDeclarationIssue>,
) {
    if node.id.trim().is_empty() {
        issues.push(ZsuiDeclarationIssue::error(
            format!("{path}.id"),
            "ui node id cannot be empty",
        ));
    } else if !ids.insert(node.id.clone()) {
        issues.push(ZsuiDeclarationIssue::error(
            format!("{path}.id"),
            format!("duplicate ui node id `{}`", node.id),
        ));
    }

    match &node.kind {
        #[cfg(feature = "button")]
        UiNodeKind::Button { label, command } => {
            if label.trim().is_empty() {
                issues.push(ZsuiDeclarationIssue::error(
                    format!("{path}.label"),
                    "button label cannot be empty",
                ));
            }
            validate_command(command, &format!("{path}.command"), issues);
        }
        #[cfg(feature = "textbox")]
        UiNodeKind::TextInput { label, .. } => {
            if label.trim().is_empty() {
                issues.push(ZsuiDeclarationIssue::error(
                    format!("{path}.label"),
                    "text input label cannot be empty",
                ));
            }
        }
        #[cfg(feature = "checkbox")]
        UiNodeKind::Checkbox { label, command, .. } => {
            if label.trim().is_empty() {
                issues.push(ZsuiDeclarationIssue::error(
                    format!("{path}.label"),
                    "checkbox label cannot be empty",
                ));
            }
            if let Some(command) = command {
                validate_command(command, &format!("{path}.command"), issues);
            }
        }
        #[cfg(feature = "toggle")]
        UiNodeKind::Toggle { command, .. } => {
            if let Some(command) = command {
                validate_command(command, &format!("{path}.command"), issues);
            }
        }
        UiNodeKind::Spacer { size } if *size == 0 => {
            issues.push(ZsuiDeclarationIssue::warning(
                format!("{path}.size"),
                "zero-sized spacer has no layout effect",
            ));
        }
        _ => {}
    }

    for (index, child) in node.children.iter().enumerate() {
        validate_ui_node(child, &format!("{path}.children[{index}]"), ids, issues);
    }
}

fn validate_tray(tray: &TraySpec, issues: &mut Vec<ZsuiDeclarationIssue>) {
    if let Some(tooltip) = &tray.tooltip {
        if tooltip.trim().is_empty() {
            issues.push(ZsuiDeclarationIssue::error(
                "tray.tooltip",
                "tray tooltip cannot be empty",
            ));
        }
    }
    if let Some(icon_path) = &tray.icon_path {
        if icon_path.trim().is_empty() {
            issues.push(ZsuiDeclarationIssue::error(
                "tray.icon_path",
                "tray icon path cannot be empty",
            ));
        }
    }
    if tray.tooltip.is_none() && tray.icon_path.is_none() && tray.menu.items.is_empty() {
        issues.push(ZsuiDeclarationIssue::warning(
            "tray",
            "tray declaration has no tooltip, icon path, or menu items",
        ));
    }

    let mut menu_ids = HashMap::new();
    validate_menu(&tray.menu, "tray.menu", &mut menu_ids, issues);
}

fn validate_menu(
    menu: &MenuSpec,
    path: &str,
    ids: &mut HashMap<String, String>,
    issues: &mut Vec<ZsuiDeclarationIssue>,
) {
    if let Some(id) = &menu.id {
        validate_optional_id(id, &format!("{path}.id"), "menu id", issues);
        validate_unique_id(id, &format!("{path}.id"), "menu id", ids, issues);
    }
    if let Some(title) = &menu.title {
        if title.trim().is_empty() {
            issues.push(ZsuiDeclarationIssue::error(
                format!("{path}.title"),
                "menu title cannot be empty",
            ));
        }
    }
    if menu.items.is_empty() {
        issues.push(ZsuiDeclarationIssue::warning(
            format!("{path}.items"),
            "menu has no items",
        ));
    }

    for (index, item) in menu.items.iter().enumerate() {
        validate_menu_item(item, &format!("{path}.items[{index}]"), ids, issues);
    }
}

fn validate_menu_item(
    item: &MenuItemSpec,
    path: &str,
    ids: &mut HashMap<String, String>,
    issues: &mut Vec<ZsuiDeclarationIssue>,
) {
    match item {
        MenuItemSpec::Command {
            id,
            label,
            command,
            accelerator,
            ..
        } => {
            if let Some(id) = id {
                validate_optional_id(id, &format!("{path}.id"), "menu item id", issues);
                validate_unique_id(id, &format!("{path}.id"), "menu item id", ids, issues);
            }
            if label.trim().is_empty() {
                issues.push(ZsuiDeclarationIssue::error(
                    format!("{path}.label"),
                    "menu item label cannot be empty",
                ));
            }
            if let Some(accelerator) = accelerator {
                if accelerator.trim().is_empty() {
                    issues.push(ZsuiDeclarationIssue::error(
                        format!("{path}.accelerator"),
                        "menu item accelerator cannot be empty",
                    ));
                }
            }
            validate_command(command, &format!("{path}.command"), issues);
        }
        MenuItemSpec::Separator => {}
        MenuItemSpec::Submenu {
            id, label, menu, ..
        } => {
            if let Some(id) = id {
                validate_optional_id(id, &format!("{path}.id"), "submenu id", issues);
                validate_unique_id(id, &format!("{path}.id"), "submenu id", ids, issues);
            }
            if label.trim().is_empty() {
                issues.push(ZsuiDeclarationIssue::error(
                    format!("{path}.label"),
                    "submenu label cannot be empty",
                ));
            }
            validate_menu(menu, &format!("{path}.menu"), ids, issues);
        }
    }
}

fn validate_hotkeys(hotkeys: &[HotkeySpec], issues: &mut Vec<ZsuiDeclarationIssue>) {
    let mut accelerators = HashMap::new();
    for (index, hotkey) in hotkeys.iter().enumerate() {
        validate_hotkey(
            hotkey,
            &format!("hotkeys[{index}]"),
            &mut accelerators,
            issues,
        );
    }
}

fn validate_hotkey(
    hotkey: &HotkeySpec,
    path: &str,
    accelerators: &mut HashMap<String, String>,
    issues: &mut Vec<ZsuiDeclarationIssue>,
) {
    if hotkey.enabled {
        if hotkey.accelerator.trim().is_empty() {
            issues.push(ZsuiDeclarationIssue::error(
                format!("{path}.accelerator"),
                "enabled hotkey accelerator cannot be empty",
            ));
        } else {
            let normalized = normalize_hotkey_accelerator(&hotkey.accelerator);
            if let Some(first_path) = accelerators.insert(normalized, path.to_string()) {
                issues.push(ZsuiDeclarationIssue::error(
                    format!("{path}.accelerator"),
                    format!(
                        "duplicate enabled hotkey accelerator `{}`; first declared at {first_path}",
                        hotkey.accelerator
                    ),
                ));
            }
        }
    }
    validate_command(&hotkey.command, &format!("{path}.command"), issues);
}

fn validate_settings_pages(
    settings_pages: &[SettingsPageSpec],
    issues: &mut Vec<ZsuiDeclarationIssue>,
) {
    let mut page_ids = HashMap::new();
    for (index, page) in settings_pages.iter().enumerate() {
        validate_settings_page(
            page,
            &format!("settings_pages[{index}]"),
            &mut page_ids,
            issues,
        );
    }
}

fn validate_settings_page(
    page: &SettingsPageSpec,
    path: &str,
    page_ids: &mut HashMap<String, String>,
    issues: &mut Vec<ZsuiDeclarationIssue>,
) {
    validate_required_id(&page.id, &format!("{path}.id"), "settings page id", issues);
    validate_unique_id(
        &page.id,
        &format!("{path}.id"),
        "settings page id",
        page_ids,
        issues,
    );
    if page.title.trim().is_empty() {
        issues.push(ZsuiDeclarationIssue::error(
            format!("{path}.title"),
            "settings page title cannot be empty",
        ));
    }
    if page.items.is_empty() {
        issues.push(ZsuiDeclarationIssue::warning(
            format!("{path}.items"),
            "settings page has no items",
        ));
    }

    let mut item_ids = HashMap::new();
    for (index, item) in page.items.iter().enumerate() {
        let item_path = format!("{path}.items[{index}]");
        validate_required_id(
            &item.id,
            &format!("{item_path}.id"),
            "settings item id",
            issues,
        );
        validate_unique_id(
            &item.id,
            &format!("{item_path}.id"),
            "settings item id",
            &mut item_ids,
            issues,
        );
        if item.label.trim().is_empty() {
            issues.push(ZsuiDeclarationIssue::error(
                format!("{item_path}.label"),
                "settings item label cannot be empty",
            ));
        }
        validate_settings_item_kind(item, &item_path, issues);
        if let Some(command) = &item.command {
            validate_command(command, &format!("{item_path}.command"), issues);
        }
    }
}

fn validate_settings_item_kind(
    item: &crate::settings::SettingsItemSpec,
    path: &str,
    issues: &mut Vec<ZsuiDeclarationIssue>,
) {
    match &item.kind {
        SettingsItemKind::Toggle => {
            validate_default_value_kind(
                item.default_value.as_ref(),
                path,
                "toggle",
                |value| matches!(value, SettingsValue::Bool(_)),
                issues,
            );
        }
        SettingsItemKind::Text => {
            validate_default_value_kind(
                item.default_value.as_ref(),
                path,
                "text",
                |value| matches!(value, SettingsValue::Text(_)),
                issues,
            );
        }
        SettingsItemKind::Number { min, max } => {
            validate_number_bound(*min, &format!("{path}.kind.min"), issues);
            validate_number_bound(*max, &format!("{path}.kind.max"), issues);
            if let (Some(min), Some(max)) = (min, max) {
                if min > max {
                    issues.push(ZsuiDeclarationIssue::error(
                        format!("{path}.kind"),
                        "number settings item min cannot be greater than max",
                    ));
                }
            }
            match item.default_value.as_ref() {
                Some(SettingsValue::Number(value)) => {
                    if !value.is_finite() {
                        issues.push(ZsuiDeclarationIssue::error(
                            format!("{path}.default_value"),
                            "number settings item default must be finite",
                        ));
                    }
                    if let Some(min) = min {
                        if value < min {
                            issues.push(ZsuiDeclarationIssue::error(
                                format!("{path}.default_value"),
                                "number settings item default is below min",
                            ));
                        }
                    }
                    if let Some(max) = max {
                        if value > max {
                            issues.push(ZsuiDeclarationIssue::error(
                                format!("{path}.default_value"),
                                "number settings item default is above max",
                            ));
                        }
                    }
                }
                Some(_) => issues.push(ZsuiDeclarationIssue::error(
                    format!("{path}.default_value"),
                    "number settings item default must be a number",
                )),
                None => {}
            }
        }
        SettingsItemKind::Choice { options } => {
            if options.is_empty() {
                issues.push(ZsuiDeclarationIssue::error(
                    format!("{path}.kind.options"),
                    "choice settings item must have at least one option",
                ));
            }
            let mut seen = HashSet::new();
            for (index, option) in options.iter().enumerate() {
                if option.trim().is_empty() {
                    issues.push(ZsuiDeclarationIssue::error(
                        format!("{path}.kind.options[{index}]"),
                        "choice settings item option cannot be empty",
                    ));
                } else if !seen.insert(option.clone()) {
                    issues.push(ZsuiDeclarationIssue::warning(
                        format!("{path}.kind.options[{index}]"),
                        format!("duplicate choice option `{option}`"),
                    ));
                }
            }
            match item.default_value.as_ref() {
                Some(SettingsValue::Choice(value)) => {
                    if !options.iter().any(|option| option == value) {
                        issues.push(ZsuiDeclarationIssue::error(
                            format!("{path}.default_value"),
                            "choice settings item default must match one of its options",
                        ));
                    }
                }
                Some(_) => issues.push(ZsuiDeclarationIssue::error(
                    format!("{path}.default_value"),
                    "choice settings item default must be a choice",
                )),
                None => {}
            }
        }
        SettingsItemKind::Button => {
            if item.command.is_none() {
                issues.push(ZsuiDeclarationIssue::error(
                    format!("{path}.command"),
                    "button settings item must declare a command",
                ));
            }
            if item.default_value.is_some() {
                issues.push(ZsuiDeclarationIssue::error(
                    format!("{path}.default_value"),
                    "button settings item cannot have a default value",
                ));
            }
        }
    }
}

fn validate_default_value_kind<F>(
    value: Option<&SettingsValue>,
    path: &str,
    label: &str,
    accepts: F,
    issues: &mut Vec<ZsuiDeclarationIssue>,
) where
    F: FnOnce(&SettingsValue) -> bool,
{
    if let Some(value) = value {
        if !accepts(value) {
            issues.push(ZsuiDeclarationIssue::error(
                format!("{path}.default_value"),
                format!("{label} settings item default has the wrong value type"),
            ));
        }
    }
}

fn validate_number_bound(value: Option<f64>, path: &str, issues: &mut Vec<ZsuiDeclarationIssue>) {
    if let Some(value) = value {
        if !value.is_finite() {
            issues.push(ZsuiDeclarationIssue::error(
                path.to_string(),
                "number settings item bound must be finite",
            ));
        }
    }
}

fn validate_command(command: &Command, path: &str, issues: &mut Vec<ZsuiDeclarationIssue>) {
    if let Command::Custom { id, .. } = command {
        validate_required_id(id, &format!("{path}.id"), "custom command id", issues);
    }
}

fn validate_required_id(id: &str, path: &str, label: &str, issues: &mut Vec<ZsuiDeclarationIssue>) {
    if id.trim().is_empty() {
        issues.push(ZsuiDeclarationIssue::error(
            path.to_string(),
            format!("{label} cannot be empty"),
        ));
    }
}

fn validate_optional_id(id: &str, path: &str, label: &str, issues: &mut Vec<ZsuiDeclarationIssue>) {
    if id.trim().is_empty() {
        issues.push(ZsuiDeclarationIssue::error(
            path.to_string(),
            format!("{label} cannot be empty when provided"),
        ));
    }
}

fn validate_unique_id(
    id: &str,
    path: &str,
    label: &str,
    ids: &mut HashMap<String, String>,
    issues: &mut Vec<ZsuiDeclarationIssue>,
) {
    if id.trim().is_empty() {
        return;
    }
    if let Some(first_path) = ids.insert(id.to_string(), path.to_string()) {
        issues.push(ZsuiDeclarationIssue::error(
            path.to_string(),
            format!("duplicate {label} `{id}`; first declared at {first_path}"),
        ));
    }
}

fn normalize_hotkey_accelerator(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn used_degraded_capabilities_for_parts(
    windows: &[WindowSpec],
    tray_declared: bool,
    hotkeys: &[HotkeySpec],
    settings_pages: &[SettingsPageSpec],
    capabilities: &HostCapabilities,
) -> Vec<String> {
    let mut degraded = Vec::new();
    for (index, window) in windows.iter().enumerate() {
        for detail in window.degraded_capabilities(capabilities) {
            degraded.push(format!("window[{index}].{detail}"));
        }
    }
    if tray_declared && capabilities.tray_or_status_menu.status != CapabilityStatus::Supported {
        degraded.push(format!(
            "tray_or_status_menu: {}",
            capabilities.tray_or_status_menu.detail
        ));
    }
    if !hotkeys.is_empty() && capabilities.global_hotkeys.status != CapabilityStatus::Supported {
        degraded.push(format!(
            "global_hotkeys: {}",
            capabilities.global_hotkeys.detail
        ));
    }
    if !settings_pages.is_empty()
        && capabilities.settings_pages.status != CapabilityStatus::Supported
    {
        degraded.push(format!(
            "settings_pages: {}",
            capabilities.settings_pages.detail
        ));
    }
    degraded
}

fn used_degraded_capabilities(app: &ZsuiApp, capabilities: &HostCapabilities) -> Vec<String> {
    used_degraded_capabilities_for_parts(
        &app.windows,
        app.tray.is_some(),
        &app.hotkeys,
        &app.settings_pages,
        capabilities,
    )
}

#[cfg(test)]
mod declaration_tests {
    use super::*;
    #[cfg(all(feature = "label", feature = "button"))]
    use crate::window::Window;
    use crate::{
        capability::{CapabilitySupport, PlatformName},
        menu::MenuItemSpec,
        settings::{SettingsItemSpec, SettingsValue},
    };

    #[test]
    fn declaration_report_rejects_empty_menu_item_labels() {
        let err = app("Example")
            .tray(TraySpec::new().menu(MenuSpec::new().item("", Command::Quit)))
            .build()
            .expect_err("empty menu labels must be invalid");

        assert!(
            matches!(err, ZsuiError::InvalidSpec { field, .. } if field == "tray.menu.items[0].label")
        );
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn declaration_report_rejects_duplicate_ui_node_ids() {
        let content = UiNode::column("root")
            .child(UiNode::text("duplicate", "One"))
            .child(UiNode::button(
                "duplicate",
                "Two",
                Command::custom("example.two"),
            ));

        let err = app("Example")
            .window(Window::new("Example").content(content))
            .build()
            .expect_err("duplicate node ids must be invalid");

        assert!(
            matches!(err, ZsuiError::InvalidSpec { field, message } if field == "windows[0].content.children[1].id" && message.contains("duplicate ui node id"))
        );
    }

    #[test]
    fn declaration_report_rejects_duplicate_enabled_hotkeys() {
        let err = app("Example")
            .global_hotkey("Ctrl+Shift+P", Command::OpenQuickPanel)
            .global_hotkey("ctrl + shift + p", Command::OpenSettings)
            .build()
            .expect_err("duplicate enabled hotkeys must be invalid");

        assert!(
            matches!(err, ZsuiError::InvalidSpec { field, message } if field == "hotkeys[1].accelerator" && message.contains("duplicate enabled hotkey"))
        );
    }

    #[test]
    fn declaration_report_rejects_empty_window_icon_path() {
        let err = app("Example")
            .window(WindowSpec::new("Example").icon_path(" "))
            .build()
            .expect_err("empty window icon paths must be invalid");

        assert!(
            matches!(err, ZsuiError::InvalidSpec { field, message } if field == "windows[0].icon_path" && message.contains("window icon path"))
        );
    }

    #[test]
    fn declaration_report_validates_settings_shape() {
        let page = SettingsPageSpec::new("general", "General")
            .item(SettingsItemSpec {
                id: "theme".to_string(),
                label: "Theme".to_string(),
                kind: SettingsItemKind::Choice {
                    options: vec!["light".to_string(), "dark".to_string()],
                },
                description: None,
                default_value: Some(SettingsValue::Choice("system".to_string())),
                command: None,
            })
            .item(SettingsItemSpec::button(
                "theme",
                "Apply",
                Command::custom("settings.apply"),
            ));

        let report = app("Example").settings_page(page).declaration_report();

        assert!(!report.is_valid());
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.path == "settings_pages[0].items[0].default_value"));
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.path == "settings_pages[0].items[1].id"
                && issue.message.contains("duplicate settings item id")));
    }

    #[test]
    fn declaration_report_includes_host_degradation_warnings() {
        let mut capabilities = HostCapabilities::all_supported(PlatformName::Unknown);
        capabilities.global_hotkeys = CapabilitySupport::unsupported("hotkeys unavailable");

        let report = app("Example")
            .global_hotkey("Alt+Space", Command::OpenQuickPanel)
            .declaration_report_for(&capabilities);

        assert!(report.is_valid());
        assert_eq!(report.error_count, 0);
        assert_eq!(report.warning_count, 1);
        assert_eq!(
            report.degraded_capabilities,
            vec!["global_hotkeys: hotkeys unavailable"]
        );
    }

    #[test]
    fn declaration_report_validates_nested_menu_ids_and_commands() {
        let menu = MenuSpec::new()
            .item("Open", Command::ShowMainWindow)
            .submenu(
                "More",
                MenuSpec::new().item(
                    "Broken",
                    Command::Custom {
                        id: "".to_string(),
                        payload: None,
                    },
                ),
            )
            .submenu(
                "Again",
                MenuSpec::new()
                    .submenu("Inner", MenuSpec::new())
                    .item("Quit", Command::Quit),
            );
        let duplicate = MenuItemSpec::command("Duplicate", Command::Quit).id("dup");
        let menu = menu
            .submenu("With id", MenuSpec::new().item("A", Command::Quit))
            .item("End", Command::Quit)
            .separator();
        let tray = TraySpec::new().menu(MenuSpec {
            id: None,
            title: None,
            items: vec![
                duplicate.clone(),
                duplicate,
                MenuItemSpec::Submenu {
                    id: Some("submenu".to_string()),
                    label: "Nested".to_string(),
                    enabled: true,
                    menu,
                },
            ],
        });

        let report = app("Example").tray(tray).declaration_report();

        assert!(!report.is_valid());
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.message.contains("duplicate menu item id")));
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.message.contains("custom command id cannot be empty")));
    }
}
