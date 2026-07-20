#![allow(non_snake_case, non_upper_case_globals)]

use windows::core::{implement, Error, IUnknown, Interface, Result, BSTR, HRESULT};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Com::SAFEARRAY;
use windows::Win32::System::Ole::{SafeArrayCreateVector, SafeArrayDestroy, SafeArrayPutElement};
use windows::Win32::System::Variant::{VARIANT, VT_I4};
use windows::Win32::UI::Accessibility::{
    ExpandCollapseState, ExpandCollapseState_Collapsed, ExpandCollapseState_Expanded,
    ExpandCollapseState_LeafNode, IExpandCollapseProvider, IExpandCollapseProvider_Impl,
    IInvokeProvider, IInvokeProvider_Impl, IRawElementProviderFragment,
    IRawElementProviderFragmentRoot, IRawElementProviderFragmentRoot_Impl,
    IRawElementProviderFragment_Impl, IRawElementProviderSimple, IRawElementProviderSimple_Impl,
    IToggleProvider, IToggleProvider_Impl, NavigateDirection, NavigateDirection_FirstChild,
    NavigateDirection_LastChild, NavigateDirection_NextSibling, NavigateDirection_Parent,
    NavigateDirection_PreviousSibling, ProviderOptions, ProviderOptions_ServerSideProvider,
    ToggleState, ToggleState_Off, ToggleState_On, UIA_AutomationIdPropertyId,
    UIA_ClassNamePropertyId, UIA_ControlTypePropertyId,
    UIA_ExpandCollapseExpandCollapseStatePropertyId, UIA_ExpandCollapsePatternId,
    UIA_FrameworkIdPropertyId, UIA_HasKeyboardFocusPropertyId, UIA_InvokePatternId,
    UIA_IsEnabledPropertyId, UIA_IsKeyboardFocusablePropertyId, UIA_MenuControlTypeId,
    UIA_MenuItemControlTypeId, UIA_NamePropertyId, UIA_NativeWindowHandlePropertyId,
    UIA_TogglePatternId, UIA_ToggleToggleStatePropertyId, UiaAppendRuntimeId,
    UiaHostProviderFromHwnd, UiaRect, UiaReturnRawElementProvider, UiaRootObjectId, UIA_PATTERN_ID,
    UIA_PROPERTY_ID,
};
use windows_core::IUnknownImpl;

#[implement(
    IRawElementProviderSimple,
    IRawElementProviderFragment,
    IRawElementProviderFragmentRoot
)]
struct WindowsMenuFlyoutUiaRootProvider {
    hwnd: isize,
}

#[implement(
    IRawElementProviderSimple,
    IRawElementProviderFragment,
    IInvokeProvider,
    IToggleProvider,
    IExpandCollapseProvider
)]
struct WindowsMenuFlyoutUiaItemProvider {
    hwnd: isize,
    path: crate::ZsMenuFlyoutPath,
}

impl WindowsMenuFlyoutUiaRootProvider_Impl {
    fn snapshot(
        &self,
    ) -> Result<crate::native_menu_accessibility::NativeMenuFlyoutAccessibilitySnapshot> {
        crate::windows_win32_host::windows_win32_window_menu_flyout_accessibility_snapshot(
            self.hwnd as _,
        )
        .ok_or_else(element_not_available)
    }
}

