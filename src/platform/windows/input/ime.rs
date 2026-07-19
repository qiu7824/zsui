impl WindowsWin32ViewInputRoute {
    fn dispatch_text_input(&mut self, text: &str) -> WindowsWin32ViewInputDispatchReport {
        self.dispatch_text_input_at(text, std::time::Instant::now())
    }

    fn dispatch_text_input_at(
        &mut self,
        text: &str,
        now: std::time::Instant,
    ) -> WindowsWin32ViewInputDispatchReport {
        let target = self.shared_focused_target();
        let accepted = accepted_windows_text_input_count(text, target);
        let report = self.shared_runtime.dispatch_text_input_at(text, now);
        self.adapt_shared_report(
            report,
            WindowsSharedInputKind::Text { accepted, target },
        )
    }

    fn dispatch_utf16_input_unit(&mut self, unit: u16) -> WindowsWin32ViewInputDispatchReport {
        if (0xd800..=0xdbff).contains(&unit) {
            self.pending_utf16_high_surrogate = Some(unit);
            return WindowsWin32ViewInputDispatchReport {
                handled: true,
                hit_target_count: self.hit_target_count(),
                events: vec!["win32_view_text_utf16_high_surrogate".to_string()],
                ..WindowsWin32ViewInputDispatchReport::default()
            };
        }
        let text = if (0xdc00..=0xdfff).contains(&unit) {
            self.pending_utf16_high_surrogate
                .take()
                .and_then(|high| char::decode_utf16([high, unit]).next())
                .and_then(Result::ok)
                .map(|character| character.to_string())
        } else {
            self.pending_utf16_high_surrogate = None;
            text_from_char_wparam(unit as usize)
        };
        match text {
            Some(text) => self.dispatch_text_input(&text),
            None => WindowsWin32ViewInputDispatchReport {
                hit_target_count: self.hit_target_count(),
                events: vec!["win32_view_text_invalid_utf16_unit".to_string()],
                ..WindowsWin32ViewInputDispatchReport::default()
            },
        }
    }
}

fn accepted_windows_text_input_count(
    text: &str,
    target: Option<crate::ViewHitTarget>,
) -> usize {
    let multiline = target.is_some_and(|target| target.kind == crate::ViewHitTargetKind::TextEditor);
    let mut previous_was_carriage_return = false;
    text.chars()
        .filter(|character| {
            let accepted = matches!(*character, '\u{8}' | '\u{7f}')
                || (multiline
                    && (*character == '\r'
                        || (*character == '\n' && !previous_was_carriage_return)))
                || !character.is_control();
            previous_was_carriage_return = *character == '\r';
            accepted
        })
        .count()
}
