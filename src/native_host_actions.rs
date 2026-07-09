use crate::{CommandId, CommandPayload, UiCommand};

pub mod command_ids {
    use super::CommandId;

    pub const TOGGLE_SEARCH: CommandId = CommandId("window.search.toggle");
    pub const UPDATE_SEARCH_TEXT: CommandId = CommandId("window.search.text.update");
    pub const INVOKE_MAIN_MENU_COMMAND: CommandId = CommandId("window.menu.invoke");
    pub const OPEN_SETTINGS: CommandId = CommandId("window.settings.open");
    pub const SAVE_SETTINGS: CommandId = CommandId("window.settings.save");
    pub const CLOSE_SETTINGS: CommandId = CommandId("window.settings.close");
    pub const OPEN_SETTINGS_CONFIG: CommandId = CommandId("window.settings.config.open");
    pub const OPEN_SETTINGS_DROPDOWN: CommandId = CommandId("window.settings.dropdown.open");
    pub const TOGGLE_SETTINGS_CONTROL: CommandId = CommandId("window.settings.control.toggle");
    pub const HIDE_WINDOW: CommandId = CommandId("window.hide");
    pub const CLOSE_WINDOW: CommandId = CommandId("window.close");
}

pub mod menu_ids {
    pub const TRAY_TOGGLE: usize = 40001;
    pub const TRAY_EXIT: usize = 40002;
    pub const TRAY_LAN_TOGGLE: usize = 40003;
    pub const TRAY_CAPTURE_TOGGLE: usize = 40004;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHostUiAction {
    ToggleSearch,
    OpenSettings,
    HideWindow,
    CloseWindow,
}

impl NativeHostUiAction {
    pub const fn action_name(self) -> &'static str {
        match self {
            Self::ToggleSearch => "toggle_search",
            Self::OpenSettings => "open_settings",
            Self::HideWindow => "hide_window",
            Self::CloseWindow => "close_window",
        }
    }

    pub const fn button_label(self) -> &'static str {
        match self {
            Self::ToggleSearch => "Search",
            Self::OpenSettings => "Settings",
            Self::HideWindow => "Hide",
            Self::CloseWindow => "Close",
        }
    }

    pub const fn command_id(self) -> CommandId {
        match self {
            Self::ToggleSearch => command_ids::TOGGLE_SEARCH,
            Self::OpenSettings => command_ids::OPEN_SETTINGS,
            Self::HideWindow => command_ids::HIDE_WINDOW,
            Self::CloseWindow => command_ids::CLOSE_WINDOW,
        }
    }

    pub fn command(self) -> UiCommand {
        UiCommand::window(self.command_id())
    }

    pub const fn opens_settings_surface(self) -> bool {
        matches!(self, Self::OpenSettings)
    }

    pub const fn toggles_search_surface(self) -> bool {
        matches!(self, Self::ToggleSearch)
    }

    pub const fn hides_main_window_surface(self) -> bool {
        matches!(self, Self::HideWindow)
    }

    pub const fn should_close_host(self) -> bool {
        matches!(self, Self::CloseWindow)
    }
}

pub const REQUIRED_NATIVE_HOST_UI_ACTIONS: [NativeHostUiAction; 4] = [
    NativeHostUiAction::ToggleSearch,
    NativeHostUiAction::OpenSettings,
    NativeHostUiAction::HideWindow,
    NativeHostUiAction::CloseWindow,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHostSearchControlAction {
    UpdateText,
}

impl NativeHostSearchControlAction {
    pub const fn action_name(self) -> &'static str {
        match self {
            Self::UpdateText => "search_text_changed",
        }
    }

    pub const fn placeholder(self) -> &'static str {
        match self {
            Self::UpdateText => "Search",
        }
    }
}

pub const REQUIRED_NATIVE_HOST_SEARCH_CONTROL_ACTIONS: [NativeHostSearchControlAction; 1] =
    [NativeHostSearchControlAction::UpdateText];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeHostSearchTextAction {
    pub text: String,
}

impl NativeHostSearchTextAction {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn command(&self) -> UiCommand {
        UiCommand::window_with_payload(
            command_ids::UPDATE_SEARCH_TEXT,
            CommandPayload::Text(self.text.clone()),
        )
    }

    pub fn normalized_text(&self) -> &str {
        self.text.trim()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHostStatusMenuAction {
    ToggleWindow,
    ToggleClipboardCapture,
    ToggleLanSync,
    Exit,
}

impl NativeHostStatusMenuAction {
    pub const fn action_name(self) -> &'static str {
        match self {
            Self::ToggleWindow => "status_toggle_window",
            Self::ToggleClipboardCapture => "status_toggle_clipboard_capture",
            Self::ToggleLanSync => "status_toggle_lan_sync",
            Self::Exit => "status_exit",
        }
    }

    pub const fn menu_label(self) -> &'static str {
        match self {
            Self::ToggleWindow => "Show Window",
            Self::ToggleClipboardCapture => "Toggle Capture",
            Self::ToggleLanSync => "Toggle LAN Sync",
            Self::Exit => "Exit",
        }
    }

