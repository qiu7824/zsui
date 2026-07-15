use zsui::{ZsLocale, ZsLocalizer, ZsMessageArgs, ZsuiResult};

const EN: &str = r#"
window-title = ZSUI localization
greeting = Welcome, { $name }.
item-count = { $count ->
    [one] One item
   *[other] { $count } items
}
"#;

const ZH_CN: &str = r#"
window-title = ZSUI 多语言
greeting = 欢迎，{ $name }。
item-count = { $count } 个项目
"#;

fn main() -> ZsuiResult<()> {
    let fallback = ZsLocale::parse("en")?;
    let mut localizer = ZsLocalizer::for_system(fallback);
    localizer.add_ftl(ZsLocale::parse("en")?, EN)?;
    localizer.add_ftl(ZsLocale::parse("zh-CN")?, ZH_CN)?;

    let greeting = localizer.format(
        "greeting",
        &ZsMessageArgs::new().with("name", "ZSUI"),
        "Welcome, ZSUI.",
    );
    let item_count = localizer.format(
        "item-count",
        &ZsMessageArgs::new().with("count", 3u32),
        "3 items",
    );

    println!("{}", localizer.text("window-title", "ZSUI"));
    println!("{greeting}");
    println!("{item_count}");
    Ok(())
}
