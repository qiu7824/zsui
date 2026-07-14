#![allow(non_snake_case, non_upper_case_globals)]

use std::sync::{Arc, Mutex};

use windows::core::{implement, Error, IUnknown, Interface, Result, BOOL, BSTR, HRESULT, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Com::SAFEARRAY;
use windows::Win32::System::Ole::{SafeArrayCreateVector, SafeArrayDestroy, SafeArrayPutElement};
use windows::Win32::System::Variant::{VARIANT, VT_R8, VT_UNKNOWN};
use windows::Win32::UI::Accessibility::{
    IRawElementProviderSimple, IRawElementProviderSimple_Impl, ITextProvider, ITextProvider_Impl,
    ITextRangeProvider, ITextRangeProvider_Impl, IValueProvider, IValueProvider_Impl,
    ProviderOptions, ProviderOptions_ServerSideProvider, SupportedTextSelection,
    SupportedTextSelection_Single, TextPatternRangeEndpoint, TextPatternRangeEndpoint_End,
    TextPatternRangeEndpoint_Start, TextUnit, TextUnit_Character, TextUnit_Document,
    TextUnit_Format, TextUnit_Line, TextUnit_Page, TextUnit_Paragraph, TextUnit_Word,
    UIA_AutomationIdPropertyId, UIA_ClassNamePropertyId, UIA_ControlTypePropertyId,
    UIA_EditControlTypeId, UIA_FrameworkIdPropertyId, UIA_HasKeyboardFocusPropertyId,
    UIA_IsEnabledPropertyId, UIA_IsKeyboardFocusablePropertyId, UIA_IsPasswordPropertyId,
    UIA_IsReadOnlyAttributeId, UIA_NativeWindowHandlePropertyId, UIA_TextPatternId,
    UIA_ValueIsReadOnlyPropertyId, UIA_ValuePatternId, UIA_ValueValuePropertyId,
    UiaGetReservedNotSupportedValue, UiaHostProviderFromHwnd, UiaPoint,
    UiaReturnRawElementProvider, UiaRootObjectId, UIA_E_ELEMENTNOTAVAILABLE,
    UIA_E_INVALIDOPERATION, UIA_E_NOTSUPPORTED, UIA_PATTERN_ID, UIA_PROPERTY_ID,
    UIA_TEXTATTRIBUTE_ID,
};
use windows_core::{AsImpl, IUnknownImpl};

#[implement(IRawElementProviderSimple, IValueProvider, ITextProvider)]
struct WindowsTextUiaProvider {
    hwnd: isize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WindowsTextRangeState {
    start: usize,
    end: usize,
}

impl WindowsTextRangeState {
    fn new(start: usize, end: usize) -> Self {
        Self {
            start: start.min(end),
            end: start.max(end),
        }
    }

    fn clamp(self, character_count: usize) -> Self {
        Self::new(
            self.start.min(character_count),
            self.end.min(character_count),
        )
    }

    const fn is_degenerate(self) -> bool {
        self.start == self.end
    }
}

#[implement(ITextRangeProvider)]
struct WindowsTextRangeProvider {
    hwnd: isize,
    widget: crate::WidgetId,
    state: Arc<Mutex<WindowsTextRangeState>>,
}

impl WindowsTextRangeProvider {
    fn range_state(&self) -> Result<WindowsTextRangeState> {
        let snapshot = crate::windows_win32_host::windows_win32_window_text_accessibility_snapshot(
            self.hwnd as _,
        )
        .ok_or_else(element_not_available)?;
        if snapshot.widget() != self.widget || snapshot.kind().is_protected() {
            return Err(element_not_available());
        }
        let mut state = self.state.lock().map_err(|_| element_not_available())?;
        *state = state.clamp(snapshot.character_count());
        Ok(*state)
    }
}

impl WindowsTextUiaProvider_Impl {
    fn snapshot(&self) -> Result<crate::native_accessibility::NativeTextAccessibilitySnapshot> {
        crate::windows_win32_host::windows_win32_window_text_accessibility_snapshot(self.hwnd as _)
            .ok_or_else(|| Error::from_hresult(HRESULT(UIA_E_ELEMENTNOTAVAILABLE as i32)))
    }
}

impl WindowsTextRangeProvider_Impl {
    fn snapshot(&self) -> Result<crate::native_accessibility::NativeTextAccessibilitySnapshot> {
        let snapshot = crate::windows_win32_host::windows_win32_window_text_accessibility_snapshot(
            self.hwnd as _,
        )
        .ok_or_else(element_not_available)?;
        if snapshot.widget() != self.widget || snapshot.kind().is_protected() {
            return Err(element_not_available());
        }
        Ok(snapshot)
    }

    fn range_state(&self) -> Result<WindowsTextRangeState> {
        let character_count = self.snapshot()?.character_count();
        let mut state = self.state.lock().map_err(|_| element_not_available())?;
        *state = state.clamp(character_count);
        Ok(*state)
    }

    fn replace_state(&self, state: WindowsTextRangeState) -> Result<()> {
        let character_count = self.snapshot()?.character_count();
        *self.state.lock().map_err(|_| element_not_available())? = state.clamp(character_count);
        Ok(())
    }

    fn other_range<'a>(
        range: &'a windows_core::Ref<'a, ITextRangeProvider>,
    ) -> Result<&'a WindowsTextRangeProvider> {
        let range = range.ok().map_err(|_| invalid_argument())?;
        Ok(unsafe { range.as_impl() })
    }

    fn ensure_same_text_provider(&self, other: &WindowsTextRangeProvider) -> Result<()> {
        if self.hwnd == other.hwnd && self.widget == other.widget {
            Ok(())
        } else {
            Err(invalid_argument())
        }
    }
}

