use ocl::enums::{
    AddressingMode, FilterMode, ImageChannelDataType, ImageChannelOrder, MemObjectType,
};
use ocl::{Context, Device, Image, Kernel, Program, Queue, Sampler};
use std::path::Path;

const SAVE_IMAGES_TO_DISK: bool = true;
const BEFORE_IMAGE_FILE_NAME: &str = "./before_example_image.png";
const AFTER_IMAGE_FILE_NAME: &str = "./after_example_image.png";

const KERNEL_SRC: &str = include_str!("shader.cl");

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

/// Generates and image then sends it through a kernel and optionally saves.
fn main() {
    let mut img = generate_image();

    if SAVE_IMAGES_TO_DISK {
        img.save(&Path::new(BEFORE_IMAGE_FILE_NAME)).unwrap();
    }

    let context = Context::builder()
        .devices(Device::specifier().first())
        .build()
        .unwrap();
    let device = context.devices()[0];
    let queue = Queue::new(&context, device, None).unwrap();

    let program = Program::builder()
        .src(KERNEL_SRC)
        .devices(device)
        .build(&context)
        .unwrap();

    let sup_img_formats = Image::<u8>::supported_formats(
        &context,
        ocl::flags::MEM_READ_WRITE,
        MemObjectType::Image2d,
    )
    .unwrap();
    println!("Image formats supported: {}.", sup_img_formats.len());
    // println!("Image Formats: {:#?}.", sup_img_formats);

    let dims = img.dimensions();

    let src_image = Image::<u8>::builder()
        .channel_order(ImageChannelOrder::Rgba)
        .channel_data_type(ImageChannelDataType::UnormInt8)
        .image_type(MemObjectType::Image2d)
        .dims(dims)
        .flags(
            ocl::flags::MEM_READ_ONLY
                | ocl::flags::MEM_HOST_WRITE_ONLY
                | ocl::flags::MEM_COPY_HOST_PTR,
        )
        .copy_host_slice(&img)
        .queue(queue.clone())
        .build()
        .unwrap();

    let dst_image = Image::<u8>::builder()
        .channel_order(ImageChannelOrder::Rgba)
        .channel_data_type(ImageChannelDataType::UnormInt8)
        .image_type(MemObjectType::Image2d)
        .dims(dims)
        .flags(
            ocl::flags::MEM_WRITE_ONLY
                | ocl::flags::MEM_HOST_READ_ONLY
                | ocl::flags::MEM_COPY_HOST_PTR,
        )
        .copy_host_slice(&img)
        .queue(queue.clone())
        .build()
        .unwrap();

    // Not sure why you'd bother creating a sampler on the host but here's how:
    let sampler = Sampler::new(&context, false, AddressingMode::None, FilterMode::Nearest).unwrap();

    let kernel = Kernel::builder()
        .program(&program)
        .name("draw_rectangle")
        .queue(queue.clone())
        .global_work_size(dims)
        .arg(100.0 as f32)
        .arg(150.0 as f32)
        .arg(&dst_image)
        .build()
        .unwrap();

    println!("Printing image info:");
    println!("Src: {}", src_image);
    println!();
    println!("Dest: {}", src_image);
    println!();

    println!("Attempting to run the gpu program...");
    unsafe {
        kernel.enq().unwrap();
    }

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
