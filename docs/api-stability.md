# ZSUI 0.2 API stability / API 稳定性

## Stable surface / 稳定接口

`zsui::stable` and `zsui::prelude` are the supported application-authoring
surface for the complete 0.2 release line. They provide an opaque retained
`Element<Message>`, typed state/update loop, logical layout units and native
window entry point without exposing renderer objects or platform handles.

`zsui::stable` 与 `zsui::prelude` 是整个 0.2 版本线受支持的应用开发接口。
它们提供不透明的 `Element<Message>`、强类型状态更新循环、逻辑布局单位和
原生窗口入口，不向应用暴露渲染对象或平台句柄。

Within `0.2.x`, ZSUI will not remove a stable item, change an existing stable
signature, weaken its safety contract, or change the meaning of a serialized
stable value. New items and new opt-in Cargo features may be added. A required
breaking change must move to the next minor release and include migration notes.

在 `0.2.x` 内，ZSUI 不删除稳定接口、不改变既有稳定签名、不削弱安全约束，
也不改变稳定序列化值的语义。允许增加新接口和新的可选 Cargo feature。
必须发生的破坏性修改只能进入下一个次版本，并提供迁移说明。

## Compatibility surface / 兼容接口

The historical crate-root modules and flattened re-exports remain callable for
source compatibility and for backend development, but are hidden from the
stable Rustdoc. They are not part of the 0.2 semver promise. Applications that
need long-lived source compatibility should import from `zsui::prelude` or
`zsui::stable` only.

历史根模块和扁平重导出继续保留，以兼容现有源码和支持后端开发，但不会进入
稳定 Rustdoc，也不属于 0.2 的语义版本承诺。需要长期源码兼容的应用应只从
`zsui::prelude` 或 `zsui::stable` 导入。

## Enforcement / 自动约束

- Stable Rustdoc is generated with every Cargo feature enabled.
- `scripts/check-rustdoc-coverage.ps1` rejects coverage below 70%.
- The stable module denies missing documentation at compile time.
- After the `v0.2.0` baseline tag exists, CI runs `cargo-semver-checks` against
  that tag and rejects incompatible changes to documented public APIs.
- Platform handles, renderer plans, proof drivers and backend adapters remain
  outside the stable authoring surface.

- 稳定 Rustdoc 使用全部 Cargo feature 生成。
- `scripts/check-rustdoc-coverage.ps1` 会拒绝低于 70% 的覆盖率。
- 稳定模块在编译期禁止缺少文档的公开项。
- `v0.2.0` 基准标签建立后，CI 使用 `cargo-semver-checks` 与该标签比较，拒绝
  对已公开文档接口的不兼容修改。
- 平台句柄、渲染计划、证明驱动器和后端适配器不进入稳定开发接口。
