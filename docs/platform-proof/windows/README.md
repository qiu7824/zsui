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
- `showcase-smoke-report.json`: machine-readable smoke counters for that interaction run.

The report confirms one attached native menu with five commands and exercises
typed menu-command routing. Native file dialogs, a visible menu capture and the
full manual interaction report are still required.
