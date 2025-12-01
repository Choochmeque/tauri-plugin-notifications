// swift-tools-version:5.5
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import PackageDescription
import Foundation

// Check if push notifications feature is enabled via marker file from Rust build
let enablePushNotifications = FileManager.default.fileExists(
  atPath: URL(fileURLWithPath: #file).deletingLastPathComponent()
    .appendingPathComponent(".push-notifications-enabled").path
)

var swiftSettings: [SwiftSetting] = []
if enablePushNotifications {
  swiftSettings.append(.define("ENABLE_PUSH_NOTIFICATIONS"))
}

let package = Package(
  name: "tauri-plugin-notifications",
  platforms: [
    .macOS(.v10_13),
    .iOS(.v13),
  ],
  products: [
    // Products define the executables and libraries a package produces, and make them visible to other packages.
    .library(
      name: "tauri-plugin-notifications",
      type: .static,
      targets: ["tauri-plugin-notifications"])
  ],
  dependencies: [
    .package(name: "Tauri", path: "../.tauri/tauri-api")
  ],
  targets: [
    // Targets are the basic building blocks of a package. A target can define a module or a test suite.
    // Targets can depend on other targets in this package, and on products in packages this package depends on.
    .target(
      name: "tauri-plugin-notifications",
      dependencies: [
        .byName(name: "Tauri")
      ],
      path: "Sources",
      swiftSettings: swiftSettings),
    .testTarget(
        name: "PluginTests",
        dependencies: ["tauri-plugin-notifications", .byName(name: "Tauri")]
    ),
  ]
)
