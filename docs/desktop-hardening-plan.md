# ZSUI 桌面框架完善计划

## 目标与边界

以普通桌面应用的可靠交互、自然布局和可控资源消耗为主要验收标准。保持一份 Rust
`State / Msg / view / update` 应用代码、平台内部适配、默认最小 features 和 Windows
缓冲无闪烁绘制。Workbench 是可选组合能力；小工具、表单和文件浏览界面均可独立完成。

0.3 的核心里程碑仍是 [Native Proof CI](v0.3-native-proof-ci.md)。本计划细化该里程碑的缺陷与应用可用性工作，
不以组件数量或新的总体完成百分比替代交互验收。产品存储、同步、模型协议、业务校验、
编解码与网络策略由应用或可选适配器持有。

## 基线与状态定义

评估日期：2026-10-04。已提交基线为 `b821cfd8e88bb6ad8db0e82bc8e5a91842ad2eb5`；
代码状态同时覆盖现有开发改动。实现、构建验证、目标交互与发布状态分别记录。

- **实现缺口**：代码路径可以确认缺失或行为限制；未自动等同于已完成目标机复现。
- **接口风险**：已有可用路径，但默认用法、错误反馈或资源策略容易导致应用错误。
- **证据缺口**：已有实现，需要指定平台、输入方式或规模的运行证据。
- **已实现、保留回归**：已有对应实现，不作为待开发功能重复安排。

## 优先问题清单

| 编号 | 优先级与状态 | 当前问题及代码定位 | 完成条件 |
| --- | --- | --- | --- |
| D01 | P0，代码已修、目标验收待完成 | `native_input_visuals.rs` 已统一可编辑 NoWrap 输入的横向几何、命中与绘制；`native.rs::ensure_text_edit_for_target` 已保持偏移并重新揭示光标，已有聚焦回归测试 | 长路径输入、Home/End、选择、IME、重建和 DPI 变化通过真实窗口验收，值不被裁剪或改写 |
| D02 | P0，部分实现 | `app_effect.rs` 已有类型化请求/结果，四宿主已接锁外执行，stable 已有对话框入口；目录选择目前仅 Windows 实现，其他后端默认 Unsupported | 打开/保存/目录选择/确认在目标平台完成；有效窗口的已执行请求结果恰好交付一次，取消和销毁后按终结策略处理；同时关闭 D15 |
| D03 | P1，实现缺口 | `render_protocol.rs::NativeDrawIconCommand` 只有 bounds，没有独立字形尺寸；Workbench 仍混用通用 standard_icon 与局部常量 | 分离槽位、字形尺寸和光学变体；导航、工具栏、状态、发送等角色各有平台指标；布局与命中不随光学补偿改变 |
| D04 | P1，实现缺口 | `workbench.rs` Inspector tab 绘制读取 `tab.label`，没有消费已声明的 `tab.icon` | 图标参与测量、绘制和文字间距；无图标时恢复文本布局；窄宽、长标题和缩放不重叠 |
| D05 | P1，部分实现 | `stable.rs::UpdateContext` 已有打开、保存、目录和确认回调；仍缺 image 构造器、稳定图像值以及通用后台任务取消/结果契约 | 最小图片工具及含后台结果的表单可只用 stable/prelude；新增入口遵守兼容策略与按需 feature 边界 |
| D06 | P1，布局能力缺口 | `ViewStyle` 已有 flex=0、justify/align，但 padding 仍是统一值，缺少通用四边间距、最大尺寸及基线等约束 | 先提供四边 padding、max size 和明确溢出规则，再按实际用例补 baseline、wrap、内容轨道；不拉伸文字、不覆盖邻居 |
| D07 | P1，资源策略缺口 | `command_protocol.rs::CommandQueue` 使用无容量限制的 VecDeque；native pending app/UI commands/effects 使用 Vec | 明确容量、背压、每次派发预算与关闭策略；只合并可替代的刷新命令，不静默丢弃用户动作；兼容旧 API |
| D08 | P1，诊断缺口 | Win32 tray tooltip 更新与显式删除忽略返回错误；菜单等资源的析构清理也存在 best-effort 路径 | 区分显式操作失败与析构清理失败；前者可向调用者或结构报告反馈，后者可诊断且不 panic；清理不重复执行 |
| D09 | P1，性能缺口 | Workbench 已限制可视消息的布局对象，但 `zs_workbench_layout_internal` 每次重新布局仍收集全部消息高度并遍历全部历史 | 高度按稳定 ID、内容修订、宽度和排版上下文缓存；滚动定位不逐条扫描全部历史；流式变化只失效必要数据 |
| D10 | P1，证据缺口 | 导航副文本、会话图标/副文本已实现；三平台证据需逐项核验，AppKit/Linux 重点补齐对应交互与窄窗证据 | 同一语义场景在三平台产生最终截图、稳定点击 ID、裁剪和辅助功能证据 |
| D11 | P1，状态报告缺口 | reference、component catalog 与 machine-readable completion 中仍有粗粒度历史缺口；运行时覆盖率不能表示全部体验完成 | 将实现、目标交互、人工体验分开记录；逐项关联实际检查，清理已完成事项的过时缺口，保留真实未完成边界 |
| D12 | P2，可选能力缺口 | Workbench 消息块仍以 Paragraph/Code/Tool/Notice 为主，缺少通用富文本选择、链接和结构化文档视口 | 富文本、可变高列表、Composer 和自适应面板可独立组合；协议解析留在应用；既有 Workbench 保持兼容 |
| D13 | P2，宿主扩展缺口 | 当前公开主线未提供 native_child_host/app_poll 对应入口；原生子视图接入缺少正式生命周期契约 | 先以独立 feature 定义安全生命周期、布局、焦点、DPI 和释放边界；目标专有对象留在后端；后台刷新继续优先使用 InvalidationHandle |
| D14 | P0，代码已修、目标验收待完成 | `view/layout.rs` 已区分首选/最小宽度，并对 ellipsis 软约束收缩；已覆盖长标签、嵌套布局、显式宽度与 CJK/emoji 测试 | 窄窗中可省略文本收缩，显式宽度和原生控件最小尺寸保持硬约束，旁侧动作真实可见且可点击 |
| D15 | P0，实现缺陷 | 四宿主 AppEffect drain 在 32 轮后只记录事件，最后从 runtime 取出的 pending effects 未归还或安排后续派发；每轮请求数也未受预算约束 | 超预算请求保留并在下一轮唤醒执行；每批派发有界并让出事件循环，原生模态服务不在持锁状态执行；窗口销毁的取消语义明确 |

