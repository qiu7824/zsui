use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf};

pub type ZsuiResult<T> = Result<T, ZsuiError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsuiError {
    Unsupported { capability: String, reason: String },
    InvalidSpec { field: String, message: String },
    Host { operation: String, message: String },
}

impl ZsuiError {
    pub fn unsupported(capability: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Unsupported {
            capability: capability.into(),
            reason: reason.into(),
        }
    }

    pub fn invalid_spec(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidSpec {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn host(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Host {
            operation: operation.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ZsuiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported { capability, reason } => {
                write!(f, "unsupported zsui capability `{capability}`: {reason}")
            }
            Self::InvalidSpec { field, message } => {
                write!(f, "invalid zsui spec field `{field}`: {message}")
            }
            Self::Host { operation, message } => {
                write!(f, "zsui host operation `{operation}` failed: {message}")
            }
        }
    }
}

impl std::error::Error for ZsuiError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrayId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HotkeyId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    ShowMainWindow,
    HideMainWindow,
    ToggleMainWindow,
    OpenQuickPanel,
    OpenSettings,
    CopySelection,
    PasteSelection,
    ReadClipboard,
    WriteClipboard,
    Quit,
    Custom { id: String, payload: Option<String> },
}

impl Command {
    pub fn custom(id: impl Into<String>) -> Self {
        Self::Custom {
            id: id.into(),
            payload: None,
        }
    }

