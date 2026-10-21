import Foundation

/// Static data that lets the UI run without a daemon, for design review and snapshots.
/// Enable with `--demo <name>` or `OPENERGY_DEMO=<name>`.
struct DemoScenario {
    var status: DaemonStatus
    var config: OpenergyConfig
    var discovered: [DiscoveredPeer]
    var trusted: TrustList
    var logs: [String]

    static var fromEnvironment: DemoScenario? {
        let arguments = ProcessInfo.processInfo.arguments
        if let index = arguments.firstIndex(of: "--demo"), arguments.indices.contains(index + 1) {
            return named(arguments[index + 1])
        }
        if let name = ProcessInfo.processInfo.environment["OPENERGY_DEMO"] {
            return named(name)
        }
        return nil
    }

    static func named(_ name: String) -> DemoScenario {
        switch name {
        case "server": server
        case "setup": setup
        case "trust": trust
        case "offline": offline
        default: client
        }
    }

    private static let macFingerprint = "3F:A2:91:0C:7B:44:E8:15:6D:C3:28:9A:B0:57:F1:2E:84:6B:DD:39:0A:C5:72:1F:E6:93:48:B7:25:5C:0D:A1"
    private static let pcFingerprint = "9F:2C:4E:81:D7:36:A0:5B:C9:12:7E:F4:68:03:BB:5D:E2:91:4A:76:0F:C8:33:9E:A5:17:6C:D0:82:4F:41:7A"
    private static let linuxFingerprint = "B7:51:0E:C4:29:8A:F3:66:D1:05:7C:E9:42:B8:13:5A:90:2D:F7:6E:C1:38:A4:0B:85:DE:57:12:69:F0:3C:E4"

    private static let permissionsGranted = Permissions(accessibility: .granted, inputMonitoring: .granted, postEvents: .granted)

    private static func baseConfig(role: Role) -> OpenergyConfig {
        OpenergyConfig(
            role: role,
            name: "MacBook Pro",
            server: .init(port: 24800, listen: "0.0.0.0"),
            client: .init(remote: "192.168.1.20", autoconnect: true),
            tls: .init(enabled: true),
            clipboard: .init(enabled: true, maxBytes: 64 * 1024 * 1024),
            input: .init(relativeMouse: false, invertScrollY: false, scrollSensitivity: 1.0, syncLanguage: true, preventSleep: false),
            switch: .init(delayMs: 0, doubleTapMs: 0, deadCorners: [], cornerSizePx: 12),
            hotkeys: .init(lockToScreen: "ctrl+alt+l"),
            screen: nil,
            derivedLinks: nil,
            macos: .init(clickNumbering: "native")
        )
    }

    private static let sampleLogs: [String] = [
        "2026-10-21 14:02:11.204 info daemon: Openergy 0.1.0 started (macos, pid 4211)",
        "2026-10-21 14:02:11.219 info net: TLS identity loaded, fingerprint 3F:A2:91:0C…0D:A1",
        "2026-10-21 14:02:11.240 info client: connecting to 192.168.1.20:24800",
        "2026-10-21 14:02:11.262 info net: TLS 1.3 established with 192.168.1.20 (trusted)",
        "2026-10-21 14:02:11.270 info client: server speaks Synergy 1.8, screen name accepted",
        "2026-10-21 14:02:11.271 info client: options: heartbeat 5000 ms, clipboard 4000 KB, relative moves on",
        "2026-10-21 14:03:40.118 info client: entered screen at 1511,604",
        "2026-10-21 14:03:52.902 info clipboard: received 2 formats (text, html), 1.8 KB",
        "2026-10-21 14:04:05.311 info client: left screen",
        "2026-10-21 14:06:13.774 warn clipboard: not sent — 6.2 MB exceeds studio-pc's limit of 4.0 MB",
    ]

