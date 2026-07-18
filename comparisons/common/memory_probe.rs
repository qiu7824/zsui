use std::path::Path;

#[derive(Debug, Clone, Copy)]
struct MemorySample {
    source: &'static str,
    resident_bytes: u64,
    peak_resident_bytes: u64,
    private_resident_bytes: Option<u64>,
    proportional_set_size_bytes: Option<u64>,
    physical_footprint_bytes: Option<u64>,
    peak_physical_footprint_bytes: Option<u64>,
    virtual_bytes: Option<u64>,
}

pub fn write_report(
    path: &Path,
    framework: &str,
    scenario: &str,
    sample_point: &str,
) -> Result<(), String> {
    let sample = capture().ok_or_else(|| "process memory is unavailable".to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let json = format!(
        concat!(
            "{{\n",
            "  \"schema\": \"zsui.ui-memory-comparison/v1\",\n",
            "  \"framework\": \"{}\",\n",
            "  \"scenario\": \"{}\",\n",
            "  \"platform\": \"{}\",\n",
            "  \"architecture\": \"{}\",\n",
            "  \"sample_point\": \"{}\",\n",
            "  \"source\": \"{}\",\n",
            "  \"resident_bytes\": {},\n",
            "  \"peak_resident_bytes\": {},\n",
            "  \"private_resident_bytes\": {},\n",
            "  \"proportional_set_size_bytes\": {},\n",
            "  \"physical_footprint_bytes\": {},\n",
            "  \"peak_physical_footprint_bytes\": {},\n",
            "  \"virtual_bytes\": {}\n",
            "}}\n"
        ),
        framework,
        scenario,
        std::env::consts::OS,
        std::env::consts::ARCH,
        sample_point,
        sample.source,
        sample.resident_bytes,
        sample.peak_resident_bytes,
        optional_number(sample.private_resident_bytes),
        optional_number(sample.proportional_set_size_bytes),
        optional_number(sample.physical_footprint_bytes),
        optional_number(sample.peak_physical_footprint_bytes),
        optional_number(sample.virtual_bytes),
    );
    std::fs::write(path, json).map_err(|error| error.to_string())
}

fn optional_number(value: Option<u64>) -> String {
    value.map_or_else(|| "null".to_string(), |value| value.to_string())
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn capture() -> Option<MemorySample> {
    let mut task = unsafe { std::mem::zeroed::<libc::mach_task_basic_info>() };
    let mut task_count = libc::MACH_TASK_BASIC_INFO_COUNT;
    let task_result = unsafe {
        libc::task_info(
            libc::mach_task_self(),
            libc::MACH_TASK_BASIC_INFO as libc::task_flavor_t,
            (&mut task as *mut libc::mach_task_basic_info).cast::<libc::integer_t>(),
            &mut task_count,
        )
    };
    if task_result != libc::KERN_SUCCESS {
        return None;
    }

    let resident_bytes = unsafe { std::ptr::addr_of!(task.resident_size).read_unaligned() };
    let peak_resident_bytes =
        unsafe { std::ptr::addr_of!(task.resident_size_max).read_unaligned() };
    let virtual_bytes = unsafe { std::ptr::addr_of!(task.virtual_size).read_unaligned() };
    let mut usage = unsafe { std::mem::zeroed::<libc::rusage_info_v4>() };
    let usage_result = unsafe {
        libc::proc_pid_rusage(
            libc::getpid(),
            libc::RUSAGE_INFO_V4,
            (&mut usage as *mut libc::rusage_info_v4).cast::<libc::rusage_info_t>(),
        )
    };
    let (physical_footprint_bytes, peak_physical_footprint_bytes) = if usage_result == 0 {
        (
            Some(usage.ri_phys_footprint),
            Some(usage.ri_lifetime_max_phys_footprint),
        )
    } else {
        (None, None)
    };

    Some(MemorySample {
        source: "macos_mach_task_basic_info_proc_pid_rusage_v4",
        resident_bytes,
        peak_resident_bytes: peak_resident_bytes.max(resident_bytes),
        private_resident_bytes: None,
        proportional_set_size_bytes: None,
        physical_footprint_bytes,
        peak_physical_footprint_bytes,
        virtual_bytes: Some(virtual_bytes),
    })
}

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
fn capture() -> Option<MemorySample> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let rollup = std::fs::read_to_string("/proc/self/smaps_rollup").ok()?;
    let resident_bytes = proc_kib(&status, "VmRSS:")?;
    let peak_resident_bytes = proc_kib(&status, "VmHWM:").unwrap_or(resident_bytes);
    let private_resident_bytes = sum_proc_kib(
        &rollup,
        &["Private_Clean:", "Private_Dirty:", "Private_Hugetlb:"],
    );

    Some(MemorySample {
        source: "linux_procfs_status_smaps_rollup",
        resident_bytes,
        peak_resident_bytes: peak_resident_bytes.max(resident_bytes),
        private_resident_bytes,
        proportional_set_size_bytes: proc_kib(&rollup, "Pss:"),
        physical_footprint_bytes: None,
        peak_physical_footprint_bytes: None,
        virtual_bytes: proc_kib(&status, "VmSize:"),
    })
}

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
fn proc_kib(contents: &str, key: &str) -> Option<u64> {
    let line = contents.lines().find(|line| line.starts_with(key))?;
    let kib = line[key.len()..]
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    kib.checked_mul(1_024)
}

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
fn sum_proc_kib(contents: &str, keys: &[&str]) -> Option<u64> {
    let values = keys
        .iter()
        .filter_map(|key| proc_kib(contents, key))
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| values.into_iter().sum())
}

#[cfg(not(any(
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
)))]
fn capture() -> Option<MemorySample> {
    None
}
