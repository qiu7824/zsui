#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{fs, io::BufWriter, path::Path};

    use serde::Serialize;
    use zsui::{
        compare_text_bgra_pixels, compare_text_geometry, directwrite_text_bgra,
        directwrite_text_proof, text_bgra_difference_image, text_geometry_overlay_svg, Color,
        HorizontalAlign, TextStyle, TextWeight, TextWrap, VerticalAlign, ZsRustTextEngine,
        ZsTextGeometryDiff, ZsTextLineMetricPolicy, ZsTextPixelDiff, ZsTextRasterProfile,
    };

    #[derive(Clone, Copy)]
    struct CorpusCase {
        id: &'static str,
        font: &'static str,
        script: &'static str,
        text: &'static str,
        wrap: TextWrap,
        width_dp: u32,
        height_dp: u32,
    }

    #[derive(Clone, Copy)]
    struct Variant {
        id: &'static str,
        size: f32,
        line_height: f32,
        weight: TextWeight,
        dpi_scale: f32,
    }

    #[derive(Serialize)]
    struct CaseSummary {
        id: String,
        corpus: &'static str,
        script: &'static str,
        requested_font: &'static str,
        variant: &'static str,
        size_dp: f32,
        line_height_dp: f32,
        weight: &'static str,
        dpi_scale: f32,
        width_px: u32,
        height_px: u32,
        geometry: ZsTextGeometryDiff,
        pixels: ZsTextPixelDiff,
    }

    #[derive(Serialize)]
    struct SuiteSummary {
        schema: &'static str,
        reference: &'static str,
        candidate: &'static str,
        case_count: usize,
        geometry_pass_count: usize,
        exact_font_face_case_count: usize,
        exact_glyph_id_case_count: usize,
        maximum_origin_delta_px: f32,
        maximum_advance_delta_px: f32,
        maximum_different_pixel_ratio: f64,
        maximum_mean_absolute_channel_delta: f64,
        cases: Vec<CaseSummary>,
    }

    let output = std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("target/text-proof"));
    fs::create_dir_all(&output)?;

    let corpora = [
        CorpusCase {
            id: "segoe-latin",
            font: "Segoe UI",
            script: "Latin",
            text: "Hamburgefontsiv AVATAR office ffi — naïve façade 0123456789",
            wrap: TextWrap::NoWrap,
            width_dp: 620,
            height_dp: 80,
        },
        CorpusCase {
            id: "yahei-chinese",
            font: "Microsoft YaHei UI",
            script: "Simplified Chinese",
            text: "发票助手：中文标点，布局、字重与基线 0123456789",
            wrap: TextWrap::NoWrap,
            width_dp: 620,
            height_dp: 80,
        },
        CorpusCase {
            id: "yu-gothic-japanese",
            font: "Yu Gothic UI",
            script: "Japanese",
            text: "設定とプレビュー：かな・カナ・漢字、0123456789",
            wrap: TextWrap::NoWrap,
            width_dp: 620,
            height_dp: 80,
        },
        CorpusCase {
            id: "malgun-korean",
            font: "Malgun Gothic",
            script: "Korean",
            text: "설정과 미리 보기: 한글 자간 및 기준선 0123456789",
            wrap: TextWrap::NoWrap,
            width_dp: 620,
            height_dp: 80,
        },
        CorpusCase {
            id: "segoe-arabic",
            font: "Segoe UI",
            script: "Arabic",
            text: "واجهة المستخدم العربية 12345 — مرحباً بالعالم",
            wrap: TextWrap::NoWrap,
            width_dp: 620,
            height_dp: 80,
        },
        CorpusCase {
            id: "segoe-hebrew",
            font: "Segoe UI",
            script: "Hebrew",
            text: "ממשק משתמש טבעי 12345 — שלום עולם",
            wrap: TextWrap::NoWrap,
            width_dp: 620,
            height_dp: 80,
        },
        CorpusCase {
            id: "nirmala-devanagari",
            font: "Nirmala UI",
            script: "Devanagari",
            text: "मूल उपयोगकर्ता इंटरफ़ेस १२३४५ — नमस्ते दुनिया",
            wrap: TextWrap::NoWrap,
            width_dp: 620,
            height_dp: 80,
        },
        CorpusCase {
            id: "leelawadee-thai",
            font: "Leelawadee UI",
            script: "Thai",
            text: "ส่วนติดต่อผู้ใช้แบบเนทีฟ 12345 — สวัสดีชาวโลก",
            wrap: TextWrap::NoWrap,
            width_dp: 620,
            height_dp: 80,
        },
        CorpusCase {
            id: "segoe-emoji",
            font: "Segoe UI Emoji",
            script: "Emoji and ZWJ",
            text: "状态 ✅ ⚠️ ❤️ 👨‍👩‍👧‍👦 🧑🏽‍💻 🚀",
            wrap: TextWrap::NoWrap,
            width_dp: 620,
            height_dp: 80,
        },
        CorpusCase {
            id: "consolas-code",
            font: "Consolas",
            script: "Monospace",
            text: "fn main() { println!(\"ZSUI -> 0x7F\"); }",
            wrap: TextWrap::NoWrap,
            width_dp: 620,
            height_dp: 80,
        },
        CorpusCase {
            id: "mixed-fallback-wrap",
            font: "Segoe UI",
            script: "Mixed fallback and BiDi",
            text: "ZSUI 原生文字 — 日本語 — 한국어 — مرحبا — שלום — नमस्ते — 👋🏽",
            wrap: TextWrap::Word,
            width_dp: 360,
            height_dp: 160,
        },
    ];
    let variants = [
        Variant {
            id: "caption-100-regular",
            size: 11.0,
            line_height: 16.0,
            weight: TextWeight::Regular,
            dpi_scale: 1.0,
        },
        Variant {
            id: "body-100-regular",
            size: 14.0,
            line_height: 20.0,
            weight: TextWeight::Regular,
            dpi_scale: 1.0,
        },
        Variant {
            id: "body-125-semibold",
            size: 14.0,
            line_height: 20.0,
            weight: TextWeight::Semibold,
            dpi_scale: 1.25,
        },
        Variant {
            id: "body-150-regular",
            size: 14.0,
            line_height: 20.0,
            weight: TextWeight::Regular,
            dpi_scale: 1.5,
        },
        Variant {
            id: "title-100-semibold",
            size: 20.0,
            line_height: 28.0,
            weight: TextWeight::Semibold,
            dpi_scale: 1.0,
        },
        Variant {
            id: "display-200-bold",
            size: 28.0,
            line_height: 36.0,
            weight: TextWeight::Bold,
            dpi_scale: 2.0,
        },
    ];

    let mut engine =
        ZsRustTextEngine::new().with_line_metric_policy(ZsTextLineMetricPolicy::WindowsDirectWrite);
    let mut summaries = Vec::new();
    for corpus in corpora {
        for variant in variants {
            let id = format!("{}--{}", corpus.id, variant.id);
            let case_dir = output.join(&id);
            fs::create_dir_all(&case_dir)?;
            let width = (corpus.width_dp as f32 * variant.dpi_scale).round() as u32;
            let height = (corpus.height_dp as f32 * variant.dpi_scale).round() as u32;
            let mut style = TextStyle::line(corpus.font, variant.size, Color::rgb(24, 24, 24));
            style.line_height = variant.line_height;
            style.weight = variant.weight;
            style.horizontal_align = HorizontalAlign::Start;
            style.vertical_align = VerticalAlign::Start;
            style.wrap = corpus.wrap;
            style.ellipsis = false;

            let rust_layout = engine.layout(
                corpus.text,
                &style,
                width as i32,
                height as i32,
                variant.dpi_scale,
            );
            let directwrite = directwrite_text_proof(
                corpus.text,
                &style,
                width as i32,
                height as i32,
                variant.dpi_scale,
            )
            .ok_or("DirectWrite geometry proof was unavailable")?;
            let geometry = compare_text_geometry(
                &directwrite,
                rust_layout.proof(),
                0.25 * variant.dpi_scale.max(1.0),
            );
            let reference_pixels = directwrite_text_bgra(
                corpus.text,
                &style,
                width,
                height,
                variant.dpi_scale,
                Color::rgb(255, 255, 255),
            )
            .ok_or("DirectWrite pixel proof was unavailable")?;
            let mut rust_pixels = vec![255_u8; width as usize * height as usize * 4];
            engine.composite_bgra(
                &rust_layout,
                &mut rust_pixels,
                width,
                height,
                width as usize * 4,
                style.color,
                ZsTextRasterProfile::subpixel_rgb(),
            )?;
            let pixels = compare_text_bgra_pixels(&reference_pixels, &rust_pixels, 16)?;
            let difference = text_bgra_difference_image(&reference_pixels, &rust_pixels)?;

            fs::write(case_dir.join("rust.json"), rust_layout.proof_json()?)?;
            fs::write(
                case_dir.join("directwrite.json"),
                serde_json::to_string_pretty(&directwrite)?,
            )?;
            fs::write(
                case_dir.join("geometry-diff.json"),
                serde_json::to_string_pretty(&geometry)?,
            )?;
            fs::write(
                case_dir.join("geometry-overlay.svg"),
                text_geometry_overlay_svg(&directwrite, rust_layout.proof()),
            )?;
            fs::write(
                case_dir.join("rust-outlines.svg"),
                engine.outline_svg(&rust_layout),
            )?;
            write_bgra_png(
                &case_dir.join("directwrite.png"),
                width,
                height,
                &reference_pixels,
            )?;
            write_bgra_png(
                &case_dir.join("rust-subpixel.png"),
                width,
                height,
                &rust_pixels,
            )?;
            write_bgra_png(
                &case_dir.join("pixel-difference.png"),
                width,
                height,
                &difference,
            )?;
            summaries.push(CaseSummary {
                id,
                corpus: corpus.id,
                script: corpus.script,
                requested_font: corpus.font,
                variant: variant.id,
                size_dp: variant.size,
                line_height_dp: variant.line_height,
                weight: weight_name(variant.weight),
                dpi_scale: variant.dpi_scale,
                width_px: width,
                height_px: height,
                geometry,
                pixels,
            });
        }
    }

    let summary = SuiteSummary {
        schema: "zsui.text-proof-suite/v1",
        reference: "windows-directwrite",
        candidate: "zsui-rust-harfrust-swash",
        case_count: summaries.len(),
        geometry_pass_count: summaries
            .iter()
            .filter(|case| case.geometry.within_tolerance)
            .count(),
        exact_font_face_case_count: summaries
            .iter()
            .filter(|case| {
                case.geometry.glyph_count_equal
                    && case.geometry.matching_font_faces == case.geometry.compared_glyphs
            })
            .count(),
        exact_glyph_id_case_count: summaries
            .iter()
            .filter(|case| {
                case.geometry.glyph_count_equal
                    && case.geometry.matching_glyph_ids == case.geometry.compared_glyphs
            })
            .count(),
        maximum_origin_delta_px: summaries
            .iter()
            .map(|case| case.geometry.max_origin_delta_px)
            .fold(0.0, f32::max),
        maximum_advance_delta_px: summaries
            .iter()
            .map(|case| case.geometry.max_advance_delta_px)
            .fold(0.0, f32::max),
        maximum_different_pixel_ratio: summaries
            .iter()
            .map(|case| case.pixels.different_pixel_ratio)
            .fold(0.0, f64::max),
        maximum_mean_absolute_channel_delta: summaries
            .iter()
            .map(|case| case.pixels.mean_absolute_channel_delta)
            .fold(0.0, f64::max),
        cases: summaries,
    };
    fs::write(
        output.join("summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    println!("{}", output.canonicalize()?.display());
    println!(
        "cases={} geometry_pass={} exact_faces={} exact_glyphs={} max_origin={:.3}px max_advance={:.3}px max_pixel_ratio={:.4}",
        summary.case_count,
        summary.geometry_pass_count,
        summary.exact_font_face_case_count,
        summary.exact_glyph_id_case_count,
        summary.maximum_origin_delta_px,
        summary.maximum_advance_delta_px,
        summary.maximum_different_pixel_ratio,
    );
    fn write_bgra_png(
        path: &Path,
        width: u32,
        height: u32,
        bgra: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut rgba = Vec::with_capacity(bgra.len());
        for pixel in bgra.chunks_exact(4) {
            rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
        let file = fs::File::create(path)?;
        let mut encoder = png::Encoder::new(BufWriter::new(file), width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&rgba)?;
        Ok(())
    }

    const fn weight_name(weight: TextWeight) -> &'static str {
        match weight {
            TextWeight::Automatic => "automatic",
            TextWeight::Regular => "regular",
            TextWeight::Medium => "medium",
            TextWeight::Semibold => "semibold",
            TextWeight::Bold => "bold",
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("text_geometry_proof requires Windows DirectWrite");
}
