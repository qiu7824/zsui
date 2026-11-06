//! STA-owned Shell data objects, drag/drop targets and native child controls.
use super::*;
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};
use windows::{
    core::{Interface, PCWSTR},
    Win32::{
        Foundation::{HWND as WinHwnd, POINT as WinPoint, POINTL},
        System::{Com::*, Ole::*, SystemServices::MODIFIERKEYS_FLAGS},
        UI::{Shell::Common::ITEMIDLIST, Shell::*},
    },
};

fn shell_error(error: windows_core::Error) -> ZsuiError {
    ZsuiError::host("windows_shell", error.to_string())
}

include!("shell_controls.rs");
include!("shell_menu.rs");

pub(super) struct OleApartment {
    _thread: PhantomData<Rc<()>>,
}
impl OleApartment {
    pub(super) fn new() -> ZsuiResult<Self> {
        unsafe {
            OleInitialize(None).map_err(shell_error)?;
        }
        Ok(Self {
            _thread: PhantomData,
        })
    }
}
impl Drop for OleApartment {
    fn drop(&mut self) {
        unsafe {
            OleUninitialize();
        }
    }
}

fn shell_parsing_name(path: &Path) -> ZsuiResult<Vec<u16>> {
    use std::{
        os::windows::ffi::OsStrExt,
        path::{Component, Prefix},
    };
    let mut name: Vec<u16> = path.as_os_str().encode_wide().collect();
    if name.is_empty() || name.contains(&0) {
        return Err(ZsuiError::invalid_spec(
            "shell.items",
            "Shell item names must be nonempty and contain no NUL",
        ));
    }
    if let Some(Component::Prefix(prefix)) = path.components().next() {
        match prefix.kind() {
            Prefix::VerbatimDisk(_) | Prefix::VerbatimUNC(_, _) => {
                if !path.is_absolute() || path.components().any(|part| matches!(part, Component::Normal(value) if matches!(value.encode_wide().last(), Some(32 | 46)))) {
                    return Err(ZsuiError::invalid_spec("shell.items", "this verbatim path cannot be represented by the Windows Shell without changing its meaning"));
                }
                name = if matches!(prefix.kind(), Prefix::VerbatimUNC(_, _)) {
                    let mut normalized = vec![b'\\' as u16, b'\\' as u16];
                    normalized.extend_from_slice(&name[8..]);
                    normalized
                } else {
                    name[4..].to_vec()
                };
            }
            Prefix::Verbatim(_) | Prefix::DeviceNS(_) => {
                return Err(ZsuiError::invalid_spec(
                    "shell.items",
                    "device namespace paths are not Windows Shell item names",
                ));
            }
            _ => {}
        }
    }
    name.push(0);
    Ok(name)
}

struct ShellItemId(*mut ITEMIDLIST);
impl ShellItemId {
    fn parse(path: &Path) -> ZsuiResult<Self> {
        let name = shell_parsing_name(path)?;
        let mut id = null_mut();
        unsafe {
            SHParseDisplayName(PCWSTR(name.as_ptr()), None, &mut id, 0, None)
                .map_err(shell_error)?;
        }
        if id.is_null() {
            return Err(ZsuiError::host(
                "SHParseDisplayName",
                "Shell returned a null item ID",
            ));
        }
        Ok(Self(id))
    }
}
impl Drop for ShellItemId {
    fn drop(&mut self) {
        unsafe {
            windows::Win32::System::Com::CoTaskMemFree(Some(self.0.cast()));
        }
    }
}

fn shell_items(paths: &[PathBuf]) -> ZsuiResult<IShellItemArray> {
    if paths.is_empty() || paths.len() > 65_536 {
        return Err(ZsuiError::invalid_spec(
            "shell.items",
            "a Shell transfer requires between 1 and 65536 items",
        ));
    }
    let ids = paths
        .iter()
        .map(|path| ShellItemId::parse(path))
        .collect::<ZsuiResult<Vec<_>>>()?;
    let pointers: Vec<*const ITEMIDLIST> = ids.iter().map(|id| id.0.cast_const()).collect();
    unsafe { SHCreateShellItemArrayFromIDLists(&pointers).map_err(shell_error) }
}

