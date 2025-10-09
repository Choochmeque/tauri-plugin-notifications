// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const COMMANDS: &[&str] = &[
    "notify",
    "request_permission",
    "is_permission_granted",
    "register_for_push_notifications",
    "register_action_types",
    "register_listener",
    "cancel",
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

    // Pass the feature flag to iOS via environment variable for xcconfig
    if enable_push {
        std::env::set_var(
            "SWIFT_ACTIVE_COMPILATION_CONDITIONS",
            "ENABLE_PUSH_NOTIFICATIONS",
        );
    }

    let result = tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .try_build();

    // when building documentation for Android the plugin build result is always Err() and is irrelevant to the crate documentation build
    if !(cfg!(docsrs) && std::env::var("TARGET").unwrap().contains("android")) {
        result.unwrap();
    }
}
