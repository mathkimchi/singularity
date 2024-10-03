use image::Rgba;
use ocl::{
    enums::{DeviceSpecifier, ImageChannelDataType, ImageChannelOrder, MemObjectType},
    prm::cl_uint,
    Context, Device, Image, Kernel, OclVec, Program, Queue,
};
use std::{path::Path, sync::LazyLock};

const SAVE_IMAGES_TO_DISK: bool = true;
const BEFORE_IMAGE_FILE_NAME: &str = "./before_example_image.png";
const AFTER_IMAGE_FILE_NAME: &str = "./after_example_image.png";

const KERNEL_SRC: &str = include_str!("shader.cl");

static CONTEXT: LazyLock<Context> = LazyLock::new(|| {
    Context::builder()
        .devices(Device::specifier().first())
        .build()
        .unwrap()
});
static DEVICE: LazyLock<Device> = LazyLock::new(|| CONTEXT.devices()[0]);
static PROGRAM: LazyLock<Program> = LazyLock::new(|| {
    Program::builder()
        .src(KERNEL_SRC)
        .devices(DEVICE.clone())
        .build(&CONTEXT)
        .unwrap()
});
/// not sure if this can be shared
static QUEUE: LazyLock<Queue> =
    LazyLock::new(|| Queue::new(&CONTEXT, DEVICE.clone(), None).unwrap());

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

enum Element {
    Triangle {
        inner_color: image::Rgba<u8>,
    },
    Rectangle {
        inner_color: image::Rgba<u8>,
        width: u32,
        height: u32,
    },
}
impl Element {
    pub fn render(&self, dst_image: &Image<u8>, dims: &(u32, u32)) {
        let kernel = match self {
            Element::Triangle { inner_color } => Kernel::builder()
                .program(&PROGRAM)
                .name("draw_rectangle")
                .queue(QUEUE.clone())
                .global_work_size(dims)
                .arg(100.0f32)
                .arg(150.0f32)
                .arg(dst_image)
                .build()
                .unwrap(),
            Element::Rectangle {
                inner_color,
                width,
                height,
            } => Kernel::builder()
                .program(&PROGRAM)
                .name("draw_rectangle")
                .queue(QUEUE.clone())
                .global_work_size(dims)
                .arg(ocl_core_vector::Uint4::new(
                    inner_color.0[0].into(),
                    inner_color.0[1].into(),
                    inner_color.0[2].into(),
                    inner_color.0[3].into(),
                ))
                .arg(width)
                .arg(height)
                .arg(dst_image)
                .build()
                .unwrap(),
        };

        println!("Attempting to run the gpu program...");
        unsafe {
            kernel.enq().unwrap();
        }
    }
}

/// Generates and image then sends it through a kernel and optionally saves.
fn main() {
    let mut img = generate_image();

    if SAVE_IMAGES_TO_DISK {
        img.save(&Path::new(BEFORE_IMAGE_FILE_NAME)).unwrap();
    }

    let dims = img.dimensions();

    let dst_image = Image::<u8>::builder()
        .channel_order(ImageChannelOrder::Rgba)
        .channel_data_type(ImageChannelDataType::UnsignedInt8)
        .image_type(MemObjectType::Image2d)
        .dims(dims)
        .copy_host_slice(&img)
        .queue(QUEUE.clone())
        .build()
        .unwrap();

    Element::Rectangle {
        inner_color: Rgba([0, 0, 255, 255]),
        width: 100,
        height: 150,
    }
    .render(&dst_image, &dims);
    Element::Rectangle {
        inner_color: Rgba([255, 0, 0, 255]),
        width: 200,
        height: 50,
    }
    .render(&dst_image, &dims);

    // put dst_image onto img
    dst_image.read(&mut img).enq().unwrap();

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