pub(super) fn data_object(paths: &[PathBuf]) -> ZsuiResult<IDataObject> {
    unsafe {
        shell_items(paths)?
            .BindToHandler(None, &BHID_DataObject)
            .map_err(shell_error)
    }
}

fn file_format() -> FORMATETC {
    FORMATETC {
        cfFormat: CF_HDROP.0,
        ptd: null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    }
}

fn object_files(data: &IDataObject) -> ZsuiResult<Option<Vec<PathBuf>>> {
    use std::os::windows::ffi::OsStringExt;
    unsafe {
        let format = file_format();
        if data.QueryGetData(&format).is_err() {
            return Ok(None);
        }
        let mut medium = data.GetData(&format).map_err(shell_error)?;
        let result = (|| {
            if medium.tymed != TYMED_HGLOBAL.0 as u32 {
                return Err(ZsuiError::host(
                    "shell.files",
                    "invalid storage medium for CF_HDROP",
                ));
            }
            let drop = HDROP(medium.u.hGlobal.0);
            let count = DragQueryFileW(drop, u32::MAX, None);
            if count > 65_536 {
                return Err(ZsuiError::invalid_spec(
                    "shell.files",
                    "too many paths in the data object",
                ));
            }
            let mut paths = Vec::with_capacity(count as usize);
            for index in 0..count {
                let length = DragQueryFileW(drop, index, None);
                if length == 0 || length > 32767 {
                    return Err(ZsuiError::invalid_spec(
                        "shell.files",
                        "invalid path length in the data object",
                    ));
                }
                let mut buffer = vec![0; length as usize + 1];
                if DragQueryFileW(drop, index, Some(&mut buffer)) != length {
                    return Err(ZsuiError::host(
                        "shell.files",
                        "Shell path changed during extraction",
                    ));
                }
                buffer.truncate(length as usize);
                paths.push(PathBuf::from(std::ffi::OsString::from_wide(&buffer)));
            }
            Ok(Some(paths))
        })();
        ReleaseStgMedium(&mut medium);
        result
    }
}

#[cfg(feature = "clipboard")]
pub(super) fn read_file_clipboard() -> ZsuiResult<Option<crate::ClipboardData>> {
    if unsafe {
        windows_sys::Win32::System::DataExchange::IsClipboardFormatAvailable(u32::from(CF_HDROP.0))
    } == 0
    {
        return Ok(None);
    }
    let _apartment = OleApartment::new()?;
    let object = unsafe { OleGetClipboard() }.map_err(shell_error)?;
    object_files(&object)?
        .map(|files| {
            files
                .into_iter()
                .map(|path| {
                    path.into_os_string().into_string().map_err(|_| {
                        ZsuiError::invalid_spec(
                            "clipboard_files",
                            "a native path cannot be represented as UTF-8",
                        )
                    })
                })
                .collect::<ZsuiResult<Vec<_>>>()
                .map(crate::ClipboardData::Files)
        })
        .transpose()
}

#[cfg(feature = "clipboard")]
pub(super) fn write_file_clipboard(files: &[String]) -> ZsuiResult<()> {
    let paths: Vec<_> = files.iter().map(PathBuf::from).collect();
    if paths.iter().any(|path| !path.is_absolute()) {
        return Err(ZsuiError::invalid_spec(
            "clipboard_files",
            "file clipboard paths must be absolute filesystem paths",
        ));
    }
    let _apartment = OleApartment::new()?;
    let object = data_object(&paths)?;
    unsafe {
        OleSetClipboard(&object).map_err(shell_error)?;
        OleFlushClipboard().map_err(shell_error)
    }
}

fn effect_from_native(effect: DROPEFFECT) -> Option<crate::DropEffect> {
    match effect.0 & 7 {
        1 => Some(crate::DropEffect::Copy),
        2 => Some(crate::DropEffect::Move),
        4 => Some(crate::DropEffect::Link),
        _ => None,
    }
}

