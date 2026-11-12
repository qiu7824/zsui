# ZSUI 框架评审：缺陷、缺失能力与改进方向

本文记录一次针对 ZSUI 源码的设计评审，回答四个问题：

1. 当前实现有哪些结构性缺陷。
2. 哪些设计更符合 Rust、更原生、内存更小。
3. 组件式架构为什么限制自由扩展，对比成熟框架还缺什么。
4. 字体显示可以怎样改进。

本文是评审意见，不是实现状态证据，也不改变任何完成度结论。实现状态仍以代码、
组件目录、目标产物和 Git 历史为准。

## 评审基线与方法

| 项目 | 值 |
| --- | --- |
| 提交 | `e3a5864`（与 `main` 相同） |
| 版本 | `0.3.0-alpha.2` |
| 测量环境 | Linux x86-64，rustc 1.97.0 |
| 方法 | 阅读源码、`rg` 统计，另编译一个独立探针打印 `size_of` |

统计口径：计数类数字排除了文件名含 `tests` 的文件，但包含源文件内联的
`#[cfg(test)]` 模块，因此是上界估计。尺寸数字来自探针实测。Windows 与 macOS
后端只做了源码阅读，没有在目标机器上运行。

## 一、总体评价

方向正确，工程纪律强。以下各项在 Rust 原生 UI 框架中并不常见，应当保留：

- 统一的 `State / Msg / view / update` 应用形态，强类型 `WidgetId` 与 `Dp/Px/Dpi`。
- Cargo feature 裁剪，默认只开 `window`、`button`、`label`。
- 三平台真机取证 CI、API 稳定性检查（`cargo-semver-checks`）、约 1000 个测试。
- 无障碍、IME、本地化、虚拟列表、原生菜单/托盘/对话框、可热重载的 Viewer。
- 文本引擎已有 hinting、亚像素、系统 ClearType 设置、彩色 emoji、可变字体。

主要问题不在功能数量，而在三处：核心数据结构偏胖、线程模型偏保守、单 crate
过大。扩展性问题则来自缺少公开的原语层。

## 二、实测到的结构性缺陷

### 2.1 节点过胖

| 类型 | `all-widgets` 等 | 仅 `button`+`label` |
| --- | --- | --- |
| `ViewNode<Msg>` | 744 字节 | 240 字节 |
| `ViewNodeKind<Msg>` | 264 字节 | 80 字节 |
| `ViewStyle` | 88 字节 | — |
| `NativeDrawCommand` | — | 72 字节 |
| `NativeDrawTextCommand` | — | 64 字节 |

左列的 feature 为 `all-widgets, shortcuts, accessibility, style`，不含窗口后端。

原因：`ViewNode`（`src/view/node.rs:1296`）把所有控件的可选字段都放在每个节点
上，包括 tooltip、表格列回调、面板缩放限制、滚动状态、快捷键列表等。一个纯
文本标签也要付出全部字段的空间，另加 `String` 的堆分配。布局结果（`bounds`、
`layout_dpi`、滚动内容高度）也存在节点里，使声明和计算结果耦合。

### 2.2 每条消息全量重建

`apply_messages` 之后固定执行 `rebuild_and_layout`（`src/view/focus.rs:594`）：
调用 `view_fn` 生成整棵新树，分配自动 ID，从旧树恢复导航和滚动状态，全树布局，
随后 `draw_plan()` 重新生成整份 `Vec<NativeDrawCommand>`，每条文本命令带一份
`String`。没有按子树跳过的机制。

### 2.3 损伤计算按位置比对

`native_draw_plan_damage`（`src/render_protocol.rs:1332`）用 `zip` 逐条比较新旧
命令。命令条数一变即整窗重绘；多处变化被合并为一个包围矩形。插入或删除一个
节点会导致后续所有命令错位。

### 2.4 UI 状态被迫 `Send` 并加锁

应用状态位于 `Arc<Mutex<Box<dyn LiveViewDriver>>>`（`src/view/focus.rs:163`），
要求 `State: Send + 'static`、`Msg: Clone + Send`。应用状态因此不能持有 `Rc`
等非 `Send` 值，每次访问都要加锁并处理 poison。

