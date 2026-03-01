fn main() {
    let now = chrono::Local::now();
    // Format: YYYY.MM.DD-H-M  (hour without leading zero, e.g. "2026.03.01-1-12")
    let date_part = now.format("%Y.%m.%d").to_string();
    let hour = now.format("%H").to_string().trim_start_matches('0').to_string();
    let hour = if hour.is_empty() { "0".to_string() } else { hour };
    let minute = now.format("%M").to_string();
    let build_dt = format!("{}-{}-{}", date_part, hour, minute);
    println!("cargo:rustc-env=BUILD_DATE_TIME={}", build_dt);
    println!("cargo:rerun-if-changed=build.rs");
}
