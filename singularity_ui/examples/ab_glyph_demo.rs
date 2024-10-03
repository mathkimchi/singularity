use ab_glyph::{Font, FontRef, Glyph};
use font_kit::source::SystemSource;
use image::Rgba;
use std::path::Path;

const SAVE_IMAGES_TO_DISK: bool = true;
const BEFORE_IMAGE_FILE_NAME: &str = "./before_glyph_example_image.png";
const AFTER_IMAGE_FILE_NAME: &str = "./after_glyph_example_image.png";

/// Generates a diagonal reddish stripe and a grey background.
fn generate_image() -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    image::ImageBuffer::from_fn(512, 512, |x, y| {
        let near_midline = (x + y < 536) && (x + y > 488);

        if near_midline {
            image::Rgba([196, 50, 50, 255u8])
        } else {
            image::Rgba([50, 50, 50, 255u8])
        }
    })
}

fn main() {
    let mut img = generate_image();

    if SAVE_IMAGES_TO_DISK {
        img.save(&Path::new(BEFORE_IMAGE_FILE_NAME)).unwrap();
    }

    let font = SystemSource::new()
        .select_best_match(
            &[font_kit::family_name::FamilyName::Monospace],
            font_kit::properties::Properties::new().weight(font_kit::properties::Weight::MEDIUM),
        )
        .unwrap()
        .load()
        .unwrap();
    let font_data = font.copy_font_data().unwrap().to_vec();
    let font = FontRef::try_from_slice(&font_data).unwrap();

    // Get a glyph for 'q' with a scale & position.
    let q_glyph: Glyph = font.glyph_id('Q').with_scale(24.0);

    // Draw it.
    if let Some(q) = font.outline_glyph(q_glyph) {
        q.draw(|x, y, c| {
            *img.get_pixel_mut(x, y) = Rgba([(c * 255.) as u8, 0, 0, u8::MAX]);
        });
    }

    if SAVE_IMAGES_TO_DISK {
        img.save(&Path::new(AFTER_IMAGE_FILE_NAME)).unwrap();
        println!(
            "Images saved as: '{}' and '{}'.",
            BEFORE_IMAGE_FILE_NAME, AFTER_IMAGE_FILE_NAME
        );
    } else {
        println!(
            "Saving images to disk disabled. \
            Enable by setting 'SAVE_IMAGES_TO_DISK' to 'true'."
        );
    }
}
