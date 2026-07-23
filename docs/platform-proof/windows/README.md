# Windows platform proof

Backend: `windows-win32`

This directory is incomplete until it contains the five required PNG files
and `interaction-report.json` defined by the parent manifest. Local smoke
reports are diagnostic evidence and do not by themselves mark the backend
complete.

Current diagnostic artifacts:

- `startup.png`: real Win32 launch capture from the shared showcase.
- `dark-theme.png`: shared theme tokens repainted through the buffered renderer.
- `text-input.png`: owner-drawn text input, focus traversal, toggle and scroll smoke capture.
- `shaped-text.png`: self-drawn notepad smoke with proportional Latin, Hebrew
  bidirectional text and CJK on the real Win32 buffered renderer.
- `notepad-interaction.png` and `.json`: a 960×640 run of the same scripted
  input, selection, scrolling and unsaved-close scenario used by AppKit and
  Linux. The PNG comes from the Win32 buffered surface; the JSON records the
  messages, focus, geometry, process memory and runtime errors. Its typography
  evidence records the live Windows message font used by every semantic UI
  text role, while role-specific size, line height and weight remain owned by
  the framework.
- `bidi-navigation-smoke-report.json`: real Win32 notepad key routing over
  `abאב`; four Right keys move through relative scalar carets `1, 4, 3, 2`,
  proving visual rather than logical traversal of the shaped primary positions.
- `showcase-smoke-report.json`: machine-readable smoke counters for that interaction run.

The report confirms one attached native menu with five commands and exercises
typed menu-command routing. Native file dialogs, a visible menu capture and the
full manual interaction report are still required.
