<div align="center">

# ZSUI

**Rust-first 的轻量原生 UI 框架**

用组合与 trait 构建界面，用强类型消息驱动状态；控件、服务和平台后端按 Cargo feature 进入编译。

[![CI](https://github.com/qiu7824/zsui/actions/workflows/ci.yml/badge.svg)](https://github.com/qiu7824/zsui/actions/workflows/ci.yml)
![Version](https://img.shields.io/badge/version-0.1.0-2f6fdf)
[![License](https://img.shields.io/github/license/qiu7824/zsui)](LICENSE)
![Core](https://img.shields.io/badge/core-Rust-dea584)
![Windows](https://img.shields.io/badge/Windows-Win32%20%2F%20GDI%2B-0078d4)
![Build](https://img.shields.io/badge/build-feature--gated-0f7b0f)

**简体中文** | [English](README.en.md)

</div>

<p align="center">
  <img src="docs/images/workbench.png" alt="ZSUI 工作台" width="100%">
</p>

<table>
  <tr>
    <td width="68%"><img src="docs/images/notepad.png" alt="ZSUI 记事本"></td>
    <td width="32%"><img src="docs/images/calculator.png" alt="ZSUI 计算器"></td>
  </tr>
  <tr>
    <td align="center">现代文档外壳 + 原生文本服务</td>
    <td align="center">现代标准计算器</td>
  </tr>
</table>

<p align="center"><a href="docs/gallery.md"><b>查看完整 Demo 与对比图库</b></a></p>

<details>
<summary><b>展开 ZSUI / egui / Windows 对比图</b></summary>

<h4>记事本</h4>
<table>
  <tr><th>ZSUI</th><th>eframe / egui</th><th>Windows Notepad</th></tr>
  <tr>
    <td><img src="docs/images/notepad.png" alt="ZSUI Notepad"></td>
    <td><img src="docs/images/notepad-egui.png" alt="egui Notepad"></td>
    <td><img src="docs/images/notepad-windows.png" alt="Windows Notepad"></td>
  </tr>
</table>

<h4>计算器</h4>
<table>
  <tr><th>ZSUI</th><th>Windows Calculator</th></tr>
  <tr>
    <td><img src="docs/images/calculator.png" alt="ZSUI Calculator"></td>
    <td><img src="docs/images/calculator-windows.png" alt="Windows Calculator"></td>
  </tr>
</table>

</details>

## 项目定位

ZSUI 不是浏览器壳，也不是对 WinUI 3 的运行时封装。它的目标是用 Rust
建立一套轻量、可组合、可裁剪的原生 UI 能力：

- 公共 API 安全，平台 `unsafe` 留在后端内部
- 组合和 trait 代替控件继承树
- 枚举和强类型 ID 代替字符串事件与全局注册表
- `State -> View -> Msg -> update` 显式状态循环
- `Dp`、`Px`、`Dpi` 和主题 token 管理布局与视觉
- 窗口、图标、位图和托盘资源由 RAII 管理
- 控件、服务、渲染器和平台能力通过 Cargo feature 按需编译
- 平台差异通过 capability/host trait 表达，不制造虚假的完全统一

Windows 是当前最完整的真实运行路径，包含 Win32 原生窗口、缓冲无闪屏绘制、
GDI+ 抗锯齿圆角、DPI、语义图标、输入路由和应用外壳。macOS/Linux 当前是
第一阶段桌面运行路径；Android/Harmony 仍处于宿主与设备验证建设阶段。

## 平台原生图标

应用和控件只使用 `ZsIcon` 语义值，不直接写字体私有码点。Windows 运行时先检测
系统的 Segoe Fluent Icons，不存在时使用 Windows 10 自带的 Segoe MDL2 Assets；
仓库不携带这两套字体。macOS 使用 SF Symbols 名称，Linux 使用当前 GTK 图标主题
的 symbolic 名称。系统源找不到图标时，可使用 `fluent-icons` 提供的 MIT Fluent
System Icons SVG 子集作为回退。

Windows 字体检测和 GDI 绘制已经接入真实运行路径。macOS 的 AppKit `NSImage`
查找和 Linux 的 `GtkIconTheme` 查找要随对应原生宿主完成，因此 capability 仍标记
为 partial，不会因为已有名称映射就标记为完成。详见
[平台原生图标](docs/native-icons.md)。

## 一句话创建原生窗口

```rust,no_run
fn main() -> zsui::ZsuiResult<()> {
    zsui::native_window("Example")
        .size(900, 620)
        .run()?;
    Ok(())
}
```

普通应用不需要接触 `HWND`、消息循环或 GDI 句柄。

## 强类型状态与消息

```rust,no_run
use zsui::{button, column, native_window, text, AppCx, ViewNode, WidgetId};

struct State {
    count: u32,
}

#[derive(Clone)]
enum Msg {
    Increment,
}

fn view(state: &State) -> ViewNode<Msg> {
    column([
        text(format!("Count: {}", state.count)),
        button("Increment")
            .id(WidgetId::new(1))
            .on_click(Msg::Increment),
    ])
}

fn update(state: &mut State, msg: Msg, _cx: &mut AppCx) {
    match msg {
        Msg::Increment => state.count += 1,
    }
}

fn main() -> zsui::ZsuiResult<()> {
    native_window("Counter")
        .size(480, 320)
        .stateful_view(State { count: 0 }, view, update)
        .run()?;
    Ok(())
}
```

状态所有权、消息来源和修改入口都可以被 Rust 与 rust-analyzer 检查。

## 按需编译

直接从 GitHub 使用：

```toml
[dependencies]
zsui = { git = "https://github.com/qiu7824/zsui", default-features = false, features = [
    "window",
    "button",
    "label",
    "scroll",
    "list",
    "dark-mode",
] }
```

高级能力独立开启：

```toml
zsui = { git = "https://github.com/qiu7824/zsui", default-features = false, features = [
    "workbench",
    "document-shell",
    "calculator",
    "windows-gdi",
] }
```

未开启的可选依赖不会进入构建；同一依赖图中的 Cargo feature 会取并集，因此
ZSUI 的目标是保持默认集合小、重依赖 optional，并在接口稳定后继续拆分较大的
控件与后端模块。这里承诺的是 feature/crate 级按需编译，不宣称编译器能自动
删除已启用 crate 中的每一个未调用符号。

## 已有应用外壳

| 能力 | 当前内容 | Feature |
| --- | --- | --- |
| 导航/卡片外壳 | 左侧导航、右侧内容、分组卡片、设置项、说明、操作区、滚动条 | `settings` / `full` |
| 工作台 | 会话导航、消息块、代码/工具块、编辑区、检查器 | `workbench` |
| 文档外壳 | 标签、命令栏、编辑器边框、状态栏、稳定命中区域 | `document-shell` |
| 计算器 | Decimal 运算、内存、历史、Fluent 键盘布局、语义图标 | `calculator` |
| 基础 View | 文本、按钮、输入、复选、开关、列表、滚动和强类型事件 | 对应 widget feature |
| 分页虚拟列表 | 可见区绘制、后台预取、请求去重、LRU 页缓存、稳定锚点 | `paged-list` |

组件目录当前记录 48 个 WinUI 风格家族：21 个已有第一阶段运行面，8 个只有
契约，19 个尚未开始。组合外壳可以投入示例使用，但不会被拿来冒充 DatePicker、
TreeView、DataGrid、WebView 等尚未完成的独立控件。

查看机器可读目录：

```rust
let summary = zsui::zsui_component_catalog_summary();
println!("{summary:#?}");
```

## 真实示例

### 三桌面统一示例

```powershell
cargo run --example desktop_native_showcase --features full
```

同一个 `State`、`Msg`、`view` 和 `update` 包含左侧导航、命令栏、单行/多行
输入、列表滚动、主题开关与原生菜单声明。Windows 已有真实 Win32 smoke 截图；
AppKit 与 GTK4 仍需按 [v0.2 三桌面原生闭环](docs/v0.2-desktop-native.md)
完成后端和目标机证据，当前不会把 Winit 路径标记成二者已完成。

### 十万行分页虚拟列表

```powershell
cargo run --example paged_virtual_list --no-default-features --features window,button,label,paged-list
```

示例只声明分页数据源、行视图和强类型消息。可见范围计算、后台连续预取、请求
去重、过期结果隔离和 5 页 LRU 缓存均由框架处理，详见
[分页虚拟列表](docs/paged-virtual-list.md)。

### 工作台

```powershell
cargo run --example workbench_shell --features full
```

### 现代记事本

```powershell
cargo run --example zsui_notepad --features notepad-demo
```

它组合自绘文档外壳和 Windows 原生多行文本服务，保留 IME 与原生编辑行为。
[测量说明](docs/notepad-demo.md)记录了代码量、包体和运行内存。

### 现代计算器

```powershell
cargo run --example zsui_calculator --no-default-features --features calculator-demo
```

标准模式包含四则运算、上下文百分比、连续等号、倒数、平方、开方、内存、
历史和键盘输入。一次本机 release 测量中，可执行文件为 0.28 MiB，任务管理器
私有工作集为 1.24 MiB；这只是可复现的单机观测，不是所有设备上的固定值。
[完整对比](docs/calculator-demo.md)同时记录了本机 Windows 计算器的独立进程与
窗口宿主进程，避免混用不同内存指标。

## 平台状态

| 平台 | 当前状态 | 说明 |
| --- | --- | --- |
| Windows | 真实运行路径 | Win32 窗口、缓冲绘制、输入、DPI、图标、托盘基础能力 |
| macOS | 第一阶段桌面路径 | 当前通过 Winit 启动；完整 AppKit 绑定与目标机证据仍待完成 |
| Linux | 第一阶段桌面路径 | 当前通过 Winit 启动；完整 GTK/libadwaita 绑定仍待完成 |
| Android | 宿主契约 | Activity/FFI 与真实设备运行仍待完成 |
| Harmony | 宿主契约 | Ability/FFI 与真实设备运行仍待完成 |

平台能力必须经过代码、目标机 smoke 和系统集成三层证据。仅有声明或脚手架时，
不会标记为完成。

## 为 AI 节约上下文

AI 不应该每次先读取整个仓库。ZSUI 提供了一个小型入口和按任务选择的上下文包：

1. 首次只读 [`docs/ai-agent.md`](docs/ai-agent.md)。
2. 查看可选任务包：

   ```powershell
   .\scripts\ai-context.ps1 -List
   ```

3. 选择当前任务，例如：

   ```powershell
   .\scripts\ai-context.ps1 -Pack calculator
   .\scripts\ai-context.ps1 -Pack windows-renderer -IncludeOptional
   ```

4. 只读取脚本返回的 required 文件；遇到阻塞再读取 optional 文件。

任务包清单位于 [`docs/ai/context-packs.json`](docs/ai/context-packs.json)。完整进度、
平台与接口参考被放在按需文档中，不再塞进默认 AI 首读文件。这与控件 feature
的思路一致：先加载最小核心，再按任务组合需要的上下文。

## 目录

- `src/`：公共 API、运行时、布局、协议和平台后端
- `examples/`：可运行的窗口、控件、工作台、记事本和计算器
- `docs/`：架构、目标机验证、应用测量和 AI 文档
- `docs/ai-agent.md`：AI 最小首读入口
- `docs/ai/context-packs.json`：AI 按需上下文包
- `scripts/check-feature-matrix.ps1`：全部公开 feature 检查
- `scripts/ai-context.ps1`：按任务输出最小文件集合

核心边界请阅读：

- [架构](docs/architecture.md)
- [Rust-first 目标](docs/framework-goals.md)
- [平台宿主约束](docs/porting.md)
- [目标机验证](docs/native-host-smoke.md)

## 验证

```powershell
cargo fmt --check
cargo test --no-default-features
cargo test --features full
.\scripts\ai-context.ps1 -Validate
.\scripts\check-feature-matrix.ps1 -Locked
```

CI 同时检查默认/无默认 feature、Windows 全功能构建、feature 矩阵，以及
Linux/macOS 桌面目标。

## 当前边界

- Windows 仍需更完整的 UI Automation、暗色/高对比度和高级输入证据
- 通用文本编辑器、文件对话框和文档生命周期服务仍需继续收口
- DatePicker、TreeView、DataGrid、WebView 等高级控件尚未完整实现
- macOS、Linux、Android 和 Harmony 需要真实目标机运行与截图证据
- 大型控件/后端将在公共契约稳定后继续拆分 crate 或 feature 模块

## 赞赏支持

如果这个项目对你有帮助，欢迎支持我继续完善 Rust 原生 UI 能力。

![赞赏支持](docs/images/donate.png)

## 许可证

本项目使用 [GPL-3.0-only](LICENSE) 许可证。
内置的 Fluent System Icons SVG 回退资源使用 MIT 许可证，详见
[第三方许可说明](THIRD_PARTY_NOTICES.md)。
