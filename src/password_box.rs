use std::fmt;

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::{
    ColorRole, Dp, Dpi, NativeDrawCommand, NativeDrawFill, NativeDrawIconCommand, NativeDrawPlan,
    NativeDrawSecureTextCommand, NativeDrawTextCommand, NativeIconColorMode, Rect,
    SemanticTextStyle, ZsIcon,
};

/// An owned password whose allocation is cleared when it is dropped.
///
/// `Debug` is always redacted and the type intentionally does not implement
/// `Serialize` or `Deserialize`. Calling [`ZsPassword::as_str`] is the explicit
/// boundary for handing the secret to authentication code.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ZsPassword(Zeroizing<String>);

impl ZsPassword {
    pub fn new(value: impl Into<String>) -> Self {
        Self(Zeroizing::new(value.into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn char_count(&self) -> usize {
        self.0.chars().count()
    }

    pub(crate) fn as_string_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

impl fmt::Debug for ZsPassword {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ZsPassword(<redacted>)")
    }
}

impl From<String> for ZsPassword {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ZsPassword {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<&ZsPassword> for ZsPassword {
    fn from(value: &ZsPassword) -> Self {
        value.clone()
    }
}

pub fn mask_password(value: &str) -> String {
    "•".repeat(value.chars().count())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsPasswordRevealMode {
    Hidden,
    Peek,
    Visible,
}

impl ZsPasswordRevealMode {
    pub const fn platform_default() -> Self {
        if cfg!(target_os = "windows") {
            Self::Peek
        } else {
            Self::Hidden
        }
    }
}

impl Default for ZsPasswordRevealMode {
    fn default() -> Self {
        Self::platform_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsPasswordBoxPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

impl ZsPasswordBoxPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(all(target_os = "linux", not(target_env = "ohos"))) {
            Self::Gtk
        } else {
            Self::Windows
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsPasswordBoxMetrics {
    pub minimum_height: Dp,
    pub radius: Dp,
    pub text_inset: Dp,
    pub reveal_width: Dp,
    pub reveal_icon_size: Dp,
}

impl ZsPasswordBoxMetrics {
    pub const fn for_platform(platform: ZsPasswordBoxPlatformStyle) -> Self {
        match platform {
            ZsPasswordBoxPlatformStyle::Windows => Self {
                minimum_height: Dp::new(32.0),
                radius: Dp::new(4.0),
                text_inset: Dp::new(8.0),
                reveal_width: Dp::new(32.0),
                reveal_icon_size: Dp::new(16.0),
            },
            ZsPasswordBoxPlatformStyle::Macos => Self {
                minimum_height: Dp::new(28.0),
                radius: Dp::new(5.0),
                text_inset: Dp::new(7.0),
                reveal_width: Dp::new(28.0),
                reveal_icon_size: Dp::new(15.0),
            },
            ZsPasswordBoxPlatformStyle::Gtk => Self {
                minimum_height: Dp::new(34.0),
                radius: Dp::new(5.0),
                text_inset: Dp::new(8.0),
                reveal_width: Dp::new(34.0),
                reveal_icon_size: Dp::new(16.0),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsPasswordBoxRenderPlan {
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub reveal_button: Option<Rect>,
    pub reveal_icon: Option<Rect>,
    pub radius: i32,
    pub platform: ZsPasswordBoxPlatformStyle,
}

pub fn zs_password_box_render_plan(
    bounds: Rect,
    reveal_mode: ZsPasswordRevealMode,
    has_value: bool,
    platform: ZsPasswordBoxPlatformStyle,
    dpi: Dpi,
) -> ZsPasswordBoxRenderPlan {
    let metrics = ZsPasswordBoxMetrics::for_platform(platform);
    let inset = metrics.text_inset.to_px(dpi).round_i32().max(0);
    let reveal_width = metrics
        .reveal_width
        .to_px(dpi)
        .round_i32()
        .min(bounds.width.max(0));
    let show_reveal = reveal_mode == ZsPasswordRevealMode::Peek && has_value && reveal_width > 0;
    let reveal_button = show_reveal.then_some(Rect {
        x: bounds
            .x
            .saturating_add(bounds.width.saturating_sub(reveal_width)),
        y: bounds.y,
        width: reveal_width,
        height: bounds.height,
    });
    let reveal_icon = reveal_button.map(|button| {
        let size = metrics
            .reveal_icon_size
            .to_px(dpi)
            .round_i32()
            .min(button.width)
            .min(button.height)
            .max(1);
        Rect {
            x: button.x + (button.width - size) / 2,
            y: button.y + (button.height - size) / 2,
            width: size,
            height: size,
        }
    });
    let trailing = reveal_button.map_or(0, |button| button.width);
    ZsPasswordBoxRenderPlan {
        bounds,
        text_bounds: Rect {
            x: bounds.x.saturating_add(inset),
            y: bounds.y,
            width: bounds
                .width
                .saturating_sub(trailing)
                .saturating_sub(inset.saturating_mul(2))
                .max(0),
            height: bounds.height,
        },
        reveal_button,
        reveal_icon,
        radius: metrics.radius.to_px(dpi).round_i32().max(1),
        platform,
    }
}

pub fn zs_password_box_native_draw_plan(
    plan: &ZsPasswordBoxRenderPlan,
    value: &ZsPassword,
    reveal_mode: ZsPasswordRevealMode,
    peek_revealed: bool,
) -> NativeDrawPlan {
    let revealed = reveal_mode == ZsPasswordRevealMode::Visible
        || (reveal_mode == ZsPasswordRevealMode::Peek && peek_revealed);
    let mut commands = vec![NativeDrawCommand::RoundRect {
        rect: plan.bounds,
        fill: NativeDrawFill::Role(ColorRole::Surface),
        stroke: Some(NativeDrawFill::Role(ColorRole::Control)),
        radius: plan.radius,
    }];
    if revealed {
        commands.push(NativeDrawCommand::SecureText(
            NativeDrawSecureTextCommand::new(
                value.clone(),
                plan.text_bounds,
                SemanticTextStyle::body(),
                true,
            ),
        ));
    } else {
        commands.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            mask_password(value.as_str()),
            plan.text_bounds,
            SemanticTextStyle::body(),
        )));
    }
    if let Some(icon) = plan.reveal_icon {
        commands.push(NativeDrawCommand::Icon(NativeDrawIconCommand::new(
            ZsIcon::PasswordReveal,
            icon,
            NativeIconColorMode::ThemeAware,
        )));
    }
    NativeDrawPlan::new(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_debug_and_json_never_contain_the_secret() {
        let password = ZsPassword::from("s3cret🙂");
        assert_eq!(format!("{password:?}"), "ZsPassword(<redacted>)");

        let render = zs_password_box_render_plan(
            Rect {
                x: 0,
                y: 0,
                width: 180,
                height: 32,
            },
            ZsPasswordRevealMode::Visible,
            true,
            ZsPasswordBoxPlatformStyle::Windows,
            Dpi::standard(),
        );
        let draw = zs_password_box_native_draw_plan(
            &render,
            &password,
            ZsPasswordRevealMode::Visible,
            false,
        );
        let debug = format!("{draw:?}");
        let json =
            serde_json::to_string(&draw).expect("secure draw plan should serialize redacted");
        assert!(!debug.contains(password.as_str()));
        assert!(!json.contains(password.as_str()));
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::SecureText(command) if command.character_count() == 7
        )));
    }

    #[test]
    fn hidden_plan_contains_only_one_mask_per_unicode_scalar() {
        let password = ZsPassword::from("a🙂中");
        let render = zs_password_box_render_plan(
            Rect {
                x: 0,
                y: 0,
                width: 180,
                height: 34,
            },
            ZsPasswordRevealMode::Peek,
            true,
            ZsPasswordBoxPlatformStyle::Gtk,
            Dpi::standard(),
        );
        let draw =
            zs_password_box_native_draw_plan(&render, &password, ZsPasswordRevealMode::Peek, false);
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "•••"
        )));
        assert!(!format!("{draw:?}").contains(password.as_str()));
        assert!(render.reveal_button.is_some());
    }

    #[test]
    fn platform_defaults_follow_native_secure_entry_conventions() {
        let windows = ZsPasswordBoxMetrics::for_platform(ZsPasswordBoxPlatformStyle::Windows);
        let macos = ZsPasswordBoxMetrics::for_platform(ZsPasswordBoxPlatformStyle::Macos);
        let gtk = ZsPasswordBoxMetrics::for_platform(ZsPasswordBoxPlatformStyle::Gtk);
        assert_eq!(windows.radius, Dp::new(4.0));
        assert_eq!(macos.minimum_height, Dp::new(28.0));
        assert_eq!(gtk.minimum_height, Dp::new(34.0));
    }
}