fn negotiated_effect(allowed: u32, keys: MODIFIERKEYS_FLAGS) -> DROPEFFECT {
    let control = keys.0 & 8 != 0;
    let shift = keys.0 & 4 != 0;
    let preferred = if control && shift {
        4
    } else if shift {
        2
    } else if control {
        1
    } else if allowed & 1 != 0 {
        1
    } else if allowed & 2 != 0 {
        2
    } else {
        4
    };
    DROPEFFECT(if allowed & preferred != 0 {
        preferred
    } else {
        0
    })
}

fn drop_target_at(
    hwnd: HWND,
    screen: &POINTL,
) -> Option<crate::shell_integration::FileDropPlacement> {
    let mut point = POINT {
        x: screen.x,
        y: screen.y,
    };
    unsafe {
        ScreenToClient(hwnd, &mut point);
    }
    let plan = window_draw_plan(hwnd)?;
    let point = crate::Point {
        x: point.x,
        y: point.y,
    };
    plan.file_drop_targets
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, target)| {
            (target.bounds.contains(point)
                && !plan.native_occlusions.iter().any(|occlusion| {
                    occlusion.drops_before > index && occlusion.bounds.contains(point)
                }))
            .then_some(*target)
        })
}

#[windows::core::implement(IDropTarget)]
struct ShellDropTarget {
    hwnd: HWND,
    data: RefCell<Option<IDataObject>>,
    helper: Option<IDropTargetHelper>,
    _apartment: Rc<OleApartment>,
}

impl ShellDropTarget_Impl {
    fn effect(
        &self,
        point: &POINTL,
        keys: MODIFIERKEYS_FLAGS,
        effect: *mut DROPEFFECT,
    ) -> windows_core::Result<()> {
        if effect.is_null() {
            return Err(windows_core::Error::from_hresult(windows_core::HRESULT(
                0x80070057u32 as i32,
            )));
        }
        let valid = self
            .data
            .borrow()
            .as_ref()
            .is_some_and(|data| unsafe { data.QueryGetData(&file_format()) }.is_ok());
        let allowed = if valid {
            drop_target_at(self.hwnd, point).map_or(0, |target| target.effects.bits())
        } else {
            0
        };
        unsafe {
            *effect = negotiated_effect((*effect).0 & allowed, keys);
        }
        Ok(())
    }
}

