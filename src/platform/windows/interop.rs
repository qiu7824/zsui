//! Window-owned Win32 child providers and pre-dispatch message filters.
//!
//! This backend boundary is the only place where native plug-ins need Win32
//! handles. Ordinary View declarations use `NativeHostId` exclusively.

use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
    sync::Arc,
};
use windows_sys::Win32::Graphics::Gdi::{CreateRectRgn, DeleteObject, SetWindowRgn};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetParent, IsChild, WS_CHILD, WS_TABSTOP};
const CHILD_IDENTITY: *const u16 = windows_sys::core::w!("ZSUI.NativeChild.3");

impl crate::NativeWindowBuilder {
    /// Registers native child providers and window-scoped message-filter factories.
    pub fn windows_hooks(mut self, hooks: WindowsWindowHooks) -> Self {
        self.window_spec_mut().windows_hooks = Some(hooks);
        self
    }
}

/// A borrowed parent valid only while a provider is being attached.
pub struct NativeParent<'a> {
    hwnd: HWND,
    _lifetime: PhantomData<&'a ()>,
}

impl NativeParent<'_> {
    /// Creates an empty native container for a COM or third-party child surface.
    pub fn container(&self) -> ZsuiResult<NativeChildWindow> {
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                wide_null("STATIC").as_ptr(),
                wide_null("").as_ptr(),
                WS_CHILD | WS_CLIPCHILDREN,
                0,
                0,
                1,
                1,
                self.hwnd,
                null_mut(),
                GetModuleHandleW(null()),
                null(),
            )
        };
        unsafe { NativeChildWindow::from_owned_handle(hwnd, self) }
    }
    /// Returns the backend handle. It must never be destroyed or retained past
    /// the owning window's lifetime by a provider.
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Creates a standard native edit control owned by the parent window.
    pub fn edit(&self, text: &str, multiline: bool) -> ZsuiResult<NativeChildWindow> {
        if text.contains('\0') {
            return Err(ZsuiError::invalid_spec(
                "native_host.text",
                "text contains a NUL",
            ));
        }
        let style = WS_CHILD
            | WS_TABSTOP
            | if multiline {
                0x0004 | 0x0040 | 0x1000
            } else {
                0x0080
            };
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                wide_null("EDIT").as_ptr(),
                wide_null(text).as_ptr(),
                style,
                0,
                0,
                1,
                1,
                self.hwnd,
                null_mut(),
                GetModuleHandleW(null()),
                null(),
            )
        };
        let mut child = unsafe { NativeChildWindow::from_owned_handle(hwnd, self)? };
        child.uses_default_font = true;
        child.update_default_font()?;
        Ok(child)
    }
}

/// An owned child HWND. Destruction is paired with provider removal or teardown.
#[derive(Debug)]
pub struct NativeChildWindow {
    hwnd: HWND,
    identity: Box<u8>,
    font: Cell<windows_sys::Win32::Graphics::Gdi::HFONT>,
    font_dpi: Cell<u32>,
    uses_default_font: bool,
    _ui_thread: PhantomData<std::rc::Rc<()>>,
}

