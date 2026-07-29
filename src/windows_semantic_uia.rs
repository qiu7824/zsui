#![allow(non_snake_case, non_upper_case_globals)]

use windows::core::{implement, Error, IUnknown, Interface, Result, BSTR, HRESULT};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Com::SAFEARRAY;
use windows::Win32::System::Ole::{SafeArrayCreateVector, SafeArrayDestroy, SafeArrayPutElement};
use windows::Win32::System::Variant::{VARIANT, VT_I4};
use windows::Win32::UI::Accessibility::{
    Assertive as LiveAssertive, IInvokeProvider, IInvokeProvider_Impl, IRawElementProviderFragment,
    IRawElementProviderFragmentRoot, IRawElementProviderFragmentRoot_Impl,
    IRawElementProviderFragment_Impl, IRawElementProviderSimple, IRawElementProviderSimple_Impl,
    ISelectionItemProvider, ISelectionItemProvider_Impl, NavigateDirection,
    NavigateDirection_FirstChild, NavigateDirection_LastChild, NavigateDirection_NextSibling,
    NavigateDirection_Parent, NavigateDirection_PreviousSibling, Polite as LivePolite,
    ProviderOptions, ProviderOptions_ServerSideProvider, UIA_AutomationIdPropertyId,
    UIA_ButtonControlTypeId, UIA_ClassNamePropertyId, UIA_ComboBoxControlTypeId,
    UIA_ControlTypePropertyId, UIA_CustomControlTypeId, UIA_DataGridControlTypeId,
    UIA_EditControlTypeId, UIA_FrameworkIdPropertyId, UIA_GroupControlTypeId,
    UIA_HasKeyboardFocusPropertyId, UIA_HeaderControlTypeId, UIA_HelpTextPropertyId,
    UIA_ImageControlTypeId, UIA_InvokePatternId, UIA_IsEnabledPropertyId,
    UIA_IsKeyboardFocusablePropertyId, UIA_ListControlTypeId, UIA_ListItemControlTypeId,
    UIA_LiveSettingPropertyId, UIA_NamePropertyId, UIA_NativeWindowHandlePropertyId,
    UIA_PaneControlTypeId, UIA_ProgressBarControlTypeId, UIA_SelectionItemIsSelectedPropertyId,
    UIA_SelectionItemPatternId, UIA_SliderControlTypeId, UIA_SpinnerControlTypeId,
    UIA_TabControlTypeId, UIA_TabItemControlTypeId, UIA_TextControlTypeId, UIA_TreeControlTypeId,
    UiaAppendRuntimeId, UiaHostProviderFromHwnd, UiaRect, UiaReturnRawElementProvider,
    UiaRootObjectId, UIA_PATTERN_ID, UIA_PROPERTY_ID,
};
use windows_core::{IUnknownImpl, BOOL};

#[implement(
    IRawElementProviderSimple,
    IRawElementProviderFragment,
    IRawElementProviderFragmentRoot
)]
struct WindowsSemanticUiaRootProvider {
    hwnd: isize,
}

#[implement(
    IRawElementProviderSimple,
    IRawElementProviderFragment,
    IInvokeProvider,
    ISelectionItemProvider
)]
struct WindowsSemanticUiaNodeProvider {
    hwnd: isize,
    widget: crate::WidgetId,
}

fn nodes(hwnd: isize) -> Result<Vec<crate::ZsAccessibilityNode>> {
    let nodes =
        crate::windows_win32_host::windows_win32_window_semantic_accessibility_nodes(hwnd as _);
    (!nodes.is_empty())
        .then_some(nodes)
        .ok_or_else(element_not_available)
}

fn node(hwnd: isize, widget: crate::WidgetId) -> Result<crate::ZsAccessibilityNode> {
    nodes(hwnd)?
        .into_iter()
        .find(|node| node.widget == widget)
        .ok_or_else(element_not_available)
}

fn root_nodes(nodes: &[crate::ZsAccessibilityNode]) -> Vec<crate::WidgetId> {
    nodes
        .iter()
        .filter(|node| {
            node.parent.is_none()
                || node
                    .parent
                    .is_some_and(|parent| !nodes.iter().any(|node| node.widget == parent))
        })
        .map(|node| node.widget)
        .collect()
}

