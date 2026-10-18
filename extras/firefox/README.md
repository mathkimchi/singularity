# Sonamu's Firefox extension

Opens every tab in its own window and puts the page's url at the start of the window title, where Sonamu reads it
from to save the session. Setup for users is in [docs/persistent_apps.md](../../docs/persistent_apps.md#firefox).

`userChrome.css` isn't part of the extension: it hides the tab bar and goes in the Firefox profile.

## Signing

Release Firefox only installs signed extensions. It's signed as unlisted on addons.mozilla.org (free, automatic,
not listed publicly), and the signed file is attached to a GitHub release tagged `firefox-ext-v<version>`.

1. Raise `version` in `manifest.json` (every upload needs a new version).
2. Zip the extension's files (`manifest.json` at the top of the zip, not in a folder). In this folder:
   ```sh
   nix run nixpkgs#zip -- sonamu.zip manifest.json background.js
   ```
3. On [addons.mozilla.org](https://addons.mozilla.org), go to the Developer Hub. The first time, choose
   Submit a New Add-on → On your own (unlisted); after that, the add-on's Upload New Version. Upload `sonamu.zip`.
   Signing is automatic and takes a few minutes.
4. On the version's page, download the signed `.xpi` (in Firefox, right-click → Save Link As…, or it tries to
   install it) into this folder.
5. Commit and push the version bump, then make the release (eg for `0.1.0`):
   ```sh
   mv <downloaded>.xpi sonamu.xpi   # the asset's name is its download url (`#label` only changes the label)
   gh release create firefox-ext-v0.1.0 sonamu.xpi \
     --title "Firefox extension 0.1.0" --notes "Sonamu's Firefox extension" --target dev
   rm sonamu.zip sonamu.xpi
   ```
   Then update the link in `docs/persistent_apps.md` to the new tag.

Instead of steps 2 to 4, `web-ext sign --channel=unlisted --ignore-files README.md userChrome.css` does the same with
an API key from <https://addons.mozilla.org/developers/addon/api/key/>; the signed file lands in `web-ext-artifacts/`.

`*.zip` and `*.xpi` are ignored by git.

To try changes without signing, load `manifest.json` in `about:debugging#/runtime/this-firefox`
(Load Temporary Add-on). It's gone when Firefox restarts.