impl IDropTarget_Impl for ShellDropTarget_Impl {
    fn DragEnter(
        &self,
        data: windows_core::Ref<IDataObject>,
        keys: MODIFIERKEYS_FLAGS,
        point: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> windows_core::Result<()> {
        *self.data.borrow_mut() = data.as_ref().cloned();
        self.effect(point, keys, effect)?;
        if let Some(helper) = &self.helper {
            if let Some(data) = data.as_ref() {
                unsafe {
                    let _ = helper.DragEnter(
                        WinHwnd(self.hwnd),
                        data,
                        &WinPoint {
                            x: point.x,
                            y: point.y,
                        },
                        *effect,
                    );
                }
            }
        }
        Ok(())
    }
    fn DragOver(
        &self,
        keys: MODIFIERKEYS_FLAGS,
        point: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> windows_core::Result<()> {
        self.effect(point, keys, effect)?;
        if let Some(helper) = &self.helper {
            unsafe {
                let _ = helper.DragOver(
                    &WinPoint {
                        x: point.x,
                        y: point.y,
                    },
                    *effect,
                );
            }
        }
        Ok(())
    }
    fn DragLeave(&self) -> windows_core::Result<()> {
        self.data.borrow_mut().take();
        if let Some(helper) = &self.helper {
            unsafe {
                let _ = helper.DragLeave();
            }
        }
        Ok(())
    }
    fn Drop(
        &self,
        data: windows_core::Ref<IDataObject>,
        keys: MODIFIERKEYS_FLAGS,
        point: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> windows_core::Result<()> {
        *self.data.borrow_mut() = data.as_ref().cloned();
        self.effect(point, keys, effect)?;
        let mut accepted = false;
        if let (Some(data), Some(target), Some(operation)) =
            (data.as_ref(), drop_target_at(self.hwnd, point), unsafe {
                effect_from_native(*effect)
            })
        {
            match object_files(data) {
                Ok(Some(files)) if !files.is_empty() => {
                    let report = dispatch_windows_win32_window_view_input(self.hwnd, |route| {
                        let report = route.shared_runtime.dispatch_files_dropped(
                            target.widget,
                            crate::FilesDropped {
                                files,
                                effect: operation,
                            },
                        );
                        route.adapt_shared_report(report, WindowsSharedInputKind::Surface)
                    });
                    accepted = report.is_some_and(|report| {
                        crate::shell_integration::accepted_drop_effect(
                            operation,
                            report.handled && report.message_count > 0,
                            report.drop_response,
                        )
                        .is_some()
                    });
                }
                Err(error) => {
                    if let Some(services) = window_services(self.hwnd) {
                        services.record_error(error);
                    }
                }
                _ => {}
            }
        }
        if !accepted {
            unsafe {
                *effect = DROPEFFECT_NONE;
            }
        }
        if let (Some(helper), Some(data)) = (&self.helper, data.as_ref()) {
            unsafe {
                let _ = helper.Drop(
                    data,
                    &WinPoint {
                        x: point.x,
                        y: point.y,
                    },
                    *effect,
                );
            }
        }
        self.data.borrow_mut().take();
        Ok(())
    }
}

pub(super) struct ShellWindow {
    hwnd: HWND,
    _target: IDropTarget,
    _apartment: Rc<OleApartment>,
}

impl ShellWindow {
    pub(super) fn new(hwnd: HWND) -> ZsuiResult<Self> {
        let apartment = Rc::new(OleApartment::new()?);
        let helper =
            unsafe { CoCreateInstance(&CLSID_DragDropHelper, None, CLSCTX_INPROC_SERVER) }.ok();
        let target: IDropTarget = ShellDropTarget {
            hwnd,
            data: RefCell::new(None),
            helper,
            _apartment: apartment.clone(),
        }
        .into();
        unsafe {
            RegisterDragDrop(WinHwnd(hwnd), &target).map_err(shell_error)?;
        }
        Ok(Self {
            hwnd,
            _target: target,
            _apartment: apartment,
        })
    }
}
impl Drop for ShellWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = RevokeDragDrop(WinHwnd(self.hwnd));
        }
    }
}

pub(super) fn start_drag(
    hwnd: HWND,
    payload: &crate::DragPayload,
    effects: crate::DragEffects,
) -> ZsuiResult<crate::DragOutcome> {
    let _apartment = OleApartment::new()?;
    let crate::DragPayload::ShellItems(paths) = payload;
    let object = data_object(paths)?;
    let effect = unsafe {
        SHDoDragDrop(
            Some(WinHwnd(hwnd)),
            &object,
            None,
            DROPEFFECT(effects.bits()),
        )
    }
    .map_err(shell_error)?;
    Ok(effect_from_native(effect)
        .map_or(crate::DragOutcome::Cancelled, crate::DragOutcome::Performed))
}

fn windows_message(message: &MSG) -> windows::Win32::UI::WindowsAndMessaging::MSG {
    windows::Win32::UI::WindowsAndMessaging::MSG {
        hwnd: WinHwnd(message.hwnd),
        message: message.message,
        wParam: windows::Win32::Foundation::WPARAM(message.wParam),
        lParam: windows::Win32::Foundation::LPARAM(message.lParam),
        time: message.time,
        pt: WinPoint {
            x: message.pt.x,
            y: message.pt.y,
        },
        ..Default::default()
    }
}

