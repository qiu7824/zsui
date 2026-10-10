use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::compare::{ProofDocument, WidgetEvidence};

#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[u8; 4]>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ImageDiffMetrics {
    pub different_pixels: usize,
    pub total_pixels: usize,
    pub difference_percent: f64,
    pub max_channel_delta: u32,
}

#[derive(Debug, Clone)]
pub struct ImageComparison {
    pub full: ImageDiffMetrics,
    pub critical: ImageDiffMetrics,
    pub critical_region_count: usize,
    pub diff_path: String,
    pub passed: bool,
}

pub fn decode_png(path: &Path) -> Result<Image, String> {
    let file =
        File::open(path).map_err(|error| format!("failed to open {}: {error}", path.display()))?;
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|error| format!("failed to decode PNG header {}: {error}", path.display()))?;
    let buffer_size = reader
        .output_buffer_size()
        .ok_or_else(|| format!("PNG {} is too large to decode", path.display()))?;
    let mut buffer = vec![0u8; buffer_size];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|error| format!("failed to decode PNG {}: {error}", path.display()))?;
    let bytes = &buffer[..info.buffer_size()];

    if info.bit_depth != png::BitDepth::Eight {
        return Err(format!(
            "unsupported PNG bit depth for {}: {:?}",
            path.display(),
            info.bit_depth
        ));
    }

    let pixels: Vec<[u8; 4]> = match info.color_type {
        png::ColorType::Rgba => bytes
            .chunks_exact(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
            .collect(),
        png::ColorType::Rgb => bytes
            .chunks_exact(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], 255])
            .collect(),
        png::ColorType::Grayscale => bytes
            .iter()
            .map(|value| [*value, *value, *value, 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => bytes
            .chunks_exact(2)
            .map(|chunk| [chunk[0], chunk[0], chunk[0], chunk[1]])
            .collect(),
        other => {
            return Err(format!(
                "unsupported PNG color type for {}: {other:?}",
                path.display()
            ))
        }
    };

    let expected = info.width as usize * info.height as usize;
    if pixels.len() != expected {
        return Err(format!(
            "PNG pixel count mismatch for {}: expected {expected}, decoded {}",
            path.display(),
            pixels.len()
        ));
    }

    Ok(Image {
        width: info.width,
        height: info.height,
        pixels,
    })
}

#[derive(Debug, Clone, Copy)]
struct Region {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Region {
    fn from_widget(image_width: u32, image_height: u32, widget: &WidgetEvidence) -> Option<Self> {
        if widget.width <= 0 || widget.height <= 0 {
            return None;
        }
        let x = widget.x.max(0) as u32;
        let y = widget.y.max(0) as u32;
        let width = widget.width as u32;
        let height = widget.height as u32;
        if x >= image_width || y >= image_height {
            return None;
        }
        Some(Self {
            x,
            y,
            width: width.min(image_width - x),
            height: height.min(image_height - y),
        })
    }

    fn from_full(image_width: u32, image_height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width: image_width,
            height: image_height,
        }
    }
}

fn channel_delta(a: [u8; 4], b: [u8; 4]) -> u32 {
    let mut delta = 0u32;
    for index in 0..4 {
        delta = delta.max((a[index] as i32 - b[index] as i32).unsigned_abs());
    }
    delta
}

fn diff_metrics(
    baseline: &Image,
    actual: &Image,
    region: Region,
    tolerance: u8,
) -> ImageDiffMetrics {
    let mut different_pixels = 0usize;
    let mut total_pixels = 0usize;
    let mut max_channel_delta = 0u32;

    for y in region.y..region.y + region.height {
        for x in region.x..region.x + region.width {
            let index = y as usize * actual.width as usize + x as usize;
            let delta = channel_delta(baseline.pixels[index], actual.pixels[index]);
            max_channel_delta = max_channel_delta.max(delta);
            total_pixels += 1;
            if delta > tolerance as u32 {
                different_pixels += 1;
            }
        }
    }

    ImageDiffMetrics {
        different_pixels,
        total_pixels,
        difference_percent: if total_pixels == 0 {
            0.0
        } else {
            different_pixels as f64 * 100.0 / total_pixels as f64
        },
        max_channel_delta,
    }
}

