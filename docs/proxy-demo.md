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
| `--measure FILE` | Run for `--duration-ms` (4000 by default), then write process memory as JSON |
| `--screenshot FILE` | With `--measure`, also save the framework's final-surface capture |

## Windows comparison

The `Proxy Demo Comparison` workflow builds this demo against `main` and the
pushed branch on Windows Server 2025, then runs
`scripts/measure-proxy-demo-comparison.ps1`:

- working set, private bytes and peak working set at 680×560 and 1000×720
  while the connection repaints every 100 ms, and how much each grows per
  pixel of window area;
- pixel identity of the first frame and of the frame left after 40 scripted
  partial updates;
- on-screen flicker while repainting every 30 ms, captured both from the
  composed desktop and through `PrintWindow`, together with paint CPU time.

Flicker is measured as the share of each captured frame that shows only the
page background; erase-then-draw painting makes that share jump. The same probe
runs against `scripts/proxy-demo-flicker-control.ps1`, an unbuffered Windows
Forms window, so a clean ZSUI result is shown to be a measurement and not a
blind spot. The summary and screenshots are uploaded as the
`proxy-demo-comparison-windows-2025` artifact.
