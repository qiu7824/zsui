# ZSUI UI Document

`ui-document` 是可选的开发与嵌入能力，不属于默认 feature。它定义版本化的语义
组件树、DP 布局约束、主题 token、本地化键、辅助功能元数据和稳定节点 ID；平台
后端仍负责生成各自的 Win32、AppKit 或 Linux 原生体验。

UI 文档只保存视觉结构。Rust 应用继续持有 `State` 和 `Msg`，并通过
`UiBindingManifest<State, Msg>` 注册状态读取器和消息映射器。序列化绑定名称只有
在显式清单中声明且类型一致时才有效，不形成反射式属性系统或全局字符串事件总线。

## 定位边界

`UiDocument` 是受限的原生 UI 声明格式，不是第二套 Web 平台。它不拥有动态脚本、
任意字符串属性、通用运行时解释、业务状态机、网络资源模型或浏览器样式系统。新增
组件必须先成为可复用的 ZSUI 组件与 Cargo feature，再进入 schema；不能为了 Viewer
预览而反向定义框架组件。

发布应用只使用按文档实际组件裁剪的 `ui-document-runtime` 或预编译嵌入结果。
`ui-viewer` 的文件监听、开发诊断、全组件覆盖与截图能力属于独立工具产物。正式应用
和 Viewer 的二进制、内存与进程数通过独立性能档位验收，详见
[UI 性能测试矩阵](ui-performance-matrix.md)。

## 校验

启用文档使用到的组件 feature，并运行：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,button,label `
  -- check examples/ui-documents/basic.json `
  --bindings examples/ui-documents/basic.bindings.json
```

ContentDialog 文档可以用同一校验器验证：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,dialog,label `
  -- check examples/ui-documents/content-dialog.json `
  --bindings examples/ui-documents/content-dialog.bindings.json
```

Toast 文档可以用同一校验器验证：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,toast,label `
  -- check examples/ui-documents/toast.json `
  --bindings examples/ui-documents/toast.bindings.json
```

Tooltip 文档可以用同一校验器验证：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,tooltip,button,label `
  -- check examples/ui-documents/tooltip.json `
  --bindings examples/ui-documents/tooltip.bindings.json
```

TeachingTip 文档可以用同一校验器验证：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,teaching-tip,button,label `
  -- check examples/ui-documents/teaching-tip.json `
  --bindings examples/ui-documents/teaching-tip.bindings.json
```

BreadcrumbBar 文档可以用同一校验器验证：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,breadcrumb,label `
  -- check examples/ui-documents/breadcrumb.json `
  --bindings examples/ui-documents/breadcrumb.bindings.json
```

Flyout 文档可以用同一校验器验证：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,flyout,button,label `
  -- check examples/ui-documents/flyout.json `
  --bindings examples/ui-documents/flyout.bindings.json
```

MenuFlyout 文档可以用同一校验器验证：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,menu-flyout,button,label `
  -- check examples/ui-documents/menu-flyout.json `
  --bindings examples/ui-documents/menu-flyout.bindings.json
```

DataGrid 文档可以用同一校验器验证：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,table,label `
  -- check examples/ui-documents/table.json `
  --bindings examples/ui-documents/table.bindings.json
```

NavigationView 文档使用同一份语义条目和内容页，由目标平台选择展开窗格、紧凑导航轨、
AppKit source list 或 Linux 侧边栏结构：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,shell `
  -- check examples/ui-documents/navigation.json `
  --bindings examples/ui-documents/navigation.bindings.json
```

CommandBar 文档使用直接子节点的稳定 ID 划分首尾命令组；按钮只声明语义图标和
`standard`、`primary`、`toolbar` 或 `icon` 呈现，平台 profile 决定最终工具栏样式：

