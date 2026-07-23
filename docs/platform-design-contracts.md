# ZSUI 平台视觉契约

ZSUI 保留一棵共享的自绘 View 树，但不把 Windows 的组件组合复制到其他桌面。平台差异由框架的组合原语、语义颜色、控件度量和后端字体/缩放解析共同决定。

## Windows / WinUI

实现依据：

- [Alignment, margin, and padding](https://learn.microsoft.com/en-us/windows/apps/design/layout/alignment-margin-padding)
- [Content layout and spacing](https://learn.microsoft.com/en-us/windows/apps/design/basics/content-basics)
- [NavigationView](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/navigationview)
- [TabView](https://learn.microsoft.com/en-us/windows/apps/design/controls/tab-view)
- [Command bar](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/command-bar)
- [InfoBadge](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/info-badge)
- [SplitView](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/split-view)
- [XAML type ramp](https://learn.microsoft.com/en-us/windows/apps/develop/platform/xaml/xaml-theme-resources)
- [NavigationView theme resources](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/NavigationView/NavigationView_themeresources.xaml)
- [TabView theme resources](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/TabView/TabView_themeresources.xaml)
- [AppBarButton theme resources](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommonStyles/AppBarButton_themeresources.xaml)

框架参数：

| 参数 | 契约 |
| --- | --- |
| 布局单位 | effective pixels；固定尺寸、边距和内边距优先使用 4 epx 的倍数 |
| 内容边距 | 窄窗口使用 12 epx，大窗口使用 24 epx |
| 常用间距 | 控件与控件 8 epx，控件与标签/内容 12 epx，表面边缘与文字 16 epx |
| 导航组合 | `NavigationView` 左侧或顶部模式；左侧导航适用于约 5–10 个同等重要的顶级分类 |
| 导航自适应 | `PaneDisplayMode=Auto`：窗口宽度至少 1008 epx 使用 320 epx 展开栏，641–1007 epx 使用 48 epx 紧凑栏，至多 640 epx 使用带 52 epx 内容标题的 Minimal 模式 |
| 选中态 | `Control` 填充和左侧 3×16 epx 指示条；这是 Windows 分支的组合契约，不用于 macOS/GTK |
| 文档标签 | `TabViewItem` 的语义图标和 Header 位于同一标签行，文字在图标右侧；内容属于所选标签 |
| 命令栏 | `DefaultLabelPosition="Right"` 时使用 48 epx compact 高度、20×20 epx primary icon、8 epx 图标/标签间距和 12 epx `AppBarButton` 标签；窄窗口的动态 overflow 是独立布局能力 |
| 标签文字 | `TabViewItemHeaderFontSize` 是 12 epx，图标为 16 epx，图标后间距 10 epx；标签仍与图标位于同一行，不把 Windows 的 12 epx 值复制到 AppKit/GTK |
| 框架窗口标题 | `WindowTitle` 使用系统 UI 字体、24/32 epx 和 Semibold；这是 ZSClip 桌面壳层的标题语义，不把内容 `Title` 的 28/36 epx 强行用于窗口标题 |
| 信息徽章 | `Dot` 使用 4 epx 圆点；数字和语义图标使用至少 16 epx 的胶囊，数字增加时只扩展宽度。徽章不参与焦点或点击，父控件负责辅助功能状态通知 |
| 拆分视图 | `SplitView` 保持内容常驻，以 296 epx 作为 Windows 首选打开宽度；应用可选择 `inline`、`overlay` 或约束驱动的 `adaptive`。覆盖层外点击与 Esc 关闭窗格，应用仍显式持有 `open` 状态 |

Windows 文字族由 Win32 `SPI_GETNONCLIENTMETRICS.message_font` 在运行时解析，
与 ZSClip 设置界面使用同一套系统消息字体（当前系统通常是 `Microsoft YaHei UI`）。
`Segoe UI` 只作为系统查询失败或字体不可用时的框架回退；Demo 和组件实现不得直接
写入字体族。

## macOS / AppKit

实现依据：

- [Sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars)
- [NSSplitViewController minimumThicknessForInlineSidebars](https://developer.apple.com/documentation/appkit/nssplitviewcontroller/minimumthicknessforinlinesidebars)
- [NSSplitViewItem sidebar initializer](https://developer.apple.com/documentation/appkit/nssplitviewitem/init%28sidebarwithviewcontroller%3A%29)
- [NSSplitView](https://developer.apple.com/documentation/appkit/nssplitview)
- [Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars)
- [NSControlSize](https://developer.apple.com/documentation/appkit/nscontrol/controlsize-swift.enum)
- [NSBezelStyle](https://developer.apple.com/documentation/appkit/nsbezelstyle)
- [NSTabView](https://developer.apple.com/documentation/appkit/nstabview)
- [Typography](https://developer.apple.com/design/human-interface-guidelines/typography)
- [NSFont](https://developer.apple.com/documentation/appkit/nsfont)
- [NSDockTile badgeLabel](https://developer.apple.com/documentation/appkit/nsdocktile/badgelabel)

组合契约：

- 侧边栏是 leading-side 的 source list，内容可以延伸到其下方；使用熟悉的符号，并保留显示/隐藏侧边栏的系统交互。
- 共享布局按侧边栏和内容的最小宽度约束决定是否折叠，不编造固定 AppKit 断点；240 pt 仅是当前 ZSUI 首选厚度，后端尚需把系统标准侧边栏的运行时最小/最大厚度反馈给布局。
- 通用 `split_view` 在 AppKit profile 中使用 NSSplitView 式 1 pt 分隔线、240 pt 首选窗格和系统表面颜色；`inline` 保持线性分区，`overlay` 只用于应用明确声明或宽度约束不足时，不复制 Windows compact rail。
- 工具栏只保留少量上下文相关的主要动作，留出可拖动的空白区域；图标型工具栏按钮不绘制 WinUI 式外框。
- 控件大小使用 AppKit 的 `regular`、`small`、`mini` 语义和系统字体配对。Apple HIG 不给出一组跨版本固定像素，因此框架不把 Windows 的 32 epx 高度冒充 AppKit 默认值。
- 选中行使用 source-list 的轻量选中背景，不使用 Windows 左侧指示条；文本和图标保持 AppKit 的主/次级层级。
- 内容标签使用 AppKit `NSTabView` 的成组等宽标签语法和系统文字，选中项属于分段表面；不绘制 WinUI accent 下划线，也不冒充 `NSWindow` 的跨窗口系统标签组。
- AppKit 没有与 WinUI `InfoBadge` 对等的通用视图控件；系统只为 Dock tile 等宿主表面定义 Badge 外观。ZSUI 的独立 Badge 因此使用 6 pt 状态点、至少 18 pt 的紧凑胶囊、Caption 系统字体和 AppKit 语义颜色，不复制 Fluent 的固定组合。应用 Dock Badge 仍属于原生宿主服务，而不是 View 控件。

AppKit 字体契约由框架统一解析，不由示例调整：

| `TextRole` | macOS 系统文字样式 | 字号 / 行高 / 默认字重 |
| --- | --- | --- |
| `Caption` | Caption 1 | 10 / 13 pt / Regular |
| `Body`、`Button`、`Monospace` | Body | 13 / 16 pt / Regular |
| `BodyLarge` | Title 3 | 15 / 20 pt / Regular |
| `Subtitle` | Title 2 | 17 / 22 pt / Regular |
| `Title` | Title 1 | 22 / 26 pt / Regular |
| `TitleLarge`、`Display` | Large Title | 26 / 32 pt / Regular |

AppKit 后端使用 `NSFont` 系统字体而不是嵌入 SF Pro；Core Text 塑形、
`NSString` 测量与最终绘制必须使用同一套字号和行高。标签固有高度、编辑器视觉行、
选区、光标和命中测试同时读取共享的 `ZsPlatformStyle::Macos` 配置，禁止在组件或
Demo 中另写 14/20 的 Windows 字体常量。`TextWeight::Automatic` 使用平台文字
样式的默认字重；应用显式指定的 Regular、Medium、Semibold 或 Bold 不得被平台
解析器覆盖。

## Linux / GTK4 + Libadwaita

实现依据：

- [Boxed lists](https://developer.gnome.org/hig/patterns/containers/boxed-lists.html)
- [Header bars](https://developer.gnome.org/hig/patterns/containers/header-bars.html)
- [Libadwaita style classes](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.9/style-classes.html)
- [Libadwaita adaptive layouts](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/adaptive-layouts.html)
- [AdwNavigationSplitView](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.NavigationSplitView.html)
- [AdwOverlaySplitView](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.OverlaySplitView.html)
- [GNOME typography](https://developer.gnome.org/hig/guidelines/typography.html)
- [GNOME tabs](https://developer.gnome.org/hig/patterns/nav/tabs.html)
- [AdwTabBar](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.TabBar.html)
- [AdwViewStackPage badge number](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ViewStackPage.html)

框架参数与组合：

| 参数 | 契约 |
| --- | --- |
| boxed list 外边距 | Adwaita 自适应示例使用上下 24 px、左右 12 px、列表之间 24 px；ZSUI 的 section 原语保留这一层级关系 |
| 行结构 | 语义分组、行间 1 px 分隔线；一行通常只放一个控件，最多两个控件 |
| 侧边栏选中态 | 使用 `navigation-sidebar` 的 neutral selected row；按官方 `$selected_color` 以当前前景色 10% 混合透明背景，不使用 accent 填充 |
| 侧边栏自适应 | 展开宽度取可用宽度的 25%，限制在 180–280 sp；默认 max-width 400 sp 进入单页模式，应用声明的内容最小宽度可以把断点向上提高 |
| header bar | 左/中/右三组对齐，只放少量主要动作；工具栏按钮尽可能 flat，并保留可拖动空白 |
| 文档标签 | 使用 `AdwTabView`/`AdwTabBar` 的可变文档集合语义；选中项是 bar 内的圆角中性表面，不复用 WinUI accent 下划线 |
| 字体 | 运行时读取 `GtkSettings:gtk-font-name` 的系统字体族和字号；Body 使用系统基准，Caption 使用 82% 字号和 140% 行高，标题按 libadwaita 标准相对级别解析 |
| 颜色与高对比度 | 使用 Adwaita 的 accent、border、disabled 和 scheme 语义，不写死蓝色或透明度 |
| 信息徽章 | 对齐 `AdwViewStackPage` 的 `badge-number`/`needs-attention` 语义：6 px 状态点、至少 18 px 胶囊、Caption/Bold 数字；`accent`、`success`、`warning`、`error` 由当前 Adwaita 颜色语义解析，不嵌入 WinUI 色值 |
| 拆分视图 | 对齐 `AdwOverlaySplitView` 的 sidebar/content 双子树和 collapsed overlay 语义；首选窗格为 260 px，宽窗口并排，内容约束不足时覆盖。窗格位置保留 leading/trailing 语义；当前共享布局使用 LTR 映射，完整 RTL 解析仍是明确的就绪门槛 |

GTK 的 HIG 同样没有承诺所有主题都使用同一像素高度。ZSUI 只把有官方来源的边距、分隔线和组合规则写入共享框架；字体、DPI、主题控件高度由 GTK 后端和 `NativeDrawPalette` 解析。

## 文字完整性

- 单行控件使用平台语义行高和尾部省略号；省略必须是明确的窄宽度降级，不得由错误的固定高度或估算宽度触发。
- 可换行标签不固定为单行高度；共享 View 树保留显式段落行，最终换行、塑形和测量分别由 DirectWrite/GDI、Core Text/AppKit 与 Pango 完成。
- AppKit 的 preferred body font 与 GTK 的 `gtk-font-name` 在宿主启动时产生同一份运行时字体比例；该比例同时进入 View 布局、DrawPlan、平台塑形、编辑器可视行、选区和光标，不允许绘制字号与布局行高各用一套值。
- 按钮最小宽度必须包含 Unicode 全宽字符、组合字符和平台塑形余量；父布局不得把控件压到其文本最小宽度以下。
- Stack 与 Grid 的固有尺寸递归包含子节点行框、控件最小尺寸、间距和两侧内边距；横向 Stack 按子节点实际获得的宽度计算换行高度。主轴空间不足时允许上层滚动或裁剪，不缩小这些硬约束。
- Native Proof 的结构报告与最终平台截图共同检查文字边界；共享 DrawPlan 不能代替 AppKit/GTK 的最终文字验收。

## 验收规则

平台验收必须同时检查：

1. 组合结构（导航、分组、工具栏/header bar、标签页和弹层）是否符合平台契约；
2. 语义度量（边距、行高、控件最小尺寸）是否没有挤压或截断文字；
3. 最终 AppKit/GTK/Win32 视图截图和结构化布局报告，而不是只保存共享 DrawPlan。

如果官方规范没有给出跨版本固定像素，ZSUI 不得自行编造一个“系统默认值”；应使用语义枚举、主题查询或后端运行时度量。

`toolbar_button` 和 `command_bar(ZsCommandBarSpec)` 是上述工具栏契约的共享实现入口。`section`、`navigation_view(ZsNavigationViewSpec)` 同样不接受平台枚举。需要自适应侧边栏时，应用通过 `.minimum_content_width(...)` 和 `.content(WidgetId, ViewNode)` 声明内容约束与稳定身份；显示模式、覆盖层、焦点和平台参数全部由框架解析。应用只提供语义动作和强类型消息，不在 Demo 内复制 AppKit、GTK 或 WinUI 的分组分支；确定性平台选择只保留为框架内部验收钩子。

所有 Demo 标签都应通过 `text`/`styled_text` 的 `TextRole` 语义入口创建。字体族、
字号、行高、字重和文字塑形由 `NativeTypographyProfile` 与平台后端共同解析；共享
字体解析器同时服务布局、绘制、文本输入和 Native Proof，Demo 不得自建字体表或
绕过框架传入平台字体名。
