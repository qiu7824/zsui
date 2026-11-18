//! Opens a PNG passed on the command line. The toolbar controls image fit.
use zsui::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: image_inspector <image.png>")?;
    let frame = ImageFrame::from_png(1, &std::fs::read(&path)?, 64 * 1024 * 1024)?;
    window("Image inspector")
        .size(720, 480)
        .stateful(
            ImageFit::Contain,
            move |fit| {
                let toolbar = grid(
                    [GridTrack::FLEX; 2],
                    [GridTrack::FLEX],
                    [
                        (
                            0,
                            0,
                            typed::button("Fit").on_click(ImageFit::Contain).into(),
                        ),
                        (0, 1, typed::button("Fill").on_click(ImageFit::Cover).into()),
                    ],
                );
                column([toolbar, image(frame.clone(), *fit).flex(1.0)])
            },
            |fit, selected, _| *fit = selected,
        )
        .run()?;
    Ok(())
}
