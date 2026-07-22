use crate::Color;
use serde::{Deserialize, Serialize};

/// HSV coordinates used by platform color-spectrum surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsHsvColor {
    /// Hue in degrees, normalized to `[0, 360)`.
    pub hue: f32,
    /// Saturation normalized to `[0, 1]`.
    pub saturation: f32,
    /// Value/brightness normalized to `[0, 1]`.
    pub value: f32,
}

impl ZsHsvColor {
    pub fn new(hue: f32, saturation: f32, value: f32) -> Self {
        let hue = if hue.is_finite() {
            hue.rem_euclid(360.0)
        } else {
            0.0
        };
        Self {
            hue,
            saturation: if saturation.is_finite() {
                saturation.clamp(0.0, 1.0)
            } else {
                0.0
            },
            value: if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                0.0
            },
        }
    }

    pub fn from_color(color: Color) -> Self {
        let red = f32::from(color.r) / 255.0;
        let green = f32::from(color.g) / 255.0;
        let blue = f32::from(color.b) / 255.0;
        let maximum = red.max(green).max(blue);
        let minimum = red.min(green).min(blue);
        let chroma = maximum - minimum;
        let hue = if chroma <= f32::EPSILON {
            0.0
        } else if (maximum - red).abs() <= f32::EPSILON {
            60.0 * ((green - blue) / chroma).rem_euclid(6.0)
        } else if (maximum - green).abs() <= f32::EPSILON {
            60.0 * (((blue - red) / chroma) + 2.0)
        } else {
            60.0 * (((red - green) / chroma) + 4.0)
        };
        let saturation = if maximum <= f32::EPSILON {
            0.0
        } else {
            chroma / maximum
        };
        Self::new(hue, saturation, maximum)
    }

    pub fn to_color(self, alpha: u8) -> Color {
        let normalized = Self::new(self.hue, self.saturation, self.value);
        let chroma = normalized.value * normalized.saturation;
        let hue_sector = normalized.hue / 60.0;
        let secondary = chroma * (1.0 - (hue_sector.rem_euclid(2.0) - 1.0).abs());
        let (red, green, blue) = match hue_sector as u8 {
            0 => (chroma, secondary, 0.0),
            1 => (secondary, chroma, 0.0),
            2 => (0.0, chroma, secondary),
            3 => (0.0, secondary, chroma),
            4 => (secondary, 0.0, chroma),
            _ => (chroma, 0.0, secondary),
        };
        let match_value = normalized.value - chroma;
        let channel = |value: f32| ((value + match_value) * 255.0).round().clamp(0.0, 255.0) as u8;
        Color::rgba(channel(red), channel(green), channel(blue), alpha)
    }

    pub fn with_hue(self, hue: f32) -> Self {
        Self::new(hue, self.saturation, self.value)
    }

    pub fn with_saturation_value(self, saturation: f32, value: f32) -> Self {
        Self::new(self.hue, saturation, value)
    }
}

/// A strongly typed editable channel in the self-drawn color picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsColorChannel {
    Red,
    Green,
    Blue,
    Alpha,
}

impl ZsColorChannel {
    pub const RGB: [Self; 3] = [Self::Red, Self::Green, Self::Blue];
    pub const RGBA: [Self; 4] = [Self::Red, Self::Green, Self::Blue, Self::Alpha];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Red => "R",
            Self::Green => "G",
            Self::Blue => "B",
            Self::Alpha => "A",
        }
    }

    pub const fn value(self, color: Color) -> u8 {
        match self {
            Self::Red => color.r,
            Self::Green => color.g,
            Self::Blue => color.b,
            Self::Alpha => color.a,
        }
    }

    pub const fn with_value(self, color: Color, value: u8) -> Color {
        match self {
            Self::Red => Color::rgba(value, color.g, color.b, color.a),
            Self::Green => Color::rgba(color.r, value, color.b, color.a),
            Self::Blue => Color::rgba(color.r, color.g, value, color.a),
            Self::Alpha => Color::rgba(color.r, color.g, color.b, value),
        }
    }

    pub const fn previous(self, alpha_enabled: bool) -> Self {
        match self {
            Self::Red => {
                if alpha_enabled {
                    Self::Alpha
                } else {
                    Self::Blue
                }
            }
            Self::Green => Self::Red,
            Self::Blue => Self::Green,
            Self::Alpha => Self::Blue,
        }
    }

    pub const fn next(self, alpha_enabled: bool) -> Self {
        match self {
            Self::Red => Self::Green,
            Self::Green => Self::Blue,
            Self::Blue if alpha_enabled => Self::Alpha,
            Self::Blue | Self::Alpha => Self::Red,
        }
    }
}

