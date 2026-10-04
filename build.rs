#[cfg(target_os = "macos")]
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Overrides the `SwiftPM` build system (`native` or `swiftbuild`). Toolchains
/// differ in their default -- Xcode 26 uses `native`, Xcode 27 uses
/// `swiftbuild` -- and the two lay their products out differently, so CI pins
/// this to cover both and users can pin it to route around a broken backend.
#[cfg(target_os = "macos")]
const SWIFT_BUILD_SYSTEM_ENV: &str = "TAURI_PLUGIN_SWIFT_BUILD_SYSTEM";

#[cfg(target_os = "macos")]
const SWIFT_LIB_NAME: &str = "tauri-plugin-notifications";

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
    "remove_all",
    "get_active",
    "check_permissions",
    "show",
    "batch",
    "list_channels",
    "delete_channel",
    "create_channel",
    "permission_state",
    "set_click_listener_active",
    "list_distributors",
    "set_distributor",
    "set_token",
];

fn main() {
    // Check if push-notifications feature is enabled
    let enable_push = cfg!(feature = "push-notifications");

    // Generate build.properties file for Android
    if std::env::var("TARGET")
        .unwrap_or_default()
        .contains("android")
    {
        let properties_content = format!("enablePushNotifications={enable_push}");
        std::fs::write("android/build.properties", properties_content)
            .expect("Failed to write build.properties");
    }

    // Generate marker file for iOS/macOS Swift build
    // Package.swift reads this file to conditionally enable ENABLE_PUSH_NOTIFICATIONS
    let ios_marker_path = std::path::Path::new("ios/.push-notifications-enabled");
    let macos_marker_path = std::path::Path::new("macos/.push-notifications-enabled");
    if enable_push {
        std::fs::write(ios_marker_path, "").expect("Failed to write iOS push marker file");
        std::fs::write(macos_marker_path, "").expect("Failed to write macOS push marker file");
    } else {
        if ios_marker_path.exists() {
            std::fs::remove_file(ios_marker_path).ok();
        }
        if macos_marker_path.exists() {
            std::fs::remove_file(macos_marker_path).ok();
        }
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

    #[cfg(target_os = "macos")]
    {
        // Only run macOS-specific build steps when building for macOS
        if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "macos" {
            // Rebuild when target architecture or deployment target changes
            println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
            println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");
            println!("cargo:rerun-if-env-changed={SWIFT_BUILD_SYSTEM_ENV}");

            let bridges = vec!["src/macos.rs"];
            for path in &bridges {
                println!("cargo:rerun-if-changed={path}");
            }

            watch_swift_inputs();

            swift_bridge_build::parse_bridges(bridges)
                .write_all_concatenated(swift_bridge_out_dir(), env!("CARGO_PKG_NAME"));

            let lib_dir = compile_swift();

            println!("cargo:rustc-link-lib=static={SWIFT_LIB_NAME}");
            println!(
                "cargo:rustc-link-search=native={}",
                path_arg(&lib_dir, "Swift library directory")
            );
        }
    }
}

/// Declares every hand-written Swift input as a rebuild trigger.
/// `Sources/generated` is deliberately skipped: this build script writes it, so
/// watching it would make cargo rebuild on every invocation.
#[cfg(target_os = "macos")]
fn watch_swift_inputs() {
    let package_manifest = manifest_dir().join("macos/Package.swift");
    println!(
        "cargo:rerun-if-changed={}",
        path_arg(&package_manifest, "Package.swift path")
    );
    watch_swift_dir(&swift_source_dir());
}

#[cfg(target_os = "macos")]
fn watch_swift_dir(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "generated") {
                continue;
            }
            watch_swift_dir(&path);
        } else {
            println!(
                "cargo:rerun-if-changed={}",
                path_arg(&path, "Swift source path")
            );
        }
    }
}

/// The build system `SwiftPM` should use, when pinned.
#[cfg(target_os = "macos")]
fn swift_build_system() -> Option<String> {
    let value = std::env::var(SWIFT_BUILD_SYSTEM_ENV).ok()?;
    if value.is_empty() {
        return None;
    }

    assert!(
        matches!(value.as_str(), "native" | "swiftbuild" | "xcode"),
        "{SWIFT_BUILD_SYSTEM_ENV}={value} is not a SwiftPM build system; \
         expected one of: native, swiftbuild, xcode"
    );

    Some(value)
}

