//! Explicit, self-contained vector icon resources with bounded raster caching.
use crate::{
    Color, NativeDrawImageCommand, NativeIconColorMode, Rect, ZsImageFrame, ZsImageFrameId,
    ZsuiError, ZsuiResult,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

/// Stable identity assigned by the resource owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IconAssetId(pub u64);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AssetDeclaration {
    id: IconAssetId,
    svg: String,
    license: String,
}
struct AssetInner {
    declaration: AssetDeclaration,
    tree: resvg::usvg::Tree,
    cache: Mutex<VecDeque<((u32, u32, Color, bool), ZsImageFrame)>>,
}

/// A validated SVG resource. Parsing happens once; no URLs or external files are loaded.
#[derive(Clone)]
pub struct IconAsset(Arc<AssetInner>);
impl std::fmt::Debug for IconAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IconAsset")
            .field("id", &self.0.declaration.id)
            .field("license", &self.0.declaration.license)
            .finish()
    }
}
impl PartialEq for IconAsset {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0.declaration == other.0.declaration
    }
}
impl Eq for IconAsset {}
impl Serialize for IconAsset {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.declaration.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for IconAsset {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let declaration = AssetDeclaration::deserialize(deserializer)?;
        Self::from_svg(declaration.id, &declaration.svg, declaration.license)
            .map_err(serde::de::Error::custom)
    }
}

