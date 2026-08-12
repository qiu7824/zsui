# Assistive technology release checklist / 辅助技术发布检查表

Release candidate: __________  Commit: __________  Tester/date: __________

## Windows 11

- [ ] Microsoft Pinyin and one additional IME show the candidate window at the active caret.
- [ ] Preedit replacement, commit, cancel, selection replacement and deletion preserve text.
- [ ] Narrator reads window, menu, tab, field, selection, dialog and validation-state labels in visual order.
- [ ] NVDA focus and browse navigation reach the same controls and invoke actions once.
- [ ] Focus returns to the initiating control after dialogs, menus and overlays close.

## macOS 15

- [ ] Pinyin and one additional input source position the candidate window at the active caret.
- [ ] Marked ranges, commit, cancel, selection replacement and deletion preserve text.
- [ ] VoiceOver reads window, toolbar, tab group, text field, selection, dialog and status in visual order.
- [ ] VoiceOver actions invoke each semantic control once and focus remains visible.
- [ ] Focus returns to the initiating control after sheets, menus and popovers close.

## Ubuntu 24.04

- [ ] IBus or Fcitx5 Pinyin positions the candidate window at the active caret on Wayland and X11.
- [ ] Preedit replacement, commit, cancel, selection replacement and deletion preserve text.
- [ ] Orca reads application, menu, tab, text field, selection, dialog and live status in visual order.
- [ ] Orca actions invoke each AT-SPI control once and focus remains visible.
- [ ] Focus returns to the initiating control after dialogs, menus and popovers close.

## Shared text and appearance

- [ ] Chinese, Latin, Arabic/Hebrew, emoji ZWJ and combining-mark navigation follows grapheme boundaries.
- [ ] High contrast/increased contrast preserves focus, selection, disabled and validation states.
- [ ] 100%, 150% and 200% scale preserve caret geometry and prevent clipping or overlap.
- [ ] No password, preedit or clipboard payload appears in logs or proof JSON.

Result: [ ] Pass  [ ] Fail

Evidence paths or issue links: ________________________________________________
