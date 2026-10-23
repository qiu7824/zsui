//! Native Linux file chooser responses preserve cancellation and provider errors.
fn response_accepted(code: u32) -> Result<bool, String> {
    match code {
        0 => Ok(true),
        1 => Ok(false),
        2 => Err("The desktop file chooser failed to complete the request".into()),
        _ => Err(format!("Invalid desktop file chooser response {code}")),
    }
}

fn decode_local_file_uri(uri: &str) -> Result<Vec<u8>, String> {
    let tail = uri
        .strip_prefix("file://")
        .ok_or("The file chooser returned a non-file URI")?;
    let path = if tail.starts_with('/') {
        tail
    } else {
        tail.strip_prefix("localhost/")
            .map(|_| &tail["localhost".len()..])
            .ok_or("Remote file URIs are not supported")?
    };
    if path.contains(['?', '#']) {
        return Err("Unescaped query or fragment in file URI".into());
    }
    let mut bytes = Vec::with_capacity(path.len());
    let input = path.as_bytes();
    let mut at = 0;
    while at < input.len() {
        let byte = if input[at] == b'%' {
            let hex = input.get(at + 1..at + 3).ok_or("Incomplete URI escape")?;
            let nibble = |b: u8| (b as char).to_digit(16).ok_or("Invalid URI escape");
            at += 3;
            ((nibble(hex[0])? << 4) | nibble(hex[1])?) as u8
        } else {
            let b = input[at];
            at += 1;
            b
        };
        if byte == 0 {
            return Err("NUL in selected file path".into());
        }
        bytes.push(byte);
    }
    if bytes.first() != Some(&b'/') {
        return Err("Selected file path is not absolute".into());
    }
    Ok(bytes)
}

#[cfg(all(
    target_os = "linux",
    not(target_env = "ohos"),
    feature = "linux-direct-host"
))]
mod native {
    use super::*;
    use crate::{FileDialogFilter, ZsuiError, ZsuiResult};
    use dbus::{
        arg::{PropMap, RefArg, Variant},
        blocking::Connection,
        message::MatchRule,
    };
    use std::{
        collections::VecDeque,
        ffi::OsString,
        hash::{BuildHasher, Hasher},
        os::unix::ffi::{OsStrExt, OsStringExt},
        path::PathBuf,
        sync::{Arc, Mutex},
        time::Duration,
    };
    const PORTAL: &str = "org.freedesktop.portal.Desktop";
    const REQUEST: &str = "org.freedesktop.portal.Request";

