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
`radio_button`、`slider`、`number_box`、`combo_box`、`date_picker`、`time_picker`、`password_box`、`list`、`tabs`、`grid`、
`progress_bar`、`progress_ring` 和 `scroll`。
其他已存在的 ZSUI 组件会被识别为“尚未进入 UiDocument schema”，不会被误报为未知组件。

## 布局与文字完整性

`layout.padding_token` 和 `layout.gap_token` 使用平台体验层的间距，不把某个平台的
固定数值复制到另外两个平台。可用值为 `xs`、`sm`、`md`、`lg`、`xl`、
`content_gap`、`content_padding` 和 `page_padding`。同一节点的 `padding` 与
`padding_token`、`gap` 与 `gap_token` 互斥；数值布局仍可用于应用确实需要的专用几何。

`text` 支持 `text_role`、`wrap`、`ellipsis`、`weight`、`horizontal_align` 和
`vertical_align`。正文说明通常使用 `"wrap": "word"` 与 `"ellipsis": false`；
语义枚举会在静态文档和绑定值解析后分别校验。`flex` 只分配父 Stack 的主轴空间，
`0` 表示按内容尺寸布局；横向行内需要吸收剩余宽度的换行说明可使用 `flex: 1`，紧凑
操作行自身可使用 `flex: 0`。

Stack 与 Grid 会递归保留子树的固有尺寸，包括文字行框、控件最小尺寸、节点间距和
容器内边距。横向 Stack 先按真实分配宽度计算换行文字高度，再决定整行高度；空间不足
时保留这些硬约束并由上层滚动或裁剪，不压缩字形或把后续节点覆盖到当前容器中。

`number_box.value` 使用 `nullable_number`，可表示数字或空值；`change` 动作使用相同
payload 类型，因此清空并提交输入仍保持类型化。`minimum`、`maximum`、`step`、
`large_step`、`fraction_digits` 和 `wraps` 直接编译到共享 NumberBox。校验会拒绝倒置范围、
非正步长、超出 0–12 的非整数小数位以及超出静态范围的字面量值。

`combo_box.options` 使用 `string_array`，`selected_index` 使用 `nullable_integer`，从而拒绝
混合类型选项和小数索引。`select` 动作发送 `integer`，`expanded_change` 发送 boolean；将
`selected_index` 和 `expanded` 同时绑定到显式状态后，选择、弹层开关和 View 重建形成
受控闭环。静态索引超出选项范围会在进入原生宿主前被拒绝。

`date_picker` 使用独立的 `date` 绑定类型，序列化形式固定为 ISO `YYYY-MM-DD`，不会把
平台区域日期文本当作状态格式。`UiBindingManifest::register_date_property` 和
`register_date_action` 在 Rust 侧直接读写强类型 `ZsDate`。`value`、`visible_month` 和
`expanded` 可分别绑定受控状态；`change`、`month_change` 和 `expanded_change` 动作使日期
选择、月份导航及弹层开关在 Viewer 重建后继续保留。`minimum`、`maximum` 和可选的固定
`today` 也使用同一日期类型；校验器与发布运行时都会拒绝非规范日期、倒置范围、越界值，
以及未使用当月第一天的 `visible_month`。最终日历尺寸、间距和视觉仍由三平台体验参数决定。

`time_picker` 使用独立的 `time` 绑定类型，序列化形式固定为 24 小时制 `HH:MM`，显示时制
仍由目标平台或显式 `clock_format` 决定。`UiBindingManifest::register_time_property` 和
`register_time_action` 在 Rust 侧直接读写强类型 `ZsTime`。`value` 与 `expanded` 可分别绑定
受控状态，`change` 与 `expanded_change` 动作在 Viewer 重建后保留选择和弹层状态。
`minute_increment` 必须是小于 60 的非零约数，且分钟值必须与步长对齐；`clock_format` 只接受
`platform_default`、`twelve_hour` 或 `twenty_four_hour`。静态校验和发布运行时都会拒绝非规范
时间、无效步长、未对齐值及未知时制，不把平台本地化显示文本写入应用状态。

`password_box.value` 只允许绑定到安全状态，禁止写成文档字面量、本地化值或普通
`values.json`。`UiBindingManifest::register_secret_property` 和
`register_secret_action` 直接读写 `ZsPassword`；运行时使用
`ui_document_view_with_secrets` 的独立动作通道，不会把密码降低为 `serde_json::Value`。
`ZsPassword` 在释放时清零，Debug 输出固定脱敏，并且不实现 Serialize/Deserialize。
Viewer 热重建把密码保存在不可序列化的 `UiSecretValues` 中，普通动作历史只记录
`<redacted>` 元数据；证明报告、交接包和发布制品均不包含密码。交接清单通过
`sensitive_values` 和属性契约的 `sensitive` 标志提示编辑器不得生成明文值。
`reveal_mode` 可选 `platform_default`、`hidden`、`peek` 或 `visible`，最终控件尺寸、
显隐交互和绘制仍由各平台 PasswordBox profile 决定。