fn child_nodes(
    nodes: &[crate::ZsAccessibilityNode],
    parent: crate::WidgetId,
) -> Vec<crate::WidgetId> {
    nodes
        .iter()
        .filter(|node| node.parent == Some(parent))
        .map(|node| node.widget)
        .collect()
}

fn siblings(
    nodes: &[crate::ZsAccessibilityNode],
    item: &crate::ZsAccessibilityNode,
) -> Vec<crate::WidgetId> {
    item.parent
        .map_or_else(|| root_nodes(nodes), |parent| child_nodes(nodes, parent))
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
    IRawElementProviderSimple::from(WindowsSemanticUiaRootProvider { hwnd })
}

fn root_fragment_provider(hwnd: isize) -> IRawElementProviderFragment {
    IRawElementProviderFragment::from(WindowsSemanticUiaRootProvider { hwnd })
}

fn root_fragment_provider_root(hwnd: isize) -> IRawElementProviderFragmentRoot {
    IRawElementProviderFragmentRoot::from(WindowsSemanticUiaRootProvider { hwnd })
}

fn node_fragment_provider(hwnd: isize, widget: crate::WidgetId) -> IRawElementProviderFragment {
    IRawElementProviderFragment::from(WindowsSemanticUiaNodeProvider { hwnd, widget })
}

