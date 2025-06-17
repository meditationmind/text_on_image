use crate::*;
use ab_glyph::{FontRef, PxScale};
use image::{ImageError, Rgba};

#[allow(dead_code)]
#[derive(Debug)]
enum PossibleErrors {
    ImageOpeningError(ImageError),
    ImageSaveFailure(ImageError),
}

const FONT: &[u8] = include_bytes!("../assets/BitstreamVeraSansMonoBold-pq1a.ttf");

#[test]
fn test_example_text() -> Result<(), PossibleErrors> {
    let mut background =
        image::open("assets/background.png").map_err(PossibleErrors::ImageOpeningError)?;
    //Set up font
    let font = FontRef::try_from_slice(FONT).unwrap();
    let font_bundle = FontBundle::new(&font, PxScale { x: 40., y: 40. }, Rgba([0, 255, 0, 255]));
    //draw on image
    text_on_image_draw_debug(
        &mut background,
        "This is Line 1
        Thisislinewithextralongtextthatneedsto wrap 2",
        &font_bundle,
        400,
        800,
        &TextJustify::Center,
        &VerticalAnchor::Center,
        &WrapBehavior::Wrap(250),
    );
    //save image
    background
        .save("./output/test_example_text.png")
        .map_err(PossibleErrors::ImageSaveFailure)?;
    Ok(())
}

#[test]
#[should_panic = "scale.x or scale.y cannot be <= 0.0"]
fn test_negative_scale() {
    let font = FontRef::try_from_slice(FONT).unwrap();
    let _font_bundle = FontBundle::new(&font, PxScale { x: -40., y: 40. }, Rgba([0, 255, 0, 255]));
}