#[windows::core::implement(IExplorerBrowserEvents)]
struct ExplorerAppearanceEvents {
    owner: HWND,
    container: HWND,
}
impl IExplorerBrowserEvents_Impl for ExplorerAppearanceEvents_Impl {
    fn OnNavigationPending(&self, _folder: *const ITEMIDLIST) -> windows_core::Result<()> {
        Ok(())
    }
    fn OnViewCreated(&self, view: windows_core::Ref<IShellView>) -> windows_core::Result<()> {
        if window_services(self.owner).is_some_and(|owner| !owner.closed.get()) {
            if let Some(view) = view.as_ref() {
                if let Ok(hwnd) = unsafe { view.GetWindow() } {
                    let mode = windows_platform_palette(
                        window_draw_plan(self.owner)
                            .map(|plan| plan.theme_mode)
                            .unwrap_or_default(),
                    )
                    .mode;
                    apply_native_child_theme(hwnd.0, mode);
                }
            }
        }
        Ok(())
    }
    fn OnNavigationComplete(&self, _folder: *const ITEMIDLIST) -> windows_core::Result<()> {
        if window_services(self.owner).is_some_and(|owner| !owner.closed.get()) {
            let mode = windows_platform_palette(
                window_draw_plan(self.owner)
                    .map(|plan| plan.theme_mode)
                    .unwrap_or_default(),
            )
            .mode;
            apply_native_child_theme(self.container, mode);
        }
        Ok(())
    }
    fn OnNavigationFailed(&self, _folder: *const ITEMIDLIST) -> windows_core::Result<()> {
        if let Some(owner) = window_services(self.owner) {
            if !owner.closed.get() {
                owner.record_error("native Explorer navigation failed");
            }
        }
        Ok(())
    }
}

struct ExplorerChild {
    browser: IExplorerBrowser,
    events_cookie: Option<u32>,
    container: interop::NativeChildWindow,
    _apartment: OleApartment,
}
impl interop::WindowsNativeChild for ExplorerChild {
    fn window(&self) -> &interop::NativeChildWindow {
        &self.container
    }
    fn layout(
        &mut self,
        placement: crate::NativeHostPlacement,
        _dpi: crate::Dpi,
    ) -> ZsuiResult<()> {
        unsafe {
            self.browser
                .SetRect(
                    None,
                    windows::Win32::Foundation::RECT {
                        left: 0,
                        top: 0,
                        right: placement.bounds.width.max(1),
                        bottom: placement.bounds.height.max(1),
                    },
                )
                .map_err(shell_error)
        }
    }
    fn pre_translate(&mut self, message: &MSG) -> bool {
        let Ok(view) = (unsafe { self.browser.GetCurrentView::<IShellView>() }) else {
            return false;
        };
        // Preserve S_FALSE; the generated Result<()> wrapper erases it.
        unsafe {
            (Interface::vtable(&view).TranslateAccelerator)(
                Interface::as_raw(&view),
                &windows_message(message),
            )
            .0 == 0
        }
    }
    fn focus(&mut self) {
        if let Ok(view) = unsafe { self.browser.GetCurrentView::<IShellView>() } {
            unsafe {
                let _ = view.UIActivate(SVUIA_ACTIVATE_FOCUS.0 as u32);
            }
        }
    }
    fn theme_changed(&mut self, mode: crate::ZsuiThemeMode) -> ZsuiResult<()> {
        apply_native_child_theme(self.container.hwnd(), mode);
        Ok(())
    }
}
impl Drop for ExplorerChild {
    fn drop(&mut self) {
        unsafe {
            if let Some(cookie) = self.events_cookie.take() {
                let _ = self.browser.Unadvise(cookie);
            }
            let _ = self.browser.Destroy();
        }
    }
}

