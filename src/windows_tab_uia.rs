#![allow(non_snake_case, non_upper_case_globals)]

use windows::core::{implement, Error, IUnknown, Interface, Result, BSTR, HRESULT};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Com::SAFEARRAY;
use windows::Win32::System::Ole::{SafeArrayCreateVector, SafeArrayDestroy, SafeArrayPutElement};
use windows::Win32::System::Variant::{VARIANT, VT_I4};
use windows::Win32::UI::Accessibility::{
    IRawElementProviderFragment, IRawElementProviderFragmentRoot,
    IRawElementProviderFragmentRoot_Impl, IRawElementProviderFragment_Impl,
    IRawElementProviderSimple, IRawElementProviderSimple_Impl, ISelectionItemProvider,
    ISelectionItemProvider_Impl, NavigateDirection, NavigateDirection_FirstChild,
    NavigateDirection_LastChild, NavigateDirection_NextSibling, NavigateDirection_Parent,
    NavigateDirection_PreviousSibling, ProviderOptions, ProviderOptions_ServerSideProvider,
    UIA_AutomationIdPropertyId, UIA_ClassNamePropertyId, UIA_ControlTypePropertyId,
    UIA_FrameworkIdPropertyId, UIA_HasKeyboardFocusPropertyId, UIA_IsEnabledPropertyId,
    UIA_IsKeyboardFocusablePropertyId, UIA_NamePropertyId, UIA_NativeWindowHandlePropertyId,
    UIA_PaneControlTypeId, UIA_SelectionItemIsSelectedPropertyId, UIA_SelectionItemPatternId,
    UIA_TabControlTypeId, UIA_TabItemControlTypeId, UiaAppendRuntimeId, UiaHostProviderFromHwnd,
    UiaRect, UiaReturnRawElementProvider, UiaRootObjectId, UIA_PATTERN_ID, UIA_PROPERTY_ID,
};
use windows_core::{IUnknownImpl, BOOL};

#[implement(
    IRawElementProviderSimple,
    IRawElementProviderFragment,
    IRawElementProviderFragmentRoot
)]
struct WindowsTabUiaRootProvider {
    hwnd: isize,
    tab_view: crate::WidgetId,
}

#[implement(
    IRawElementProviderSimple,
    IRawElementProviderFragment,
    ISelectionItemProvider
)]
struct WindowsTabUiaItemProvider {
    hwnd: isize,
    tab_view: crate::WidgetId,
    tab: crate::ZsTabId,
}

#[implement(IRawElementProviderSimple, IRawElementProviderFragment)]
struct WindowsTabUiaPanelProvider {
    hwnd: isize,
    tab_view: crate::WidgetId,
}

fn snapshots(hwnd: isize) -> Vec<crate::native_tab_accessibility::NativeTabAccessibilitySnapshot> {
    crate::windows_win32_host::windows_win32_window_tab_accessibility_snapshots(hwnd as _)
}

fn snapshot(
    hwnd: isize,
    tab_view: crate::WidgetId,
) -> Result<crate::native_tab_accessibility::NativeTabAccessibilitySnapshot> {
    snapshots(hwnd)
        .into_iter()
        .find(|snapshot| snapshot.tab_view == tab_view)
        .ok_or_else(element_not_available)
}

fn snapshot_and_item(
    hwnd: isize,
    tab_view: crate::WidgetId,
    tab: crate::ZsTabId,
) -> Result<(
    crate::native_tab_accessibility::NativeTabAccessibilitySnapshot,
    crate::native_tab_accessibility::NativeTabAccessibilityItem,
)> {
    let snapshot = snapshot(hwnd, tab_view)?;
    let item = snapshot
        .item(tab)
        .cloned()
        .ok_or_else(element_not_available)?;
    Ok((snapshot, item))
}

fn element_not_available() -> Error {
    Error::from_hresult(HRESULT(
        windows::Win32::UI::Accessibility::UIA_E_ELEMENTNOTAVAILABLE as i32,
    ))
}

fn invalid_operation() -> Error {
    Error::from_hresult(HRESULT(
        windows::Win32::UI::Accessibility::UIA_E_INVALIDOPERATION as i32,
    ))
}

fn not_implemented() -> Error {
    Error::from_hresult(HRESULT(0x8000_4001_u32 as i32))
}

fn out_of_memory() -> Error {
    Error::from_hresult(HRESULT(0x8007_000e_u32 as i32))
}

