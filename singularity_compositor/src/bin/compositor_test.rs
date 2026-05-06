use singularity_compositor::{ClientState, WaylandApplet};
use smithay::{
    backend::renderer::{
        Color32F, Frame as _, Renderer as _,
        element::{
            Kind,
            surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
        },
        pixman::PixmanRenderer,
        utils::draw_render_elements,
    },
    reexports::{
        calloop::EventLoop,
        wayland_server::{ListeningSocket, protocol::wl_surface},
    },
    utils::{Size, Transform},
    wayland::compositor::{SurfaceAttributes, TraversalAction, with_surface_tree_downward},
};
use std::{env::set_var, sync::Arc};

fn main() {
    init_logging();

    let mut event_loop: EventLoop<WaylandApplet> = EventLoop::try_new().unwrap();

    let mut display: smithay::reexports::wayland_server::Display<WaylandApplet> =
        smithay::reexports::wayland_server::Display::new().unwrap();

    let mut state = WaylandApplet::new(&mut event_loop, &mut display);

    let listener = ListeningSocket::bind("wayland-5").unwrap();

    unsafe {
        // set_var("WAYLAND_DISPLAY", &state.socket_name);
        set_var("WAYLAND_DISPLAY", "wayland-5");
    }
    std::process::Command::new("kitty").spawn().ok();

    let mut renderer = PixmanRenderer::new().unwrap();
    // let mut output = pixman::Image::new(pixman::FormatCode::A8R8G8B8, 100, 100, true).unwrap();

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
        // let size = backend.window_size();
        // let damage = Rectangle::from_size(size);
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

            // let mut frame = renderer
            //     .render(&mut framebuffer, Size::new(200, 200), Transform::Flipped180)
            //     .unwrap();
            // frame
            //     .clear(Color32F::new(0.1, 0.0, 0.0, 1.0), &[damage])
            //     .unwrap();
            // draw_render_elements(&mut frame, 1.0, &elements, &[damage]).unwrap();
            // // We rely on the nested compositor to do the sync for us
            // let _ = frame.finish().unwrap();

            for surface in state.xdg_shell_state.toplevel_surfaces() {
                send_frames_surface_tree(
                    surface.wl_surface(),
                    state.start_time.elapsed().as_millis() as u32,
                );
            }

            if let Some(stream) = listener.accept().unwrap() {
                println!("Got a client: {:?}", stream);

                let client = display
                    .handle()
                    .insert_client(stream, Arc::new(ClientState::default()))
                    .unwrap();
                // clients.push(client);
            }

            display.dispatch_clients(&mut state).unwrap();
            display.flush_clients().unwrap();
        }

        // // It is important that all events on the display have been dispatched and flushed to clients before
        // // swapping buffers because this operation may block.
        // backend.submit(Some(&[damage])).unwrap();
    }
}

pub fn send_frames_surface_tree(surface: &wl_surface::WlSurface, time: u32) {
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
