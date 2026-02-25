APP_ID := "io.github.YOUR_USERNAME.CosmicBgSync"
PREFIX := "/usr"

# Build a release binary
build:
    cargo build --release

# Run clippy and format checks
check:
    cargo fmt --check
    cargo clippy -- -D warnings

# Auto-format source
fmt:
    cargo fmt

# Install to PREFIX (default: /usr)
install:
    install -Dm755 target/release/cosmic-bg-sync {{PREFIX}}/bin/cosmic-bg-sync
    install -Dm644 res/{{APP_ID}}.desktop {{PREFIX}}/share/applications/{{APP_ID}}.desktop
    install -Dm644 res/{{APP_ID}}.metainfo.xml {{PREFIX}}/share/metainfo/{{APP_ID}}.metainfo.xml
    install -Dm644 res/icons/{{APP_ID}}.svg {{PREFIX}}/share/icons/hicolor/scalable/apps/{{APP_ID}}.svg

# Uninstall from PREFIX
uninstall:
    rm -f {{PREFIX}}/bin/cosmic-bg-sync
    rm -f {{PREFIX}}/share/applications/{{APP_ID}}.desktop
    rm -f {{PREFIX}}/share/metainfo/{{APP_ID}}.metainfo.xml
    rm -f {{PREFIX}}/share/icons/hicolor/scalable/apps/{{APP_ID}}.svg

# Vendor all Cargo dependencies for offline / Flatpak builds
vendor:
    cargo vendor
    mkdir -p .cargo
    printf '[source.crates-io]\nreplace-with = "vendored-sources"\n\n[source.vendored-sources]\ndirectory = "vendor"\n' > .cargo/config.toml

# Generate Flatpak cargo sources file (requires flatpak-cargo-generator.py)
flatpak-sources:
    flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
