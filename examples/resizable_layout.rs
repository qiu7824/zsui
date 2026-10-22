#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{
    env, fs,
    io::Cursor,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use zsui::{
    column, data_grid, icon, native_window, navigation_item, navigation_view, row, split_view,
    styled_text, text, ColorRole, Dp, NativeWindowSmokeRunOptions, SemanticTextStyle, TextRole,
    ViewHitTargetKind, ViewNode, WidgetId, ZsIcon, ZsIconRole, ZsNavigationViewSpec, ZsResizeAxis,
    ZsResizeLimits, ZsSplitViewDisplayMode, ZsSplitViewPanePlacement, ZsSplitViewSpec,
    ZsTableColumn, ZsTableColumnResize, ZsTableColumnWidth, ZsTableRow, ZsTableSort, ZsuiThemeMode,
};

const SIDEBAR: WidgetId = WidgetId::new(901);
const PANEL: WidgetId = WidgetId::new(902);
const VERTICAL_PANEL: WidgetId = WidgetId::new(903);

fn capture_has_visible_content(bytes: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let size = reader
        .output_buffer_size()
        .ok_or("PNG capture is too large")?;
    if size > 128 * 1024 * 1024 {
        return Err("PNG capture exceeds the proof limit".into());
    }
    let mut pixels = vec![0; size];
    let info = reader.next_frame(&mut pixels)?;
    let channels = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        _ => return Err("native capture must contain RGB pixels".into()),
    };
    // Alpha variation alone is not visible content: black glyphs over an
    // unpainted transparent surface otherwise pass a byte-variation check.
    let mut first = None;
    let mut different = 0;
    for pixel in pixels[..info.buffer_size()].chunks_exact(channels) {
        if channels == 4 && pixel[3] == 0 {
            continue;
        }
        let rgb = [pixel[0], pixel[1], pixel[2]];
        if let Some(background) = first {
            if rgb != background {
                different += 1;
            }
        } else {
            first = Some(rgb);
        }
    }
    Ok(different >= 64)
}
const TABLE: WidgetId = WidgetId::new(904);

#[derive(Clone)]
enum Message {
    Sidebar(Dp),
    Panel(Dp),
    Vertical(Dp),
    Column(ZsTableColumnResize),
    Open(bool),
    Page(usize),
    Sort(ZsTableSort),
}

struct State {
    sidebar: Dp,
    panel: Dp,
    vertical: Dp,
    open: bool,
    page: usize,
    columns: Vec<ZsTableColumn>,
    changes: usize,
    sort_requests: usize,
}

fn limits(minimum: f32, maximum: f32) -> ZsResizeLimits {
    ZsResizeLimits::new(Dp::new(minimum), Dp::new(maximum))
        .expect("constant positive layout limits")
}

fn status_icons() -> ViewNode<Message> {
    row([
        icon(ZsIcon::Info)
            .icon_role(ZsIconRole::Status)
            .icon_color(ColorRole::Success),
        icon(ZsIcon::Info)
            .icon_role(ZsIconRole::Status)
            .icon_color(ColorRole::Warning),
        icon(ZsIcon::Info)
            .icon_role(ZsIconRole::Status)
            .icon_color(ColorRole::Danger),
    ])
    .gap(Dp::new(16.0))
    .height(Dp::new(32.0))
}

