// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "Openergy",
    platforms: [.macOS(.v15)],
    products: [
        .executable(name: "Openergy", targets: ["Openergy"]),
    ],
    dependencies: [
        .package(url: "https://github.com/sparkle-project/Sparkle", exact: "2.10.0"),
    ],
    targets: [
        .executableTarget(
            name: "Openergy",
            dependencies: [.product(name: "Sparkle", package: "Sparkle")],
            path: "Sources/Openergy",
            linkerSettings: [.unsafeFlags(["-Xlinker", "-rpath", "-Xlinker", "@executable_path/../Frameworks"])]
        ),
    ],
    swiftLanguageModes: [.v6]
)
