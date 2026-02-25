# CosmicWallShift

A lightweight system-tray app for the [COSMIC Desktop Environment](https://github.com/pop-os/cosmic-epoch) that automatically switches your wallpaper when the system theme toggles between **Light** and **Dark** mode.

![Rust](https://img.shields.io/badge/Rust-2024--edition-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-Linux%20%2F%20Wayland-informational)

---

## Features

- Detects COSMIC theme changes in real-time using **inotify** (no polling)
- Automatically applies the correct wallpaper when switching between light and dark mode
- **System tray icon** — sits quietly in the background; click "Show Window" to bring back the UI
- Closing the window sends it to the tray instead of quitting
- Settings window built with **libcosmic** for a native COSMIC look-and-feel
- Live **activity log** showing every applied wallpaper change

---

## Screenshots

> _Coming soon_

---

## Requirements

- **Linux** with **Wayland** and the **COSMIC Desktop Environment** ([pop-os/cosmic-epoch](https://github.com/pop-os/cosmic-epoch))
- The app reads and writes COSMIC-specific config files under `~/.config/cosmic/`
- System tray requires the COSMIC panel (provides the StatusNotifierItem host)

---

## Installation

### Flatpak (recommended)

> Once submitted to Flathub, install with:

```bash
flatpak install flathub io.github.nagyrenato.CosmicWallShift
```

### Build from source

**Prerequisites:**
- **Rust** toolchain (stable, 1.80+) — install via [rustup](https://rustup.rs)
- Standard COSMIC DE build dependencies (`libwayland-dev`, `libxkbcommon-dev`, etc.)

On Pop!_OS / Ubuntu:

```bash
sudo apt install git curl build-essential libwayland-dev libxkbcommon-dev
```

```bash
git clone https://github.com/nagyrenato/cosmic-theme-background-switcher
cd cosmic-theme-background-switcher
cargo build --release
```

**Install with [just](https://github.com/casey/just):**

```bash
just build
sudo just install
```

Or manually:

```bash
sudo install -Dm755 target/release/cosmic-wallshift /usr/bin/cosmic-wallshift
```

---

## Usage

### Run directly

```bash
cargo run --release
# or, after installing:
cosmic-wallshift
```

### Configure wallpaper paths

1. Launch the app — the settings window opens automatically.
2. Enter the full path to your **light** wallpaper (e.g. `/home/user/Pictures/Light.png`).
3. Enter the full path to your **dark** wallpaper (e.g. `/home/user/Pictures/Dark.png`).
4. Close the window — the app moves to the system tray and continues monitoring.

The wallpaper is applied immediately when a path is entered if that theme is currently active, and on every subsequent theme switch.

### Autostart on login

Create a desktop entry in `~/.config/autostart/`:

```bash
mkdir -p ~/.config/autostart
cat > ~/.config/autostart/cosmic-wallshift.desktop << EOF
[Desktop Entry]
Type=Application
Name=CosmicWallShift
Exec=/usr/local/bin/cosmic-wallshift
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
EOF
```

---

## How it works

```
~/.config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark
                          │
                    inotify watch
                          │
                    Theme changed?
                    ┌─────┴─────┐
                  Dark        Light
                    │             │
              dark_wp         light_wp
                    │             │
                    └──────┬──────┘
                           │
        Write RON to ~/.config/cosmic/
          com.system76.CosmicBackground/v1/all
                           │
                      touch the file
                           │
               COSMIC background daemon
                  picks up the change
```

1. **`watcher.rs`** — uses `notify` (inotify/kqueue) to watch the COSMIC theme mode file. When it changes, reads the file and emits a `ThemeChanged(bool)` message.
2. **`app.rs`** — receives the message, selects the appropriate wallpaper path, and calls `wallpaper::apply`.
3. **`wallpaper.rs`** — writes a RON-format config file that the `cosmic-bg` daemon reads. Touches the file to ensure the daemon detects the update.
4. **`tray.rs`** — registers a StatusNotifierItem tray icon via `ksni`. "Show Window" reopens the UI; "Quit" exits the process.

---

## Configuration files

| File | Purpose |
|------|---------|
| `~/.config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark` | Read — current theme mode (`true` = dark) |
| `~/.config/cosmic/com.system76.CosmicBackground/v1/all` | Written — wallpaper config (RON format) |

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| [`libcosmic`](https://github.com/pop-os/libcosmic) | git (main) | UI framework, app runtime, settings window |
| [`tokio`](https://tokio.rs) | 1.49 | Async runtime |
| [`notify`](https://github.com/notify-rs/notify) | 8.2 | Filesystem event watcher (inotify on Linux) |
| [`ksni`](https://github.com/iovxw/ksni) | 0.3.3 | StatusNotifierItem system tray |

---

## Project structure

```
cosmic-theme-background-switcher/
├── Cargo.toml
├── justfile                                            # Build / install targets
├── io.github.nagyrenato.CosmicWallShift.yml               # Flatpak manifest
├── cargo-sources.json                                  # Vendored Cargo deps (offline Flatpak builds)
├── res/
│   ├── io.github.nagyrenato.CosmicWallShift.desktop       # Desktop entry
│   ├── io.github.nagyrenato.CosmicWallShift.metainfo.xml  # AppStream metadata
│   ├── screenshots/                                    # TODO: add app screenshots here
│   └── icons/
│       └── io.github.nagyrenato.CosmicWallShift.svg       # App icon
└── src/
    ├── main.rs        # Entry point — configures and launches the libcosmic app
    ├── app.rs         # Application struct, view, update logic
    ├── message.rs     # Message enum (events passed through the Elm-style runtime)
    ├── wallpaper.rs   # Writes the COSMIC background RON config
    ├── watcher.rs     # inotify subscription for the theme mode file
    └── tray.rs        # System tray icon via ksni (StatusNotifierItem)
```

---

## Troubleshooting

**Tray icon does not appear**

The StatusNotifierWatcher service must be running. On COSMIC this is provided by the panel. Check stderr:

```bash
cosmic-wallshift 2>&1 | grep -i tray
```

If you see `System tray unavailable: ...`, the watcher is not available in your session.

**Wallpaper does not change**

- Confirm the `cosmic-bg` daemon is running: `systemctl --user status cosmic-bg`
- Check the activity log in the settings window for error messages
- Verify the wallpaper paths exist and are readable

**Window does not reopen after closing**

This is a known limitation of Wayland: surfaces cannot be re-shown after being hidden. The app works around this by destroying and recreating the window. If it still doesn't appear, click the tray icon again.

---

## Flatpak packaging

The project is ready for Flatpak submission. `cargo-sources.json` (vendored Cargo deps for offline builds) and the app icon are already committed.

Test a local Flatpak build:

```bash
# Install flatpak-builder if needed
sudo apt install flatpak-builder

# Build and install locally
flatpak-builder --install --user --force-clean build-dir io.github.nagyrenato.CosmicWallShift.yml

# Run it
flatpak run io.github.nagyrenato.CosmicWallShift
```

**Before submitting to Flathub**, take a screenshot of the running app and add it:

```bash
mkdir -p res/screenshots
cp /path/to/screenshot.png res/screenshots/main.png
# Then uncomment the <screenshots> block in:
# res/io.github.nagyrenato.CosmicWallShift.metainfo.xml
```

Validate the metainfo:

```bash
appstreamcli validate res/io.github.nagyrenato.CosmicWallShift.metainfo.xml
```

To regenerate `cargo-sources.json` after updating `Cargo.lock`:

```bash
pip install aiohttp tomlkit
curl -sL https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py | python3 - Cargo.lock -o cargo-sources.json
```

---

## Contributing

Issues and pull requests are welcome. Please make sure the project compiles cleanly before submitting:

```bash
just check
```

---

## License

MIT — see [LICENSE](LICENSE) for details.
