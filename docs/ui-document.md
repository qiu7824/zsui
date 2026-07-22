# ZSUI UI Document

`ui-document` 是可选的开发与嵌入能力，不属于默认 feature。它定义版本化的语义
组件树、DP 布局约束、主题 token、本地化键、辅助功能元数据和稳定节点 ID；平台
后端仍负责生成各自的 Win32、AppKit 或 Linux 原生体验。

UI 文档只保存视觉结构。Rust 应用继续持有 `State` 和 `Msg`，并通过
`UiBindingManifest<State, Msg>` 注册状态读取器和消息映射器。序列化绑定名称只有
在显式清单中声明且类型一致时才有效，不形成反射式属性系统或全局字符串事件总线。

## 校验

启用文档使用到的组件 feature，并运行：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,button,label `
  -- check examples/ui-documents/basic.json `
  --bindings examples/ui-documents/basic.bindings.json
```

`zsui-uic check` 会拒绝：

- 不兼容的 schema 版本；
- 无效或重复的稳定节点 ID；
- 未知组件和当前尚未进入文档 schema 的组件；
- 未启用对应 Cargo feature 的组件；
- 未知属性、属性类型错误和非法子节点数量；
- 未解析或类型不一致的状态、动作绑定；
- 非法布局值、主题 token、本地化键和辅助功能字段。

加上 `--json` 可输出确定性结构化诊断。当前第一阶段支持 `stack`、`border`、
`text`、`button`、`toggle_button`、`checkbox`、`toggle`、`textbox`、
`radio_button`、`slider`、`progress_bar` 和 `scroll`。其他已存在的 ZSUI 组件会被识别为
“尚未进入 UiDocument schema”，不会被误报为未知组件。

`scroll` 必须有且只有一个内容子节点，并要求非负的 `content_height` 数值属性。
`offset_y` 是可选的非负数值属性；需要在 View 重建后保留位置时，将它绑定到 number
属性，同时把 `scroll` 动作绑定到 number 动作。滚动事件产生的新偏移会更新该显式
属性状态，下一棵 View 树继续使用同一偏移；布局会按当前视口和内容高度夹取越界值。

## 原生 Viewer

`ui-viewer` 是独立的可选开发 feature。只需编译 Viewer 一次：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/basic.json `
  --bindings examples/ui-documents/basic.bindings.json `
  --values examples/ui-documents/basic.values.json
```

Viewer 直接进入当前目标的 Win32、AppKit 或 Linux 原生宿主。运行期间每 250 ms
检查一次文档和绑定 schema；保存有效修改后，同一个窗口重建真实 `ViewNode` 树，
不重新运行 Cargo。无效修改显示诊断并继续保留最后一份有效文档。`--poll-ms`
可以调整检查周期，`--width` 和 `--height` 可以固定窗口尺寸。

带值交互示例：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/interactive.json `
  --bindings examples/ui-documents/interactive.bindings.json `
  --values examples/ui-documents/interactive.values.json
```

每个 `UiNodeId` 都确定性映射到文档专用的 `WidgetId` 命名空间。Viewer 的属性值和
动作历史位于普通 `UiViewerState` 中，View 重建不会替换该状态。每次接受有效修改后，
`UiViewerSourceSnapshot::last_reload` 都会确定性列出保留、新增和必须重置的节点 ID。
节点 ID 和控件状态类别均兼容时，原生输入运行时保留焦点、文本选择以及文本编辑器的
纵向/横向视口；节点被删除、同一 ID 改为其他控件类型或文本框在单行/多行间切换时，
旧焦点、选择、拖动和 IME 临时态会显式清除，避免把旧控件状态错误路由到新控件。
`button.click`、`radio_button.choose`、`textbox.change`、`toggle_button.toggle`、
`checkbox.toggle`、`toggle.toggle`、`slider.slide` 和 `scroll.scroll` 均走类型化 Viewer
消息。带值控件
通过按控件持有的 `ViewMessageMapper` 捕获稳定节点 ID、动作绑定和可选属性绑定；普通
函数指针路径不分配堆内存，只有显式使用 `*_with` 捕获回调时才分配共享闭包。动作 payload
通过 binding schema 校验后记录在 `UiViewerState`；存在对应属性绑定时，Viewer 同步
更新该显式状态值，使受控文本、开关、滑块和滚动交互不会在 View 重建后回退。这里没有
全局注册表、反射属性或字符串事件总线。

受控滚动示例使用相同的机制：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/scrolling.json `
  --bindings examples/ui-documents/scrolling.bindings.json `
  --values examples/ui-documents/scrolling.values.json
```

`scroll.offset_y` 与 `scroll.scroll` 的 number 绑定使原生滚轮输入、显式 Viewer 状态和
重建后的新 View 树形成闭环。

原生证明可由同一可执行文件生成：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/scrolling.json `
  --bindings examples/ui-documents/scrolling.bindings.json `
  --values examples/ui-documents/scrolling.values.json `
  --smoke target/ui-viewer-proof `
  --smoke-scroll 360 260 96