fn element_not_available() -> Error {
    Error::from_hresult(HRESULT(UIA_E_ELEMENTNOTAVAILABLE as i32))
}

fn not_supported() -> Error {
    Error::from_hresult(HRESULT(UIA_E_NOTSUPPORTED as i32))
}

fn invalid_operation() -> Error {
    Error::from_hresult(HRESULT(UIA_E_INVALIDOPERATION as i32))
}

fn invalid_argument() -> Error {
    Error::from_hresult(HRESULT(0x8007_0057_u32 as i32))
}

fn not_implemented() -> Error {
    Error::from_hresult(HRESULT(0x8000_4001_u32 as i32))
}

fn out_of_memory() -> Error {
    Error::from_hresult(HRESULT(0x8007_000e_u32 as i32))
}

fn text_provider(hwnd: isize) -> IRawElementProviderSimple {
    IRawElementProviderSimple::from(WindowsTextUiaProvider { hwnd })
}

fn text_range_provider(
    hwnd: isize,
    widget: crate::WidgetId,
    state: WindowsTextRangeState,
) -> ITextRangeProvider {
    ITextRangeProvider::from(WindowsTextRangeProvider {
        hwnd,
        widget,
        state: Arc::new(Mutex::new(state)),
    })
}

fn safe_array_from_com_slice(values: &[IUnknown]) -> Result<*mut SAFEARRAY> {
    let len = u32::try_from(values.len()).map_err(|_| out_of_memory())?;
    let array = unsafe { SafeArrayCreateVector(VT_UNKNOWN, 0, len) };
    if array.is_null() {
        return Err(out_of_memory());
    }
    for (index, value) in values.iter().enumerate() {
        let index = i32::try_from(index).map_err(|_| out_of_memory())?;
        if let Err(error) =
            unsafe { SafeArrayPutElement(array, &index, core::mem::transmute_copy(value)) }
        {
            unsafe {
                let _ = SafeArrayDestroy(array);
            }
            return Err(error);
        }
    }
    Ok(array)
}

fn safe_array_from_f64_slice(values: &[f64]) -> Result<*mut SAFEARRAY> {
    let len = u32::try_from(values.len()).map_err(|_| out_of_memory())?;
    let array = unsafe { SafeArrayCreateVector(VT_R8, 0, len) };
    if array.is_null() {
        return Err(out_of_memory());
    }
    for (index, value) in values.iter().enumerate() {
        let index = i32::try_from(index).map_err(|_| out_of_memory())?;
        if let Err(error) =
            unsafe { SafeArrayPutElement(array, &index, (value as *const f64).cast()) }
        {
            unsafe {
                let _ = SafeArrayDestroy(array);
            }
            return Err(error);
        }
    }
    Ok(array)
}

fn endpoint_index(
    state: WindowsTextRangeState,
    endpoint: TextPatternRangeEndpoint,
) -> Result<usize> {
    if endpoint == TextPatternRangeEndpoint_Start {
        Ok(state.start)
    } else if endpoint == TextPatternRangeEndpoint_End {
        Ok(state.end)
    } else {
        Err(invalid_argument())
    }
}

