fn main() {
    tauri_build::build();

    // Workaround for linking issues with Swift on macOS in debug mode (for notifications plugin)
    #[cfg(all(target_os = "macos", debug_assertions))]
    {
        if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "macos" {
            let p = "/Library/Developer/CommandLineTools/usr/lib/swift-5.5/macosx";
            println!("cargo:rustc-link-search=native={p}");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{p}");
        }
    }
}