impl interop::WindowsWindowHooks {
    /// Registers a native Explorer browser; the folder belongs to application state.
    pub fn explorer(self, id: crate::NativeHostId, folder: impl Into<PathBuf>) -> ZsuiResult<Self> {
        let folder = folder.into();
        self.child(id, move |parent| {
            #[cfg(feature = "dark-mode")]
            let _appearance = dark_mode_compatibility::AppearanceScope::new(
                windows_platform_palette(
                    window_draw_plan(parent.hwnd())
                        .map(|plan| plan.theme_mode)
                        .unwrap_or_default(),
                )
                .mode,
            );
            let apartment = OleApartment::new()?;
            let container = parent.container()?;
            let browser: IExplorerBrowser =
                unsafe { CoCreateInstance(&ExplorerBrowser, None, CLSCTX_INPROC_SERVER) }
                    .map_err(shell_error)?;
            let mut child = ExplorerChild {
                browser,
                events_cookie: None,
                container,
                _apartment: apartment,
            };
            let item = ShellItemId::parse(&folder)?;
            unsafe {
                child
                    .browser
                    .SetOptions(EBO_NOBORDER | EBO_NOTRAVELLOG)
                    .map_err(shell_error)?;
                child
                    .browser
                    .Initialize(
                        WinHwnd(child.container.hwnd()),
                        &windows::Win32::Foundation::RECT {
                            left: 0,
                            top: 0,
                            right: 1,
                            bottom: 1,
                        },
                        None,
                    )
                    .map_err(shell_error)?;
                let events: IExplorerBrowserEvents = ExplorerAppearanceEvents {
                    owner: parent.hwnd(),
                    container: child.container.hwnd(),
                }
                .into();
                child.events_cookie = Some(child.browser.Advise(&events).map_err(shell_error)?);
                child
                    .browser
                    .BrowseToIDList(item.0, SBSP_ABSOLUTE)
                    .map_err(shell_error)?;
            }
            Ok(Box::new(child))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_shell_paths_keep_unicode_and_reject_ambiguous_names() {
        let disk = shell_parsing_name(Path::new(r"\\?\C:\数据 文件\item.txt")).unwrap();
        assert_eq!(
            String::from_utf16(&disk[..disk.len() - 1]).unwrap(),
            r"C:\数据 文件\item.txt"
        );
        let unc = shell_parsing_name(Path::new(r"\\?\UNC\server\share\item.txt")).unwrap();
        assert_eq!(
            String::from_utf16(&unc[..unc.len() - 1]).unwrap(),
            r"\\server\share\item.txt"
        );
        assert!(shell_parsing_name(Path::new(r"\\?\C:\trailing.\item.txt")).is_err());
        assert!(shell_parsing_name(Path::new(r"\\.\PhysicalDrive0")).is_err());
    }
    #[test]
    fn shell_data_object_round_trips_unicode_paths_and_rejects_missing_items() {
        super::super::integration_tests::on_ui(|| {
            let _apartment = OleApartment::new().unwrap();
            let root = std::env::temp_dir().join(format!("zsui-shell-data-{}", std::process::id()));
            std::fs::create_dir(&root).unwrap();
            struct Fixture {
                root: PathBuf,
                paths: Vec<PathBuf>,
            }
            impl Drop for Fixture {
                fn drop(&mut self) {
                    for path in &self.paths {
                        let _ = std::fs::remove_file(path);
                    }
                    let _ = std::fs::remove_dir(&self.root);
                }
            }
            let fixture = Fixture {
                paths: vec![root.join("数据 空格.txt"), root.join("second.txt")],
                root,
            };
            for path in &fixture.paths {
                std::fs::write(path, b"native transfer fixture").unwrap();
            }
            let object = data_object(&fixture.paths).unwrap();
            assert_eq!(object_files(&object).unwrap(), Some(fixture.paths.clone()));
            let canonical: Vec<_> = fixture
                .paths
                .iter()
                .map(|path| path.canonicalize().unwrap())
                .collect();
            let object = data_object(&canonical).unwrap();
            let returned: Vec<_> = object_files(&object)
                .unwrap()
                .unwrap()
                .into_iter()
                .map(|path| path.canonicalize().unwrap())
                .collect();
            assert_eq!(returned, canonical);
            assert!(data_object(&[fixture.root.join("missing-file.txt")]).is_err());
            assert!(data_object(&[]).is_err());
        });
    }

    #[test]
    fn drag_negotiation_never_exceeds_the_source_or_target_contract() {
        assert_eq!(negotiated_effect(1, MODIFIERKEYS_FLAGS(0)), DROPEFFECT_COPY);
        assert_eq!(negotiated_effect(3, MODIFIERKEYS_FLAGS(4)), DROPEFFECT_MOVE);
        assert_eq!(negotiated_effect(1, MODIFIERKEYS_FLAGS(4)), DROPEFFECT_NONE);
        assert_eq!(
            negotiated_effect(7, MODIFIERKEYS_FLAGS(12)),
            DROPEFFECT_LINK
        );
        assert_eq!(negotiated_effect(0, MODIFIERKEYS_FLAGS(0)), DROPEFFECT_NONE);
    }
}