impl WindowsMenuFlyoutUiaItemProvider_Impl {
    fn snapshot_and_item(
        &self,
    ) -> Result<(
        crate::native_menu_accessibility::NativeMenuFlyoutAccessibilitySnapshot,
        crate::native_menu_accessibility::NativeMenuFlyoutAccessibilityItem,
    )> {
        let snapshot =
            crate::windows_win32_host::windows_win32_window_menu_flyout_accessibility_snapshot(
                self.hwnd as _,
            )
            .ok_or_else(element_not_available)?;
        let item = snapshot
            .item(self.path)
            .cloned()
            .ok_or_else(element_not_available)?;
        Ok((snapshot, item))
    }
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

fn root_simple_provider(hwnd: isize) -> IRawElementProviderSimple {
    IRawElementProviderSimple::from(WindowsMenuFlyoutUiaRootProvider { hwnd })
}

fn root_fragment_provider(hwnd: isize) -> IRawElementProviderFragment {
    IRawElementProviderFragment::from(WindowsMenuFlyoutUiaRootProvider { hwnd })
}

fn root_fragment_provider_root(hwnd: isize) -> IRawElementProviderFragmentRoot {
    IRawElementProviderFragmentRoot::from(WindowsMenuFlyoutUiaRootProvider { hwnd })
}

fn item_fragment_provider(
    hwnd: isize,
    path: crate::ZsMenuFlyoutPath,
) -> IRawElementProviderFragment {
    IRawElementProviderFragment::from(WindowsMenuFlyoutUiaItemProvider { hwnd, path })
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

fn path_runtime_id(path: crate::ZsMenuFlyoutPath) -> i32 {
    let mut value = i32::try_from(path.level()).unwrap_or(i32::MAX);
    let mut cursor = Some(path);
    while let Some(current) = cursor {
        value = value
            .wrapping_mul(257)
            .wrapping_add(i32::try_from(current.item()).unwrap_or(i32::MAX));
        cursor = current.parent();
    }
    value
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

fn direct_children<'a>(
    snapshot: &'a crate::native_menu_accessibility::NativeMenuFlyoutAccessibilitySnapshot,
    parent: Option<crate::ZsMenuFlyoutPath>,
) -> impl DoubleEndedIterator<
    Item = &'a crate::native_menu_accessibility::NativeMenuFlyoutAccessibilityItem,
> {
    snapshot
        .items
        .iter()
        .filter(move |item| item.path().parent() == parent)
}

impl IRawElementProviderSimple_Impl for WindowsMenuFlyoutUiaRootProvider_Impl {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider)
    }

    fn GetPatternProvider(&self, _pattern_id: UIA_PATTERN_ID) -> Result<IUnknown> {
        self.snapshot()?;
        Err(Error::empty())
    }

    fn GetPropertyValue(&self, property_id: UIA_PROPERTY_ID) -> Result<VARIANT> {
        let snapshot = self.snapshot()?;
        let value = match property_id {
            UIA_ControlTypePropertyId => VARIANT::from(UIA_MenuControlTypeId.0),
            UIA_HasKeyboardFocusPropertyId => VARIANT::from(snapshot.highlighted_item().is_some()),
            UIA_IsEnabledPropertyId | UIA_IsKeyboardFocusablePropertyId => VARIANT::from(true),
            UIA_NativeWindowHandlePropertyId => VARIANT::from(self.hwnd as i32),
            UIA_FrameworkIdPropertyId => VARIANT::from(BSTR::from("ZSUI")),
            UIA_ClassNamePropertyId => VARIANT::from(BSTR::from("ZsuiMenuFlyout")),
            UIA_AutomationIdPropertyId => VARIANT::from(BSTR::from(format!(
                "zsui-menu-flyout-{}",
                snapshot.widget.0
            ))),
            UIA_NamePropertyId => VARIANT::from(BSTR::from("MenuFlyout")),
            _ => VARIANT::default(),
        };
        Ok(value)
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        self.snapshot()?;
        unsafe { UiaHostProviderFromHwnd(HWND(self.hwnd as *mut core::ffi::c_void)) }
    }
}

impl IRawElementProviderFragment_Impl for WindowsMenuFlyoutUiaRootProvider_Impl {
    fn Navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        let snapshot = self.snapshot()?;
        let item = if direction == NavigateDirection_FirstChild {
            direct_children(&snapshot, None).next()
        } else if direction == NavigateDirection_LastChild {
            direct_children(&snapshot, None).next_back()
        } else {
            None
        };
        item.map(|item| item_fragment_provider(self.hwnd, item.path()))
            .ok_or_else(Error::empty)
    }

    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        self.snapshot()?;
        Err(not_implemented())
    }

    fn BoundingRectangle(&self) -> Result<UiaRect> {
        Ok(rect_in_screen(self.hwnd, self.snapshot()?.bounds))
    }

    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        self.snapshot()?;
        Ok(std::ptr::null_mut())
    }

    fn SetFocus(&self) -> Result<()> {
        let snapshot = self.snapshot()?;
        let Some(item) = snapshot.highlighted_item() else {
            return Err(invalid_operation());
        };
        crate::windows_win32_host::focus_windows_win32_window_accessible_menu_flyout_item(
            self.hwnd as _,
            item.path(),
        )
        .then_some(())
        .ok_or_else(element_not_available)
    }

    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        self.snapshot()?;
        Ok(root_fragment_provider_root(self.hwnd))
    }
}