fn node_simple_provider(hwnd: isize, widget: crate::WidgetId) -> IRawElementProviderSimple {
    IRawElementProviderSimple::from(WindowsSemanticUiaNodeProvider { hwnd, widget })
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

fn root_bounds(nodes: &[crate::ZsAccessibilityNode]) -> crate::Rect {
    let left = nodes.iter().map(|node| node.bounds.x).min().unwrap_or(0);
    let top = nodes.iter().map(|node| node.bounds.y).min().unwrap_or(0);
    let right = nodes
        .iter()
        .map(|node| node.bounds.x.saturating_add(node.bounds.width.max(0)))
        .max()
        .unwrap_or(left);
    let bottom = nodes
        .iter()
        .map(|node| node.bounds.y.saturating_add(node.bounds.height.max(0)))
        .max()
        .unwrap_or(top);
    crate::Rect {
        x: left,
        y: top,
        width: right.saturating_sub(left),
        height: bottom.saturating_sub(top),
    }
}

fn control_type(role: crate::ZsAccessibilityRole) -> i32 {
    match role {
        crate::ZsAccessibilityRole::Button => UIA_ButtonControlTypeId.0,
        crate::ZsAccessibilityRole::ColorWell => UIA_CustomControlTypeId.0,
        crate::ZsAccessibilityRole::ComboBox => UIA_ComboBoxControlTypeId.0,
        crate::ZsAccessibilityRole::Grid => UIA_DataGridControlTypeId.0,
        crate::ZsAccessibilityRole::Heading => UIA_HeaderControlTypeId.0,
        crate::ZsAccessibilityRole::Image => UIA_ImageControlTypeId.0,
        crate::ZsAccessibilityRole::List => UIA_ListControlTypeId.0,
        crate::ZsAccessibilityRole::ListItem => UIA_ListItemControlTypeId.0,
        crate::ZsAccessibilityRole::ProgressBar => UIA_ProgressBarControlTypeId.0,
        crate::ZsAccessibilityRole::Slider => UIA_SliderControlTypeId.0,
        crate::ZsAccessibilityRole::SpinButton => UIA_SpinnerControlTypeId.0,
        crate::ZsAccessibilityRole::Tab => UIA_TabItemControlTypeId.0,
        crate::ZsAccessibilityRole::TabList => UIA_TabControlTypeId.0,
        crate::ZsAccessibilityRole::Text => UIA_TextControlTypeId.0,
        crate::ZsAccessibilityRole::TextBox
        | crate::ZsAccessibilityRole::DatePicker
        | crate::ZsAccessibilityRole::TimePicker => UIA_EditControlTypeId.0,
        crate::ZsAccessibilityRole::Tree => UIA_TreeControlTypeId.0,
        crate::ZsAccessibilityRole::Application => UIA_PaneControlTypeId.0,
        crate::ZsAccessibilityRole::Article
        | crate::ZsAccessibilityRole::Complementary
        | crate::ZsAccessibilityRole::Dialog
        | crate::ZsAccessibilityRole::Form
        | crate::ZsAccessibilityRole::Group
        | crate::ZsAccessibilityRole::Log
        | crate::ZsAccessibilityRole::Main
        | crate::ZsAccessibilityRole::Navigation
        | crate::ZsAccessibilityRole::Region
        | crate::ZsAccessibilityRole::Status
        | crate::ZsAccessibilityRole::TabPanel => UIA_GroupControlTypeId.0,
    }
}

fn keyboard_focusable(role: crate::ZsAccessibilityRole) -> bool {
    matches!(
        role,
        crate::ZsAccessibilityRole::Button
            | crate::ZsAccessibilityRole::ColorWell
            | crate::ZsAccessibilityRole::ComboBox
            | crate::ZsAccessibilityRole::DatePicker
            | crate::ZsAccessibilityRole::ListItem
            | crate::ZsAccessibilityRole::Slider
            | crate::ZsAccessibilityRole::SpinButton
            | crate::ZsAccessibilityRole::Tab
            | crate::ZsAccessibilityRole::TextBox
            | crate::ZsAccessibilityRole::TimePicker
    )
}

impl IRawElementProviderSimple_Impl for WindowsSemanticUiaRootProvider_Impl {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider)
    }

    fn GetPatternProvider(&self, _pattern_id: UIA_PATTERN_ID) -> Result<IUnknown> {
        nodes(self.hwnd)?;
        Err(Error::empty())
    }

    fn GetPropertyValue(&self, property_id: UIA_PROPERTY_ID) -> Result<VARIANT> {
        let nodes = nodes(self.hwnd)?;
        let value = match property_id {
            UIA_ControlTypePropertyId => VARIANT::from(UIA_PaneControlTypeId.0),
            UIA_HasKeyboardFocusPropertyId | UIA_IsKeyboardFocusablePropertyId => {
                VARIANT::from(false)
            }
            UIA_IsEnabledPropertyId => VARIANT::from(true),
            UIA_NativeWindowHandlePropertyId => VARIANT::from(self.hwnd as i32),
            UIA_FrameworkIdPropertyId => VARIANT::from(BSTR::from("ZSUI")),
            UIA_ClassNamePropertyId => VARIANT::from(BSTR::from("ZsuiSemanticRoot")),
            UIA_AutomationIdPropertyId => VARIANT::from(BSTR::from("zsui-semantic-root")),
            UIA_NamePropertyId => VARIANT::from(BSTR::from(
                root_nodes(&nodes)
                    .first()
                    .and_then(|widget| nodes.iter().find(|node| node.widget == *widget))
                    .and_then(|node| node.label.as_deref())
                    .unwrap_or("ZSUI content"),
            )),
            _ => VARIANT::default(),
        };
        Ok(value)
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        nodes(self.hwnd)?;
        unsafe { UiaHostProviderFromHwnd(HWND(self.hwnd as *mut core::ffi::c_void)) }
    }
}

impl IRawElementProviderFragment_Impl for WindowsSemanticUiaRootProvider_Impl {
    fn Navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        let nodes = nodes(self.hwnd)?;
        let roots = root_nodes(&nodes);
        let widget = if direction == NavigateDirection_FirstChild {
            roots.first()
        } else if direction == NavigateDirection_LastChild {
            roots.last()
        } else {
            None
        };
        widget
            .copied()
            .map(|widget| node_fragment_provider(self.hwnd, widget))
            .ok_or_else(Error::empty)
    }

    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        nodes(self.hwnd)?;
        Err(not_implemented())
    }

    fn BoundingRectangle(&self) -> Result<UiaRect> {
        let nodes = nodes(self.hwnd)?;
        Ok(rect_in_screen(self.hwnd, root_bounds(&nodes)))
    }

    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        nodes(self.hwnd)?;
        Ok(std::ptr::null_mut())
    }

    fn SetFocus(&self) -> Result<()> {
        let nodes = nodes(self.hwnd)?;
        nodes
            .iter()
            .find(|node| keyboard_focusable(node.role))
            .map(|node| node.widget)
            .filter(|widget| {
                crate::windows_win32_host::focus_windows_win32_window_accessible_semantic_node(
                    self.hwnd as _,
                    *widget,
                )
            })
            .map(|_| ())
            .ok_or_else(invalid_operation)
    }

    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        nodes(self.hwnd)?;
        Ok(root_fragment_provider_root(self.hwnd))
    }
}

