use ropey::Rope;
use singularity_sar::applet::BasicApplet;
use singularity_sttk::{
    creatable_applet::CreatableNodularApplet,
    nodular_applet::{
        AppletSpawner, AppletSpawnerTrait, NodularApplet, NodularAppletInitializer,
        NodularRunnerHook, recursive_node_applet::RecursiveNodeApplet,
    },
    standard_keybinds::handle_standard_keybinds,
};
use sonamu_ui::{
    color::Color,
    ui_element::{CharCellStyle, CharGrid},
    ui_event::{Key, KeyModifiers},
};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

/// Handles scroll offset
/// TODO: fold, lines that span multiple rows, in-line embeds like comments or document embeds (for custom note-taking language), horizontal scroll
struct ViewOffset {
    /// What text line is the uppermost row?
    scroll: usize,

    most_recent_num_rows: Option<usize>,
}
impl ViewOffset {
    // TODO: make something that iterates the line_idx's?
    const fn disp_row_to_line_idx(&self, display_row: usize) -> usize {
        display_row + self.scroll
    }

    /// Positive is down
    /// Negative is up
    fn add_scroll(&mut self, offset: isize) {
        self.scroll = self.scroll.saturating_add_signed(offset);
    }

    fn clamp_to_cursor(&mut self, cursor_line: usize) {
        // the top row displayed (scroll) should be at cursor_line or above (minus)
        self.scroll = self.scroll.clamp(
            if let Some(num_rows) = self.most_recent_num_rows {
                (cursor_line + 1).saturating_sub(num_rows)
            } else {
                0
            },
            cursor_line,
        );
    }
}

#[derive(Debug, Clone, Copy)]
enum EditorMode {
    Normal,
    Insert,
}

pub struct CodeEditorApplet {
    file_path: PathBuf,

    buffer: Rope,

    /// In char, not bytes
    cursor: usize,
    focused: bool,
    mode: EditorMode,

    view_offset: ViewOffset,

    hook: Box<dyn NodularRunnerHook>,
}
impl<P> CreatableNodularApplet<P> for CodeEditorApplet
where
    P: AsRef<std::path::Path>,
    PathBuf: From<P>,
{
    fn new(file_path: P, hook: Box<dyn NodularRunnerHook + 'static>) -> Self {
        // Is reader overkill? Should I just have read?
        let buffer = Rope::from_reader(BufReader::new(File::open(&file_path).unwrap())).unwrap();
        let file_path = PathBuf::from(file_path);

        Self {
            file_path,
            buffer,

            cursor: 0,
            focused: true,
            mode: EditorMode::Normal,

            view_offset: ViewOffset {
                scroll: 0,
                most_recent_num_rows: None,
            },

            hook,
        }
    }
}
impl CodeEditorApplet {
    #[must_use]
    pub fn get_applet_spawner() -> AppletSpawner {
        struct EditorSpawner;
        impl AppletSpawnerTrait for EditorSpawner {
            fn create_initializer(&self, args: &[&str]) -> Option<NodularAppletInitializer> {
                let file_path = args.first()?;

                Some(RecursiveNodeApplet::boxed_get_boxed_initializer(
                    CodeEditorApplet::get_boxed_initiator(file_path.to_string()),
                ))
            }

            fn duplicate(&self) -> AppletSpawner {
                Box::new(Self)
            }
        }
        Box::new(EditorSpawner)
    }

