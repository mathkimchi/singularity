use ab_glyph::{Font, FontArc};
use font_kit::source::SystemSource;
use image::Rgba;
use singularity_ui::task_logger::do_task;
use std::{path::Path, sync::LazyLock};

const SAVE_IMAGES_TO_DISK: bool = true;
const BEFORE_IMAGE_FILE_NAME: &str = "./before_glyph_example_image.png";
const AFTER_IMAGE_FILE_NAME: &str = "./after_glyph_example_image.png";

static FONT_DATA: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let font = SystemSource::new()
        .select_best_match(
            &[font_kit::family_name::FamilyName::Monospace],
            font_kit::properties::Properties::new().weight(font_kit::properties::Weight::MEDIUM),
        )
        .unwrap()
        .load()
        .unwrap();
    font.copy_font_data().unwrap().to_vec()
});
static FONT: LazyLock<FontArc> = LazyLock::new(|| FontArc::try_from_slice(&FONT_DATA).unwrap());

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

    for pts in [12., 24., 60.] {
        println!("Pts: {pts}");
        for i in 0..3 {
            println!("Trial: {i} with pts: {pts}");
            do_task("Trial", || {
                let q_glyph = do_task("Load", || FONT.glyph_id('Q').with_scale(pts));

                do_task("Draw", || {
                    // Draw it.
                    if let Some(q) = FONT.outline_glyph(q_glyph) {
                        q.draw(|x, y, c| {
                            *img.get_pixel_mut(x + 10, y + 10) =
                                Rgba([(c * 255.) as u8, 0, 0, u8::MAX]);
                        });
                    }
                });
            });
        }
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
