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
`radio_button`、`slider` 和 `progress_bar`。其他已存在的 ZSUI 组件会被识别为
“尚未进入 UiDocument schema”，不会被误报为未知组件。

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

每个 `UiNodeId` 都确定性映射到文档专用的 `WidgetId` 命名空间。Viewer 的属性值和
动作历史位于普通 `UiViewerState` 中，View 重建不会替换该状态。`button.click` 和
`radio_button.choose` 已走类型化 Viewer 消息；依赖带值回调的文本、切换和滑块动作
仍需先扩展共享 View 回调存储，不能借助全局注册表绕过类型系统。

原生证明可由同一可执行文件生成：

```powershell
cargo run --bin zsui-viewer `
  --no-default-features `
  --features ui-viewer `
  -- examples/ui-documents/basic.json `
  --values examples/ui-documents/basic.values.json `
  --smoke target/ui-viewer-proof
```

输出包含平台最终窗口截图 `window.png` 和带窗口、字体、内存、绘制及输入证据的
`proof.json`。完整组件覆盖、所有控件的焦点/选择/滚动状态迁移、确定性 AI 交接包、
三平台固定 Runner 基准和发布期文档嵌入仍是后续 v0.2 切片。浏览器投影不能替代
原生运行证据。
