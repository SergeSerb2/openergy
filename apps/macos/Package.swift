// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "Openergy",
    platforms: [.macOS(.v15)],
    products: [
        .executable(name: "Openergy", targets: ["Openergy"]),
    ],
    targets: [
        .executableTarget(
            name: "Openergy",
            path: "Sources/Openergy"
        ),
    ],
    swiftLanguageModes: [.v6]
)
