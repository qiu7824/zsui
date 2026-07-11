use std::str::FromStr;

use rust_decimal::{Decimal, MathematicalOps};
use serde::{Deserialize, Serialize};

use crate::{
    ColorRole, Dp, Dpi, HorizontalAlign, NativeDrawCommand, NativeDrawFill, NativeDrawIconCommand,
    NativeDrawPlan, NativeDrawTextCommand, NativeIconColorMode, Point, Rect, SemanticTextStyle,
    TextRole, TextWeight, TextWrap, VerticalAlign, ZsIcon,
};

const HEADER_HEIGHT_DP: f32 = 56.0;
const DISPLAY_HEIGHT_DP: f32 = 124.0;
const MEMORY_HEIGHT_DP: f32 = 36.0;
const SURFACE_MARGIN_DP: f32 = 8.0;
const BUTTON_GAP_DP: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsCalculatorBinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl ZsCalculatorBinaryOperator {
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "×",
            Self::Divide => "÷",
        }
    }

    fn apply(self, lhs: Decimal, rhs: Decimal) -> Option<Decimal> {
        match self {
            Self::Add => lhs.checked_add(rhs),
            Self::Subtract => lhs.checked_sub(rhs),
            Self::Multiply => lhs.checked_mul(rhs),
            Self::Divide if rhs.is_zero() => None,
            Self::Divide => lhs.checked_div(rhs),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsCalculatorAction {
    Digit(u8),
    DecimalPoint,
    ToggleSign,
    Percent,
    ClearEntry,
    ClearAll,
    Backspace,
    Reciprocal,
    Square,
    SquareRoot,
    Binary(ZsCalculatorBinaryOperator),
    Equals,
    MemoryClear,
    MemoryRecall,
    MemoryAdd,
    MemorySubtract,
    MemoryStore,
    ToggleHistory,
    ClearHistory,
}

impl ZsCalculatorAction {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Digit(0) => "0",
            Self::Digit(1) => "1",
            Self::Digit(2) => "2",
            Self::Digit(3) => "3",
            Self::Digit(4) => "4",
            Self::Digit(5) => "5",
            Self::Digit(6) => "6",
            Self::Digit(7) => "7",
            Self::Digit(8) => "8",
            Self::Digit(9) => "9",
            Self::Digit(_) => "",
            Self::DecimalPoint => ".",
            Self::ToggleSign => "±",
            Self::Percent => "%",
            Self::ClearEntry => "CE",
            Self::ClearAll => "C",
            Self::Backspace => "",
            Self::Reciprocal => "1/x",
            Self::Square => "x²",
            Self::SquareRoot => "√x",
            Self::Binary(operator) => operator.symbol(),
            Self::Equals => "=",
            Self::MemoryClear => "MC",
            Self::MemoryRecall => "MR",
            Self::MemoryAdd => "M+",
            Self::MemorySubtract => "M-",
            Self::MemoryStore => "MS",
            Self::ToggleHistory => "",
            Self::ClearHistory => "Clear",
        }
    }

    pub const fn icon(self) -> Option<ZsIcon> {
        match self {
            Self::Backspace => Some(ZsIcon::Backspace),
            Self::ToggleHistory => Some(ZsIcon::History),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsCalculatorHistoryEntry {
    pub expression: String,
    pub result: String,
}

impl ZsCalculatorHistoryEntry {
    fn new(expression: impl Into<String>, result: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
            result: result.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZsCalculatorEngine {
    display: String,
    expression: String,
    accumulator: Option<Decimal>,
    pending_operator: Option<ZsCalculatorBinaryOperator>,
    last_operator: Option<ZsCalculatorBinaryOperator>,
    last_operand: Option<Decimal>,
    overwrite: bool,
    operand_entered: bool,
    memory: Decimal,
    history: Vec<ZsCalculatorHistoryEntry>,
    error: bool,
}

impl Default for ZsCalculatorEngine {
    fn default() -> Self {
        Self {
            display: "0".to_string(),
            expression: String::new(),
            accumulator: None,
            pending_operator: None,
            last_operator: None,
            last_operand: None,
            overwrite: true,
            operand_entered: false,
            memory: Decimal::ZERO,
            history: Vec::new(),
            error: false,
        }
    }
}

impl ZsCalculatorEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display(&self) -> &str {
        &self.display
    }

    pub fn expression(&self) -> &str {
        &self.expression
    }

    pub fn history(&self) -> &[ZsCalculatorHistoryEntry] {
        &self.history
    }

    pub fn memory_active(&self) -> bool {
        !self.memory.is_zero()
    }

    pub fn has_error(&self) -> bool {
        self.error
    }

    pub fn apply(&mut self, action: ZsCalculatorAction) {
        if self.error
            && !matches!(
                action,
                ZsCalculatorAction::ClearAll
                    | ZsCalculatorAction::ClearEntry
                    | ZsCalculatorAction::Digit(_)
                    | ZsCalculatorAction::DecimalPoint
            )
        {
            return;
        }
        if self.error {
            self.clear_all();
        }

        match action {
            ZsCalculatorAction::Digit(digit) if digit <= 9 => self.input_digit(digit),
            ZsCalculatorAction::Digit(_) => {}
            ZsCalculatorAction::DecimalPoint => self.input_decimal_point(),
            ZsCalculatorAction::ToggleSign => self.toggle_sign(),
            ZsCalculatorAction::Percent => self.percent(),
            ZsCalculatorAction::ClearEntry => self.clear_entry(),
            ZsCalculatorAction::ClearAll => self.clear_all(),
            ZsCalculatorAction::Backspace => self.backspace(),
            ZsCalculatorAction::Reciprocal => self.reciprocal(),
            ZsCalculatorAction::Square => self.square(),
            ZsCalculatorAction::SquareRoot => self.square_root(),
            ZsCalculatorAction::Binary(operator) => self.binary(operator),
            ZsCalculatorAction::Equals => self.equals(),
            ZsCalculatorAction::MemoryClear => self.memory = Decimal::ZERO,
            ZsCalculatorAction::MemoryRecall => {
                self.set_display_value(self.memory);
                self.overwrite = true;
                self.operand_entered = self.pending_operator.is_some();
                if self.pending_operator.is_none() {
                    self.last_operator = None;
                    self.last_operand = None;
                }
            }
            ZsCalculatorAction::MemoryAdd => {
                if let Some(value) = self.current_value() {
                    if let Some(result) = self.memory.checked_add(value) {
                        self.memory = result;
                    }
                }
            }
            ZsCalculatorAction::MemorySubtract => {
                if let Some(value) = self.current_value() {
                    if let Some(result) = self.memory.checked_sub(value) {
                        self.memory = result;
                    }
                }
            }
            ZsCalculatorAction::MemoryStore => {
                if let Some(value) = self.current_value() {
                    self.memory = value;
                }
            }
            ZsCalculatorAction::ClearHistory => self.history.clear(),
            ZsCalculatorAction::ToggleHistory => {}
        }
    }

    fn input_digit(&mut self, digit: u8) {
        if self.overwrite {
            self.display = digit.to_string();
            self.overwrite = false;
            if self.pending_operator.is_none() {
                self.expression.clear();
                self.last_operator = None;
                self.last_operand = None;
            } else {
                self.operand_entered = true;
            }
            return;
        }
        if self.pending_operator.is_some() {
            self.operand_entered = true;
        }
        let digit_count = self.display.bytes().filter(u8::is_ascii_digit).count();
        if digit_count >= 18 {
            return;
        }
        if self.display == "0" {
            self.display.clear();
        }
        self.display.push(char::from(b'0' + digit));
    }

    fn input_decimal_point(&mut self) {
        if self.overwrite {
            self.display = "0.".to_string();
            self.overwrite = false;
            if self.pending_operator.is_none() {
                self.expression.clear();
                self.last_operator = None;
                self.last_operand = None;
            } else {
                self.operand_entered = true;
            }
        } else if !self.display.contains('.') {
            self.display.push('.');
            if self.pending_operator.is_some() {
                self.operand_entered = true;
            }
        }
    }

    fn toggle_sign(&mut self) {
        if self.display == "0" {
            return;
        }
        if self.display.starts_with('-') {
            self.display.remove(0);
        } else {
            self.display.insert(0, '-');
        }
        if self.pending_operator.is_some() {
            self.operand_entered = true;
        }
    }

    fn percent(&mut self) {
        let Some(value) = self.current_value() else {
            return;
        };
        let base = match (self.accumulator, self.pending_operator) {
            (
                Some(lhs),
                Some(ZsCalculatorBinaryOperator::Add | ZsCalculatorBinaryOperator::Subtract),
            ) => lhs.checked_mul(value),
            _ => Some(value),
        };
        let Some(result) = base.and_then(|value| value.checked_div(Decimal::from(100))) else {
            self.set_error("Value is out of range");
            return;
        };
        self.set_display_value(result);
        self.overwrite = true;
        self.operand_entered = self.pending_operator.is_some();
        if self.pending_operator.is_none() {
            self.last_operator = None;
            self.last_operand = None;
        }
    }

    fn clear_entry(&mut self) {
        self.display = "0".to_string();
        self.overwrite = true;
        self.operand_entered = self.pending_operator.is_some();
        self.error = false;
    }

    fn clear_all(&mut self) {
        self.display = "0".to_string();
        self.expression.clear();
        self.accumulator = None;
        self.pending_operator = None;
        self.last_operator = None;
        self.last_operand = None;
        self.overwrite = true;
        self.operand_entered = false;
        self.error = false;
    }

    fn backspace(&mut self) {
        if self.overwrite {
            return;
        }
        self.display.pop();
        if self.display.is_empty() || self.display == "-" {
            self.display = "0".to_string();
            self.overwrite = true;
        }
    }

    fn reciprocal(&mut self) {
        let Some(value) = self.current_value() else {
            return;
        };
        if value.is_zero() {
            self.set_error("Cannot divide by zero");
            return;
        }
        let Some(result) = Decimal::ONE.checked_div(value) else {
            self.set_error("Value is out of range");
            return;
        };
        self.record_unary(format!("1/({})", format_decimal(value)), result);
    }

    fn square(&mut self) {
        let Some(value) = self.current_value() else {
            return;
        };
        let Some(result) = value.checked_mul(value) else {
            self.set_error("Value is out of range");
            return;
        };
        self.record_unary(format!("sqr({})", format_decimal(value)), result);
    }

    fn square_root(&mut self) {
        let Some(value) = self.current_value() else {
            return;
        };
        if value.is_sign_negative() {
            self.set_error("Invalid input");
            return;
        }
        let Some(result) = value.sqrt() else {
            self.set_error("Value is out of range");
            return;
        };
        self.record_unary(format!("√({})", format_decimal(value)), result);
    }

    fn binary(&mut self, operator: ZsCalculatorBinaryOperator) {
        let Some(mut value) = self.current_value() else {
            return;
        };
        if let (Some(lhs), Some(pending)) = (self.accumulator, self.pending_operator) {
            if self.operand_entered {
                let Some(result) = pending.apply(lhs, value) else {
                    self.set_error(if pending == ZsCalculatorBinaryOperator::Divide {
                        "Cannot divide by zero"
                    } else {
                        "Value is out of range"
                    });
                    return;
                };
                value = result;
                self.set_display_value(result);
            } else {
                value = lhs;
            }
        }
        self.accumulator = Some(value);
        self.pending_operator = Some(operator);
        self.last_operator = None;
        self.last_operand = None;
        self.expression = format!("{} {}", format_decimal(value), operator.symbol());
        self.overwrite = true;
        self.operand_entered = false;
    }

    fn equals(&mut self) {
        let current = match self.current_value() {
            Some(value) => value,
            None => return,
        };
        let operation = if let (Some(lhs), Some(operator)) =
            (self.accumulator.take(), self.pending_operator.take())
        {
            let rhs = if self.operand_entered { current } else { lhs };
            Some((lhs, operator, rhs))
        } else if let (Some(operator), Some(rhs)) = (self.last_operator, self.last_operand) {
            Some((current, operator, rhs))
        } else {
            None
        };
        let Some((lhs, operator, rhs)) = operation else {
            self.overwrite = true;
            return;
        };
        let Some(result) = operator.apply(lhs, rhs) else {
            self.set_error(if operator == ZsCalculatorBinaryOperator::Divide {
                "Cannot divide by zero"
            } else {
                "Value is out of range"
            });
            return;
        };
        let expression = format!(
            "{} {} {} =",
            format_decimal(lhs),
            operator.symbol(),
            format_decimal(rhs)
        );
        let formatted = format_decimal(result);
        self.history
            .push(ZsCalculatorHistoryEntry::new(&expression, &formatted));
        self.expression = expression;
        self.display = formatted;
        self.last_operator = Some(operator);
        self.last_operand = Some(rhs);
        self.overwrite = true;
        self.operand_entered = false;
    }

    fn record_unary(&mut self, expression: String, result: Decimal) {
        let has_pending_operator = self.pending_operator.is_some();
        let formatted = format_decimal(result);
        self.history
            .push(ZsCalculatorHistoryEntry::new(&expression, &formatted));
        self.expression = expression;
        self.display = formatted;
        self.overwrite = true;
        self.operand_entered = has_pending_operator;
        if !has_pending_operator {
            self.last_operator = None;
            self.last_operand = None;
        }
    }

    fn current_value(&self) -> Option<Decimal> {
        let source = self.display.strip_suffix('.').unwrap_or(&self.display);
        Decimal::from_str(source).ok()
    }

    fn set_display_value(&mut self, value: Decimal) {
        self.display = format_decimal(value);
        self.error = false;
    }

    fn set_error(&mut self, message: &str) {
        self.display = message.to_string();
        self.expression.clear();
        self.accumulator = None;
        self.pending_operator = None;
        self.last_operator = None;
        self.last_operand = None;
        self.overwrite = true;
        self.operand_entered = false;
        self.error = true;
    }
}

fn format_decimal(value: Decimal) -> String {
    let value = if value.is_zero() {
        Decimal::ZERO
    } else {
        value.normalize()
    };
    value.to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsCalculatorButtonKind {
    Number,
    Function,
    Operator,
    Accent,
    Memory,
    Header,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsCalculatorButtonRegion {
    pub action: ZsCalculatorAction,
    pub bounds: Rect,
    pub kind: ZsCalculatorButtonKind,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsCalculatorInteraction {
    pub hovered: Option<ZsCalculatorAction>,
    pub pressed: Option<ZsCalculatorAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsCalculatorLayout {
    pub surface: Rect,
    pub header: Rect,
    pub expression: Rect,
    pub display: Rect,
    pub memory_row: Rect,
    pub keypad: Rect,
    pub history_panel: Rect,
    pub button_regions: Vec<ZsCalculatorButtonRegion>,
}

impl ZsCalculatorLayout {
    pub fn action_at(&self, point: Point) -> Option<ZsCalculatorAction> {
        self.button_regions
            .iter()
            .find(|region| region.enabled && region.bounds.contains(point))
            .map(|region| region.action)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsCalculatorShellSpec {
    pub title: String,
    pub display: String,
    pub expression: String,
    pub memory_active: bool,
    pub history: Vec<ZsCalculatorHistoryEntry>,
    pub history_visible: bool,
}

impl ZsCalculatorShellSpec {
    pub fn from_engine(engine: &ZsCalculatorEngine) -> Self {
        Self {
            title: "Standard".to_string(),
            display: engine.display().to_string(),
            expression: engine.expression().to_string(),
            memory_active: engine.memory_active(),
            history: engine.history().to_vec(),
            history_visible: false,
        }
    }

    pub fn history_visible(mut self, history_visible: bool) -> Self {
        self.history_visible = history_visible;
        self
    }

    pub fn layout(&self, surface: Rect, dpi: Dpi) -> ZsCalculatorLayout {
        let margin = px(SURFACE_MARGIN_DP, dpi);
        let gap = px(BUTTON_GAP_DP, dpi);
        let header_height = px(HEADER_HEIGHT_DP, dpi);
        let display_height = px(DISPLAY_HEIGHT_DP, dpi);
        let memory_height = px(MEMORY_HEIGHT_DP, dpi);
        let header = Rect {
            x: surface.x,
            y: surface.y,
            width: surface.width.max(0),
            height: header_height,
        };
        let expression = Rect {
            x: surface.x + margin,
            y: header.y + header.height + px(8.0, dpi),
            width: (surface.width - margin * 2).max(0),
            height: px(24.0, dpi),
        };
        let display = Rect {
            x: expression.x,
            y: expression.y + expression.height,
            width: expression.width,
            height: (display_height - expression.height - px(8.0, dpi)).max(0),
        };
        let memory_row = Rect {
            x: surface.x + margin,
            y: header.y + header.height + display_height,
            width: (surface.width - margin * 2).max(0),
            height: memory_height,
        };
        let keypad = Rect {
            x: surface.x + margin,
            y: memory_row.y + memory_row.height + gap,
            width: (surface.width - margin * 2).max(0),
            height: (surface.y + surface.height - margin - memory_row.y - memory_row.height - gap)
                .max(0),
        };
        let history_panel = Rect {
            x: surface.x + margin,
            y: header.y + header.height,
            width: (surface.width - margin * 2).max(0),
            height: (surface.height - header.height - margin).max(0),
        };

        let mut button_regions = vec![ZsCalculatorButtonRegion {
            action: ZsCalculatorAction::ToggleHistory,
            bounds: Rect {
                x: surface.x + surface.width - margin - px(36.0, dpi),
                y: surface.y + (header_height - px(36.0, dpi)) / 2,
                width: px(36.0, dpi),
                height: px(36.0, dpi),
            },
            kind: ZsCalculatorButtonKind::Header,
            enabled: true,
        }];

        if self.history_visible {
            button_regions.push(ZsCalculatorButtonRegion {
                action: ZsCalculatorAction::ClearHistory,
                bounds: Rect {
                    x: history_panel.x + history_panel.width - px(78.0, dpi),
                    y: history_panel.y + px(10.0, dpi),
                    width: px(68.0, dpi),
                    height: px(30.0, dpi),
                },
                kind: ZsCalculatorButtonKind::Function,
                enabled: !self.history.is_empty(),
            });
        } else {
            push_memory_regions(&mut button_regions, memory_row, gap, self.memory_active);
            push_keypad_regions(&mut button_regions, keypad, gap);
        }

        ZsCalculatorLayout {
            surface,
            header,
            expression,
            display,
            memory_row,
            keypad,
            history_panel,
            button_regions,
        }
    }

    pub fn native_draw_plan(
        &self,
        surface: Rect,
        dpi: Dpi,
        interaction: ZsCalculatorInteraction,
    ) -> NativeDrawPlan {
        let layout = self.layout(surface, dpi);
        let mut commands = vec![fill(surface, NativeDrawFill::Role(ColorRole::Surface))];

        commands.push(icon(
            ZsIcon::Calculator,
            Rect {
                x: surface.x + px(14.0, dpi),
                y: surface.y + (layout.header.height - px(20.0, dpi)) / 2,
                width: px(20.0, dpi),
                height: px(20.0, dpi),
            },
            ColorRole::Accent,
        ));
        commands.push(text(
            &self.title,
            Rect {
                x: surface.x + px(44.0, dpi),
                y: layout.header.y,
                width: (layout.header.width - px(96.0, dpi)).max(0),
                height: layout.header.height,
            },
            text_style(
                TextRole::Subtitle,
                ColorRole::PrimaryText,
                TextWeight::Semibold,
                HorizontalAlign::Start,
            ),
        ));

        let history_button = layout
            .button_regions
            .iter()
            .find(|region| region.action == ZsCalculatorAction::ToggleHistory)
            .expect("history button is always present");
        paint_button(history_button, interaction, dpi, &mut commands);

        if self.history_visible {
            paint_history(self, &layout, interaction, dpi, &mut commands);
        } else {
            if self.memory_active {
                commands.push(text(
                    "M",
                    Rect {
                        x: layout.expression.x,
                        y: layout.expression.y,
                        width: px(20.0, dpi),
                        height: layout.expression.height,
                    },
                    text_style(
                        TextRole::Caption,
                        ColorRole::Accent,
                        TextWeight::Semibold,
                        HorizontalAlign::Start,
                    ),
                ));
            }
            commands.push(text(
                &self.expression,
                layout.expression,
                text_style(
                    TextRole::Caption,
                    ColorRole::SecondaryText,
                    TextWeight::Regular,
                    HorizontalAlign::End,
                ),
            ));
            let display_role = if self.display.chars().count() > 16 {
                TextRole::Title
            } else {
                TextRole::Display
            };
            commands.push(text(
                &self.display,
                layout.display,
                text_style(
                    display_role,
                    ColorRole::PrimaryText,
                    TextWeight::Semibold,
                    HorizontalAlign::End,
                ),
            ));
            for region in layout
                .button_regions
                .iter()
                .filter(|region| region.action != ZsCalculatorAction::ToggleHistory)
            {
                paint_button(region, interaction, dpi, &mut commands);
            }
        }

        NativeDrawPlan::new(commands)
    }
}

const MEMORY_ACTIONS: [ZsCalculatorAction; 5] = [
    ZsCalculatorAction::MemoryClear,
    ZsCalculatorAction::MemoryRecall,
    ZsCalculatorAction::MemoryAdd,
    ZsCalculatorAction::MemorySubtract,
    ZsCalculatorAction::MemoryStore,
];

const KEYPAD_ACTIONS: [[ZsCalculatorAction; 4]; 6] = [
    [
        ZsCalculatorAction::Percent,
        ZsCalculatorAction::ClearEntry,
        ZsCalculatorAction::ClearAll,
        ZsCalculatorAction::Backspace,
    ],
    [
        ZsCalculatorAction::Reciprocal,
        ZsCalculatorAction::Square,
        ZsCalculatorAction::SquareRoot,
        ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Divide),
    ],
    [
        ZsCalculatorAction::Digit(7),
        ZsCalculatorAction::Digit(8),
        ZsCalculatorAction::Digit(9),
        ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Multiply),
    ],
    [
        ZsCalculatorAction::Digit(4),
        ZsCalculatorAction::Digit(5),
        ZsCalculatorAction::Digit(6),
        ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Subtract),
    ],
    [
        ZsCalculatorAction::Digit(1),
        ZsCalculatorAction::Digit(2),
        ZsCalculatorAction::Digit(3),
        ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Add),
    ],
    [
        ZsCalculatorAction::ToggleSign,
        ZsCalculatorAction::Digit(0),
        ZsCalculatorAction::DecimalPoint,
        ZsCalculatorAction::Equals,
    ],
];

fn push_memory_regions(
    regions: &mut Vec<ZsCalculatorButtonRegion>,
    bounds: Rect,
    gap: i32,
    memory_active: bool,
) {
    let width = ((bounds.width - gap * (MEMORY_ACTIONS.len() as i32 - 1))
        / MEMORY_ACTIONS.len() as i32)
        .max(1);
    for (index, action) in MEMORY_ACTIONS.into_iter().enumerate() {
        regions.push(ZsCalculatorButtonRegion {
            action,
            bounds: Rect {
                x: bounds.x + index as i32 * (width + gap),
                y: bounds.y,
                width,
                height: bounds.height,
            },
            kind: ZsCalculatorButtonKind::Memory,
            enabled: !matches!(
                action,
                ZsCalculatorAction::MemoryClear | ZsCalculatorAction::MemoryRecall
            ) || memory_active,
        });
    }
}

fn push_keypad_regions(regions: &mut Vec<ZsCalculatorButtonRegion>, bounds: Rect, gap: i32) {
    let width = ((bounds.width - gap * 3) / 4).max(1);
    let height = ((bounds.height - gap * 5) / 6).max(1);
    for (row, actions) in KEYPAD_ACTIONS.into_iter().enumerate() {
        for (column, action) in actions.into_iter().enumerate() {
            regions.push(ZsCalculatorButtonRegion {
                action,
                bounds: Rect {
                    x: bounds.x + column as i32 * (width + gap),
                    y: bounds.y + row as i32 * (height + gap),
                    width,
                    height,
                },
                kind: match action {
                    ZsCalculatorAction::Digit(_)
                    | ZsCalculatorAction::DecimalPoint
                    | ZsCalculatorAction::ToggleSign => ZsCalculatorButtonKind::Number,
                    ZsCalculatorAction::Equals => ZsCalculatorButtonKind::Accent,
                    ZsCalculatorAction::Binary(_) => ZsCalculatorButtonKind::Operator,
                    _ => ZsCalculatorButtonKind::Function,
                },
                enabled: true,
            });
        }
    }
}

fn paint_history(
    spec: &ZsCalculatorShellSpec,
    layout: &ZsCalculatorLayout,
    interaction: ZsCalculatorInteraction,
    dpi: Dpi,
    commands: &mut Vec<NativeDrawCommand>,
) {
    commands.push(round_rect(
        layout.history_panel,
        NativeDrawFill::Role(ColorRole::SurfaceRaised),
        Some(NativeDrawFill::Role(ColorRole::Border)),
        px(8.0, dpi),
    ));
    commands.push(text(
        "History",
        Rect {
            x: layout.history_panel.x + px(16.0, dpi),
            y: layout.history_panel.y + px(8.0, dpi),
            width: (layout.history_panel.width - px(108.0, dpi)).max(0),
            height: px(36.0, dpi),
        },
        text_style(
            TextRole::Subtitle,
            ColorRole::PrimaryText,
            TextWeight::Semibold,
            HorizontalAlign::Start,
        ),
    ));
    if let Some(clear) = layout
        .button_regions
        .iter()
        .find(|region| region.action == ZsCalculatorAction::ClearHistory)
    {
        paint_button(clear, interaction, dpi, commands);
    }

    if spec.history.is_empty() {
        commands.push(text(
            "No history yet",
            Rect {
                x: layout.history_panel.x + px(16.0, dpi),
                y: layout.history_panel.y + px(72.0, dpi),
                width: (layout.history_panel.width - px(32.0, dpi)).max(0),
                height: px(28.0, dpi),
            },
            text_style(
                TextRole::Body,
                ColorRole::SecondaryText,
                TextWeight::Regular,
                HorizontalAlign::Start,
            ),
        ));
        return;
    }

    let mut y = layout.history_panel.y + px(58.0, dpi);
    for entry in spec.history.iter().rev().take(7) {
        commands.push(text(
            &entry.expression,
            Rect {
                x: layout.history_panel.x + px(16.0, dpi),
                y,
                width: (layout.history_panel.width - px(32.0, dpi)).max(0),
                height: px(22.0, dpi),
            },
            text_style(
                TextRole::Caption,
                ColorRole::SecondaryText,
                TextWeight::Regular,
                HorizontalAlign::End,
            ),
        ));
        y += px(22.0, dpi);
        commands.push(text(
            &entry.result,
            Rect {
                x: layout.history_panel.x + px(16.0, dpi),
                y,
                width: (layout.history_panel.width - px(32.0, dpi)).max(0),
                height: px(30.0, dpi),
            },
            text_style(
                TextRole::BodyLarge,
                ColorRole::PrimaryText,
                TextWeight::Semibold,
                HorizontalAlign::End,
            ),
        ));
        y += px(42.0, dpi);
    }
}

fn paint_button(
    region: &ZsCalculatorButtonRegion,
    interaction: ZsCalculatorInteraction,
    dpi: Dpi,
    commands: &mut Vec<NativeDrawCommand>,
) {
    let hovered = interaction.hovered == Some(region.action) && region.enabled;
    let pressed = interaction.pressed == Some(region.action) && region.enabled;
    let base_fill = match region.kind {
        ZsCalculatorButtonKind::Number => Some(NativeDrawFill::Role(ColorRole::SurfaceRaised)),
        ZsCalculatorButtonKind::Function | ZsCalculatorButtonKind::Operator => {
            Some(NativeDrawFill::Role(ColorRole::Control))
        }
        ZsCalculatorButtonKind::Accent => Some(NativeDrawFill::Role(ColorRole::Accent)),
        ZsCalculatorButtonKind::Memory | ZsCalculatorButtonKind::Header => None,
    };
    let fill = if pressed {
        Some(NativeDrawFill::RoleWithAlpha {
            role: ColorRole::Accent,
            alpha: if region.kind == ZsCalculatorButtonKind::Accent {
                210
            } else {
                50
            },
        })
    } else if hovered {
        Some(if region.kind == ZsCalculatorButtonKind::Accent {
            NativeDrawFill::RoleWithAlpha {
                role: ColorRole::Accent,
                alpha: 224,
            }
        } else {
            NativeDrawFill::RoleWithAlpha {
                role: ColorRole::Accent,
                alpha: 24,
            }
        })
    } else {
        base_fill
    };
    if let Some(fill) = fill {
        commands.push(round_rect(
            region.bounds,
            fill,
            if matches!(
                region.kind,
                ZsCalculatorButtonKind::Number
                    | ZsCalculatorButtonKind::Function
                    | ZsCalculatorButtonKind::Operator
            ) {
                Some(NativeDrawFill::Role(ColorRole::Border))
            } else {
                None
            },
            px(6.0, dpi),
        ));
    }

    let color = if !region.enabled {
        ColorRole::DisabledText
    } else if region.kind == ZsCalculatorButtonKind::Accent {
        ColorRole::AccentText
    } else {
        ColorRole::PrimaryText
    };
    if let Some(icon_value) = region.action.icon() {
        let size = px(18.0, dpi);
        commands.push(icon(
            icon_value,
            Rect {
                x: region.bounds.x + (region.bounds.width - size) / 2,
                y: region.bounds.y + (region.bounds.height - size) / 2,
                width: size,
                height: size,
            },
            color,
        ));
    } else {
        commands.push(text(
            region.action.label(),
            region.bounds,
            text_style(
                if region.kind == ZsCalculatorButtonKind::Number {
                    TextRole::BodyLarge
                } else {
                    TextRole::Button
                },
                color,
                if region.kind == ZsCalculatorButtonKind::Accent {
                    TextWeight::Semibold
                } else {
                    TextWeight::Regular
                },
                HorizontalAlign::Center,
            ),
        ));
    }
}

fn px(value: f32, dpi: Dpi) -> i32 {
    Dp::new(value).to_px(dpi).round_i32().max(1)
}

fn fill(rect: Rect, fill: NativeDrawFill) -> NativeDrawCommand {
    NativeDrawCommand::FillRect { rect, fill }
}

fn round_rect(
    rect: Rect,
    fill: NativeDrawFill,
    stroke: Option<NativeDrawFill>,
    radius: i32,
) -> NativeDrawCommand {
    NativeDrawCommand::RoundRect {
        rect,
        fill,
        stroke,
        radius,
    }
}

fn text(value: impl Into<String>, bounds: Rect, style: SemanticTextStyle) -> NativeDrawCommand {
    NativeDrawCommand::Text(NativeDrawTextCommand::new(value, bounds, style))
}

fn icon(value: ZsIcon, bounds: Rect, color: ColorRole) -> NativeDrawCommand {
    NativeDrawCommand::Icon(
        NativeDrawIconCommand::new(value, bounds, NativeIconColorMode::ThemeAware)
            .with_color(color),
    )
}

fn text_style(
    role: TextRole,
    color: ColorRole,
    weight: TextWeight,
    horizontal_align: HorizontalAlign,
) -> SemanticTextStyle {
    SemanticTextStyle {
        role,
        color,
        weight,
        horizontal_align,
        vertical_align: VerticalAlign::Center,
        wrap: TextWrap::NoWrap,
        ellipsis: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(engine: &mut ZsCalculatorEngine, actions: &[ZsCalculatorAction]) {
        for action in actions {
            engine.apply(*action);
        }
    }

    #[test]
    fn decimal_arithmetic_avoids_binary_float_error() {
        let mut engine = ZsCalculatorEngine::new();
        apply(
            &mut engine,
            &[
                ZsCalculatorAction::DecimalPoint,
                ZsCalculatorAction::Digit(1),
                ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Add),
                ZsCalculatorAction::DecimalPoint,
                ZsCalculatorAction::Digit(2),
                ZsCalculatorAction::Equals,
            ],
        );
        assert_eq!(engine.display(), "0.3");
    }

    #[test]
    fn repeated_equals_reuses_last_operation() {
        let mut engine = ZsCalculatorEngine::new();
        apply(
            &mut engine,
            &[
                ZsCalculatorAction::Digit(2),
                ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Add),
                ZsCalculatorAction::Digit(3),
                ZsCalculatorAction::Equals,
                ZsCalculatorAction::Equals,
            ],
        );
        assert_eq!(engine.display(), "8");
        assert_eq!(engine.history().len(), 2);
    }

    #[test]
    fn divide_by_zero_requires_clear_before_more_operations() {
        let mut engine = ZsCalculatorEngine::new();
        apply(
            &mut engine,
            &[
                ZsCalculatorAction::Digit(8),
                ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Divide),
                ZsCalculatorAction::Digit(0),
                ZsCalculatorAction::Equals,
            ],
        );
        assert!(engine.has_error());
        assert_eq!(engine.display(), "Cannot divide by zero");
        engine.apply(ZsCalculatorAction::ClearAll);
        assert_eq!(engine.display(), "0");
    }

    #[test]
    fn memory_and_unary_operations_are_explicit() {
        let mut engine = ZsCalculatorEngine::new();
        apply(
            &mut engine,
            &[
                ZsCalculatorAction::Digit(9),
                ZsCalculatorAction::MemoryStore,
                ZsCalculatorAction::ClearAll,
                ZsCalculatorAction::MemoryRecall,
                ZsCalculatorAction::SquareRoot,
            ],
        );
        assert_eq!(engine.display(), "3");
        assert!(engine.memory_active());
    }

    #[test]
    fn percent_uses_standard_additive_context() {
        let mut engine = ZsCalculatorEngine::new();
        apply(
            &mut engine,
            &[
                ZsCalculatorAction::Digit(2),
                ZsCalculatorAction::Digit(0),
                ZsCalculatorAction::Digit(0),
                ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Add),
                ZsCalculatorAction::Digit(1),
                ZsCalculatorAction::Digit(0),
                ZsCalculatorAction::Percent,
                ZsCalculatorAction::Equals,
            ],
        );
        assert_eq!(engine.display(), "220");
    }

    #[test]
    fn unary_result_is_an_explicit_pending_operand() {
        let mut engine = ZsCalculatorEngine::new();
        apply(
            &mut engine,
            &[
                ZsCalculatorAction::Digit(9),
                ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Add),
                ZsCalculatorAction::Digit(1),
                ZsCalculatorAction::Digit(6),
                ZsCalculatorAction::SquareRoot,
                ZsCalculatorAction::Equals,
            ],
        );
        assert_eq!(engine.display(), "13");
    }

    #[test]
    fn clear_entry_supplies_zero_to_a_pending_operation() {
        let mut engine = ZsCalculatorEngine::new();
        apply(
            &mut engine,
            &[
                ZsCalculatorAction::Digit(2),
                ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Add),
                ZsCalculatorAction::Digit(3),
                ZsCalculatorAction::ClearEntry,
                ZsCalculatorAction::Equals,
            ],
        );
        assert_eq!(engine.display(), "2");
    }

    #[test]
    fn new_input_does_not_reuse_a_previous_equals_operation() {
        let mut engine = ZsCalculatorEngine::new();
        apply(
            &mut engine,
            &[
                ZsCalculatorAction::Digit(2),
                ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Add),
                ZsCalculatorAction::Digit(3),
                ZsCalculatorAction::Equals,
                ZsCalculatorAction::Digit(7),
                ZsCalculatorAction::Equals,
            ],
        );
        assert_eq!(engine.display(), "7");
    }

    #[test]
    fn shell_layout_has_memory_and_six_keypad_rows() {
        let engine = ZsCalculatorEngine::new();
        let spec = ZsCalculatorShellSpec::from_engine(&engine);
        let layout = spec.layout(
            Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 640,
            },
            Dpi::standard(),
        );
        assert_eq!(layout.button_regions.len(), 1 + 5 + 24);
        assert!(layout.keypad.height > 0);
    }

    #[test]
    fn history_mode_blocks_underlying_keypad_hits() {
        let mut engine = ZsCalculatorEngine::new();
        engine.apply(ZsCalculatorAction::Digit(2));
        engine.apply(ZsCalculatorAction::Square);
        let spec = ZsCalculatorShellSpec::from_engine(&engine).history_visible(true);
        let layout = spec.layout(
            Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 640,
            },
            Dpi::standard(),
        );
        assert_eq!(layout.button_regions.len(), 2);
        assert!(layout
            .button_regions
            .iter()
            .any(|region| region.action == ZsCalculatorAction::ClearHistory));
    }

    #[test]
    fn shell_draw_plan_uses_icons_without_icon_text_glyphs() {
        let engine = ZsCalculatorEngine::new();
        let spec = ZsCalculatorShellSpec::from_engine(&engine);
        let plan = spec.native_draw_plan(
            Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 640,
            },
            Dpi::standard(),
            ZsCalculatorInteraction::default(),
        );
        assert!(plan.icon_count() >= 3);
        assert!(!plan.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(command) if command.style.role == TextRole::Icon
        )));
    }

    #[test]
    fn long_display_values_use_the_smaller_title_role() {
        let mut engine = ZsCalculatorEngine::new();
        for digit in [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7] {
            engine.apply(ZsCalculatorAction::Digit(digit));
        }
        let plan = ZsCalculatorShellSpec::from_engine(&engine).native_draw_plan(
            Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 640,
            },
            Dpi::standard(),
            ZsCalculatorInteraction::default(),
        );
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(command)
                if command.text == engine.display() && command.style.role == TextRole::Title
        )));
    }
}
