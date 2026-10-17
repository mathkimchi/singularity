# Screen recording

Sonamu supports `ext-image-copy-capture`, so tools like `grim` and `wf-recorder` work directly.
OBS and browsers go through xdg-desktop-portal, which needs the setup below.
While anything captures the screen, `REC` flashes in the status bar.

## In a window (eg inside niri)

Record through the host compositor: in OBS, add a "Screen Capture (PipeWire)" source and pick the Sonamu window.

## On a tty (as the actual compositor)

1. NixOS config (then rebuild and log back in):
   ```nix
   services.pipewire.enable = true;
   xdg.portal = {
     enable = true;
     extraPortals = [ pkgs.xdg-desktop-portal-wlr pkgs.xdg-desktop-portal-gtk ];
     config.sonamu = {
       default = [ "gtk" ];
       "org.freedesktop.impl.portal.ScreenCast" = [ "wlr" ];
       "org.freedesktop.impl.portal.Screenshot" = [ "wlr" ];
     };
   };
   ```
   Elsewhere, install xdg-desktop-portal-wlr (0.8+) and copy `sonamu-portals.conf` to `~/.config/xdg-desktop-portal/`.
2. So the portal shares the one output without asking:
   ```sh
   mkdir -p ~/.config/xdg-desktop-portal-wlr
   cp assets/portal/xdg-desktop-portal-wlr.conf ~/.config/xdg-desktop-portal-wlr/config
   ```
3. Start Sonamu on the tty. It exports `WAYLAND_DISPLAY` and `XDG_CURRENT_DESKTOP=sonamu` to D-Bus activated services.
4. If the portals were already running from an earlier session:
   ```sh
   systemctl --user restart xdg-desktop-portal xdg-desktop-portal-wlr
   ```
5. Start OBS from the Command Hub, add a "Screen Capture (PipeWire)" source, and start recording.

If the source stays black, check `journalctl --user -u xdg-desktop-portal-wlr -b`.