/// The `swift build` arguments shared by the real build and by the
/// `--show-bin-path` query, so the two can never describe different builds.
#[cfg(target_os = "macos")]
fn swift_build_args() -> Vec<String> {
    let mut args = vec![
        "build".to_owned(),
        // Build into OUT_DIR (under target/) instead of the default `.build`
        // inside the crate source. Source-tree writes don't survive a clean
        // registry re-extraction / cache restore, which leaves cargo's
        // fingerprint saying "built" while the linked artifact is gone.
        "--scratch-path".to_owned(),
        path_arg(&swift_build_dir(), "Swift build path"),
        "--triple".to_owned(),
        swift_target_triple(),
        "-Xswiftc".to_owned(),
        "-import-objc-header".to_owned(),
        "-Xswiftc".to_owned(),
        path_arg(
            &swift_source_dir().join("bridging-header.h"),
            "Bridging header path",
        ),
    ];

    if let Some(build_system) = swift_build_system() {
        args.push("--build-system".to_owned());
        args.push(build_system);
    }

    if is_release_build() {
        args.push("-c".to_owned());
        args.push("release".to_owned());
        // The swiftbuild backend prelinks a static target with `clang -r -Os`
        // and LTO, which internalises everything outside the module's public
        // API. swift-bridge's `@_cdecl` wrappers are internal, so they survive
        // as `T` in Objects-normal/<arch>/*.o and come back `t` from the
        // prelink, leaving the Rust link with undefined
        // `__swift_bridge__$...` symbols. Testability keeps them external.
        // This is not whole-module optimisation: the native backend also uses
        // `-wmo -O` for release and keeps the symbols. Debug is unaffected
        // because SwiftPM already passes `-enable-testing` there.
        // `-Xlinker -exported_symbol`, `-Xlinker -keep_private_externs` and
        // `--disable-dead-strip` were all measured and do not help, because
        // SwiftPM does not forward them to that prelink step. The narrow fix
        // is for swift-bridge to generate `public` wrappers (upstream).
        args.push("-Xswiftc".to_owned());
        args.push("-enable-testing".to_owned());
    }

    args
}

/// Builds the Swift package and returns the directory holding the static
/// library to link against.
#[cfg(target_os = "macos")]
fn compile_swift() -> PathBuf {
    let package_dir = manifest_dir().join("macos");
    let args = swift_build_args();

    let output = Command::new("swift")
        .current_dir(&package_dir)
        .args(&args)
        .output()
        .unwrap_or_else(|error| {
            panic!(
                "Failed to run `swift {}` in {}: {error}",
                args.join(" "),
                package_dir.display()
            )
        });

    assert!(
        output.status.success(),
        r"
Swift build failed.
Command:   swift {}
Directory: {}
Status:    {}
Stdout:
{}
Stderr:
{}
",
        args.join(" "),
        package_dir.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let bin_dir = swift_bin_dir(&package_dir, &args);
    swift_static_lib_dir(&bin_dir, &args)
}

/// Asks `SwiftPM` where it put the products instead of assuming a layout: the
/// native backend writes `<triple>/<config>`, the swiftbuild backend writes
/// `<triple>/Products/<Config>`.
#[cfg(target_os = "macos")]
fn swift_bin_dir(package_dir: &Path, build_args: &[String]) -> PathBuf {
    let mut args = build_args.to_vec();
    args.push("--show-bin-path".to_owned());

    let output = Command::new("swift")
        .current_dir(package_dir)
        .args(&args)
        .output()
        .unwrap_or_else(|error| {
            panic!(
                "Failed to run `swift {}` in {}: {error}",
                args.join(" "),
                package_dir.display()
            )
        });

    // Only the line terminator is stripped; a directory name may end in a space.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let bin_path = stdout.trim_end_matches(['\n', '\r']);

    assert!(
        output.status.success() && !bin_path.is_empty() && !bin_path.contains(['\n', '\r']),
        r"
`swift build --show-bin-path` did not print exactly one path.
Command:   swift {}
Directory: {}
Status:    {}
Stdout:
{}
Stderr:
{}
",
        args.join(" "),
        package_dir.display(),
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr),
    );

    let bin_dir = Path::new(bin_path);
    if bin_dir.is_absolute() {
        bin_dir.to_path_buf()
    } else {
        package_dir.join(bin_dir)
    }
}