impl IRawElementProviderFragmentRoot_Impl for WindowsSemanticUiaRootProvider_Impl {
    fn ElementProviderFromPoint(&self, x: f64, y: f64) -> Result<IRawElementProviderFragment> {
        let nodes = nodes(self.hwnd)?;
        nodes
            .iter()
            .filter(|node| {
                let rect = rect_in_screen(self.hwnd, node.bounds);
                x >= rect.left
                    && x < rect.left + rect.width
                    && y >= rect.top
                    && y < rect.top + rect.height
            })
            .min_by_key(|node| {
                i64::from(node.bounds.width.max(0)) * i64::from(node.bounds.height.max(0))
            })
            .map(|node| node_fragment_provider(self.hwnd, node.widget))
            .ok_or_else(Error::empty)
    }

    fn GetFocus(&self) -> Result<IRawElementProviderFragment> {
        let nodes = nodes(self.hwnd)?;
        let focused = crate::windows_win32_host::windows_win32_window_semantic_accessibility_focus(
            self.hwnd as _,
        );
        focused
            .filter(|widget| nodes.iter().any(|node| node.widget == *widget))
            .map(|widget| node_fragment_provider(self.hwnd, widget))
            .ok_or_else(Error::empty)
    }
}

impl IRawElementProviderSimple_Impl for WindowsSemanticUiaNodeProvider_Impl {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider)
    }

    fn GetPatternProvider(&self, pattern_id: UIA_PATTERN_ID) -> Result<IUnknown> {
        let node = node(self.hwnd, self.widget)?;
        if pattern_id == UIA_InvokePatternId
            && node.enabled
            && matches!(
                node.role,
                crate::ZsAccessibilityRole::Button
                    | crate::ZsAccessibilityRole::ListItem
                    | crate::ZsAccessibilityRole::Tab
            )
        {
            let provider: IInvokeProvider = self.to_interface();
            return provider.cast();
        }
        if pattern_id == UIA_SelectionItemPatternId
            && node.enabled
            && node.role == crate::ZsAccessibilityRole::Tab
        {
            let provider: ISelectionItemProvider = self.to_interface();
            return provider.cast();
        }
        #[cfg(feature = "text-input-core")]
        if node.role == crate::ZsAccessibilityRole::TextBox {
            return crate::windows_uia::text_pattern_provider(self.hwnd, pattern_id);
        }
        Err(Error::empty())
    }

    fn GetPropertyValue(&self, property_id: UIA_PROPERTY_ID) -> Result<VARIANT> {
        let node = node(self.hwnd, self.widget)?;
        let focused = crate::windows_win32_host::windows_win32_window_semantic_accessibility_focus(
            self.hwnd as _,
        ) == Some(node.widget);
        let value = match property_id {
            UIA_ControlTypePropertyId => VARIANT::from(control_type(node.role)),
            UIA_HasKeyboardFocusPropertyId => VARIANT::from(focused),
            UIA_IsKeyboardFocusablePropertyId => VARIANT::from(keyboard_focusable(node.role)),
            UIA_IsEnabledPropertyId => VARIANT::from(node.enabled),
            UIA_FrameworkIdPropertyId => VARIANT::from(BSTR::from("ZSUI")),
            UIA_ClassNamePropertyId => {
                VARIANT::from(BSTR::from(format!("ZsuiSemantic{}", node.role)))
            }
            UIA_AutomationIdPropertyId => {
                VARIANT::from(BSTR::from(format!("zsui-semantic-{}", node.widget.0)))
            }
            UIA_NamePropertyId => VARIANT::from(BSTR::from(node.label.unwrap_or_default())),
            UIA_HelpTextPropertyId => {
                VARIANT::from(BSTR::from(node.description.unwrap_or_default()))
            }
            UIA_LiveSettingPropertyId => match node.live_region {
                Some(crate::ZsAccessibilityLiveRegion::Polite) => VARIANT::from(LivePolite.0),
                Some(crate::ZsAccessibilityLiveRegion::Assertive) => VARIANT::from(LiveAssertive.0),
                None => VARIANT::default(),
            },
            UIA_SelectionItemIsSelectedPropertyId => {
                node.selected.map(VARIANT::from).unwrap_or_default()
            }
            _ => VARIANT::default(),
        };
        Ok(value)
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        node(self.hwnd, self.widget)?;
        Err(Error::empty())
    }
}