    static let client = DemoScenario(
        status: DaemonStatus(
            daemon: .init(version: "0.1.0", pid: 4211, uptimeS: 7_380, platform: "macos"),
            role: .client,
            state: .connected,
            name: "MacBook Pro",
            activeScreen: "MacBook Pro",
            cursor: .init(screen: "MacBook Pro", x: 812, y: 455),
            localScreen: .init(w: 1512, h: 982, displays: [.init(x: 0, y: 0, w: 1512, h: 982, scale: 2)]),
            server: nil,
            client: .init(remote: "192.168.1.20:24800", nextRetryMs: nil, attempt: 0),
            peers: [
                Peer(name: "studio-pc", address: "192.168.1.20:24800", role: "server", connected: true,
                     sinceUnixMs: Date().addingTimeInterval(-7_200).timeIntervalSince1970 * 1000, rttMs: 0.9,
                     protocol: .init(product: "Synergy", major: 1, minor: 8),
                     screen: .init(w: 2560, h: 1440),
                     tls: .init(enabled: true, version: "TLS 1.3", fingerprint: pcFingerprint, trusted: true),
                     secureInputApp: nil, os: "windows"),
            ],
            tls: .init(enabled: true, fingerprint: macFingerprint),
            permissions: permissionsGranted,
            secureInput: .init(localApp: nil),
            clipboard: .init(
                enabled: true,
                lastTransfer: .init(direction: "in", bytes: 1_843, formats: ["text", "html"], unixMs: Date().addingTimeInterval(-300).timeIntervalSince1970 * 1000),
                lastBlocked: .init(bytes: 6_500_000, limit: 4_096_000, peer: "studio-pc", unixMs: Date().addingTimeInterval(-120).timeIntervalSince1970 * 1000)
            ),
            pendingTrust: [],
            lastError: nil,
            stats: .init(rssBytes: 9_400_000, eventsIn: 182_331, eventsOut: 2_014)
        ),
        config: baseConfig(role: .client),
        discovered: [
            DiscoveredPeer(name: "studio-pc", host: "192.168.1.20", port: 24800, fingerprint: pcFingerprint),
            DiscoveredPeer(name: "studio-linux", host: "192.168.1.31", port: 24800, fingerprint: linuxFingerprint),
        ],
        trusted: TrustList(servers: [pcFingerprint], clients: []),
        logs: sampleLogs
    )

    static let server: DemoScenario = {
        var scenario = client
        scenario.status.role = .server
        scenario.status.state = .listening
        scenario.status.server = .init(listen: "0.0.0.0:24800")
        scenario.status.client = nil
        scenario.status.activeScreen = "studio-pc"
        scenario.status.cursor = .init(screen: "studio-pc", x: 1200, y: 700)
        scenario.status.peers = [
            Peer(name: "studio-pc", address: "192.168.1.20:52114", role: "client", connected: true,
                 sinceUnixMs: Date().addingTimeInterval(-3_600).timeIntervalSince1970 * 1000, rttMs: 1.1,
                 protocol: .init(product: "Synergy", major: 1, minor: 8),
                 screen: .init(w: 2560, h: 1440),
                 tls: .init(enabled: true, version: "TLS 1.3", fingerprint: pcFingerprint, trusted: true),
                 secureInputApp: nil, os: "windows"),
            Peer(name: "studio-linux", address: "192.168.1.31:40022", role: "client", connected: false,
                 sinceUnixMs: nil, rttMs: nil, protocol: nil, screen: .init(w: 1920, h: 1080),
                 tls: .init(enabled: true, version: nil, fingerprint: linuxFingerprint, trusted: true),
                 secureInputApp: nil, os: "linux"),
        ]
        scenario.config = baseConfig(role: .server)
        scenario.config.screens = [
            .init(name: "MacBook Pro", rect: .init(x: 0, y: 300, w: 1512, h: 982), os: "macos"),
            .init(name: "studio-pc", rect: .init(x: 1512, y: 0, w: 2560, h: 1440), os: "windows"),
            .init(name: "studio-linux", rect: nil, os: "linux"),
        ]
        scenario.trusted = TrustList(servers: [], clients: [pcFingerprint, linuxFingerprint])
        return scenario
    }()

    static let setup: DemoScenario = {
        var scenario = client
        scenario.status.role = .off
        scenario.status.state = .idle
        scenario.status.peers = []
        scenario.status.permissions = Permissions(accessibility: .granted, inputMonitoring: .denied, postEvents: .granted)
        scenario.config = baseConfig(role: .off)
        return scenario
    }()

    static let trust: DemoScenario = {
        var scenario = client
        scenario.status.state = .connecting
        scenario.status.peers = []
        scenario.status.pendingTrust = [
            PendingTrust(fingerprint: pcFingerprint, address: "192.168.1.20:24800", name: "studio-pc",
                         expiresUnixMs: Date().addingTimeInterval(25).timeIntervalSince1970 * 1000),
        ]
        scenario.trusted = TrustList(servers: [], clients: [])
        return scenario
    }()

    static let offline: DemoScenario = {
        var scenario = client
        scenario.status.state = .reconnecting
        scenario.status.peers = []
        scenario.status.client = .init(remote: "192.168.1.20:24800", nextRetryMs: 4_000, attempt: 5)
        scenario.status.lastError = .init(code: "connect_timeout", message: "No answer from 192.168.1.20:24800. Retrying in 4 s.", unixMs: nil)
        return scenario
    }()
}
