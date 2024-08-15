use crate::manager::ManagerProxy;
use ratatui::{buffer::Buffer, crossterm::event::Event, layout::Rect};

pub mod std_subapps;

pub struct Subapp {
    pub manager_proxy: ManagerProxy,
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
    fn render(
        &mut self,
        area: Rect,
        display_buffer: &mut Buffer,
        manager_proxy: &mut ManagerProxy,
        is_focused: bool,
    );

    /// FIXME: currently, events are not agnostic
    fn handle_input(&mut self, manager_proxy: &mut ManagerProxy, event: Event);
}
