import Foundation

/// The daemon's full status object (`status.get` / `status.changed`).
/// Every field is optional so a newer or older daemon never breaks decoding.
struct DaemonStatus: Codable, Equatable, Sendable {
    var daemon: DaemonInfo?
    var role: Role?
    var state: ConnectionState?
    var name: String?
    var activeScreen: String?
    var cursor: CursorInfo?
    var localScreen: LocalScreen?
    var server: ServerInfo?
    var client: ClientInfo?
    var peers: [Peer]?
    var tls: OwnTLS?
    var permissions: Permissions?
    var secureInput: SecureInput?
    var clipboard: ClipboardStatus?
    var pendingTrust: [PendingTrust]?
    var lastError: DaemonError?
    var stats: Stats?

    static let empty = DaemonStatus()
}

enum Role: String, Codable, Sendable, CaseIterable, Identifiable {
    case off, server, client
    var id: String { rawValue }

    /// The words the UI uses for each role.
    var title: String {
        switch self {
        case .off: "Off"
        case .server: "Host"
        case .client: "Join"
        }
    }

    var summary: String {
        switch self {
        case .off: "Not sharing"
        case .server: "Shares this Mac's keyboard and mouse"
        case .client: "Uses another computer's keyboard and mouse"
        }
    }
}

enum ConnectionState: String, Codable, Sendable {
    case idle, starting, listening, connecting, connected, reconnecting, error

    init(from decoder: Decoder) throws {
        let raw = try decoder.singleValueContainer().decode(String.self)
        self = ConnectionState(rawValue: raw) ?? .error
    }
}

struct DaemonInfo: Codable, Equatable, Sendable {
    var version: String?
    var pid: Int32?
    var uptimeS: Double?
    var platform: String?
}

struct CursorInfo: Codable, Equatable, Sendable {
    var screen: String?
    var x: Double?
    var y: Double?
}

struct LocalScreen: Codable, Equatable, Sendable {
    var w: Double?
    var h: Double?
    var displays: [Display]?

    struct Display: Codable, Equatable, Sendable {
        var x: Double
        var y: Double
        var w: Double
        var h: Double
        var scale: Double?
    }
}

struct ServerInfo: Codable, Equatable, Sendable {
    var listen: String?
}

struct ClientInfo: Codable, Equatable, Sendable {
    var remote: String?
    var nextRetryMs: Double?
    var attempt: Int?
}

struct Peer: Codable, Equatable, Sendable, Identifiable {
    var name: String
    var address: String?
    var role: String?
    var connected: Bool?
    var sinceUnixMs: Double?
    var rttMs: Double?
    var `protocol`: ProtocolInfo?
    var screen: PeerScreen?
    var tls: PeerTLS?
    var secureInputApp: String?
    var os: String?

    var id: String { name }

    struct ProtocolInfo: Codable, Equatable, Sendable {
        var product: String?
        var major: Int?
        var minor: Int?

        var label: String {
            guard let major, let minor else { return product ?? "Unknown" }
            return "\(product ?? "Synergy") \(major).\(minor)"
        }
    }

    struct PeerScreen: Codable, Equatable, Sendable {
        var w: Double?
        var h: Double?
    }

    struct PeerTLS: Codable, Equatable, Sendable {
        var enabled: Bool?
        var version: String?
        var fingerprint: String?
        var trusted: Bool?
    }
}

struct OwnTLS: Codable, Equatable, Sendable {
    var enabled: Bool?
    var fingerprint: String?
}

enum PermissionState: String, Codable, Sendable {
    case granted, denied, unknown
    case notRequired = "not_required"

    init(from decoder: Decoder) throws {
        let raw = try decoder.singleValueContainer().decode(String.self)
        self = PermissionState(rawValue: raw) ?? .unknown
    }

    var isSatisfied: Bool { self == .granted || self == .notRequired }
}

struct Permissions: Codable, Equatable, Sendable {
    var accessibility: PermissionState?
    var inputMonitoring: PermissionState?
    var postEvents: PermissionState?

    /// True when something the current role needs is missing.
    func missing(for role: Role) -> [PermissionKind] {
        var result: [PermissionKind] = []
        if role != .off, accessibility.map({ !$0.isSatisfied }) ?? false { result.append(.accessibility) }
        if role == .server, inputMonitoring.map({ !$0.isSatisfied }) ?? false { result.append(.inputMonitoring) }
        return result
    }
}

enum PermissionKind: String, Sendable, CaseIterable, Identifiable {
    case accessibility
    case inputMonitoring = "input_monitoring"
    var id: String { rawValue }

    var title: String {
        switch self {
        case .accessibility: "Accessibility"
        case .inputMonitoring: "Input Monitoring"
        }
    }

    var purpose: String {
        switch self {
        case .accessibility: "Moves the pointer and types on this Mac"
        case .inputMonitoring: "Sends this Mac's keyboard and mouse to other computers"
        }
    }

    var settingsURL: URL {
        switch self {
        case .accessibility:
            URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!
        case .inputMonitoring:
            URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")!
        }
    }
}

struct SecureInput: Codable, Equatable, Sendable {
    var localApp: String?
}

struct ClipboardStatus: Codable, Equatable, Sendable {
    var enabled: Bool?
    var lastTransfer: Transfer?
    var lastBlocked: Blocked?

    struct Transfer: Codable, Equatable, Sendable {
        var direction: String?
        var bytes: Double?
        var formats: [String]?
        var unixMs: Double?
    }

    struct Blocked: Codable, Equatable, Sendable {
        var bytes: Double?
        var limit: Double?
        var peer: String?
        var unixMs: Double?
    }
}

struct PendingTrust: Codable, Equatable, Sendable, Identifiable {
    var fingerprint: String
    var address: String?
    var name: String?
    var expiresUnixMs: Double?
    var id: String { fingerprint }
}

struct DaemonError: Codable, Equatable, Sendable {
    var code: String?
    var message: String?
    var unixMs: Double?
}

struct Stats: Codable, Equatable, Sendable {
    var rssBytes: Double?
    var eventsIn: Double?
    var eventsOut: Double?
}

struct DiscoveredPeer: Codable, Equatable, Sendable, Identifiable {
    var name: String
    var host: String
    var port: Int?
    var fingerprintHint: String?
    var id: String { "\(name)@\(host):\(port ?? 0)" }

    var address: String { port.map { "\(host):\($0)" } ?? host }
}

struct TrustList: Codable, Equatable, Sendable {
    var servers: [String]?
    var clients: [String]?
}
