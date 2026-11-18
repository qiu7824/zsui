#[derive(Debug, Clone, Default)]
struct ViewMetadata {
    max_width: Option<Dp>,
    max_height: Option<Dp>,
    #[cfg(feature = "custom-icons")]
    icon_source: Option<crate::IconSource>,
    #[cfg(feature = "tooltip")]
    tooltip: Option<crate::ZsTooltipSpec>,
    #[cfg(feature = "accessibility")]
    accessibility: Option<crate::ZsAccessibilitySpec>,
    theme: Option<crate::AppTheme>,
    appearance: Option<Arc<crate::Appearance>>,
    effective: Option<Arc<crate::Appearance>>,
    state: crate::ControlState,
}
impl ViewMetadata {
    const EMPTY: Self = Self {
        max_width: None,
        max_height: None,
        #[cfg(feature = "custom-icons")]
        icon_source: None,
        #[cfg(feature = "tooltip")]
        tooltip: None,
        #[cfg(feature = "accessibility")]
        accessibility: None,
        theme: None,
        appearance: None,
        effective: None,
        state: crate::ControlState {
            hover: false,
            pressed: false,
            selected: false,
            focus_visible: false,
            disabled: false,
            invalid: false,
        },
    };
}
impl<Msg> ViewNode<Msg> {
    fn paints_own_surface(&self) -> bool {
        match self.kind {
            #[cfg(feature = "button")]
            ViewNodeKind::Button { presentation, .. } => {
                !matches!(presentation, ZsButtonPresentation::Toolbar { .. })
            }
            #[cfg(feature = "textbox")]
            ViewNodeKind::Textbox { .. } => true,
            #[cfg(feature = "password-box")]
            ViewNodeKind::PasswordBox { .. } => true,
            #[cfg(feature = "number-box")]
            ViewNodeKind::NumberBox { .. } => true,
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox { .. } => true,
            #[cfg(feature = "auto-suggest")]
            ViewNodeKind::AutoSuggestBox { .. } => true,
            _ => false,
        }
    }
    /// Sets artwork size while preserving the element's layout and hit slot.
    pub fn glyph_size(mut self, size: Dp) -> Self {
        let mut style = self
            .metadata()
            .appearance
            .as_ref()
            .map(|style| (**style).clone())
            .unwrap_or_default();
        style.icon_size = Some(crate::ZsFontSize::from_dip(size.0));
        self.metadata_mut().appearance = Some(Arc::new(style));
        self
    }
    /// Assigns an application vector or semantic icon to a standard icon/button.
    #[cfg(feature = "custom-icons")]
    pub fn icon_source(mut self, source: impl Into<crate::IconSource>) -> Self {
        let source = source.into();
        let symbol = match &source {
            crate::IconSource::Semantic(icon) => *icon,
            crate::IconSource::Vector(_) => crate::ZsIcon::Error,
        };
        match &mut self.kind {
            ViewNodeKind::Icon { icon, .. } => *icon = symbol,
            #[cfg(feature = "button")]
            ViewNodeKind::Button { presentation, .. } => match presentation {
                ZsButtonPresentation::Icon { icon }
                | ZsButtonPresentation::Toolbar { icon, .. }
                | ZsButtonPresentation::NavigationItem { icon, .. } => *icon = symbol,
                _ => {}
            },
            _ => {}
        }
        self.metadata_mut().icon_source = Some(source);
        self
    }
    #[cfg(feature = "button")]
    fn button_art_size_px(&self, dpi: Dpi) -> i32 {
        self.effective_appearance()
            .and_then(|style| style.icon_size)
            .map(|size| Dp::new(size.dip()))
            .unwrap_or_else(|| {
                ZsToolbarMetrics::for_platform(self.resolved_platform_style()).icon_size
            })
            .to_px(dpi)
            .round_i32()
            .max(1)
    }
    #[cfg(feature = "button")]
    fn button_leading_icon(&self) -> Option<crate::ZsIcon> {
        #[cfg(feature = "custom-icons")]
        if let Some(source) = &self.metadata().icon_source {
            return Some(match source {
                crate::IconSource::Semantic(icon) => *icon,
                crate::IconSource::Vector(_) => crate::ZsIcon::Error,
            });
        }
        None
    }
    #[cfg(feature = "button")]
    fn paint_button_label(
        &self,
        cx: &mut ViewPaintCx,
        bounds: Rect,
        label: &str,
        style: SemanticTextStyle,
    ) {
        let mut content = button_content_bounds(bounds, self.resolved_padding(), cx.dpi);
        if let Some(icon) = self.button_leading_icon() {
            let side = self
                .button_art_size_px(cx.dpi)
                .min(content.width.min(content.height).max(1));
            let icon_bounds = Rect {
                x: content.x,
                y: content.y + (content.height - side) / 2,
                width: side,
                height: side,
            };
            cx.draw(NativeDrawCommand::Icon(
                crate::NativeDrawIconCommand::new(
                    icon,
                    icon_bounds,
                    crate::NativeIconColorMode::ThemeAware,
                )
                .with_color(style.color),
            ));
            let gap = ZsToolbarMetrics::for_platform(self.resolved_platform_style())
                .content_gap
                .to_px(cx.dpi)
                .round_i32()
                .max(0);
            let offset = side.saturating_add(gap).min(content.width);
            content.x += offset;
            content.width -= offset;
        }
        cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
            label, content, style,
        )));
    }
    fn metadata(&self) -> &ViewMetadata {
        self.metadata.as_deref().unwrap_or(&ViewMetadata::EMPTY)
    }
    fn metadata_mut(&mut self) -> &mut ViewMetadata {
        self.metadata
            .get_or_insert_with(|| Box::new(ViewMetadata::default()))
    }
    /// Assigns a shared theme to this subtree while preserving native behavior.
    pub fn theme(mut self, theme: crate::AppTheme) -> Self {
        self.metadata_mut().theme = Some(theme);
        self
    }
    /// Overrides this element's appearance, after inherited theme values.
    pub fn appearance(mut self, appearance: crate::Appearance) -> Self {
        self.metadata_mut().appearance = Some(Arc::new(appearance));
        self
    }
    /// Assigns typography to this element and its descendants.
    pub fn font(mut self, font: crate::FontSpec) -> Self {
        let mut appearance = self
            .metadata()
            .theme
            .as_ref()
            .map(|theme| (*theme.0).clone())
            .unwrap_or_default();
        appearance.font = font;
        self.metadata_mut().theme = Some(crate::AppTheme::new(appearance));
        self
    }
    /// Adds application state without replacing native focus, press or hover.
    pub fn visual_state(mut self, state: crate::ControlState) -> Self {
        self.metadata_mut().state = state;
        self
    }
    /// Marks validation failure for the invalid appearance state.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.metadata_mut().state.invalid = invalid;
        self
    }
    fn effective_appearance(&self) -> Option<&Arc<crate::Appearance>> {
        self.metadata().effective.as_ref()
    }
    fn effective_font(&self) -> Option<Arc<crate::FontSpec>> {
        self.effective_appearance()
            .filter(|style| style.font != crate::FontSpec::default())
            .map(|style| Arc::new(style.font.clone()))
    }
    fn resolved_padding(&self) -> Option<[Dp; 4]> {
        self.style.resolved_padding().or_else(|| {
            self.effective_appearance()
                .and_then(|style| style.padding)
                .map(|sides| sides.map(|side| Dp::new(side.dip())))
        })
    }
    fn padding_extent_px(&self, dpi: Dpi, vertical: bool) -> i32 {
        let sides = self.resolved_padding().unwrap_or([Dp::new(0.0); 4]);
        let axis = usize::from(vertical);
        sides[axis]
            .to_px(dpi)
            .round_i32()
            .max(0)
            .saturating_add(sides[axis + 2].to_px(dpi).round_i32().max(0))
    }
    fn resolved_gap(&self) -> Option<Dp> {
        self.style.gap.or_else(|| {
            self.effective_appearance()
                .and_then(|style| style.gap)
                .map(|gap| Dp::new(gap.dip()))
        })
    }
    fn native_height_px(&self, value: Dp, dpi: Dpi, scale: f32) -> i32 {
        if !self.typography_scaled_height {
            return value.to_px(dpi).round_i32().max(0);
        }
        let mut height = value.0 * scale;
        let mut minimum_height = 0;
        if let Some(appearance) = self.effective_appearance() {
            let role = crate::TextRole::Body;
            #[cfg(feature = "label")]
            let role = if let ViewNodeKind::Text { style, .. } = &self.kind {
                style.role
            } else {
                role
            };
            let metrics = role.metrics_for(crate::ZsTypographyPlatformStyle::current());
            let (_, line) = appearance
                .font
                .resolved_metrics(metrics.size * scale, metrics.line_height * scale);
            if appearance.font.size.is_some() || appearance.font.line_height.is_some() {
                height = (height - metrics.line_height * scale).max(0.0) + line;
            }
            let is_label = matches!(&self.kind, ViewNodeKind::Spacer);
            #[cfg(feature = "label")]
            let is_label = is_label || matches!(&self.kind, ViewNodeKind::Text { .. });
            if !is_label {
                let density = match appearance.density {
                    Some(crate::ControlDensity::Compact) => 0.875,
                    Some(crate::ControlDensity::Touch) => 1.375,
                    _ => 1.0,
                };
                height = height.min(line) + (height - line).max(0.0) * density;
            }
            if appearance.font.size.is_some()
                || appearance.font.line_height.is_some()
                || appearance.density.is_some()
            {
                let padding = if self.resolved_padding().is_some() {
                    self.padding_extent_px(dpi, true)
                } else {
                    #[allow(unused_variables)]
                    let base = crate::ZsBaseControlMetrics::for_platform(
                        crate::ZsBaseControlPlatformStyle::current(),
                    );
                    let (top, bottom) = match self.kind {
                        #[cfg(feature = "button")]
                        ViewNodeKind::Button { .. } => {
                            (base.button_padding_top, base.button_padding_bottom)
                        }
                        #[cfg(feature = "textbox")]
                        ViewNodeKind::Textbox { .. } => {
                            (base.text_input_padding_top, base.text_input_padding_bottom)
                        }
                        #[cfg(feature = "password-box")]
                        ViewNodeKind::PasswordBox { .. } => {
                            (base.text_input_padding_top, base.text_input_padding_bottom)
                        }
                        #[cfg(feature = "number-box")]
                        ViewNodeKind::NumberBox { .. } => {
                            (base.text_input_padding_top, base.text_input_padding_bottom)
                        }
                        #[cfg(feature = "combo")]
                        ViewNodeKind::ComboBox { .. } => {
                            (base.text_input_padding_top, base.text_input_padding_bottom)
                        }
                        #[cfg(feature = "auto-suggest")]
                        ViewNodeKind::AutoSuggestBox { .. } => {
                            (base.text_input_padding_top, base.text_input_padding_bottom)
                        }
                        _ => (Dp::new(0.0), Dp::new(0.0)),
                    };
                    top.to_px(dpi)
                        .round_i32()
                        .max(0)
                        .saturating_add(bottom.to_px(dpi).round_i32().max(0))
                };
                minimum_height = Dp::new(line)
                    .to_px(dpi)
                    .round_i32()
                    .max(1)
                    .saturating_add(padding);
            }
        }
        Dp::new(height).to_px(dpi).round_i32().max(minimum_height)
    }
    pub(crate) fn widget_font(&self, widget: WidgetId) -> Option<Arc<crate::FontSpec>> {
        if self.id == Some(widget) {
            return self.effective_font();
        }
        self.children
            .iter()
            .find_map(|child| child.widget_font(widget))
    }
    pub(crate) fn widget_text_bounds(&self, widget: WidgetId) -> Option<(Rect, bool)> {
        #[cfg(feature = "textbox")]
        if self.id == Some(widget) {
            if let (Some(bounds), ViewNodeKind::Textbox { multiline, .. }) =
                (self.bounds, &self.kind)
            {
                if self.effective_font().is_some() || self.resolved_padding().is_some() {
                    return Some((
                        text_input_content_bounds(bounds, self.resolved_padding(), self.layout_dpi),
                        *multiline,
                    ));
                }
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_text_bounds(widget))
    }
    fn resolve_appearance_tree(&mut self, parent: Option<&Arc<crate::Appearance>>) {
        let inherited = match (&self.metadata().theme, parent) {
            (Some(theme), Some(parent)) => Some(Arc::new(parent.cascade(&theme.0))),
            (Some(theme), None) => Some(theme.0.clone()),
            (None, parent) => parent.cloned(),
        };
        let effective = match (&self.metadata().appearance, &inherited) {
            (Some(local), Some(parent)) => Some(Arc::new(parent.cascade(local))),
            (Some(local), None) => Some(local.clone()),
            (None, parent) => parent.clone(),
        };
        if self.metadata().effective != effective {
            self.metadata_mut().effective = effective;
        }
        for child in &mut self.children {
            child.resolve_appearance_tree(inherited.as_ref());
        }
    }
    fn appearance_state(&self) -> crate::ControlState {
        #[allow(unused_mut)]
        let mut state = self.metadata().state;
        match &self.kind {
            #[cfg(feature = "button")]
            ViewNodeKind::Button { enabled, .. } => state.disabled |= !*enabled,
            #[cfg(feature = "toggle-button")]
            ViewNodeKind::ToggleButton { checked, .. } => state.selected |= *checked,
            #[cfg(feature = "checkbox")]
            ViewNodeKind::Checkbox { checked, .. } => state.selected |= *checked,
            #[cfg(feature = "toggle")]
            ViewNodeKind::Toggle { checked, .. } => state.selected |= *checked,
            _ => {}
        }
        state
    }
}

