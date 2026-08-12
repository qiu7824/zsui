# Distribution, signing and updates / 安装、签名与更新

ZSUI's release pipeline distributes the component gallery as a portable binary
and native platform package. Framework applications can reuse the same files
under `packaging/` without linking packaging or update code into their runtime.

ZSUI 发布流水线同时提供便携程序与平台原生安装包。框架应用可复用
`packaging/` 中的文件，安装和更新代码不会链接进应用运行时。

| Platform | Portable artifact | Installer | Native trust |
| --- | --- | --- | --- |
| Windows | GUI-subsystem `.exe` | Inno Setup per-user installer | Authenticode signs both application and installer when certificate secrets are configured |
| macOS | `.app` inside the image | Read-only `.dmg` | Developer ID signing and Apple notarization when Apple credentials are configured |
| Linux | `.tar.gz` | Debian `.deb` with desktop entry | Exact artifact identity is protected by GitHub/Sigstore build provenance |

Every installable release artifact, `update-manifest.json` and `SHA256SUMS`
receives a GitHub artifact attestation. GitHub signs that provenance through
Sigstore and binds the digest to this repository, workflow, commit and tag.
Native Windows/macOS signatures are a second layer; their secrets are never
stored in the repository.

所有可安装发布附件、`update-manifest.json` 与 `SHA256SUMS` 都生成 GitHub 构建
证明。GitHub 通过 Sigstore 对证明签名，并将摘要绑定到本仓库、工作流、提交和
标签。Windows 与 macOS 原生签名属于第二层；证书机密不会写入仓库。

## Required signing secrets / 签名机密

- Windows: `WINDOWS_CERTIFICATE_BASE64`, `WINDOWS_CERTIFICATE_PASSWORD`.
- macOS signing: `APPLE_CERTIFICATE_BASE64`,
  `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`.
- macOS notarization: `APPLE_NOTARY_KEY_BASE64`, `APPLE_NOTARY_KEY_ID`,
  `APPLE_NOTARY_ISSUER_ID`.

If the native certificate set is absent, the pipeline still emits reproducible
packages with GitHub/Sigstore provenance, but reports them as not OS-signed.
A product that requires SmartScreen or Gatekeeper trust must configure the
corresponding secrets before marking its release stable.

缺少平台证书时，流水线仍生成带 GitHub/Sigstore 证明的可复现安装包，但会明确
标记为未进行系统代码签名。需要 SmartScreen 或 Gatekeeper 信任的产品必须先配置
相应机密，才能把版本标记为稳定发布。

## Automatic update protocol / 自动更新协议

`scripts/new-update-manifest.ps1` produces a versioned manifest containing only
HTTPS release URLs, exact byte lengths, SHA-256 digests, platform, architecture
and package kind. An updater must apply these checks in order:

1. Accept only `zsui.update-manifest/v1` downloaded from the configured HTTPS
   repository and reject redirects to another origin.
2. Compare semantic versions and reject downgrade unless the application has
   explicitly entered a rollback recovery flow.
3. Select one exact platform/architecture target and download to a temporary
   file on the same volume as the installation.
4. Verify byte length and SHA-256, then verify the GitHub artifact attestation
   for repository `qiu7824/zsui`, the release tag and expected workflow.
5. On Windows verify Authenticode publisher identity; on macOS verify Developer
   ID, notarization and stapled ticket. Never weaken these checks after retry.
6. Close the application, invoke the native installer, and replace files
   transactionally. Preserve the previous version until the new application
   completes a startup health check; otherwise roll back.
7. Record no document content, clipboard data, preedit text or credentials in
   update logs.

自动更新器必须按以上顺序验证协议版本、来源、版本单调性、目标平台、文件长度、
SHA-256、GitHub 构建证明和平台代码签名，再通过系统安装器事务更新。旧版本保留到
新版本完成启动健康检查；失败时回滚。更新日志不得包含文档、剪贴板、预编辑文本
或凭据。

The updater is deliberately a product companion rather than a mandatory ZSUI
feature. This keeps framework-only applications free from HTTP, archive,
signature and background-service dependencies, so unused update support adds
zero executable size and zero runtime memory.