fn root_simple_provider(hwnd: isize, tab_view: crate::WidgetId) -> IRawElementProviderSimple {
    IRawElementProviderSimple::from(WindowsTabUiaRootProvider { hwnd, tab_view })
}

fn root_fragment_provider(hwnd: isize, tab_view: crate::WidgetId) -> IRawElementProviderFragment {
    IRawElementProviderFragment::from(WindowsTabUiaRootProvider { hwnd, tab_view })
}

fn root_fragment_provider_root(
    hwnd: isize,
    tab_view: crate::WidgetId,
) -> IRawElementProviderFragmentRoot {
    IRawElementProviderFragmentRoot::from(WindowsTabUiaRootProvider { hwnd, tab_view })
}

fn item_fragment_provider(
    hwnd: isize,
    tab_view: crate::WidgetId,
    tab: crate::ZsTabId,
) -> IRawElementProviderFragment {
    IRawElementProviderFragment::from(WindowsTabUiaItemProvider {
        hwnd,
        tab_view,
        tab,
    })
}

fn panel_fragment_provider(hwnd: isize, tab_view: crate::WidgetId) -> IRawElementProviderFragment {
    IRawElementProviderFragment::from(WindowsTabUiaPanelProvider { hwnd, tab_view })
}

fn safe_array_from_i32_slice(values: &[i32]) -> Result<*mut SAFEARRAY> {
    let len = u32::try_from(values.len()).map_err(|_| out_of_memory())?;
    let array = unsafe { SafeArrayCreateVector(VT_I4, 0, len) };
    if array.is_null() {
        return Err(out_of_memory());
    }
    for (index, value) in values.iter().enumerate() {
        let index = i32::try_from(index).map_err(|_| out_of_memory())?;
        if let Err(error) =
            unsafe { SafeArrayPutElement(array, &index, (value as *const i32).cast()) }
        {
            unsafe {
                let _ = SafeArrayDestroy(array);
            }
            return Err(error);
        }
    }
    Ok(array)
}

fn rect_in_screen(hwnd: isize, rect: crate::Rect) -> UiaRect {
    let mut point = windows_sys::Win32::Foundation::POINT {
        x: rect.x,
        y: rect.y,
    };
    unsafe {
        windows_sys::Win32::Graphics::Gdi::ClientToScreen(hwnd as _, &mut point);
    }
    UiaRect {
        left: f64::from(point.x),
        top: f64::from(point.y),
        width: f64::from(rect.width.max(0)),
        height: f64::from(rect.height.max(0)),
    }
}

fn root_bounds(
    snapshot: &crate::native_tab_accessibility::NativeTabAccessibilitySnapshot,
) -> crate::Rect {
    crate::Rect {
        x: snapshot.list_bounds.x.min(snapshot.panel_bounds.x),
        y: snapshot.list_bounds.y.min(snapshot.panel_bounds.y),
        width: snapshot
            .list_bounds
            .x
            .saturating_add(snapshot.list_bounds.width.max(0))
            .max(
                snapshot
                    .panel_bounds
                    .x
                    .saturating_add(snapshot.panel_bounds.width.max(0)),
            )
            .saturating_sub(snapshot.list_bounds.x.min(snapshot.panel_bounds.x)),
        height: snapshot
            .list_bounds
            .y
            .saturating_add(snapshot.list_bounds.height.max(0))
            .max(
                snapshot
                    .panel_bounds
                    .y
                    .saturating_add(snapshot.panel_bounds.height.max(0)),
            )
            .saturating_sub(snapshot.list_bounds.y.min(snapshot.panel_bounds.y)),
    }
}

impl IRawElementProviderSimple_Impl for WindowsTabUiaRootProvider_Impl {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider)
    }

    fn GetPatternProvider(&self, _pattern_id: UIA_PATTERN_ID) -> Result<IUnknown> {
        snapshot(self.hwnd, self.tab_view)?;
        Err(Error::empty())
    }

    fn GetPropertyValue(&self, property_id: UIA_PROPERTY_ID) -> Result<VARIANT> {
        let snapshot = snapshot(self.hwnd, self.tab_view)?;
        let value = match property_id {
            UIA_ControlTypePropertyId => VARIANT::from(UIA_TabControlTypeId.0),
            UIA_HasKeyboardFocusPropertyId => VARIANT::from(snapshot.focused_item().is_some()),
            UIA_IsEnabledPropertyId | UIA_IsKeyboardFocusablePropertyId => VARIANT::from(true),
            UIA_NativeWindowHandlePropertyId => VARIANT::from(self.hwnd as i32),
            UIA_FrameworkIdPropertyId => VARIANT::from(BSTR::from("ZSUI")),
            UIA_ClassNamePropertyId => VARIANT::from(BSTR::from("ZsuiTabList")),
            UIA_AutomationIdPropertyId => {
                VARIANT::from(BSTR::from(format!("zsui-tab-list-{}", self.tab_view.0)))
            }
            UIA_NamePropertyId => VARIANT::from(BSTR::from("Tabs")),
            _ => VARIANT::default(),
        };
        Ok(value)
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        snapshot(self.hwnd, self.tab_view)?;
        unsafe { UiaHostProviderFromHwnd(HWND(self.hwnd as *mut core::ffi::c_void)) }
    }
}

