# Linux platform proof

Backend: `linux-direct`

The Ubuntu CI job launches a real X11 window under Xvfb, presents the
Cairo/Pango software surface and emits PNG plus structured proof JSON. The
separate real Weston Wayland, AT-SPI and menu proof is also operational. Final
completion still requires the full artifact set and reviewed regression
baselines defined by the parent manifest.

Reviewed documentation evidence:

- `gallery-inputs-light.png` and `.json`: the shared Gallery scene captured from
  the final X11 Softbuffer surface with Cairo/Pango.
- `notepad-interaction.png` and `.json`: the shared Notepad interaction captured
  from the same default Linux profile.
- `notepad-interaction-lite.png` and `.json`: the unchanged Notepad source using
  the optional cosmic-text/swash, tiny-skia and Softbuffer profile.

The workflow also launches a real Weston Wayland compositor with `DISPLAY`
unset, verifies the display handle, exercises an external AT-SPI client and
probes the native menu surface. Those per-run artifacts remain in CI rather than
being duplicated in this small documentation set.