fn set_endpoint_index(
    state: &mut WindowsTextRangeState,
    endpoint: TextPatternRangeEndpoint,
    index: usize,
) -> Result<()> {
    if endpoint == TextPatternRangeEndpoint_Start {
        state.start = index;
        if state.start > state.end {
            state.end = state.start;
        }
        Ok(())
    } else if endpoint == TextPatternRangeEndpoint_End {
        state.end = index;
        if state.end < state.start {
            state.start = state.end;
        }
        Ok(())
    } else {
        Err(invalid_argument())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextCharacterClass {
    Whitespace,
    Word,
    Punctuation,
}

fn character_class(character: char) -> TextCharacterClass {
    if character.is_whitespace() {
        TextCharacterClass::Whitespace
    } else if character.is_alphanumeric() || character == '_' {
        TextCharacterClass::Word
    } else {
        TextCharacterClass::Punctuation
    }
}

fn text_unit_boundaries(text: &str, unit: TextUnit) -> Result<Vec<usize>> {
    let characters = text.chars().collect::<Vec<_>>();
    let mut boundaries = vec![0];
    if unit == TextUnit_Character {
        boundaries = crate::native_text_edit::grapheme_boundaries(text);
    } else if unit == TextUnit_Word {
        let grapheme_boundaries = crate::native_text_edit::grapheme_boundaries(text);
        let mut previous_class = characters.first().copied().map(character_class);
        for index in grapheme_boundaries
            .iter()
            .copied()
            .skip(1)
            .take_while(|index| *index < characters.len())
        {
            let class = character_class(characters[index]);
            if previous_class.is_some_and(|previous| previous != class) {
                boundaries.push(index);
            }
            previous_class = Some(class);
        }
        boundaries.push(characters.len());
    } else if unit == TextUnit_Line || unit == TextUnit_Paragraph {
        for (index, character) in characters.iter().enumerate() {
            if *character == '\n' {
                boundaries.push(index.saturating_add(1));
            }
        }
        boundaries.push(characters.len());
    } else if unit == TextUnit_Format || unit == TextUnit_Page || unit == TextUnit_Document {
        boundaries.push(characters.len());
    } else {
        return Err(invalid_argument());
    }
    boundaries.sort_unstable();
    boundaries.dedup();
    Ok(boundaries)
}

fn enclosing_text_unit(text: &str, index: usize, unit: TextUnit) -> Result<WindowsTextRangeState> {
    let length = text.chars().count();
    let index = index.min(length);
    if unit == TextUnit_Document || unit == TextUnit_Page || unit == TextUnit_Format {
        return Ok(WindowsTextRangeState::new(0, length));
    }
    if index == length && unit == TextUnit_Character {
        return Ok(WindowsTextRangeState::new(length, length));
    }
    let boundaries = text_unit_boundaries(text, unit)?;
    let start = boundaries
        .iter()
        .copied()
        .take_while(|boundary| *boundary <= index)
        .last()
        .unwrap_or(0);
    let end = boundaries
        .iter()
        .copied()
        .find(|boundary| *boundary > start)
        .unwrap_or(length);
    Ok(WindowsTextRangeState::new(start, end))
}

fn move_text_position(
    text: &str,
    index: usize,
    unit: TextUnit,
    count: i32,
) -> Result<(usize, i32)> {
    if count == 0 {
        return Ok((index.min(text.chars().count()), 0));
    }
    let boundaries = text_unit_boundaries(text, unit)?;
    let mut position = index.min(text.chars().count());
    let mut moved = 0_i32;
    if count > 0 {
        let steps = count.unsigned_abs().min(boundaries.len() as u32);
        for _ in 0..steps {
            let Some(next) = boundaries
                .iter()
                .copied()
                .find(|boundary| *boundary > position)
            else {
                break;
            };
            position = next;
            moved = moved.saturating_add(1);
        }
    } else {
        let steps = count.unsigned_abs().min(boundaries.len() as u32);
        for _ in 0..steps {
            let Some(previous) = boundaries
                .iter()
                .copied()
                .take_while(|boundary| *boundary < position)
                .last()
            else {
                break;
            };
            position = previous;
            moved = moved.saturating_sub(1);
        }
    }
    Ok((position, moved))
}

fn scalar_text(text: &str, state: WindowsTextRangeState, maximum_length: i32) -> String {
    let available = state.end.saturating_sub(state.start);
    let take = if maximum_length < 0 {
        available
    } else {
        available.min(maximum_length as usize)
    };
    text.chars().skip(state.start).take(take).collect()
}

impl IRawElementProviderSimple_Impl for WindowsTextUiaProvider_Impl {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider)
    }

    fn GetPatternProvider(&self, pattern_id: UIA_PATTERN_ID) -> Result<IUnknown> {
        let snapshot = self.snapshot()?;
        if pattern_id == UIA_ValuePatternId && !snapshot.kind().is_protected() {
            let provider: IValueProvider = self.to_interface();
            return provider.cast();
        }
        if pattern_id == UIA_TextPatternId && !snapshot.kind().is_protected() {
            let provider: ITextProvider = self.to_interface();
            return provider.cast();
        }
        Err(Error::empty())
    }

    fn GetPropertyValue(&self, property_id: UIA_PROPERTY_ID) -> Result<VARIANT> {
        let snapshot = self.snapshot()?;
        let value = match property_id {
            UIA_ControlTypePropertyId => VARIANT::from(UIA_EditControlTypeId.0),
            UIA_HasKeyboardFocusPropertyId
            | UIA_IsEnabledPropertyId
            | UIA_IsKeyboardFocusablePropertyId => VARIANT::from(true),
            UIA_IsPasswordPropertyId => VARIANT::from(snapshot.kind().is_protected()),
            UIA_NativeWindowHandlePropertyId => VARIANT::from(self.hwnd as i32),
            UIA_FrameworkIdPropertyId => VARIANT::from(BSTR::from("ZSUI")),
            UIA_ClassNamePropertyId => VARIANT::from(BSTR::from("ZsuiTextInput")),
            UIA_AutomationIdPropertyId => {
                VARIANT::from(BSTR::from(format!("zsui-widget-{}", snapshot.widget().0)))
            }
            UIA_ValueIsReadOnlyPropertyId => VARIANT::from(false),
            UIA_ValueValuePropertyId if !snapshot.kind().is_protected() => {
                VARIANT::from(BSTR::from(snapshot.exposed_text()))
            }
            _ => VARIANT::default(),
        };
        Ok(value)
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        unsafe { UiaHostProviderFromHwnd(HWND(self.hwnd as *mut core::ffi::c_void)) }
    }
}

