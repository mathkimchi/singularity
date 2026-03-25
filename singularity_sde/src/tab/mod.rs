use singularity_sporg::applet_data::{AppletSpawnMethod, AppletTypeId};
use singularity_ui::{display_units::DisplayArea, ui_element::UIElement};

/// Represents Applet on the server side.
pub struct AppletHandler {
    pub applet_type_id: Option<AppletTypeId>,

    /// REVIEW: call title?
    pub tab_name: String,
    pub tab_area: DisplayArea,
    pub tab_display: UIElement,
    pub applet_session_storage: serde_json::Value,
    pub applet_spawn_method: Option<AppletSpawnMethod>,
    // pub applet_communication: AppletCommunication,
}
// impl AppletHandler {
//     /// TODO: allow setting focus
//     /// TODO: make this take ownership of spawn_data
//     pub fn spawn(spawn_data: &AppletSpawnData, tab_area: DisplayArea) -> Self {
//         let applet_communication = match &spawn_data.method {
//             AppletSpawnMethod::PipeChildProcess { program, args } => {
//                 let mut tab_spawn_command = Command::new(program)
//                     .args(args)
//                     .stdin(Stdio::piped())
//                     .stdout(Stdio::piped())
//                     .spawn()
//                     .unwrap();
//                 let byte_stream =
//                     CombinedByteStream::take_from_child(&mut tab_spawn_command).unwrap();

//                 AppletCommunication::ProcessStreamAppletCommunication(
//                     ProcessStreamAppletCommunication {
//                         tab_process: tab_spawn_command,
//                         stream: UniversalServerStream::new(byte_stream),
//                     },
//                 )
//             }
//             AppletSpawnMethod::Dylib { path } => AppletCommunication::DylibAppletCommunication(
//                 DylibAppletCommunication::load_plugin(path).unwrap(),
//             ),
//         };
//         Self {
//             applet_type_id: spawn_data.applet_type_id.clone(),

//             tab_name: String::new(),
//             tab_area,
//             tab_display: UIElement::Nothing,
//             applet_session_storage: spawn_data.initial_session_storage.clone(),
//             applet_spawn_method: Some(spawn_data.method.clone()),

//             applet_communication,
//         }
//     }

//     pub fn send_event(&mut self, event: SDEEvent) {
//         self.applet_communication.send_event_union(event);
//     }

//     #[must_use]
//     pub fn handle_incoming(
//         &mut self,
//         query_responders: &mut Vec<&mut dyn QueryDataResponder>,
//     ) -> Vec<SDERequest> {
//         // returns all pending requests (I assume that means this ends instead of waiting)
//         self.applet_communication.handle_incoming(query_responders)
//     }

//     // pub fn get_respond_channels(&self) -> &RespondChannels {
//     //     &self.tab_channels.respond_channels
//     // }

//     pub fn get_display(&self) -> &UIElement {
//         // self.tab_channels.ui_element.lock().unwrap().clone()
//         &self.tab_display
//     }

//     pub fn get_area(&self) -> DisplayArea {
//         self.tab_area
//     }

//     pub fn set_area(&mut self, new_area: DisplayArea) {
//         if self.tab_area == new_area {
//             // optimization, but might cause annoying behavior
//             return;
//         }

//         self.tab_area = new_area;

//         self.send_event(SDEEvent::DisplayEvent(DisplayEvent::Resize(ResizeEvent(
//             new_area,
//         ))));
//     }

//     // pub fn get_tab_data(&self) -> &TabData {
//     //     &self.tab_data
//     // }

//     pub fn kill(self) {
//         // all we need to kill is the communications
//         match self.applet_communication {
//             AppletCommunication::ProcessStreamAppletCommunication(
//                 mut process_stream_applet_communication,
//             ) => {
//                 process_stream_applet_communication
//                     .tab_process
//                     .kill()
//                     .unwrap();
//             }
//             AppletCommunication::DylibAppletCommunication(_dylib_applet_communication) => {
//                 // drop should automatically work
//             }
//         }
//     }
// }

// pub enum AppletCommunication {
//     ProcessStreamAppletCommunication(ProcessStreamAppletCommunication),
//     DylibAppletCommunication(DylibAppletCommunication),
// }

// // pub struct DylibAppletCommunication {
// //     // TODO: use referenece instead if performance is problem with many applets of same type open
// //     library: DylibAppletLibrary,

// //     applet_instance_bytes: *mut std::ffi::c_void,

// //     applet_context: AppletContext,
// // }
// // impl Drop for DylibAppletCommunication {
// //     fn drop(&mut self) {
// //         self.library.drop_applet_context(self.applet_instance_bytes);
// //     }
// // }

// // /// TODO: move to its own file
// // pub struct ProcessStreamAppletCommunication {
// //     tab_process: Child,

// //     stream: UniversalServerStream<CombinedByteStream<ByteReaderWrapper, ChildStdin>>,
// // }