注意：`docs/memory-ownership.md` 已指出这把锁不能直接删除。Windows 的 UIA
提供器使用 `ProviderOptions_ServerSideProvider` 且未启用 COM 线程模型，回调会
从其他线程进入。去锁之前必须先提供不可变快照或明确的 UI 线程调度协议。

### 2.5 Win32 用全局表保存窗口状态

`src/platform/windows/mod.rs:130` 起有 5 张
`static OnceLock<Mutex<Vec<…Record>>>`，分别保存绘制计划、输入路由、完成报告、
shell 输入路由和菜单命令表。`window_paint_state`
（`src/platform/windows/window.rs`）在绘制时按 `hwnd` 线性查找并克隆 `Arc`。
这些虽不是控件注册表，但与"不引入全局可变注册表"的规则精神不一致。

### 2.6 文本编辑的内存放大

- 撤销历史最多保存 100 份完整文本快照（`src/native_text_edit.rs:13`）。
- 每次编辑通过 `ViewMessageMapper<String, Msg>` 把整份文本作为消息传回。
- 1 MB 文档在最坏情况下撤销历史约 100 MB，每次按键复制整份文本。

### 2.7 `unsafe` 缺少说明与工具约束

- 约 662 个 `unsafe {` 块，只有 10 处 `SAFETY` 注释。
- CI 工作流和脚本中没有 clippy 或 miri。
- `src/lib.rs` 没有 crate 级 lint 属性，`Cargo.toml` 没有 `[lints]` 和
  `rust-version`。

### 2.8 默认构建不够小

- `serde` 与 `serde_json` 是必选依赖。
- 以下模块无条件编入库：`agent_context`、`feature_manifest`、
  `component_catalog`、`mobile_host`、`android_activity_host`、`native_smoke`、
  `native_proof`、`product_adapter`，合计约 9000 行。它们多为清单、取证和契约
  数据，不是应用运行时必需。

### 2.9 单 crate 过大

- `src` 约 18.8 万行、200 个文件。
- `src/native.rs` 17071 行，`src/ui_document.rs` 12924 行，
  `src/widget_render.rs` 9427 行，`src/ui_document_runtime.rs` 9412 行。
- 约 4300 处 `#[cfg(`。部分 feature 组合下会出现 dead-code 警告。

## 三、更符合 Rust、更原生、更省内存的设计

`docs/memory-ownership.md` 的"Retained architecture contract"已定义了
声明、身份、布局、呈现四层分离的迁移目标。以下建议与该目标一致，可视为其
具体落地步骤。

| 序号 | 建议 | 对应缺陷 | 说明 |
| --- | --- | --- | --- |
| 1 | 瘦节点 | 2.1 | 节点只留 `id / kind / style / children`；少用修饰项放入 `Option<Box<Extras>>`；大变体装箱；布局结果移到按先序索引的旁路数组；静态文本用 `Cow<'static, str>` 或 `Arc<str>`。目标约 100 字节 |
| 2 | 带 key 的子树记忆化 | 2.2 | 提供 `memo(key, \|\| view)`，key 不变则复用上次子树、布局和绘制段。这是缓存，不是响应式运行时 |
| 3 | 按节点分段的绘制计划 | 2.3 | 每个 `WidgetId` 一段命令，损伤按节点计算；文本与节点共享 `Arc<str>`；`Vec` 跨帧复用 |
| 4 | 窗口独占状态 | 2.4、2.5 | Win32 把 `Box<WindowState>` 挂到 `GWLP_USERDATA`，AppKit 放实例变量，删除全局表。前置条件是先完成无障碍的快照或调度协议 |
| 5 | 拆分 workspace | 2.8、2.9 | `zsui-core`（几何、View、布局、绘制协议，`#![forbid(unsafe_code)]`，不依赖 serde）、各平台后端、文本、文档、取证工具各自成 crate，`zsui` 只做门面 |
| 6 | 增量文本 | 2.6 | 撤销保存编辑操作（范围、删除的文本、插入的文本）；`on_change` 可选传增量；大文档再评估 piece table |
| 7 | `unsafe` 治理 | 2.7 | 启用 `unsafe_op_in_unsafe_fn` 与 `clippy::undocumented_unsafe_blocks`；句柄统一 RAII；核心 crate 跑 miri |
| 8 | `serde` 改为可选 | 2.8 | 核心类型的派生放到 `serde` feature 下，`serde_json` 只留在文档与取证路径 |

