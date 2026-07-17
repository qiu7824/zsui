# ZSUI 平台视觉契约

ZSUI 保留一棵共享的自绘 View 树，但不把 Windows 的组件组合复制到其他桌面。平台差异由框架的组合原语、语义颜色、控件度量和后端字体/缩放解析共同决定。

## Windows / WinUI

实现依据：

- [Alignment, margin, and padding](https://learn.microsoft.com/en-us/windows/apps/design/layout/alignment-margin-padding)
- [Content layout and spacing](https://learn.microsoft.com/en-us/windows/apps/design/basics/content-basics)
- [NavigationView](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/navigationview)
- [TabView](https://learn.microsoft.com/en-us/windows/apps/design/controls/tab-view)
- [Command bar](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/command-bar)
- [XAML type ramp](https://learn.microsoft.com/en-us/windows/apps/develop/platform/xaml/xaml-theme-resources)

框架参数：

| 参数 | 契约 |
| --- | --- |
| 布局单位 | effective pixels；固定尺寸、边距和内边距优先使用 4 epx 的倍数 |
| 内容边距 | 窄窗口使用 12 epx，大窗口使用 24 epx |
| 常用间距 | 控件与控件 8 epx，控件与标签/内容 12 epx，表面边缘与文字 16 epx |
| 导航组合 | `NavigationView` 左侧或顶部模式；左侧导航适用于约 5–10 个同等重要的顶级分类 |
| 选中态 | `Control` 填充和左侧 3×16 epx 指示条；这是 Windows 分支的组合契约，不用于 macOS/GTK |
| 文档标签 | `TabViewItem` 的语义图标和 Header 位于同一标签行，文字在图标右侧；内容属于所选标签 |
| 命令栏 | Primary command 图标使用 20×20 epx；宽窗口可把 14 epx Body 标签放在图标右侧，紧凑态隐藏标签 |
| 标签文字 | 标签 Header 是 14 epx Body 控件内容，不使用 12 epx Caption；Caption 只承担次级元数据 |

## macOS / AppKit

实现依据：

- [Sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars)
- [Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars)
- [NSControlSize](https://developer.apple.com/documentation/appkit/nscontrol/controlsize-swift.enum)
- [NSBezelStyle](https://developer.apple.com/documentation/appkit/nsbezelstyle)
- [NSTabView](https://developer.apple.com/documentation/appkit/nstabview)
- [Typography](https://developer.apple.com/design/human-interface-guidelines/typography)
- [NSFont](https://developer.apple.com/documentation/appkit/nsfont)

组合契约：

- 侧边栏是 leading-side 的 source list，内容可以延伸到其下方；使用熟悉的符号，并保留显示/隐藏侧边栏的系统交互。
- 工具栏只保留少量上下文相关的主要动作，留出可拖动的空白区域；图标型工具栏按钮不绘制 WinUI 式外框。
- 控件大小使用 AppKit 的 `regular`、`small`、`mini` 语义和系统字体配对。Apple HIG 不给出一组跨版本固定像素，因此框架不把 Windows 的 32 epx 高度冒充 AppKit 默认值。
- 选中行使用 source-list 的轻量选中背景，不使用 Windows 左侧指示条；文本和图标保持 AppKit 的主/次级层级。
- 内容标签使用 AppKit `NSTabView` 的成组等宽标签语法和系统文字，选中项属于分段表面；不绘制 WinUI accent 下划线，也不冒充 `NSWindow` 的跨窗口系统标签组。

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
选区、光标和命中测试同时读取 `ZsTypographyPlatformStyle::Macos`，禁止在组件或
Demo 中另写 14/20 的 Windows 字体常量。`TextWeight::Automatic` 使用平台文字
样式的默认字重；应用显式指定的 Regular、Medium、Semibold 或 Bold 不得被平台
解析器覆盖。

## Linux / GTK4 + Libadwaita

实现依据：

- [Boxed lists](https://developer.gnome.org/hig/patterns/containers/boxed-lists.html)
- [Header bars](https://developer.gnome.org/hig/patterns/containers/header-bars.html)
- [Libadwaita style classes](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.9/style-classes.html)
- [Libadwaita adaptive layouts](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/adaptive-layouts.html)
- [GNOME typography](https://developer.gnome.org/hig/guidelines/typography.html)
- [GNOME tabs](https://developer.gnome.org/hig/patterns/nav/tabs.html)
- [AdwTabBar](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.TabBar.html)

框架参数与组合：

| 参数 | 契约 |
| --- | --- |
| boxed list 外边距 | Adwaita 自适应示例使用上下 24 px、左右 12 px、列表之间 24 px；ZSUI 的 section 原语保留这一层级关系 |
| 行结构 | 语义分组、行间 1 px 分隔线；一行通常只放一个控件，最多两个控件 |
| 侧边栏选中态 | 使用 `navigation-sidebar` 的 neutral selected row，不使用 accent 填充；accent 留给可操作控件 |
| header bar | 左/中/右三组对齐，只放少量主要动作；工具栏按钮尽可能 flat，并保留可拖动空白 |
| 文档标签 | 使用 `AdwTabView`/`AdwTabBar` 的可变文档集合语义；选中项是 bar 内的圆角中性表面，不复用 WinUI accent 下划线 |
| 字体 | 运行时读取 `GtkSettings:gtk-font-name` 的系统字体族；Body 使用系统基准，Caption 使用 82% 字号和 140% 行高，标题按 libadwaita 标准相对级别解析 |
| 颜色与高对比度 | 使用 Adwaita 的 accent、border、disabled 和 scheme 语义，不写死蓝色或透明度 |

GTK 的 HIG 同样没有承诺所有主题都使用同一像素高度。ZSUI 只把有官方来源的边距、分隔线和组合规则写入共享框架；字体、DPI、主题控件高度由 GTK 后端和 `NativeDrawPalette` 解析。

## 验收规则

平台验收必须同时检查：

1. 组合结构（导航、分组、工具栏/header bar、标签页和弹层）是否符合平台契约；
2. 语义度量（边距、行高、控件最小尺寸）是否没有挤压或截断文字；
3. 最终 AppKit/GTK/Win32 视图截图和结构化布局报告，而不是只保存共享 DrawPlan。

如果官方规范没有给出跨版本固定像素，ZSUI 不得自行编造一个“系统默认值”；应使用语义枚举、主题查询或后端运行时度量。

`toolbar_button` 和 `platform_document_command_bar_for_style` 是上述工具栏契约的共享实现入口。应用只提供语义动作和强类型消息，不在 Demo 内复制 AppKit、GTK 或 WinUI 的分组分支。
