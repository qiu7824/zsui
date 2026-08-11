#!/usr/bin/env bash
set -euo pipefail

workspace="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
target_dir="${CARGO_TARGET_DIR:-$workspace/target/ui-performance-matrix/build}"
binary_dir="$workspace/target/ui-performance-matrix/bin"
mkdir -p "$target_dir" "$binary_dir"
export CARGO_TARGET_DIR="$target_dir"

build_copy() {
  local output="$1"
  local built="$2"
  shift 2
  cargo build --release --locked "$@"
  cp "$target_dir/release/$built" "$binary_dir/$output"
  chmod +x "$binary_dir/$output"
}

build_copy zsui-minimal examples/ui_performance_minimal \
  --manifest-path "$workspace/Cargo.toml" \
  --example ui_performance_minimal \
  --no-default-features \
  --features window,button,label
build_copy zsui-common examples/invoice_workbench \
  --manifest-path "$workspace/Cargo.toml" \
  --example invoice_workbench \
  --no-default-features \
  --features window,workbench,list,dialog
build_copy zsui-full examples/component_gallery \
  --manifest-path "$workspace/Cargo.toml" \
  --example component_gallery \
  --no-default-features \
  --features component-gallery-demo
build_copy zsui-viewer zsui-viewer \
  --manifest-path "$workspace/Cargo.toml" \
  --bin zsui-viewer \
  --no-default-features \
  --features ui-viewer

for framework in egui iced slint; do
  manifest="$workspace/comparisons/${framework}_notepad/Cargo.toml"
  build_copy "$framework-minimal" "$framework-ui-performance" \
    --manifest-path "$manifest" \
    --bin "$framework-ui-performance" \
    --features perf-minimal
  build_copy "$framework-common" "$framework-invoice-tool" \
    --manifest-path "$manifest" \
    --bin "$framework-invoice-tool"
  build_copy "$framework-full" "$framework-ui-performance" \
    --manifest-path "$manifest" \
    --bin "$framework-ui-performance" \
    --features perf-full
  build_copy "$framework-viewer" "$framework-ui-performance" \
    --manifest-path "$manifest" \
    --bin "$framework-ui-performance" \
    --features perf-viewer
done

tauri_manifest="$workspace/comparisons/tauri_notepad/Cargo.toml"
build_copy tauri-minimal tauri-ui-performance \
  --manifest-path "$tauri_manifest" \
  --bin tauri-ui-performance \
  --no-default-features \
  --features perf-minimal
build_copy tauri-common tauri-invoice-tool \
  --manifest-path "$tauri_manifest" \
  --bin tauri-invoice-tool \
  --no-default-features \
  --features perf-common
build_copy tauri-full tauri-ui-performance \
  --manifest-path "$tauri_manifest" \
  --bin tauri-ui-performance \
  --no-default-features \
  --features perf-full
build_copy tauri-viewer tauri-ui-performance \
  --manifest-path "$tauri_manifest" \
  --bin tauri-ui-performance \
  --no-default-features \
  --features perf-viewer
