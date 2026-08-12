# ARM64 native runtime proof / ARM64 原生运行证明

ZSUI uses GitHub's native standard ARM64 runners, not cross-compilation or
emulation, for Windows and Linux release evidence.

ZSUI 使用 GitHub 原生标准 ARM64 Runner，而不是交叉编译或模拟器，生成
Windows 与 Linux 发布证据。

| Target | Fixed runner | Required runtime evidence |
| --- | --- | --- |
| Windows ARM64 | `windows-11-arm` | ARM64 OS and Rust host, PE machine `0xAA64`, real Win32 window, typed input and IME path, UIA tree, final `WM_PRINTCLIENT` PNG |
| Linux ARM64 | `ubuntu-24.04-arm` | `aarch64` kernel and Rust host, ELF AArch64 binary, real X11 window under Xvfb, typed input and IME path, AccessKit/AT-SPI tree, final Softbuffer/Pango/Cairo PNG |

The workflow rejects a build-only result. Each process must create a native
window, complete its first frame, route text and IME messages, export a final
surface PNG, produce structured runtime evidence and exit without errors.

工作流拒绝仅编译结果。每个平台进程必须创建原生窗口、完成首帧、路由文本和
IME 消息、导出最终表面 PNG、生成结构化运行证据，并且无错误退出。

Artifacts are uploaded for 30 days by
`.github/workflows/arm64-native-proof.yml`. ARM64 is considered proven only
after both jobs pass on the commit being released.
