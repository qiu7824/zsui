impl ZsRustTextEngine {
    fn ellipsized_layout(
        &mut self,
        text: &str,
        style: &TextStyle,
        width: i32,
        height: i32,
        scale: f32,
    ) -> Arc<ZsRustTextLayout> {
        use unicode_segmentation::UnicodeSegmentation;
        struct SourceSpan {
            rendered: Range<usize>,
            source: Range<usize>,
            marker: bool,
        }
        let mut rendered = String::new();
        let mut mapping = Vec::new();
        let mut source_start = 0;
        for line in text.split_inclusive('\n') {
            let content = line.trim_end_matches(['\r', '\n']);
            let ending = &line[content.len()..];
            let unbounded = self.layout(content, style, 0, height, scale);
            let start = rendered.len();
            if unbounded.size().width <= width {
                rendered.push_str(content);
                mapping.push(SourceSpan {
                    rendered: start..rendered.len(),
                    source: source_start..source_start + content.len(),
                    marker: false,
                });
            } else {
                let marker = self.layout("…", style, 0, height, scale);
                if marker.size().width <= width {
                    let boundaries = content
                        .grapheme_indices(true)
                        .map(|(index, _)| index)
                        .chain(std::iter::once(content.len()))
                        .collect::<Vec<_>>();
                    let mut low = 0;
                    let mut high = boundaries.len() - 1;
                    while low < high {
                        let middle = (low + high + 1) / 2;
                        let candidate = format!("{}…", &content[..boundaries[middle]]);
                        if self
                            .layout(&candidate, style, 0, height, scale)
                            .size()
                            .width
                            <= width
                        {
                            low = middle;
                        } else {
                            high = middle - 1;
                        }
                    }
                    let kept = boundaries[low];
                    rendered.push_str(&content[..kept]);
                    mapping.push(SourceSpan {
                        rendered: start..rendered.len(),
                        source: source_start..source_start + kept,
                        marker: false,
                    });
                    let marker_start = rendered.len();
                    rendered.push('…');
                    mapping.push(SourceSpan {
                        rendered: marker_start..rendered.len(),
                        source: source_start + kept..source_start + content.len(),
                        marker: true,
                    });
                }
            }
            let start = rendered.len();
            rendered.push_str(ending);
            mapping.push(SourceSpan {
                rendered: start..rendered.len(),
                source: source_start + content.len()..source_start + line.len(),
                marker: false,
            });
            source_start += line.len();
        }
        let layout = self.layout(&rendered, style, width, height, scale);
        let mut result = (*layout).clone();
        let remap = |start: usize, end: usize| {
            let first = mapping
                .iter()
                .find(|span| span.rendered.start <= start && start < span.rendered.end);
            let last = mapping
                .iter()
                .rev()
                .find(|span| span.rendered.start < end && end <= span.rendered.end);
            (
                first.map_or(text.len(), |span| {
                    if span.marker {
                        span.source.start
                    } else {
                        span.source.start + start - span.rendered.start
                    }
                }),
                last.map_or(text.len(), |span| {
                    if span.marker {
                        span.source.end
                    } else {
                        span.source.start + end - span.rendered.start
                    }
                }),
            )
        };
        for glyph in &mut result.glyphs {
            (glyph.cluster_start, glyph.cluster_end) =
                remap(glyph.cluster_start, glyph.cluster_end);
        }
        #[cfg(feature = "rust-text-proof")]
        for line in &mut result.proof.lines {
            for glyph in &mut line.glyphs {
                (glyph.cluster_start, glyph.cluster_end) =
                    remap(glyph.cluster_start, glyph.cluster_end);
            }
        }
        Arc::new(result)
    }
}

#[cfg(test)]
mod ellipsis_tests {
    use super::*;
    #[test]
    fn truncation_uses_grapheme_edges_and_keeps_the_rust_backend() {
        use unicode_segmentation::UnicodeSegmentation;
        let mut engine = ZsRustTextEngine::new();
        for text in [
            "A中文123 and a long label",
            "e\u{301}e\u{301} 👨‍👩‍👧‍👦 long label",
            "مرحبا بالعالم نص طويل",
            "first line overflow\nsecond line overflow",
        ] {
            let style = TextStyle::line("sans-serif", 14.0, Color::rgb(0, 0, 0));
            let layout = engine.layout(text, &style, 70, 70, 1.0);
            assert!(layout.size().width <= 70);
            let boundaries = text
                .grapheme_indices(true)
                .map(|(index, _)| index)
                .chain(std::iter::once(text.len()))
                .collect::<Vec<_>>();
            for glyph in layout.glyphs() {
                assert!(boundaries.contains(&glyph.cluster_start));
                assert!(boundaries.contains(&glyph.cluster_end));
            }
            assert!(Arc::ptr_eq(
                &layout,
                &engine.layout(text, &style, 70, 70, 1.0)
            ));
        }
    }
    #[test]
    fn text_that_fits_preserves_identical_face_geometry() {
        let mut engine = ZsRustTextEngine::new();
        let mut style = TextStyle::line("sans-serif", 14.0, Color::rgb(0, 0, 0));
        let clipped = engine.layout("Test中文", &style, 300, 40, 1.25);
        style.ellipsis = false;
        let plain = engine.layout("Test中文", &style, 300, 40, 1.25);
        assert!(Arc::ptr_eq(&clipped, &plain));
    }
}
