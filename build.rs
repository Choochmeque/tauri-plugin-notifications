// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const COMMANDS: &[&str] = &[
    "register_listener",
    "remove_listener",
    "notify",
    "request_permission",
    "is_permission_granted",
    "register_for_push_notifications",
    "unregister_for_push_notifications",
    "register_action_types",
    "cancel",
    "cancel_all",
    "get_pending",
    "remove_active",
    "get_active",
    "check_permissions",
    "show",
    "batch",
    "list_channels",
    "delete_channel",
    "create_channel",
    "permission_state",
    "set_click_listener_active",
];

fn main() {
    // Check if push-notifications feature is enabled
    let enable_push = cfg!(feature = "push-notifications");

    // Generate build.properties file for Android
    if std::env::var("TARGET")
        .unwrap_or_default()
        .contains("android")
    {
        let properties_content = format!("enablePushNotifications={}", enable_push);
        std::fs::write("android/build.properties", properties_content)
            .expect("Failed to write build.properties");
    }

    // Generate marker file for iOS Swift build
    // Package.swift reads this file to conditionally enable ENABLE_PUSH_NOTIFICATIONS
    let ios_marker_path = std::path::Path::new("ios/.push-notifications-enabled");
    if enable_push {
        std::fs::write(ios_marker_path, "").expect("Failed to write iOS push marker file");
    } else if ios_marker_path.exists() {
        std::fs::remove_file(ios_marker_path).ok();
    }

    let result = tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .try_build();

    // when building documentation for Android the plugin build result is always Err() and is irrelevant to the crate documentation build
    if !(cfg!(docsrs)
        && std::env::var("TARGET")
            .expect("Failed to get TARGET environment variable")
            .contains("android"))
    {
        result.expect("Failed to build Tauri plugin");
    }
}
