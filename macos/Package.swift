// swift-tools-version:5.7
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "tauri-plugin-notifications",
    platforms: [
        .macOS(.v13),
        .iOS(.v15),
    ],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "tauri-plugin-notifications",
            type: .static,
            targets: ["tauri-plugin-notifications"]),
    ],
    dependencies: [ ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "tauri-plugin-notifications",
            dependencies: [ ],
            path: "Sources",
            swiftSettings: [
                .unsafeFlags([
                    "-import-objc-header", "\(Context.packageDirectory)/Sources/bridging-header.h",
                    "-disable-bridging-pch"
                ])
            ],
            linkerSettings: [
                .linkedFramework("StoreKit")
            ]
        ),
        .testTarget(
            name: "PluginTests",
            dependencies: ["tauri-plugin-notifications"],
            swiftSettings: [
                .unsafeFlags([
                    "-import-objc-header", "\(Context.packageDirectory)/Sources/bridging-header.h",
                    "-disable-bridging-pch"
                ])
            ]
        ),
    ]
)
