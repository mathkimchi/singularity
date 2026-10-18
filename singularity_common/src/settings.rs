//! User settings, read from the config dir. See `.claude/SETTINGS.md`.

use notify::Watcher as _;
use serde::Deserialize;
use sonamu_ui::color::Color;
use std::{
    collections::HashSet,
    fs, io,
    io::Read,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::{
        LazyLock, Mutex, PoisonError, RwLock,
        mpsc::{self, Receiver},
    },
    thread,
    time::{Duration, Instant},
};

const PROGRAM_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub palette: Palette,
    pub border: Border,
    pub gap: f32,
    pub radius: f32,
    pub font_size: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            palette: Palette::default(),
            border: Border::default(),
            gap: 0.0,
            radius: 0.0,
            font_size: 16.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct Palette {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub inactive: Color,
}
impl Default for Palette {
    fn default() -> Self {
        Self {
            bg: Color::BLACK,
            fg: Color::LIGHT_GRAY,
            accent: Color::LIGHT_YELLOW,
            inactive: Color::MEDIUM_GRAY,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct Border {
    pub width: f32,
    pub focused: Color,
    pub unfocused: Color,
}
impl Default for Border {
    fn default() -> Self {
        Self {
            width: 1.0,
            focused: Color::LIGHT_YELLOW,
            unfocused: Color::LIGHT_GREEN,
        }
    }
}

/// What the settings file or program gives.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Loaded {
    #[serde(flatten)]
    pub settings: Settings,
    /// Extra files to watch for changes, eg the ones the program reads.
    #[serde(default)]
    pub watch: Vec<PathBuf>,
}

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Timeout,
    Exit(ExitStatus),
    Json(serde_json::Error),
}
impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Timeout => write!(f, "settings program timed out after {PROGRAM_TIMEOUT:?}"),
            Self::Exit(status) => write!(f, "settings program failed: {status}"),
            Self::Json(err) => write!(f, "invalid settings: {err}"),
        }
    }
}
impl std::error::Error for LoadError {}
impl From<io::Error> for LoadError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
impl From<serde_json::Error> for LoadError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

/// `$SONAMU_CONFIG`, else `$XDG_CONFIG_HOME/sonamu`, else `~/.config/sonamu`.
pub fn config_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("SONAMU_CONFIG") {
        return (!dir.is_empty()).then(|| dir.into());
    }
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(config.join("sonamu"))
}

/// Runs `settings` in `dir` if it is executable, else reads `settings.json`.
/// Defaults if there is neither.
pub fn load(dir: &Path) -> Result<Loaded, LoadError> {
    let program = dir.join("settings");
    let text = if is_executable(&program) {
        run(&program)?
    } else {
        match fs::read_to_string(dir.join("settings.json")) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Loaded::default()),
            Err(err) => return Err(err.into()),
        }
    };
    Ok(serde_json::from_str(&text)?)
}

fn is_executable(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

fn run(program: &Path) -> Result<String, LoadError> {
    let mut child = Command::new(program)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdout = child.stdout.take().expect("stdout is piped");
    // read while waiting, so a full pipe can't block the program
    let reader = thread::spawn(move || {
        let mut text = String::new();
        stdout.read_to_string(&mut text).map(|_| text)
    });

    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > PROGRAM_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(LoadError::Timeout);
        }
        thread::sleep(Duration::from_millis(10));
    };
    if !status.success() {
        return Err(LoadError::Exit(status));
    }
    Ok(reader.join().expect("settings reader panicked")?)
}

static CURRENT: LazyLock<RwLock<Settings>> = LazyLock::new(Default::default);
static SUBSCRIBERS: Mutex<Vec<calloop::channel::Sender<()>>> = Mutex::new(Vec::new());

