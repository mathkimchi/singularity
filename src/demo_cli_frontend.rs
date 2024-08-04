use crate::backend::utils::{RootedTree, TreeNodePath};
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
    widgets::{Block, Clear, Paragraph},
    Frame, Terminal,
};
use std::{
    io::{self, stdout},
    thread::sleep,
    time::Duration,
};

struct BackendAppState {
    subapps: RootedTree<String>,
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
        let mut subapps = RootedTree::from_root("E".to_string());
        subapps.add_node("0".to_string(), &TreeNodePath::from([]));
        subapps.add_node("00".to_string(), &TreeNodePath::from([0]));
        subapps.add_node("1".to_string(), &TreeNodePath::from([]));
        subapps.add_node("10".to_string(), &TreeNodePath::from([1]));
        subapps.add_node("11".to_string(), &TreeNodePath::from([1]));
        subapps.add_node("110".to_string(), &TreeNodePath::from([1, 1]));
        subapps.add_node("2".to_string(), &TreeNodePath::from([]));

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

        _ => {
            // TODO: send event to focused subapp
        }
    }
}

fn draw_app(frame: &mut Frame, app_state: &AppState) {
    frame.render_widget(Clear, frame.size());

    // for (path, i) in app_state
    //     .backend
    //     .subapps
    //     .iter_paths_dfs()
    //     .zip([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
    // {
    //     dbg!(path);
    //     dbg!(i);
    // }

    for (index, subapp_path) in app_state.backend.subapps.iter_paths_dfs().enumerate() {
        let subapp = &app_state.backend.subapps[&subapp_path];

        let mut widget =
            Paragraph::new(subapp.to_string()).block(Block::bordered().title("Subapp"));

        if subapp_path == app_state.frontend.focused_subapp {
            frame.set_cursor((2 * subapp_path.depth() + 1) as u16, (3 * index + 1) as u16);
        }

        if let Some(focusing_index) = &app_state.frontend.app_focuser_index {
            if subapp_path == focusing_index.clone() {
                widget = widget.bg(Color::Cyan);
            }
        }

        frame.render_widget(
            widget,
            Rect::new(2 * subapp_path.depth() as u16, (3 * index) as u16, 5, 3),
        );
    }

    if let Some(focusing_index) = &app_state.frontend.app_focuser_index {
        frame.render_widget(Clear, Rect::new(13, 5, 10, 5));
        frame.render_widget(
            Paragraph::new(format!("{:?}", focusing_index))
                .block(Block::bordered().title("Choose Subapp")),
            Rect::new(13, 5, 30, 5),
        );
    }
}
