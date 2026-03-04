// use alacritty_terminal::Term;
// use alacritty_terminal::event::EventListener;
// use alacritty_terminal::grid::Dimensions;
// use alacritty_terminal::index::Column;
// use alacritty_terminal::index::Line;
// use alacritty_terminal::term::Config;
// use alacritty_terminal::term::test::TermSize;
// use alacritty_terminal::vte::ansi::Handler;
// use singularity_common::sync::EncapsulatedLock;
// use singularity_common::utils::tree::world_tree::WorldTreePath;
// use singularity_common::utils::tree::world_tree::world_tree_traversal::WorldTreeTraversalOperation;
// use singularity_sar::applet::BasicApplet;
// use singularity_sar::applet::BasicRunnerHook;
// use singularity_sttk::nodular_applet::recursive_node_applet::RecursiveNodeApplet;
// use singularity_sttk::nodular_applet::{NodularApplet, NodularRunnerHook};
// use singularity_ui::ui_element::CharCell;
// use singularity_ui::ui_element::CharGrid;
// use singularity_ui::{
//     color::Color,
//     ui_element::UIElement,
//     ui_event::{KeyModifiers, KeyTrait, UIEvent},
// };

// struct TerminalEventListener {
//     title: EncapsulatedLock<String>,
// }
// impl EventListener for TerminalEventListener {
//     fn send_event(&self, event: alacritty_terminal::event::Event) {
//         match event {
//             alacritty_terminal::event::Event::Title(new_title) => {
//                 self.title.set(new_title);
//             }
//             alacritty_terminal::event::Event::ResetTitle => {
//                 self.title.set("Terminal".to_string());
//             }
//             // alacritty_terminal::event::Event::MouseCursorDirty => todo!(),
//             // alacritty_terminal::event::Event::ClipboardStore(clipboard_type, _) => todo!(),
//             // alacritty_terminal::event::Event::ClipboardLoad(clipboard_type, _) => todo!(),
//             // alacritty_terminal::event::Event::ColorRequest(_, _) => todo!(),
//             // alacritty_terminal::event::Event::PtyWrite(_) => todo!(),
//             // alacritty_terminal::event::Event::TextAreaSizeRequest(_) => todo!(),
//             // alacritty_terminal::event::Event::CursorBlinkingChange => todo!(),
//             // alacritty_terminal::event::Event::Wakeup => todo!(),
//             // alacritty_terminal::event::Event::Bell => todo!(),
//             // alacritty_terminal::event::Event::Exit => todo!(),
//             // alacritty_terminal::event::Event::ChildExit(_) => todo!(),
//             _ => {}
//         }
//     }
// }

// /// Terminal Applet based on [Alacritty](https://github.com/alacritty/alacritty)'s
// /// library, [`alacritty_terminal`].
// pub struct TerminalApplet {
//     hook: Box<dyn NodularRunnerHook>,
//     term: Term<TerminalEventListener>,
//     title: EncapsulatedLock<String>,
//     focused: bool,
// }
// impl TerminalApplet {
//     pub fn new(hook: Box<dyn NodularRunnerHook>) -> Self {
//         let title = EncapsulatedLock::new("Terminal".to_string());

//         TerminalApplet {
//             hook,
//             term: Term::new(
//                 Config::default(),
//                 &TermSize::new(60, 40),
//                 TerminalEventListener {
//                     title: title.clone(),
//                 },
//             ),
//             focused: true,
//             title,
//         }
//     }
//     pub fn get_initiator() -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Self {
//         |hook: Box<dyn NodularRunnerHook>| Self::new(hook)
//     }
//     pub fn get_boxed_initiator() -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>
//     {
//         |hook: Box<dyn NodularRunnerHook>| Box::new(Self::new(hook))
//     }

//     fn ansi_color_to_singularity_color(
//         color: alacritty_terminal::vte::ansi::Color,
//         colors: &alacritty_terminal::term::color::Colors,
//     ) -> Option<Color> {
//         let rgb = match color {
//             alacritty_terminal::vte::ansi::Color::Named(named_color) => colors[named_color],
//             alacritty_terminal::vte::ansi::Color::Spec(rgb) => Some(rgb),
//             alacritty_terminal::vte::ansi::Color::Indexed(index) => colors[index as usize],
//         }?;
//         Some(Color([rgb.r, rgb.g, rgb.b, u8::MAX]))
//     }
// }
// impl BasicApplet for TerminalApplet {
//     fn handle_ui_event(&mut self, ui_event: UIEvent) {
//         // println!("{ui_event:?}");
//         // self.hook.update_display(&UIElement::Backgrounded(
//         //     Box::new(UIElement::CharGrid(CharGrid::from(format!("{ui_event:?}")))),
//         //     Color::BLACK,
//         // ));
//         // self.hook.update_display(&UIElement::Text("a".to_string()));

