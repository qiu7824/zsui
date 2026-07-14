#![allow(non_snake_case, non_upper_case_globals)]

use windows::core::{implement, Error, IUnknown, Interface, Result, BOOL, BSTR, HRESULT, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    IRawElementProviderSimple, IRawElementProviderSimple_Impl, IValueProvider, IValueProvider_Impl,
    ProviderOptions, ProviderOptions_ServerSideProvider, UIA_AutomationIdPropertyId,
    UIA_ClassNamePropertyId, UIA_ControlTypePropertyId, UIA_EditControlTypeId,
    UIA_FrameworkIdPropertyId, UIA_HasKeyboardFocusPropertyId, UIA_IsEnabledPropertyId,
    UIA_IsKeyboardFocusablePropertyId, UIA_IsPasswordPropertyId, UIA_NativeWindowHandlePropertyId,
    UIA_ValueIsReadOnlyPropertyId, UIA_ValuePatternId, UIA_ValueValuePropertyId,
    UiaHostProviderFromHwnd, UiaReturnRawElementProvider, UiaRootObjectId,
    UIA_E_ELEMENTNOTAVAILABLE, UIA_E_NOTSUPPORTED, UIA_PATTERN_ID, UIA_PROPERTY_ID,
};
use windows_core::IUnknownImpl;

#[implement(IRawElementProviderSimple, IValueProvider)]
struct WindowsTextUiaProvider {
    hwnd: isize,
}

impl WindowsTextUiaProvider_Impl {
    fn snapshot(&self) -> Result<crate::native_accessibility::NativeTextAccessibilitySnapshot> {
        crate::windows_win32_host::windows_win32_window_text_accessibility_snapshot(self.hwnd as _)
            .ok_or_else(|| Error::from_hresult(HRESULT(UIA_E_ELEMENTNOTAVAILABLE as i32)))
    }
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

pub(crate) fn handle_get_object(
    hwnd: windows_sys::Win32::Foundation::HWND,
    wparam: windows_sys::Win32::Foundation::WPARAM,
    lparam: windows_sys::Win32::Foundation::LPARAM,
) -> Option<windows_sys::Win32::Foundation::LRESULT> {
    if lparam != UiaRootObjectId as isize {
        return None;
    }
    crate::windows_win32_host::windows_win32_window_text_accessibility_snapshot(hwnd)?;
    let provider = IRawElementProviderSimple::from(WindowsTextUiaProvider {
        hwnd: hwnd as isize,
    });
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
    fn provider_exposes_the_native_server_side_contract() {
        let provider = IRawElementProviderSimple::from(WindowsTextUiaProvider { hwnd: 1 });
        assert!(provider.cast::<IValueProvider>().is_ok());
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
        let value: IValueProvider = raw.cast().expect("implemented Value interface");
        assert!(unsafe { value.Value() }.is_err());
        let replacement = windows::core::HSTRING::from("leaked");
        assert!(unsafe { value.SetValue(&replacement) }.is_err());
        let snapshot =
            crate::windows_win32_host::windows_win32_window_text_accessibility_snapshot(hwnd)
                .expect("focused password route should keep a redacted snapshot");
        assert_eq!(snapshot.exposed_text(), "••••••");

        crate::windows_win32_host::clear_windows_win32_window_view_input_route(hwnd);
    }
}