impl IconAsset {
    /// Loads a bounded SVG file explicitly selected by the application.
    pub fn from_svg_file(
        id: IconAssetId,
        path: impl AsRef<std::path::Path>,
        license: impl Into<String>,
    ) -> ZsuiResult<Self> {
        use std::io::Read;
        let file = std::fs::File::open(path.as_ref()).map_err(|error| {
            ZsuiError::host(
                "icon.resource",
                format!("{}: {error}", path.as_ref().display()),
            )
        })?;
        let mut svg = String::new();
        file.take(256 * 1024 + 1)
            .read_to_string(&mut svg)
            .map_err(|error| ZsuiError::invalid_spec("icon.svg", error.to_string()))?;
        Self::from_svg(id, &svg, license)
    }
    /// Parses an SVG icon of at most 256 KiB and 4,096 elements.
    /// Text, images, scripting, animations, external references and filters are rejected.
    pub fn from_svg(id: IconAssetId, svg: &str, license: impl Into<String>) -> ZsuiResult<Self> {
        if svg.len() > 256 * 1024 {
            return Err(ZsuiError::invalid_spec("icon.svg", "SVG exceeds 256 KiB"));
        }
        let license = license.into();
        if license.trim().is_empty() {
            return Err(ZsuiError::invalid_spec(
                "icon.license",
                "resource license or ownership declaration is required",
            ));
        }
        let xml = roxmltree::Document::parse(svg)
            .map_err(|error| ZsuiError::invalid_spec("icon.svg", error.to_string()))?;
        if xml.root_element().tag_name().name() != "svg" {
            return Err(ZsuiError::invalid_spec("icon.svg", "root must be svg"));
        }
        let mut count = 0;
        for element in xml.descendants().filter(|node| node.is_element()) {
            count += 1;
            if count > 4096 {
                return Err(ZsuiError::invalid_spec(
                    "icon.svg",
                    "too many vector elements",
                ));
            }
            if !matches!(
                element.tag_name().name(),
                "svg"
                    | "g"
                    | "path"
                    | "rect"
                    | "circle"
                    | "ellipse"
                    | "line"
                    | "polyline"
                    | "polygon"
                    | "defs"
                    | "clipPath"
                    | "linearGradient"
                    | "radialGradient"
                    | "stop"
                    | "title"
                    | "desc"
            ) {
                return Err(ZsuiError::invalid_spec(
                    "icon.svg",
                    format!("unsupported element {}", element.tag_name().name()),
                ));
            }
            for attribute in element.attributes() {
                if attribute.name().starts_with("on")
                    || matches!(attribute.name(), "href" | "src")
                    || attribute.value().contains("@import")
                    || attribute
                        .value()
                        .split("url(")
                        .skip(1)
                        .any(|value| !value.starts_with('#'))
                {
                    return Err(ZsuiError::invalid_spec(
                        "icon.svg",
                        "external references or executable content are not permitted",
                    ));
                }
            }
        }
        let tree = resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default())
            .map_err(|error| ZsuiError::invalid_spec("icon.svg", error.to_string()))?;
        if tree.size().width() > 4096.0 || tree.size().height() > 4096.0 {
            return Err(ZsuiError::invalid_spec(
                "icon.svg",
                "view size exceeds 4096 logical units",
            ));
        }
        Ok(Self(Arc::new(AssetInner {
            declaration: AssetDeclaration {
                id,
                svg: svg.into(),
                license,
            },
            tree,
            cache: Mutex::new(VecDeque::new()),
        })))
    }
    /// Identity supplied by the resource owner.
    pub fn id(&self) -> IconAssetId {
        self.0.declaration.id
    }
    /// Resource attribution or ownership declaration.
    pub fn license(&self) -> &str {
        &self.0.declaration.license
    }
    /// Rasterizes a single icon. Cache retention is at most 16 entries and 1 MiB.
    pub fn rasterize(
        &self,
        width: u32,
        height: u32,
        tint: Color,
        original: bool,
    ) -> ZsuiResult<ZsImageFrame> {
        if width == 0 || height == 0 || width > 512 || height > 512 {
            return Err(ZsuiError::invalid_spec(
                "icon.size",
                "raster icon dimensions must be between 1 and 512 pixels",
            ));
        }
        let key = (width, height, tint, original);
        if let Ok(mut cache) = self.0.cache.lock() {
            if let Some(index) = cache.iter().position(|entry| entry.0 == key) {
                let entry = cache.remove(index).unwrap();
                let image = entry.1.clone();
                cache.push_back(entry);
                return Ok(image);
            }
        }
        let mut pixmap = tiny_skia::Pixmap::new(width, height)
            .ok_or_else(|| ZsuiError::host("icon.raster", "could not allocate icon surface"))?;
        let size = self.0.tree.size();
        let scale = (width as f32 / size.width()).min(height as f32 / size.height());
        let transform = tiny_skia::Transform::from_scale(scale, scale).post_translate(
            (width as f32 - size.width() * scale) / 2.0,
            (height as f32 - size.height() * scale) / 2.0,
        );
        resvg::render(&self.0.tree, transform, &mut pixmap.as_mut());
        let mut bgra = pixmap.take();
        for pixel in bgra.chunks_exact_mut(4) {
            if original {
                pixel.swap(0, 2);
            } else {
                let alpha = (u16::from(pixel[3]) * u16::from(tint.a) + 127) / 255;
                pixel[0] = ((u16::from(tint.b) * alpha + 127) / 255) as u8;
                pixel[1] = ((u16::from(tint.g) * alpha + 127) / 255) as u8;
                pixel[2] = ((u16::from(tint.r) * alpha + 127) / 255) as u8;
                pixel[3] = alpha as u8;
            }
        }
        let image = ZsImageFrame::from_premultiplied_bgra8(
            ZsImageFrameId(self.id().0),
            width,
            height,
            bgra,
        )?;
        if let Ok(mut cache) = self.0.cache.lock() {
            cache.push_back((key, image.clone()));
            while cache.len() > 16
                || cache
                    .iter()
                    .map(|entry| entry.1.decoded_bytes())
                    .sum::<usize>()
                    > 1024 * 1024
            {
                cache.pop_front();
            }
        }
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_icons_preserve_tint_alpha_and_bounded_cache_identity() {
        let asset=IconAsset::from_svg(IconAssetId(2),r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><circle cx="10" cy="10" r="8"/></svg>"#,"Application-owned").unwrap();
        let tint = Color::rgba(200, 80, 40, 128);
        let first = asset.rasterize(40, 40, tint, false).unwrap();
        let second = asset.rasterize(40, 40, tint, false).unwrap();
        assert!(first.is_same_frame(&second));
        let center = &first.premultiplied_bgra8()[(20 * 40 + 20) * 4..(20 * 40 + 20) * 4 + 4];
        assert_eq!(center, &[20, 40, 100, 128]);
        for side in 16..80 {
            asset.rasterize(side, side, tint, false).unwrap();
        }
        let cache = asset.0.cache.lock().unwrap();
        assert!(cache.len() <= 16);
        assert!(
            cache
                .iter()
                .map(|entry| entry.1.decoded_bytes())
                .sum::<usize>()
                <= 1024 * 1024
        );
    }
    #[test]
    fn custom_svg_rejects_external_or_executable_resources() {
        for content in [
            r#"<image href="file:///secret.png"/>"#,
            r#"<script>alert(1)</script>"#,
            r#"<text>font dependency</text>"#,
            r#"<path fill="url(#ok) url(https://example.test/icon)" d="M0 0L1 1"/>"#,
        ] {
            let source = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">{content}</svg>"#
            );
            assert!(IconAsset::from_svg(IconAssetId(3), &source, "Owned").is_err());
        }
        assert!(IconAsset::from_svg(IconAssetId(3), "not xml", "Owned").is_err());
        assert!(IconAsset::from_svg(
            IconAssetId(3),
            r#"<svg xmlns="http://www.w3.org/2000/svg"/>"#,
            ""
        )
        .is_err());
    }
    #[test]
    fn typed_paths_serialize_with_validation_and_no_runtime_resource_state() {
        let asset = IconAsset::from_path(
            IconAssetId(4),
            20.0,
            20.0,
            &[
                IconPathSegment::Move(2.0, 2.0),
                IconPathSegment::Line(18.0, 2.0),
                IconPathSegment::Line(10.0, 18.0),
                IconPathSegment::Close,
            ],
            "Owned",
        )
        .unwrap();
        let json = serde_json::to_string(&asset).unwrap();
        let decoded: IconAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(asset, decoded);
        assert!(!json.contains("cache"));
        assert!(IconAsset::from_path(
            IconAssetId(5),
            20.0,
            20.0,
            &[IconPathSegment::Move(f32::NAN, 0.0)],
            "Owned"
        )
        .is_err());
    }
}

