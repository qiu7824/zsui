use crate::native_input_visuals::NativeTextShaper;

#[cfg(windows)]
pub(crate) trait NativePlatformTextShaper: NativeTextShaper + Send + Sync {}

#[cfg(windows)]
impl<T> NativePlatformTextShaper for T where T: NativeTextShaper + Send + Sync {}

#[cfg(not(windows))]
pub(crate) trait NativePlatformTextShaper: NativeTextShaper {}

#[cfg(not(windows))]
impl<T> NativePlatformTextShaper for T where T: NativeTextShaper {}