```powershell
cargo run --bin zsui-uic `
  --no-default-features `
  --features ui-document,document-shell,label `
  -- check examples/ui-documents/command-bar.json `
  --bindings examples/ui-documents/command-bar.bindings.json
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
`radio_button`、`slider`、`number_box`、`combo_box`、`auto_suggest`、`command_palette`、`tree`、`grid_view`、`table`、`date_picker`、`time_picker`、`color_picker`、`password_box`、`list`、`tabs`、`grid`、
`progress_bar`、`progress_ring`、`toast`、`info_bar`、`content_dialog`、`tooltip`、
`teaching_tip`、`flyout`、`menu_flyout`、`breadcrumb`、`navigation`、`command_bar` 和 `scroll`。
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

`auto_suggest.suggestions` 使用 `auto_suggestion_array`，每项由合法且唯一的稳定字符串 ID
和显示文字组成；项目重排不会改变身份。`query`、`highlighted` 与 `expanded` 分别保存查询、
可空高亮 ID 和展开状态。`text_change` 发送 string，`choose` 发送
`auto_suggestion_id`，`submit` 发送同时包含 `query` 与可空 `chosen` 的
`auto_suggest_submission`，`expanded_change` 发送 boolean。
`UiBindingManifest::register_auto_suggestions_property`、
`register_auto_suggestion_id_property`、`register_auto_suggestion_id_action` 和
`register_auto_suggest_submission_action` 在 Rust 侧保留强类型语义 ID；发布运行时才把
节点 ID 与建议 ID 确定性映射为私有 `ZsAutoSuggestionId`。重复、非法或不存在的高亮 ID
会在进入平台宿主前被拒绝，Viewer 受控重建不会依赖数组下标或全局注册表。

`command_palette.items` 使用 `command_palette_item_array`，每个命令包含稳定字符串 ID、
非空标题、可选副标题、搜索关键字、快捷键、`ZsIcon` 语义图标和启用状态。`query`、
`highlighted` 与 `open` 分别保存查询、可空高亮 ID 和打开状态；`query_change`、
`highlight_change`、`invoke` 与 `open_change` 形成完整受控闭环。Rust 应用可通过
`register_command_palette_items_property`、`register_command_palette_item_id_property`
和 `register_command_palette_item_id_action` 保留强类型语义 ID。发布运行时根据拥有者
节点和命令 ID 生成私有 `ZsCommandPaletteItemId`，并拒绝碰撞、不存在、被禁用或不匹配
当前查询的高亮项。命令重排不会改变身份，ZSUI 只返回调用 ID，不执行产品命令。

`tree.nodes` 使用递归 `tree_node_array`，节点包含全树唯一的稳定字符串 ID、非空标签、
可选语义图标、子节点和惰性子节点标记。`expanded` 使用去重的 `tree_node_id_array` 保存
完整展开集合，`selected` 使用 `nullable_tree_node_id` 保存允许暂时隐藏的选择；`select`、
`expanded_change` 与 `invoke` 分别返回稳定语义 ID 或完整的新展开集合。Rust 应用可通过
`register_tree_nodes_property`、`register_tree_node_ids_property`、
`register_tree_node_id_property` 及对应 action helper 保持强类型状态。发布运行时根据拥有者
节点和语义 ID 生成私有 `ZsTreeNodeId`，拒绝碰撞、未知选择以及指向叶节点的展开状态；
节点重排、折叠和 Viewer 热重建不会把身份降级为路径或数组下标。

`navigation.items` 与可选 `footer_items` 使用 `navigation_item_array`，每项包含跨分组唯一的
稳定字符串 ID、非空标签、语义 `ZsIcon` 和启用状态；`selected` 使用
`nullable_navigation_item_id`，`select` 返回 `navigation_item_id`。Rust 应用通过
`register_navigation_items_property`、`register_navigation_item_id_property` 和
`register_navigation_item_id_action` 保持强类型状态。文档只包含一个语义内容子树，发布
运行时为导航行生成保留命名空间内的稳定子控件 ID，并拒绝跨分组重复、不存在或禁用的
选择。应用可声明平台无关的 `pane_width` 与 `minimum_content_width`，但展开、紧凑、覆盖、
source list 及侧边栏组合仍由 Windows、AppKit 和 Linux 的体验 profile 决定。

`command_bar` 至少包含一个直接子节点，`trailing` 使用去重的稳定子节点 ID 数组划分
尾部命令组；未列出的子节点保持原始顺序留在首部。按钮的 `presentation` 与 `icon` 是
平台无关的语义元数据：工具栏按钮和纯图标按钮必须声明 `ZsIcon`，普通和主按钮不接收
图标。发布运行时把文档编译为同一个 `ZsCommandBarSpec` 与 Button 事件路径，平台仍拥有
工具栏尺寸、间距、图标来源和 chrome；文档不包含平台枚举，也不静默丢弃溢出命令。

`grid_view.items` 使用 `grid_view_item_array`，每个磁贴包含唯一稳定字符串 ID、非空标题、
可选副标题和 `ZsIcon` 语义图标。`selected` 使用 `nullable_grid_view_item_id` 保存显式单选
状态，`select` 与 `invoke` 分别返回 `grid_view_item_id`。Rust 应用通过
`register_grid_view_items_property`、`register_grid_view_item_id_property` 和
`register_grid_view_item_id_action` 保留强类型语义 ID；发布运行时根据拥有者节点与项目
ID 生成私有 `ZsGridViewItemId`，拒绝碰撞、重复项目及不存在的选择。项目重排和响应式
列数变化不会改变身份，最终列宽、磁贴高度、间距和选择视觉继续由目标平台 profile 决定。

`table.columns` 与 `table.rows` 分别使用 `table_column_array` 和 `table_row_array`。列与行都
使用唯一稳定字符串 ID；每一行的 `cells` 以列 ID 为键并且必须恰好覆盖全部列，因此重排列
不会把数据移动到错误的语义字段。列宽明确为正数 DP 或正整数 fill 权重，并可声明对齐和
是否允许排序。`selected` 与 `sort` 使用 `nullable_table_row_id` 和
`nullable_table_sort`，`select`、`invoke`、`sort` 分别返回强类型行 ID 或列 ID 加方向。
Rust 应用通过 `register_table_*` property/action helper 保持这些作者身份；发布运行时根据
拥有者节点生成私有 `ZsTableColumnId`/`ZsTableRowId`，拒绝映射碰撞、缺失单元格、不存在的
选择以及指向不可排序列的排序。最终表头、行高、分隔线、选择与排序指示继续由目标平台的
DataGrid profile 决定。

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

`color_picker` 使用独立的 `color` 绑定类型，规范序列化形式固定为大写 `#RRGGBBAA`。
`UiBindingManifest::register_color_property` 和 `register_color_action` 在 Rust 侧直接读写
`Color`。`value`、`expanded` 与 `active_channel` 可分别绑定受控状态，三个对应动作在
Viewer 重建后保持 RGBA、弹层和当前编辑通道；通道值只接受 `red`、`green`、`blue` 或
`alpha`。关闭 `alpha_enabled` 时，校验器和发布运行时都会拒绝非 `FF` alpha 以及活动
alpha 通道。最终色谱、滑轨、密度与弹层结构仍由 WinUI、AppKit 或 GTK 体验参数决定。

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