impl IValueProvider_Impl for WindowsTextUiaProvider_Impl {
    fn SetValue(&self, value: &PCWSTR) -> Result<()> {
        if self.snapshot()?.kind().is_protected() {
            return Err(Error::from_hresult(HRESULT(UIA_E_NOTSUPPORTED as i32)));
        }
        let value = unsafe { value.to_string()? };
        crate::windows_win32_host::set_windows_win32_window_accessible_text_value(
            self.hwnd as _,
            &value,
        )
        .then_some(())
        .ok_or_else(|| Error::from_hresult(HRESULT(UIA_E_ELEMENTNOTAVAILABLE as i32)))
    }

    fn Value(&self) -> Result<BSTR> {
        let snapshot = self.snapshot()?;
        if snapshot.kind().is_protected() {
            return Err(Error::from_hresult(HRESULT(UIA_E_NOTSUPPORTED as i32)));
        }
        Ok(BSTR::from(snapshot.exposed_text()))
    }

    fn IsReadOnly(&self) -> Result<BOOL> {
        Ok(false.into())
    }
}

impl ITextProvider_Impl for WindowsTextUiaProvider_Impl {
    fn GetSelection(&self) -> Result<*mut SAFEARRAY> {
        let snapshot = self.snapshot()?;
        if snapshot.kind().is_protected() {
            return Err(not_supported());
        }
        let selection = snapshot.ordered_selection();
        let range = text_range_provider(
            self.hwnd,
            snapshot.widget(),
            WindowsTextRangeState::new(selection.start, selection.end),
        );
        safe_array_from_com_slice(&[range.cast()?])
    }

    fn GetVisibleRanges(&self) -> Result<*mut SAFEARRAY> {
        let snapshot = self.snapshot()?;
        if snapshot.kind().is_protected() {
            return Err(not_supported());
        }
        let (widget, visible) =
            crate::windows_win32_host::windows_win32_window_text_accessibility_visible_range(
                self.hwnd as _,
            )
            .ok_or_else(element_not_available)?;
        if widget != snapshot.widget() {
            return Err(element_not_available());
        }
        let range = text_range_provider(
            self.hwnd,
            widget,
            WindowsTextRangeState::new(visible.start, visible.end),
        );
        safe_array_from_com_slice(&[range.cast()?])
    }

    fn RangeFromChild(
        &self,
        _child_element: windows_core::Ref<IRawElementProviderSimple>,
    ) -> Result<ITextRangeProvider> {
        self.snapshot()?;
        Err(not_implemented())
    }

    fn RangeFromPoint(&self, point: &UiaPoint) -> Result<ITextRangeProvider> {
        let snapshot = self.snapshot()?;
        if snapshot.kind().is_protected() {
            return Err(not_supported());
        }
        let (widget, index) =
            crate::windows_win32_host::windows_win32_window_text_accessibility_index_for_screen_point(
                self.hwnd as _,
                point.x,
                point.y,
            )
            .ok_or_else(element_not_available)?;
        if widget != snapshot.widget() {
            return Err(element_not_available());
        }
        Ok(text_range_provider(
            self.hwnd,
            widget,
            WindowsTextRangeState::new(index, index),
        ))
    }

