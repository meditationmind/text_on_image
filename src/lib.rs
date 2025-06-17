#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_truncation)]

//! A library to make placing text on images easier. Extends the functionality of the [draw_text_mut](https://docs.rs/imageproc/latest/imageproc/drawing/fn.draw_text_mut.html) function from [imageproc](https://docs.rs/imageproc/latest/imageproc/index.html).

use std::fmt::Display;

use ab_glyph::{Font, ScaleFont};
use image::{DynamicImage, ImageError};
use imageproc::drawing::{draw_text_mut, text_size};

pub use ab_glyph::{FontRef, PxScale};
pub use image::{Rgba, open};

#[derive(Debug)]
pub enum TextOnImageError {
    ImageError(ImageError),
}

/// Defines how the text extends from the point you place it.
#[derive(Default)]
pub enum TextJustify {
    Left,
    #[default]
    Center,
    Right,
}

/// Defines where the text sits relative to the vertical coordinate provided.
#[derive(Default)]
pub enum VerticalAnchor {
    Top,
    #[default]
    Center,
    Bottom,
}

/// Choose whether text wraps if it would extend beyond a specified pixel length.
#[derive(Default)]
pub enum WrapBehavior {
    #[default]
    NoWrap,
    Wrap(u32),
}
impl WrapBehavior {
    pub fn new(max_width: u32) -> Self {
        WrapBehavior::Wrap(max_width)
    }
}

/// A bundle of font-related values.
pub struct FontBundle<'a> {
    font: &'a FontRef<'a>,
    scale: PxScale,
    color: Rgba<u8>,
}

impl Display for FontBundle<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FontBundle{{font: {:?}, scale: {:?}, color: {:?}}}",
            self.font, self.scale, self.color
        )
    }
}

impl<'a> FontBundle<'a> {
    /// # Panics
    ///
    /// Will panic if [`FontBundle`] `scale.x` or `scale.y` is negative
    pub fn new(font_: &'a FontRef<'a>, scale_: PxScale, color_: Rgba<u8>) -> Self {
        assert!(
            !(scale_.x <= 0. || scale_.y <= 0.),
            "text_on_image: FontBundle scale.x or scale.y cannot be <= 0.0!"
        );
        FontBundle {
            font: font_,
            scale: scale_,
            color: color_,
        }
    }

    /// # Panics
    ///
    /// Will panic if [`FontBundle`] `scale.x` or `scale.y` is negative
    pub fn set_scale(&mut self, scale_: PxScale) {
        assert!(
            !(scale_.x <= 0. || scale_.y <= 0.),
            "text_on_image: FontBundle scale.x or scale.y cannot be <= 0.0!"
        );
        self.scale = scale_;
    }

    pub fn set_color(&mut self, color_: Rgba<u8>) {
        self.color = color_;
    }
}

#[allow(clippy::too_many_arguments)]
/// Draws text on an image with support for text jusification, vertical anchor, and line wrapping.
///
/// # Panics
///
/// Will panic if [`WrapBehavior::Wrap`] `max_width` is below 2 em
pub fn text_on_image<T: AsRef<str>>(
    image: &mut DynamicImage,
    text: T,
    font_bundle: &FontBundle<'_>,
    pixels_from_left: i32,
    pixels_from_top: i32,
    horizontal_justify: &TextJustify,
    vertical_anchor: &VerticalAnchor,
    wrap_behavior: &WrapBehavior,
) {
    let lines: Vec<&str> = text.as_ref().lines().map(str::trim).collect();
    match wrap_behavior {
        WrapBehavior::NoWrap => position_and_draw(
            image,
            &lines,
            font_bundle,
            pixels_from_left,
            pixels_from_top,
            horizontal_justify,
            vertical_anchor,
        ),
        WrapBehavior::Wrap(max_width) => {
            assert!(
                (max_width >= &get_text_width(font_bundle, "mm")),
                "text_on_image: Cannot set max_width for wrapping below 2 ems! Try setting max_width to at least {}",
                get_text_width(font_bundle, "mm")
            );
            let mut lines_altered: Vec<String> = vec![];
            for &line in &lines {
                let mut buffer: String = String::new();
                for word in line.split_whitespace() {
                    let buffer_copy = buffer.clone();
                    let buffer_width = get_text_width(font_bundle, buffer_copy.as_str());
                    let space_word = String::from(" ") + word;
                    let buffer_space_word_width =
                        buffer_width + get_text_width(font_bundle, space_word.as_str());
                    if cfg!(debug_assertions) {
                        println!(
                            "\"{buffer_copy} {word}\" has width {buffer_space_word_width}. Compare to max_width {max_width}"
                        );
                    }
                    let optional_space_width: u32 = if buffer.is_empty() {
                        get_text_width(font_bundle, " ")
                    } else {
                        0
                    };
                    if buffer_space_word_width <= max_width + optional_space_width {
                        // Add word to line
                        if cfg!(debug_assertions) {
                            println!("Word {word} gets added to line");
                        }
                        if buffer.is_empty() {
                            buffer.push_str(word);
                        } else {
                            buffer.push_str(space_word.as_str());
                        }
                    } else if buffer_space_word_width > *max_width && buffer.is_empty() {
                        // Add partial word with a dash at the end
                        let word_chars = word.chars();
                        for word_char in word_chars {
                            if (get_text_width(font_bundle, buffer.as_str())
                                + get_text_width(font_bundle, "-"))
                                <= *max_width
                            {
                                buffer.push(word_char);
                            } else {
                                buffer.push('-');
                                lines_altered.push(buffer);
                                buffer = String::from(word_char);
                            }
                        }
                    } else if buffer_space_word_width > *max_width && !buffer.is_empty() {
                        if cfg!(debug_assertions) {
                            println!("Word {word} goes over max width && buffer is not empty.");
                        }
                        // Write buffer to lines_altered, empty buffer, evaluate as new line
                        lines_altered.push(buffer);
                        buffer = String::new();
                        let word_chars = word.chars();
                        for word_char in word_chars {
                            if (get_text_width(font_bundle, buffer.as_str())
                                + get_text_width(font_bundle, "-"))
                                <= *max_width
                            {
                                buffer.push(word_char);
                            } else {
                                buffer.push('-');
                                lines_altered.push(buffer);
                                buffer = String::new();
                            }
                        }
                    }
                }
                lines_altered.push(buffer);
            }
            let lines_altered: Vec<&str> = lines_altered.iter().map(String::as_str).collect();
            if cfg!(debug_assertions) {
                println!("Lines altered:\n{lines_altered:?}");
            }
            position_and_draw(
                image,
                &lines_altered,
                font_bundle,
                pixels_from_left,
                pixels_from_top,
                horizontal_justify,
                vertical_anchor,
            );
        }
    }
}

