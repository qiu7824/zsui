#[path = "authoring/cases.rs"]
mod cases;
use zsui::{row, spacer, Dp, Dpi, Rect, View, ViewLayoutCx};

fn surface(width: i32, height: i32) -> Rect {
    Rect {
        x: 0,
        y: 0,
        width,
        height,
    }
}

#[test]
fn capped_flex_returns_unused_space_to_its_sibling() {
    let mut root = row::<()>([
        spacer().flex(1.0).max_width(Dp::new(40.0)),
        spacer().flex(1.0),
    ]);
    root.layout(&mut ViewLayoutCx::new(surface(150, 20), Dpi::standard()));
    assert_eq!(root.children[0].bounds().unwrap().width, 40);
    assert_eq!(
        root.children[1].bounds().unwrap(),
        Rect {
            x: 40,
            ..surface(110, 20)
        }
    );
}

#[cfg(feature = "grid")]
#[test]
fn capped_grid_child_leaves_room_for_the_next_track() {
    use zsui::{grid, ZsGridCell, ZsGridTrack};
    let mut root = grid(
        [ZsGridTrack::FLEX; 2],
        [ZsGridTrack::FLEX],
        [
            ZsGridCell::new(
                0,
                0,
                spacer::<()>()
                    .width(Dp::new(800.0))
                    .max_width(Dp::new(40.0))
                    .max_height(Dp::new(0.0)),
            ),
            ZsGridCell::new(0, 1, spacer()),
        ],
    );
    root.layout(&mut ViewLayoutCx::new(surface(200, 80), Dpi::standard()));
    assert_eq!(root.children[0].bounds(), Some(surface(40, 0)));
    assert_eq!(
        root.children[1].bounds(),
        Some(Rect {
            x: 100,
            ..surface(100, 80)
        })
    );
}

#[cfg(all(
    feature = "ui-document-runtime",
    feature = "button",
    feature = "textbox"
))]
#[test]
fn bounded_form_matches_rust_geometry_and_delivers_save() {
    use zsui::ui_document::{UiBindingSchema, UiDocument, UiValueType};
    use zsui::ui_document_runtime::ui_document_view;
    let document = UiDocument::from_json(include_str!("authoring/name-form.json")).unwrap();
    let mut bindings = UiBindingSchema::default();
    bindings.actions.insert("save".into(), UiValueType::Null);
    let mut form =
        ui_document_view(&document, &bindings, &Default::default(), |action| action).unwrap();
    let mut rust = cases::name_form();
    for scale in [1, 2, 3] {
        let dpi = Dpi::new(48.0 * (scale + 1) as f32);
        for (width, height) in [(320, 160), (640, 240), (960, 480)] {
            let pixels = |dp: i32| dp * (scale + 1) / 2;
            let bounds = surface(pixels(width), pixels(height));
            let actual = form.layout(&mut ViewLayoutCx::new(bounds, dpi));
            let expected = rust.layout(&mut ViewLayoutCx::new(bounds, dpi));
            assert_eq!(
                actual.bounds,
                surface(pixels(width.min(640)), pixels(height.min(240)))
            );
            assert_eq!(actual.bounds, expected.bounds);
            assert_eq!(rust.children.len(), 2);
            assert_eq!(form.children.len(), 2);
            for (a, b) in form.children.iter().zip(&rust.children) {
                assert_eq!(a.bounds(), b.bounds());
            }
        }
    }
    let mut events = zsui::ViewEventCx::new();
    form.event(
        &mut events,
        &zsui::ViewEvent::Click {
            widget: document.root.children[1].id.widget_id(),
        },
    );
    let messages = events.into_messages();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].binding, "save");
    let mut rust_events = zsui::ViewEventCx::new();
    rust.event(
        &mut rust_events,
        &zsui::ViewEvent::Click {
            widget: zsui::WidgetId(2),
        },
    );
    assert_eq!(rust_events.into_messages(), ["save"]);
}

#[cfg(feature = "canvas")]
#[test]
fn custom_panel_paints_half_of_its_final_width() {
    use zsui::{ColorRole, NativeDrawCommand, NativeDrawFill, ViewPaintCx};
    let mut panel = cases::custom_panel();
    for width in [160, 480] {
        let dpi = Dpi::standard();
        panel.layout(&mut ViewLayoutCx::new(surface(width, 64), dpi));
        let mut paint = ViewPaintCx::new(dpi);
        panel.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|c| matches!(c,
            NativeDrawCommand::FillRect { rect, fill: NativeDrawFill::Role(ColorRole::Accent) } if rect.x == 0 && rect.y == 0 && rect.width == width.min(320) / 2 && rect.height == 8)));
    }
}

#[cfg(all(feature = "virtual-list", feature = "label"))]
#[test]
fn virtual_records_keep_only_the_requested_rows() {
    use zsui::WidgetId;
    let mut list = cases::records();
    list.layout(&mut ViewLayoutCx::new(surface(160, 80), Dpi::standard()));
    assert_eq!(list.children.len(), 4);
    let mut events = zsui::ViewEventCx::new();
    list.event(
        &mut events,
        &zsui::ViewEvent::Click {
            widget: WidgetId(1012),
        },
    );
    assert_eq!(events.into_messages(), [12]);
}