/// A semantic symbol or an application-owned vector resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IconSource {
    Semantic(crate::ZsIcon),
    Vector(IconAsset),
}
impl From<crate::ZsIcon> for IconSource {
    fn from(icon: crate::ZsIcon) -> Self {
        Self::Semantic(icon)
    }
}
impl From<IconAsset> for IconSource {
    fn from(icon: IconAsset) -> Self {
        Self::Vector(icon)
    }
}

/// A vector icon projected through native semantic color resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeDrawVectorIcon {
    pub asset: IconAsset,
    pub bounds: Rect,
    pub color: crate::ColorRole,
    pub custom_color: Option<Color>,
    pub color_mode: NativeIconColorMode,
}
impl NativeDrawVectorIcon {
    pub(crate) fn image(
        &self,
        semantic: Color,
        high_contrast: bool,
    ) -> ZsuiResult<NativeDrawImageCommand> {
        let width = self.bounds.width.max(1) as u32;
        let height = self.bounds.height.max(1) as u32;
        let image = self.asset.rasterize(
            width,
            height,
            if high_contrast {
                semantic
            } else {
                self.custom_color.unwrap_or(semantic)
            },
            !high_contrast && self.color_mode == NativeIconColorMode::Original,
        )?;
        Ok(NativeDrawImageCommand::new(
            image,
            Rect {
                x: 0,
                y: 0,
                width: width as i32,
                height: height as i32,
            },
            self.bounds,
        ))
    }
}

/// Typed path operations in a vector icon's logical coordinate system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IconPathSegment {
    Move(f32, f32),
    Line(f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Quadratic(f32, f32, f32, f32),
    Close,
}
impl IconAsset {
    /// Creates a filled vector path through the same validated SVG pipeline.
    pub fn from_path(
        id: IconAssetId,
        width: f32,
        height: f32,
        path: &[IconPathSegment],
        license: impl Into<String>,
    ) -> ZsuiResult<Self> {
        if !width.is_finite()
            || !height.is_finite()
            || width <= 0.0
            || height <= 0.0
            || path.len() > 16384
        {
            return Err(ZsuiError::invalid_spec(
                "icon.path",
                "invalid path size or segment count",
            ));
        }
        let mut data = String::new();
        for segment in path {
            use std::fmt::Write;
            match *segment {
                IconPathSegment::Move(x, y) => write!(data, "M{x} {y} "),
                IconPathSegment::Line(x, y) => write!(data, "L{x} {y} "),
                IconPathSegment::Cubic(a, b, c, d, x, y) => {
                    write!(data, "C{a} {b} {c} {d} {x} {y} ")
                }
                IconPathSegment::Quadratic(a, b, x, y) => write!(data, "Q{a} {b} {x} {y} "),
                IconPathSegment::Close => write!(data, "Z "),
            }
            .map_err(|error| ZsuiError::invalid_spec("icon.path", error.to_string()))?;
        }
        if data.contains("NaN") || data.contains("inf") {
            return Err(ZsuiError::invalid_spec(
                "icon.path",
                "path coordinates must be finite",
            ));
        }
        Self::from_svg(
            id,
            &format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}"><path d="{data}"/></svg>"#
            ),
            license,
        )
    }
}
