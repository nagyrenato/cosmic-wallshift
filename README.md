# CosmicWallShift

A lightweight system-tray app for the [COSMIC Desktop Environment](https://github.com/pop-os/cosmic-epoch) that automatically switches your wallpaper when the system theme toggles between **Light** and **Dark** mode.

![Rust](https://img.shields.io/badge/Rust-2024--edition-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-Linux%20%2F%20Wayland-informational)

---

## Screenshots

> _Coming soon_

---

## Requirements

- **Linux** with **Wayland** and the **COSMIC Desktop Environment** ([pop-os/cosmic-epoch](https://github.com/pop-os/cosmic-epoch))

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

---

## Usage

1. Launch the app — the settings window opens automatically.
2. Enter the full path to your **light** wallpaper (e.g. `/home/user/Pictures/Light.png`).
3. Enter the full path to your **dark** wallpaper (e.g. `/home/user/Pictures/Dark.png`).
4. Close the window — the app moves to the system tray and continues monitoring.

**Supported formats:** `jpg`, `jpeg`, `png`, `webp`

> **Note:** Wallpapers are applied with `filter_by_theme: true`, meaning COSMIC will apply a subtle tint to match the active theme. This is the default COSMIC behaviour and cannot currently be changed from the app.

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

## Flatpak packaging

Test a local Flatpak build:

```bash
flatpak-builder --install --user --force-clean build-dir io.github.nagyrenato.CosmicWallShift.yml
flatpak run io.github.nagyrenato.CosmicWallShift
```

To regenerate `cargo-sources.json` after updating `Cargo.lock`:

```bash
pip install aiohttp tomlkit
python3 flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
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