    fn DocumentRange(&self) -> Result<ITextRangeProvider> {
        let snapshot = self.snapshot()?;
        if snapshot.kind().is_protected() {
            return Err(not_supported());
        }
        Ok(text_range_provider(
            self.hwnd,
            snapshot.widget(),
            WindowsTextRangeState::new(0, snapshot.character_count()),
        ))
    }

    fn SupportedTextSelection(&self) -> Result<SupportedTextSelection> {
        if self.snapshot()?.kind().is_protected() {
            return Err(not_supported());
        }
        Ok(SupportedTextSelection_Single)
    }
}

impl ITextRangeProvider_Impl for WindowsTextRangeProvider_Impl {
    fn Clone(&self) -> Result<ITextRangeProvider> {
        Ok(text_range_provider(
            self.hwnd,
            self.widget,
            self.range_state()?,
        ))
    }

    fn Compare(&self, range: windows_core::Ref<ITextRangeProvider>) -> Result<BOOL> {
        let other = Self::other_range(&range)?;
        self.ensure_same_text_provider(other)?;
        Ok((self.range_state()? == other.range_state()?).into())
    }

    fn CompareEndpoints(
        &self,
        endpoint: TextPatternRangeEndpoint,
        target_range: windows_core::Ref<ITextRangeProvider>,
        target_endpoint: TextPatternRangeEndpoint,
    ) -> Result<i32> {
        let other = Self::other_range(&target_range)?;
        self.ensure_same_text_provider(other)?;
        let current = endpoint_index(self.range_state()?, endpoint)?;
        let target = endpoint_index(other.range_state()?, target_endpoint)?;
        Ok(current.cmp(&target) as i32)
    }

    fn ExpandToEnclosingUnit(&self, unit: TextUnit) -> Result<()> {
        let snapshot = self.snapshot()?;
        let state = self.range_state()?;
        self.replace_state(enclosing_text_unit(
            snapshot.exposed_text(),
            state.start,
            unit,
        )?)
    }

    fn FindAttribute(
        &self,
        _attribute_id: UIA_TEXTATTRIBUTE_ID,
        _value: &VARIANT,
        _backward: BOOL,
    ) -> Result<ITextRangeProvider> {
        self.snapshot()?;
        Err(Error::empty())
    }

    fn FindText(
        &self,
        text: &BSTR,
        backward: BOOL,
        ignore_case: BOOL,
    ) -> Result<ITextRangeProvider> {
        let snapshot = self.snapshot()?;
        let state = self.range_state()?;
        let needle = text.to_string();
        if needle.is_empty() {
            return Err(Error::empty());
        }
        let haystack = snapshot
            .text_in_range(state.start..state.end)
            .ok_or_else(element_not_available)?;
        let needle_length = needle.chars().count();
        let haystack_characters = haystack.chars().collect::<Vec<_>>();
        if needle_length > haystack_characters.len() {
            return Err(Error::empty());
        }
        let needle_key = if bool::from(ignore_case) {
            needle.to_lowercase()
        } else {
            needle
        };
        let mut candidates = 0..=haystack_characters.len().saturating_sub(needle_length);
        let found = if bool::from(backward) {
            candidates.rev().find(|start| {
                let candidate = haystack_characters[*start..*start + needle_length]
                    .iter()
                    .collect::<String>();
                if bool::from(ignore_case) {
                    candidate.to_lowercase() == needle_key
                } else {
                    candidate == needle_key
                }
            })
        } else {
            candidates.find(|start| {
                let candidate = haystack_characters[*start..*start + needle_length]
                    .iter()
                    .collect::<String>();
                if bool::from(ignore_case) {
                    candidate.to_lowercase() == needle_key
                } else {
                    candidate == needle_key
                }
            })
        };
        let Some(found) = found else {
            return Err(Error::empty());
        };
        let start = state.start + found;
        Ok(text_range_provider(
            self.hwnd,
            self.widget,
            WindowsTextRangeState::new(start, start + needle_length),
        ))
    }

    fn GetAttributeValue(&self, attribute_id: UIA_TEXTATTRIBUTE_ID) -> Result<VARIANT> {
        self.snapshot()?;
        if attribute_id == UIA_IsReadOnlyAttributeId {
            return Ok(VARIANT::from(false));
        }
        Ok(unsafe { UiaGetReservedNotSupportedValue()? }.into())
    }

