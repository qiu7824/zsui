/// Stable widget identifiers for one calculator View instance.
///
/// Each namespace reserves 64 widget identifiers. Applications can create
/// more than one calculator by assigning each instance a different namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZsCalculatorViewIds {
    namespace: u64,
}

impl ZsCalculatorViewIds {
    const SHIFT: u32 = 6;
    const MAX_NAMESPACE: u64 = u64::MAX >> Self::SHIFT;

    pub const fn new(namespace: u64) -> Self {
        assert!(
            namespace <= Self::MAX_NAMESPACE,
            "calculator widget namespace exceeds 58 bits"
        );
        Self { namespace }
    }

    pub const fn for_action(self, action: crate::ZsCalculatorAction) -> WidgetId {
        WidgetId((self.namespace << Self::SHIFT) | calculator_action_code(action))
    }
}

const fn calculator_action_code(action: crate::ZsCalculatorAction) -> u64 {
    use crate::{ZsCalculatorAction as Action, ZsCalculatorBinaryOperator as Operator};

    match action {
        Action::Digit(value) if value <= 9 => 1 + value as u64,
        Action::Digit(_) => 63,
        Action::DecimalPoint => 11,
        Action::ToggleSign => 12,
        Action::Percent => 13,
        Action::ClearEntry => 14,
        Action::ClearAll => 15,
        Action::Backspace => 16,
        Action::Reciprocal => 17,
        Action::Square => 18,
        Action::SquareRoot => 19,
        Action::Binary(Operator::Add) => 20,
        Action::Binary(Operator::Subtract) => 21,
        Action::Binary(Operator::Multiply) => 22,
        Action::Binary(Operator::Divide) => 23,
        Action::Equals => 24,
        Action::MemoryClear => 25,
        Action::MemoryRecall => 26,
        Action::MemoryAdd => 27,
        Action::MemorySubtract => 28,
        Action::MemoryStore => 29,
        Action::ToggleHistory => 30,
        Action::ClearHistory => 31,
    }
}

/// Builds the reusable standard-calculator surface with one platform-neutral
/// State/Msg/View contract. Buttons, spacing, type metrics and section
/// composition resolve to the active Fluent, AppKit or Linux experience inside
/// the framework; application code never selects a platform style.
pub fn calculator_view<Msg: Clone>(
    spec: &crate::ZsCalculatorShellSpec,
    ids: ZsCalculatorViewIds,
    mut message_for_action: impl FnMut(crate::ZsCalculatorAction) -> Msg,
) -> ViewNode<Msg> {
    let spacing = crate::ZsuiSpacingTokens::default();
    let header = row([
        styled_text(
            &spec.title,
            calculator_text_style(
                crate::TextRole::Subtitle,
                crate::ColorRole::PrimaryText,
                crate::TextWeight::Semibold,
                crate::HorizontalAlign::Start,
            ),
        ),
        icon_button("History", crate::ZsIcon::History)
            .id(ids.for_action(crate::ZsCalculatorAction::ToggleHistory))
            .on_click(message_for_action(
                crate::ZsCalculatorAction::ToggleHistory,
            )),
    ])
    .gap(spacing.sm);

    let content = if spec.history_visible {
        calculator_history_view(spec, ids, &mut message_for_action)
    } else {
        calculator_keypad_view(spec, ids, &mut message_for_action)
    };

    column([header, content])
        .gap(spacing.sm)
        .padding(spacing.sm)
        .bg(crate::ThemeColorToken::Surface)
}

fn calculator_keypad_view<Msg: Clone>(
    spec: &crate::ZsCalculatorShellSpec,
    ids: ZsCalculatorViewIds,
    message_for_action: &mut impl FnMut(crate::ZsCalculatorAction) -> Msg,
) -> ViewNode<Msg> {
    let spacing = crate::ZsuiSpacingTokens::default();
    let expression = styled_text(
        &spec.expression,
        calculator_text_style(
            crate::TextRole::Caption,
            crate::ColorRole::SecondaryText,
            crate::TextWeight::Regular,
            crate::HorizontalAlign::End,
        ),
    );
    let display_role = if spec.display.chars().count() > 16 {
        crate::TextRole::Title
    } else {
        crate::TextRole::Display
    };
    let display = styled_text(
        &spec.display,
        calculator_text_style(
            display_role,
            crate::ColorRole::PrimaryText,
            crate::TextWeight::Semibold,
            crate::HorizontalAlign::End,
        ),
    );

    let memory_items = crate::calculator::MEMORY_ACTIONS
        .into_iter()
        .enumerate()
        .map(|(column_index, action)| {
            let enabled = !matches!(
                action,
                crate::ZsCalculatorAction::MemoryClear
                    | crate::ZsCalculatorAction::MemoryRecall
            ) || spec.memory_active;
            ZsGridCell::new(
                0,
                column_index,
                calculator_action_button(action, ids, enabled, message_for_action),
            )
        })
        .collect::<Vec<_>>();
    let memory = grid(
        [ZsGridTrack::FLEX; 5],
        [ZsGridTrack::FLEX],
        memory_items,
    )
    .column_gap(spacing.xs)
    .height(crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    )
    .button_height)
    .flex(0.0);

    let keypad_items = crate::calculator::KEYPAD_ACTIONS
        .into_iter()
        .enumerate()
        .flat_map(|(row_index, actions)| {
            actions
                .into_iter()
                .enumerate()
                .map(move |(column_index, action)| (row_index, column_index, action))
        })
        .map(|(row_index, column_index, action)| {
            ZsGridCell::new(
                row_index,
                column_index,
                calculator_action_button(action, ids, true, message_for_action),
            )
        })
        .collect::<Vec<_>>();
    let keypad = grid(
        [ZsGridTrack::FLEX; 4],
        [ZsGridTrack::FLEX; 6],
        keypad_items,
    )
    .column_gap(spacing.xs)
    .row_gap(spacing.xs);

    column([expression, display, memory, keypad]).gap(spacing.xs)
}