/// Normalises whatever `SwiftPM` produced into a directory holding
/// `lib<name>.a`. The swiftbuild backend emits a bare object even for a
/// `type: .static` product, so archive it ourselves in that case.
#[cfg(target_os = "macos")]
fn swift_static_lib_dir(bin_dir: &Path, build_args: &[String]) -> PathBuf {
    if bin_dir.join(format!("lib{SWIFT_LIB_NAME}.a")).is_file() {
        return bin_dir.to_path_buf();
    }

    let object = bin_dir.join(format!("{SWIFT_LIB_NAME}.o"));
    assert!(
        object.is_file(),
        r"
Could not find a Swift library or object to link.
Command:  swift {}
Looked in: {}
Expected:  lib{}.a or {}.o
Contents:
{}
",
        build_args.join(" "),
        bin_dir.display(),
        SWIFT_LIB_NAME,
        SWIFT_LIB_NAME,
        list_dir(bin_dir),
    );

    // Start from an empty directory so an archive left by a previous backend
    // can never shadow the product that was just built.
    let lib_dir = out_dir().join("swift-lib");
    match std::fs::remove_dir_all(&lib_dir) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!(
            "Failed to clear the Swift library directory {}: {error}",
            lib_dir.display()
        ),
    }
    std::fs::create_dir_all(&lib_dir).unwrap_or_else(|error| {
        panic!(
            "Failed to create the Swift library directory {}: {error}",
            lib_dir.display()
        )
    });

    let archive = lib_dir.join(format!("lib{SWIFT_LIB_NAME}.a"));
    archive_object(&object, &archive);

    assert!(
        archive.is_file(),
        "Archiving {} produced no file at {}",
        object.display(),
        archive.display()
    );

    lib_dir
}

/// Wraps a single object file in a static archive.
#[cfg(target_os = "macos")]
fn archive_object(object: &Path, archive: &Path) {
    let object = path_arg(object, "Swift object path");
    let archive = path_arg(archive, "Swift archive path");

    // `libtool -static` is the archiver to use here: the swiftbuild backend
    // emits a universal (x86_64 + arm64) object, and `ar` writes an archive
    // that `ranlib` then refuses to index.
    let attempts: [Vec<&str>; 2] = [
        vec!["xcrun", "libtool", "-static", "-o", &archive, &object],
        vec!["libtool", "-static", "-o", &archive, &object],
    ];

    let mut failures = Vec::new();
    for attempt in &attempts {
        let Some((program, args)) = attempt.split_first() else {
            continue;
        };

        match Command::new(program).args(args).output() {
            Ok(output) if output.status.success() => return,
            Ok(output) => failures.push(format!(
                "`{program} {}` exited with {}: {}",
                args.join(" "),
                output.status,
                String::from_utf8_lossy(&output.stderr).trim_end()
            )),
            Err(error) => failures.push(format!("`{program}` could not be run: {error}")),
        }
    }

    panic!(
        "Failed to archive {object} into {archive}:\n{}",
        failures.join("\n")
    );
}

#[cfg(target_os = "macos")]
fn list_dir(dir: &Path) -> String {
    std::fs::read_dir(dir).map_or_else(
        |error| format!("  <unreadable: {error}>"),
        |entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| format!("  {}", entry.file_name().to_string_lossy()))
                .collect::<Vec<_>>()
                .join("\n")
        },
    )
}

#[cfg(target_os = "macos")]
fn path_arg(path: &Path, label: &str) -> String {
    path.to_str()
        .unwrap_or_else(|| panic!("{label} must be valid UTF-8: {}", path.display()))
        .to_owned()
}

#[cfg(target_os = "macos")]
fn swift_bridge_out_dir() -> PathBuf {
    generated_code_dir()
}

#[cfg(target_os = "macos")]
fn manifest_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set");
    PathBuf::from(manifest_dir)
}

#[cfg(target_os = "macos")]
fn out_dir() -> PathBuf {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    PathBuf::from(out_dir)
}

/// `SwiftPM` scratch (build) directory, under `OUT_DIR` so it lives in `target/`
/// and is covered by cargo's fingerprint and any build cache.
#[cfg(target_os = "macos")]
fn swift_build_dir() -> PathBuf {
    out_dir().join("swift-build")
}

#[cfg(target_os = "macos")]
fn is_release_build() -> bool {
    std::env::var("PROFILE").expect("PROFILE must be set") == "release"
}

#[cfg(target_os = "macos")]
fn swift_source_dir() -> PathBuf {
    manifest_dir().join("macos/Sources")
}

#[cfg(target_os = "macos")]
fn generated_code_dir() -> PathBuf {
    swift_source_dir().join("generated")
}

#[cfg(target_os = "macos")]
fn target_arch() -> String {
    std::env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH must be set")
}

#[cfg(target_os = "macos")]
fn swift_arch() -> &'static str {
    match target_arch().as_str() {
        "aarch64" => "arm64",
        "x86_64" => "x86_64",
        arch => panic!("Unsupported architecture for macOS: {arch}"),
    }
}

#[cfg(target_os = "macos")]
fn macos_deployment_target() -> String {
    std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "13.0".to_string())
}

#[cfg(target_os = "macos")]
fn swift_target_triple() -> String {
    format!("{}-apple-macosx{}", swift_arch(), macos_deployment_target())
}