## 四、组件式架构对自由扩展的限制

### 4.1 根本原因

框架只有组件层，没有公开的原语层。内置组件通过内部权限直接绘制
（`src/widget_render.rs`、`src/view/paint.rs`），不是用公开 API 组合出来的。
组件目录之外的需求没有可走的路径。

### 4.2 具体表现

| 限制 | 现状 |
| --- | --- |
| 控件集合封闭 | `ViewNodeKind` 是封闭枚举，约 40 个变体。`View` trait 公开，但子节点类型固定为 `Vec<ViewNode<Msg>>`，用户实现无法插入树中 |
| Canvas 词汇少 | `ZsCanvasPrimitive` 共 10 种：填充/描边矩形、圆弧、三角形、两种圆角矩形、文本、图标及其带色版本。没有路径、直线、渐变、阴影、透明度、变换、图片。`component_catalog.rs` 也将路径构造列为 Canvas 缺口 |
| 样式是白名单 | `ViewStyle` 有 14 个字段，背景和边框只能取 11 个 `ThemeColorToken`。没有 margin、最大宽高、绝对定位、层叠、流式换行、按边圆角、边框粗细 |
| 无自定义布局 | 只有 Stack、Grid、Scroll 等内置容器，没有叠放容器，也没有布局 trait |
| 无动画 API | 只有内部的进度环时钟和导航悬停过渡，应用层不能声明过渡或缓动 |

### 4.3 建议的分层

上层必须只使用下层的公开 API 实现：

1. **绘制原语**：路径、渐变、图层（透明度、裁剪、变换）、阴影。
2. **交互与布局原语**：`container`、`stack`、`zstack`、`pressable`、
   `hoverable`、`focusable`、`text`，配合统一的修饰器链。
3. **内置组件**：用第 2 层组合而成，框架自身成为原语层的第一个用户。
4. **复合外壳**：导航、文档等。

另外保留一个 `Custom(Box<dyn Widget<Msg>>)` 变体作为扩展出口。内置控件继续
用枚举分发，保持小而快。

颜色方面，允许"自定义色加高对比度回退角色"。Canvas 的 `ColoredIcon`
已经采用这一模式，可推广到普通节点，不破坏主题和无障碍约束。

这一分层对 AI 编写界面尤其重要：组合少量正交原语的出错率远低于记忆几十套
各不相同的 Spec 结构。

## 五、对比成熟框架的能力缺口

| 缺失能力 | 核实情况 |
| --- | --- |
| 动画与过渡系统 | 无公开 API |
| 富文本（行内样式、链接、选择） | 一个文本节点只有一种样式；`desktop-hardening-plan.md` 的 D12 已列出 |
| 矢量图形与视觉效果 | 绘制协议中没有路径、渐变、阴影、透明图层 |
| 定时器、订阅、通用异步任务 | 未找到公开的定时器或订阅 API；D05 已列出后台任务取消与结果契约缺口 |
| 图片格式 | 只有 PNG；未见 JPEG、WebP、GIF、SVG 解码 |
| 触摸与多指手势 | 组件目录自身列为缺口 |
| 打印 | 无 |
| 面向应用的 UI 测试接口 | 只有内部 smoke 与 proof，没有供应用编写无窗口 UI 测试的稳定 API |
| GPU 渲染路径 | 仅 Windows 可选 Direct2D，其余为 CPU 软件绘制。有利于内存，不利于大面积动画和滚动 |
| 自定义字体注册 | 未找到 `load_font` 或 `register_font` 一类入口 |

以下三项本次没有逐一确认，需要另行核查：多窗口、应用内拖拽排序、RTL 整体
镜像布局。

## 六、字体显示的改进

### 6.1 现状

`src/rust_text_renderer.rs` 的 `ZsRustTextEngine` 基于 cosmic-text 0.19
（内部用 HarfRust 整形）、swash 和 skrifa，已实现：

