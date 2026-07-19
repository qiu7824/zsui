#[cfg(all(windows, feature = "windows-gdi"))]
pub(super) fn capture_windows(
    sample_point: &'static str,
) -> Option<crate::NativeProofProcessMemoryEvidence> {
    use windows_sys::Win32::System::{
        ProcessStatus::{
            GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
        },
        Threading::GetCurrentProcess,
    };

    let mut counters = unsafe { std::mem::zeroed::<PROCESS_MEMORY_COUNTERS_EX>() };
    counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
    let captured = unsafe {
        GetProcessMemoryInfo(
            GetCurrentProcess(),
            (&mut counters as *mut PROCESS_MEMORY_COUNTERS_EX).cast::<PROCESS_MEMORY_COUNTERS>(),
            counters.cb,
        )
    };
    (captured != 0).then(|| crate::NativeProofProcessMemoryEvidence {
        source: "win32_get_process_memory_info",
        sample_point,
        resident_bytes: counters.WorkingSetSize as u64,
        peak_resident_bytes: counters.PeakWorkingSetSize as u64,
        private_bytes: Some(counters.PrivateUsage as u64),
        peak_private_bytes: Some(counters.PeakPagefileUsage as u64),
        proportional_set_size_bytes: None,
        virtual_bytes: None,
    })
}

#[cfg(all(target_os = "macos", feature = "macos-appkit"))]
#[allow(deprecated)]
pub(super) fn capture_macos(
    sample_point: &'static str,
) -> Option<crate::NativeProofProcessMemoryEvidence> {
    let mut info = unsafe { std::mem::zeroed::<libc::mach_task_basic_info>() };
    let mut count = libc::MACH_TASK_BASIC_INFO_COUNT;
    let captured = unsafe {
        libc::task_info(
            libc::mach_task_self(),
            libc::MACH_TASK_BASIC_INFO as libc::task_flavor_t,
            (&mut info as *mut libc::mach_task_basic_info).cast::<libc::integer_t>(),
            &mut count,
        )
    };
    if captured != libc::KERN_SUCCESS {
        return None;
    }
    let resident_bytes = unsafe { std::ptr::addr_of!(info.resident_size).read_unaligned() };
    let peak_resident_bytes =
        unsafe { std::ptr::addr_of!(info.resident_size_max).read_unaligned() };
    let virtual_bytes = unsafe { std::ptr::addr_of!(info.virtual_size).read_unaligned() };
    Some(crate::NativeProofProcessMemoryEvidence {
        source: "macos_mach_task_basic_info",
        sample_point,
        resident_bytes,
        peak_resident_bytes: peak_resident_bytes.max(resident_bytes),
        private_bytes: None,
        peak_private_bytes: None,
        proportional_set_size_bytes: None,
        virtual_bytes: Some(virtual_bytes),
    })
}

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
pub(super) fn capture_linux(
    sample_point: &'static str,
) -> Option<crate::NativeProofProcessMemoryEvidence> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let resident_bytes = linux_proc_kib(&status, "VmRSS:")?;
    let peak_resident_bytes = linux_proc_kib(&status, "VmHWM:").unwrap_or(resident_bytes);
    let virtual_bytes = linux_proc_kib(&status, "VmSize:");
    let rollup = std::fs::read_to_string("/proc/self/smaps_rollup").ok();
    let private_bytes = rollup.as_deref().map(|rollup| {
        ["Private_Clean:", "Private_Dirty:", "Private_Hugetlb:"]
            .into_iter()
            .filter_map(|key| linux_proc_kib(rollup, key))
            .sum::<u64>()
    });
    let proportional_set_size_bytes = rollup
        .as_deref()
        .and_then(|rollup| linux_proc_kib(rollup, "Pss:"));
    Some(crate::NativeProofProcessMemoryEvidence {
        source: "linux_procfs_status_smaps_rollup",
        sample_point,
        resident_bytes,
        peak_resident_bytes: peak_resident_bytes.max(resident_bytes),
        private_bytes,
        peak_private_bytes: None,
        proportional_set_size_bytes,
        virtual_bytes,
    })
}

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
fn linux_proc_kib(contents: &str, key: &str) -> Option<u64> {
    let line = contents.lines().find(|line| line.starts_with(key))?;
    let kib = line[key.len()..]
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    kib.checked_mul(1_024)
}
