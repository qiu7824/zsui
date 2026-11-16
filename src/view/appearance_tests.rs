use super::*;
#[allow(unused_imports)]
use crate::{AppTheme, Appearance, Color, ControlStyle, FocusAppearance, FontSpec, StateStyles};

#[test]
#[cfg(all(feature = "button", feature = "custom-icons"))]
fn custom_button_icons_preserve_chrome_and_reserve_label_space() {
    let asset = crate::IconAsset::from_svg(crate::IconAssetId(81),
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><circle cx="10" cy="10" r="8"/></svg>"#,
        "Application-owned").unwrap();
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let dpi = Dpi::new(96.0 * scale);
        for size in [16.0, 18.0, 20.0, 24.0, 32.0] {
            for button in [crate::button::<()>("Save"), crate::primary_button("Save")] {
                let baseline = render(button.clone());
                let fill = baseline
                    .commands
                    .iter()
                    .find_map(|command| {
                        if let NativeDrawCommand::RoundRect { fill, .. } = command {
                            Some(*fill)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                let bounds = Rect {
                    x: 0,
                    y: 0,
                    width: Dp::new(220.0).to_px(dpi).round_i32(),
                    height: Dp::new(52.0).to_px(dpi).round_i32(),
                };
                let mut view = button
                    .icon_source(asset.clone())
                    .glyph_size(Dp::new(size))
                    .id(WidgetId(81));
                view.layout(&mut ViewLayoutCx::new(bounds, dpi));
                let mut cx = ViewPaintCx::new(dpi);
                view.paint(&mut cx);
                let plan = cx.into_plan();
                assert_eq!(view.bounds(), Some(bounds));
                assert!(plan.commands.iter().any(|command| matches!(command,
                    NativeDrawCommand::RoundRect { fill: actual, .. } if *actual == fill)));
                let icon = plan
                    .commands
                    .iter()
                    .find_map(|command| {
                        if let NativeDrawCommand::VectorIcon(icon) = command {
                            Some(icon.bounds)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                let label = plan
                    .commands
                    .iter()
                    .find_map(|command| {
                        if let NativeDrawCommand::Text(text) = command {
                            Some(text.bounds)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                assert_eq!(icon.width, Dp::new(size).to_px(dpi).round_i32());
                assert!(icon.x >= bounds.x && icon.y >= bounds.y);
                assert!(icon.y + icon.height <= bounds.y + bounds.height);
                assert!(
                    icon.x + icon.width <= label.x,
                    "artwork must leave the label unobstructed"
                );
            }
        }
    }
}

fn render(mut view: ViewNode<()>) -> NativeDrawPlan {
    view.layout(&mut ViewLayoutCx::new(
        Rect {
            x: 0,
            y: 0,
            width: 320,
            height: 180,
        },
        Dpi::standard(),
    ));
    let mut cx = ViewPaintCx::new(Dpi::standard());
    view.paint(&mut cx);
    cx.into_plan()
}

#[test]
#[cfg(all(feature = "button", feature = "label"))]
fn inherited_palette_and_instance_state_reach_standard_controls() {
    let accent = Color::rgb(120, 40, 160);
    let foreground = Color::rgb(245, 240, 250);
    let pressed = Color::rgb(40, 130, 60);
    let state = StateStyles {
        normal: ControlStyle::default()
            .background(accent)
            .foreground(foreground),
        pressed: ControlStyle::default().background(pressed),
        ..Default::default()
    };
    let id = WidgetId(101);
    let mut plan = render(
        column([
            crate::text("中文 Application"),
            crate::button("Save").id(id),
        ])
        .theme(AppTheme::new(
            Appearance::default().states(state).font(
                FontSpec::family("Example Primary")
                    .fallback("Example CJK")
                    .size(Dp::new(18.0)),
            ),
        )),
    );
    assert!(plan.commands.iter().any(|command|matches!(command,NativeDrawCommand::Text(text) if text.text=="Save"&&text.color==Some(foreground)&&text.font.as_ref().is_some_and(|font|font.families==["Example Primary","Example CJK"]))));
    plan.apply_appearance_states(Some(id), Some(id), Some(id), true, Dpi::standard());
    assert!(plan.commands.iter().any(|command|matches!(command,NativeDrawCommand::RoundRect{fill:NativeDrawFill::RoleOverride{color,..},..} if *color==pressed)));
}

#[test]
#[cfg(feature = "button")]
fn existing_button_background_and_radius_are_not_overwritten_by_internal_chrome() {
    let plan = render(
        crate::button("Save")
            .bg(ThemeColorToken::Danger)
            .radius(Dp::new(12.0)),
    );
    assert!(plan.commands.iter().any(|command| matches!(
        command,
        NativeDrawCommand::RoundRect {
            fill: NativeDrawFill::Role(ColorRole::Danger),
            radius: 12,
            ..
        }
    )));
}

#[test]
#[cfg(feature = "button")]
fn translucent_palette_is_idempotent_and_wide_rounded_borders_are_explicit() {
    let appearance = Appearance::default()
        .color(ColorRole::Control, Color::rgba(90, 50, 130, 180))
        .states(StateStyles {
            normal: ControlStyle::default()
                .border(Color::rgb(10, 10, 10), Dp::new(4.0))
                .radius(Dp::new(10.0)),
            ..Default::default()
        });
    let mut plan = render(
        crate::button("Save")
            .id(WidgetId(105))
            .appearance(appearance),
    );
    assert!(plan.commands.iter().any(|command| matches!(
        command,
        NativeDrawCommand::StyledRoundRect {
            width: 4,
            radius: 10,
            ..
        }
    )));
    let before = plan.commands.clone();
    plan.apply_appearance_states(None, None, None, false, Dpi::standard());
    assert_eq!(plan.commands, before);
    let chosen = crate::appearance::override_color(
        Color::rgba(1, 2, 3, 10),
        Color::rgb(240, 240, 0),
        64,
        true,
    );
    assert_eq!(chosen, Color::rgba(240, 240, 0, 64));
}

#[test]
#[cfg(all(feature = "button", feature = "label"))]
fn local_overrides_do_not_bleed_into_siblings() {
    let first = Color::rgb(10, 80, 200);
    let second = Color::rgb(180, 30, 40);
    let plan = render(
        column([
            crate::text("first").appearance(Appearance::default().states(StateStyles {
                normal: ControlStyle::default().foreground(first),
                ..Default::default()
            })),
            crate::text("second"),
        ])
        .theme(AppTheme::new(
            Appearance::default().color(ColorRole::PrimaryText, second),
        )),
    );
    let colors = plan
        .commands
        .iter()
        .filter_map(|command| match command {
            NativeDrawCommand::Text(text) => Some((text.text.as_str(), text.color)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        colors,
        vec![("first", Some(first)), ("second", Some(second))]
    );
}

#[test]
#[cfg(feature = "textbox")]
fn focus_appearance_changes_chrome_without_changing_focus_identity() {
    let id = WidgetId(104);
    let mut view = crate::textbox::<()>("中文")
        .id(id)
        .appearance(Appearance::default().focus(FocusAppearance::Outline));
    view.layout(&mut ViewLayoutCx::new(
        Rect {
            x: 0,
            y: 0,
            width: 240,
            height: 44,
        },
        Dpi::standard(),
    ));
    let interaction = view.interaction_plan();
    let mut paint = ViewPaintCx::new(Dpi::standard());
    view.paint(&mut paint);
    let mut plan = paint.into_plan();
    let rect = crate::native_input_visuals::decorate_native_focus_ring(
        &mut plan,
        &interaction,
        Some(id),
        true,
        Dpi::standard(),
    )
    .unwrap();
    assert_eq!(interaction.next_focus_target(None, 1).unwrap().widget, id);
    assert!(
        matches!(plan.commands.last(),Some(NativeDrawCommand::StrokeRect{rect:actual,..}) if *actual==rect)
    );
}

#[test]
#[cfg(feature = "label")]
fn explicit_font_height_changes_layout_and_measurement_identity() {
    let mut view = crate::text::<()>("中文").font(
        FontSpec::family("Noto Sans CJK SC")
            .size(Dp::new(20.0))
            .line_height(Dp::new(30.0)),
    );
    view.layout(&mut ViewLayoutCx::new(
        Rect {
            x: 0,
            y: 0,
            width: 300,
            height: 80,
        },
        Dpi::standard(),
    ));
    assert!(
        intrinsic_min_height_px(
            &view,
            300,
            Dpi::standard(),
            1.0,
            &ViewTextMeasurements::default()
        ) >= 30
    );
    let style = crate::SemanticTextStyle::body();
    let mut measurements = ViewTextMeasurements::default();
    let font = Arc::new(FontSpec::family("Example"));
    measurements.insert_with_font(
        "中文",
        style,
        0,
        crate::Size {
            width: 80,
            height: 32,
        },
        Some(font.clone()),
    );
    assert!(measurements.measure("中文", style, 0).is_none());
    assert_eq!(
        measurements.with_font(Some(font)).measure("中文", style, 0),
        Some(crate::Size {
            width: 80,
            height: 32
        })
    );
}

#[test]
#[cfg(all(feature = "label", feature = "button"))]
fn short_line_height_and_compact_density_preserve_the_resolved_text_line() {
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let dpi = Dpi::new(96.0 * scale);
        let font = FontSpec::default()
            .size(Dp::new(40.0))
            .line_height(Dp::new(5.0));
        let mut view = column([
            crate::text::<()>("中文 Label").font(font.clone()),
            crate::button("Save")
                .font(font)
                .appearance(Appearance::default().density(crate::ControlDensity::Compact)),
        ])
        .align(ViewAlign::Stretch);
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: Dp::new(320.0).to_px(dpi).round_i32(),
                height: Dp::new(240.0).to_px(dpi).round_i32(),
            },
            dpi,
        ));
        let mut paint = ViewPaintCx::new(dpi);
        view.paint(&mut paint);
        let mut checked = 0;
        for command in paint.into_plan().commands {
            if let NativeDrawCommand::Text(text) = command {
                let mut style = crate::TextStyle::line("sans-serif", 14.0, Color::rgb(0, 0, 0));
                text.apply_size_override(&mut style);
                assert_eq!(style.size, 40.0);
                assert_eq!(style.line_height, 40.0);
                assert!(
                    text.bounds.height >= Dp::new(style.line_height).to_px(dpi).round_i32(),
                    "{} at {}x: content {:?}, line {} DP",
                    text.text,
                    scale,
                    text.bounds,
                    style.line_height
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 2);
    }
}

#[test]
#[cfg(feature = "tabs")]
fn tab_presentations_keep_selection_and_keyboard_targets() {
    for mode in [
        crate::TabAppearance::Underline,
        crate::TabAppearance::Filled,
        crate::TabAppearance::Pill,
        crate::TabAppearance::Segmented,
    ] {
        let mut view = crate::tab_view(
            [
                crate::ZsTabItem::new(crate::ZsTabId(1), "One", crate::spacer::<()>()),
                crate::ZsTabItem::new(crate::ZsTabId(2), "Two", crate::spacer()),
            ],
            Some(crate::ZsTabId(1)),
        )
        .id(WidgetId(301))
        .appearance(Appearance::default().tabs(mode));
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 200,
            },
            Dpi::standard(),
        ));
        let targets = view.interaction_plan().hit_targets;
        assert!(targets.iter().any(|target| matches!(
            target.kind,
            ViewHitTargetKind::Tab {
                tab: crate::ZsTabId(1),
                ..
            }
        )));
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint
            .into_plan()
            .commands
            .iter()
            .any(|command| matches!(command,NativeDrawCommand::Text(text) if text.text=="One")));
    }
}
