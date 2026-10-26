<div align="center">

# ZSUI

**Rust 原生桌面 UI 框架**

用同一份 Rust 状态、消息和界面声明构建 Windows、macOS 与 Linux 应用。

[![crates.io](https://img.shields.io/crates/v/zsui?include_prereleases)](https://crates.io/crates/zsui)
[![Docs](https://docs.rs/zsui/badge.svg)](https://docs.rs/zsui)
[![CI](https://github.com/qiu7824/zsui/actions/workflows/ci.yml/badge.svg)](https://github.com/qiu7824/zsui/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/qiu7824/zsui)](https://github.com/qiu7824/zsui/blob/main/LICENSE)

[简体中文](https://github.com/qiu7824/zsui/blob/main/README.md) · [English](https://github.com/qiu7824/zsui/blob/main/README.en.md)

</div>

ZSUI 使用真实的系统窗口和平台服务，在窗口内绘制由框架管理的控件。
应用通过 `State → view → Msg → update` 更新界面，平台后端负责窗口事件、
文字、绘制、输入法和系统集成。当前版本为 **`0.3.0-alpha.2`**，属于预发布版本。

[快速开始](#快速开始) · [Cargo features](#cargo-features) · [平台能力](#平台与能力边界) · [示例](#运行示例) · [文档](#文档导航)

## 能力概览

- **一份应用代码**：共享状态、消息、布局和组件声明，使用 Win32、AppKit 或 Linux 原生宿主。
- **按需编译**：控件、系统服务和后端通过 Cargo feature 选择，默认启用 `window`、`button` 和 `label`。
- **显式状态与身份**：类型化消息、强类型 ID、受控输入和稳定的组件身份。
- **平台视觉**：语义主题、系统字体、图标、DPI 和平台组件参数共同决定最终界面。
- **开发与发布两条路径**：直接编写 Rust View，或使用可校验、可热重载、可嵌入的 UI 文档。
- **真实窗口验证**：CI 运行目标平台应用，保留最终窗口截图及布局、输入、焦点等结构化报告。

组件目录包含 **47 个组件家族的运行实现**。各组件的交互、辅助功能和平台覆盖
仍有差异，具体范围见[组件目录](https://github.com/qiu7824/zsui/blob/main/src/component_catalog.rs)。

## 界面预览

Gallery 在三个桌面平台上的实际窗口截图：

<table>
  <tr><th>Windows · Win32</th><th>macOS · AppKit</th><th>Linux · X11</th></tr>
  <tr>
    <td><img src="https://raw.githubusercontent.com/qiu7824/zsui/main/docs/images/component-gallery-preview7-windows.png" alt="Windows Win32 组件界面" width="320"></td>
    <td><img src="https://raw.githubusercontent.com/qiu7824/zsui/main/docs/platform-proof/macos/gallery-inputs-light.png" alt="macOS AppKit 组件界面" width="320"></td>
    <td><img src="https://raw.githubusercontent.com/qiu7824/zsui/main/docs/platform-proof/linux/gallery-inputs-light.png" alt="Linux X11 组件界面" width="320"></td>
  </tr>
</table>

截图展示各平台的组件布局与视觉；窗口尺寸不同，逐像素比较应使用固定场景的验证报告。
更多记事本、计算器、组件与框架对比图见[示例图库](https://github.com/qiu7824/zsui/blob/main/docs/gallery.md)。

## 快速开始

在 Rust 二进制项目的 `Cargo.toml` 中添加：

```toml
[dependencies]
zsui = "0.3.0-alpha.2"
```

将下面的代码保存为 `src/main.rs`：

```rust,no_run
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use zsui::prelude::{button, column, text, window, Dp, Element, UpdateContext};

#[derive(Clone)]
enum Message {
    Increment,
}

struct State {
    count: u32,
}

fn view(state: &State) -> Element<Message> {
    column([
        text(format!("Count: {}", state.count)),
        button("Increment").on_click(Message::Increment),
    ])
    .gap(Dp::new(12.0))
    .padding(Dp::new(20.0))
}

fn update(state: &mut State, message: Message, _cx: &mut UpdateContext<'_>) {
    match message {
        Message::Increment => state.count += 1,
    }
}

fn main() -> Result<(), zsui::stable::Error> {
    window("ZSUI")
        .size(480, 320)
        .stateful(State { count: 0 }, view, update)
        .run()
}
```

```sh
cargo run
```

按钮产生 `Message`，`update` 修改应用状态，运行时随后重建界面并重绘。
普通应用从 `zsui::prelude` 或 `zsui::stable` 导入；接口兼容范围见
[API 稳定性](https://github.com/qiu7824/zsui/blob/main/docs/api-stability.md)。

### 构建环境

使用当前稳定版 Rust 工具链及对应平台的本地开发工具：

| 平台 | 环境 |
| --- | --- |
| Windows | MSVC 工具链、Visual Studio Build Tools 的 C++ 构建工具与 Windows SDK |
| macOS | Xcode Command Line Tools |
| Linux 默认后端 | C/C++ 构建工具、pkg-config、D-Bus、Cairo 和 Pango 开发包 |
| Linux GTK4 后端 | 在对应构建环境中额外安装 GTK4 4.10 或更新版本的开发包 |

Ubuntu 24.04 的默认 Linux 后端依赖可安装为：

```sh
sudo apt-get install build-essential pkg-config libdbus-1-dev libcairo2-dev libpango1.0-dev
```

选择 `linux-gtk` 时还需要 `libgtk-4-dev`。其他发行版安装对应开发包。
后端选择与移植约束见[平台移植](https://github.com/qiu7824/zsui/blob/main/docs/porting.md)。

## Cargo features

默认配置提供窗口、按钮与文本。需要额外控件时按功能启用：

```toml
[dependencies]
zsui = { version = "0.3.0-alpha.2", default-features = false, features = [
    "window", "button", "label", "textbox", "clipboard", "dialog",
] }
```

| 用途 | 主要 features |
| --- | --- |
| 窗口与后端 | `window`、`windows-win32`、`macos-appkit`、`linux-direct`、`linux-direct-lite`、`linux-gtk` |
| 基础界面与布局 | `button`、`label`、`icon`、`grid`、`scroll`、`split-view`、`resize` |
| 文本与数值输入 | `textbox`、`password-box`、`number-box`、`auto-suggest`、`command-palette` |
| 选择与日期 | `checkbox`、`toggle`、`toggle-button`、`radio`、`slider`、`combo`、`date-picker`、`time-picker`、`color-picker` |
| 集合 | `list`、`virtual-list`、`paged-list`、`grid-view`、`tree`、`table` |
| 弹层与反馈 | `dialog`、`flyout`、`menu-flyout`、`context-menu`、`tooltip`、`toast`、`info-bar`、`teaching-tip`、`progress`、`progress-ring` |
| 组合组件 | `shell`、`document-shell`、`tabs`、`accordion`、`breadcrumb`、`badge`、`calculator` |
| 媒体 | `image`、`image-preview`、`video` |
| 系统与主题 | `clipboard`、`tray`、`hotkey`、`settings`、`style`、`dark-mode`、`localization` |
| 辅助功能 | `accessibility`、`linux-direct-accessibility` |
| UI 文档与工具 | `ui-document`、`ui-document-runtime`、`ui-viewer` |
| 汇总配置 | `all-widgets`、`full` |

`window` 通过目标平台依赖选择默认桌面宿主。Linux 默认使用 `linux-direct`；
选择 GTK4 或 `linux-direct-lite` 时，关闭默认 features 并显式选择后端，避免同时
带入默认 Linux 渲染栈。`linux-direct-lite` 使用 tiny-skia 和框架的 Rust 文字引擎。

`accessibility` 提供共享语义及 Win32、AppKit、GTK4 桥接；Linux Direct 的
AccessKit/AT-SPI 桥接另外启用 `linux-direct-accessibility`。
Cargo features 在依赖图中取并集，`full` 会开启广泛的控件、后端和验证能力，
适合框架验证；应用应选择自己使用的功能。项目目前以单个 crate 发布。

## 平台与能力边界

| 平台 | 原生宿主与绘制 | 系统集成 |
| --- | --- | --- |
| Windows | Win32 窗口、缓冲 GDI/GDI+ 绘制，默认使用 Rust 文字引擎 | IME、UI Automation、剪贴板、文件面板、菜单、托盘 |
| macOS | NSApplication/NSWindow、AppKit/Core Graphics、Core Text | NSTextInputClient、NSAccessibility、剪贴板、文件面板、菜单、NSStatusItem |
| Linux | 默认 Wayland/X11 + Softbuffer + Cairo/Pango；可选 tiny-skia 或 GTK4 宿主 | IME、剪贴板、XDG portal、菜单；AT-SPI 由所选辅助功能桥接提供 |
| Android | 宿主、生命周期与 FFI 契约 | Activity 运行实现和真机验证仍待完成 |

控件由 ZSUI 在原生窗口中绘制；平台宿主提供系统窗口、事件循环和桌面服务，
组件布局与视觉参数由各平台实现管理。系统服务的可用范围随后端和桌面环境变化，
以能力报告及操作返回值为准。Linux Direct 的托盘宿主尚未接入统一事件循环。

应用使用语义图标。Windows 使用系统图标字体，macOS 使用 SF Symbols；
Linux 默认使用内置符号图形，精确的桌面图标主题解码通过 `linux-system-icons` 启用。
SVG 回退及许可见[平台图标](https://github.com/qiu7824/zsui/blob/main/docs/native-icons.md)。

输入法协议、光标几何和辅助功能桥接有自动化验证；真实候选窗位置与
Narrator/NVDA、VoiceOver、Orca 的朗读和焦点体验还需目标环境人工检查。
触摸与惯性滚动、复杂富文本、部分集合控件的编辑/虚拟化，以及 Android 运行时
仍在扩展。组件与平台的运行实现、目标验证和完整体验分别记录。

## 状态、后台任务与资源

应用拥有业务状态，界面通过类型化消息更新。普通控件可使用结构路径生成的自动身份；
列表重排、跨组件引用或外部定向操作应提供显式稳定 ID。

后台任务使用窗口的 `InvalidationHandle` 请求重建。请求会合并，并由原生 UI 线程
执行状态化界面重建。文件操作、异步任务与原生服务的使用见
[应用效果与命令](https://github.com/qiu7824/zsui/blob/main/docs/app-effects.md)。

文字引擎使用按条目及字节限制的布局/字形缓存，并定期老化成形 run。
这些上限约束对应的框架缓存；应用数据、系统字体和平台资源仍有各自的生命周期。
窗口可选择 `release_view_when_hidden()` 释放隐藏期间的界面资源；Wayland 等宿主对
主动隐藏、恢复和焦点请求有平台限制。详见
[文字渲染](https://github.com/qiu7824/zsui/blob/main/docs/text-rendering.md)、
[桌面运行约束](https://github.com/qiu7824/zsui/blob/main/docs/desktop-hardening-plan.md)。

## UI 文档与原生 Viewer

`ui-document` 定义版本化的语义组件树和类型化绑定清单。Rust 持有状态与消息，
文档描述组件、布局、主题、本地化键、辅助功能和稳定节点 ID。

- `zsui-uic check`：检查 schema、features、属性、节点身份和绑定类型。
- `zsui-viewer`：在原生窗口中重载有效文档，保留兼容的焦点、选择、滚动和控件状态。
- `zsui-uic embed` 与 `ui-document-runtime`：将验证后的文档嵌入发布应用，按所用组件裁剪。

在仓库根目录运行校验器或 Viewer：

```sh
cargo run --locked --bin zsui-uic --no-default-features --features ui-document,button,label -- check examples/ui-documents/basic.json --bindings examples/ui-documents/basic.bindings.json
cargo run --locked --bin zsui-viewer --no-default-features --features ui-viewer -- examples/ui-documents/basic.json --bindings examples/ui-documents/basic.bindings.json --values examples/ui-documents/basic.values.json
```

Viewer 编译后，保存兼容的文档修改即可更新同一个窗口，无需重新调用 Cargo。
正式应用使用 `ui-document-runtime`，Viewer 的轮询、开发诊断和全组件配置属于独立工具。
格式、绑定、密码安全通道与发布嵌入见
[UI Document](https://github.com/qiu7824/zsui/blob/main/docs/ui-document.md)。

## 运行示例

克隆仓库后，在仓库根目录执行：

```sh
cargo run --locked --example stable_api
cargo run --locked --example component_gallery --no-default-features --features component-gallery-demo
cargo run --locked --example zsui_notepad --no-default-features --features notepad-demo
cargo run --locked --example zsui_calculator --no-default-features --features calculator-demo
```

| 示例 | 内容 | 文档 |
| --- | --- | --- |
| `component_gallery` | 组件、主题和平台布局 | [图库](https://github.com/qiu7824/zsui/blob/main/docs/gallery.md) |
| `zsui_notepad` | 同一份三平台记事本、编辑命令与文件面板 | [记事本](https://github.com/qiu7824/zsui/blob/main/docs/notepad-demo.md) |
| `zsui_calculator` | Decimal 运算、内存与历史、自适应键盘 | [计算器](https://github.com/qiu7824/zsui/blob/main/docs/calculator-demo.md) |
| `resizable_layout` | 导航栏、SplitView 与 DataGrid 的受控尺寸 | [可调整布局](https://github.com/qiu7824/zsui/blob/main/docs/resizable-layout.md) |
| `paged_virtual_list` | 后台分页、预取、稳定滚动锚点和 LRU 页缓存 | [分页虚拟列表](https://github.com/qiu7824/zsui/blob/main/docs/paged-virtual-list.md) |
| `image_preview` | 后台 PNG 解码、请求合并与完整帧切换 | [图片预览](https://github.com/qiu7824/zsui/blob/main/docs/image-preview.md) |
| `zsui_proxy_client` | VLESS 分享链接解析、模拟流量刷新与内存/闪屏对比 | [代理演示](https://github.com/qiu7824/zsui/blob/main/docs/proxy-demo.md) |

`video` 接收应用提供的最新像素帧并处理布局与呈现；摄像头采集、编解码、音频、
拖动进度和字幕由媒体适配层负责。接口范围见
[视频与帧源](https://github.com/qiu7824/zsui/blob/main/docs/video.md)。

## 文档导航

| 主题 | 文档 |
| --- | --- |
| 架构与职责 | [架构](https://github.com/qiu7824/zsui/blob/main/docs/architecture.md) · [框架目标](https://github.com/qiu7824/zsui/blob/main/docs/framework-goals.md) |
| API 与兼容性 | [Rustdoc](https://docs.rs/zsui) · [API 稳定性](https://github.com/qiu7824/zsui/blob/main/docs/api-stability.md) |
| 布局、主题与平台视觉 | [平台设计契约](https://github.com/qiu7824/zsui/blob/main/docs/platform-design-contracts.md) · [可调整布局](https://github.com/qiu7824/zsui/blob/main/docs/resizable-layout.md) |
| 本地化 | [Fluent 与语言切换](https://github.com/qiu7824/zsui/blob/main/docs/localization.md) |
| 输入与辅助功能 | [输入体验](https://github.com/qiu7824/zsui/blob/main/docs/v0.3-input-experience.md) · [人工检查清单](https://github.com/qiu7824/zsui/blob/main/docs/quality/manual-assistive-technology-checklist.md) |
| 原生验证 | [Native Proof CI](https://github.com/qiu7824/zsui/blob/main/docs/v0.3-native-proof-ci.md) · [目标机验证](https://github.com/qiu7824/zsui/blob/main/docs/native-host-smoke.md) |
| 性能测量 | [四档性能矩阵](https://github.com/qiu7824/zsui/blob/main/docs/ui-performance-matrix.md) · [应用对比](https://github.com/qiu7824/zsui/blob/main/docs/invoice-tool-comparison.md) |
| 分发与更新 | [安装包、签名与更新清单](https://github.com/qiu7824/zsui/blob/main/docs/distribution.md) |
| 发布记录 | [0.3.0-alpha.2](https://github.com/qiu7824/zsui/blob/main/docs/releases/v0.3.0-alpha.2.md) |

性能文档按平台、构建 features、窗口场景和工作负载记录结果。
Minimal、Common、Full Native App 与 Viewer 分别测量，比较时应保持相同条件。

## 开发与验证

```powershell
cargo fmt --all -- --check
cargo test --locked --no-default-features
cargo test --locked --features full
cargo check --locked --manifest-path tools/native-proof/Cargo.toml
.\scripts\check-native-boundary.ps1 -Locked
.\scripts\check-feature-matrix.ps1 -Locked
```

CI 还运行桌面目标编译、原生交互、最终表面截图和 ARM64 验证。
贡献代码前阅读[开发入口](https://github.com/qiu7824/zsui/blob/main/docs/ai-agent.md)，
按模块加载对应上下文和检查命令。

## 支持与许可证

ZSUI 使用 [MPL-2.0](https://github.com/qiu7824/zsui/blob/main/LICENSE)。
内置 Fluent System Icons SVG 回退资源使用 MIT 许可证，详见
[第三方许可](https://github.com/qiu7824/zsui/blob/main/THIRD_PARTY_NOTICES.md)。

欢迎通过 [GitHub Star](https://github.com/qiu7824/zsui) 或赞赏支持项目。

<details>
<summary>赞赏支持</summary>

![赞赏支持](https://raw.githubusercontent.com/qiu7824/zsui/main/docs/images/donate.png)

</details>

[![GitHub 收藏量趋势](https://raw.githubusercontent.com/qiu7824/zsui/main/docs/images/star-history.svg)](https://github.com/qiu7824/zsui/stargazers)