    fn GetBoundingRectangles(&self) -> Result<*mut SAFEARRAY> {
        let state = self.range_state()?;
        let rectangles =
            crate::windows_win32_host::windows_win32_window_text_accessibility_range_rectangles(
                self.hwnd as _,
                self.widget,
                crate::native_text_edit::NativeTextSelection {
                    anchor: state.start,
                    caret: state.end,
                },
            )
            .ok_or_else(element_not_available)?;
        if rectangles.is_empty() {
            return Ok(core::ptr::null_mut());
        }
        let mut values = Vec::with_capacity(rectangles.len() * 4);
        for rectangle in rectangles {
            let mut point = windows_sys::Win32::Foundation::POINT {
                x: rectangle.x,
                y: rectangle.y,
            };
            if unsafe {
                windows_sys::Win32::Graphics::Gdi::ClientToScreen(self.hwnd as _, &mut point)
            } == 0
            {
                return Err(element_not_available());
            }
            values.extend([
                f64::from(point.x),
                f64::from(point.y),
                f64::from(rectangle.width),
                f64::from(rectangle.height),
            ]);
        }
        safe_array_from_f64_slice(&values)
    }

    fn GetEnclosingElement(&self) -> Result<IRawElementProviderSimple> {
        self.snapshot()?;
        Ok(text_provider(self.hwnd))
    }

    fn GetText(&self, maximum_length: i32) -> Result<BSTR> {
        let snapshot = self.snapshot()?;
        Ok(BSTR::from(scalar_text(
            snapshot.exposed_text(),
            self.range_state()?,
            maximum_length,
        )))
    }

    fn Move(&self, unit: TextUnit, count: i32) -> Result<i32> {
        let snapshot = self.snapshot()?;
        let state = self.range_state()?;
        let degenerate = state.is_degenerate();
        let origin = if degenerate {
            state.start
        } else {
            enclosing_text_unit(snapshot.exposed_text(), state.start, unit)?.start
        };
        let (start, moved) = move_text_position(snapshot.exposed_text(), origin, unit, count)?;
        if moved != 0 {
            let next = if degenerate {
                WindowsTextRangeState::new(start, start)
            } else {
                enclosing_text_unit(snapshot.exposed_text(), start, unit)?
            };
            self.replace_state(next)?;
        }
        Ok(moved)
    }

    fn MoveEndpointByUnit(
        &self,
        endpoint: TextPatternRangeEndpoint,
        unit: TextUnit,
        count: i32,
    ) -> Result<i32> {
        let snapshot = self.snapshot()?;
        let mut state = self.range_state()?;
        let index = endpoint_index(state, endpoint)?;
        let (index, moved) = move_text_position(snapshot.exposed_text(), index, unit, count)?;
        set_endpoint_index(&mut state, endpoint, index)?;
        self.replace_state(state)?;
        Ok(moved)
    }

    fn MoveEndpointByRange(
        &self,
        endpoint: TextPatternRangeEndpoint,
        target_range: windows_core::Ref<ITextRangeProvider>,
        target_endpoint: TextPatternRangeEndpoint,
    ) -> Result<()> {
        let other = Self::other_range(&target_range)?;
        self.ensure_same_text_provider(other)?;
        let index = endpoint_index(other.range_state()?, target_endpoint)?;
        let mut state = self.range_state()?;
        set_endpoint_index(&mut state, endpoint, index)?;
        self.replace_state(state)
    }

    fn Select(&self) -> Result<()> {
        let state = self.range_state()?;
        crate::windows_win32_host::set_windows_win32_window_accessible_text_selection(
            self.hwnd as _,
            self.widget,
            crate::native_text_edit::NativeTextSelection {
                anchor: state.start,
                caret: state.end,
            },
        )
        .then_some(())
        .ok_or_else(element_not_available)
    }

    fn AddToSelection(&self) -> Result<()> {
        Err(invalid_operation())
    }

    fn RemoveFromSelection(&self) -> Result<()> {
        Err(invalid_operation())
    }

    fn ScrollIntoView(&self, _align_to_top: BOOL) -> Result<()> {
        self.snapshot()?;
        Err(not_supported())
    }

    fn GetChildren(&self) -> Result<*mut SAFEARRAY> {
        self.snapshot()?;
        safe_array_from_com_slice(&[])
    }
}

pub(crate) fn handle_get_object(
    hwnd: windows_sys::Win32::Foundation::HWND,
    wparam: windows_sys::Win32::Foundation::WPARAM,
    lparam: windows_sys::Win32::Foundation::LPARAM,
) -> Option<windows_sys::Win32::Foundation::LRESULT> {
    if lparam != UiaRootObjectId as isize {
        return None;
    }
    crate::windows_win32_host::windows_win32_window_text_accessibility_snapshot(hwnd)?;
    let provider = text_provider(hwnd as isize);
    let result = unsafe {
        UiaReturnRawElementProvider(HWND(hwnd.cast()), WPARAM(wparam), LPARAM(lparam), &provider)
    };
    Some(result.0)
}