impl NativeChildWindow {
    /// Adopts a child created for `parent` on the current UI thread.
    ///
    /// # Safety
    /// The caller transfers exclusive destruction ownership of a live HWND
    /// created on the current thread. No other owner may destroy/reuse it.
    pub unsafe fn from_owned_handle(hwnd: HWND, parent: &NativeParent<'_>) -> ZsuiResult<Self> {
        let mut process = 0;
        let thread = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(
            hwnd,
            &mut process,
        );
        if hwnd.is_null()
            || IsWindow(hwnd) == 0
            || GetParent(hwnd) != parent.hwnd
            || thread != windows_sys::Win32::System::Threading::GetCurrentThreadId()
        {
            return Err(ZsuiError::host(
                "native_host.create",
                "provider did not return a live child of this window on its UI thread",
            ));
        }
        if !windows_sys::Win32::UI::WindowsAndMessaging::GetPropW(hwnd, CHILD_IDENTITY).is_null() {
            return Err(ZsuiError::invalid_spec(
                "native_host.owner",
                "the native child already has an owner",
            ));
        }
        let identity = Box::new(0u8);
        if windows_sys::Win32::UI::WindowsAndMessaging::SetPropW(
            hwnd,
            CHILD_IDENTITY,
            (&*identity as *const u8).cast_mut().cast(),
        ) == 0
        {
            DestroyWindow(hwnd);
            return Err(ZsuiError::host(
                "native_host.owner",
                "could not attach child lifetime identity",
            ));
        }
        Ok(Self {
            hwnd,
            identity,
            font: Cell::new(null_mut()),
            font_dpi: Cell::new(0),
            uses_default_font: false,
            _ui_thread: PhantomData,
        })
    }
    /// Returns the backend child handle.
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
    fn is_current(&self) -> bool {
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::GetPropW(self.hwnd, CHILD_IDENTITY)
                == (&*self.identity as *const u8).cast_mut().cast()
        }
    }
    fn update_default_font(&self) -> ZsuiResult<()> {
        if !self.uses_default_font {
            return Ok(());
        }
        let dpi = unsafe { GetDpiForWindow(self.hwnd) }.max(96);
        if self.font_dpi.get() == dpi {
            return Ok(());
        }
        let typography = crate::windows_gdi_renderer::windows_native_typography_profile();
        let mut font: windows_sys::Win32::Graphics::Gdi::LOGFONTW = unsafe { zeroed() };
        font.lfHeight = -(typography.body_metrics.size * dpi as f32 / 96.0)
            .round()
            .max(1.0) as i32;
        font.lfWeight = 400;
        font.lfCharSet = windows_sys::Win32::Graphics::Gdi::DEFAULT_CHARSET;
        font.lfQuality = windows_sys::Win32::Graphics::Gdi::CLEARTYPE_QUALITY;
        for (slot, ch) in font
            .lfFaceName
            .iter_mut()
            .take(31)
            .zip(typography.ui_font_family.encode_utf16())
        {
            *slot = ch;
        }
        let handle = unsafe { windows_sys::Win32::Graphics::Gdi::CreateFontIndirectW(&font) };
        if handle.is_null() {
            return Err(ZsuiError::host(
                "native_host.font",
                "CreateFontIndirectW failed",
            ));
        }
        unsafe {
            SendMessageW(
                self.hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::WM_SETFONT,
                handle as usize,
                1,
            );
        }
        let previous = self.font.replace(handle);
        if !previous.is_null() {
            unsafe {
                DeleteObject(previous);
            }
        }
        self.font_dpi.set(dpi);
        Ok(())
    }
    /// Replaces the native control's text without exposing a handle to View code.
    pub fn set_text(&self, text: &str) -> ZsuiResult<()> {
        if !self.is_current() {
            return Err(ZsuiError::host(
                "native_host.set_text",
                "child window no longer exists",
            ));
        }
        if text.contains('\0') {
            return Err(ZsuiError::invalid_spec(
                "native_host.text",
                "text contains a NUL",
            ));
        }
        if unsafe { SetWindowTextW(self.hwnd, wide_null(text).as_ptr()) } == 0 {
            return Err(ZsuiError::host(
                "native_host.set_text",
                "SetWindowTextW failed",
            ));
        }
        Ok(())
    }
    fn place(
        &self,
        placement: crate::NativeHostPlacement,
        visible: bool,
        occlusions: impl Iterator<Item = crate::Rect>,
    ) -> ZsuiResult<()> {
        if !self.is_current() {
            return Err(ZsuiError::host(
                "native_host.place",
                "child window no longer exists",
            ));
        }
        self.update_default_font()?;
        let b = placement.bounds;
        let c = placement.clip;
        unsafe {
            if SetWindowPos(
                self.hwnd,
                null_mut(),
                b.x,
                b.y,
                b.width.max(1),
                b.height.max(1),
                SWP_NOACTIVATE | SWP_NOZORDER,
            ) == 0
            {
                return Err(ZsuiError::host("native_host.place", "SetWindowPos failed"));
            }
            let region = CreateRectRgn(
                c.x.saturating_sub(b.x),
                c.y.saturating_sub(b.y),
                c.x.saturating_sub(b.x).saturating_add(c.width.max(0)),
                c.y.saturating_sub(b.y).saturating_add(c.height.max(0)),
            );
            if region.is_null() {
                return Err(ZsuiError::host("native_host.clip", "CreateRectRgn failed"));
            }
            for occlusion in occlusions {
                let cut = CreateRectRgn(
                    occlusion.x.saturating_sub(b.x),
                    occlusion.y.saturating_sub(b.y),
                    occlusion
                        .x
                        .saturating_sub(b.x)
                        .saturating_add(occlusion.width),
                    occlusion
                        .y
                        .saturating_sub(b.y)
                        .saturating_add(occlusion.height),
                );
                if cut.is_null() {
                    DeleteObject(region);
                    return Err(ZsuiError::host(
                        "native_host.clip",
                        "could not allocate overlay clip",
                    ));
                }
                let result = windows_sys::Win32::Graphics::Gdi::CombineRgn(
                    region,
                    region,
                    cut,
                    windows_sys::Win32::Graphics::Gdi::RGN_DIFF,
                );
                DeleteObject(cut);
                if result == 0 {
                    DeleteObject(region);
                    return Err(ZsuiError::host(
                        "native_host.clip",
                        "could not subtract overlay clip",
                    ));
                }
            }
            if SetWindowRgn(self.hwnd, region, 0) == 0 {
                DeleteObject(region);
                return Err(ZsuiError::host("native_host.clip", "SetWindowRgn failed"));
            }
            ShowWindow(
                self.hwnd,
                if visible && c.width > 0 && c.height > 0 {
                    SW_SHOWNOACTIVATE
                } else {
                    SW_HIDE
                },
            );
            windows_sys::Win32::Graphics::Gdi::RedrawWindow(
                self.hwnd,
                null(),
                null_mut(),
                windows_sys::Win32::Graphics::Gdi::RDW_INVALIDATE
                    | windows_sys::Win32::Graphics::Gdi::RDW_ALLCHILDREN
                    | windows_sys::Win32::Graphics::Gdi::RDW_NOERASE,
            );
        }
        Ok(())
    }
}

