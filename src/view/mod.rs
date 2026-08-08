use std::collections::BTreeSet;
#[cfg(any(feature = "slider", feature = "number-box"))]
use std::ops::RangeInclusive;
use std::{
    fmt,
    marker::PhantomData,
    sync::{Arc, Mutex, MutexGuard},
};

#[cfg(any(feature = "button", feature = "label"))]
use crate::render_protocol::TextRole;
#[cfg(any(
    feature = "badge",
    feature = "label",
    feature = "button",
    feature = "textbox",
    feature = "checkbox",
    feature = "radio"
))]
use crate::render_protocol::{NativeDrawTextCommand, SemanticTextStyle};
#[cfg(feature = "date-picker")]
use crate::ZsDate;
use crate::{
    geometry::{ComponentId, Dp, Dpi, LayoutNode, LayoutOutput, Point, Rect},
    render_protocol::{ColorRole, NativeDrawCommand, NativeDrawFill, NativeDrawPlan},
    style::{ThemeColorToken, ZsuiThemeMode},
    Command, UiCommand,
};
#[cfg(any(feature = "image-preview", feature = "video"))]
use crate::{NativeDrawIconCommand, NativeIconColorMode, NativeImageInterpolation};
#[cfg(feature = "time-picker")]
use crate::{ZsClockFormat, ZsMinuteIncrement, ZsTime, ZsTimePickerPlatformStyle};
#[cfg(feature = "color-picker")]
use crate::{ZsColorChannel, ZsColorPickerPlatformStyle, ZsColorPickerState};
#[cfg(feature = "image-preview")]
use crate::{ZsImageFit, ZsImagePreviewSnapshot};
#[cfg(feature = "tabs")]
use crate::{ZsTabId, ZsTabSpec};
#[cfg(feature = "video")]
use crate::{ZsVideoFit, ZsVideoPlaybackState, ZsVideoSource};
use serde::{Deserialize, Serialize};

include!("node.rs");
include!("widgets/mod.rs");
include!("event.rs");
include!("focus.rs");
include!("paint.rs");
include!("overlay.rs");
include!("layout.rs");
include!("tests.rs");