`content_dialog` 必须有且只有一个内容子树。`open` 控制模态层是否显示，`title`、`content`、
`primary_button`、`secondary_button` 和 `close_button` 描述语义内容；`close_button` 和
`content` 不能为空。`default_button` 与 `destructive_button` 只接受 `primary`、
`secondary` 或 `close`，并且必须指向已提供的按钮且彼此不同。`result` 动作发送
`"primary"`、`"secondary"` 或 `"close"` 字符串，Rust 绑定仍通过显式
`UiBindingManifest` 声明。绑定 `open` 时必须同时声明布尔型 `open_change` 动作；响应会
先发送结果，再发送 `false` 并回写同一状态绑定，因此 Viewer 重建后不会重新打开已经
关闭的弹窗。Viewer 和发布运行时使用同一个稳定节点 ID，类型化回调直接进入应用消息，
不使用全局事件表。

`info_bar` 是不接受子节点的内联状态表面。`message` 必须是非空字符串，`title` 和
`action_label` 提供时也不能为空；`severity` 只接受 `informational`、`success`、
`warning` 或 `error`，`closable` 默认为 `true`。`event` 动作发送 `"action"` 或
`"close"` 字符串，应用仍通过显式强类型绑定决定后续状态和是否从下一棵 View 中移除
该节点。UiDocument 运行时只编译语义 `ZsInfoBarSpec`，最终高度、图标、动作按钮处理、
圆角与间距继续分别由 Win32、AppKit 和 Linux 的组件 profile 决定。闭包回调保留稳定
节点和绑定身份，不依赖全局控件表。