impl Drop for NativeChildWindow {
    fn drop(&mut self) {
        unsafe {
            if self.is_current() {
                windows_sys::Win32::UI::WindowsAndMessaging::RemovePropW(self.hwnd, CHILD_IDENTITY);
                DestroyWindow(self.hwnd);
            }
            if !self.font.get().is_null() {
                DeleteObject(self.font.get());
            }
        }
    }
}

/// UI-thread-owned native child. The framework owns placement and focus routing.
pub trait WindowsNativeChild {
    /// The owned child surface.
    fn window(&self) -> &NativeChildWindow;
    /// Updates child-specific layout after the framework moves and clips its HWND.
    fn layout(
        &mut self,
        _placement: crate::NativeHostPlacement,
        _dpi: crate::Dpi,
    ) -> ZsuiResult<()> {
        Ok(())
    }
    /// Gives the embedded control first refusal of a keyboard message directed to it.
    fn pre_translate(&mut self, _message: &MSG) -> bool {
        false
    }
    /// Called after the system appearance changes.
    fn theme_changed(&mut self, _mode: crate::ZsuiThemeMode) -> ZsuiResult<()> {
        Ok(())
    }
    /// Moves keyboard focus into the child.
    fn focus(&mut self) {
        if self.window().is_current() {
            unsafe {
                SetFocus(self.window().hwnd);
            }
        }
    }
    /// Gives a child a chance to commit edits before keyboard traversal leaves it.
    fn blur(&mut self) {}
}