/// Application-owned value and navigation state for `color_picker`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsColorPickerState {
    pub color: Color,
    pub expanded: bool,
    pub active_channel: ZsColorChannel,
    pub alpha_enabled: bool,
}

impl ZsColorPickerState {
    pub const fn new(color: Color) -> Self {
        Self {
            color,
            expanded: false,
            active_channel: ZsColorChannel::Red,
            alpha_enabled: true,
        }
    }

    pub const fn without_alpha(mut self) -> Self {
        self.alpha_enabled = false;
        self.color.a = 255;
        if matches!(self.active_channel, ZsColorChannel::Alpha) {
            self.active_channel = ZsColorChannel::Red;
        }
        self
    }

    pub const fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub const fn with_active_channel(mut self, channel: ZsColorChannel) -> Self {
        self.active_channel = if !self.alpha_enabled && matches!(channel, ZsColorChannel::Alpha) {
            ZsColorChannel::Red
        } else {
            channel
        };
        self
    }

    pub const fn normalized(mut self) -> Self {
        if !self.alpha_enabled {
            self.color.a = 255;
            if matches!(self.active_channel, ZsColorChannel::Alpha) {
                self.active_channel = ZsColorChannel::Red;
            }
        }
        self
    }

    pub const fn channel_value(self, channel: ZsColorChannel) -> u8 {
        channel.value(self.color)
    }

    pub const fn with_channel_value(mut self, channel: ZsColorChannel, value: u8) -> Self {
        if self.alpha_enabled || !matches!(channel, ZsColorChannel::Alpha) {
            self.color = channel.with_value(self.color, value);
            self.active_channel = channel;
        }
        self.normalized()
    }

    pub const fn channels(self) -> &'static [ZsColorChannel] {
        if self.alpha_enabled {
            &ZsColorChannel::RGBA
        } else {
            &ZsColorChannel::RGB
        }
    }

    pub fn hex_label(self) -> String {
        if self.alpha_enabled {
            self.color.hex_rgba()
        } else {
            format!(
                "#{:02X}{:02X}{:02X}",
                self.color.r, self.color.g, self.color.b
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_edits_preserve_other_components() {
        let state = ZsColorPickerState::new(Color::rgba(10, 20, 30, 40))
            .with_channel_value(ZsColorChannel::Green, 220);

        assert_eq!(state.color, Color::rgba(10, 220, 30, 40));
        assert_eq!(state.active_channel, ZsColorChannel::Green);
        assert_eq!(state.hex_label(), "#0ADC1E28");
    }

    #[test]
    fn disabling_alpha_normalizes_value_and_channel_navigation() {
        let state = ZsColorPickerState::new(Color::rgba(10, 20, 30, 40))
            .with_active_channel(ZsColorChannel::Alpha)
            .without_alpha();

        assert_eq!(state.color.a, 255);
        assert_eq!(state.active_channel, ZsColorChannel::Red);
        assert_eq!(state.channels(), &ZsColorChannel::RGB);
        assert_eq!(ZsColorChannel::Blue.next(false), ZsColorChannel::Red);
        assert_eq!(state.hex_label(), "#0A141E");
    }

    #[test]
    fn hsv_round_trip_and_spectrum_coordinates_preserve_alpha() {
        for color in [
            Color::rgba(255, 0, 0, 18),
            Color::rgba(32, 96, 160, 72),
            Color::rgba(250, 240, 30, 200),
            Color::rgba(96, 96, 96, 255),
        ] {
            let round_trip = ZsHsvColor::from_color(color).to_color(color.a);
            assert!((i16::from(round_trip.r) - i16::from(color.r)).abs() <= 1);
            assert!((i16::from(round_trip.g) - i16::from(color.g)).abs() <= 1);
            assert!((i16::from(round_trip.b) - i16::from(color.b)).abs() <= 1);
            assert_eq!(round_trip.a, color.a);
        }

        assert_eq!(
            ZsHsvColor::new(120.0, 1.0, 1.0).to_color(144),
            Color::rgba(0, 255, 0, 144)
        );
    }
}
