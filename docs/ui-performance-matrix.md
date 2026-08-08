# UI 性能测试矩阵

ZSUI 的性能基线固定为四种独立构建，并只允许同一档位横向比较。正式应用与
`zsui-viewer` 是不同产物；Viewer 的文件监听、文档校验和全组件覆盖不得计入
正式应用的框架成本。

## 固定工作负载

| 配置 | 固定内容 | ZSUI 产物边界 |
| --- | --- | --- |
| Minimal | 一个 1000×700 窗口、标题、正文和按钮 | `window + label + button` |
| Common | 导航、可编辑表单、两行列表、确认表面和对话框能力 | 正式应用，不启用 UiDocument 或 Viewer |
| Full Native App | 导航、输入、选择、集合、进度和操作等 24 个可见常用控件实例 | 正式应用，`component-gallery-demo` 验收配置 |
| Viewer | 单一文档表面、250 ms 文件轮询和当前 schema 支持的 26 种文档组件 | 独立 `ui-viewer` 开发工具 |

Viewer 对照实现不额外加入三栏编辑器外壳。这样测到的是 UiDocument、热重载和
文档组件覆盖本身，而不是某个框架独有的开发工具界面。正式应用的 Cargo 构建不启用
`ui-document`、`ui-document-runtime` 或 `ui-viewer`，发布应用不会携带文件监听、
预览诊断器、浏览器运行时或额外进程。

## Windows 实测结果

测量环境为 Windows NT 10.0.26200.0、16 个逻辑处理器和 Rust 1.94.0。每个配置
使用独立 release 二进制，启动 5 次，内存预热 3 秒后采样 6 次，空闲与持续重绘
CPU 各采样 3 秒。除 Tauri 外均为单进程；Tauri 为 1 个应用进程加 6 个 WebView2
子进程。

“首次帧”是进程启动到可见非零客户区完成强制重绘和 `DwmFlush` 的代理值。
“首次”没有清空 Windows 文件缓存，因此只是构建后的首次启动近似值；“暖启动”
是其余 4 次的中位数。CPU 为整机百分比。峰值 RSS 取空窗口、完整页面、隐藏后和
持续重绘四阶段的最大值。

### Windows 文字后端 A/B/C

下表保留的是生产/证明代码拆分和按字节 LRU 缓存落地前的候选基线，不作为当前
提交优化后的结果；必须用同一脚本重新构建、测量后才更新数字。正式应用只启用
`windows-rust-text`，不得把 `rust-text-proof`/`windows-text-proof` 的代码体积或
内存计入生产后端。

拆分后的同一 Minimal release 构建已复核为 2,263.5 KiB，比表内旧构建增加
5.0 KiB；RSS/Private RSS 尚未按完整交替运行方法重测，因此仍保留旧内存行。这个
结果说明旧 release 的 Thin LTO 已经移除了大部分未调用 proof 符号：feature 拆分
首先保证生产 API、依赖边界和运行时分配正确，并不会虚构二进制下降。新增的紧凑
字形/行几何、布局 LRU 和缓存统计抵消了这部分链接裁剪。后续二进制优化应处理
Harfrust/Swash 的字体解析代码代际重复，不能通过删掉正确的塑形或回退能力实现。

`windows-directwrite` 使用系统 DirectWrite 做塑形、字体回退和字形栅格化；
`windows-rust-text` 使用 ZSUI 自有 Harfrust/Swash 候选管线。两者都把结果合成到
既有 Win32 缓冲 DIB，GDI 仍是可裁剪基线。以下为同一台机器、同一份 1000×700
Minimal release 应用交替运行 5 次的实测，暖态结果排除每种后端的首次运行，预热
3 秒并采样 6 次。