fn calculator_history_view<Msg: Clone>(
    spec: &crate::ZsCalculatorShellSpec,
    ids: ZsCalculatorViewIds,
    message_for_action: &mut impl FnMut(crate::ZsCalculatorAction) -> Msg,
) -> ViewNode<Msg> {
    let spacing = crate::ZsuiSpacingTokens::default();
    let mut children = Vec::new();
    if spec.history.is_empty() {
        children.push(styled_text(
            "No history yet",
            calculator_text_style(
                crate::TextRole::Body,
                crate::ColorRole::SecondaryText,
                crate::TextWeight::Regular,
                crate::HorizontalAlign::Start,
            ),
        ));
    } else {
        children.push(
            button("Clear history")
                .id(ids.for_action(crate::ZsCalculatorAction::ClearHistory))
                .on_click(message_for_action(
                    crate::ZsCalculatorAction::ClearHistory,
                )),
        );
        children.extend(spec.history.iter().rev().take(7).map(|entry| {
            column([
                styled_text(
                    &entry.expression,
                    calculator_text_style(
                        crate::TextRole::Caption,
                        crate::ColorRole::SecondaryText,
                        crate::TextWeight::Regular,
                        crate::HorizontalAlign::End,
                    ),
                ),
                styled_text(
                    &entry.result,
                    calculator_text_style(
                        crate::TextRole::BodyLarge,
                        crate::ColorRole::PrimaryText,
                        crate::TextWeight::Semibold,
                        crate::HorizontalAlign::End,
                    ),
                ),
            ])
            .gap(spacing.xs)
            .flex(0.0)
        }));
    }
    section("History", children)
}

fn calculator_action_button<Msg: Clone>(
    action: crate::ZsCalculatorAction,
    ids: ZsCalculatorViewIds,
    enabled: bool,
    message_for_action: &mut impl FnMut(crate::ZsCalculatorAction) -> Msg,
) -> ViewNode<Msg> {
    let node = if let Some(icon) = action.icon() {
        icon_button(action_accessible_label(action), icon)
    } else if action == crate::ZsCalculatorAction::Equals {
        primary_button(action.label())
    } else {
        button(action.label())
    };
    node.id(ids.for_action(action))
        .enabled(enabled)
        .on_click(message_for_action(action))
}

const fn action_accessible_label(action: crate::ZsCalculatorAction) -> &'static str {
    match action {
        crate::ZsCalculatorAction::Backspace => "Backspace",
        crate::ZsCalculatorAction::ToggleHistory => "History",
        _ => action.label(),
    }
}

const fn calculator_text_style(
    role: crate::TextRole,
    color: crate::ColorRole,
    weight: crate::TextWeight,
    horizontal_align: crate::HorizontalAlign,
) -> crate::SemanticTextStyle {
    crate::SemanticTextStyle {
        role,
        color,
        weight,
        horizontal_align,
        vertical_align: crate::VerticalAlign::Center,
        wrap: crate::TextWrap::NoWrap,
        ellipsis: true,
    }
}

#[cfg(test)]
mod calculator_view_tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Msg {
        Action(crate::ZsCalculatorAction),
    }

    #[test]
    fn calculator_view_exposes_stable_actions_without_platform_input() {
        let ids = ZsCalculatorViewIds::new(7);
        let spec = crate::ZsCalculatorShellSpec::from_engine(&crate::ZsCalculatorEngine::new());
        let mut view = calculator_view(&spec, ids, Msg::Action);
        let mut layout = ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 680,
            },
            crate::Dpi::standard(),
        );
        view.layout(&mut layout);
        let interaction = view.interaction_plan();

        assert_eq!(interaction.hit_target_count(), 28);
        assert!(interaction
            .hit_target_for_widget(ids.for_action(crate::ZsCalculatorAction::Equals))
            .is_some());
        assert!(interaction
            .hit_target_for_widget(ids.for_action(crate::ZsCalculatorAction::MemoryRecall))
            .is_none());
    }

    #[test]
    fn calculator_action_namespaces_do_not_overlap() {
        let first = ZsCalculatorViewIds::new(1);
        let second = ZsCalculatorViewIds::new(2);
        for action in crate::calculator::MEMORY_ACTIONS
            .into_iter()
            .chain(crate::calculator::KEYPAD_ACTIONS.into_iter().flatten())
            .chain([
                crate::ZsCalculatorAction::ToggleHistory,
                crate::ZsCalculatorAction::ClearHistory,
            ])
        {
            assert_ne!(first.for_action(action), second.for_action(action));
        }
    }
}
