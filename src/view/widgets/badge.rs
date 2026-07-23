/// Creates a compact, noninteractive semantic information badge.
///
/// The application chooses dot, numeric or semantic-icon content plus a tone.
/// Each platform profile owns the final dimensions, typography and native
/// color resolution. Accessibility announcements belong to the badge's
/// focusable parent because the badge itself is not an action target.
pub fn badge<Msg>(content: crate::ZsBadgeContent) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Badge {
        content,
        tone: crate::ZsBadgeTone::default(),
    })
    .flex(0.0)
}

impl<Msg> ViewNode<Msg> {
    /// Selects a semantic badge tone without embedding a target palette.
    pub fn badge_tone(mut self, tone: crate::ZsBadgeTone) -> Self {
        if let ViewNodeKind::Badge { tone: current, .. } = &mut self.kind {
            *current = tone;
        }
        self
    }
}