pub(crate) fn disconnect(hwnd: windows_sys::Win32::Foundation::HWND) {
    unsafe {
        let _ = UiaReturnRawElementProvider(
            HWND(hwnd.cast()),
            WPARAM(0),
            LPARAM(0),
            None::<&IRawElementProviderSimple>,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "password-box")]
    use crate::View;

    #[test]
    fn text_character_units_follow_extended_grapheme_boundaries() {
        assert_eq!(
            text_unit_boundaries("e\u{301}中", TextUnit_Character).expect("character boundaries"),
            vec![0, 2, 3]
        );
        assert_eq!(
            text_unit_boundaries("e\u{301} 中", TextUnit_Word).expect("word boundaries"),
            vec![0, 2, 3, 4]
        );
    }

    #[test]
    fn provider_exposes_the_native_server_side_contract() {
        let provider = IRawElementProviderSimple::from(WindowsTextUiaProvider { hwnd: 1 });
        assert!(provider.cast::<IValueProvider>().is_ok());
        assert!(provider.cast::<ITextProvider>().is_ok());
        let options = unsafe { provider.ProviderOptions() }.expect("provider options");
        assert_eq!(options, ProviderOptions_ServerSideProvider);
    }

    #[cfg(feature = "textbox")]
    #[test]
    fn value_pattern_reads_and_replaces_the_focused_typed_text_route() {
        let _guard = crate::windows_win32_host::windows_win32_view_input_route_test_lock();
        let hwnd = 0x5a51isize as windows_sys::Win32::Foundation::HWND;
        let widget = crate::WidgetId::new(0x5a51);
        crate::windows_win32_host::clear_windows_win32_window_view_input_routes();
        let route = crate::windows_win32_host::WindowsWin32ViewInputRoute::new(
            crate::ViewInteractionPlan::new([crate::ViewHitTarget::with_kind(
                widget,
                crate::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 40,
                },
                crate::ViewHitTargetKind::Textbox,
            )]),
            crate::textbox::<crate::UiCommand>("A😀中").id(widget),
        );
        assert!(crate::windows_win32_host::set_windows_win32_window_view_input_route(hwnd, route));
        crate::windows_win32_host::dispatch_windows_win32_window_view_click(
            hwnd,
            crate::Point { x: 20, y: 20 },
        )
        .expect("registered text route should accept focus");

        let raw = IRawElementProviderSimple::from(WindowsTextUiaProvider {
            hwnd: hwnd as isize,
        });
        let pattern = unsafe { raw.GetPatternProvider(UIA_ValuePatternId) }
            .expect("focused textbox should expose the UIA Value pattern");
        let value: IValueProvider = pattern.cast().expect("Value pattern interface");
        assert_eq!(
            unsafe { value.Value() }
                .expect("UIA value should be readable")
                .to_string(),
            "A😀中"
        );

        let pattern = unsafe { raw.GetPatternProvider(UIA_TextPatternId) }
            .expect("focused textbox should expose the UIA Text pattern");
        let text: ITextProvider = pattern.cast().expect("Text pattern interface");
        assert_eq!(
            unsafe { text.SupportedTextSelection() }.expect("selection contract"),
            SupportedTextSelection_Single
        );
        let document = unsafe { text.DocumentRange() }.expect("document range");
        assert_eq!(
            unsafe { document.GetText(-1) }
                .expect("document text")
                .to_string(),
            "A😀中"
        );
        let rectangles =
            crate::windows_win32_host::windows_win32_window_text_accessibility_range_rectangles(
                hwnd,
                widget,
                crate::native_text_edit::NativeTextSelection {
                    anchor: 0,
                    caret: 3,
                },
            )
            .expect("document range rectangles");
        assert!(!rectangles.is_empty());
        let selected = unsafe { text.GetSelection() }.expect("selected range array");
        assert!(!selected.is_null());
        unsafe { SafeArrayDestroy(selected) }.expect("destroy selected range array");
        let visible = unsafe { text.GetVisibleRanges() }.expect("visible range array");
        assert!(!visible.is_null());
        unsafe { SafeArrayDestroy(visible) }.expect("destroy visible range array");

        let moved = unsafe { document.Clone() }.expect("independent text range");
        assert_eq!(
            unsafe {
                moved.MoveEndpointByUnit(TextPatternRangeEndpoint_Start, TextUnit_Character, 1)
            }
            .expect("move start by one character"),
            1
        );
        assert_eq!(
            unsafe { moved.GetText(-1) }
                .expect("moved range text")
                .to_string(),
            "😀中"
        );
        assert!(!bool::from(
            unsafe { document.Compare(&moved) }.expect("compare independent ranges")
        ));
        assert_eq!(
            unsafe {
                document.CompareEndpoints(
                    TextPatternRangeEndpoint_Start,
                    &moved,
                    TextPatternRangeEndpoint_Start,
                )
            }
            .expect("compare range endpoints"),
            -1
        );
        let emoji = unsafe { document.FindText(&BSTR::from("😀"), false, false) }
            .expect("find emoji range");
        assert_eq!(
            unsafe { emoji.GetText(-1) }
                .expect("found emoji text")
                .to_string(),
            "😀"
        );
        let copied_endpoints = unsafe { document.Clone() }.expect("endpoint copy range");
        unsafe {
            copied_endpoints.MoveEndpointByRange(
                TextPatternRangeEndpoint_Start,
                &emoji,
                TextPatternRangeEndpoint_Start,
            )
        }
        .expect("copy range start endpoint");
        unsafe {
            copied_endpoints.MoveEndpointByRange(
                TextPatternRangeEndpoint_End,
                &emoji,
                TextPatternRangeEndpoint_End,
            )
        }
        .expect("copy range end endpoint");
        assert_eq!(
            unsafe { copied_endpoints.GetText(-1) }
                .expect("copied endpoint text")
                .to_string(),
            "😀"
        );
        unsafe { moved.Select() }.expect("text range should route native selection");
        let selected_snapshot =
            crate::windows_win32_host::windows_win32_window_text_accessibility_snapshot(hwnd)
                .expect("selection should preserve the focused accessibility snapshot");
        assert_eq!(selected_snapshot.selection().ordered(), (1, 3));

        let replacement = windows::core::HSTRING::from("native UIA");
        unsafe { value.SetValue(&replacement) }.expect("UIA should route typed replacement");
        let snapshot =
            crate::windows_win32_host::windows_win32_window_text_accessibility_snapshot(hwnd)
                .expect("focused route should keep an accessibility snapshot");
        assert_eq!(snapshot.exposed_text(), "native UIA");
        assert_eq!(snapshot.selection().caret, 10);

        crate::windows_win32_host::clear_windows_win32_window_view_input_route(hwnd);
    }

    #[cfg(feature = "password-box")]
    #[test]
    fn protected_text_route_does_not_advertise_or_accept_the_value_pattern() {
        let _guard = crate::windows_win32_host::windows_win32_view_input_route_test_lock();
        let hwnd = 0x5a52isize as windows_sys::Win32::Foundation::HWND;
        let widget = crate::WidgetId::new(0x5a52);
        crate::windows_win32_host::clear_windows_win32_window_view_input_routes();
        let mut view = crate::password_box::<crate::UiCommand>("secret").id(widget);
        let mut layout = crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 180,
                height: 40,
            },
            crate::Dpi::standard(),
        );
        view.layout(&mut layout);
        let route = crate::windows_win32_host::WindowsWin32ViewInputRoute::new(
            view.interaction_plan(),
            view,
        );
        assert!(crate::windows_win32_host::set_windows_win32_window_view_input_route(hwnd, route));
        crate::windows_win32_host::dispatch_windows_win32_window_view_click(
            hwnd,
            crate::Point { x: 20, y: 20 },
        )
        .expect("registered password route should accept focus");

        let raw = IRawElementProviderSimple::from(WindowsTextUiaProvider {
            hwnd: hwnd as isize,
        });
        let focused =
            crate::windows_win32_host::windows_win32_window_text_accessibility_snapshot(hwnd)
                .expect("focused password route should keep a redacted snapshot");
        assert!(focused.kind().is_protected());
        assert!(unsafe { raw.GetPatternProvider(UIA_ValuePatternId) }.is_err());
        assert!(unsafe { raw.GetPatternProvider(UIA_TextPatternId) }.is_err());
        let value: IValueProvider = raw.cast().expect("implemented Value interface");
        assert!(unsafe { value.Value() }.is_err());
        let replacement = windows::core::HSTRING::from("leaked");
        assert!(unsafe { value.SetValue(&replacement) }.is_err());
        let text: ITextProvider = raw.cast().expect("implemented Text interface");
        assert!(unsafe { text.DocumentRange() }.is_err());
        let snapshot =
            crate::windows_win32_host::windows_win32_window_text_accessibility_snapshot(hwnd)
                .expect("focused password route should keep a redacted snapshot");
        assert_eq!(snapshot.exposed_text(), "••••••");

        crate::windows_win32_host::clear_windows_win32_window_view_input_route(hwnd);
    }
}