`toast` 包装且只包装一个页面子树。`open` 和 `message` 是必需属性；`action_label`
提供时不能为空，`duration` 只接受 `short`、`long` 或 `persistent`。`result` 动作
发送 `"action"`、`"close"`、`"escape"` 或 `"timeout"`，`open_change` 在任意响应后
发送 `false`，并可直接更新受控 `open` 绑定。Viewer 因此不会在重建时重新显示已关闭的
Toast；超时仍沿用平台运行时的计时器。Toast 的定位、圆角、按钮间距和计时调度继续由
各平台 profile 与宿主负责，文档运行时只生成语义 `ZsToastSpec` 和强类型回调。

`tooltip` 包装且只包装一个子控件。`text` 是必需的非空字符串，`placement` 只接受
`auto`、`top`、`bottom`、`left` 或 `right`；可选的 `open_delay_ms` 是非负整数，
主要用于固定预览和自动化验收。运行时把 `ZsTooltipSpec` 附着到子控件并保留子控件的
稳定 `WidgetId`，因此不会新增命中区域、控件注册项或第二条事件路径。悬停/焦点计时、
平台尺寸、圆角、文字行框和最终放置仍由 Win32、AppKit 与 Linux 的组件 profile 和
宿主负责。

`teaching_tip` 包装且只包装一个页面子树。`target` 使用该子树内的稳定 `UiNodeId`，
静态文档和解析后的绑定值都会验证引用存在；`title` 与 `subtitle` 至少提供一个，
可选 `action_label` 以及 `auto`、`top`、`bottom`、`left`、`right` 放置语义。
`result` 动作发送 `"action"`、`"close"` 或 `"escape"`；任意响应后
`open_change` 都发送 `false` 并可回写受控 `open` 绑定，因此 Viewer 重建不会重新打开
已经关闭的提示。文档运行时只组装 `ZsTeachingTipSpec`、稳定目标和类型化回调；尾部、
尺寸、字体、按钮顺序与最终放置继续由各平台 profile 和共享原生宿主决定。

`flyout` 恰好包含两个直接子树：第一个是普通页面，第二个是浮层中的任意 View 内容。
`target` 必须引用页面子树内的稳定 `UiNodeId`，不能指向浮层自身；静态文档与绑定解析后
都会校验该引用。`content_width` 和 `content_height` 必须为正数，`placement` 只接受
`auto`、`top`、`bottom`、`left` 或 `right`。`dismiss` 使用
`flyout_dismiss_reason`，只发送 `light_dismiss` 或 `escape`；绑定 `open` 时必须同时声明
布尔型 `open_change`，关闭后会发送 `false` 并回写状态。运行时因此可在 Viewer 热重建
后保持关闭状态，同时让浮层内部按钮继续使用普通强类型动作路径。首次聚焦跳过仅用于
布局的稳定容器节点，优先落到浮层中的第一个可交互控件；尺寸、圆角、阴影和最终放置
仍由 Win32、AppKit 与 Linux 的 Flyout profile 决定。

`menu_flyout` 恰好包装一个普通页面子树，`target` 必须引用该页面中的稳定
`UiNodeId`。`items` 使用 `menu_flyout_item_array` 表达命令、分隔符和最多八层的子菜单；
命令与子菜单 ID 在整棵菜单中共享唯一命名空间，重排菜单项不会改变身份。校验器会拒绝
空菜单、空标签、重复 ID、首尾或连续分隔符、过深子菜单和非规范快捷键。快捷键用结构化
修饰键与可移植键名声明，其中 `primary` 在 Windows/Linux 映射为 Control，在 macOS
映射为 Command。`invoke` 返回 `menu_flyout_item_id`，`open` 与布尔型
`open_change` 形成受控状态闭环。发布运行时只将稳定作者 ID 映射为私有 `Command`，不会
执行产品命令或引入字符串事件总线；行高、级联方向、选中标记、快捷键显示、字体和延时
继续由 Win32、AppKit 与 Linux 各自的 MenuFlyout profile 和原生宿主决定。

