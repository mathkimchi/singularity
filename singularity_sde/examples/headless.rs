//! THROWAWAY headless harness: drives the root like sar would.
use singularity_common::sap::{
    detached::DetachedTree,
    packets::{StandardEvent, StandardRequest, StateKind},
    shared_state::SharedState,
};
use singularity_common::utils::tree::world_tree::WorldTree;
use singularity_standard_tabs::{code_editor::CodeEditorApplet, file_manager::FileManagerApplet};
use singularity_sttk::{
    command_hub::CommandHubApplet, creatable_applet::CreatableNodularApplet,
    nodular_applet::root_node_applet::RootNodeApplet,
};
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_element::{UIElement, UITree},
    ui_event::{KeyModifiers, KeySymbol, MouseEvent, MouseEventKind, UIEvent},
};
use std::{collections::BTreeMap, time::Duration};

struct Server {
    surface: SharedState<UITree>,
    events: calloop::channel::Sender<StandardEvent>,
    plucked: Option<DetachedTree>,
    log: Vec<String>,
}

fn main() {
    let dir = std::env::temp_dir().join("sonamu_fm_test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("adir")).unwrap();
    std::fs::write(dir.join("adir/inner.txt"), "inner\n").unwrap();
    std::fs::write(dir.join("b.txt"), "hello b\n").unwrap();
    std::fs::write(dir.join("c.txt"), "hello c\n").unwrap();
    std::fs::write(dir.join("bin.dat"), [0xff, 0xfe, 0x00]).unwrap();

    let mut registry = BTreeMap::new();
    registry.insert("code_editor".into(), CodeEditorApplet::get_applet_spawner());
    registry.insert("file_manager".into(), FileManagerApplet::get_applet_spawner());
    registry.insert("command_hub".into(), CommandHubApplet::get_applet_spawner());
    let root_kind = std::env::args().nth(1).unwrap_or_default();
    let root_init = if root_kind == "hub" {
        CommandHubApplet::get_boxed_initiator()
    } else {
        FileManagerApplet::get_boxed_initiator(dir.clone())
    };
    let init = RootNodeApplet::get_initializer(root_init, registry);

    let surface = SharedState::new(StateKind::Surface, UITree::leaf(UIElement::Nothing));
    let treeview = SharedState::new(StateKind::Treeview, WorldTree::new_base(String::new()));
    let (ev_tx, ev_rx) = calloop::channel::channel();
    let (req_tx, req_rx) = calloop::channel::channel::<StandardRequest>();
    {
        let surface = surface.clone();
        std::thread::spawn(move || init.init(surface, treeview, ev_rx, req_tx));
    }

    let mut event_loop: calloop::EventLoop<Server> = calloop::EventLoop::try_new().unwrap();
    event_loop
        .handle()
        .insert_source(req_rx, |event, &mut (), server: &mut Server| {
            let calloop::channel::Event::Msg(request) = event else {
                return;
            };
            match request {
                StandardRequest::Damage(StateKind::Surface) => {
                    let _ = server.surface.get();
                    let _ = server.events.send(StandardEvent::DamageAck(StateKind::Surface));
                }
                StandardRequest::Plucked(tree) => {
                    server.log.push(format!(
                        "Plucked({})",
                        tree.as_ref()
                            .map_or("None".into(), |t| t.peek_treeview().outer_world_to_string(None))
                    ));
                    server.plucked = tree;
                }
                other => server.log.push(format!("{other:?}")),
            }
        })
        .unwrap();
    let mut server = Server {
        surface,
        events: ev_tx,
        plucked: None,
        log: Vec::new(),
    };
    let pump = |event_loop: &mut calloop::EventLoop<Server>, server: &mut Server| {
        for _ in 0..30 {
            event_loop
                .dispatch(Duration::from_millis(10), server)
                .unwrap();
        }
    };
    let send = |server: &Server, event: UIEvent| {
        server.events.send(StandardEvent::UIEvent(event)).unwrap();
    };
    let key = |symbol: KeySymbol, modifiers: KeyModifiers| UIEvent::Key {
        symbol: Some(symbol),
        modifiers,
        raw_keycode: 0,
        pressed: true,
    };
    let show = |server: &mut Server, step: &str| {
        let text = labels(&format!("{:?}", server.surface.get()));
        println!("== {step}\n  surface: {text}\n  log: {:?}", server.log);
        server.log.clear();
    };

    send(&server, UIEvent::WindowResized(DisplayContainerSize::new(2000, 1200)));
    pump(&mut event_loop, &mut server);
    show(&mut server, "start");

    let mut wl_queues = Vec::new();
    for (step, symbol, mods) in std::env::args().skip(2).map(|arg| parse(&arg)) {
        match step.as_str() {
            "pluck" => server.events.send(StandardEvent::Pluck(None)).unwrap(),
            "wait" => std::thread::sleep(Duration::from_millis(500)),
            // a new wl toplevel, with a dummy queue
            "wl" => {
                let (tx, rx) = calloop::channel::channel();
                wl_queues.push(rx);
                server
                    .events
                    .send(StandardEvent::WlSurfaceRegistered {
                        surface_id: singularity_common::sap::packets::WlSurfaceId::from_u64(
                            (1 << 32) | wl_queues.len() as u64,
                        ),
                        wl_event_queue: tx,
                    })
                    .unwrap();
            }
            // `~` is a space
            s if s.starts_with("type:") => {
                for c in s[5..].chars() {
                    let c = if c == '~' { ' ' } else { c };
                    send(&server, key(KeySymbol::Char(c), KeyModifiers::NONE));
                }
            }
            "place" => {
                let tree = server.plucked.take().expect("nothing plucked");
                server.events.send(StandardEvent::Place(None, tree)).unwrap();
            }
            s if s.starts_with("click") => {
                let xy: Vec<f64> = s[6..].split(',').map(|v| v.parse().unwrap()).collect();
                for pressed in [true, false] {
                    send(
                        &server,
                        UIEvent::Mouse(MouseEvent {
                            position: [xy[0], xy[1]],
                            kind: MouseEventKind::Button {
                                button: 0x110,
                                pressed,
                            },
                        }),
                    );
                }
            }
            s if s.starts_with("rpluck") || s.starts_with("rplace") => {
                // like sar answering a treeview drag
                let raw = &s[7..];
                let path = parse_path(raw);
                if s.starts_with("rpluck") {
                    server.events.send(StandardEvent::Pluck(Some(path))).unwrap();
                } else {
                    let tree = server.plucked.take().expect("nothing plucked");
                    server
                        .events
                        .send(StandardEvent::Place(Some(path), tree))
                        .unwrap();
                }
            }
            _ => send(&server, key(symbol.unwrap(), mods)),
        }
        pump(&mut event_loop, &mut server);
        show(&mut server, &step);
    }
}

/// `a|b|c` world path of `,`-separated indices, eg `|0` = [[], [0]]
fn parse_path(raw: &str) -> singularity_common::utils::tree::world_tree::WorldTreePath {
    use singularity_common::utils::tree::tree_node_path::TreeNodePath;
    singularity_common::utils::tree::world_tree::WorldTreePath(
        raw.split('|')
            .map(|part| {
                TreeNodePath(
                    part.split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.parse().unwrap())
                        .collect(),
                )
            })
            .collect(),
    )
}