impl WindowsNativeChild for NativeChildWindow {
    fn window(&self) -> &NativeChildWindow {
        self
    }
}

/// A window-scoped filter invoked before TranslateMessage/DispatchMessage.
pub trait NativeMessageFilter {
    /// Returns true only when this filter has consumed the complete message.
    fn pre_translate(&mut self, message: &MSG) -> bool;
}

type ChildFactory =
    Arc<dyn for<'a> Fn(NativeParent<'a>) -> ZsuiResult<Box<dyn WindowsNativeChild>> + Send + Sync>;
type FilterFactory = Arc<dyn Fn() -> Box<dyn NativeMessageFilter> + Send + Sync>;

#[derive(Clone, Default)]
struct HookDeclarations {
    children: BTreeMap<crate::NativeHostId, ChildFactory>,
    filters: Vec<FilterFactory>,
}

/// Cloneable provider declarations. Each created window gets independent UI state.
#[derive(Clone, Default)]
pub struct WindowsWindowHooks(Arc<HookDeclarations>);

impl std::fmt::Debug for WindowsWindowHooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowsWindowHooks")
            .field("children", &self.0.children.len())
            .field("filters", &self.0.filters.len())
            .finish()
    }
}
impl PartialEq for WindowsWindowHooks {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for WindowsWindowHooks {}

impl WindowsWindowHooks {
    /// Starts an empty set of per-window provider declarations.
    pub fn new() -> Self {
        Self::default()
    }
    /// Registers one provider for a stable native slot ID.
    pub fn child(
        mut self,
        id: crate::NativeHostId,
        factory: impl for<'a> Fn(NativeParent<'a>) -> ZsuiResult<Box<dyn WindowsNativeChild>>
            + Send
            + Sync
            + 'static,
    ) -> ZsuiResult<Self> {
        let declarations = Arc::make_mut(&mut self.0);
        if declarations.children.contains_key(&id) {
            return Err(ZsuiError::invalid_spec(
                "native_host.id",
                "duplicate native child provider ID",
            ));
        }
        declarations.children.insert(id, Arc::new(factory));
        Ok(self)
    }
    /// Adds a filter factory; filters run in declaration order on the owning UI thread.
    pub fn message_filter(
        mut self,
        factory: impl Fn() -> Box<dyn NativeMessageFilter> + Send + Sync + 'static,
    ) -> Self {
        Arc::make_mut(&mut self.0).filters.push(Arc::new(factory));
        self
    }
    /// Registers a native edit control without application-side Win32 calls.
    pub fn edit(
        self,
        id: crate::NativeHostId,
        text: impl Into<String>,
        multiline: bool,
    ) -> ZsuiResult<Self> {
        let text = text.into();
        self.child(id, move |parent| {
            Ok(Box::new(parent.edit(&text, multiline)?))
        })
    }
}

pub(super) struct NativeHostRuntime {
    declarations: WindowsWindowHooks,
    children:
        BTreeMap<crate::NativeHostId, (Box<dyn WindowsNativeChild>, crate::NativeHostPlacement)>,
    filters: Vec<Box<dyn NativeMessageFilter>>,
    last_occlusions: Vec<crate::native_host::NativeSurfaceOcclusion>,
    last_dpi: u32,
    last_visible: bool,
}