`breadcrumb.items` 使用 `breadcrumb_item_array`，每个路径项包含唯一稳定字符串 ID 和
非空标签；数组顺序只表达从根到当前位置的路径，不作为项目身份。`expanded` 与
`expanded_change` 形成受控溢出开关，`select` 返回 `breadcrumb_item_id`。Rust 应用可用
`register_breadcrumb_items_property` 和 `register_breadcrumb_item_id_action` 保留强类型
语义 ID。发布运行时根据拥有者节点和项目 ID 确定性生成私有 `ZsBreadcrumbId`，重排或
Viewer 热重建不会改变身份；折叠策略、行高、分隔符、字体和溢出表面继续由 Win32、
AppKit 与 Linux 各自的 BreadcrumbBar profile 决定。

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
`checkbox.toggle`、`toggle.toggle`、`slider.slide`、AutoSuggestBox 四类状态动作、CommandPalette 四类状态动作、TreeView 三类状态动作、GridView 两类状态动作、DataGrid 三类状态动作、DatePicker 三类状态动作、TimePicker 两类状态动作、ColorPicker 三类状态动作和 `scroll.scroll` 均走类型化 Viewer
消息。带值控件
通过按控件持有的 `ViewMessageMapper` 捕获稳定节点 ID、动作绑定和可选属性绑定；普通
函数指针路径不分配堆内存，只有显式使用 `*_with` 捕获回调时才分配共享闭包。动作 payload
通过 binding schema 校验后记录在 `UiViewerState`；存在对应属性绑定时，Viewer 同步
更新该显式状态值，使受控文本、开关、滑块和滚动交互不会在 View 重建后回退。这里没有
全局注册表、反射属性或字符串事件总线。

受控 ContentDialog 示例使用布尔 `open` 状态；点击按钮后通过 `open_change` 回写关闭状态：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/content-dialog.json `
  --bindings examples/ui-documents/content-dialog.bindings.json `
  --values examples/ui-documents/content-dialog.values.json
```

`result` 先发送语义按钮结果，`open_change` 再发送 `false` 并更新 `delete_open`；
因此文件监听触发的下一次重建仍保持弹窗关闭。

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

受控颜色示例使用规范 RGBA 状态，并由目标平台选择自身的 ColorPicker 体验参数：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/color-picker.json `
  --bindings examples/ui-documents/color-picker.bindings.json `
  --values examples/ui-documents/color-picker.values.json
```

受控自动建议示例使用稳定语义 ID，并由目标平台选择搜索框与建议弹层参数：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/auto-suggest.json `
  --bindings examples/ui-documents/auto-suggest.bindings.json `
  --values examples/ui-documents/auto-suggest.values.json
```

受控命令面板示例使用稳定命令 ID 和语义图标，平台仍分别选择 Fluent、AppKit 或 GTK
体验参数：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/command-palette.json `
  --bindings examples/ui-documents/command-palette.bindings.json `
  --values examples/ui-documents/command-palette.values.json
```

受控项目树示例使用稳定层级 ID、完整展开集合和平台语义图标：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/tree.json `
  --bindings examples/ui-documents/tree.bindings.json `
  --values examples/ui-documents/tree.values.json
```

受控资源库示例使用稳定项目 ID、响应式列和平台语义图标：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/grid-view.json `
  --bindings examples/ui-documents/grid-view.bindings.json `
  --values examples/ui-documents/grid-view.values.json
```

受控 DataGrid 示例使用稳定列/行 ID、按列 ID 键控的单元格和显式排序状态：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/table.json `
  --bindings examples/ui-documents/table.bindings.json `
  --values examples/ui-documents/table.values.json
```

受控 NavigationView 示例使用同一组语义条目和内容树，目标平台自行选择导航组合：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/navigation.json `
  --bindings examples/ui-documents/navigation.bindings.json `
  --values examples/ui-documents/navigation.values.json
```

