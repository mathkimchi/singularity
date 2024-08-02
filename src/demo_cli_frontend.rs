use crate::{
    backend::utils::{RootedTree, TreeNodePath},
    rooted_tree,
};
use ratatui::{
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::Rect,
    prelude::CrosstermBackend,
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
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app_state: AppState = {
        let subapps = rooted_tree!["A"=> ["B", rooted_tree!["C" => ["D"]]]];
        let focused_subapp = subapps.get_root_path();

        AppState {
            backend: BackendAppState { subapps },
            frontend: FrontendAppState {
                app_focuser_index: None,
                focused_subapp,
                is_running: true,
            },
        }
    };

    while app_state.frontend.is_running {
        terminal.draw(|f| draw_app(f, &app_state))?;
        handle_input(crossterm::event::read()?, &mut app_state);
    }

    // Undo what has been done
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
                            .traverse_to_first_child()
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
                            .traverse_to_next_sibling()
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

        _ => {}
    }
}

fn draw_app(frame: &mut Frame, app_state: &AppState) {
    frame.render_widget(Clear, frame.size());

    for (index, subapp_path) in app_state.backend.subapps.iter_paths_dfs().enumerate() {
        let subapp = &app_state.backend.subapps[&subapp_path];

        frame.render_widget(
            Paragraph::new(subapp.to_string()).block(Block::bordered().title("Subapp")),
            Rect::new(2 * subapp_path.depth() as u16, (3 * index) as u16, 5, 3),
        );

        if subapp_path == app_state.frontend.focused_subapp {
            frame.set_cursor(1, (3 * index + 1) as u16);
        }
    }

    if let Some(focusing_index) = &app_state.frontend.app_focuser_index {
        frame.render_widget(Clear, Rect::new(13, 5, 10, 5));
        frame.render_widget(
            Paragraph::new(format!("{:?}", focusing_index))
                .block(Block::bordered().title("Choose Subapp")),
            Rect::new(13, 5, 10, 5),
        );
    }
}