impl IRawElementProviderFragmentRoot_Impl for WindowsMenuFlyoutUiaRootProvider_Impl {
    fn ElementProviderFromPoint(&self, x: f64, y: f64) -> Result<IRawElementProviderFragment> {
        let snapshot = self.snapshot()?;
        snapshot
            .items
            .iter()
            .find(|item| {
                let rect = rect_in_screen(self.hwnd, item.target.bounds);
                x >= rect.left
                    && x < rect.left + rect.width
                    && y >= rect.top
                    && y < rect.top + rect.height
            })
            .map(|item| item_fragment_provider(self.hwnd, item.path()))
            .ok_or_else(Error::empty)
    }

    fn GetFocus(&self) -> Result<IRawElementProviderFragment> {
        let snapshot = self.snapshot()?;
        snapshot
            .highlighted_item()
            .map(|item| item_fragment_provider(self.hwnd, item.path()))
            .ok_or_else(Error::empty)
    }
}

impl IRawElementProviderSimple_Impl for WindowsMenuFlyoutUiaItemProvider_Impl {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider)
    }

    fn GetPatternProvider(&self, pattern_id: UIA_PATTERN_ID) -> Result<IUnknown> {
        let (_, item) = self.snapshot_and_item()?;
        if pattern_id == UIA_InvokePatternId {
            let provider: IInvokeProvider = self.to_interface();
            return provider.cast();
        }
        if pattern_id == UIA_TogglePatternId && item.checked() == Some(true) {
            let provider: IToggleProvider = self.to_interface();
            return provider.cast();
        }
        if pattern_id == UIA_ExpandCollapsePatternId && item.expanded().is_some() {
            let provider: IExpandCollapseProvider = self.to_interface();
            return provider.cast();
        }
        Err(Error::empty())
    }

    fn GetPropertyValue(&self, property_id: UIA_PROPERTY_ID) -> Result<VARIANT> {
        let (_, item) = self.snapshot_and_item()?;
        let value = match property_id {
            UIA_ControlTypePropertyId => VARIANT::from(UIA_MenuItemControlTypeId.0),
            UIA_HasKeyboardFocusPropertyId => VARIANT::from(item.highlighted()),
            UIA_IsEnabledPropertyId => VARIANT::from(item.enabled),
            UIA_IsKeyboardFocusablePropertyId => VARIANT::from(item.enabled),
            UIA_FrameworkIdPropertyId => VARIANT::from(BSTR::from("ZSUI")),
            UIA_ClassNamePropertyId => VARIANT::from(BSTR::from(if item.expanded().is_some() {
                "ZsuiSubmenuItem"
            } else if item.checked() == Some(true) {
                "ZsuiCheckedMenuItem"
            } else {
                "ZsuiMenuItem"
            })),
            UIA_AutomationIdPropertyId => VARIANT::from(BSTR::from(format!(
                "zsui-menu-flyout-item-{}",
                path_runtime_id(item.path())
            ))),
            UIA_NamePropertyId => VARIANT::from(BSTR::from(item.label)),
            UIA_ToggleToggleStatePropertyId if item.checked().is_some() => {
                VARIANT::from(if item.checked() == Some(true) {
                    ToggleState_On.0
                } else {
                    ToggleState_Off.0
                })
            }
            UIA_ExpandCollapseExpandCollapseStatePropertyId if item.expanded().is_some() => {
                VARIANT::from(if item.expanded() == Some(true) {
                    ExpandCollapseState_Expanded.0
                } else {
                    ExpandCollapseState_Collapsed.0
                })
            }
            _ => VARIANT::default(),
        };
        Ok(value)
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        self.snapshot_and_item()?;
        Err(Error::empty())
    }
}