fn view(scene: &str, state: &State) -> ViewNode<Message> {
    match scene {
        "navigation" => navigation_view(
            ZsNavigationViewSpec::new("Navigation", "Resizable sidebar")
                .pane_width(state.sidebar)
                .pane_open(state.open)
                .minimum_content_width(Dp::new(320.0))
                .item(
                    navigation_item("Overview", ZsIcon::App, state.page == 0)
                        .on_click(Message::Page(0)),
                )
                .item(
                    navigation_item("Details", ZsIcon::File, state.page == 1)
                        .on_click(Message::Page(1)),
                )
                .content(
                    SIDEBAR,
                    column([text("Content"), status_icons()]).padding(Dp::new(24.0)),
                ),
        )
        .pane_resize_limits(limits(100.0, 400.0))
        .on_pane_resize(Message::Sidebar)
        .on_navigation_open_change(Message::Open)
        .accessibility_label("Sidebar"),
        "panels" => split_view(
            PANEL,
            ZsSplitViewSpec::new(true)
                .display_mode(ZsSplitViewDisplayMode::Inline)
                .pane_placement(ZsSplitViewPanePlacement::Trailing)
                .pane_extent(state.panel)
                .minimum_content_extent(Dp::new(320.0)),
            column([text("Side panel"), status_icons()]).padding(Dp::new(16.0)),
            split_view(
                VERTICAL_PANEL,
                ZsSplitViewSpec::new(true)
                    .display_mode(ZsSplitViewDisplayMode::Inline)
                    .axis(ZsResizeAxis::Vertical)
                    .pane_extent(state.vertical)
                    .minimum_content_extent(Dp::new(160.0)),
                column([text("Top panel"), status_icons()]).padding(Dp::new(16.0)),
                column([text("Bottom panel")]).padding(Dp::new(16.0)),
            )
            .pane_resize_limits(limits(160.0, 420.0))
            .on_pane_resize(Message::Vertical)
            .accessibility_label("Top panel"),
        )
        .pane_resize_limits(limits(100.0, 360.0))
        .on_pane_resize(Message::Panel)
        .accessibility_label("Side panel"),
        "table" => column([
            text("Resizable columns"),
            status_icons(),
            data_grid(
                state.columns.clone(),
                [
                    ZsTableRow::new(1, ["Report", "Document", "24 KB"]),
                    ZsTableRow::new(2, ["Diagram", "Image", "160 KB"]),
                ],
            )
            .id(TABLE)
            .on_table_column_resize(Message::Column)
            .on_table_sort(Message::Sort)
            .flex(1.0),
        ])
        .padding(Dp::new(20.0))
        .gap(Dp::new(12.0)),
        "icons" => {
            let mut rows = vec![text("Semantic icons"), status_icons()];
            for chunk in ZsIcon::ALL.chunks(9) {
                rows.push(
                    row(chunk.iter().map(|value| {
                        column([
                            icon(*value).icon_role(ZsIconRole::Large),
                            styled_text(
                                format!("{value:?}"),
                                SemanticTextStyle {
                                    role: TextRole::Caption,
                                    ellipsis: true,
                                    ..SemanticTextStyle::body()
                                },
                            ),
                        ])
                        .width(Dp::new(88.0))
                        .height(Dp::new(72.0))
                        .gap(Dp::new(4.0))
                    }))
                    .gap(Dp::new(8.0)),
                );
            }
            column(rows).padding(Dp::new(16.0)).gap(Dp::new(8.0))
        }
        _ => unreachable!(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let argument = |name: &str| {
        args.windows(2)
            .find(|pair| pair[0] == name)
            .map(|pair| pair[1].clone())
    };
    let scene = argument("--scene").unwrap_or_else(|| "panels".to_owned());
    if !["navigation", "panels", "table", "icons"].contains(&scene.as_str()) {
        return Err("invalid scene".into());
    }
    let theme = match argument("--theme").as_deref().unwrap_or("system") {
        "system" => ZsuiThemeMode::System,
        "light" => ZsuiThemeMode::Light,
        "dark" => ZsuiThemeMode::Dark,
        "high-contrast" => ZsuiThemeMode::HighContrast,
        _ => return Err("invalid theme".into()),
    };
    let state = Arc::new(Mutex::new(State {
        sidebar: Dp::new(240.0),
        panel: Dp::new(180.0),
        vertical: Dp::new(280.0),
        open: true,
        page: 0,
        columns: vec![
            ZsTableColumn::new(1, "Name")
                .fixed_width(Dp::new(180.0))
                .sortable(true),
            ZsTableColumn::new(2, "Type")
                .fixed_width(Dp::new(140.0))
                .sortable(true),
            ZsTableColumn::new(3, "Size")
                .fixed_width(Dp::new(120.0))
                .sortable(true),
        ],
        changes: 0,
        sort_requests: 0,
    }));
    let view_scene = scene.clone();
    let builder = native_window("ZSUI Layout Controls")
        .size(960, 640)
        .stateful_view(
            state.clone(),
            move |state| view(&view_scene, &state.lock().unwrap()).theme_mode(theme),
            |state, message, _cx| {
                let mut state = state.lock().unwrap();
                match message {
                    Message::Sidebar(value) => {
                        state.sidebar = value;
                        state.changes += 1;
                    }
                    Message::Panel(value) => {
                        state.panel = value;
                        state.changes += 1;
                    }
                    Message::Vertical(value) => {
                        state.vertical = value;
                        state.changes += 1;
                    }
                    Message::Column(change) => {
                        if let Some(column) = state
                            .columns
                            .iter_mut()
                            .find(|column| column.id() == change.column)
                        {
                            column.resize_to(change.width);
                            state.changes += 1;
                        }
                    }
                    Message::Open(open) => state.open = open,
                    Message::Page(page) => state.page = page,
                    Message::Sort(sort) => {
                        let _ = sort;
                        state.sort_requests += 1;
                    }
                }
            },
        );
    if !args.iter().any(|arg| arg == "--smoke") {
        builder.run()?;
        return Ok(());
    }
    let output = PathBuf::from(argument("--output").ok_or("--smoke requires --output")?);
    fs::create_dir_all(&output)?;
    let mut handles = builder
        .native_view_interaction_plan()
        .unwrap()
        .hit_targets
        .iter()
        .filter(|target| matches!(target.kind, ViewHitTargetKind::ResizeHandle(_)))
        .map(|target| target.widget)
        .collect::<Vec<_>>();
    handles.sort_by_key(|id| id.0);
    let expected = match scene.as_str() {
        "navigation" => 1,
        "panels" => 2,
        "table" => 3,
        _ => 0,
    };
    assert_eq!(handles.len(), expected);
    let duration = argument("--duration-ms")
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(1400);
    let mut options = NativeWindowSmokeRunOptions::new(duration)
        .screenshot_file(output.join("window.png").to_string_lossy())
        .require_screenshot(true);
    let external_input = args.iter().any(|arg| arg == "--external-input");
    if !external_input {
        for handle in handles {
            options = options.native_view_drag_widget(handle);
        }
    }
    let report = builder.run_smoke(options)?;
    assert!(
        capture_has_visible_content(&fs::read(output.join("window.png"))?)?,
        "native scene capture is blank or lacks visible RGB content"
    );
    let state = state.lock().unwrap();
    assert!(report.visible_window_was_created() && report.screenshot_captured);
    if !external_input {
        assert_eq!(state.changes, expected);
        assert_eq!(state.sort_requests, 0);
        match scene.as_str() {
            "navigation" => assert!((state.sidebar.0 - 304.0).abs() < 1.0),
            "panels" => {
                assert!((state.panel.0 - 244.0).abs() < 1.0);
                assert!((state.vertical.0 - 344.0).abs() < 1.0);
            }
            "table" => {
                for (column, expected) in state.columns.iter().zip([244.0, 204.0, 184.0]) {
                    assert!(
                        matches!(column.width(), ZsTableColumnWidth::Fixed(width) if (width.0-expected).abs()<1.0)
                    );
                }
            }
            _ => {}
        }
    }
    fs::write(
        output.join("proof.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "scene": scene, "theme": theme, "resize_messages": state.changes, "sort_requests": state.sort_requests,
            "sidebar_width": state.sidebar.0, "panel_width": state.panel.0, "panel_height": state.vertical.0,
            "columns": state.columns, "runtime": report, "screenshot_nonblank": true,
        }))?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba_png(pixels: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, 16, 16);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(pixels)
            .unwrap();
        bytes
    }

    #[test]
    fn transparent_black_glyphs_are_not_accepted_as_visible_capture() {
        let pixels: Vec<_> = (0..256).flat_map(|alpha| [0, 0, 0, alpha as u8]).collect();
        assert!(!capture_has_visible_content(&rgba_png(&pixels)).unwrap());
    }

    #[test]
    fn opaque_surface_and_foreground_are_visible_capture() {
        let pixels: Vec<_> = (0..256)
            .flat_map(|index| {
                let color = if index < 128 { 255 } else { 0 };
                [color, color, color, 255]
            })
            .collect();
        assert!(capture_has_visible_content(&rgba_png(&pixels)).unwrap());
    }
}
