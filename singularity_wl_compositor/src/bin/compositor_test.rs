use image::RgbaImage;
use singularity_wl_compositor::{ClientState, WaylandCompositor};
use smithay::{
    backend::renderer::{
        Bind, Color32F, Frame as _, Renderer as _,
        element::{
            Kind,
            surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
        },
        pixman::PixmanRenderer,
        utils::draw_render_elements,
    },
    reexports::{
        calloop::EventLoop,
        pixman,
        wayland_server::{ListeningSocket, protocol::wl_surface},
    },
    utils::{Rectangle, Size, Transform},
    wayland::compositor::{SurfaceAttributes, TraversalAction, with_surface_tree_downward},
};
use std::{env::set_var, sync::Arc};

fn main() {
    init_logging();

    let mut event_loop: EventLoop<WaylandCompositor> = EventLoop::try_new().unwrap();

    let mut display: smithay::reexports::wayland_server::Display<WaylandCompositor> =
        smithay::reexports::wayland_server::Display::new().unwrap();

    let mut state = WaylandCompositor::new(&mut event_loop, &mut display);

    let listener = ListeningSocket::bind("wayland-5").unwrap();

    unsafe {
        // set_var("WAYLAND_DISPLAY", &state.socket_name);
        set_var("WAYLAND_DISPLAY", "wayland-5");
    }
    std::process::Command::new("kitty").spawn().ok();

    let mut renderer = PixmanRenderer::new().unwrap();
    let mut image = pixman::Image::new(pixman::FormatCode::R8G8B8A8, 800, 600, false).unwrap();
    // let mut target = renderer.bind(&mut image).unwrap();

    // event_loop
    //     .run(None, &mut state, |state| {
    //         dbg!(state.surface.is_none());

    //         if let Some(surface) = &state.surface {
    //             // main_client.get_data();
    //             // state.display_handle.get_client(ObjectId:: main_client);
    //             // let elements: Vec<WaylandSurfaceRenderElement<_>> =
    //             //     render_elements_from_surface_tree(
    //             //         &mut renderer,
    //             //         surface,
    //             //         (0, 0),
    //             //         1.0,
    //             //         1.0,
    //             //         smithay::backend::renderer::element::Kind::Unspecified,
    //             //     );

    //             with_states(surface, |states| {
    //                 let mut binding = states.cached_state.get::<SurfaceAttributes>();
    //                 let buffer = binding.current().buffer.as_ref().unwrap();

    //                 // renderer.import_shm_buffer(buffer, Some(surface.data().unwrap()), &[])

    //                 if let BufferAssignment::NewBuffer(buffer) = buffer {
    //                     with_buffer_contents(buffer, |ptr, len, data| {
    //                         let slice = unsafe { std::slice::from_raw_parts(ptr, len) };

    //                         print!("Slice: {:?}", slice);
    //                     })
    //                     .unwrap();
    //                 }
    //             });
    //         }
    //     })
    //     .unwrap();

    loop {
        let mut target = renderer.bind(&mut image).unwrap();

        // // let size: Size<usize, smithay::utils::Physical> = Size::new(image.width(), image.height());
        // let size = target.size();
        // let damage = Rectangle::from_size(size);
        let damage = Rectangle::from_size(Size::new(800, 600));
        {
            let elements = state
                .xdg_shell_state
                .toplevel_surfaces()
                .iter()
                .flat_map(|surface| {
                    render_elements_from_surface_tree(
                        &mut renderer,
                        surface.wl_surface(),
                        (0, 0),
                        1.0,
                        1.0,
                        Kind::Unspecified,
                    )
                })
                .collect::<Vec<WaylandSurfaceRenderElement<PixmanRenderer>>>();

            let mut frame = renderer
                .render(&mut target, Size::new(800, 600), Transform::Normal)
                .unwrap();
            frame
                .clear(Color32F::new(0.1, 0.0, 0.0, 1.0), &[damage])
                .unwrap();
            draw_render_elements(&mut frame, 1.0, &elements, &[damage]).unwrap();
            // We rely on the nested compositor to do the sync for us
            let _ = frame.finish().unwrap();

            for surface in state.xdg_shell_state.toplevel_surfaces() {
                send_frames_surface_tree(
                    surface.wl_surface(),
                    state.start_time.elapsed().as_millis() as u32,
                );
            }

            if let Some(stream) = listener.accept().unwrap() {
                println!("Got a client: {:?}", stream);

                let _client = display
                    .handle()
                    .insert_client(stream, Arc::new(ClientState::default()))
                    .unwrap();
                // clients.push(client);
            }

            display.dispatch_clients(&mut state).unwrap();
            display.flush_clients().unwrap();

            let raw_image_data: Vec<_> =
                unsafe { std::slice::from_raw_parts(image.data(), image.width() * image.height()) }
                    .iter()
                    .flat_map(|pixel| pixel.to_be_bytes())
                    .collect();

            let rgba_image = RgbaImage::from_vec(800, 600, raw_image_data).unwrap();

            rgba_image.save("examples/smithay.png").unwrap();

            // save_buffer(
            //     "examples/smithay.png",
            //     &raw_image_data,
            //     800,
            //     600,
            //     ColorType::Rgba8,
            // )
            // .unwrap();

            // I need this bc if I quit while rendering, it doesn't work
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // // It is important that all events on the display have been dispatched and flushed to clients before
        // // swapping buffers because this operation may block.
        // backend.submit(Some(&[damage])).unwrap();
    }
}

fn send_frames_surface_tree(surface: &wl_surface::WlSurface, time: u32) {
    with_surface_tree_downward(
        surface,
        (),
        |_, _, &()| TraversalAction::DoChildren(()),
        |_surf, states, &()| {
            // the surface may not have any user_data if it is a subsurface and has not
            // yet been committed
            for callback in states
                .cached_state
                .get::<SurfaceAttributes>()
                .current()
                .frame_callbacks
                .drain(..)
            {
                callback.done(time);
            }
        },
        |_, _, &()| true,
    );
}

fn init_logging() {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }
}