    fn get_title(&self) -> String {
        self.file_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    fn save_buffer(&self) {
        let mut dest = BufWriter::new(File::create(&self.file_path).unwrap());
        self.buffer.write_to(&mut dest).unwrap();
        dest.flush().unwrap();
    }

    fn clamp_view_to_cursor(&mut self) {
        self.view_offset
            .clamp_to_cursor(self.buffer.char_to_line(self.cursor));
    }

    // fn set_cursor(&mut self, new_cursor: usize) {
    //     self.cursor = new_cursor;
    //     self.clamp_view_to_cursor();
    // }

    fn handle_keypress_normal(&mut self, key: Key, key_modifiers: KeyModifiers) {
        match (key, key_modifiers) {
            (Key::Char('i'), KeyModifiers::NONE) => {
                self.mode = EditorMode::Insert;
            }
            (Key::Char('a'), KeyModifiers::NONE) => {
                self.mode = EditorMode::Insert;
                // TODO: do this with actions system (will need to make an actions system)
                self.cursor = (self.cursor + 1).min(self.buffer.len_chars());
                self.clamp_view_to_cursor();
            }
            _ => {}
        }
    }

    fn handle_keypress_insert(&mut self, key: Key, key_modifiers: KeyModifiers) {
        match (key, key_modifiers) {
            (Key::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.buffer.insert_char(self.cursor, c);
                self.cursor += 1;
                self.clamp_view_to_cursor();
            }
            (Key::Enter, KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.buffer.insert_char(self.cursor, '\n');
                self.cursor += 1;
                self.clamp_view_to_cursor();
            }
            (Key::Backspace, KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                if let Some(prev_idx) = self.cursor.checked_sub(1) {
                    self.buffer.remove(prev_idx..self.cursor);
                    self.cursor = prev_idx;
                    self.clamp_view_to_cursor();
                }
            }
            (Key::Escape, _) => {
                self.mode = EditorMode::Normal;
            }
            (Key::Char('v'), KeyModifiers::CTRL) | (Key::Char('V'), KeyModifiers::CTRL_SHIFT) => {
                if let Ok(text) = arboard::Clipboard::new().unwrap().get_text() {
                    self.buffer.insert(self.cursor, &text);
                    self.cursor += text.len();
                    self.clamp_view_to_cursor();
                }
            }
            _ => {}
        }
    }

    fn handle_keypress(&mut self, key: Key, key_modifiers: KeyModifiers) {
        match (key, key_modifiers) {
            // global keypresses
            (Key::Char('s'), KeyModifiers::CTRL) => {
                // log::info!("Saving!");
                self.save_buffer();
            }
            (Key::ArrowKeyLeft, KeyModifiers::NONE) => {
                self.cursor = self.cursor.saturating_sub(1);
                self.clamp_view_to_cursor();
            }
            (Key::ArrowKeyRight, KeyModifiers::NONE) => {
                self.cursor = (self.cursor + 1).min(self.buffer.len_chars());
                self.clamp_view_to_cursor();
            }
            (Key::ArrowKeyUp, KeyModifiers::NONE) => {
                let line = self.buffer.char_to_line(self.cursor);
                let column = self.cursor - self.buffer.line_to_char(line);

                let new_line = line.saturating_sub(1);
                let unclamped_new_cursor_pos = self.buffer.line_to_char(new_line) + column;

                self.cursor = unclamped_new_cursor_pos.min(
                    self.buffer
                        .try_line_to_char(new_line + 1)
                        .unwrap_or_else(|_| self.buffer.len_chars())
                        - 1,
                );
                self.clamp_view_to_cursor();
            }
            (Key::ArrowKeyDown, KeyModifiers::NONE) => {
                let line = self.buffer.char_to_line(self.cursor);
                let column = self.cursor - self.buffer.line_to_char(line);

                let new_line = (line + 1).min(self.buffer.len_lines() - 1);
                let unclamped_new_cursor_pos = self.buffer.line_to_char(new_line) + column;

                self.cursor = unclamped_new_cursor_pos.min(
                    self.buffer
                        .try_line_to_char(new_line + 1)
                        .unwrap_or_else(|_| self.buffer.len_chars()),
                );
                self.clamp_view_to_cursor();
            }
            // these are mostly here for debug purposes
            (Key::PageDown, KeyModifiers::NONE) => self.view_offset.add_scroll(1),
            (Key::PageUp, KeyModifiers::NONE) => self.view_offset.add_scroll(-1),
            _ => match self.mode {
                EditorMode::Normal => self.handle_keypress_normal(key, key_modifiers),
                EditorMode::Insert => self.handle_keypress_insert(key, key_modifiers),
            },
        }
    }
}
impl BasicApplet for CodeEditorApplet {
    fn handle_ui_event(&mut self, ui_event: sonamu_ui::ui_event::UIEvent) {
        if handle_standard_keybinds(&ui_event, &self.hook) {
            return;
        }

        match ui_event {
            sonamu_ui::ui_event::UIEvent::KeyPress(key, key_modifiers) => {
                self.handle_keypress(key, key_modifiers);
            }
            sonamu_ui::ui_event::UIEvent::WindowResized(display_container_size) => {
                let (_width, height) = CharGrid::largest_fittable_size(display_container_size);

                self.view_offset.most_recent_num_rows = Some(height);
            }
            sonamu_ui::ui_event::UIEvent::MousePress(_, _display_area) => {
                log::debug!("TODO");
            }
        }

        self.hook.damage_window();
    }