- skrifa hinting 及实例缓存，按 `gasp` 表决定对称平滑。
- 灰度、RGB、BGR 三种光栅模式，带 gamma 和对比度配置。
- Windows 上读取系统字体平滑设置（`SPI_GETFONTSMOOTHING*`）。
- 彩色 emoji、可变字体轴、合成粗体、GPOS 字距修正。
- 有界缓存：字形 2048 项 / 4 MiB，布局 256 项 / 2 MiB，并定期修剪整形缓存。
- Windows 上测量与绘制共用同一引擎。

这一基础是合格的，下列改进集中在内存和结构。

### 6.2 改进项

| 序号 | 改进 | 理由 |
| --- | --- | --- |
| 1 | 字体发现与回退交给系统，按需映射字体文件 | `ZsRustTextEngine::new()` 调用 `FontSystem::new()`，把全部已安装字体登记进 fontdb，启动耗时和内存随字体数量增长。改为由 DirectWrite、Core Text、fontconfig 回答"哪个字体覆盖这个字符"，只映射实际用到的文件，整形和光栅仍用 Rust |
| 2 | 合并重复的字体解析库 | `Cargo.lock` 同时包含 skrifa 0.40 与 0.44、read-fonts 0.37 与 0.41，另有 ttf-parser 和 swash，相当于四套解析代码进入二进制 |
| 3 | 字形缓存改为图集 | 当前是 `HashMap` 中每个字形一个 `Arc<Image>`，每项都有堆分配和哈希开销。图集分页加货架式装箱可降低分配次数与碎片，对 CJK 界面收益明显。设计约束见 `docs/memory-ownership.md` 的 Glyph atlas 一节 |
| 4 | 支持富文本 | 布局结果按"行 → 样式段 → 字形"组织，一个文本块可带多段样式和链接命中区 |
| 5 | 公开应用字体注册 API | 品牌字体和图标字体目前只能依赖系统已安装字体 |
| 6 | 静态标签走轻量路径 | cosmic-text 的 `Buffer` 面向编辑器；单行标签可整形一次后只保存字形序列 |

macOS 使用 Core Text、Linux 使用 Pango 的决定应当保留。它换来各平台的原生
观感，代价是维护多套后端，这一取舍在项目决策记录中已有说明。

## 七、建议的实施顺序

| 阶段 | 内容 | 理由 |
| --- | --- | --- |
| 1 | 瘦节点（三.1） | 改动集中在 `src/view/node.rs`，公共 API 基本不变，内存直接下降 |
| 2 | 原语层（四.3） | 同时为扩展性、动画、富文本提供落脚点 |
| 3 | 字体按需加载与依赖合并（六.2 的 1、2） | 启动时间和二进制体积收益最直接 |
| 4 | 拆分 workspace 与 `serde` 可选（三.5、三.8） | 降低编译时间，用 crate 边界替代大量 `cfg` |
| 5 | 子树记忆化与分段绘制计划（三.2、三.3） | 依赖阶段 1 的不可变节点 |
| 6 | 窗口独占状态（三.4） | 需先完成无障碍快照或调度协议 |
| 持续 | `unsafe` 治理、增量文本（三.6、三.7） | 可与其他阶段并行 |

每个阶段都应附带前后对比：节点尺寸、驻留内存、二进制体积和既有取证场景的
像素一致性。

## 八、与现有文档的关系

- `docs/memory-ownership.md`：已定义保留式架构目标和紧凑存储的约束。本文
  第三节是其落地步骤，第 2.4 节的去锁建议受其线程与无障碍约束限制。
- `docs/desktop-hardening-plan.md`：D05、D06、D12 与本文第五节部分重合。
  其中 D06 称 padding 仍为统一值，但 `ViewStyle` 已有 `padding_sides` 字段，
  该行描述可能需要更新。
- `docs/text-rendering.md`：描述当前文本机制与取证矩阵。本文第六节只补充
  内存与结构方面的改进方向。
- `docs/ai/project-memory.md`：约 1770 行，包含大量运行编号和测量值，实际
  承担了进度日志的作用。建议把运行证据移到单独文件，使其回到"持久决策"的
  定位。
