// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "FocusLab",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "FocusLabFixture", targets: ["FocusLabFixture"]),
        .executable(name: "FocusLabMenuBar", targets: ["FocusLabMenuBar"]),
        .executable(name: "focus-lab", targets: ["FocusLabDriver"]),
    ],
    targets: [
        .target(name: "FocusLabSupport"),
        .executableTarget(name: "FocusLabFixture", dependencies: ["FocusLabSupport"]),
        .executableTarget(name: "FocusLabMenuBar", dependencies: ["FocusLabSupport"]),
        .executableTarget(name: "FocusLabDriver", dependencies: ["FocusLabSupport"]),
    ],
    swiftLanguageModes: [.v6]
)