impl IRawElementProviderFragment_Impl for WindowsTabUiaRootProvider_Impl {
    fn Navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        let snapshot = snapshot(self.hwnd, self.tab_view)?;
        if direction == NavigateDirection_FirstChild {
            return snapshot
                .items
                .first()
                .map(|item| item_fragment_provider(self.hwnd, self.tab_view, item.tab()))
                .ok_or_else(Error::empty);
        }
        if direction == NavigateDirection_LastChild {
            return Ok(panel_fragment_provider(self.hwnd, self.tab_view));
        }
        Err(Error::empty())
    }

    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        snapshot(self.hwnd, self.tab_view)?;
        Err(not_implemented())
    }

    fn BoundingRectangle(&self) -> Result<UiaRect> {
        Ok(rect_in_screen(
            self.hwnd,
            root_bounds(&snapshot(self.hwnd, self.tab_view)?),
        ))
    }

    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        snapshot(self.hwnd, self.tab_view)?;
        Ok(std::ptr::null_mut())
    }

    fn SetFocus(&self) -> Result<()> {
        let snapshot = snapshot(self.hwnd, self.tab_view)?;
        let item = snapshot
            .focused_item()
            .or_else(|| snapshot.selected_item())
            .ok_or_else(invalid_operation)?;
        crate::windows_win32_host::focus_windows_win32_window_accessible_tab(
            self.hwnd as _,
            item.tab(),
        )
        .then_some(())
        .ok_or_else(element_not_available)
    }

    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        snapshot(self.hwnd, self.tab_view)?;
        Ok(root_fragment_provider_root(self.hwnd, self.tab_view))
    }
}

impl IRawElementProviderFragmentRoot_Impl for WindowsTabUiaRootProvider_Impl {
    fn ElementProviderFromPoint(&self, x: f64, y: f64) -> Result<IRawElementProviderFragment> {
        let snapshot = snapshot(self.hwnd, self.tab_view)?;
        if let Some(item) = snapshot.items.iter().find(|item| {
            let rect = rect_in_screen(self.hwnd, item.target.bounds);
            x >= rect.left
                && x < rect.left + rect.width
                && y >= rect.top
                && y < rect.top + rect.height
        }) {
            return Ok(item_fragment_provider(self.hwnd, self.tab_view, item.tab()));
        }
        let panel = rect_in_screen(self.hwnd, snapshot.panel_bounds);
        if x >= panel.left
            && x < panel.left + panel.width
            && y >= panel.top
            && y < panel.top + panel.height
        {
            return Ok(panel_fragment_provider(self.hwnd, self.tab_view));
        }
        Err(Error::empty())
    }

    fn GetFocus(&self) -> Result<IRawElementProviderFragment> {
        let snapshot = snapshot(self.hwnd, self.tab_view)?;
        snapshot
            .focused_item()
            .or_else(|| snapshot.selected_item())
            .map(|item| item_fragment_provider(self.hwnd, self.tab_view, item.tab()))
            .ok_or_else(Error::empty)
    }
}