impl IRawElementProviderFragment_Impl for WindowsSemanticUiaNodeProvider_Impl {
    fn Navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        let nodes = nodes(self.hwnd)?;
        let item = nodes
            .iter()
            .find(|node| node.widget == self.widget)
            .ok_or_else(element_not_available)?;
        if direction == NavigateDirection_Parent {
            return item
                .parent
                .filter(|parent| nodes.iter().any(|node| node.widget == *parent))
                .map_or_else(
                    || Ok(root_fragment_provider(self.hwnd)),
                    |parent| Ok(node_fragment_provider(self.hwnd, parent)),
                );
        }
        let children = child_nodes(&nodes, item.widget);
        if direction == NavigateDirection_FirstChild {
            return children
                .first()
                .copied()
                .map(|widget| node_fragment_provider(self.hwnd, widget))
                .ok_or_else(Error::empty);
        }
        if direction == NavigateDirection_LastChild {
            return children
                .last()
                .copied()
                .map(|widget| node_fragment_provider(self.hwnd, widget))
                .ok_or_else(Error::empty);
        }
        let siblings = siblings(&nodes, item);
        let index = siblings
            .iter()
            .position(|widget| *widget == item.widget)
            .ok_or_else(element_not_available)?;
        if direction == NavigateDirection_NextSibling {
            return siblings
                .get(index.saturating_add(1))
                .copied()
                .map(|widget| node_fragment_provider(self.hwnd, widget))
                .ok_or_else(Error::empty);
        }
        if direction == NavigateDirection_PreviousSibling {
            return index
                .checked_sub(1)
                .and_then(|index| siblings.get(index))
                .copied()
                .map(|widget| node_fragment_provider(self.hwnd, widget))
                .ok_or_else(Error::empty);
        }
        Err(Error::empty())
    }

    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        node(self.hwnd, self.widget)?;
        safe_array_from_i32_slice(&[
            UiaAppendRuntimeId as i32,
            (self.widget.0 >> 32) as u32 as i32,
            self.widget.0 as u32 as i32,
        ])
    }

    fn BoundingRectangle(&self) -> Result<UiaRect> {
        Ok(rect_in_screen(
            self.hwnd,
            node(self.hwnd, self.widget)?.bounds,
        ))
    }

    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        node(self.hwnd, self.widget)?;
        Ok(std::ptr::null_mut())
    }

    fn SetFocus(&self) -> Result<()> {
        let node = node(self.hwnd, self.widget)?;
        if !keyboard_focusable(node.role) {
            return Err(invalid_operation());
        }
        crate::windows_win32_host::focus_windows_win32_window_accessible_semantic_node(
            self.hwnd as _,
            self.widget,
        )
        .then_some(())
        .ok_or_else(element_not_available)
    }

    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        node(self.hwnd, self.widget)?;
        Ok(root_fragment_provider_root(self.hwnd))
    }
}

impl IInvokeProvider_Impl for WindowsSemanticUiaNodeProvider_Impl {
    fn Invoke(&self) -> Result<()> {
        let node = node(self.hwnd, self.widget)?;
        if !node.enabled
            || !matches!(
                node.role,
                crate::ZsAccessibilityRole::Button
                    | crate::ZsAccessibilityRole::ListItem
                    | crate::ZsAccessibilityRole::Tab
            )
        {
            return Err(invalid_operation());
        }
        crate::windows_win32_host::invoke_windows_win32_window_accessible_semantic_node(
            self.hwnd as _,
            self.widget,
        )
        .then_some(())
        .ok_or_else(element_not_available)
    }
}