    pub const fn tray_action(self) -> MainTrayMenuAction {
        match self {
            Self::ToggleWindow => MainTrayMenuAction::ToggleWindow,
            Self::ToggleClipboardCapture => MainTrayMenuAction::ToggleClipboardCapture,
            Self::ToggleLanSync => MainTrayMenuAction::ToggleLanSync,
            Self::Exit => MainTrayMenuAction::Exit,
        }
    }

    pub const fn menu_id(self) -> usize {
        self.tray_action().command_id()
    }

    pub fn command(self) -> UiCommand {
        main_menu_command_for_id(self.menu_id())
            .expect("native status menu action must map to tray menu command")
    }

    pub const fn should_exit_host(self) -> bool {
        matches!(self, Self::Exit)
    }

    pub const fn toggles_main_window_surface(self) -> bool {
        matches!(self, Self::ToggleWindow)
    }
}

pub const REQUIRED_NATIVE_HOST_STATUS_MENU_ACTIONS: [NativeHostStatusMenuAction; 4] = [
    NativeHostStatusMenuAction::ToggleWindow,
    NativeHostStatusMenuAction::ToggleClipboardCapture,
    NativeHostStatusMenuAction::ToggleLanSync,
    NativeHostStatusMenuAction::Exit,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusMenuEntry {
    Command {
        action: NativeHostStatusMenuAction,
        label: String,
        icon_name: String,
    },
    Separator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTrayMenuAction {
    ToggleWindow,
    ToggleClipboardCapture,
    ToggleLanSync,
    Exit,
}

impl MainTrayMenuAction {
    pub const fn command_id(self) -> usize {
        match self {
            Self::ToggleWindow => menu_ids::TRAY_TOGGLE,
            Self::ToggleClipboardCapture => menu_ids::TRAY_CAPTURE_TOGGLE,
            Self::ToggleLanSync => menu_ids::TRAY_LAN_TOGGLE,
            Self::Exit => menu_ids::TRAY_EXIT,
        }
    }

    pub const fn status_menu_action(self) -> NativeHostStatusMenuAction {
        match self {
            Self::ToggleWindow => NativeHostStatusMenuAction::ToggleWindow,
            Self::ToggleClipboardCapture => NativeHostStatusMenuAction::ToggleClipboardCapture,
            Self::ToggleLanSync => NativeHostStatusMenuAction::ToggleLanSync,
            Self::Exit => NativeHostStatusMenuAction::Exit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTrayMenuText {
    ToggleWindow,
    EnableClipboardCapture,
    DisableClipboardCapture,
    LanSyncOn,
    LanSyncOff,
    Exit,
}

impl MainTrayMenuText {
    pub const fn label(self) -> &'static str {
        match self {
            Self::ToggleWindow => "Show Window",
            Self::EnableClipboardCapture => "Enable Capture",
            Self::DisableClipboardCapture => "Disable Capture",
            Self::LanSyncOn => "Disable LAN Sync",
            Self::LanSyncOff => "Enable LAN Sync",
            Self::Exit => "Exit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTrayMenuItem {
    Command {
        action: MainTrayMenuAction,
        text: MainTrayMenuText,
    },
    Separator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainTrayMenuInput {
    pub clipboard_capture_enabled: bool,
    pub lan_sync_enabled: bool,
}

pub fn main_tray_menu_plan(input: MainTrayMenuInput) -> Vec<MainTrayMenuItem> {
    vec![
        MainTrayMenuItem::Command {
            action: MainTrayMenuAction::ToggleWindow,
            text: MainTrayMenuText::ToggleWindow,
        },
        MainTrayMenuItem::Command {
            action: MainTrayMenuAction::ToggleClipboardCapture,
            text: if input.clipboard_capture_enabled {
                MainTrayMenuText::DisableClipboardCapture
            } else {
                MainTrayMenuText::EnableClipboardCapture
            },
        },
        MainTrayMenuItem::Command {
            action: MainTrayMenuAction::ToggleLanSync,
            text: if input.lan_sync_enabled {
                MainTrayMenuText::LanSyncOn
            } else {
                MainTrayMenuText::LanSyncOff
            },
        },
        MainTrayMenuItem::Separator,
        MainTrayMenuItem::Command {
            action: MainTrayMenuAction::Exit,
            text: MainTrayMenuText::Exit,
        },
    ]
}

pub fn native_host_status_menu_entries(input: MainTrayMenuInput) -> Vec<StatusMenuEntry> {
    main_tray_menu_plan(input)
        .into_iter()
        .map(|item| match item {
            MainTrayMenuItem::Command { action, text } => StatusMenuEntry::Command {
                action: action.status_menu_action(),
                label: text.label().to_string(),
                icon_name: native_status_menu_action_icon_name(action.status_menu_action())
                    .to_string(),
            },
            MainTrayMenuItem::Separator => StatusMenuEntry::Separator,
        })
        .collect()
}

pub const fn native_status_menu_action_icon_name(
    action: NativeHostStatusMenuAction,
) -> &'static str {
    match action {
        NativeHostStatusMenuAction::ToggleWindow => "window-new-symbolic",
        NativeHostStatusMenuAction::ToggleClipboardCapture => "media-record-symbolic",
        NativeHostStatusMenuAction::ToggleLanSync => "network-wireless-symbolic",
        NativeHostStatusMenuAction::Exit => "application-exit-symbolic",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainTrayActionInput {
    pub action: MainTrayMenuAction,
    pub clipboard_capture_enabled: bool,
    pub lan_sync_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTrayActionPlan {
    ToggleWindow,
    SetClipboardCapture { enabled: bool },
    SetLanSync { enabled: bool },
    Exit,
}

pub fn main_tray_action_plan(input: MainTrayActionInput) -> MainTrayActionPlan {
    match input.action {
        MainTrayMenuAction::ToggleWindow => MainTrayActionPlan::ToggleWindow,
        MainTrayMenuAction::ToggleClipboardCapture => MainTrayActionPlan::SetClipboardCapture {
            enabled: !input.clipboard_capture_enabled,
        },
        MainTrayMenuAction::ToggleLanSync => MainTrayActionPlan::SetLanSync {
            enabled: !input.lan_sync_enabled,
        },
        MainTrayMenuAction::Exit => MainTrayActionPlan::Exit,
    }
}

pub fn main_menu_command_for_id(id: usize) -> Option<UiCommand> {
    match id {
        menu_ids::TRAY_TOGGLE
        | menu_ids::TRAY_LAN_TOGGLE
        | menu_ids::TRAY_CAPTURE_TOGGLE
        | menu_ids::TRAY_EXIT => Some(UiCommand::window_with_payload(
            command_ids::INVOKE_MAIN_MENU_COMMAND,
            CommandPayload::ControlId(id as i64),
        )),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsControlRole {
    Save,
    Close,
    OpenConfig,
    Dropdown,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHostSettingsAction {
    Save,
    Close,
    OpenConfig,
}

impl NativeHostSettingsAction {
    pub const fn action_name(self) -> &'static str {
        match self {
            Self::Save => "save_settings",
            Self::Close => "close_settings",
            Self::OpenConfig => "open_settings_config",
        }
    }

    pub const fn button_label(self) -> &'static str {
        match self {
            Self::Save => "Save",
            Self::Close => "Close",
            Self::OpenConfig => "Open Config",
        }
    }

    pub const fn command_id(self) -> CommandId {
        match self {
            Self::Save => command_ids::SAVE_SETTINGS,
            Self::Close => command_ids::CLOSE_SETTINGS,
            Self::OpenConfig => command_ids::OPEN_SETTINGS_CONFIG,
        }
    }

    pub fn command(self) -> UiCommand {
        UiCommand::window(self.command_id())
    }

    pub const fn should_close_settings_surface(self) -> bool {
        matches!(self, Self::Close)
    }
}

pub const REQUIRED_NATIVE_HOST_SETTINGS_ACTIONS: [NativeHostSettingsAction; 3] = [
    NativeHostSettingsAction::Save,
    NativeHostSettingsAction::Close,
    NativeHostSettingsAction::OpenConfig,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHostSettingsControlAction {
    ToggleAutostart,
    ToggleClipboardCapture,
    ToggleLanSync,
    ToggleCloudSync,
    OpenSyncModeDropdown,
}

impl NativeHostSettingsControlAction {
    pub const fn action_name(self) -> &'static str {
        match self {
            Self::ToggleAutostart => "settings_toggle_autostart",
            Self::ToggleClipboardCapture => "settings_toggle_clipboard_capture",
            Self::ToggleLanSync => "settings_toggle_lan_sync",
            Self::ToggleCloudSync => "settings_toggle_cloud_sync",
            Self::OpenSyncModeDropdown => "settings_open_sync_mode_dropdown",
        }
    }

    pub const fn button_label(self) -> &'static str {
        match self {
            Self::ToggleAutostart => "Auto Start",
            Self::ToggleClipboardCapture => "Capture",
            Self::ToggleLanSync => "LAN Sync",
            Self::ToggleCloudSync => "Cloud Sync",
            Self::OpenSyncModeDropdown => "Sync Mode",
        }
    }

    pub const fn control_id(self) -> i64 {
        match self {
            Self::ToggleAutostart => 5_010,
            Self::ToggleClipboardCapture => 5_101,
            Self::ToggleLanSync => 7_102,
            Self::ToggleCloudSync => 7_103,
            Self::OpenSyncModeDropdown => 6_102,
        }
    }

    pub const fn role(self) -> SettingsControlRole {
        match self {
            Self::OpenSyncModeDropdown => SettingsControlRole::Dropdown,
            Self::ToggleAutostart
            | Self::ToggleClipboardCapture
            | Self::ToggleLanSync
            | Self::ToggleCloudSync => SettingsControlRole::Toggle,
        }
    }

    pub const fn binding_control_key(self) -> Option<&'static str> {
        match self {
            Self::ToggleAutostart => Some("auto_start"),
            Self::ToggleClipboardCapture => Some("capture_enable"),
            Self::ToggleLanSync => Some("lan_enable"),
            Self::ToggleCloudSync => Some("cloud_enable"),
            Self::OpenSyncModeDropdown => Some("multi_sync_mode"),
        }
    }

    pub fn command(self) -> UiCommand {
        settings_command_for_control_role(self.role(), self.control_id())
    }
}

pub const REQUIRED_NATIVE_HOST_SETTINGS_CONTROL_ACTIONS: [NativeHostSettingsControlAction; 5] = [
    NativeHostSettingsControlAction::ToggleAutostart,
    NativeHostSettingsControlAction::ToggleClipboardCapture,
    NativeHostSettingsControlAction::ToggleLanSync,
    NativeHostSettingsControlAction::ToggleCloudSync,
    NativeHostSettingsControlAction::OpenSyncModeDropdown,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHostSettingsPlatformAction {
    OpenSourceRepository,
    CheckForUpdates,
    OpenDocs,
    DisableSystemClipboardHistory,
    EnableSystemClipboardHistory,
    RestartSystemShell,
}

impl NativeHostSettingsPlatformAction {
    pub const fn action_name(self) -> &'static str {
        match self {
            Self::OpenSourceRepository => "settings_open_source_repository",
            Self::CheckForUpdates => "settings_check_for_updates",
            Self::OpenDocs => "settings_open_docs",
            Self::DisableSystemClipboardHistory => "settings_disable_system_clipboard_history",
            Self::EnableSystemClipboardHistory => "settings_enable_system_clipboard_history",
            Self::RestartSystemShell => "settings_restart_system_shell",
        }
    }

    pub const fn button_label(self) -> &'static str {
        match self {
            Self::OpenSourceRepository => "Open Source",
            Self::CheckForUpdates => "Check Updates",
            Self::OpenDocs => "Open Docs",
            Self::DisableSystemClipboardHistory => "Disable Clipboard History",
            Self::EnableSystemClipboardHistory => "Enable Clipboard History",
            Self::RestartSystemShell => "Restart Shell",
        }
    }

    pub const fn settings_action(self) -> SettingsAction {
        match self {
            Self::OpenSourceRepository => SettingsAction::OpenSourceRepository,
            Self::CheckForUpdates => SettingsAction::CheckForUpdates,
            Self::OpenDocs => SettingsAction::OpenDocs,
            Self::DisableSystemClipboardHistory => SettingsAction::DisableSystemClipboardHistory,
            Self::EnableSystemClipboardHistory => SettingsAction::EnableSystemClipboardHistory,
            Self::RestartSystemShell => SettingsAction::RestartSystemShell,
        }
    }
}

pub const REQUIRED_NATIVE_HOST_SETTINGS_PLATFORM_ACTIONS: [NativeHostSettingsPlatformAction; 6] = [
    NativeHostSettingsPlatformAction::OpenSourceRepository,
    NativeHostSettingsPlatformAction::CheckForUpdates,
    NativeHostSettingsPlatformAction::OpenDocs,
    NativeHostSettingsPlatformAction::DisableSystemClipboardHistory,
    NativeHostSettingsPlatformAction::EnableSystemClipboardHistory,
    NativeHostSettingsPlatformAction::RestartSystemShell,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHostSettingsGroupAction {
    ShowRecords,
    ShowPhrases,
    Add,
    Rename,
    Delete,
    MoveUp,
    MoveDown,
}

impl NativeHostSettingsGroupAction {
    pub const fn action_name(self) -> &'static str {
        match self {
            Self::ShowRecords => "settings_group_show_records",
            Self::ShowPhrases => "settings_group_show_phrases",
            Self::Add => "settings_group_add",
            Self::Rename => "settings_group_rename",
            Self::Delete => "settings_group_delete",
            Self::MoveUp => "settings_group_move_up",
            Self::MoveDown => "settings_group_move_down",
        }
    }

    pub const fn button_label(self) -> &'static str {
        match self {
            Self::ShowRecords => "Records",
            Self::ShowPhrases => "Phrases",
            Self::Add => "Add",
            Self::Rename => "Rename",
            Self::Delete => "Delete",
            Self::MoveUp => "Up",
            Self::MoveDown => "Down",
        }
    }

    pub const fn target_category(self) -> Option<i64> {
        match self {
            Self::ShowRecords => Some(0),
            Self::ShowPhrases => Some(1),
            _ => None,
        }
    }

    pub const fn move_step(self) -> Option<i64> {
        match self {
            Self::MoveUp => Some(-1),
            Self::MoveDown => Some(1),
            _ => None,
        }
    }

    pub const fn settings_action(self) -> SettingsAction {
        match self {
            Self::ShowRecords => SettingsAction::ShowRecordGroups,
            Self::ShowPhrases => SettingsAction::ShowPhraseGroups,
            Self::Add => SettingsAction::AddGroup,
            Self::Rename => SettingsAction::RenameGroup,
            Self::Delete => SettingsAction::DeleteGroup,
            Self::MoveUp => SettingsAction::MoveGroupUp,
            Self::MoveDown => SettingsAction::MoveGroupDown,
        }
    }
}

pub const REQUIRED_NATIVE_HOST_SETTINGS_GROUP_ACTIONS: [NativeHostSettingsGroupAction; 7] = [
    NativeHostSettingsGroupAction::ShowRecords,
    NativeHostSettingsGroupAction::ShowPhrases,
    NativeHostSettingsGroupAction::Add,
    NativeHostSettingsGroupAction::Rename,
    NativeHostSettingsGroupAction::Delete,
    NativeHostSettingsGroupAction::MoveUp,
    NativeHostSettingsGroupAction::MoveDown,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsAction {
    ToggleHotkeyRecording,
    AddGroup,
    RenameGroup,
    DeleteGroup,
    MoveGroupUp,
    MoveGroupDown,
    GroupSelectionChanged,
    ShowRecordGroups,
    ShowPhraseGroups,
    PickPasteSound,
    CaptureSkippedWindowClass,
    RestoreSearchEnginePreset,
    DetectOcrRuntime,
    OpenMailMerge,
    OpenDocs,
    OpenSourceRepository,
    CheckForUpdates,
    DisableSystemClipboardHistory,
    EnableSystemClipboardHistory,
    RestartSystemShell,
    SyncWebDavNow,
    UploadWebDavConfig,
    ApplyWebDavConfig,
    RestoreWebDavBackup,
    RefreshLanDevices,
    PairLanDevice,
    AcceptLanPairing,
    RejectLanPairing,
    CopyLanPairUrl,
    CopyLanSetupUrl,
    OpenLanSetupPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsActionRoute {
    Sync,
    Group,
    Platform,
}

pub const fn settings_action_route(action: SettingsAction) -> SettingsActionRoute {
    match action {
        SettingsAction::SyncWebDavNow
        | SettingsAction::UploadWebDavConfig
        | SettingsAction::ApplyWebDavConfig
        | SettingsAction::RestoreWebDavBackup
        | SettingsAction::RefreshLanDevices
        | SettingsAction::PairLanDevice
        | SettingsAction::AcceptLanPairing
        | SettingsAction::RejectLanPairing
        | SettingsAction::CopyLanPairUrl
        | SettingsAction::CopyLanSetupUrl
        | SettingsAction::OpenLanSetupPage => SettingsActionRoute::Sync,
        SettingsAction::AddGroup
        | SettingsAction::RenameGroup
        | SettingsAction::DeleteGroup
        | SettingsAction::MoveGroupUp
        | SettingsAction::MoveGroupDown
        | SettingsAction::GroupSelectionChanged
        | SettingsAction::ShowRecordGroups
        | SettingsAction::ShowPhraseGroups => SettingsActionRoute::Group,
        SettingsAction::ToggleHotkeyRecording
        | SettingsAction::PickPasteSound
        | SettingsAction::CaptureSkippedWindowClass
        | SettingsAction::RestoreSearchEnginePreset
        | SettingsAction::DetectOcrRuntime
        | SettingsAction::OpenMailMerge
        | SettingsAction::OpenDocs
        | SettingsAction::OpenSourceRepository
        | SettingsAction::CheckForUpdates
        | SettingsAction::DisableSystemClipboardHistory
        | SettingsAction::EnableSystemClipboardHistory
        | SettingsAction::RestartSystemShell => SettingsActionRoute::Platform,
    }
}

pub fn settings_action_for_route(route_name: &str, action_name: &str) -> Option<SettingsAction> {
    let action = match (route_name, action_name) {
        ("settings_sync", "sync_webdav_now") => SettingsAction::SyncWebDavNow,
        ("settings_sync", "upload_webdav_config") => SettingsAction::UploadWebDavConfig,
        ("settings_sync", "apply_webdav_config") => SettingsAction::ApplyWebDavConfig,
        ("settings_sync", "restore_webdav_backup") => SettingsAction::RestoreWebDavBackup,
        ("settings_sync", "refresh_lan_devices") => SettingsAction::RefreshLanDevices,
        ("settings_sync", "pair_lan_device") => SettingsAction::PairLanDevice,
        ("settings_sync", "accept_lan_pairing") => SettingsAction::AcceptLanPairing,
        ("settings_sync", "reject_lan_pairing") => SettingsAction::RejectLanPairing,
        ("settings_sync", "copy_lan_pair_url") => SettingsAction::CopyLanPairUrl,
        ("settings_sync", "copy_lan_setup_url") => SettingsAction::CopyLanSetupUrl,
        ("settings_sync", "open_lan_setup_page") => SettingsAction::OpenLanSetupPage,
        ("settings_group", "show_record_groups") => SettingsAction::ShowRecordGroups,
        ("settings_group", "show_phrase_groups") => SettingsAction::ShowPhraseGroups,
        ("settings_group", "add_group") => SettingsAction::AddGroup,
        ("settings_group", "rename_group") => SettingsAction::RenameGroup,
        ("settings_group", "delete_group") => SettingsAction::DeleteGroup,
        ("settings_group", "move_group_up") => SettingsAction::MoveGroupUp,
        ("settings_group", "move_group_down") => SettingsAction::MoveGroupDown,
        ("settings_platform", "toggle_hotkey_recording") => SettingsAction::ToggleHotkeyRecording,
        ("settings_platform", "pick_paste_sound") => SettingsAction::PickPasteSound,
        ("settings_platform", "capture_skipped_window_class") => {
            SettingsAction::CaptureSkippedWindowClass
        }
        ("settings_platform", "restore_search_engine_preset") => {
            SettingsAction::RestoreSearchEnginePreset
        }
        ("settings_platform", "detect_ocr_runtime") => SettingsAction::DetectOcrRuntime,
        ("settings_platform", "open_mail_merge") => SettingsAction::OpenMailMerge,
        ("settings_platform", "open_docs") => SettingsAction::OpenDocs,
        ("settings_platform", "open_source_repository") => SettingsAction::OpenSourceRepository,
        ("settings_platform", "check_for_updates") => SettingsAction::CheckForUpdates,
        ("settings_platform", "disable_system_clipboard_history") => {
            SettingsAction::DisableSystemClipboardHistory
        }
        ("settings_platform", "enable_system_clipboard_history") => {
            SettingsAction::EnableSystemClipboardHistory
        }
        ("settings_platform", "restart_system_shell") => SettingsAction::RestartSystemShell,
        _ => return None,
    };
    Some(action)
}

pub trait SettingsActionExecutor {
    type Context;

    fn execute_sync(&mut self, context: &mut Self::Context, action: SettingsAction) -> bool;

    fn execute_group(&mut self, context: &mut Self::Context, action: SettingsAction) -> bool;

    fn execute_platform(&mut self, context: &mut Self::Context, action: SettingsAction) -> bool;
}

pub fn dispatch_settings_action<E: SettingsActionExecutor>(
    executor: &mut E,
    context: &mut E::Context,
    action: SettingsAction,
) -> bool {
    match settings_action_route(action) {
        SettingsActionRoute::Sync => executor.execute_sync(context, action),
        SettingsActionRoute::Group => executor.execute_group(context, action),
        SettingsActionRoute::Platform => executor.execute_platform(context, action),
    }
}

pub fn settings_command_id_for_role(role: SettingsControlRole) -> CommandId {
    match role {
        SettingsControlRole::Save => command_ids::SAVE_SETTINGS,
        SettingsControlRole::Close => command_ids::CLOSE_SETTINGS,
        SettingsControlRole::OpenConfig => command_ids::OPEN_SETTINGS_CONFIG,
        SettingsControlRole::Dropdown => command_ids::OPEN_SETTINGS_DROPDOWN,
        SettingsControlRole::Toggle => command_ids::TOGGLE_SETTINGS_CONTROL,
    }
}

pub fn settings_command_for_control_role(role: SettingsControlRole, control_id: i64) -> UiCommand {
    let id = settings_command_id_for_role(role);
    match role {
        SettingsControlRole::Dropdown | SettingsControlRole::Toggle => {
            UiCommand::window_with_payload(id, CommandPayload::ControlId(control_id))
        }
        SettingsControlRole::Save
        | SettingsControlRole::Close
        | SettingsControlRole::OpenConfig => UiCommand::window(id),
    }
}

pub fn required_native_host_status_menu_action_names() -> Vec<&'static str> {
    REQUIRED_NATIVE_HOST_STATUS_MENU_ACTIONS
        .iter()
        .map(|action| action.action_name())
        .collect()
}

pub fn required_native_host_settings_action_names() -> Vec<&'static str> {
    REQUIRED_NATIVE_HOST_SETTINGS_ACTIONS
        .iter()
        .map(|action| action.action_name())
        .collect()
}

pub fn required_native_host_settings_control_action_names() -> Vec<&'static str> {
    REQUIRED_NATIVE_HOST_SETTINGS_CONTROL_ACTIONS
        .iter()
        .map(|action| action.action_name())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CommandScope, UiCommand};

    #[test]
    fn native_host_status_menu_actions_map_to_tray_commands() {
        let cases = [
            (
                NativeHostStatusMenuAction::ToggleWindow,
                "status_toggle_window",
                "Show Window",
                MainTrayMenuAction::ToggleWindow,
                menu_ids::TRAY_TOGGLE,
                false,
            ),
            (
                NativeHostStatusMenuAction::ToggleClipboardCapture,
                "status_toggle_clipboard_capture",
                "Toggle Capture",
                MainTrayMenuAction::ToggleClipboardCapture,
                menu_ids::TRAY_CAPTURE_TOGGLE,
                false,
            ),
            (
                NativeHostStatusMenuAction::ToggleLanSync,
                "status_toggle_lan_sync",
                "Toggle LAN Sync",
                MainTrayMenuAction::ToggleLanSync,
                menu_ids::TRAY_LAN_TOGGLE,
                false,
            ),
            (
                NativeHostStatusMenuAction::Exit,
                "status_exit",
                "Exit",
                MainTrayMenuAction::Exit,
                menu_ids::TRAY_EXIT,
                true,
            ),
        ];

        for (action, action_name, menu_label, tray_action, menu_id, should_exit) in cases {
            let command = action.command();
            assert_eq!(action.action_name(), action_name);
            assert_eq!(action.menu_label(), menu_label);
            assert_eq!(action.tray_action(), tray_action);
            assert_eq!(action.menu_id(), menu_id);
            assert_eq!(action.should_exit_host(), should_exit);
            assert_eq!(command.id, command_ids::INVOKE_MAIN_MENU_COMMAND);
            assert_eq!(command.scope, CommandScope::Window);
            assert_eq!(command.payload, CommandPayload::ControlId(menu_id as i64));
        }
    }

    #[test]
    fn tray_menu_plan_describes_status_items_without_host_menu() {
        assert_eq!(
            main_tray_menu_plan(MainTrayMenuInput {
                clipboard_capture_enabled: true,
                lan_sync_enabled: false,
            }),
            vec![
                MainTrayMenuItem::Command {
                    action: MainTrayMenuAction::ToggleWindow,
                    text: MainTrayMenuText::ToggleWindow,
                },
                MainTrayMenuItem::Command {
                    action: MainTrayMenuAction::ToggleClipboardCapture,
                    text: MainTrayMenuText::DisableClipboardCapture,
                },
                MainTrayMenuItem::Command {
                    action: MainTrayMenuAction::ToggleLanSync,
                    text: MainTrayMenuText::LanSyncOff,
                },
                MainTrayMenuItem::Separator,
                MainTrayMenuItem::Command {
                    action: MainTrayMenuAction::Exit,
                    text: MainTrayMenuText::Exit,
                },
            ]
        );

        assert_eq!(
            main_tray_action_plan(MainTrayActionInput {
                action: MainTrayMenuAction::ToggleClipboardCapture,
                clipboard_capture_enabled: true,
                lan_sync_enabled: false,
            }),
            MainTrayActionPlan::SetClipboardCapture { enabled: false }
        );
    }

    #[test]
    fn native_host_settings_control_actions_map_to_settings_commands() {
        let cases = [
            (
                NativeHostSettingsControlAction::ToggleAutostart,
                "settings_toggle_autostart",
                "Auto Start",
                SettingsControlRole::Toggle,
                command_ids::TOGGLE_SETTINGS_CONTROL,
                5_010,
                Some("auto_start"),
            ),
            (
                NativeHostSettingsControlAction::ToggleClipboardCapture,
                "settings_toggle_clipboard_capture",
                "Capture",
                SettingsControlRole::Toggle,
                command_ids::TOGGLE_SETTINGS_CONTROL,
                5_101,
                Some("capture_enable"),
            ),
            (
                NativeHostSettingsControlAction::ToggleLanSync,
                "settings_toggle_lan_sync",
                "LAN Sync",
                SettingsControlRole::Toggle,
                command_ids::TOGGLE_SETTINGS_CONTROL,
                7_102,
                Some("lan_enable"),
            ),
            (
                NativeHostSettingsControlAction::ToggleCloudSync,
                "settings_toggle_cloud_sync",
                "Cloud Sync",
                SettingsControlRole::Toggle,
                command_ids::TOGGLE_SETTINGS_CONTROL,
                7_103,
                Some("cloud_enable"),
            ),
            (
                NativeHostSettingsControlAction::OpenSyncModeDropdown,
                "settings_open_sync_mode_dropdown",
                "Sync Mode",
                SettingsControlRole::Dropdown,
                command_ids::OPEN_SETTINGS_DROPDOWN,
                6_102,
                Some("multi_sync_mode"),
            ),
        ];

        for (action, action_name, button_label, role, command_id, control_id, binding_key) in cases
        {
            let command = action.command();
            assert_eq!(action.action_name(), action_name);
            assert_eq!(action.button_label(), button_label);
            assert_eq!(action.role(), role);
            assert_eq!(action.control_id(), control_id);
            assert_eq!(action.binding_control_key(), binding_key);
            assert_eq!(command.id, command_id);
            assert_eq!(command.scope, CommandScope::Window);
            assert_eq!(command.payload, CommandPayload::ControlId(control_id));
        }
    }

    #[test]
    fn settings_control_roles_map_to_platform_neutral_commands() {
        assert_eq!(
            settings_command_id_for_role(SettingsControlRole::Save),
            command_ids::SAVE_SETTINGS
        );
        assert_eq!(
            settings_command_id_for_role(SettingsControlRole::Close),
            command_ids::CLOSE_SETTINGS
        );
        assert_eq!(
            settings_command_for_control_role(SettingsControlRole::OpenConfig, 99),
            UiCommand::window(command_ids::OPEN_SETTINGS_CONFIG)
        );
        assert_eq!(
            settings_command_for_control_role(SettingsControlRole::Dropdown, 6102),
            UiCommand::window_with_payload(
                command_ids::OPEN_SETTINGS_DROPDOWN,
                CommandPayload::ControlId(6102)
            )
        );
        assert_eq!(
            settings_command_for_control_role(SettingsControlRole::Toggle, 7101),
            UiCommand::window_with_payload(
                command_ids::TOGGLE_SETTINGS_CONTROL,
                CommandPayload::ControlId(7101)
            )
        );
    }

    #[test]
    fn settings_actions_dispatch_to_platform_neutral_executor_domains() {
        #[derive(Default)]
        struct FakeExecutor {
            routes: Vec<(SettingsActionRoute, SettingsAction)>,
        }

        impl SettingsActionExecutor for FakeExecutor {
            type Context = usize;

            fn execute_sync(
                &mut self,
                context: &mut Self::Context,
                action: SettingsAction,
            ) -> bool {
                *context += 1;
                self.routes.push((SettingsActionRoute::Sync, action));
                true
            }

            fn execute_group(
                &mut self,
                context: &mut Self::Context,
                action: SettingsAction,
            ) -> bool {
                *context += 10;
                self.routes.push((SettingsActionRoute::Group, action));
                true
            }

            fn execute_platform(
                &mut self,
                context: &mut Self::Context,
                action: SettingsAction,
            ) -> bool {
                *context += 100;
                self.routes.push((SettingsActionRoute::Platform, action));
                true
            }
        }

        let mut executor = FakeExecutor::default();
        let mut context = 0;
        assert!(dispatch_settings_action(
            &mut executor,
            &mut context,
            SettingsAction::SyncWebDavNow
        ));
        assert!(dispatch_settings_action(
            &mut executor,
            &mut context,
            SettingsAction::AddGroup
        ));
        assert!(dispatch_settings_action(
            &mut executor,
            &mut context,
            SettingsAction::OpenSourceRepository
        ));

        assert_eq!(context, 111);
        assert_eq!(
            executor.routes,
            vec![
                (SettingsActionRoute::Sync, SettingsAction::SyncWebDavNow),
                (SettingsActionRoute::Group, SettingsAction::AddGroup),
                (
                    SettingsActionRoute::Platform,
                    SettingsAction::OpenSourceRepository
                ),
            ]
        );
        assert_eq!(
            settings_action_for_route("settings_platform", "restart_system_shell"),
            Some(SettingsAction::RestartSystemShell)
        );
    }
}
