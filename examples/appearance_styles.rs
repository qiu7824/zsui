//! One standard control tree with native, violet and compact green appearances.
use zsui::prelude::*;

#[derive(Clone)]
enum Message {
    Theme(usize),
    Value(String),
    Tab(TabId),
    Toggle(bool),
    Save,
}
struct State {
    theme: usize,
    value: String,
    tab: TabId,
    enabled: bool,
    saved: bool,
}

fn theme(index: usize) -> AppTheme {
    if index == 0 {
        return AppTheme::default();
    }
    let accent = if index == 1 {
        Color::rgb(120, 65, 185)
    } else {
        Color::rgb(18, 126, 92)
    };
    AppTheme::new(
        Appearance::default()
            .color(ColorRole::Accent, accent)
            .color(ColorRole::Control, Color::rgb(248, 247, 250))
            .font(
                FontSpec::family("Microsoft YaHei UI")
                    .fallback("Noto Sans CJK SC")
                    .fallback("Segoe UI Emoji")
                    .size(Dp::new(14.0))
                    .line_height(Dp::new(22.0)),
            )
            .focus(FocusAppearance::Outline)
            .tabs(if index == 1 {
                TabAppearance::Pill
            } else {
                TabAppearance::Segmented
            })
            .density(if index == 1 {
                ControlDensity::Comfortable
            } else {
                ControlDensity::Compact
            }),
    )
}
fn view(state: &State) -> Element<Message> {
    let actions = row([
        button("Native").on_click(Message::Theme(0)),
        button("Violet").on_click(Message::Theme(1)),
        button("Green").on_click(Message::Theme(2)),
    ])
    .gap(Dp::new(8.0));
    let form = column([
        text("Display and audio / 显示与音频"),
        row([icon(Icon::Monitor), text("Display name").flex(1.0)]).gap(Dp::new(8.0)),
        textbox(&state.value)
            .on_change(Message::Value)
            .invalid(state.value.is_empty())
            .appearance(Appearance::default().states(StateStyles {
                invalid: ControlStyle::default().border(Color::rgb(180, 35, 50), Dp::new(2.0)),
                ..Default::default()
            })),
        row([
            icon(Icon::Mic),
            text("Audio enabled").flex(1.0),
            toggle(state.enabled).on_toggle(Message::Toggle),
        ])
        .gap(Dp::new(8.0)),
        button("Save")
            .on_click(Message::Save)
            .appearance(Appearance::default().states(StateStyles {
                normal: ControlStyle::default().radius(Dp::new(8.0)),
                hover: ControlStyle::default().background(Color::rgb(226, 218, 241)),
                pressed: ControlStyle::default().background(Color::rgb(200, 187, 225)),
                ..Default::default()
            })),
        text(if state.saved {
            "Saved / 已保存"
        } else {
            "Ready / 就绪"
        }),
    ])
    .gap(Dp::new(12.0))
    .padding(Dp::new(16.0));
    column([
        actions,
        tabs(
            [
                Tab::new(TabId::new(1), "Settings", form),
                Tab::new(
                    TabId::new(2),
                    "Details",
                    text("Shared keyboard and accessibility behavior"),
                ),
            ],
            Some(state.tab),
        )
        .on_tab_select(Message::Tab)
        .flex(1.0),
    ])
    .gap(Dp::new(16.0))
    .padding(Dp::new(20.0))
    .theme(theme(state.theme))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    window("Appearance styles")
        .size(640, 480)
        .stateful(
            State {
                theme: 0,
                value: "Main display / 主显示器".into(),
                tab: TabId::new(1),
                enabled: true,
                saved: false,
            },
            view,
            |state, message, _| match message {
                Message::Theme(theme) => state.theme = theme,
                Message::Value(value) => {
                    state.value = value;
                    state.saved = false;
                }
                Message::Tab(tab) => state.tab = tab,
                Message::Toggle(value) => state.enabled = value,
                Message::Save => state.saved = true,
            },
        )
        .run()?;
    Ok(())
}