impl ISelectionItemProvider_Impl for WindowsSemanticUiaNodeProvider_Impl {
    fn Select(&self) -> Result<()> {
        let node = node(self.hwnd, self.widget)?;
        if !node.enabled || node.role != crate::ZsAccessibilityRole::Tab {
            return Err(invalid_operation());
        }
        crate::windows_win32_host::invoke_windows_win32_window_accessible_semantic_node(
            self.hwnd as _,
            self.widget,
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
        let node = node(self.hwnd, self.widget)?;
        if node.role != crate::ZsAccessibilityRole::Tab {
            return Err(invalid_operation());
        }
        Ok(BOOL::from(node.selected.unwrap_or(false)))
    }

    fn SelectionContainer(&self) -> Result<IRawElementProviderSimple> {
        let node = node(self.hwnd, self.widget)?;
        if node.role != crate::ZsAccessibilityRole::Tab {
            return Err(invalid_operation());
        }
        Ok(node.parent.map_or_else(
            || root_simple_provider(self.hwnd),
            |parent| node_simple_provider(self.hwnd, parent),
        ))
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
    let semantic_nodes =
        crate::windows_win32_host::windows_win32_window_semantic_accessibility_nodes(hwnd);
    if semantic_nodes.is_empty() {
        return None;
    }
    let provider = root_simple_provider(hwnd as isize);
    let result = unsafe {
        UiaReturnRawElementProvider(HWND(hwnd.cast()), WPARAM(wparam), LPARAM(lparam), &provider)
    };
    Some(result.0)
}

fn provider_subtree_count(provider: &IRawElementProviderFragment) -> usize {
    let mut count = 1_usize;
    let Ok(mut child) = (unsafe { provider.Navigate(NavigateDirection_FirstChild) }) else {
        return count;
    };
    loop {
        count = count.saturating_add(provider_subtree_count(&child));
        let Ok(next) = (unsafe { child.Navigate(NavigateDirection_NextSibling) }) else {
            break;
        };
        child = next;
    }
    count
}

pub(crate) fn proof_provider_tree(
    hwnd: windows_sys::Win32::Foundation::HWND,
) -> Option<(usize, usize)> {
    let semantic_nodes =
        crate::windows_win32_host::windows_win32_window_semantic_accessibility_nodes(hwnd);
    if semantic_nodes.is_empty() {
        return None;
    }
    handle_get_object(hwnd, 0, UiaRootObjectId as isize)?;
    let provider = root_fragment_provider(hwnd as isize);
    let provider_node_count = provider_subtree_count(&provider).saturating_sub(1);
    if provider_node_count != semantic_nodes.len() {
        return None;
    }
    let action_count = semantic_nodes
        .iter()
        .filter(|node| {
            node.enabled
                && matches!(
                    node.role,
                    crate::ZsAccessibilityRole::Button
                        | crate::ZsAccessibilityRole::ListItem
                        | crate::ZsAccessibilityRole::Tab
                        | crate::ZsAccessibilityRole::TextBox
                )
        })
        .count();
    Some((provider_node_count, action_count))
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

    #[test]
    fn semantic_providers_expose_fragment_and_invoke_contracts() {
        let root = IRawElementProviderSimple::from(WindowsSemanticUiaRootProvider { hwnd: 1 });
        assert!(root.cast::<IRawElementProviderFragment>().is_ok());
        assert!(root.cast::<IRawElementProviderFragmentRoot>().is_ok());

        let node = IRawElementProviderSimple::from(WindowsSemanticUiaNodeProvider {
            hwnd: 1,
            widget: crate::WidgetId(10),
        });
        assert!(node.cast::<IRawElementProviderFragment>().is_ok());
        assert!(node.cast::<IInvokeProvider>().is_ok());
        assert!(node.cast::<ISelectionItemProvider>().is_ok());
    }

    #[test]
    fn semantic_roles_map_to_native_control_types() {
        assert_eq!(
            control_type(crate::ZsAccessibilityRole::Image),
            UIA_ImageControlTypeId.0
        );
        assert_eq!(
            control_type(crate::ZsAccessibilityRole::Group),
            UIA_GroupControlTypeId.0
        );
        assert_eq!(
            control_type(crate::ZsAccessibilityRole::TextBox),
            UIA_EditControlTypeId.0
        );
    }
}