    fn get_window(
        &self,
        container_size: sonamu_ui::display_units::DisplayContainerSize,
    ) -> sonamu_ui::ui_element::UIElement {
        let (width, height) = CharGrid::largest_fittable_size(container_size);

        // let line_idx = self.buffer.char_to_line(self.cursor);
        // log::info!("Cursor position: {}", self.cursor);

        // From ropey documentation: `char_idx` can be one-past-the-end, which will return the last line index.
        let cursor_line = self.buffer.char_to_line(self.cursor);

        let mut content = CharGrid::new_empty(width, height);
        for row in 0..height {
            let line_idx = self.view_offset.disp_row_to_line_idx(row);

            let Some(line) = self.buffer.get_line(line_idx) else {
                break;
            };

            for (col, mut c) in line.chars().take(width).enumerate() {
                if c == '\n' {
                    // we can't automatically rule out all final characters, bc of the final line
                    c = ' ';
                }

                content.set_char(c, row, col);

                // // currently scroll is only horizontal
                // // ^- Erhm actchually, currently, scroll DNE. But the architecture for it only supports horizontal
                // //    ^- Erhm actchuahllie, scroll exists now
                // let content_idx = self.buffer.line_to_char(line_idx) + col;
            }

            if line_idx == cursor_line {
                let cursor_col = self.cursor - self.buffer.line_to_char(line_idx);
                if cursor_col < width {
                    match self.mode {
                        EditorMode::Normal => {
                            let cursor_bg = if self.focused {
                                Color::ORANGE
                            } else {
                                Color::MEDIUM_GRAY
                            };
                            content
                                .get_char_mut(row, cursor_col)
                                .set_fg(Color::TRANSPARENT)
                                .set_bg(cursor_bg);
                        }
                        EditorMode::Insert => {
                            content
                                .get_char_mut(row, cursor_col)
                                .add_style(CharCellStyle::CURSOR_LINE);
                            // log::debug!(
                            //     "Cursor style: {}",
                            //     (content.get_char(row, col).style.0 >> 2) & 1
                            // );
                        }
                    }
                }
            }
        }

        /// I think this is good, but it might be a tad bid disorienting to resize a lot (it feels jiggly)
        const EXPAND_EDITOR_TEXT: bool = true;

        if EXPAND_EDITOR_TEXT {
            content.element()
        } else {
            content.contained_element()
        }
    }
}
impl NodularApplet for CodeEditorApplet {
    fn handle_nodular_event(
        &mut self,
        nodular_event: singularity_sttk::nodular_applet::NodularEvent,
    ) {
        match nodular_event {
            singularity_sttk::nodular_applet::NodularEvent::Highlighted(_) => todo!(),
            singularity_sttk::nodular_applet::NodularEvent::Focused(focus) => {
                self.focused = focus;
                // println!("Yay focus {focus}!");
                // println!("The text is: {}", &self.buffer.);

                self.hook.damage_window();
            }
        }
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        singularity_common::utils::tree::world_tree::WorldTree::Base(self.get_title())
    }

    fn get_focus_path(&self) -> singularity_common::utils::tree::world_tree::WorldTreePath {
        // WorldTreePath::new_empty()
        singularity_common::utils::tree::world_tree::WorldTreePath::new_into()
    }
}
