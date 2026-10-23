# ZSUI Native Proof Comparator

Standalone comparator for ZSUI 0.3 native proof artifacts. It is intentionally
kept outside the ZSUI runtime dependency graph.

## Commands

```bash
# Compare generated actual evidence against a read-only baseline.
cargo run --locked \
  --manifest-path tools/native-proof/Cargo.toml \
  -- compare \
  --baseline tests/native-baselines/macos-15-arm64 \
  --actual target/native-proof/macos-15-arm64 \
  --diff target/native-proof-diff/macos-15-arm64

# Promote a reviewed actual run into a new baseline directory.
cargo run --locked \
  --manifest-path tools/native-proof/Cargo.toml \
  -- init \
  --actual target/native-proof/macos-15-arm64 \
  --baseline tests/native-baselines/macos-15-arm64
```

## Comparison Rules

- Recursively discovers proof `*.json` files under `--actual`.
- Strictly compares semantic fields, widget order/geometry/focus, messages,
  unhandled commands and errors.
- Requires a PNG next to every JSON and validates it against the proof window
  pixel size.
- Compares full-window pixels and every widget critical region.
- Writes magenta-highlighted diff PNGs and `summary.md` under `--diff`.
- Returns a non-zero exit code when any scenario fails.

Thresholds default to the 0.3 policy: `0.5%` critical regions, `1.0%` full
window, channel tolerance `32`. Override with `--critical-threshold`,
`--full-threshold` and `--color-tolerance`.