/// Helper function to get text width.
fn get_text_width<T: AsRef<str>>(font_bundle: &FontBundle, text: T) -> u32 {
    text_size(font_bundle.scale, &font_bundle.font, text.as_ref()).0
}

/// Helper function to get text height.
fn get_text_height(font_bundle: &FontBundle) -> i32 {
    let scaled_font = font_bundle.font.as_scaled(font_bundle.scale);
    (scaled_font.ascent() - scaled_font.descent() + scaled_font.line_gap()) as i32
}

#[allow(clippy::too_many_arguments)]
/// Draws text on an image with a small cross where the coordinates are.
pub fn text_on_image_draw_debug<T: AsRef<str>>(
    image: &mut DynamicImage,
    text: T,
    font_bundle: &FontBundle<'_>,
    pixels_from_left: i32,
    pixels_from_top: i32,
    horizontal_justify: &TextJustify,
    vertical_justify: &VerticalAnchor,
    wrap_behavior: &WrapBehavior,
) {
    imageproc::drawing::draw_cross_mut(
        image,
        Rgba([255, 0, 0, 255]),
        pixels_from_left,
        pixels_from_top,
    );
    text_on_image(
        image,
        text,
        font_bundle,
        pixels_from_left,
        pixels_from_top,
        horizontal_justify,
        vertical_justify,
        wrap_behavior,
    );
}

fn position_and_draw(
    image: &mut DynamicImage,
    lines: &[&str],
    font_bundle: &FontBundle<'_>,
    pixels_from_left: i32,
    pixels_from_top: i32,
    horizontal_justify: &TextJustify,
    vertical_anchor: &VerticalAnchor,
) {
    let lines_len = lines.len().cast_signed() as i32;
    for (i, &line) in lines.iter().enumerate() {
        if cfg!(debug_assertions) {
            println!("{line} width: {}", get_text_width(font_bundle, line));
        }
        let current_line = i.cast_signed() as i32;
        let vertical_offset: i32 = match vertical_anchor {
            VerticalAnchor::Top => get_text_height(font_bundle) * current_line,
            VerticalAnchor::Center => {
                (get_text_height(font_bundle) * current_line
                    - get_text_height(font_bundle) * (lines_len - current_line))
                    / 2
            }
            VerticalAnchor::Bottom => -(get_text_height(font_bundle) * (lines_len - current_line)),
        };
        let horizontal_offset = match horizontal_justify {
            TextJustify::Left => 0,
            TextJustify::Center => get_text_width(font_bundle, line) / 2,
            TextJustify::Right => get_text_width(font_bundle, line),
        };
        draw_text_mut(
            image,
            font_bundle.color,
            pixels_from_left - horizontal_offset.cast_signed(),
            pixels_from_top + vertical_offset,
            font_bundle.scale,
            &font_bundle.font,
            line,
        );
        if cfg!(debug_assertions) {
            println!("pixels_from_left for line {line}: {pixels_from_left}");
        }
    }
}

#[cfg(test)]
mod test;