P0 表示优先处理实际交互阻断，不表示所有应用必然触发。D02 已存在锁外执行基础，
应完善应用契约并验证重入场景，避免另造一条并行命令系统。

非交互节点的辅助功能 ID 需要单独核验：当前显式设置 accessibility 的节点已经参与
自动 ID 分配，不能将“所有 Label/Image 都必须手写 ID”作为现状结论。检查重点是普通
语义节点的默认暴露、重排身份以及 stable 图像入口的实际使用成本。

## 已实现能力的回归集合

- 默认内容尺寸 `flex=0`、主轴/交叉轴对齐；不再安排一次重复的默认 flex 修复。
- 鼠标逻辑焦点与键盘可见焦点分离；选中态与焦点框分别验证。
- 可编辑能力驱动的全选、复制、粘贴、剪切、撤销；PasswordBox 保持受保护策略。
- 文本组件统一 placeholder；提示不得进入值、剪贴板、撤销或安全输入数据。
- Toast 超时后的关闭状态跨重建保留；ContextMenu 与 Accordion 已有实现。
- Win32 托盘右键路由、命令派发、Explorer 重启恢复；人工托盘选择证据仍独立验收。
- Workbench 使用真实文字测量、保留布局、滚动裁剪及可视区域物化；这不等于完整的增量渲染架构。
- 非视觉 InvalidationHandle 合并后台唤醒；保持空闲无轮询，不使用隐藏 Video 驱动业务刷新。
- Shell 导航副文本、Workbench 会话图标/副文本；与 D04 的 Inspector tab 图标是不同路径。

## GPUI Kit 差异评估