impl IRawElementProviderSimple_Impl for WindowsTabUiaItemProvider_Impl {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider)
    }

    fn GetPatternProvider(&self, pattern_id: UIA_PATTERN_ID) -> Result<IUnknown> {
        snapshot_and_item(self.hwnd, self.tab_view, self.tab)?;
        if pattern_id == UIA_SelectionItemPatternId {
            let provider: ISelectionItemProvider = self.to_interface();
            return provider.cast();
        }
        Err(Error::empty())
    }

    fn GetPropertyValue(&self, property_id: UIA_PROPERTY_ID) -> Result<VARIANT> {
        let (_, item) = snapshot_and_item(self.hwnd, self.tab_view, self.tab)?;
        let value = match property_id {
            UIA_ControlTypePropertyId => VARIANT::from(UIA_TabItemControlTypeId.0),
            UIA_HasKeyboardFocusPropertyId => VARIANT::from(item.focused),
            UIA_IsEnabledPropertyId | UIA_IsKeyboardFocusablePropertyId => VARIANT::from(true),
            UIA_FrameworkIdPropertyId => VARIANT::from(BSTR::from("ZSUI")),
            UIA_ClassNamePropertyId => VARIANT::from(BSTR::from("ZsuiTabItem")),
            UIA_AutomationIdPropertyId => VARIANT::from(BSTR::from(format!(
                "zsui-tab-item-{}-{}",
                self.tab_view.0, self.tab.0
            ))),
            UIA_NamePropertyId => VARIANT::from(BSTR::from(item.label)),
            UIA_SelectionItemIsSelectedPropertyId => VARIANT::from(item.selected),
            _ => VARIANT::default(),
        };
        Ok(value)
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        snapshot_and_item(self.hwnd, self.tab_view, self.tab)?;
        Err(Error::empty())
    }
}

impl IRawElementProviderFragment_Impl for WindowsTabUiaItemProvider_Impl {
    fn Navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        let (snapshot, item) = snapshot_and_item(self.hwnd, self.tab_view, self.tab)?;
        if direction == NavigateDirection_Parent {
            return Ok(root_fragment_provider(self.hwnd, self.tab_view));
        }
        let index = snapshot
            .items
            .iter()
            .position(|candidate| candidate.tab() == item.tab())
            .ok_or_else(element_not_available)?;
        if direction == NavigateDirection_NextSibling {
            return snapshot.items.get(index.saturating_add(1)).map_or_else(
                || Ok(panel_fragment_provider(self.hwnd, self.tab_view)),
                |next| Ok(item_fragment_provider(self.hwnd, self.tab_view, next.tab())),
            );
        }
        if direction == NavigateDirection_PreviousSibling {
            return index
                .checked_sub(1)
                .and_then(|index| snapshot.items.get(index))
                .map(|previous| item_fragment_provider(self.hwnd, self.tab_view, previous.tab()))
                .ok_or_else(Error::empty);
        }
        Err(Error::empty())
    }

    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        snapshot_and_item(self.hwnd, self.tab_view, self.tab)?;
        safe_array_from_i32_slice(&[
            UiaAppendRuntimeId as i32,
            (self.tab_view.0 & i32::MAX as u64) as i32,
            (self.tab.0 & i32::MAX as u64) as i32,
        ])
    }

    fn BoundingRectangle(&self) -> Result<UiaRect> {
        let (_, item) = snapshot_and_item(self.hwnd, self.tab_view, self.tab)?;
        Ok(rect_in_screen(self.hwnd, item.target.bounds))
    }

    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        snapshot_and_item(self.hwnd, self.tab_view, self.tab)?;
        Ok(std::ptr::null_mut())
    }

    fn SetFocus(&self) -> Result<()> {
        snapshot_and_item(self.hwnd, self.tab_view, self.tab)?;
        crate::windows_win32_host::focus_windows_win32_window_accessible_tab(
            self.hwnd as _,
            self.tab,
        )
        .then_some(())
        .ok_or_else(element_not_available)
    }

    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        snapshot_and_item(self.hwnd, self.tab_view, self.tab)?;
        Ok(root_fragment_provider_root(self.hwnd, self.tab_view))
    }
}

impl ISelectionItemProvider_Impl for WindowsTabUiaItemProvider_Impl {
    fn Select(&self) -> Result<()> {
        snapshot_and_item(self.hwnd, self.tab_view, self.tab)?;
        crate::windows_win32_host::select_windows_win32_window_accessible_tab(
            self.hwnd as _,
            self.tab,
        )
        .then_some(())
        .ok_or_else(element_not_available)
    }

    fn AddToSelection(&self) -> Result<()> {
        self.Select()
    }

    fn RemoveFromSelection(&self) -> Result<()> {
        Err(invalid_operation())
    }

    fn IsSelected(&self) -> Result<BOOL> {
        let (_, item) = snapshot_and_item(self.hwnd, self.tab_view, self.tab)?;
        Ok(BOOL::from(item.selected))
    }

    fn SelectionContainer(&self) -> Result<IRawElementProviderSimple> {
        snapshot_and_item(self.hwnd, self.tab_view, self.tab)?;
        Ok(root_simple_provider(self.hwnd, self.tab_view))
    }
}