CommandBar 示例使用同一份稳定命令组、语义图标和类型化按钮动作，由目标平台决定工具栏
度量和视觉：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/command-bar.json `
  --bindings examples/ui-documents/command-bar.bindings.json `
  --values examples/ui-documents/command-bar.values.json
```

受控面包屑示例使用稳定路径项目 ID，并由目标平台决定折叠、分隔符和溢出表面：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/breadcrumb.json `
  --bindings examples/ui-documents/breadcrumb.bindings.json `
  --values examples/ui-documents/breadcrumb.values.json
```

受控 Flyout 示例让同一份页面与任意浮层内容进入目标平台，并保留强类型关闭原因：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/flyout.json `
  --bindings examples/ui-documents/flyout.bindings.json `
  --values examples/ui-documents/flyout.values.json
```

受控 MenuFlyout 示例使用稳定嵌套菜单 ID、勾选状态和平台快捷键表示：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/menu-flyout.json `
  --bindings examples/ui-documents/menu-flyout.bindings.json `
  --values examples/ui-documents/menu-flyout.values.json
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

Windows AutoSuggestBox 受控示例的本地原生证明使用一次真实建议行点击，记录 1 次已处理
点击、4 条 Viewer 消息、0 次未处理点击，并在最终 Win32 PNG 中保留提交后的查询。
Windows CommandPalette 受控示例的本地原生证明点击第二个真实命令行，记录 1 次调用、
1 次关闭、2 条 Viewer 消息和 0 次未处理点击；最终 Win32 PNG 来自关闭后的共享页面。
Windows TreeView 受控示例点击一个真实文件行，记录 1 次选择、1 次调用、2 条 Viewer
消息和 0 次未处理点击；最终 Win32 PNG 保留完整页面文字、层级图标和新的稳定 ID 选择。
Windows GridView 受控示例点击一个真实磁贴，记录 1 次选择、1 次调用、2 条 Viewer 消息
和 0 次未处理点击；最终 Win32 PNG 保留完整双语页面、六个语义图标和新的稳定 ID 选择。
Windows DataGrid 受控示例依次点击一个真实数据行和可排序表头，记录 1 次选择、1 次调用、
1 次排序、3 条 Viewer 消息和 0 次未处理点击；最终 Win32 PNG 保留新的稳定行选择、降序
指示、完整双语列头和不压缩的单元格文字。
Windows NavigationView 受控示例在 1200×720 最终窗口中使用展开窗格，点击真实导航行后
记录 1 条 Viewer 消息和 0 次未处理点击；进程拆除前 RSS 为 17,022,976 字节，Private
bytes 为 5,955,584 字节。紧凑宽度下同一文档自动折叠为导航轨，不需要文档平台分支。
Windows CommandBar 示例点击真实 Save 工具栏按钮后记录 1 条 Viewer 消息和 0 次未处理
点击；960×640 最终 Win32 PNG 保留首部 Save/Undo/Cut 与尾部 About 命令，进程拆除前
RSS 为 16,035,840 字节，Private bytes 为 4,882,432 字节。
Windows BreadcrumbBar 受控示例点击真实溢出按钮，记录 1 次展开状态变化、1 条 Viewer
消息和 0 次未处理点击；最终 Win32 PNG 保留目标平台绘制的路径和溢出表面。
Windows Flyout 受控示例分别点击浮层内真实按钮与外部遮罩：两条场景都记录 1 次已处理
点击和 0 次未处理点击，内部动作场景累计 3 条 Viewer 消息，轻触关闭场景累计 2 条；
最终 Win32 PNG 分别保留平台浮层和关闭后的普通页面。首次焦点落在真实按钮，不再把
仅用于布局的 UiDocument Stack 绘制成整块蓝色焦点框。
Windows MenuFlyout 受控示例点击一个真实命令行，记录 1 次命令调用、1 次受控关闭、
2 条 Viewer 消息和 0 次未处理点击。最终 Win32 PNG 来自关闭后的普通页面；单独的打开
状态捕获保留完整双语菜单、勾选状态、子菜单指示和未截断的 `Ctrl+N`/`Ctrl+S` 快捷键。

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
