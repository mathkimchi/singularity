pub mod std_subapps;

use ratatui::{buffer::Buffer, crossterm::event::Event, layout::Rect};

pub struct Subapp {
    pub subapp_data: SubappData,
    pub user_interface: Box<dyn SubappUI>,
}

/// Only required data, the backend is in charge of this, subapps should only view this or indirectly cause change via the backend
/// Subapp specific data should be stored in whatever is implementing `SubappUI`
/// TODO
pub struct SubappData {
    // pub size: Rect,
}

pub trait SubappUI {
    fn get_title(&self) -> String;

    /// FIXME: currently, graphics are not agnostic
    ///
    /// FIXME: add element system, and give access to only the necessary parts of the buffer
    fn render(&self, area: Rect, buffer: &mut Buffer, is_focused: bool);

    /// FIXME: currently, events are not agnostic
    fn handle_input(&mut self, event: Event);
}
