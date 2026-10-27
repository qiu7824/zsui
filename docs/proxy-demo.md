# ZSUI Proxy Client Demo

`examples/zsui_proxy_client.rs` is a VLESS proxy client interface built with
the ordinary `native_window(...).stateful_view(...)` application shape. It is
a repaint workload for memory and flicker comparisons, not a proxy: it parses
`vless://` share links and simulates traffic, and it never opens a network
connection.

```sh
cargo run --locked --example zsui_proxy_client --no-default-features \
  --features window,button,label,textbox,toggle,list
```

## Importing a node

Paste a share link into the text box and press Enter or 导入. A link can also
be supplied at startup:

```sh
cargo run --example zsui_proxy_client --no-default-features \
  --features window,button,label,textbox,toggle,list -- --import "vless://..."
```

`ZSUI_PROXY_DEMO_LINK` is read when `--import` is absent. Links contain the
server credentials, so the demo keeps them in memory only, clears the text box
after a successful import and masks the user ID and public key in the details
panel. Do not commit real links to this repository; `--demo-node` loads a
placeholder node on a TEST-NET address for screenshots and measurements.

The parser understands the parameters used by Reality and TLS share links:
`encryption`, `flow`, `security`, `sni`, `fp`, `pbk`, `sid`, `spx`, `type` and
`headerType`, percent-encoded names and bracketed IPv6 hosts.

## Simulated traffic

Switching the connection on starts a worker thread that updates rate and total
counters and wakes the window through `UiInvalidationHandle`. It stops as soon
as the connection is switched off, so an idle window does not poll.

| Flag | Effect |
| --- | --- |
| `--demo-node` | Import the placeholder node |
| `--connect` | Start the simulated connection immediately |
| `--tick-ms N` | Update interval, 500 ms by default |
| `--ticks N` | Stop after N updates and use simulated time, so two builds render identical frames |
| `--width`, `--height` | Initial client size |
| `--always-on-top` | Keep the window above others for screen capture |
| `--tear-probe` | Frame the window with full-height bars that change color together on every update |
| `--measure FILE` | Run for `--duration-ms` (4000 by default), then write process memory as JSON |
| `--screenshot FILE` | With `--measure`, also save the framework's final-surface capture |

## Windows comparison

The `Proxy Demo Comparison` workflow builds this demo and `zsui_notepad`
(with `accessibility`) from `main` and from the pushed branch on Windows Server
2025, then runs `scripts/measure-proxy-demo-comparison.ps1`. It follows the
method in [memory-stability-plan.md](memory-stability-plan.md):

- private bytes, working set, peak working set and paint CPU of this demo at
  680×560 and 1000×720 while the connection repaints every 100 ms, in
  visible, hidden and restored phases. Ten alternating process pairs take 20
  samples per phase; each process median is one observation and intervals
  come from 10,000 paired bootstrap resamples. Growth per pixel of window area
  is reported for both memory measures;
- input, scroll and restore p95 latency of the notepad through
  `scripts/measure-windows-interaction-latency.ps1`, also as ten alternating
  pairs, against the `max(5%, 1 ms)` regression limit. That probe forces a full
  repaint per operation, which exercises the update-sized buffered path;
- pixel identity after scripted partial updates in this demo, and after
  editing, scrolling, a modal dialog with scrim and shadow, its dismissal,
  sixteen rapid resizes and occlusion in the notepad. Captures only paint the
  already invalid region, so missed damage stays visible;
- tearing with `--tear-probe`: a frame in which either bar shows two colors
  was presented while only part of an update had reached it;
- erase flicker, measured as the share of each frame that shows only the page
  background. The same probe runs against
  `scripts/proxy-demo-flicker-control.ps1`, an unbuffered Windows Forms
  window, so a clean result is shown to be a measurement and not a blind spot.

Tearing and flicker are captured both from the composed desktop and through
client-only `PrintWindow`. Desktop sampling waits for DWM and uses physical
pixels. Off-screen capture adopts the target window's DPI-awareness context;
for a DPI-unaware application this is a logical bitmap, not a native-resolution
high-DPI surface. Reports record window DPI awareness and DPI separately.
The workflow repeats everything at 100% and, at 1920×1080, at
150% and 175% scale, set on the running desktop by
`scripts/set-windows-display-scale.ps1` the way the Settings app does. That is
the largest mode the hosted display adapter offers, and Windows offers at most
175% for it, so 200% needs a larger physical or virtual display. Window sizes
are given in DIPs and scale with the display; a captured window that would no
longer fit the work area shrinks evenly, the same for both builds, and memory
rows report the client size the windows received. Above
100% the demo and the notepad also switch the live desktop to 100% and back,
which sends the same
`WM_DPICHANGED` as moving to a monitor with another DPI, and compare pixels
after each switch. When a second monitor is present the notepad is moved to
it and back; hosted runners have one monitor, so run the script locally for
physical multi-monitor evidence. Compositor first-frame time and long-running
loops are not covered. Each scale uploads its summary, per-process data and
screenshots as `proxy-demo-comparison-windows-2025-<scale>`.
