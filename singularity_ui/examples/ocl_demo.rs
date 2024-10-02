/// Exploded version. Boom!
///
/// The functions above use `ProQue` and other abstractions to greatly reduce
/// the amount of boilerplate and configuration necessary to do basic work.
/// Many tasks, however, will require more configuration and will necessitate
/// doing away with `ProQue` altogether. Enqueuing kernels and reading/writing
/// from buffers and images usually requires a more explicit interface.
///
/// The following function performs the exact same steps that the above
/// functions did, with many of the convenience abstractions peeled away.
///
/// See the function below this to take things a step deeper...
///
#[allow(dead_code)]
fn trivial_exploded() -> ocl::Result<()> {
    use ocl::{flags, Buffer, Context, Device, Kernel, Platform, Program, Queue};

    let src = r#"
        __kernel void add(__global float* buffer, float scalar) {
            buffer[get_global_id(0)] += scalar;
        }
    "#;

    // (1) Define which platform and device(s) to use. Create a context,
    // queue, and program then define some dims (compare to step 1 above).
    let platform = Platform::default();
    let device = Device::first(platform)?;
    let context = Context::builder()
        .platform(platform)
        .devices(device)
        .build()?;
    let program = Program::builder()
        .devices(device)
        .src(src)
        .build(&context)?;
    let queue = Queue::new(&context, device, None)?;
    let dims = 1 << 20;
    // [NOTE]: At this point we could manually assemble a ProQue by calling:
    // `ProQue::new(context, queue, program, Some(dims))`. One might want to
    // do this when only one program and queue are all that's needed. Wrapping
    // it up into a single struct makes passing it around simpler.

    // (2) Create a `Buffer`:
    let buffer = Buffer::<f32>::builder()
        .queue(queue.clone())
        .flags(flags::MEM_READ_WRITE)
        .len(dims)
        .fill_val(0f32)
        .build()?;

    // (3) Create a kernel with arguments matching those in the source above:
    let kernel = Kernel::builder()
        .program(&program)
        .name("add")
        .queue(queue.clone())
        .global_work_size(dims)
        .arg(&buffer)
        .arg(10.0f32)
        .build()?;

    // (4) Run the kernel (default parameters shown for demonstration purposes):
    unsafe {
        kernel
            .cmd()
            .queue(&queue)
            .global_work_offset(kernel.default_global_work_offset())
            .global_work_size(dims)
            .local_work_size(kernel.default_local_work_size())
            .enq()?;
    }

    // (5) Read results from the device into a vector (`::block` not shown):
    let mut vec = vec![0.0f32; dims];
    buffer.cmd().queue(&queue).offset(0).read(&mut vec).enq()?;

    // Print an element:
    println!("The value at index [{}] is now '{}'!", 200007, vec[200007]);
    Ok(())
}

fn main() {
    trivial_exploded().unwrap();
}