`list` 的每个直接子节点都是一个可选行，子节点的稳定 `UiNodeId` 同时作为公开选择值。
`selected` string 属性与 `select` string 动作组成受控选择闭环；调整子节点声明顺序后，
同一个 ID 仍指向同一个语义项目。静态或绑定选择值如果未命中直接子节点，会在进入宿主
前被拒绝；列表至少需要一个子节点。选择动作由 List 自身持有的类型化回调发送，不依赖
全局控件注册表或位置字符串。共享 List builder 会按当前平台的 selection 行高设置不可
压缩的最小行框，并使用平台 spacing token 留出水平内容边距；字体缩放继续抬高该行框，
不会把中英文基线挤进固定像素高度。

`progress_ring.value` 使用 `nullable_number`：数字表示确定进度，`null` 或未提供表示不确定
进度。`minimum`、`maximum`、`active` 和 `size` 分别控制范围、动画状态与平台原生的
`small`、`medium`、`large` 尺寸。静态校验和发布运行时都会拒绝倒置范围、越界值与未知
尺寸；最终直径、描边与动画节奏仍由 Win32、AppKit 和 Linux experience profile 决定。

`tabs` 的每个直接子节点都是一个内容槽位，子节点的稳定 `UiNodeId` 同时作为公开选择值
和内部强类型 `ZsTabId` 的确定性来源。`labels` 使用以子节点 ID 为键的 `string_map`，
`icons` 可使用相同键映射到 `ZsIcon` 语义枚举名；标签必须完整覆盖子节点，额外键、无效
图标和不存在的 `selected` 都会被拒绝。`selected` string 属性和 `select` string 动作绑定后，
切换页面会更新显式状态，热重建继续选中同一稳定槽位。Tabs 至少需要一个内容子节点。
框架会在标签条与当前内容页之间应用平台内容边距，并继续叠加节点自身的 `layout.padding`；
窄窗口不会把标签文字压缩到平台最小宽度以下，超出标签条的部分由标签条裁剪。

`grid.columns` 和 `grid.rows` 使用 `grid_track_array`，每条轨道明确声明为非负 DP
固定尺寸或正整数权重的 fraction；`placements` 使用 `grid_placement_map`，以每个直接
子节点的稳定 `UiNodeId` 为键，保存零起点行列和可选的正数跨度。校验要求放置表完整
覆盖且只引用直接子节点，并拒绝越过已声明轨道、空轨道、零权重、零跨度和负间距。
因此调整子节点声明顺序不会改变已有节点的单元格，Viewer 与发布嵌入也使用同一份
强类型放置契约。`column_gap` 和 `row_gap` 可分别覆盖通用 `layout.gap`。
固定轨道、文字原生行框和控件最小尺寸属于硬布局约束；可用空间不足时 Grid 保留这些
尺寸并让父视口负责裁剪或滚动，不通过缩小文字行框或控件来强行塞入窗口。

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

每个 `UiNodeId` 都确定性映射到文档专用的 `WidgetId` 命名空间。Viewer 的普通属性值和
动作历史位于 `UiViewerState` 中，PasswordBox 则使用跳过序列化的安全状态槽；View
重建不会替换这些状态。每次接受有效修改后，
`UiViewerSourceSnapshot::last_reload` 都会确定性列出保留、新增和必须重置的节点 ID。
节点 ID 和控件状态类别均兼容时，原生输入运行时保留焦点、文本选择以及文本编辑器的
纵向/横向视口；节点被删除、同一 ID 改为其他控件类型或文本框在单行/多行间切换时，
旧焦点、选择、拖动和 IME 临时态会显式清除，避免把旧控件状态错误路由到新控件。
`button.click`、`radio_button.choose`、`textbox.change`、`toggle_button.toggle`、
`checkbox.toggle`、`toggle.toggle`、`slider.slide`、DatePicker 三类状态动作、TimePicker 两类状态动作和 `scroll.scroll` 均走类型化 Viewer
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

受控日期示例同样由预编译 Viewer 加载：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/date-picker.json `
  --bindings examples/ui-documents/date-picker.bindings.json `
  --values examples/ui-documents/date-picker.values.json
```

受控时间示例使用同一份 Viewer，并保留平台默认显示时制：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/time-picker.json `
  --bindings examples/ui-documents/time-picker.bindings.json `
  --values examples/ui-documents/time-picker.values.json
```

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
真实路由该滚动输入并增加 `native_view_scroll_count` 时才通过。`--smoke-click x y` 可重复
提供固定点击序列；只有每次点击均进入目标宿主且至少产生一条类型化 Viewer 消息时才通过。

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
  --features ui-document,label,checkbox,textbox,slider,number-box,combo,list,tabs,grid,progress-ring `
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
