use calloop::channel::Sender;
use sonamu_ui::{
    color::Color,
    ui_element::{Display, FONT_SIZE_F, Style, TrackSize, UIElement, UITree},
};
use std::{fs, process::Command, thread, time::Duration};

/// The strip at the top of the window the bar gets, in px
pub const STATUS_BAR_HEIGHT: u32 = FONT_SIZE_F as u32 + 8;

const BAR_BG: Color = Color([2, 2, 3, 0xFF]);
const BAR_TEXT: Color = Color::LIGHT_GRAY;

/// Sends the bar's text now and whenever it changes, until the receiver is gone
pub fn spawn_ticker(sender: Sender<String>) {
    thread::spawn(move || {
        let mut last = String::new();
        loop {
            let text = status_text();
            if text != last {
                if sender.send(text.clone()).is_err() {
                    return;
                }
                last = text;
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}

/// eg `2026-01-01 08:00:00PM | bat 80% | ram 42% | vol 45%`, leaving out what can't be read
fn status_text() -> String {
    [time(), battery(), ram(), volume()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" | ")
}

fn run(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn time() -> Option<String> {
    run("date", &["+%Y-%m-%d %I:%M:%S%p"])
}

fn battery() -> Option<String> {
    let capacity = fs::read_to_string("/sys/class/power_supply/BAT0/capacity").ok()?;
    Some(format!("bat {}%", capacity.trim()))
}

fn ram() -> Option<String> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
    let field = |name: &str| -> Option<u64> {
        meminfo
            .lines()
            .find_map(|line| line.strip_prefix(name)?.strip_prefix(':'))?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    };
    let total = field("MemTotal")?;
    let used = total.saturating_sub(field("MemAvailable")?);
    Some(format!("ram {}%", used * 100 / total.max(1)))
}

/// From `wpctl`, eg `Volume: 0.45`
fn volume() -> Option<String> {
    let output = run("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"])?;
    let volume: f32 = output
        .strip_prefix("Volume:")?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    let muted = if output.contains("[MUTED]") { " muted" } else { "" };
    Some(format!("vol {:.0}%{muted}", volume * 100.0))
}

/// How long `REC` is shown, then hidden, while the screen is captured
pub const REC_FLASH: Duration = Duration::from_millis(500);
const REC_WIDTH: f32 = FONT_SIZE_F * 3.0;

/// The bar, `STATUS_BAR_HEIGHT` tall. `rec` shows a red `REC` at the right end.
pub fn render(text: &str, rec: bool) -> UITree {
    let text = UITree::leaf_styled(
        UIElement::Text(vec![(text.to_string(), UIElement::glyphon_attr(BAR_TEXT))]),
        Style::absolute([8.0, 4.0], [4096.0, FONT_SIZE_F]),
    );
    let rec = if rec {
        UITree::new_styled(
            UIElement::Backgrounded(Color::RED),
            Style::full_size(),
            vec![UITree::leaf_styled(
                UIElement::Text(vec![("REC".to_string(), UIElement::glyphon_attr(Color::WHITE))]),
                Style::absolute([4.0, 4.0], [REC_WIDTH, FONT_SIZE_F]),
            )],
        )
    } else {
        UITree::leaf(UIElement::Nothing)
    };
    UITree::new_styled(
        UIElement::Backgrounded(BAR_BG),
        Style {
            display: Display::Grid,
            grid_template_columns: vec![TrackSize::Fr(1.0), TrackSize::Px(REC_WIDTH)],
            ..Style::full_size()
        },
        // the text is absolute, so out of the grid; the empty cell keeps `REC` in the right column
        vec![text, UITree::leaf(UIElement::Nothing), rec],
    )
}