| 文字后端 | 二进制 | 暖态 RSS | Private RSS | Private Bytes | 峰值 RSS |
| --- | ---: | ---: | ---: | ---: | ---: |
| GDI | 658.0 KiB | 14.20 MiB | 1.47 MiB | 2.18 MiB | 16.87 MiB |
| DirectWrite | 670.5 KiB | 15.17 MiB | 1.62 MiB | 2.24 MiB | 17.85 MiB |
| ZSUI Rust text | 2,258.5 KiB | 15.12 MiB | 1.78 MiB | 2.42 MiB | 17.79 MiB |
| DirectWrite 相对 GDI | +12.5 KiB | +0.97 MiB | +0.15 MiB | +0.06 MiB | +0.98 MiB |
| Rust text 相对 GDI | +1,600.5 KiB | +0.92 MiB | +0.31 MiB | +0.24 MiB | +0.92 MiB |

两个高级文字后端的暖态 RSS 增量都约为 1 MiB，不能全部视为应用独占内存。
Rust text 的应用独占增量更高，但本次 Minimal 实测仍为 0.31 MiB Private RSS 和
0.24 MiB Private Bytes；主要成本是 release 二进制增加约 1.56 MiB。模块检查确认
只有 DirectWrite 版本加载 `dwrite.dll`；三个版本均未加载 `d2d1.dll` 或 WebView。
未启用对应特性时不会构建其字体系统、布局、字形缓存或复用位图表面。Rust text
仍需通过 `docs/text-rendering.md` 的完整多字体矩阵，不能仅凭内存结果替代正确性
验收。

### Minimal

| 框架 | 二进制 | 首次帧 | 暖启动首帧 | 空窗口 RSS | 页面 RSS | 隐藏 RSS | 峰值 RSS | Private RSS | 空闲 CPU | 重绘 CPU |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ZSUI | 0.65 MiB | 128 ms | 52 ms | 10.52 MiB | 11.98 MiB | 11.99 MiB | 14.68 MiB | 1.46 MiB | 0% | 0.340% |
| eframe/egui | 5.53 MiB | 308 ms | 267 ms | 98.82 MiB | 97.32 MiB | 97.32 MiB | 111.28 MiB | 68.02 MiB | 0% | 5.244% |
| Iced | 3.97 MiB | 296 ms | 96 ms | 18.84 MiB | 19.76 MiB | 19.78 MiB | 19.87 MiB | 5.97 MiB | 0% | 0.677% |
| Slint | 7.58 MiB | 77 ms | 128 ms | 20.33 MiB | 22.25 MiB | 22.30 MiB | 22.30 MiB | 5.19 MiB | 0.082% | 0.135% |
| Tauri 2 / WebView2 | 2.65 MiB* | 531 ms | 447 ms | 332.11 MiB | 335.74 MiB | 345.15 MiB | 351.35 MiB | 77.90 MiB | 0% | 1.005% |

### Common

| 框架 | 二进制 | 首次帧 | 暖启动首帧 | 空窗口 RSS | 页面 RSS | 隐藏 RSS | 峰值 RSS | Private RSS | 空闲 CPU | 重绘 CPU |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ZSUI | 0.74 MiB | 181 ms | 80 ms | 14.19 MiB | 12.13 MiB | 12.14 MiB | 17.24 MiB | 1.57 MiB | 0.028% | 1.682% |
| eframe/egui | 5.54 MiB | 582 ms | 258 ms | 97.36 MiB | 98.62 MiB | 103.74 MiB | 112.02 MiB | 69.24 MiB | 0% | 3.012% |
| Iced | 3.98 MiB | 723 ms | 210 ms | 18.84 MiB | 21.39 MiB | 21.39 MiB | 21.39 MiB | 6.87 MiB | 0% | 0.369% |
| Slint | 7.65 MiB | 852 ms | 75 ms | 20.38 MiB | 23.63 MiB | 23.64 MiB | 23.72 MiB | 5.53 MiB | 0.029% | 0.255% |
| Tauri 2 / WebView2 | 2.65 MiB* | 507 ms | 758 ms | 336.11 MiB | 344.54 MiB | 353.59 MiB | 361.13 MiB | 80.48 MiB | 0% | 1.416% |

### Full Native App