impl IRawElementProviderFragment_Impl for WindowsMenuFlyoutUiaItemProvider_Impl {
    fn Navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        let (snapshot, item) = self.snapshot_and_item()?;
        if direction == NavigateDirection_Parent {
            return item
                .path()
                .parent()
                .map(|parent| item_fragment_provider(self.hwnd, parent))
                .map_or_else(
                    || Ok(root_fragment_provider(self.hwnd)),
                    |provider| Ok(provider),
                );
        }
        if direction == NavigateDirection_FirstChild {
            return direct_children(&snapshot, Some(item.path()))
                .next()
                .map(|child| item_fragment_provider(self.hwnd, child.path()))
                .ok_or_else(Error::empty);
        }
        if direction == NavigateDirection_LastChild {
            return direct_children(&snapshot, Some(item.path()))
                .next_back()
                .map(|child| item_fragment_provider(self.hwnd, child.path()))
                .ok_or_else(Error::empty);
        }
        let parent = item.path().parent();
        let siblings = direct_children(&snapshot, parent).collect::<Vec<_>>();
        let Some(index) = siblings
            .iter()
            .position(|sibling| sibling.path() == item.path())
        else {
            return Err(element_not_available());
        };
        let sibling = if direction == NavigateDirection_NextSibling {
            siblings.get(index.saturating_add(1))
        } else if direction == NavigateDirection_PreviousSibling {
            index.checked_sub(1).and_then(|index| siblings.get(index))
        } else {
            None
        };
        sibling
            .map(|sibling| item_fragment_provider(self.hwnd, sibling.path()))
            .ok_or_else(Error::empty)
    }

    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        let (snapshot, item) = self.snapshot_and_item()?;
        safe_array_from_i32_slice(&[
            UiaAppendRuntimeId as i32,
            (snapshot.widget.0 & i32::MAX as u64) as i32,
            path_runtime_id(item.path()),
        ])
    }

    fn BoundingRectangle(&self) -> Result<UiaRect> {
        let (_, item) = self.snapshot_and_item()?;
        Ok(rect_in_screen(self.hwnd, item.target.bounds))
    }

    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        self.snapshot_and_item()?;
        Ok(std::ptr::null_mut())
    }

    fn SetFocus(&self) -> Result<()> {
        self.snapshot_and_item()?;
        crate::windows_win32_host::focus_windows_win32_window_accessible_menu_flyout_item(
            self.hwnd as _,
            self.path,
        )
        .then_some(())
        .ok_or_else(element_not_available)
    }

    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        self.snapshot_and_item()?;
        Ok(root_fragment_provider_root(self.hwnd))
    }
}

impl IInvokeProvider_Impl for WindowsMenuFlyoutUiaItemProvider_Impl {
    fn Invoke(&self) -> Result<()> {
        self.snapshot_and_item()?;
        crate::windows_win32_host::invoke_windows_win32_window_accessible_menu_flyout_item(
            self.hwnd as _,
            self.path,
        )
        .then_some(())
        .ok_or_else(element_not_available)
    }
}

impl IToggleProvider_Impl for WindowsMenuFlyoutUiaItemProvider_Impl {
    fn Toggle(&self) -> Result<()> {
        let (_, item) = self.snapshot_and_item()?;
        if item.checked() != Some(true) {
            return Err(invalid_operation());
        }
        IInvokeProvider_Impl::Invoke(self)
    }

    fn ToggleState(&self) -> Result<ToggleState> {
        let (_, item) = self.snapshot_and_item()?;
        item.checked()
            .map(|checked| {
                if checked {
                    ToggleState_On
                } else {
                    ToggleState_Off
                }
            })
            .ok_or_else(invalid_operation)
    }
}

impl IExpandCollapseProvider_Impl for WindowsMenuFlyoutUiaItemProvider_Impl {
    fn Expand(&self) -> Result<()> {
        let (_, item) = self.snapshot_and_item()?;
        if item.expanded().is_none() {
            return Err(invalid_operation());
        }
        crate::windows_win32_host::set_windows_win32_window_accessible_menu_flyout_item_expanded(
            self.hwnd as _,
            self.path,
            true,
        )
        .then_some(())
        .ok_or_else(element_not_available)
    }

    fn Collapse(&self) -> Result<()> {
        let (_, item) = self.snapshot_and_item()?;
        if item.expanded().is_none() {
            return Err(invalid_operation());
        }
        crate::windows_win32_host::set_windows_win32_window_accessible_menu_flyout_item_expanded(
            self.hwnd as _,
            self.path,
            false,
        )
        .then_some(())
        .ok_or_else(element_not_available)
    }

    fn ExpandCollapseState(&self) -> Result<ExpandCollapseState> {
        let (_, item) = self.snapshot_and_item()?;
        Ok(match item.expanded() {
            Some(true) => ExpandCollapseState_Expanded,
            Some(false) => ExpandCollapseState_Collapsed,
            None => ExpandCollapseState_LeafNode,
        })
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
    crate::windows_win32_host::windows_win32_window_menu_flyout_accessibility_snapshot(hwnd)?;
    let provider = root_simple_provider(hwnd as isize);
    let result = unsafe {
        UiaReturnRawElementProvider(HWND(hwnd.cast()), WPARAM(wparam), LPARAM(lparam), &provider)
    };
    Some(result.0)
}

#[cfg(not(feature = "text-input-core"))]
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
    use crate::View;

