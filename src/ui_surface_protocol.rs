#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiHostSurface {
    MainWindow,
    SettingsWindow,
    SettingsDropdown,
    InputDialog,
    EditDialog,
}

impl UiHostSurface {
    pub const fn adapter_name(self) -> &'static str {
        match self {
            Self::MainWindow => "main_window_host_event_from_message",
            Self::SettingsWindow => "settings_window_host_event_from_message",
            Self::SettingsDropdown => "dropdown_window_host_event_from_message",
            Self::InputDialog => "input_dialog_host_event_from_message",
            Self::EditDialog => "edit_dialog_host_event_from_message",
        }
    }
}

pub const REQUIRED_UI_HOST_SURFACES: [UiHostSurface; 5] = [
    UiHostSurface::MainWindow,
    UiHostSurface::SettingsWindow,
    UiHostSurface::SettingsDropdown,
    UiHostSurface::InputDialog,
    UiHostSurface::EditDialog,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_host_surfaces_keep_stable_adapter_names() {
        let names: Vec<_> = REQUIRED_UI_HOST_SURFACES
            .iter()
            .map(|surface| surface.adapter_name())
            .collect();

        assert_eq!(
            names,
            vec![
                "main_window_host_event_from_message",
                "settings_window_host_event_from_message",
                "dropdown_window_host_event_from_message",
                "input_dialog_host_event_from_message",
                "edit_dialog_host_event_from_message",
            ]
        );
    }
}