对照对象为 Longbridge GPUI Kit v0.7.0（2026-09-28），固定源码提交
`0c830f4d257e69fdd17200650533ab4ca9a40cc0`。GPUI 是底层窗口、实体、布局和 GPU 运行时；
Kit 整合 `gpui-base` 行为基础层与 `gpui-component` 外观组件层，JavaScript 扩展另外启用。
参见[官方架构](https://gpui-kit.com/docs/)与[固定发行版](https://github.com/longbridge/gpui-kit/releases/tag/v0.7.0)。

| 维度 | GPUI Kit 已提供的能力 | ZSUI 当前能力与下一步 |
| --- | --- | --- |
| 行为与外观 | Base 提供受控状态、焦点、浮层、选择和虚拟化，Component 提供完整视觉系统；见 [Base](https://gpui-kit.com/base/) | 已有 typed View、editable_text、平台 profile；进一步统一可组合行为与状态样式，保持显式 State/Msg |
| 视觉体系 | 多尺寸、语义主题、状态 token、主题资源；见 [Theme](https://gpui-kit.com/component/theme/) | 已有系统与公开主题模式切换、明暗/高对比和语义字体；缺应用自定义 token 在普通 View/组合控件间的一致传播契约、统一状态样式及图标光学规格 |
| 布局与工作区 | 可分割 Dock、拖动标签组、布局序列化；见 [Dock](https://gpui-kit.com/component/dock/) | Stack/Grid/SplitView 和静态 Tabs 可用；先补约束布局及文档标签，再另设完整 Dock 里程碑 |
| 数据表格 | DataTable 有虚拟滚动、列操作、单元格选择、键盘与排序委托；普通 Table 是静态表；见 [DataTable](https://gpui-kit.com/component/data-table/) | 当前 DataGrid 仍对全部行列生成计划，主要支持只读字符串和单行选择；应先复用统一虚拟化、选择和列几何 |
| 列表与树 | Variable-size VirtualList 和虚拟树；尺寸表更新仍可能重建全量索引，见[固定列表源码](https://github.com/longbridge/gpui-kit/blob/0c830f4d257e69fdd17200650533ab4ca9a40cc0/crates/base/src/virtual_list.rs) | ItemsRepeater 已有稀疏可变高 metrics；缺持久索引和自动测量反馈，PagedList 锚点仍依赖固定行高，Tree 仍物化全部展开行 |
| 富文本与编辑 | TextView 支持 Markdown、选择复制和流式内容；Editor 是另一个具备高亮、搜索等能力的层；见 [TextView](https://gpui-kit.com/component/text-view/) 与 [Editor](https://gpui-kit.com/base/primitives/editor/) | 已有可靠文字整形和输入底座；单样式 Text 与四种 Workbench 块尚不能替代通用富文本，应先完成只读富文本再扩展编辑器 |
| 开发工具 | 组件展示、API 文档、headless UI 输入/焦点/几何/语义测试；像素 offscreen 目前有平台限制；见 [Testing](https://gpui-kit.com/docs/test/) | 已有 Viewer 热重载和真实平台 Native Proof；应补可复用测试驱动与开发者 Inspector，Workbench InspectorPanel 不承担此职责 |
| 可访问性与平台 | AccessKit 语义、键盘和动作集成；headless 测试不能替代真实屏幕阅读器；见 [Accessibility](https://gpui-kit.com/docs/accessibility/) | 已有 UIA、AppKit 与 Linux 桥接及部分真实目标证明；重点补各组件关系、动作、焦点和人工体验矩阵 |

ZSUI 的 feature 裁剪、平台各自的原生风格、显式消息模型、UI 文档/Viewer 分离继续保留。
当前没有同机同场景的双方性能测量；GPU、120 FPS 或大文档宣传不能转化为 ZSUI 慢多少、
内存多多少的结论。51 个 runtime families 与 75+ components/primitives 的统计口径不同。

GPUI Kit 的 WebAssembly 标为 showcase，移动端仍是实验性/依平台而定；本阶段不追齐这些
扩展范围。参见 [WebAssembly](https://gpui-kit.com/docs/webassembly/) 与
[Mobile](https://gpui-kit.com/docs/mobile/)。WebView、JS 运行时及浏览器投影不进入 ZSUI 原生核心。

## 完整阶段：桌面应用基础完善

阶段目标：普通应用通过稳定入口完成表单、后台效果、图片、长列表/表格、富文本阅读和
文档标签，具备统一视觉/交互规则、明确资源预算和三平台回归证据。A—G 是同一完整阶段的
交付切片，全部满足出口才关闭阶段；不按组件声明数量结项。

| 切片 | 主要成果 | 依赖 | 主要归属 |
| --- | --- | --- | --- |
| A | 输入修复收口、无丢失效果调度、错误与队列预算 | 无 | native input、app_effect、desktop services |
| B | Stable 入口、布局约束、图标与状态样式 | A 的公共契约 | stable、ViewStyle、component profile |
| C | 表单/焦点/浮层/选择的共享行为 | A、B | view event/focus/overlay、accessibility |
| D | 有索引的虚拟集合、实用 DataGrid/Tree | A 的预算、B/C 的几何与行为 | list、paged-list、table、tree |
| E | 只读富文本和文档标签 | B、C、D | 可选 rich content、tabs |
| F | Viewer Inspector、测试驱动、性能诊断 | 随 A—E 同步建设 | ui-viewer、test/proof features |
| G | 三平台验收、裁剪与兼容性收口 | A—F | CI、native proof、catalog/reference |

### A：可靠交互与效果调度

- 保留 D01/D14 已有代码与回归，补真实长路径输入、中文/emoji、Home/End、选择、重建、
  DPI 和标签旁动作命中；同一个几何结果服务绘制、命中、光标和辅助功能。
- 关闭 D15：效果预算耗尽后保留待处理队列，通过下一次事件唤醒继续执行；预算同时覆盖
  请求数和派发工作量。不得在 UI 线程等待自身消费队列，也不得静默丢弃不可替代动作。
- 用受控测试效果执行器覆盖至少 100 轮链式响应、1,000 个排队请求、重复响应、取消、
  执行前/执行中/结果入队后销毁及中途失败；已接收且未取消的请求恰好执行一次，有效窗口
  的已执行请求结果恰好交付一次；取消或销毁后按终结策略停止调用 update，剩余队列可排空。
- 完成 AppKit、Linux Direct 和 GTK 的安全目录选择；声明支持的正常环境必须成功选择和
  取消。Unsupported 只用于服务确实不可用的配置或环境，不能作为后端实现验收通过。
  打开/保存/目录/确认都使用既有 AppEffect 和 Msg 路径，保持 UI 线程亲和性与锁外执行。
- 给 CommandQueue、pending command/effect 增加明确容量和拒绝/合并策略，公共 API 兼容；
  清理与 tooltip 等显式失败可诊断，析构幂等且不 panic。

出口：D01/D02/D07/D08/D14/D15 有行为测试与目标证据；模态服务往返不发生持锁重入，
有效结果不丢失，取消/销毁严格遵循终结策略；超预算后的按需唤醒不会变成空闲轮询。

### B：稳定入口、布局与视觉规范

- Stable/prelude 增加图像值、image、icon、必要布局与输入配置；后台工作使用可取消的
  类型化结果交付和既有 InvalidationHandle，应用仍拥有任务数据与业务生命周期。
- 增加四边 padding、max width/height、明确 Auto/Fill 约束与 baseline 对齐；保留默认
  flex=0、原生硬最小尺寸与 ellipsis 软约束。旧 `.padding(Dp)` 仍表示四边一致。
- 图标槽位与字形尺寸独立，按导航、工具栏、状态、发送、展开箭头定义平台角色；Inspector
  tab 的 icon 参与测量与绘制。协议变化提供兼容构造或版本迁移，不直接破坏既有结构体调用。
- 统一 normal/hover/pressed/selected/focus-visible/disabled/read-only/invalid 状态与
  compact/standard/comfortable 密度；公共主题配置传播到普通 View 和组合控件。
- 先实现通用 transition、按需时钟和系统 reduced-motion；取消、隐藏暂停和静止零动画
  唤醒进入契约，spring 与复杂 GPU 特效留给后续扩展。

出口：表单与图片工具的应用源码不引用平台类型，不依赖 Workbench；640/900/1280/1600 DP
宽度、100%/150%/200% 缩放及明暗/高对比均无交叠、错误焦点框或缺失动作。

### C：共享行为和表单基础

- 建立可组合 Field：标签、必填、帮助、错误、校验状态与控件关系；应用提供校验规则，
  框架负责布局、主题、焦点和辅助功能关联；Rust 与 UiDocument 使用同一语义契约。
- 统一 input 的 disabled/read-only、选择、剪贴板、撤销和 IME 行为，保留 PasswordBox
  保护策略；滚动和复制不得泄漏安全值或把 placeholder 当输入值。
- 收口浮层层级、focus scope、Escape、Tab、点击外部、失焦、resize、焦点恢复和边缘定位。
  窗口局部状态按强 ID 持有，不建立全局可变控件注册表。
- 以 trait、可组合能力和显式消息复用控制逻辑，平台差异通过内部 profile 表达，
  不要求应用匹配操作系统，不引入新的响应式运行时。

出口：相同语义行为同时通过纯状态测试、View 输入测试和目标宿主输入；嵌套两层浮层不会
把输入送到底层页面，关闭后恢复有效焦点；UIA/AppKit/AT-SPI 能读取字段关系并调用动作。

### D：虚拟化与数据控件

- 扩展已存在的 ItemsRepeater 可变高 metrics，建立持久前缀高度索引和可见区定位，避免
  每次 viewport 都复制、排序并扫描全部 metrics。以稳定 key、内容修订、宽度、DPI 和
  排版上下文作为测量失效依据。
- 接通测量反馈、估计高度、插入/删除/重排、向前加载和行内像素锚点；PagedList 使用同一
  变高协议，旧固定行高路径继续有效。加载、排序、过滤和数据存储由应用控制。
- DataGrid 在同一虚拟化底座上增加可视行/列物化、列宽调整、固定列、列重排、行范围/多选、
  单元格键盘焦点及复制；排序只发类型化请求。可编辑单元格和业务校验不属于这个切片。
- Tree 增加可视行物化、增量展开索引与完整键盘导航/层级语义；树数据仍由应用拥有。
  拖放重排和多选树另设后续切片。
- Workbench 消费同一高度索引与缓存，追加一个块不重新测量全部历史；保留现有兼容 builder。

出口：100,000 行表格/树及 10,000 条混合高度内容只物化视口与有界 overscan；稳定索引后的
滚动查找目标为 O(log N + 可见项)，单行更新不重新测量无关历史；全宽/DPI 变更可重建索引。
排序、加载和图片高度更新后，原可见 key 与行内偏移保持正确。

### E：富文本阅读与文档标签

- 提供可选 RichText span/paragraph/link/code-block 模型、跨块选择、复制和键盘链接激活。
  Markdown 解析是独立可选适配器；链接打开和代码动作返回消息，由应用决定执行。
- 追加内容按块 ID/修订增量处理，解析、排版和图片缓存有字节上限；选择与滚动锚点在
  不相关块更新后保留；代码块先提供正确字体、换行/横滚和复制，不包含 LSP 编辑器。
- 文档标签增加 close request、dirty/pinned 状态、重排、溢出菜单与键盘关闭/切换；
  应用决定是否真正关闭或保存，普通静态 Tabs 保持兼容。

出口：长 Markdown 中链接、重复文本、代码、CJK/RTL/emoji 的选择复制正确；流式追加
不重置用户阅读位置；关闭脏标签可取消且不会误删应用内容，重排保留稳定身份。

### F：开发者工具与持续验证

- 在 Viewer 中增加节点拾取、布局/裁剪/命中/focus 路径、token 来源、事件顺序与
  layout/paint/cache 计数；Inspector 是开发工具，不进入普通应用的默认依赖。
- 提供可复用的 test-support：按稳定 ID 发送指针、键盘、IME、resize、theme 与异步结果，
  检查值、焦点、几何、命令次数与辅助语义。不得让测试直接篡改内部状态来冒充宿主输入。
- 每个核心能力在 Gallery 提供 normal、disabled、keyboard、overflow、error 状态场景，
  自动截取目标表面并与受审查基准比较；UI 文档与 Rust builder 共用行为场景。
- 根据观测的局部变化成本推进 type+key reconciliation、dirty subtree 和重绘边界。
  保持既有软件路径；GPU 后端不是阶段前置条件，完整 LayerTree 重构另行决策。

出口：一个控件缺图标、命中偏移、焦点异常或全量重测可在 Inspector 与回归报告中定位；
普通 release 依赖图不包含 Viewer、测试驱动、截图、诊断序列化或预览常驻进程。

### G：目标平台、性能与发布收口

所有切片同步建立平台证据，最后统一核验。Win32、AppKit、Linux Direct 使用同一应用源码；
GTK 和 Lite 属于独立配置，不能由其他后端的证据代替。云端 Linux 可完成共享逻辑和 Linux
目标检查，Windows/macOS 运行证据由真实对应 runner 提供；交叉编译不替代目标运行。

| 验收应用 | 固定场景 | 必需证据 |
| --- | --- | --- |
| 表单与图片工具 | 长路径、字段错误、图片、文件/目录选择、取消、后台进度、窗口先关闭 | 类型化结果次数、焦点/IME 几何、系统服务结果、最终截图 |
| 数据浏览器 | 100,000 行表格和树、列操作、过滤/排序请求、键盘选择、分页与变高行 | 物化量、测量量、锚点、延迟、RSS/private/PSS、系统辅助动作 |
| 文档阅读器 | 10,000 个混合块、Markdown 链接/代码、流式追加、文档标签 | 复制结果、选择身份、阅读位置、增量工作量、缓存预算、最终截图 |

尺寸矩阵采用 640/900/1280/1600 DP 和 100%/150%/200% 缩放；字体包含 CJK、RTL 与组合
emoji，主题包含浅色、深色和高对比。对话框、菜单与目录选择必须记录创建、交互、取消、
销毁，不将服务声明计作完成。

性能测量固定硬件、系统、字体、release 配置、物理尺寸和内容，至少五轮；分别报告冷/暖
首帧、布局耗时、p95/p99 帧时、空闲唤醒、UI 主进程与子进程内存。60 Hz 场景以 p95
帧时不超过 16.7 ms 为目标，最终阈值在基线硬件上固定；超过阈值必须定位并明确修复或
调整适用范围，不能选择更轻内容替代原场景。

缓存按字节、队列按请求数设硬上限；应用持有的历史数据及索引单列。50 次窗口开关及
100 次页面往返后框架资源应进入稳定平台期。空闲 60 秒内无业务轮询造成的周期重建。
不把工作集强制回收、删除 CJK 字体或把子进程移出统计当作性能改善。

每项状态分别为“实现、自动逻辑验证、目标平台交互、人工体验、发布”；修正 catalog 和
reference 中过时条目，特别是已存在的可变高 metrics。真实 CJK 候选窗以及 Narrator/NVDA、
VoiceOver、Orca 人工体验是发布门槛；自动化不能宣称观察到未经实测的朗读行为。

## 依赖、兼容与阶段外范围

- 默认仍为 window/button/label。图像、富文本、Markdown、动画、数据高级能力、DevTools
  各自按 feature 裁剪；名称与边界在实现时登记到 feature manifest 与锁定矩阵。
- 稳定 API 保持兼容；新增布局/图标/文档字段明确默认值与 schema 演进。既有应用通过
  兼容层迁移，不进行一次性全仓重写，不为拆 crate 而提前拆 crate。
- 全量 Dock/浮动窗口、LSP/多光标编辑器、数据网格编辑、完整媒体播放、移动宿主、Web/JS
  扩展、GPU 合成以及 D13 原生子视图属于后续里程碑。当前阶段先建立它们可复用的行为基础。
- 完整阶段的结束条件是 A—G 的验收应用与门槛全部满足；代码已写、单次截图或库测试全绿
  均不能单独关闭目标交互与人工体验项。

## 验证门禁

先执行所属 context pack 的聚焦检查；共享输入、公共 API、渲染协议、后端与 feature 变更
运行完整门禁。构建使用一个编译任务，避免并行链接带来的内存峰值。

```powershell
$env:CARGO_BUILD_JOBS = '1'
cargo fmt --all -- --check
cargo test --no-default-features --quiet --locked -j 1
cargo test --features full --quiet --locked -j 1
.\scripts\check-feature-matrix.ps1 -Locked
.\scripts\check-native-boundary.ps1
.\scripts\ai-context.ps1 -Validate
git diff --check
```

构建、原生 proof、性能与人工体验分别保留结果；目标截图基准不得因测试失败自动覆盖。
优先交付 A，再推进 B/C、D、E，F 随各切片建设，G 完成统一收口。