    pub(crate) struct Picker {
        pub title: String,
        pub current_path: Option<PathBuf>,
        pub suggested_name: Option<String>,
        pub filters: Vec<FileDialogFilter>,
        pub multiple: bool,
        pub directory: bool,
        pub save: bool,
        pub parent_xid: Option<u64>,
    }
    enum Failure {
        Unavailable(String),
        Failed(String),
    }
    impl From<dbus::Error> for Failure {
        fn from(error: dbus::Error) -> Self {
            match error.name().unwrap_or("") {
                "org.freedesktop.DBus.Error.ServiceUnknown"
                | "org.freedesktop.DBus.Error.NameHasNoOwner"
                | "org.freedesktop.DBus.Error.UnknownMethod"
                | "org.freedesktop.DBus.Error.UnknownInterface" => {
                    Self::Unavailable(error.to_string())
                }
                _ => Self::Failed(error.to_string()),
            }
        }
    }
    impl Picker {
        pub fn run(self) -> ZsuiResult<Option<Vec<PathBuf>>> {
            if self.title.contains('\0')
                || self
                    .suggested_name
                    .as_ref()
                    .is_some_and(|s| s.contains('\0'))
                || self
                    .current_path
                    .as_ref()
                    .is_some_and(|p| p.as_os_str().as_bytes().contains(&0))
                || self
                    .filters
                    .iter()
                    .any(|f| f.name.contains('\0') || f.patterns.iter().any(|p| p.contains('\0')))
            {
                return Err(ZsuiError::invalid_spec(
                    "file_dialog",
                    "NUL in file chooser specification",
                ));
            }
            match self.portal() {
                Ok(result) => Ok(result),
                Err(Failure::Failed(message)) => Err(ZsuiError::host("linux_file_dialog", message)),
                Err(Failure::Unavailable(portal_error)) => self.zenity().map_err(|error| match error.kind() {
                    std::io::ErrorKind::NotFound => ZsuiError::unsupported("linux_file_dialog", format!("No desktop portal or Zenity file chooser is available: {portal_error}")),
                    _ => ZsuiError::host("linux_file_dialog", error.to_string()),
                })?,
            }
        }
        fn portal(&self) -> Result<Option<Vec<PathBuf>>, Failure> {
            let connection =
                Connection::new_session().map_err(|e| Failure::Unavailable(e.to_string()))?;
            let mut hash = std::collections::hash_map::RandomState::new().build_hasher();
            hash.write_u32(std::process::id());
            let token = format!("zsui_{:016x}", hash.finish());
            let responses = Arc::new(Mutex::new(VecDeque::new()));
            let received = responses.clone();
            let mut rule = MatchRule::new_signal(REQUEST, "Response");
            rule.sender = Some(PORTAL.into());
            // Subscribe before calling the portal; immediate responses must not be lost.
            connection.add_match(rule, move |(code, values): (u32, PropMap), _, message| {
                if let Some(path) = message.path() {
                    #[cfg(feature = "native-smoke")]
                    eprintln!("Native file chooser response: path={path}, code={code}");
                    let mut queue = received
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if queue.len() < 16 {
                        queue.push_back((path.to_string(), code, values));
                    }
                }
                true
            })?;
            let disappeared = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let lost = disappeared.clone();
            let mut owner_rule = MatchRule::new_signal("org.freedesktop.DBus", "NameOwnerChanged");
            owner_rule.sender = Some("org.freedesktop.DBus".into());
            connection.add_match(
                owner_rule,
                move |(name, old, new): (String, String, String), _, _| {
                    if name == PORTAL && !old.is_empty() && new != old {
                        lost.store(true, std::sync::atomic::Ordering::Release);
                    }
                    true
                },
            )?;
            let mut options: PropMap = PropMap::new();
            options.insert("handle_token".into(), Variant(Box::new(token)));
            options.insert("modal".into(), Variant(Box::new(true)));
            if !self.save {
                options.insert("multiple".into(), Variant(Box::new(self.multiple)));
                options.insert("directory".into(), Variant(Box::new(self.directory)));
            }
            if let Some(folder) = crate::native_file_dialog::native_file_dialog_initial_directory(
                self.current_path.as_deref(),
            ) {
                let mut bytes = folder.as_os_str().as_bytes().to_vec();
                bytes.push(0);
                options.insert("current_folder".into(), Variant(Box::new(bytes)));
            }
            if let Some(name) = &self.suggested_name {
                options.insert("current_name".into(), Variant(Box::new(name.clone())));
            }
            if !self.filters.is_empty() {
                let filters: Vec<(String, Vec<(u32, String)>)> = self
                    .filters
                    .iter()
                    .map(|f| {
                        (
                            f.name.clone(),
                            f.patterns.iter().map(|p| (0, p.clone())).collect(),
                        )
                    })
                    .collect();
                options.insert("filters".into(), Variant(Box::new(filters)));
            }
            let parent = self
                .parent_xid
                .map(|x| format!("x11:{x:x}"))
                .unwrap_or_default();
            let proxy = connection.with_proxy(
                PORTAL,
                "/org/freedesktop/portal/desktop",
                Duration::from_secs(30),
            );
            let (path,): (dbus::Path<'static>,) = proxy.method_call(
                "org.freedesktop.portal.FileChooser",
                if self.save { "SaveFile" } else { "OpenFile" },
                (parent, self.title.clone(), options),
            )?;
            #[cfg(feature = "native-smoke")]
            eprintln!("Native file chooser request: {path}");
            loop {
                if let Some((_, code, values)) = responses
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .pop_front()
                    .filter(|(p, _, _)| p == &path.to_string())
                {
                    if !response_accepted(code).map_err(Failure::Failed)? {
                        return Ok(None);
                    }
                    let uris = values
                        .get("uris")
                        .and_then(|v| v.0.as_iter())
                        .ok_or_else(|| {
                            Failure::Failed("The file chooser returned no URI array".into())
                        })?;
                    let mut paths = Vec::new();
                    for value in uris {
                        let uri = value
                            .as_str()
                            .ok_or_else(|| Failure::Failed("Invalid file URI value".into()))?;
                        let bytes = decode_local_file_uri(uri).map_err(Failure::Failed)?;
                        paths.push(PathBuf::from(OsString::from_vec(bytes)));
                    }
                    if paths.is_empty()
                        || ((!self.multiple || self.directory || self.save) && paths.len() != 1)
                    {
                        return Err(Failure::Failed(
                            "The file chooser returned an invalid selection count".into(),
                        ));
                    }
                    if self.directory && !paths[0].is_dir() {
                        return Err(Failure::Failed(
                            "The selected path is not a directory".into(),
                        ));
                    }
                    return Ok(Some(paths));
                }
                if disappeared.load(std::sync::atomic::Ordering::Acquire) {
                    return Err(Failure::Failed(
                        "The desktop file chooser provider disconnected".into(),
                    ));
                }
                connection.process(Duration::from_millis(100))?;
            }
        }
        fn zenity(&self) -> std::io::Result<ZsuiResult<Option<Vec<PathBuf>>>> {
            let mut command = std::process::Command::new("zenity");
            command
                .arg("--file-selection")
                .arg("--title")
                .arg(&self.title);
            if self.directory {
                command.arg("--directory");
            }
            if self.multiple {
                command.arg("--multiple").arg("--separator=\n");
            }
            if self.save {
                command.arg("--save").arg("--confirm-overwrite");
            }
            if let Some(id) = self.parent_xid {
                command.arg(format!("--attach={id}"));
            }
            if let Some(folder) = crate::native_file_dialog::native_file_dialog_initial_directory(
                self.current_path.as_deref(),
            ) {
                let name = self.suggested_name.as_deref().unwrap_or("");
                let path = if self.save {
                    folder.join(name)
                } else {
                    folder.join("")
                };
                let mut bytes = path.into_os_string().into_vec();
                if !self.save && !bytes.ends_with(b"/") {
                    bytes.push(b'/');
                }
                command.arg("--filename").arg(OsString::from_vec(bytes));
            } else if let Some(name) = &self.suggested_name {
                command.arg("--filename").arg(name);
            }
            for filter in &self.filters {
                command.arg("--file-filter").arg(format!(
                    "{} | {}",
                    filter.name,
                    filter.patterns.join(" ")
                ));
            }
            let result = command.output()?;
            match result.status.code() {
                Some(1) => Ok(Ok(None)),
                Some(0) => {
                    let raw = result.stdout.strip_suffix(b"\n").unwrap_or(&result.stdout);
                    let paths: Vec<_> = raw
                        .split(|b| self.multiple && *b == b'\n')
                        .filter(|p| !p.is_empty())
                        .map(|p| PathBuf::from(OsString::from_vec(p.to_vec())))
                        .collect();
                    if paths.is_empty()
                        || paths
                            .iter()
                            .any(|p| !p.is_absolute() || p.as_os_str().as_bytes().contains(&0))
                        || (self.directory && !paths[0].is_dir())
                    {
                        Ok(Err(ZsuiError::host(
                            "linux_file_dialog",
                            "Zenity returned an invalid selection",
                        )))
                    } else {
                        Ok(Ok(Some(paths)))
                    }
                }
                code => Ok(Err(ZsuiError::host(
                    "linux_file_dialog",
                    format!(
                        "Zenity failed ({code:?}): {}",
                        String::from_utf8_lossy(&result.stderr)
                            .chars()
                            .take(2048)
                            .collect::<String>()
                    ),
                ))),
            }
        }
    }
}
#[cfg(all(
    target_os = "linux",
    not(target_env = "ohos"),
    feature = "linux-direct-host"
))]
pub(crate) use native::Picker;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn linux_provider_cancellation_is_distinct_from_failure() {
        assert_eq!(response_accepted(0), Ok(true));
        assert_eq!(response_accepted(1), Ok(false));
        assert!(response_accepted(2).is_err());
        assert!(response_accepted(17).is_err());
    }
    #[test]
    fn linux_provider_uris_preserve_spaces_unicode_and_literal_percent() {
        assert_eq!(
            decode_local_file_uri("file:///tmp/%E4%B8%AD%E6%96%87%20a%25").unwrap(),
            "/tmp/中文 a%".as_bytes()
        );
        assert_eq!(
            decode_local_file_uri("file://localhost/tmp/a").unwrap(),
            b"/tmp/a"
        );
        assert_eq!(
            decode_local_file_uri("file:///tmp/%FF").unwrap(),
            b"/tmp/\xff"
        );
    }
    #[test]
    fn linux_provider_rejects_nonlocal_and_malformed_results() {
        for uri in [
            "https://x/a",
            "file://remote/tmp/a",
            "file:///tmp/%00",
            "file:///tmp/%",
            "file:///tmp/%XX",
            "file:///tmp/a#fragment",
        ] {
            assert!(decode_local_file_uri(uri).is_err(), "{uri}");
        }
    }
}