impl ViewPaintCx {
    fn begin_appearance<Msg>(&mut self, node: &ViewNode<Msg>) {
        let mut appearance = node.effective_appearance().cloned();
        if node.style.background.is_some()
            || node.style.border.is_some()
            || node.style.radius.is_some()
        {
            let mut local = appearance
                .as_ref()
                .map(|style| (**style).clone())
                .unwrap_or_default();
            local.states.normal.background_role = node
                .style
                .background
                .map(color_role_for_token)
                .or(local.states.normal.background_role);
            local.states.normal.border_role = node
                .style
                .border
                .map(color_role_for_token)
                .or(local.states.normal.border_role);
            local.states.normal.radius = node
                .style
                .radius
                .map(|value| crate::ZsFontSize::from_dip(value.0))
                .or(local.states.normal.radius);
            appearance = Some(Arc::new(local));
        }
        #[cfg(feature = "custom-icons")]
        let appearance = appearance.or_else(|| {
            node.metadata()
                .icon_source
                .as_ref()
                .map(|_| Arc::new(crate::Appearance::default()))
        });
        let record = if let (Some(appearance), Some(bounds)) = (appearance, node.bounds) {
            #[allow(unused_mut)]
            let mut fixed_icon_slots = true;
            #[cfg(feature = "icon")]
            if matches!(node.kind, ViewNodeKind::Icon { .. }) {
                fixed_icon_slots = false;
            }
            let index = self.plan.appearance.len();
            self.plan
                .appearance
                .push(crate::appearance::PaintAppearance {
                    widget: node.id,
                    bounds,
                    fixed_icon_slots,
                    style: appearance.clone(),
                    state: node.appearance_state(),
                    commands: Vec::new(),
                    #[cfg(feature = "custom-icons")]
                    icon_source: node.metadata().icon_source.clone(),
                });
            Some(index)
        } else {
            None
        };
        self.appearance_stack.push(record);
    }
    fn end_appearance(&mut self) {
        self.appearance_stack.pop();
    }

