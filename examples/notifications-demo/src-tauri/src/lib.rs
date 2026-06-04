#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Show INFO and above from everything by default, plus DEBUG from the
    // notifications plugin so UnifiedPush / D-Bus events are visible while
    // exercising the demo. Override at runtime with RUST_LOG=...
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or("info,tauri_plugin_notifications=debug"),
    )
    .format_timestamp_millis()
    .init();

    log::info!("notifications-demo starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notifications::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
