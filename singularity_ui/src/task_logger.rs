// //! TODO: move this somewhere else

// use std::time::Instant;

// pub struct TaskLogger {
//     start_time: Instant,
//     task_name: String,
// }
// impl TaskLogger {
//     pub fn start(task_name: impl ToString) -> Self {
//         let task_name = task_name.to_string();

//         println!("Starting {task_name}.");

//         Self {
//             task_name,
//             start_time: Instant::now(),
//         }
//     }
// }
