// use std::process::Command;

// use singularity_common::utils::usock_tools::UnixServerHost;
// use singularity_sap::{
//     standard_packets::display_packets::{DisplayEvent, FocusedEvent},
//     universal_stream::universal_server_stream::UniversalServerStream,
// };

// #[test]
// fn run() {
//     println!("Hi from server");
//     let server_host = UnixServerHost::bind_new().unwrap();
//     let client_command = Command::new("cargo")
//         .args([
//             "run",
//             "--package",
//             "singularity_standard_tabs",
//             "--bin",
//             "print_test_tab",
//             // "--",
//             // "run",
//             // "--exact",
//             // "--show-output",
//         ])
//         .spawn()
//         .unwrap();
//     println!("Server side: spawned client");

//     let mut server_stream = UniversalServerStream::new(server_host.listener.accept().unwrap().0);
//     println!("Server side: accepted client conn");

//     server_stream.send_event_union(DisplayEvent::Focused(FocusedEvent)); // should not be parsed
//     server_stream.send_event(FocusedEvent);
//     println!("Server side: sent events");

//     client_command.wait_with_output().unwrap();
//     println!("Server side: finished");
// }