impl NativeHostRuntime {
    pub(super) fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
    pub(super) fn new(declarations: WindowsWindowHooks) -> Self {
        let filters = declarations
            .0
            .filters
            .iter()
            .map(|factory| factory())
            .collect();
        Self {
            declarations,
            children: BTreeMap::new(),
            filters,
            last_occlusions: Vec::new(),
            last_dpi: 0,
            last_visible: false,
        }
    }
    pub(super) fn reconcile(
        &mut self,
        hwnd: HWND,
        placements: &[crate::NativeHostPlacement],
        visible: bool,
        occlusions: &[crate::native_host::NativeSurfaceOcclusion],
    ) -> ZsuiResult<()> {
        let mut wanted = BTreeSet::new();
        for placement in placements {
            if !wanted.insert(placement.id) {
                return Err(ZsuiError::invalid_spec(
                    "native_host.id",
                    "duplicate native host ID in the View",
                ));
            }
            if !self.declarations.0.children.contains_key(&placement.id) {
                return Err(ZsuiError::invalid_spec(
                    "native_host.provider",
                    format!(
                        "no provider registered for native slot {}",
                        placement.id.get()
                    ),
                ));
            }
        }
        self.children.retain(|id, _| wanted.contains(id));
        let dpi_value = unsafe { GetDpiForWindow(hwnd) }.max(96);
        let dpi = crate::Dpi::new(dpi_value as f32);
        let environment_changed = self.last_dpi != dpi_value
            || self.last_visible != visible
            || self.last_occlusions != occlusions;
        for (index, placement) in placements.iter().enumerate() {
            let created = !self.children.contains_key(&placement.id);
            if created {
                let factory = &self.declarations.0.children[&placement.id];
                let child = factory(NativeParent {
                    hwnd,
                    _lifetime: PhantomData,
                })?;
                if unsafe { GetParent(child.window().hwnd) } != hwnd {
                    return Err(ZsuiError::host(
                        "native_host.create",
                        "child belongs to another parent",
                    ));
                }
                self.children.insert(placement.id, (child, *placement));
                let mode = super::windows_platform_palette(
                    super::window_draw_plan(hwnd)
                        .map(|plan| plan.theme_mode)
                        .unwrap_or_default(),
                )
                .mode;
                super::apply_native_child_theme(self.children[&placement.id].0.window().hwnd, mode);
            }
            let (child, previous) = self
                .children
                .get_mut(&placement.id)
                .expect("provider inserted");
            if created || environment_changed || previous != placement {
                child.window().place(
                    *placement,
                    visible,
                    occlusions
                        .iter()
                        .filter(|occlusion| occlusion.hosts_before > index)
                        .map(|occlusion| occlusion.bounds),
                )?;
                child.layout(*placement, dpi)?;
            }
            *previous = *placement;
        }
        self.last_dpi = dpi_value;
        self.last_visible = visible;
        self.last_occlusions = occlusions.to_vec();
        Ok(())
    }
    pub(super) fn pre_translate(&mut self, msg: &MSG) -> bool {
        for (child, _) in self.children.values_mut() {
            let hwnd = child.window().hwnd;
            if (msg.hwnd == hwnd || unsafe { IsChild(hwnd, msg.hwnd) } != 0)
                && child.pre_translate(msg)
            {
                return true;
            }
        }
        self.filters
            .iter_mut()
            .any(|filter| filter.pre_translate(msg))
    }
    pub(super) fn child_at_focus(&self, focused: HWND) -> Option<crate::NativeHostPlacement> {
        self.children.values().find_map(|(child, placement)| {
            (focused == child.window().hwnd
                || unsafe { IsChild(child.window().hwnd, focused) } != 0)
                .then_some(*placement)
        })
    }
    pub(super) fn focus(&mut self, id: crate::NativeHostId) {
        if let Some((child, placement)) = self.children.get_mut(&id) {
            if placement.focusable {
                child.focus();
            }
        }
    }
    pub(super) fn blur(&mut self, id: crate::NativeHostId) {
        if let Some((child, _)) = self.children.get_mut(&id) {
            child.blur();
        }
    }
    pub(super) fn theme_changed(&mut self, mode: crate::ZsuiThemeMode) -> ZsuiResult<()> {
        for (child, _) in self.children.values_mut() {
            super::apply_native_child_theme(child.window().hwnd, mode);
            child.theme_changed(mode)?;
        }
        Ok(())
    }
}