| 框架 | 二进制 | 首次帧 | 暖启动首帧 | 空窗口 RSS | 页面 RSS | 隐藏 RSS | 峰值 RSS | Private RSS | 空闲 CPU | 重绘 CPU |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ZSUI | 1.27 MiB | 77 ms | 65 ms | 10.88 MiB | 18.23 MiB | 15.71 MiB | 18.38 MiB | 4.94 MiB | 0.227% | 0.028% |
| eframe/egui | 5.63 MiB | 594 ms | 553 ms | 96.94 MiB | 100.32 MiB | 100.45 MiB | 112.35 MiB | 70.83 MiB | 0% | 6.726% |
| Iced | 4.09 MiB | 115 ms | 101 ms | 18.90 MiB | 21.00 MiB | 21.00 MiB | 21.00 MiB | 6.72 MiB | 0% | 0.456% |
| Slint | 9.84 MiB | 118 ms | 70 ms | 21.07 MiB | 24.69 MiB | 24.69 MiB | 24.70 MiB | 5.89 MiB | 0% | 0.228% |
| Tauri 2 / WebView2 | 2.65 MiB* | 899 ms | 421 ms | 346.14 MiB | 347.27 MiB | 356.47 MiB | 363.90 MiB | 82.31 MiB | 0.051% | 0.973% |

### Viewer

| 框架 | 二进制 | 首次帧 | 暖启动首帧 | 空窗口 RSS | 页面 RSS | 隐藏 RSS | 峰值 RSS | Private RSS | 空闲 CPU | 重绘 CPU |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ZSUI | 1.33 MiB | 82 ms | 82 ms | 10.97 MiB | 15.91 MiB | 15.50 MiB | 18.17 MiB | 4.73 MiB | 0.199% | 2.502% |
| eframe/egui | 5.61 MiB | 303 ms | 448 ms | 98.77 MiB | 97.86 MiB | 103.38 MiB | 112.26 MiB | 68.46 MiB | 0% | 5.061% |
| Iced | 4.08 MiB | 127 ms | 104 ms | 18.91 MiB | 20.55 MiB | 20.55 MiB | 20.98 MiB | 6.36 MiB | 0.057% | 0.340% |
| Slint | 9.79 MiB | 113 ms | 72 ms | 20.95 MiB | 24.20 MiB | 24.21 MiB | 24.21 MiB | 5.69 MiB | 0% | 0.254% |
| Tauri 2 / WebView2 | 2.65 MiB* | 449 ms | 400 ms | 337.45 MiB | 348.10 MiB | 357.31 MiB | 365.14 MiB | 84.88 MiB | 0.194% | 0.873% |

Windows 没有提供与 Linux `/proc/<pid>/smaps` 等价的 PSS 计数，因此本轮 PSS
明确记为不可用，不以 RSS 或 Private RSS 冒充。Private RSS 使用 Windows 私有
工作集。Tauri 的 `*` 表示二进制大小不包含系统安装的 WebView2 运行时，内存则
包含完整 WebView2 进程树。

## 结论边界

- ZSUI 正式应用没有因 Viewer 或 UiDocument 开发工具被迫增重：Common 页面 RSS
  为 12.13 MiB，Full Native App 为 18.23 MiB；独立 Viewer 为 15.91 MiB。
- 同档 Full Native App 中，ZSUI 页面 RSS 为 18.23 MiB，Iced 为 21.00 MiB，
  Slint 为 24.69 MiB，egui 为 100.32 MiB，Tauri/WebView2 为 347.27 MiB。
- 单次 Windows CPU 和启动数字会受调度、驱动缓存和后台活动影响，适合发现量级
  回归，不应被解释为跨机器的绝对排名。
- 视觉复杂度、窗口尺寸、语言、主题、数据和动画状态必须固定；不同档位的数据不
  能交叉比较。

## 复现

```powershell
.\scripts\measure-ui-performance-matrix.ps1 `
  -MemorySamples 6 `
  -StartupRuns 5 `
  -WarmupSeconds 3 `
  -CpuSampleSeconds 3
```

脚本构建 20 个独立 release 产物，递归采样进程树，并为每个页面保存截图。构建、
截图和 JSON 报告默认写入 Git 工作区外的 `zsui-ui-benchmark-support`，避免把生成
文件或对照框架运行时带入 ZSUI 包。
