# IME and screen-reader matrix / 输入法与屏幕阅读器矩阵

ZSUI separates repeatable native-protocol automation from the final human
experience check. Automated proof must pass on every UI-changing commit; the
real candidate-window and spoken-output checklist must pass for a release.

ZSUI 将可重复的原生协议自动验收与最终人工体验验收分开。每次 UI 修改必须通过
自动证明；正式发布还必须通过真实候选窗和朗读输出检查。

| Platform | Automated IME proof | Automated accessibility client | Release reader |
| --- | --- | --- | --- |
| Windows | Real Win32 window routes IMM32 preedit, commit, cancel and candidate/caret geometry | An external `System.Windows.Automation` client reads the fragment tree and exercises `ValuePattern` and `TextPattern` | Narrator and NVDA |
| macOS | Real `NSApplication`/`NSView` exercises `NSTextInputClient`, marked text, commit, cancel and character-range rectangle | AppKit runtime queries the `NSAccessibility` tree, roles, values, focus and actions | VoiceOver |
| Linux | Real Wayland surface exercises preedit, commit, cancel and cursor area | An external `python3-pyatspi` client reads and activates the AccessKit AT-SPI tree | Orca |

The automated proof intentionally does not claim to validate the operating
system's visible candidate panel or the words a screen reader speaks. Those
depend on installed input methods, voices, user preferences and interactive
desktop permissions. The required release evidence is recorded with
`manual-assistive-technology-checklist.md`.

自动证明不会冒充对系统候选窗可见效果或屏幕阅读器实际朗读内容的验证。这些行为
取决于输入法、语音、用户设置与交互桌面权限。正式发布必须使用
`manual-assistive-technology-checklist.md` 留存人工验收结果。

The machine-readable source of truth is
`tests/quality/ime-screen-reader-matrix.json`; CI validates its workflows,
probe paths and required event coverage through
`scripts/check-ime-screen-reader-matrix.ps1`.
