//! Offscreen appearance and icon metrics. This executable creates no UI windows.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{fs, io::BufWriter, path::PathBuf};
    use zsui::windows_gdi_renderer::{render_windows_offscreen, WindowsOffscreenFrame};
    use zsui::{
        AppTheme, Appearance, Color, ColorRole, ControlStyle, Dp, Dpi, FocusAppearance, FontSpec,
        NativeDrawCommand as Draw, NativeDrawIconCommand, NativeDrawPlan, NativeIconColorMode,
        Rect, StateStyles, TabAppearance, View, ViewLayoutCx, ViewPaintCx, ZsIcon,
    };
    let root = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("provide an absolute output directory")?,
    );
    if !root.is_absolute() {
        return Err("output directory must be absolute".into());
    }
    fs::create_dir_all(&root)?;
    fn save(
        path: &std::path::Path,
        frame: &WindowsOffscreenFrame,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut encoder = png::Encoder::new(
            BufWriter::new(fs::File::create(path)?),
            frame.width,
            frame.height,
        );
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&frame.rgba)?;
        Ok(())
    }
    fn ink(frame: &WindowsOffscreenFrame, rect: Rect) -> serde_json::Value {
        let mut left = rect.width;
        let mut top = rect.height;
        let mut right = 0;
        let mut bottom = 0;
        let mut count = 0u64;
        let mut sx = 0u64;
        let mut sy = 0u64;
        for y in 0..rect.height {
            for x in 0..rect.width {
                let index =
                    (((rect.y + y) as u32 * frame.width + (rect.x + x) as u32) * 4) as usize;
                let pixel = &frame.rgba[index..index + 3];
                if pixel.iter().any(|channel| channel.abs_diff(243) > 10) {
                    left = left.min(x);
                    top = top.min(y);
                    right = right.max(x + 1);
                    bottom = bottom.max(y + 1);
                    count += 1;
                    sx += x as u64;
                    sy += y as u64;
                }
            }
        }
        serde_json::json!({"left":left,"top":top,"width":(right-left).max(0),"height":(bottom-top).max(0),"ink_pixels":count,"centroid_x":if count>0{sx as f64/count as f64}else{0.0},"centroid_y":if count>0{sy as f64/count as f64}else{0.0}})
    }
    let mut metrics = Vec::new();
    for scale in [1.0_f32, 1.25, 1.5, 2.0] {
        let dpi = Dpi::new(96.0 * scale);
        let slot = (48.0 * scale).round() as i32;
        let width = slot * 5;
        let height = slot * ZsIcon::ALL.len() as i32;
        for source in ["system", "symbolic", "svg"] {
            let mut commands = Vec::new();
            let mut cells = Vec::new();
            for (row, icon) in ZsIcon::ALL.into_iter().enumerate() {
                for (column, size) in [16, 18, 20, 24, 32].into_iter().enumerate() {
                    let side = (size as f32 * scale).round() as i32;
                    let rect = Rect {
                        x: column as i32 * slot + (slot - side) / 2,
                        y: row as i32 * slot + (slot - side) / 2,
                        width: side,
                        height: side,
                    };
                    if source == "svg" {
                        let asset = zsui::IconAsset::from_svg(
                            zsui::IconAssetId(row as u64),
                            std::str::from_utf8(icon.fluent_svg_bytes())?,
                            "Microsoft Fluent UI System Icons / MIT",
                        )?;
                        commands.push(Draw::VectorIcon(zsui::NativeDrawVectorIcon {
                            asset,
                            bounds: rect,
                            color: ColorRole::PrimaryText,
                            custom_color: None,
                            color_mode: NativeIconColorMode::ThemeAware,
                        }));
                    } else {
                        commands.push(Draw::Icon(NativeDrawIconCommand::new(
                            icon,
                            rect,
                            NativeIconColorMode::ThemeAware,
                        )));
                    }
                    cells.push((format!("{icon:?}"), size, rect));
                }
            }
            let frame = render_windows_offscreen(
                &NativeDrawPlan::new(commands),
                width as u32,
                height as u32,
                dpi,
                source == "symbolic",
            )?;
            let filename = format!("icons-{source}-{}.png", (scale * 100.0) as u32);
            save(&root.join(&filename), &frame)?;
            for (index, (icon, size, rect)) in cells.into_iter().enumerate() {
                let cell = Rect {
                    x: (index % 5) as i32 * slot,
                    y: (index / 5) as i32 * slot,
                    width: slot,
                    height: slot,
                };
                let allocation_ink = ink(&frame, rect);
                let cell_ink = ink(&frame, cell);
                if allocation_ink["ink_pixels"].as_u64() == Some(0)
                    || allocation_ink["ink_pixels"] != cell_ink["ink_pixels"]
                {
                    return Err(format!(
                        "{icon} ({source}, {size} DP, {scale}x) is blank or exceeds its allocation"
                    )
                    .into());
                }
                metrics.push(serde_json::json!({"icon":icon,"logical_em_or_viewbox":size,"scale":scale,"requested_source":source,"actual_source":frame.icon_sources.get(index),"allocated_bounds":rect,"ink_inside_allocation":allocation_ink,"ink_inside_cell":cell_ink,"artifact":filename}));
            }
        }
    }
    let mut styles = Vec::new();
    let custom = zsui::IconAsset::from_svg(
        zsui::IconAssetId(900),
        std::str::from_utf8(ZsIcon::Monitor.fluent_svg_bytes())?,
        "Microsoft Fluent UI System Icons / MIT",
    )?;
    for (name, tabs, color) in [
        ("native", TabAppearance::Platform, Color::rgb(0, 103, 192)),
        ("violet", TabAppearance::Pill, Color::rgb(120, 65, 185)),
        ("green", TabAppearance::Segmented, Color::rgb(18, 126, 92)),
    ] {
        let appearance = Appearance::default()
            .color(ColorRole::Accent, color)
            .tabs(tabs)
            .focus(FocusAppearance::Outline)
            .font(
                FontSpec::family("Microsoft YaHei UI")
                    .fallback("Segoe UI Emoji")
                    .size(Dp::new(14.0))
                    .line_height(Dp::new(22.0)),
            );
        let form = zsui::column([
            zsui::text::<()>("中文 English 123 🙂 — Visual styles"),
            zsui::textbox("Controlled text / 受控文本")
                .id(zsui::WidgetId(21))
                .width(Dp::new(600.0)),
            zsui::row([
                zsui::button("Normal"),
                zsui::button("Hovered").visual_state(zsui::ControlState {
                    hover: true,
                    ..Default::default()
                }),
                zsui::button("Disabled").enabled(false),
            ])
            .gap(Dp::new(12.0)),
            zsui::row([16.0, 20.0, 32.0].map(|size| {
                zsui::button(format!("Icon {size:.0} / 图标"))
                    .icon_source(custom.clone())
                    .glyph_size(Dp::new(size))
                    .height(Dp::new(52.0))
            }))
            .gap(Dp::new(12.0)),
        ])
        .gap(Dp::new(14.0))
        .padding(Dp::new(16.0));
        let mut view = zsui::tab_view(
            [
                zsui::ZsTabItem::new(zsui::ZsTabId(1), "Settings", form),
                zsui::ZsTabItem::new(zsui::ZsTabId(2), "Details", zsui::spacer()),
            ],
            Some(zsui::ZsTabId(1)),
        )
        .id(zsui::WidgetId(20))
        .theme(if name == "native" {
            AppTheme::default()
        } else {
            AppTheme::new(appearance.states(StateStyles {
                normal: ControlStyle::default().radius(Dp::new(8.0)),
                hover: ControlStyle::default().background(Color::rgb(224, 217, 239)),
                disabled: ControlStyle::default().foreground(Color::rgb(140, 140, 140)),
                ..Default::default()
            }))
        });
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 720,
                height: 340,
            },
            Dpi::standard(),
        ));
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        let plan = paint.into_plan();
        let frame = render_windows_offscreen(&plan, 720, 340, Dpi::standard(), false)?;
        let file = format!("controls-{name}.png");
        save(&root.join(&file), &frame)?;
        styles
            .push(serde_json::json!({"name":name,"commands":plan.commands.len(),"artifact":file}));
    }
    let report = serde_json::json!({"schema":"zsui.appearance-offscreen/v1","platform":"windows","architecture":std::env::consts::ARCH,"native_windows_created":0,"input_injection":false,"icon_count":ZsIcon::ALL.len(),"icon_measurements":metrics,"controls":styles,"memory":zsui::NativeProofProcessMemoryEvidence::capture(),"measurement":"native memory-DC pixels; em/viewbox and visible ink reported separately"});
    fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}
#[cfg(not(windows))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("this offscreen proof requires the Windows raster backend".into())
}