fn write_diff_png(
    path: &Path,
    baseline: &Image,
    actual: &Image,
    tolerance: u8,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    let mut pixels = actual.pixels.clone();
    for (index, pixel) in pixels.iter_mut().enumerate() {
        if channel_delta(baseline.pixels[index], actual.pixels[index]) > tolerance as u32 {
            *pixel = [255, 0, 255, 255];
        }
    }

    let file = File::create(path)
        .map_err(|error| format!("failed to create {}: {error}", path.display()))?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, actual.width, actual.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|error| format!("failed to write PNG header {}: {error}", path.display()))?;
    let mut flat = Vec::with_capacity(pixels.len() * 4);
    for pixel in pixels {
        flat.extend_from_slice(&pixel);
    }
    writer
        .write_image_data(&flat)
        .map_err(|error| format!("failed to write PNG data {}: {error}", path.display()))?;

    Ok(())
}

pub fn compare_image_pair(
    baseline_path: &Path,
    actual_path: &Path,
    diff_path: &Path,
    baseline_document: &ProofDocument,
    actual_document: &ProofDocument,
    tolerance: u8,
    critical_threshold_percent: f64,
    full_threshold_percent: f64,
) -> Result<ImageComparison, String> {
    let baseline = decode_png(baseline_path)?;
    let actual = decode_png(actual_path)?;

    if baseline.width != actual.width || baseline.height != actual.height {
        return Err(format!(
            "PNG dimensions differ: baseline {}x{}, actual {}x{}",
            baseline.width, baseline.height, actual.width, actual.height
        ));
    }
    if actual.width as u64 != actual_document.window.pixel_width
        || actual.height as u64 != actual_document.window.pixel_height
    {
        return Err(format!(
            "actual PNG dimensions {}x{} do not match proof window pixel size {}x{}",
            actual.width,
            actual.height,
            actual_document.window.pixel_width,
            actual_document.window.pixel_height
        ));
    }
    if baseline.width as u64 != baseline_document.window.pixel_width
        || baseline.height as u64 != baseline_document.window.pixel_height
    {
        return Err(format!(
            "baseline PNG dimensions {}x{} do not match proof window pixel size {}x{}",
            baseline.width,
            baseline.height,
            baseline_document.window.pixel_width,
            baseline_document.window.pixel_height
        ));
    }

    let full = diff_metrics(
        &baseline,
        &actual,
        Region::from_full(actual.width, actual.height),
        tolerance,
    );

    let mut critical_accumulator = ImageDiffMetrics {
        different_pixels: 0,
        total_pixels: 0,
        difference_percent: 0.0,
        max_channel_delta: 0,
    };
    let mut critical_region_count = 0usize;
    for widget in &actual_document.widgets {
        if let Some(region) = Region::from_widget(actual.width, actual.height, widget) {
            let metrics = diff_metrics(&baseline, &actual, region, tolerance);
            critical_accumulator.different_pixels += metrics.different_pixels;
            critical_accumulator.total_pixels += metrics.total_pixels;
            critical_accumulator.max_channel_delta = critical_accumulator
                .max_channel_delta
                .max(metrics.max_channel_delta);
            critical_region_count += 1;
        }
    }
    critical_accumulator.difference_percent = if critical_accumulator.total_pixels == 0 {
        0.0
    } else {
        critical_accumulator.different_pixels as f64 * 100.0
            / critical_accumulator.total_pixels as f64
    };

    write_diff_png(diff_path, &baseline, &actual, tolerance)?;

    let passed = critical_accumulator.difference_percent <= critical_threshold_percent
        && full.difference_percent <= full_threshold_percent;

    Ok(ImageComparison {
        full,
        critical: critical_accumulator,
        critical_region_count,
        diff_path: diff_path.display().to_string(),
        passed,
    })
}
