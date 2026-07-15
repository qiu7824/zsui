# Localization

ZSUI provides an optional, application-owned localization service through the
`localization` Cargo feature. It uses Unicode language identifiers and Fluent
resources while keeping the default framework build small.

```toml
[dependencies]
zsui = { version = "0.2", features = ["localization"] }
```

## Resource model

Use stable semantic message identifiers in every locale. Source text is a
message value, not an identifier.

```text
# locales/en.ftl
action-save = Save
welcome-user = Welcome, { $name }.
item-count = { $count ->
    [one] One item
   *[other] { $count } items
}
```

```text
# locales/zh-CN.ftl
action-save = 保存
welcome-user = 欢迎，{ $name }。
item-count = { $count } 个项目
```

Stable identifiers allow source copy to change without invalidating every
translation. Fluent parameters and selectors cover dynamic values and plural
rules without concatenating translated fragments in application code.

## Application state

`ZsLocalizer` is owned by application state. It does not install a mutable
process-global catalog.

```rust
use zsui::{ZsLocale, ZsLocalizer, ZsMessageArgs, ZsuiResult};

fn localizer() -> ZsuiResult<ZsLocalizer> {
    let fallback = ZsLocale::parse("en")?;
    let mut localizer = ZsLocalizer::for_system(fallback);
    localizer.add_ftl(
        ZsLocale::parse("en")?,
        include_str!("../locales/en.ftl"),
    )?;
    localizer.add_ftl(
        ZsLocale::parse("zh-CN")?,
        include_str!("../locales/zh-CN.ftl"),
    )?;
    Ok(localizer)
}
```

Resolve text while building the View:

```rust
# use zsui::{button, ZsLocale, ZsLocalizer};
# fn view(localizer: &ZsLocalizer) {
let save = button::<()>(localizer.text("action-save", "Save"));
# let _ = save;
# }
```

Changing the active locale is an explicit state update. The following View
rebuild resolves all messages with the new catalog and naturally recomputes
intrinsic label widths:

```rust
# use zsui::{ZsLocale, ZsLocalizer, ZsuiResult};
# fn change(localizer: &mut ZsLocalizer) -> ZsuiResult<()> {
localizer.set_locale_tag("zh-CN")?;
# Ok(())
# }
```

For dynamic messages, pass named values:

```rust
# use zsui::{ZsLocalizer, ZsMessageArgs};
# fn count(localizer: &ZsLocalizer) {
let label = localizer.format(
    "item-count",
    &ZsMessageArgs::new().with("count", 12u32),
    "12 items",
);
# let _ = label;
# }
```

The lookup order is the active locale, its parent locales, the configured
fallback locale and its parents. For example, `zh-Hant-TW` falls back through
`zh-Hant` and `zh`. Call-site fallback text is used only when no catalog has the
message or formatting fails. `try_format` exposes catalog errors when an
application wants strict diagnostics.

## Direction and loading

`ZsLocale::direction()` and `ZsLocalizer::direction()` report left-to-right,
right-to-left or top-to-bottom writing direction. Fluent keeps interpolated
values inside Unicode direction-isolation marks, preventing user content from
changing the surrounding message direction.

Resources can be embedded with `include_str!` for deterministic packaging or
loaded from an application-owned path with `add_ftl_file`. Embedded fallback
resources are recommended so a missing external translation never produces an
empty UI.
