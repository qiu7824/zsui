# ZSUI 平台视觉契约

ZSUI 保留一棵共享的自绘 View 树，但不把 Windows 的组件组合复制到其他桌面。平台差异由框架的组合原语、语义颜色、控件度量和后端字体/缩放解析共同决定。

## Windows / WinUI

实现依据：

- [Alignment, margin, and padding](https://learn.microsoft.com/en-us/windows/apps/design/layout/alignment-margin-padding)
- [Content layout and spacing](https://learn.microsoft.com/en-us/windows/apps/design/basics/content-basics)
- [NavigationView](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/navigationview)

框架参数：

| 参数 | 契约 |
| --- | --- |
| 布局单位 | effective pixels；固定尺寸、边距和内边距优先使用 4 epx 的倍数 |
| 内容边距 | 窄窗口使用 12 epx，大窗口使用 24 epx |
| 常用间距 | 控件与控件 8 epx，控件与标签/内容 12 epx，表面边缘与文字 16 epx |
| 导航组合 | `NavigationView` 左侧或顶部模式；左侧导航适用于约 5–10 个同等重要的顶级分类 |
| 选中态 | `Control` 填充和左侧 3×16 epx 指示条；这是 Windows 分支的组合契约，不用于 macOS/GTK |

## macOS / AppKit

实现依据：

- [Sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars)
- [Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars)
- [NSControlSize](https://developer.apple.com/documentation/appkit/nscontrol/controlsize-swift.enum)
- [NSBezelStyle](https://developer.apple.com/documentation/appkit/nsbezelstyle)

组合契约：

- 侧边栏是 leading-side 的 source list，内容可以延伸到其下方；使用熟悉的符号，并保留显示/隐藏侧边栏的系统交互。
- 工具栏只保留少量上下文相关的主要动作，留出可拖动的空白区域；图标型工具栏按钮不绘制 WinUI 式外框。
- 控件大小使用 AppKit 的 `regular`、`small`、`mini` 语义和系统字体配对。Apple HIG 不给出一组跨版本固定像素，因此框架不把 Windows 的 32 epx 高度冒充 AppKit 默认值。
- 选中行使用 source-list 的轻量选中背景，不使用 Windows 左侧指示条；文本和图标保持 AppKit 的主/次级层级。

## Linux / GTK4 + Libadwaita

实现依据：

- [Boxed lists](https://developer.gnome.org/hig/patterns/containers/boxed-lists.html)
- [Header bars](https://developer.gnome.org/hig/patterns/containers/header-bars.html)
- [Libadwaita style classes](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.9/style-classes.html)
- [Libadwaita adaptive layouts](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/adaptive-layouts.html)

框架参数与组合：

| 参数 | 契约 |
| --- | --- |
| boxed list 外边距 | Adwaita 自适应示例使用上下 24 px、左右 12 px、列表之间 24 px；ZSUI 的 section 原语保留这一层级关系 |
| 行结构 | 语义分组、行间 1 px 分隔线；一行通常只放一个控件，最多两个控件 |
| 侧边栏选中态 | 使用 `navigation-sidebar` 的 neutral selected row，不使用 accent 填充；accent 留给可操作控件 |
| header bar | 左/中/右三组对齐，只放少量主要动作；工具栏按钮尽可能 flat，并保留可拖动空白 |
| 颜色与高对比度 | 使用 Adwaita 的 accent、border、disabled 和 scheme 语义，不写死蓝色或透明度 |

GTK 的 HIG 同样没有承诺所有主题都使用同一像素高度。ZSUI 只把有官方来源的边距、分隔线和组合规则写入共享框架；字体、DPI、主题控件高度由 GTK 后端和 `NativeDrawPalette` 解析。

## 验收规则

平台验收必须同时检查：

1. 组合结构（导航、分组、工具栏/header bar、标签页和弹层）是否符合平台契约；
2. 语义度量（边距、行高、控件最小尺寸）是否没有挤压或截断文字；
3. 最终 AppKit/GTK/Win32 视图截图和结构化布局报告，而不是只保存共享 DrawPlan。

如果官方规范没有给出跨版本固定像素，ZSUI 不得自行编造一个“系统默认值”；应使用语义枚举、主题查询或后端运行时度量。
