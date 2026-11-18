#[cfg(any(feature = "grid", feature = "canvas"))]
use super::Dp;
use super::Element;

/// A fixed logical dimension or a positive fractional weight.
#[cfg(feature = "grid")]
#[derive(Clone, Copy)]
pub enum GridTrack {
    /// A track with an explicit length.
    Fixed(Dp),
    /// A share of available space.
    Fraction(std::num::NonZeroU16),
}

#[cfg(feature = "grid")]
impl GridTrack {
    /// One flexible share.
    pub const FLEX: Self = Self::Fraction(std::num::NonZeroU16::MIN);
    fn into_raw(self) -> crate::ZsGridTrack {
        match self {
            Self::Fixed(size) => crate::ZsGridTrack::fixed(size.into()),
            Self::Fraction(weight) => crate::ZsGridTrack::fraction(
                crate::ZsGridFraction::new(weight.get()).expect("positive grid fraction"),
            ),
        }
    }
}

/// Arranges elements in explicit zero-based (row, column, element) cells.
#[cfg(feature = "grid")]
pub fn grid<M>(
    columns: impl IntoIterator<Item = GridTrack>,
    rows: impl IntoIterator<Item = GridTrack>,
    cells: impl IntoIterator<Item = (usize, usize, Element<M>)>,
) -> Element<M> {
    Element::from_raw(crate::grid(
        columns.into_iter().map(GridTrack::into_raw),
        rows.into_iter().map(GridTrack::into_raw),
        cells
            .into_iter()
            .map(|(row, column, element)| crate::ZsGridCell::new(row, column, element.into_raw())),
    ))
}

/// An immutable decoded image. Cloning shares the pixel storage.
#[cfg(feature = "image-preview")]
#[derive(Clone)]
pub struct ImageFrame(crate::ZsImageFrame);

#[cfg(feature = "image-preview")]
impl ImageFrame {
    /// Decodes a PNG within the given RGBA byte budget; the ID identifies this frame.
    pub fn from_png(id: u64, bytes: &[u8], max_decoded_bytes: usize) -> Result<Self, super::Error> {
        crate::image_preview::decode_png_frame(
            crate::ZsImageFrameId::new(id),
            bytes,
            max_decoded_bytes,
        )
        .map(Self)
        .map_err(super::Error)
    }
}

/// How the decoded image fits its allocated rectangle.
#[cfg(feature = "image-preview")]
pub use crate::ZsImageFit as ImageFit;

/// Displays an immutable image using a specified fit.
#[cfg(feature = "image-preview")]
pub fn image<M: Clone>(frame: ImageFrame, fit: ImageFit) -> Element<M> {
    Element::from_raw(crate::image(frame.0).image_fit(fit))
}

/// A semantic drawing color resolved by the current theme.
#[cfg(feature = "canvas")]
pub use crate::ColorRole as CanvasColor;

/// A native drawing scene in local logical coordinates.
#[cfg(feature = "canvas")]
#[derive(Default)]
pub struct CanvasScene(crate::ZsCanvasScene);

#[cfg(feature = "canvas")]
impl CanvasScene {
    /// Fills a rectangle with a palette role in declaration order.
    pub fn fill_rect(self, x: Dp, y: Dp, width: Dp, height: Dp, color: CanvasColor) -> Self {
        let rect = crate::ZsCanvasRect::new(x.into(), y.into(), width.into(), height.into());
        Self(self.0.with(crate::ZsCanvasPrimitive::fill_rect(
            rect,
            crate::NativeDrawFill::role(color),
        )))
    }
}

/// Creates a drawing surface from its allocated logical width and height.
#[cfg(feature = "canvas")]
pub fn canvas_with<M>(build: impl Fn(Dp, Dp) -> CanvasScene + Send + Sync + 'static) -> Element<M> {
    Element::from_raw(crate::canvas_with(move |cx| {
        let size = cx.size();
        build(Dp::new(size.width.0), Dp::new(size.height.0)).0
    }))
}

#[cfg(all(test, feature = "image-preview", feature = "grid", feature = "canvas"))]
mod tests {
    use super::*;
    use crate::{Dpi, NativeDrawCommand, Rect, View, ViewLayoutCx, ViewPaintCx};

    #[test]
    fn stable_grid_composes_a_png_and_size_aware_native_drawing() {
        let mut png = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png, 2, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&[255; 8])
                .unwrap();
        }
        assert!(ImageFrame::from_png(1, &png, 7).is_err());
        let frame = ImageFrame::from_png(1, &png, 8).unwrap();
        let drawing = canvas_with(|width, height| {
            CanvasScene::default().fill_rect(
                Dp::new(0.0),
                Dp::new(0.0),
                width,
                height,
                CanvasColor::Accent,
            )
        });
        let mut widget = grid::<()>(
            [GridTrack::Fixed(Dp::new(64.0)), GridTrack::FLEX],
            [GridTrack::FLEX],
            [(0, 0, image(frame, ImageFit::Contain)), (0, 1, drawing)],
        )
        .into_raw();
        let dpi = Dpi::standard();
        widget.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 160,
                height: 80,
            },
            dpi,
        ));
        let mut paint = ViewPaintCx::new(dpi);
        widget.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|c| matches!(c,
            NativeDrawCommand::Image(i) if i.bounds.width == 64 && i.bounds.height == 32)));
        assert!(paint.plan().commands.iter().any(|c| matches!(c,
            NativeDrawCommand::FillRect{rect,fill:crate::NativeDrawFill::Role(CanvasColor::Accent)}
                if *rect == Rect{x:64,y:0,width:96,height:80})));
    }
}
