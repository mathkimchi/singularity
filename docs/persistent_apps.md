# Persistent apps

Sonamu remembers your tree (layout, projects, native applets and Wayland apps) and rebuilds it when it starts.
Every Wayland window is saved with its command line, working directory, app id and title, and started again into
the same spot. While an app is starting, its spot shows "Starting …". Ctrl+W on a spot drops it.

Sonamu does the restoring, so turn off the apps' own session restore, or you'd get every window twice.

The session is saved every 2 seconds (if it changed) and before quitting, to `~/.local/state/sonamu/session.json`
(`$XDG_STATE_HOME/sonamu/session.json`). To use another file, set `SONAMU_SESSION=/path/to/file.json`.
To turn sessions off, set it to nothing (`SONAMU_SESSION=`). To start fresh, delete the file.
A file that can't be read is moved to `session.json.bad`.

The recommended apps are qutebrowser and foot: Super+T starts foot and Super+N starts qutebrowser.

## qutebrowser

Sonamu organizes the pages, so qutebrowser should open one page per window, and put each page's url in its title,
where Sonamu reads it from (after the last ` | `). On restore, each window is reopened with
`qutebrowser --target window <url>`.

| Setting | Value | Why |
| --- | --- | --- |
| `tabs.tabs_are_windows` | `True` | One page per window |
| `new_instance_open_target` | `'window'` | Links and restores open new windows, not tabs |
| `window.title_format` | `'{perc}{current_title} \| {current_url}'` | Sonamu saves the url from the title |
| `auto_save.session` | `False` | Sonamu restores the windows |

### NixOS with Home Manager

```nix
programs.qutebrowser = {
  enable = true;
  settings = {
    tabs.tabs_are_windows = true;
    new_instance_open_target = "window";
    window.title_format = "{perc}{current_title} | {current_url}";
    auto_save.session = false;
  };
};
```

### Without Home Manager

Install it (on NixOS, `environment.systemPackages = [ pkgs.qutebrowser ];`), then add to
`~/.config/qutebrowser/config.py`:

```python
config.load_autoconfig(False)
c.tabs.tabs_are_windows = True
c.new_instance_open_target = 'window'
c.window.title_format = '{perc}{current_title} | {current_url}'
c.auto_save.session = False
```

### Limits

All qutebrowser windows come from one process, so Sonamu matches restored windows to their spots by app id,
in the order they open. Two windows may come back in each other's spots.

## foot

Run plain `foot` for each window, not `foot --server` with `footclient`: with one process per window, Sonamu knows
which window is which and where its shell is. On restore, foot starts in the shell's last working directory.

The working directory is read when the window's title changes, so have the shell set the title on every prompt.
fish and nushell already do.

### NixOS with Home Manager

```nix
programs.foot = {
  enable = true;
  server.enable = false;
};

# bash: the working directory in the title on every prompt
programs.bash.initExtra = ''
  sonamu_title() { printf '\e]2;%s\a' "$PWD"; }
  PROMPT_COMMAND="sonamu_title''${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
'';

# or zsh
programs.zsh.initContent = ''
  autoload -Uz add-zsh-hook
  sonamu_title() { print -Pn '\e]2;%~\a' }
  add-zsh-hook precmd sonamu_title
'';

# or nushell (its default; set here in case your config turned it off)
programs.nushell.extraConfig = ''
  $env.config.shell_integration.osc2 = true
'';
```

On older Home Manager releases, `programs.zsh.initContent` is `programs.zsh.initExtra`.

### Without Home Manager

Install it (on NixOS, `environment.systemPackages = [ pkgs.foot ];`), don't enable the foot server
(`foot.service` / `foot-server`), and add the shell lines above to `~/.bashrc`, `~/.zshrc` or nushell's `config.nu`
(`$nu.config-path`). In `~/.bashrc`, write `${PROMPT_COMMAND:+;$PROMPT_COMMAND}` without the `''`.

### Keeping programs running across restarts

A restore starts a new shell, so whatever ran in it is gone. To keep it, run the shell in a session that outlives the
terminal: Sonamu saves the command line as is, so restoring reattaches.

```nix
home.packages = [ pkgs.shpool ];   # or pkgs.dtach / pkgs.abduco
```

Then open the terminal with a session name, eg `$ foot -e shpool attach notes`. Each window needs its own name.
The session survives Sonamu restarting, not a reboot.

## Nix notes

- Apps started from a `/nix/store/...` path are restored by name (eg `foot`), since the store path can be garbage
  collected after an update. Nix's `.foo-wrapped` binaries are restored through their wrapper `foo`.
  So keep the apps on your `PATH`, eg in `home.packages` or `environment.systemPackages`.
- Apps that run through an interpreter (eg qutebrowser through Python) can't be restored from their command line.
  Sonamu restores qutebrowser and firefox by their app id instead.

## Other apps

Any Wayland app is restored by its command line and working directory. Apps that open every window from one process
(eg `kitty -1`, firefox) share one working directory, and are matched back by app id.
Firefox comes back as a blank window unless its title ends with ` | <url>` (it needs an extension for that).
