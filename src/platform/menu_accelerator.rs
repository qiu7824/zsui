use crate::{ZsAccelerator, ZsAcceleratorKey};

#[cfg(any(
    test,
    all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk")
))]
pub(crate) fn gtk_accelerator(accelerator: &ZsAccelerator) -> String {
    let mut value = String::new();
    if accelerator.uses_primary() {
        value.push_str("<Primary>");
    }
    if accelerator.uses_super() {
        value.push_str("<Super>");
    }
    if accelerator.uses_alt() {
        value.push_str("<Alt>");
    }
    if accelerator.uses_shift() {
        value.push_str("<Shift>");
    }
    value.push_str(&gtk_key_name(accelerator.key()));
    value
}

#[cfg(any(
    test,
    all(target_os = "linux", not(target_env = "ohos"), feature = "linux-gtk")
))]
fn gtk_key_name(key: ZsAcceleratorKey) -> String {
    match key {
        ZsAcceleratorKey::Character(key) => key.to_ascii_uppercase().to_string(),
        ZsAcceleratorKey::Enter => "Return".to_string(),
        ZsAcceleratorKey::Escape => "Escape".to_string(),
        ZsAcceleratorKey::Tab => "Tab".to_string(),
        ZsAcceleratorKey::Space => "space".to_string(),
        ZsAcceleratorKey::Backspace => "BackSpace".to_string(),
        ZsAcceleratorKey::Delete => "Delete".to_string(),
        ZsAcceleratorKey::Up => "Up".to_string(),
        ZsAcceleratorKey::Down => "Down".to_string(),
        ZsAcceleratorKey::Left => "Left".to_string(),
        ZsAcceleratorKey::Right => "Right".to_string(),
        ZsAcceleratorKey::Home => "Home".to_string(),
        ZsAcceleratorKey::End => "End".to_string(),
        ZsAcceleratorKey::PageUp => "Page_Up".to_string(),
        ZsAcceleratorKey::PageDown => "Page_Down".to_string(),
        ZsAcceleratorKey::Function(number) => format!("F{number}"),
    }
}

#[cfg(any(test, all(target_os = "macos", feature = "macos-appkit")))]
pub(crate) fn appkit_key_equivalent(accelerator: &ZsAccelerator) -> Option<String> {
    let value = match accelerator.key() {
        ZsAcceleratorKey::Character(key) => key.to_ascii_lowercase().to_string(),
        ZsAcceleratorKey::Enter => "\r".to_string(),
        ZsAcceleratorKey::Escape => "\u{1b}".to_string(),
        ZsAcceleratorKey::Tab => "\t".to_string(),
        ZsAcceleratorKey::Space => " ".to_string(),
        ZsAcceleratorKey::Backspace => "\u{8}".to_string(),
        ZsAcceleratorKey::Delete => "\u{7f}".to_string(),
        ZsAcceleratorKey::Up => "\u{f700}".to_string(),
        ZsAcceleratorKey::Down => "\u{f701}".to_string(),
        ZsAcceleratorKey::Left => "\u{f702}".to_string(),
        ZsAcceleratorKey::Right => "\u{f703}".to_string(),
        ZsAcceleratorKey::Home => "\u{f729}".to_string(),
        ZsAcceleratorKey::End => "\u{f72b}".to_string(),
        ZsAcceleratorKey::PageUp => "\u{f72c}".to_string(),
        ZsAcceleratorKey::PageDown => "\u{f72d}".to_string(),
        ZsAcceleratorKey::Function(number) if (1..=24).contains(&number) => {
            char::from_u32(0xf703 + u32::from(number))?.to_string()
        }
        ZsAcceleratorKey::Function(_) => return None,
    };
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_encoders_project_one_semantic_accelerator() {
        let accelerator = ZsAccelerator::primary_character('o').with_alt().shifted();

        assert_eq!(gtk_accelerator(&accelerator), "<Primary><Alt><Shift>O");
        assert_eq!(appkit_key_equivalent(&accelerator).as_deref(), Some("o"));
        assert_eq!(
            appkit_key_equivalent(&ZsAccelerator::new(ZsAcceleratorKey::Function(25))),
            None
        );
    }
}