    #[cfg(feature = "tabs")]
    fn custom_tabs<Msg>(
        &mut self,
        node: &ViewNode<Msg>,
        plan: &crate::ZsTabViewRenderPlan,
        tabs: &[crate::ZsTabSpec],
    ) -> bool {
        let Some(appearance) = node
            .effective_appearance()
            .filter(|style| {
                style
                    .tabs
                    .is_some_and(|tabs| tabs != crate::TabAppearance::Platform)
            })
            .cloned()
        else {
            return false;
        };
        let mode = appearance.tabs.unwrap();
        self.draw(NativeDrawCommand::PushClip {
            rect: plan.strip_bounds,
        });
        for (header, tab) in plan.headers.iter().zip(tabs) {
            let widget = node.id.map(|owner| tab.id.header_widget_id(owner));
            let index = self.plan.appearance.len();
            let state = crate::ControlState {
                selected: header.selected,
                ..node.appearance_state()
            };
            self.plan
                .appearance
                .push(crate::appearance::PaintAppearance {
                    widget,
                    bounds: header.bounds,
                    fixed_icon_slots: true,
                    style: appearance.clone(),
                    state,
                    commands: Vec::new(),
                    #[cfg(feature = "custom-icons")]
                    icon_source: None,
                });
            self.appearance_stack.push(Some(index));
            let filled = header.selected && mode != crate::TabAppearance::Underline;
            let fill = if filled {
                NativeDrawFill::Role(ColorRole::Accent)
            } else if mode == crate::TabAppearance::Segmented {
                NativeDrawFill::Role(ColorRole::Control)
            } else {
                NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Control,
                    alpha: 0,
                }
            };
            let radius = match mode {
                crate::TabAppearance::Pill => header.bounds.height / 2,
                crate::TabAppearance::Filled => Dp::new(4.0).to_px(self.dpi).round_i32(),
                _ => 0,
            };
            self.draw(NativeDrawCommand::RoundRect {
                rect: header.bounds,
                fill,
                stroke: None,
                radius,
            });
            if header.selected && mode == crate::TabAppearance::Underline {
                let thickness = Dp::new(2.0).to_px(self.dpi).round_i32().max(1);
                self.draw(NativeDrawCommand::FillRect {
                    rect: Rect {
                        x: header.bounds.x,
                        y: header.bounds.y + header.bounds.height - thickness,
                        width: header.bounds.width,
                        height: thickness,
                    },
                    fill: NativeDrawFill::Role(ColorRole::Accent),
                });
            }
            let role = if filled {
                ColorRole::AccentText
            } else {
                ColorRole::PrimaryText
            };
            if let (Some(icon), Some(bounds)) = (tab.icon, header.icon_bounds) {
                self.draw(NativeDrawCommand::Icon(
                    crate::NativeDrawIconCommand::new(
                        icon,
                        bounds,
                        crate::NativeIconColorMode::ThemeAware,
                    )
                    .with_color(role),
                ));
            }
            let mut style = crate::SemanticTextStyle::for_role(crate::TextRole::Button);
            style.color = role;
            style.horizontal_align = crate::HorizontalAlign::Center;
            self.draw(NativeDrawCommand::Text(crate::NativeDrawTextCommand::new(
                &tab.label,
                header.text_bounds,
                style,
            )));
            self.appearance_stack.pop();
        }
        self.draw(NativeDrawCommand::PopClip);
        true
    }
}
