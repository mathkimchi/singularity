use crate::{
    backend::utils::{RootedTree, TreeNodePath},
    subapp::{
        std_subapps::{DemoSubapp, TextReader},
        Subapp, SubappData,
    },
};
use ratatui::{
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        style::Color,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::Rect,
    prelude::CrosstermBackend,
    style::Stylize,
    widgets::{Clear, Paragraph},
    Frame, Terminal,
};
use std::{
    io::{self, stdout},
    thread::sleep,
    time::Duration,
};

struct BackendAppState {
    subapps: RootedTree<Subapp>,
}

struct FrontendAppState {
    /// App focuser is a special window
    /// This value is None if the app focuser isn't being used, if Some then it represents the index that the user wants
    app_focuser_index: Option<TreeNodePath>,
    focused_subapp: TreeNodePath,
    is_running: bool,
}

struct AppState {
    backend: BackendAppState,
    frontend: FrontendAppState,
}

pub fn run() -> io::Result<()> {
    // set up terminal stuff
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    std::panic::set_hook(Box::new(|panic_info| {
        // In case there is a panic, revert the terminal to its original state
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);

        println!("{}", panic_info);
    }));

    let mut app_state: AppState = {
        let mut subapps = RootedTree::from_root(Subapp {
            subapp_data: SubappData {},
            user_interface: DemoSubapp::box_from_title("Root"),
        });
        subapps.add_node(
            Subapp {
                subapp_data: SubappData {},
                user_interface: TextReader::subapp_from_file("examples/lorem_ipsum.txt"),
            },
            &TreeNodePath::from([]),
        );
        subapps.add_node(
            Subapp {
                subapp_data: SubappData {},
                user_interface: DemoSubapp::box_from_title("some child"),
            },
            &TreeNodePath::from([]),
        );

        AppState {
            backend: BackendAppState { subapps },
            frontend: FrontendAppState {
                app_focuser_index: None,
                focused_subapp: TreeNodePath::new_root(),
                is_running: true,
            },
        }
    };

    while app_state.frontend.is_running {
        terminal.draw(|f| draw_app(f, &app_state))?;
        handle_input(crossterm::event::read()?, &mut app_state);
    }

    // revert the terminal to its original state
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

/// returns if loop should continue
fn handle_input(event: Event, app_state: &mut AppState) {
    match event {
        Event::Key(KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('q'),
            kind: KeyEventKind::Press,
            ..
        }) => {
            println!("Sayonara!");
            sleep(Duration::from_secs_f32(0.2));
            app_state.frontend.is_running = false;
        }

        Event::Key(KeyEvent {
            modifiers: KeyModifiers::ALT,
            code,
            kind: KeyEventKind::Press,
            ..
        }) => {
            // Alt + arrows should be like alt tab for Windows and Linux but tree based
            // Alt + Enter either opens the subapp chooser or closes it and chooses the subapp

            if code == KeyCode::Enter && app_state.frontend.app_focuser_index.is_some() {
                // save tree index and close window

                let new_focus_index = app_state.frontend.app_focuser_index.take().unwrap();

                app_state.frontend.focused_subapp = new_focus_index;
            } else {
                let mut new_focus_index = app_state
                    .frontend
                    .app_focuser_index
                    .clone()
                    .unwrap_or(app_state.frontend.focused_subapp.clone());

                app_state.frontend.app_focuser_index = match code {
                    KeyCode::Enter => Some(new_focus_index),
                    KeyCode::Char('a') => {
                        new_focus_index = new_focus_index
                            .traverse_to_parent()
                            .unwrap_or(new_focus_index);
                        Some(new_focus_index)
                    }
                    KeyCode::Char('d') => {
                        new_focus_index = new_focus_index
                            .traverse_to_first_child(&app_state.backend.subapps)
                            .unwrap_or(new_focus_index);
                        Some(new_focus_index)
                    }
                    KeyCode::Char('w') => {
                        new_focus_index = new_focus_index
                            .traverse_to_previous_sibling()
                            .unwrap_or(new_focus_index);
                        Some(new_focus_index)
                    }
                    KeyCode::Char('s') => {
                        new_focus_index = new_focus_index
                            .traverse_to_next_sibling(&app_state.backend.subapps)
                            .unwrap_or(new_focus_index);
                        Some(new_focus_index)
                    }
                    _ => None,
                };
            }
        }

        Event::Key(KeyEvent {
            modifiers: KeyModifiers::SHIFT,
            code,
            kind: KeyEventKind::Press,
            ..
        }) => {
            println!("[Shift + {:?}] pressed.", code);
        }

        Event::Key(KeyEvent {
            modifiers: KeyModifiers::SHIFT,
            code,
            kind: KeyEventKind::Release,
            ..
        }) => {
            // Doesn't work
            println!("[Shift + {:?}] released.", code);
        }

        event => {
            app_state.backend.subapps[&app_state.frontend.focused_subapp]
                .user_interface
                .handle_input(event);
        }
    }
}

fn draw_app(frame: &mut Frame, app_state: &AppState) {
    frame.render_widget(Clear, frame.size());

    for (index, subapp_path) in app_state.backend.subapps.iter_paths_dfs().enumerate() {
        let subapp = &app_state.backend.subapps[&subapp_path];

        subapp.user_interface.render(
            Rect::new(2 * subapp_path.depth() as u16, (8 * index) as u16, 50, 8),
            frame.buffer_mut(),
            subapp_path == app_state.frontend.focused_subapp,
        );
    }

    if let Some(focusing_index) = &app_state.frontend.app_focuser_index {
        let num_subapps = app_state.backend.subapps.num_nodes();

        frame.render_widget(Clear, Rect::new(13, 5, 30, (2 + num_subapps) as u16));
        frame.render_widget(
            Paragraph::new("").block(ratatui::widgets::Block::bordered().title("Choose Subapp")),
            Rect::new(13, 5, 30, (2 + num_subapps) as u16),
        );

        for (index, subapp_path) in app_state.backend.subapps.iter_paths_dfs().enumerate() {
            let subapp = &app_state.backend.subapps[&subapp_path];

            let mut widget = Paragraph::new(subapp.user_interface.get_title().clone());

            if subapp_path == app_state.frontend.focused_subapp {
                widget = widget.fg(Color::Yellow);
            }

            if subapp_path == focusing_index.clone() {
                widget = widget.bg(Color::Cyan);
            }

            frame.render_widget(
                widget,
                Rect::new(
                    (13 + 1 + 2 * subapp_path.depth()) as u16,
                    (5 + 1 + index) as u16,
                    subapp.user_interface.get_title().len() as u16,
                    1,
                ),
            );
        }
    }
}
