pub mod color;
pub mod display_units;
pub mod task_logger;
pub mod ui_element;

#[cfg(feature = "wayland_backend")]
mod wayland_backend;
#[cfg(feature = "wayland_backend")]
pub use wayland_backend::ui_event;
#[cfg(feature = "wayland_backend")]
pub use wayland_backend::UIDisplay;

#[cfg(not(any(feature = "wayland_backend")))]
compile_error!("need to choose a gui backend");

#[cfg(test)]
mod test;