```

输出包含平台最终窗口截图 `window.png` 和带窗口、字体、内存、绘制及输入证据的
`proof.json`。`--smoke-scroll x y delta-y` 可选；提供后，smoke 只有在目标原生宿主
真实路由该滚动输入并增加 `native_view_scroll_count` 时才通过。

`proof.json` 使用 `zsui.ui-viewer-proof/v1`。顶层记录实际平台、最终视图捕获后端、
显示服务器以及逻辑/像素窗口尺寸；`source.nodes` 按组件树先序稳定输出文档路径、节点 ID、
确定性 `WidgetId`、组件、布局约束和子节点数量；`runtime` 继续记录焦点、事件、消息、
滚动处理、字体、最终平台视图捕获和进程内存。该报告索引 UiDocument 结构，但截图必须
来自 AppKit `NSView` 或 Linux 最终呈现表面，不能用共享 DrawPlan PNG 代替。

固定 Native Proof CI 在 `macos-15` AppKit 和 Ubuntu 24.04 Linux Direct 上运行同一份
`scrolling.json`，注入同一滚动场景并校验结构报告、类型化消息、内存采样和最终 PNG。
Native UI Proof 运行 `29883039068` 已在提交
`348808b6f5b862d90c19d8687a15f991e8790344` 上通过两项固定目标：两份报告均包含
15 个确定性节点、1 次已处理滚动、1 条 Viewer 消息和目标最终表面 PNG。该次托管运行
记录的驻留内存约为 AppKit 61.83 MiB、Linux Direct 27.26 MiB；这些数值是单次 Runner
证据，不作为跨机器性能基准。

## 确定性 AI 交接包

`zsui-uic handoff` 把已校验的文档、绑定 schema、可选值快照和可选原生最终视图 PNG
整理为一个稳定目录：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-viewer `
  -- handoff examples/ui-documents/interactive.json `
  --bindings examples/ui-documents/interactive.bindings.json `
  --values examples/ui-documents/interactive.values.json `
  --output target/ui-ai-handoff `
  --preview target/ui-viewer-proof/window.png `
  --force
```

输出目录固定包含 `document.json`、`bindings.json` 和 `handoff.json`，按输入情况增加
`values.json` 与 `preview.png`。`handoff.json` 使用稳定排序且不写入时间、绝对路径或
随机 ID，记录：

- 文件字节数和 `fnv1a64` 内容变更指纹；该指纹用于确定性变更检测，不作为密码学完整性校验；
- ZSUI 与文档 schema 版本、所需 Cargo features；
- 每个节点的文档路径、稳定 ID、`WidgetId`、属性、绑定和子节点数量；
- 实际用到的组件属性、动作 payload 与子节点契约；
- 已提供和缺失的属性值，以及可选 PNG 的尺寸和媒体类型。

交接前会再次执行文档、feature、绑定和值类型校验；非法 PNG 也会被拒绝。已存在的非空
输出目录默认不会覆盖，显式 `--force` 只替换上述固定交接文件，不递归删除目录中的其他
内容。AI 或可视化工具修改 `document.json` 后，可以直接用 `zsui-uic check` 校验，并由
已编译 Viewer 热重载；交接包不引入 WebView、平台类型、文件监听服务或全局控件注册表。

浏览器投影不能替代原生运行证据。

## 发布嵌入

发布构建可先把文档与绑定 schema 编译为确定性的 `.zsui` 制品：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,label,checkbox,textbox,slider `
  -- embed examples/ui-documents/interactive.json `
  --bindings examples/ui-documents/interactive.bindings.json `
  --output target/release-ui/interactive.zsui `
  --force
```

`embed` 在写入前执行与 Viewer 相同的 schema、组件 feature、稳定 ID、属性和绑定校验。
制品头包含固定 magic、制品版本、文档 schema、分段长度和 payload 变更指纹，payload
只含规范化文档与绑定 schema；不含源文件路径、时间、截图、文件监听状态或诊断历史。
payload 指纹使用 FNV-1a 检测意外变更，不是密码学完整性或签名机制。

应用使用 `include_bytes!` 把制品放进最终二进制，并用应用真实的强类型绑定清单校验和
解码：

```rust
use zsui::ui_document::{UiEmbeddedDocument, UiFeatureSet};
use zsui::ui_document_runtime::{ui_document_view, UiDocumentAction};

static MAIN_UI: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/main.zsui"));

#[derive(Clone)]
enum Msg {
    Ui(UiDocumentAction),
}

let binding_schema = binding_manifest.schema();
let embedded = UiEmbeddedDocument::decode(
    MAIN_UI,
    &UiFeatureSet::compiled(),
    &binding_schema,
)?;
let view = ui_document_view(
    &embedded.document,
    &embedded.bindings,
    &state.ui_values,
    Msg::Ui,
)?;
```

发布应用启用 `ui-document-runtime` 以及文档实际使用的组件 features；
`ui-document-runtime` 自身只依赖 `ui-document`，不会自动启用全部控件。解码时会检查
制品版本、长度、payload 指纹、文档 schema、应用绑定 schema 和当前编译 features，
再把文档编译为共享 `ViewNode<Msg>`。平台宿主继续按 Win32、AppKit 或 Linux experience
profile 完成布局和绘制。该路径不依赖 `ui-viewer`，因此不会携带轮询器、预览 PNG、
原生 smoke 驱动或强制额外进程。

完整组件覆盖和高级控件状态迁移仍是后续 v0.2 切片。AppKit 与 Linux Direct 已有固定
Runner Viewer 证据；Windows Viewer 当前具有本地真实宿主证据，仍需固定 Runner 基准。
浏览器投影不能替代原生运行证据。