/// The current settings.
pub fn get() -> Settings {
    CURRENT
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// `sender` gets a message whenever the settings change.
pub fn subscribe(sender: calloop::channel::Sender<()>) {
    SUBSCRIBERS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(sender);
}

/// Loads the settings into [`get`]. On failure, keeps the current ones and logs.
pub fn reload() {
    if let Some(dir) = config_dir() {
        reload_from(&dir);
    }
}

/// Returns the extra paths to watch, `None` on failure.
fn reload_from(dir: &Path) -> Option<Vec<PathBuf>> {
    match load(dir) {
        Ok(Loaded { settings, watch }) => {
            let changed = {
                let mut current = CURRENT.write().unwrap_or_else(PoisonError::into_inner);
                std::mem::replace(&mut *current, settings) != *current
            };
            if changed {
                // drops the subscribers that are gone
                SUBSCRIBERS
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .retain(|sender| sender.send(()).is_ok());
            }
            Some(watch.into_iter().map(|path| dir.join(path)).collect())
        }
        Err(err) => {
            log::warn!("Keeping previous settings, could not load from {}: {err}", dir.display());
            None
        }
    }
}

/// Loads the settings, then reloads them on a thread whenever the settings file or program,
/// or a file it asked to watch, changes.
pub fn load_and_watch() {
    let Some(dir) = config_dir() else {
        log::warn!("No config dir, using default settings");
        return;
    };
    let extra = reload_from(&dir);
    let (event_sender, events) = mpsc::channel();
    let watcher = match notify::recommended_watcher(event_sender) {
        Ok(watcher) => watcher,
        Err(err) => {
            log::warn!("Settings won't live reload: {err}");
            return;
        }
    };
    let mut watcher = Watcher {
        watcher,
        dir,
        dirs: HashSet::new(),
        extra: HashSet::new(),
        ignored: HashSet::new(),
    };
    watcher.sync(extra.unwrap_or_default());
    thread::spawn(move || watcher.run(&events));
}

/// How long changes must stop before reloading
const SETTLE: Duration = Duration::from_millis(100);

struct Watcher {
    watcher: notify::RecommendedWatcher,
    dir: PathBuf,
    /// Files are watched through their dirs, so files replaced by a rename are seen too
    dirs: HashSet<PathBuf>,
    /// The extra paths from the last load
    extra: HashSet<PathBuf>,
    /// Extra paths the program changes itself, which would reload forever
    ignored: HashSet<PathBuf>,
}
impl Watcher {
    fn run(mut self, events: &Receiver<notify::Result<notify::Event>>) {
        while let Ok(event) = events.recv() {
            if self.changed_paths(event).is_empty() {
                continue;
            }
            let mut pending = true;
            while pending {
                self.settle(events);
                let ran_program = is_executable(&self.dir.join("settings"));
                if let Some(extra) = reload_from(&self.dir) {
                    self.sync(extra);
                }
                // changed while loading
                pending = false;
                for path in self.settle(events) {
                    if ran_program && self.extra.contains(&path) {
                        log::warn!(
                            "The settings program changes {}, ignoring it so settings don't reload forever",
                            path.display()
                        );
                        self.ignored.insert(path);
                    } else {
                        pending = true;
                    }
                }
            }
        }
    }

    /// The watched paths changed until nothing changes for `SETTLE`
    fn settle(&self, events: &Receiver<notify::Result<notify::Event>>) -> HashSet<PathBuf> {
        let mut changed = HashSet::new();
        while let Ok(event) = events.recv_timeout(SETTLE) {
            changed.extend(self.changed_paths(event));
        }
        changed
    }

    fn changed_paths(&self, event: notify::Result<notify::Event>) -> Vec<PathBuf> {
        let event = match event {
            Ok(event) => event,
            Err(err) => {
                log::warn!("Settings watch error: {err}");
                return Vec::new();
            }
        };
        if matches!(event.kind, notify::EventKind::Access(_)) {
            return Vec::new();
        }
        event
            .paths
            .into_iter()
            .filter(|path| self.is_watched(path))
            .collect()
    }

    fn is_watched(&self, path: &Path) -> bool {
        let program = self.dir.join("settings");
        // `settings.json` is unused while there is a program
        path == program
            || (path == self.dir.join("settings.json") && !is_executable(&program))
            || (self.extra.contains(path) && !self.ignored.contains(path))
    }

    fn sync(&mut self, extra: Vec<PathBuf>) {
        self.extra = extra.into_iter().collect();
        let dirs: HashSet<PathBuf> = std::iter::once(self.dir.clone())
            .chain(self.extra.iter().filter_map(|path| path.parent().map(Path::to_path_buf)))
            .collect();
        for dir in self.dirs.difference(&dirs) {
            let _ = self.watcher.unwatch(dir);
        }
        for dir in dirs.difference(&self.dirs) {
            if let Err(err) = self.watcher.watch(dir, notify::RecursiveMode::NonRecursive) {
                log::warn!("Can't watch {} for settings: {err}", dir.display());
            }
        }
        self.dirs = dirs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_json_uses_defaults() {
        let loaded: Loaded =
            serde_json::from_str(r##"{"border": {"focused": "#a0cafd"}, "watch": ["/a"]}"##)
                .unwrap();
        assert_eq!(
            loaded.settings.border,
            Border {
                focused: Color([0xa0, 0xca, 0xfd, 0xff]),
                ..Border::default()
            }
        );
        assert_eq!(loaded.settings.palette, Palette::default());
        assert_eq!(loaded.watch, vec![PathBuf::from("/a")]);
    }

    #[test]
    fn color_parsing() {
        assert_eq!("#11223344".parse(), Ok(Color([0x11, 0x22, 0x33, 0x44])));
        assert!("112233".parse::<Color>().is_err());
        assert!("#+12345".parse::<Color>().is_err());
        assert!("#12345".parse::<Color>().is_err());
    }
}
