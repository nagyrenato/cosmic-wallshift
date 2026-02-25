/// Writes the COSMIC background config RON file and touches it so the daemon picks up the change.
///
/// Returns `Ok(applied_path)` on success, `Err(reason)` on failure.
pub fn apply(path: &str, is_dark: bool) -> Result<String, String> {
    let user = std::env::var("USER").unwrap_or_default();
    let bg_config = format!(
        "/home/{}/.config/cosmic/com.system76.CosmicBackground/v1/all",
        user
    );

    let ron = format!(
        r#"(
    output: "all",
    source: Path("{}"),
    filter_by_theme: true,
    rotation_frequency: 300,
    filter_method: Lanczos,
    scaling_mode: Zoom,
    sampling_method: Alphanumeric,
)"#,
        path
    );

    std::fs::write(&bg_config, &ron)
        .map_err(|e| format!("could not write config: {e}"))?;

    let _ = std::process::Command::new("touch").arg(&bg_config).status();

    Ok(format!(
        "Applied {} wallpaper: {}",
        if is_dark { "dark" } else { "light" },
        path
    ))
}
