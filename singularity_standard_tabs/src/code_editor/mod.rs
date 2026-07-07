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
use singularity_ui::{
    color::Color,
    ui_element::UIElement,
    ui_event::{Key, KeyModifiers},
};
use std::{fs::File, io::BufReader, path::PathBuf};

pub struct CodeEditorApplet {
    file_path: PathBuf,

    buffer: Rope,

    /// In char, not bytes
    cursor: usize,
    focused: bool,

    hook: Box<dyn NodularRunnerHook>,
}
impl<P> CreatableNodularApplet<P> for CodeEditorApplet
where
    P: AsRef<std::path::Path>,
    PathBuf: std::convert::From<P>,
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

            hook,
        }
    }
}
impl CodeEditorApplet {
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
}
impl BasicApplet for CodeEditorApplet {
    fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
        if handle_standard_keybinds(&ui_event, &self.hook) {
            return;
        }

        match ui_event {
            singularity_ui::ui_event::UIEvent::KeyPress(key, key_modifiers) => {
                match (key, key_modifiers) {
                    (Key::ArrowKeyLeft, KeyModifiers::NONE) => {
                        self.cursor = self.cursor.saturating_sub(1);
                    }
                    (Key::ArrowKeyRight, KeyModifiers::NONE) => {
                        self.cursor = (self.cursor + 1).min(self.buffer.len_chars() - 1);
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
                    }
                    (Key::ArrowKeyDown, KeyModifiers::NONE) => {
                        let line = self.buffer.char_to_line(self.cursor);
                        let column = self.cursor - self.buffer.line_to_char(line);

                        let new_line = (line + 1).min(self.buffer.len_lines() - 1);
                        let unclamped_new_cursor_pos = self.buffer.line_to_char(new_line) + column;

                        self.cursor = unclamped_new_cursor_pos.min(
                            self.buffer
                                .try_line_to_char(new_line + 1)
                                .unwrap_or_else(|_| self.buffer.len_chars())
                                - 1,
                        );
                    }
                    (Key::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                        self.buffer.insert_char(self.cursor, c);
                        self.cursor += 1;
                    }
                    (Key::Enter, KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                        self.buffer.insert_char(self.cursor, '\n');
                        self.cursor += 1;
                    }
                    _ => {}
                }
            }
            singularity_ui::ui_event::UIEvent::WindowResized(_display_container_size) => {}
            singularity_ui::ui_event::UIEvent::MousePress(_, _display_area) => {
                log::debug!("TODO");
            }
        }

        self.hook.damage_window();
    }

    fn get_window(
        &self,
        container_size: singularity_ui::display_units::DisplayContainerSize,
    ) -> singularity_ui::ui_element::UIElement {
        let cursor_color = if self.focused {
            Color::LIGHT_YELLOW
        } else {
            Color::MEDIUM_GRAY
        };

        // let line_idx = self.buffer.char_to_line(self.cursor);

        log::info!("Cursor position: {}", self.cursor);

        UIElement::Text(vec![
            (
                self.buffer.slice(..self.cursor).to_string(),
                UIElement::glyphon_attr(Color::WHITE),
            ),
            (
                self.buffer.char(self.cursor).to_string(),
                // TODO: right now, this changes fg color,
                // glyphon doesn't do bg so I'll have to deal w that manually later
                // might even need to do glyph rendering manually bruh (i'm cryng)
                UIElement::glyphon_attr(cursor_color),
            ),
            (
                self.buffer.slice((self.cursor + 1)..).to_string(),
                UIElement::glyphon_attr(Color::WHITE),
            ),
        ])
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