impl IRawElementProviderSimple_Impl for WindowsTabUiaPanelProvider_Impl {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider)
    }

    fn GetPatternProvider(&self, _pattern_id: UIA_PATTERN_ID) -> Result<IUnknown> {
        snapshot(self.hwnd, self.tab_view)?;
        Err(Error::empty())
    }

    fn GetPropertyValue(&self, property_id: UIA_PROPERTY_ID) -> Result<VARIANT> {
        let snapshot = snapshot(self.hwnd, self.tab_view)?;
        let value = match property_id {
            UIA_ControlTypePropertyId => VARIANT::from(UIA_PaneControlTypeId.0),
            UIA_HasKeyboardFocusPropertyId | UIA_IsKeyboardFocusablePropertyId => {
                VARIANT::from(false)
            }
            UIA_IsEnabledPropertyId => VARIANT::from(true),
            UIA_FrameworkIdPropertyId => VARIANT::from(BSTR::from("ZSUI")),
            UIA_ClassNamePropertyId => VARIANT::from(BSTR::from("ZsuiTabPanel")),
            UIA_AutomationIdPropertyId => {
                VARIANT::from(BSTR::from(format!("zsui-tab-panel-{}", self.tab_view.0)))
            }
            UIA_NamePropertyId => VARIANT::from(BSTR::from(
                snapshot
                    .selected_item()
                    .map_or("Tab panel", |item| item.label.as_str()),
            )),
            _ => VARIANT::default(),
        };
        Ok(value)
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        snapshot(self.hwnd, self.tab_view)?;
        Err(Error::empty())
    }
}

impl IRawElementProviderFragment_Impl for WindowsTabUiaPanelProvider_Impl {
    fn Navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        let snapshot = snapshot(self.hwnd, self.tab_view)?;
        if direction == NavigateDirection_Parent {
            return Ok(root_fragment_provider(self.hwnd, self.tab_view));
        }
        if direction == NavigateDirection_PreviousSibling {
            return snapshot
                .items
                .last()
                .map(|item| item_fragment_provider(self.hwnd, self.tab_view, item.tab()))
                .ok_or_else(Error::empty);
        }
        Err(Error::empty())
    }

    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        snapshot(self.hwnd, self.tab_view)?;
        safe_array_from_i32_slice(&[
            UiaAppendRuntimeId as i32,
            (self.tab_view.0 & i32::MAX as u64) as i32,
            i32::MAX,
        ])
    }

    fn BoundingRectangle(&self) -> Result<UiaRect> {
        Ok(rect_in_screen(
            self.hwnd,
            snapshot(self.hwnd, self.tab_view)?.panel_bounds,
        ))
    }

    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        snapshot(self.hwnd, self.tab_view)?;
        Ok(std::ptr::null_mut())
    }

    fn SetFocus(&self) -> Result<()> {
        Err(invalid_operation())
    }

    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        snapshot(self.hwnd, self.tab_view)?;
        Ok(root_fragment_provider_root(self.hwnd, self.tab_view))
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
    let snapshot =
        crate::windows_win32_host::windows_win32_window_tab_accessibility_snapshots(hwnd)
            .into_iter()
            .next()?;
    let provider = root_simple_provider(hwnd as isize, snapshot.tab_view);
    let result = unsafe {
        UiaReturnRawElementProvider(HWND(hwnd.cast()), WPARAM(wparam), LPARAM(lparam), &provider)
    };
    Some(result.0)
}

#[cfg(all(not(feature = "text-input-core"), not(feature = "menu-flyout")))]
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

    #[test]
    fn tab_providers_expose_fragment_and_selection_item_contracts() {
        let root = IRawElementProviderSimple::from(WindowsTabUiaRootProvider {
            hwnd: 1,
            tab_view: crate::WidgetId(10),
        });
        assert!(root.cast::<IRawElementProviderFragment>().is_ok());
        assert!(root.cast::<IRawElementProviderFragmentRoot>().is_ok());

        let item = IRawElementProviderSimple::from(WindowsTabUiaItemProvider {
            hwnd: 1,
            tab_view: crate::WidgetId(10),
            tab: crate::ZsTabId::new(11),
        });
        assert!(item.cast::<IRawElementProviderFragment>().is_ok());
        assert!(item.cast::<ISelectionItemProvider>().is_ok());

        let panel = IRawElementProviderSimple::from(WindowsTabUiaPanelProvider {
            hwnd: 1,
            tab_view: crate::WidgetId(10),
        });
        assert!(panel.cast::<IRawElementProviderFragment>().is_ok());
    }
}
