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

预构建原生 Viewer、稳定 ID 状态迁移、文件监听、三平台原生重载、AI 交接包和
发布期文档嵌入仍是后续 v0.2 切片。浏览器投影不能替代原生运行证据。