    pub fn custom_with_payload(id: impl Into<String>, payload: impl Into<String>) -> Self {
        Self::Custom {
            id: id.into(),
            payload: Some(payload.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppEvent {
    Started,
    WindowCreated { window: WindowId },
    WindowShown { window: WindowId },
    WindowHidden { window: WindowId },
    TrayCommand { command: Command },
    MenuCommand { command: Command },
    HotkeyPressed { hotkey: HotkeyId, command: Command },
    ClipboardChanged,
    SettingsChanged { page: String, item: String },
    DialogClosed { response: DialogResponse },
    QuitRequested,
    HostDegraded { capability: String, reason: String },
    Custom { id: String, payload: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileDialogFilter {
    pub name: String,
    pub patterns: Vec<String>,
}

impl FileDialogFilter {
    pub fn new(
        name: impl Into<String>,
        patterns: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            patterns: patterns.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileDialogSpec {
    pub title: String,
    pub current_path: Option<PathBuf>,
    pub filters: Vec<FileDialogFilter>,
    pub allow_multiple: bool,
}

impl FileDialogSpec {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            current_path: None,
            filters: Vec::new(),
            allow_multiple: false,
        }
    }

    pub fn current_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.current_path = Some(path.into());
        self
    }

    pub fn filter(
        mut self,
        name: impl Into<String>,
        patterns: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.filters.push(FileDialogFilter::new(name, patterns));
        self
    }

    pub fn allow_multiple(mut self, allow_multiple: bool) -> Self {
        self.allow_multiple = allow_multiple;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialogLevel {
    Info,
    Warning,
    Error,
    Question,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialogButtons {
    Ok,
    OkCancel,
    YesNo,
    YesNoCancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialogResponse {
    Ok,
    Cancel,
    Yes,
    No,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DialogButtonLabels {
    pub ok: String,
    pub cancel: String,
    pub yes: String,
    pub no: String,
}

impl DialogButtonLabels {
    pub fn new(
        ok: impl Into<String>,
        cancel: impl Into<String>,
        yes: impl Into<String>,
        no: impl Into<String>,
    ) -> Self {
        Self {
            ok: ok.into(),
            cancel: cancel.into(),
            yes: yes.into(),
            no: no.into(),
        }
    }

    pub fn label(&self, response: DialogResponse) -> &str {
        match response {
            DialogResponse::Ok => &self.ok,
            DialogResponse::Cancel => &self.cancel,
            DialogResponse::Yes => &self.yes,
            DialogResponse::No => &self.no,
        }
    }

    pub fn response_for_label(
        &self,
        buttons: DialogButtons,
        label: &str,
    ) -> Option<DialogResponse> {
        active_dialog_responses(buttons)
            .iter()
            .copied()
            .find(|response| self.label(*response) == label)
    }

    pub fn validate_for(&self, buttons: DialogButtons) -> ZsuiResult<()> {
        let responses = active_dialog_responses(buttons);
        for (index, response) in responses.iter().enumerate() {
            let label = self.label(*response);
            if label.trim().is_empty() {
                return Err(ZsuiError::invalid_spec(
                    format!("dialog.button_labels.{response:?}").to_ascii_lowercase(),
                    "button label must not be empty",
                ));
            }
            if label.contains('\0') {
                return Err(ZsuiError::invalid_spec(
                    format!("dialog.button_labels.{response:?}").to_ascii_lowercase(),
                    "button label must not contain a NUL character",
                ));
            }
            if responses[..index]
                .iter()
                .any(|other| self.label(*other) == label)
            {
                return Err(ZsuiError::invalid_spec(
                    "dialog.button_labels",
                    "visible dialog button labels must be unique",
                ));
            }
        }
        Ok(())
    }
}

impl Default for DialogButtonLabels {
    fn default() -> Self {
        Self::new("OK", "Cancel", "Yes", "No")
    }
}

fn active_dialog_responses(buttons: DialogButtons) -> &'static [DialogResponse] {
    match buttons {
        DialogButtons::Ok => &[DialogResponse::Ok],
        DialogButtons::OkCancel => &[DialogResponse::Ok, DialogResponse::Cancel],
        DialogButtons::YesNo => &[DialogResponse::Yes, DialogResponse::No],
        DialogButtons::YesNoCancel => &[
            DialogResponse::Yes,
            DialogResponse::No,
            DialogResponse::Cancel,
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NativeDialogSpec {
    pub title: String,
    pub message: String,
    pub level: DialogLevel,
    pub buttons: DialogButtons,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub button_labels: Option<DialogButtonLabels>,
}

impl NativeDialogSpec {
    pub fn message(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            level: DialogLevel::Info,
            buttons: DialogButtons::Ok,
            button_labels: None,
        }
    }

    pub fn level(mut self, level: DialogLevel) -> Self {
        self.level = level;
        self
    }

    pub fn buttons(mut self, buttons: DialogButtons) -> Self {
        self.buttons = buttons;
        self
    }

    pub fn button_labels(mut self, labels: DialogButtonLabels) -> Self {
        self.button_labels = Some(labels);
        self
    }

    pub fn resolved_button_labels(&self) -> DialogButtonLabels {
        self.button_labels.clone().unwrap_or_default()
    }

    pub fn validate(&self) -> ZsuiResult<()> {
        if self.title.contains('\0') {
            return Err(ZsuiError::invalid_spec(
                "dialog.title",
                "dialog title must not contain a NUL character",
            ));
        }
        if self.message.contains('\0') {
            return Err(ZsuiError::invalid_spec(
                "dialog.message",
                "dialog message must not contain a NUL character",
            ));
        }
        if let Some(labels) = &self.button_labels {
            labels.validate_for(self.buttons)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_dialog_custom_labels_keep_semantic_response_identity() {
        let labels = DialogButtonLabels::new("确定", "取消", "是", "否");
        labels
            .validate_for(DialogButtons::YesNoCancel)
            .expect("localized visible labels should be valid");
        assert_eq!(
            labels.response_for_label(DialogButtons::YesNoCancel, "否"),
            Some(DialogResponse::No)
        );
        assert_eq!(
            labels.response_for_label(DialogButtons::OkCancel, "是"),
            None
        );
    }

    #[test]
    fn native_dialog_rejects_ambiguous_or_native_unsafe_labels() {
        let duplicate = DialogButtonLabels::new("确定", "取消", "选择", "选择");
        assert!(duplicate.validate_for(DialogButtons::YesNo).is_err());

        let nul = NativeDialogSpec::message("Title", "Message")
            .button_labels(DialogButtonLabels::new("确\0定", "取消", "是", "否"));
        assert!(nul.validate().is_err());
    }

    #[test]
    fn native_dialog_label_override_is_backward_compatible_in_json() {
        let legacy = r#"{"title":"Title","message":"Message","level":"Info","buttons":"Ok"}"#;
        let decoded: NativeDialogSpec =
            serde_json::from_str(legacy).expect("legacy dialog JSON should decode");
        assert_eq!(decoded.button_labels, None);
        assert_eq!(
            decoded.resolved_button_labels(),
            DialogButtonLabels::default()
        );
    }
}
