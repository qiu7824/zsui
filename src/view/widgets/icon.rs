/// Creates a noninteractive standalone semantic icon.
///
/// The application chooses only the semantic symbol, size role and color
/// role. Each desktop renderer resolves the actual WinUI glyph, SF Symbol or
/// Linux symbolic icon and the platform profile owns its logical dimensions.
pub fn icon<Msg>(icon: crate::ZsIcon) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Icon {
        icon,
        size: crate::ZsIconSize::Standard,
        color: ColorRole::PrimaryText,
    })
    .flex(0.0)
}

impl<Msg> ViewNode<Msg> {
    /// Selects a semantic icon size without exposing target pixel constants.
    pub fn icon_size(mut self, size: crate::ZsIconSize) -> Self {
        if let ViewNodeKind::Icon { size: current, .. } = &mut self.kind {
            *current = size;
        }
        self
    }

    /// Selects a theme-aware semantic color for a standalone icon.
    pub fn icon_color(mut self, color: ColorRole) -> Self {
        if let ViewNodeKind::Icon { color: current, .. } = &mut self.kind {
            *current = color;
        }
        self
    }
}