    #[test]
    fn paths_produce_distinct_stable_fragment_runtime_ids() {
        let root = crate::ZsMenuFlyoutPath::root(2);
        let child = root.descendant(0).expect("nested MenuFlyout path");
        let grandchild = child.descendant(0).expect("third MenuFlyout level");
        assert_ne!(path_runtime_id(root), path_runtime_id(child));
        assert_ne!(path_runtime_id(child), path_runtime_id(grandchild));
        assert_eq!(path_runtime_id(child), path_runtime_id(child));
    }

    #[test]
    fn providers_expose_fragment_and_menu_patterns() {
        let root = IRawElementProviderSimple::from(WindowsMenuFlyoutUiaRootProvider { hwnd: 1 });
        assert!(root.cast::<IRawElementProviderFragment>().is_ok());
        assert!(root.cast::<IRawElementProviderFragmentRoot>().is_ok());

        let item = IRawElementProviderSimple::from(WindowsMenuFlyoutUiaItemProvider {
            hwnd: 1,
            path: crate::ZsMenuFlyoutPath::root(0),
        });
        assert!(item.cast::<IRawElementProviderFragment>().is_ok());
        assert!(item.cast::<IInvokeProvider>().is_ok());
        assert!(item.cast::<IToggleProvider>().is_ok());
        assert!(item.cast::<IExpandCollapseProvider>().is_ok());
    }

    #[test]
    fn win32_fragment_provider_reads_and_expands_the_shared_menu_state() {
        let _guard = crate::windows_win32_host::windows_win32_view_input_route_test_lock();
        let hwnd = 0x5a82isize as windows_sys::Win32::Foundation::HWND;
        let presenter = crate::WidgetId::new(0x5a82);
        let target = crate::WidgetId::new(0x5a83);
        let mut menu = crate::MenuSpec::new();
        menu.items.push(
            crate::MenuItemSpec::command(
                "自动保存 / Auto save",
                crate::Command::custom("auto-save"),
            )
            .checked(true),
        );
        menu = menu.submenu(
            "更多 / More",
            crate::MenuSpec::new().item("复制 / Copy", crate::Command::custom("copy")),
        );
        let mut view = crate::menu_flyout(
            presenter,
            true,
            target,
            menu,
            crate::button("菜单 / Menu").id(target),
        );
        view.layout(&mut crate::ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 400,
            },
            crate::Dpi::standard(),
        ));
        let interaction = view.interaction_plan();
        let mut paint = crate::ViewPaintCx::new(crate::Dpi::standard());
        view.paint(&mut paint);
        let draw_plan = paint.into_plan();
        crate::windows_win32_host::clear_windows_win32_window_view_input_routes();
        crate::windows_win32_host::clear_windows_win32_window_draw_plans();
        assert!(
            crate::windows_win32_host::set_windows_win32_window_view_input_route(
                hwnd,
                crate::windows_win32_host::WindowsWin32ViewInputRoute::new(interaction, view),
            )
        );
        assert!(crate::windows_win32_host::set_windows_win32_window_draw_plan(hwnd, draw_plan,));

        let snapshot =
            crate::windows_win32_host::windows_win32_window_menu_flyout_accessibility_snapshot(
                hwnd,
            )
            .expect("open MenuFlyout UIA snapshot");
        let checked = snapshot
            .items
            .iter()
            .find(|item| item.checked() == Some(true))
            .expect("checked MenuFlyout item");
        let submenu = snapshot
            .items
            .iter()
            .find(|item| item.expanded().is_some())
            .expect("submenu MenuFlyout item");
        assert_eq!(checked.label, "自动保存 / Auto save");
        assert_eq!(submenu.label, "更多 / More");
        assert_eq!(submenu.expanded(), Some(false));

        let raw = IRawElementProviderSimple::from(WindowsMenuFlyoutUiaItemProvider {
            hwnd: hwnd as isize,
            path: submenu.path(),
        });
        let pattern = unsafe { raw.GetPatternProvider(UIA_ExpandCollapsePatternId) }
            .expect("submenu should expose the UIA ExpandCollapse pattern");
        let expand: IExpandCollapseProvider =
            pattern.cast().expect("ExpandCollapse pattern interface");
        unsafe { expand.Expand() }.expect("UIA should expand the shared submenu state");
        let expanded =
            crate::windows_win32_host::windows_win32_window_menu_flyout_accessibility_snapshot(
                hwnd,
            )
            .and_then(|snapshot| snapshot.item(submenu.path()).cloned())
            .expect("expanded submenu snapshot");
        assert_eq!(expanded.expanded(), Some(true));

        crate::windows_win32_host::clear_windows_win32_window_view_input_routes();
        crate::windows_win32_host::clear_windows_win32_window_draw_plans();
    }
}
