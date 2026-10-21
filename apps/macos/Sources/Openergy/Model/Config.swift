import Foundation

/// The daemon's configuration as returned by `config.get` (JSON form of config.toml).
/// The app edits it only through `config.patch` merge patches.
struct OpenergyConfig: Codable, Equatable, Sendable {
    var role: Role?
    var name: String?
    var server: Server?
    var client: Client?
    var tls: TLS?
    var clipboard: Clipboard?
    var input: Input?
    var `switch`: Switch?
    var hotkeys: Hotkeys?
    /// Server layout. The daemon uses the TOML name `screen` (from `[[screen]]`).
    var screen: [Screen]?
    var derivedLinks: [Link]?
    var macos: MacOS?

    var screens: [Screen]? {
        get { screen }
        set { screen = newValue }
    }

    /// `config.get` and `config.patch` answer `{config, derived_links}`.
    static func decode(fromResult result: JSONValue) throws -> OpenergyConfig {
        var config = try (result["config"] ?? result).decode(OpenergyConfig.self)
        if let links = result["derived_links"], let decoded = try? links.decode([Link].self) {
            config.derivedLinks = decoded
        }
        return config
    }

    static let empty = OpenergyConfig()

    struct Server: Codable, Equatable, Sendable {
        var port: Int?
        var listen: String?
    }

    struct Client: Codable, Equatable, Sendable {
        var remote: String?
        var autoconnect: Bool?
    }

    struct TLS: Codable, Equatable, Sendable {
        var enabled: Bool?
    }

    struct Clipboard: Codable, Equatable, Sendable {
        var enabled: Bool?
        var maxBytes: Double?
    }

    struct Input: Codable, Equatable, Sendable {
        var relativeMouse: Bool?
        var invertScrollY: Bool?
        var scrollSensitivity: Double?
        var syncLanguage: Bool?
        var preventSleep: Bool?
    }

    struct Switch: Codable, Equatable, Sendable {
        var delayMs: Double?
        var doubleTapMs: Double?
        var deadCorners: [String]?
        var cornerSizePx: Double?
    }

    struct Hotkeys: Codable, Equatable, Sendable {
        var lockToScreen: String?
    }

    struct Screen: Codable, Equatable, Sendable, Identifiable {
        var name: String
        var rect: LayoutRect?
        var os: String?
        var id: String { name }
    }

    struct LayoutRect: Codable, Equatable, Sendable {
        var x: Double
        var y: Double
        var w: Double
        var h: Double

        var cgRect: CGRect { CGRect(x: x, y: y, width: w, height: h) }
        init(x: Double, y: Double, w: Double, h: Double) {
            self.x = x; self.y = y; self.w = w; self.h = h
        }
        init(_ rect: CGRect) {
            self.init(x: rect.minX, y: rect.minY, w: rect.width, h: rect.height)
        }
    }

    struct Link: Codable, Equatable, Sendable, Hashable {
        var from: String
        var edge: String
        var fromRange: [Double]?
        var to: String
        var toRange: [Double]?
    }

    struct MacOS: Codable, Equatable, Sendable {
        var clickNumbering: String?
    }
}
