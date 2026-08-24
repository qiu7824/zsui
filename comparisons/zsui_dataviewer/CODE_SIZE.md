# DataViewer Source Size Baseline

Baseline date: 2026-08-09. The reference is
`kusutori/DataViewer@d6af795331ff5012e6273ca17461e9696fa51461`.
The ZSUI side is the source under this comparison package.

## Production source

| Area | Reference files | Reference physical | Reference nonblank | ZSUI files | ZSUI physical | ZSUI nonblank |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| UI and view composition | 3 | 1,010 | 918 | 1 | 270 | 259 |
| State and messages | 3 | 154 | 144 | 1 | 254 | 234 |
| Data loading and query | 10 | 237 | 186 | 1 | 280 | 254 |
| Platform/application integration | 0 | 0 | 0 | 3 | 284 | 262 |
| **Total** | **16** | **1,401** | **1,248** | **6** | **1,088** | **1,009** |

The ZSUI implementation is 313 physical lines (22.3%) and 239 nonblank lines
(19.2%) smaller in production source under this classification. Its explicit
platform/application row contains startup, menu, native file dialogs,
clipboard, export and asynchronous effects that are distributed across the
reference rows.

## Tests and total authored Rust

The reference commit has no executable test source. ZSUI has three embedded
test modules containing 9 tests: 116 physical and 104 nonblank lines.

| Scope | Reference physical | Reference nonblank | ZSUI physical | ZSUI nonblank |
| --- | ---: | ---: | ---: | ---: |
| Production | 1,401 | 1,248 | 1,088 | 1,009 |
| Tests | 0 | 0 | 116 | 104 |
| **Authored total** | **1,401** | **1,248** | **1,204** | **1,113** |

Including tests, ZSUI is 197 physical lines (14.1%) and 135 nonblank lines
(10.8%) smaller.

## Counting rules

- Count UTF-8 text lines once; a nonblank line contains at least one non-space
  character.
- Reference production includes `App.cs`, `Controls/*.cs`, `Data/*.cs`,
  `Services/*.cs` and `State/*.cs`.
- ZSUI production includes `src/*.rs` before each terminal `#[cfg(test)]`
  module. Those test modules are counted separately.
- Exclude comments only from neither metric: physical/nonblank counting avoids
  subjective semantic-line normalization.
- Exclude lockfiles, manifests, build files, fixtures, generated output,
  vendored dependencies and capture artifacts.
- Source size is one engineering signal. Feature coverage, native behavior,
  package size, startup, memory, latency and visual evidence remain separate
  comparison dimensions.
