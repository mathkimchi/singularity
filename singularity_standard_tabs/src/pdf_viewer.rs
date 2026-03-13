// use singularity_common::utils::tree::world_tree::WorldTreePath;
// use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
// use singularity_sttk::nodular_applet::{
//     AppletSpawner, AppletSpawnerTrait, NodularAppletInitializer,
// };
// use singularity_sttk::standard_keybinds::handle_standard_keybinds;
// use singularity_sttk::{
//     components::text_box::TextBox,
//     nodular_applet::{NodularApplet, NodularRunnerHook},
// };
// use singularity_ui::ui_event::{KeyModifiers, KeyTrait as _};
// use singularity_ui::{color::Color, ui_element::UIElement, ui_event::UIEvent};
// use std::path::PathBuf;

// /// Just turns pdf into image and then shows the image.
// pub struct PdfViewerApplet {
//     file_path: PathBuf,

//     textbox: TextBox,
//     focused: bool,

//     hook: Box<dyn NodularRunnerHook>,
// }
// impl PdfViewerApplet {
//     pub fn new<P>(file_path: P, hook: Box<dyn NodularRunnerHook>) -> Self
//     where
//         P: AsRef<std::path::Path>,
//         PathBuf: std::convert::From<P>,
//     {
//         let textbox = TextBox::new(Self::get_content(&file_path));
//         let file_path = PathBuf::from(file_path);

//         Self {
//             file_path,
//             textbox,
//             focused: true,
//             hook,
//         }
//     }
//     pub fn get_initiator<P>(file_path: P) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Self
//     where
//         P: AsRef<std::path::Path>,
//         PathBuf: std::convert::From<P>,
//     {
//         |hook: Box<dyn NodularRunnerHook>| Self::new(file_path, hook)
//     }
//     pub fn get_boxed_initiator<P>(
//         file_path: P,
//     ) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>
//     where
//         P: AsRef<std::path::Path>,
//         PathBuf: std::convert::From<P>,
//     {
//         |hook: Box<dyn NodularRunnerHook>| Box::new(Self::new(file_path, hook))
//     }
//     pub fn get_applet_spawner() -> AppletSpawner {
//         struct EditorSpawner;
//         impl AppletSpawnerTrait for EditorSpawner {
//             fn create_initializer(&self, args: &[&str]) -> Option<NodularAppletInitializer> {
//                 let file_path = args.first()?;

//                 Some(Box::new(PdfViewerApplet::get_boxed_initiator(
//                     file_path.to_string(),
//                 )))
//             }

//             fn duplicate(&self) -> AppletSpawner {
//                 Box::new(Self)
//             }
//         }
//         Box::new(EditorSpawner)
//     }

//     fn get_content(file_path: impl AsRef<std::path::Path>) -> String {
//         std::fs::read_to_string(&file_path).unwrap()
//     }

//     fn get_title(&self) -> String {
//         self.file_path
//             .file_name()
//             .unwrap()
//             .to_str()
//             .unwrap()
//             .to_string()
//     }

//     fn save_to_file(&self) {
//         // let new_path = if self.save_to_temp {
//         //     self.file_path.to_str().unwrap().to_string()
//         //         + ".temp"
//         //         + &std::time::SystemTime::now()
//         //             .duration_since(std::time::UNIX_EPOCH)
//         //             .unwrap()
//         //             .as_millis()
//         //             .to_string()
//         // } else {
//         //     self.file_path.to_str().unwrap().to_string()
//         // };

//         std::fs::write(&self.file_path, self.textbox.get_text_as_string()).unwrap();
//     }
// }
// impl BasicApplet for PdfViewerApplet {
//     fn handle_ui_event(&mut self, ui_event: UIEvent) {
//         if handle_standard_keybinds(&ui_event, &self.hook) {
//             return;
//         }

//         if let UIEvent::KeyPress(key, KeyModifiers::CTRL) = &ui_event
//             && key.to_char() == Some('s')
//         {
//             self.save_to_file();

//             return;
//         }

//         self.textbox.handle_event(ui_event);

//         self.hook.damage_window();
//         // self.hook.damage_treeview();
//     }

//     fn get_window(&self) -> UIElement {
//         let cursor_color = if self.focused {
//             Color::LIGHT_YELLOW
//         } else {
//             Color::MEDIUM_GRAY
//         };

//         self.textbox
//             .render_grid_with_color((Color::BLACK, cursor_color))
//             .element()
//             .fill_bg(Color::BLACK)
//     }
// }
// impl NodularApplet for PdfViewerApplet {
//     fn handle_nodular_event(
//         &mut self,
//         nodular_event: singularity_sttk::nodular_applet::NodularEvent,
//     ) {
//         match nodular_event {
//             singularity_sttk::nodular_applet::NodularEvent::Highlighted(_) => todo!(),
//             singularity_sttk::nodular_applet::NodularEvent::Focused(focus) => {
//                 self.focused = focus;
//                 println!("Yay focus {focus}!");
//                 println!("The text is: {}", &self.textbox.get_text_as_string());

//                 self.hook.damage_window();
//             }
//         }
//     }

//     fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
//         singularity_common::utils::tree::world_tree::WorldTree::Base(self.get_title())
//     }

//     fn get_focus_path(&self) -> singularity_common::utils::tree::world_tree::WorldTreePath {
//         // WorldTreePath::new_empty()
//         WorldTreePath::new_into()
//     }
// }
