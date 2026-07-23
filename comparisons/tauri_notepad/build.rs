fn main() {
    let config = if std::env::var_os("CARGO_FEATURE_PERF_MINIMAL").is_some() {
        "tauri.perf-minimal.conf.json"
    } else if std::env::var_os("CARGO_FEATURE_PERF_COMMON").is_some() {
        "tauri.perf-common.conf.json"
    } else if std::env::var_os("CARGO_FEATURE_PERF_FULL").is_some() {
        "tauri.perf-full.conf.json"
    } else if std::env::var_os("CARGO_FEATURE_PERF_VIEWER").is_some() {
        "tauri.perf-viewer.conf.json"
    } else {
        "tauri.conf.json"
    };
    if config != "tauri.conf.json" {
        let overlay = std::fs::read_to_string(config)
            .unwrap_or_else(|error| panic!("failed to read {config}: {error}"));
        let compact_overlay = overlay.lines().map(str::trim).collect::<String>();
        // Cargo runs this build script as a dedicated process, before any worker
        // threads exist, so changing its private environment is race-free.
        unsafe { std::env::set_var("TAURI_CONFIG", &compact_overlay) };
        println!("cargo:rustc-env=TAURI_CONFIG={compact_overlay}");
        println!("cargo:rerun-if-changed={config}");
    }
    tauri_build::build();
}
