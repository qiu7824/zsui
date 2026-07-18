# Linux platform proof

Backend: `linux-direct`

The Ubuntu CI job launches a real X11 window under Xvfb, presents the
Cairo/Pango software surface and emits PNG plus structured proof JSON. Final
completion still requires the full artifact set from the parent manifest and a
separate real Wayland run.