fn parse(arg: &str) -> (String, Option<KeySymbol>, KeyModifiers) {
    let (mods, name) = match arg.rsplit_once('+') {
        Some((mods, name)) if !name.is_empty() => (mods, name),
        _ => ("", arg),
    };
    let modifiers = KeyModifiers {
        ctrl: mods.contains("C"),
        alt: mods.contains("A"),
        shift: mods.contains("S"),
        logo: mods.contains("L"),
        ..KeyModifiers::NONE
    };
    let symbol = match name {
        "right" => Some(KeySymbol::ArrowKeyRight),
        "bs" => Some(KeySymbol::Backspace),
        "pgup" => Some(KeySymbol::PageUp),
        "pgdn" => Some(KeySymbol::PageDown),
        "space" => Some(KeySymbol::Char(' ')),
        "down" => Some(KeySymbol::ArrowKeyDown),
        "up" => Some(KeySymbol::ArrowKeyUp),
        "enter" => Some(KeySymbol::Enter),
        "esc" => Some(KeySymbol::Escape),
        "tab" => Some(KeySymbol::Char('\t')),
        "left" => Some(KeySymbol::ArrowKeyLeft),
        n if n.chars().count() == 1 => n.chars().next().map(KeySymbol::Char),
        _ => None,
    };
    (arg.to_string(), symbol, modifiers)
}

/// The text strings in a debug dump, then each char grid's non-blank rows
fn labels(debug: &str) -> String {
    let mut out = Vec::new();
    let mut grids = Vec::new();
    let mut rest = debug;
    while let Some(start) = rest.find("CharGrid { width: ") {
        rest = &rest[start + 18..];
        let width: usize = rest[..rest.find(',').unwrap()].parse().unwrap();
        let height_at = rest.find("height: ").unwrap() + 8;
        let height: usize = rest[height_at..height_at + rest[height_at..].find(',').unwrap()]
            .parse()
            .unwrap();
        let mut chars = Vec::new();
        for _ in 0..width * height {
            let at = rest.find("character: ").unwrap() + 11;
            rest = &rest[at..];
            let end = rest.find(',').unwrap();
            chars.push(char::from_u32(rest[..end].parse().unwrap()).unwrap_or('?'));
        }
        for row in chars.chunks(width.max(1)) {
            let line: String = row.iter().collect();
            if !line.trim().is_empty() {
                grids.push(format!("\n    |{}", line.trim_end()));
            }
        }
    }
    let debug = debug.replace("CharGrid", "");
    let debug = debug.as_str();
    let mut rest = debug;
    while let Some(start) = rest.find("(\"") {
        rest = &rest[start + 2..];
        let end = rest.find('"').unwrap_or(rest.len());
        out.push(&rest[..end]);
        rest = &rest[end..];
    }
    out.join(" | ") + &grids.concat()
}