//         if let UIEvent::KeyPress(
//             key,
//             KeyModifiers {
//                 ctrl: true,
//                 alt: false,
//                 shift: true,
//                 caps_lock: false,
//                 logo: false,
//             },
//         ) = &ui_event
//             && key.to_char() == Some('Q')
//         {
//             self.hook.close();
//             return;
//         }

//         if let UIEvent::KeyPress(
//             key,
//             KeyModifiers {
//                 ctrl: true,
//                 alt: false,
//                 shift: true,
//                 caps_lock: false,
//                 logo: false,
//             },
//         ) = &ui_event
//             && key.to_char() == Some('+')
//         {
//             // TODO
//             // self.hook
//             //     .add_child(Box::new(TextBoxApplet::get_boxed_initiator(String::new())));
//             self.hook
//                 .add_child(Box::new(RecursiveNodeApplet::get_boxed_initializer(
//                     TerminalApplet::get_boxed_initiator(),
//                 )));
//             return;
//         }

//         if let UIEvent::KeyPress(
//             key,
//             KeyModifiers {
//                 ctrl: false,
//                 alt: true,
//                 shift: false,
//                 caps_lock: false,
//                 logo: false,
//             },
//         ) = &ui_event
//             && key.to_char() == Some('q')
//         {
//             self.hook
//                 .change_focus(WorldTreeTraversalOperation::PrevLayer);
//             return;
//         }

//         // forward event to terminal
//         if let UIEvent::KeyPress(
//             key,
//             KeyModifiers {
//                 ctrl: false,
//                 alt: false,
//                 shift: _,
//                 caps_lock: false,
//                 logo: false,
//             },
//         ) = &ui_event
//             && let Some(key_char) = key.to_char()
//         {
//             self.term.input(key_char);
//         }
//         if let UIEvent::KeyPress(
//             key,
//             KeyModifiers {
//                 ctrl: false,
//                 alt: false,
//                 shift: _,
//                 caps_lock: false,
//                 logo: false,
//             },
//         ) = &ui_event
//             && let Some('\n') = key.to_char()
//         {
//             self.term.newline();
//         }
//         if let UIEvent::KeyPress(
//             key,
//             KeyModifiers {
//                 ctrl: false,
//                 alt: false,
//                 shift: _,
//                 caps_lock: false,
//                 logo: false,
//             },
//         ) = &ui_event
//             && let Some('\u{8}') = key.to_char()
//         {
//             self.term.backspace();
//         }

//         self.hook.damage_window();
//     }

//     fn get_window(&self) -> UIElement {
//         let cursor_color = if self.focused {
//             Color::LIGHT_YELLOW
//         } else {
//             Color::MEDIUM_GRAY
//         };

//         let content = self.term.renderable_content();
//         let grid = self.term.grid();
//         let colors = content.colors;

//         CharGrid {
//             content: (0..grid.screen_lines())
//                 .map(|row_index| {
//                     let row = &grid[Line(row_index as i32)];

//                     (0..grid.columns())
//                         .map(|col_index| {
//                             let cell = &row[Column(col_index)];
//                             CharCell {
//                                 character: cell.c,
//                                 // TODO: figure out how to use cell.fg and cell.bg colors
//                                 fg: Self::ansi_color_to_singularity_color(cell.fg, colors)
//                                     .unwrap_or(cursor_color),
//                                 bg: Self::ansi_color_to_singularity_color(cell.bg, colors)
//                                     .unwrap_or(Color::BLACK),
//                             }
//                         })
//                         .collect()
//                 })
//                 .collect(),
//         }
//         .element()
//         .fill_bg(Color::BLACK)

//         // for cell in self.term.grid().display_iter() {
//         //     cell.point.
//         // }
//     }
// }
// impl NodularApplet for TerminalApplet {
//     fn handle_nodular_event(
//         &mut self,
//         nodular_event: singularity_sttk::nodular_applet::NodularEvent,
//     ) {
//         match nodular_event {
//             singularity_sttk::nodular_applet::NodularEvent::Highlighted(_) => todo!(),
//             singularity_sttk::nodular_applet::NodularEvent::Focused(focus) => {
//                 self.focused = focus;
//                 println!("Yay focus {focus}!");
//                 // println!("The text is: {}", &self.textbox.get_text_as_string());

//                 self.hook.damage_window();
//             }
//         }
//     }

//     fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
//         singularity_common::utils::tree::world_tree::WorldTree::Base(self.title.get())
//     }

//     fn get_focus_path(&self) -> singularity_common::utils::tree::world_tree::WorldTreePath {
//         // WorldTreePath::new_empty()
//         WorldTreePath::new_into()
//     }
// }
