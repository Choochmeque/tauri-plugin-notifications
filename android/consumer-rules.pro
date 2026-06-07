# Preserve Jackson @JsonValue annotations and the enums that rely on them
# (Importance, Visibility). Without these rules R8 strips the annotation in
# release builds and Jackson falls back to serializing enums as `name()`
# (e.g. "Default") instead of the integer value the Rust side expects.
-keepattributes RuntimeVisibleAnnotations

-keep enum app.tauri.notification.Importance { *; }
-keep enum app.tauri.notification.Visibility { *; }
