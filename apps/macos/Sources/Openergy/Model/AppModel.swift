import AppKit
import Observation
import OSLog
import ServiceManagement

/// The app's single source of truth. It mirrors daemon state and sends user intent back.
@MainActor
@Observable
final class AppModel {
    enum ServiceLink: Equatable {
        case connecting
        case connected
        case unavailable(String)
    }

    private(set) var status = DaemonStatus.empty
    private(set) var config = OpenergyConfig.empty
    private(set) var discovered: [DiscoveredPeer] = []
    private(set) var trusted = TrustList()
    private(set) var logLines: [String] = []
    private(set) var link: ServiceLink = .connecting
    private(set) var lastActionError: String?
    private(set) var launchAtLogin = SMAppService.mainApp.status == .enabled
    let isDemo: Bool

    static let shared = AppModel()

    private let client = DaemonClient()
    private let supervisor = DaemonSupervisor()
    private let log = Logger(subsystem: "dev.openergy.app", category: "model")
    private var connectTask: Task<Void, Never>?
    private var logsSubscribed = false

    init(demo: DemoScenario? = DemoScenario.fromEnvironment) {
        isDemo = demo != nil
        if let demo {
            status = demo.status
            config = demo.config
            discovered = demo.discovered
            trusted = demo.trusted
            logLines = demo.logs
            link = .connected
        }
    }

    // MARK: Derived state

    var role: Role { status.role ?? config.role ?? .off }
    var state: ConnectionState { status.state ?? .idle }
    var connectedPeers: [Peer] { (status.peers ?? []).filter { $0.connected == true } }
    var missingPermissions: [PermissionKind] { status.permissions?.missing(for: role) ?? [] }
    var pendingTrust: [PendingTrust] { status.pendingTrust ?? [] }
    var localName: String { status.name ?? config.name ?? Host.current().localizedName ?? "This Mac" }

    var headline: String {
        switch link {
        case .connecting: return "Starting…"
        case .unavailable: return "Service unavailable"
        case .connected: break
        }
        switch role {
        case .off:
            return "Off"
        case .server:
            let count = connectedPeers.count
            if count == 0 { return state == .error ? "Cannot host" : "Waiting for computers" }
            return count == 1 ? "Sharing with \(connectedPeers[0].name)" : "Sharing with \(count) computers"
        case .client:
            if let peer = connectedPeers.first { return "Using \(peer.name)" }
            switch state {
            case .connecting, .starting: return "Connecting…"
            case .reconnecting: return "Reconnecting…"
            case .error: return "Cannot connect"
            default: return "Not connected"
            }
        }
    }

    var detail: String {
        if case .unavailable(let message) = link { return message }
        switch role {
        case .off:
            return "Choose Host or Join to start."
        case .server:
            if let error = status.lastError?.message, state == .error { return error }
            let listen = status.server?.listen ?? "port \(config.server?.port ?? 24800)"
            return connectedPeers.isEmpty ? "Listening on \(listen)" : "This Mac's keyboard and mouse"
        case .client:
            if let peer = connectedPeers.first {
                return "\(peer.address ?? "") · \(peer.tls?.enabled == true ? (peer.tls?.version ?? "TLS") : "Not encrypted")"
            }
            if let error = status.lastError?.message, state != .connected {
                guard let retry = status.client?.nextRetryMs, retry > 0 else { return error }
                return "\(error) Trying again in \(Int((retry / 1000).rounded(.up))) s."
            }
            if let remote = config.client?.remote, !remote.isEmpty { return "Server \(remote)" }
            return "Choose a computer to join"
        }
    }

    var tone: Tone {
        if case .unavailable = link { return .bad }
        if !missingPermissions.isEmpty || !pendingTrust.isEmpty { return .warning }
        switch (role, state) {
        case (.off, _): return .neutral
        case (_, .connected): return .good
        case (.server, .listening): return connectedPeers.isEmpty ? .waiting : .good
        case (_, .error): return .bad
        default: return .waiting
        }
    }

    enum Tone { case good, waiting, warning, bad, neutral }

    // MARK: Lifecycle

    func start() {
        guard !isDemo, connectTask == nil else { return }
        client.onEvent = { [weak self] name, data in self?.handleEvent(name, data) }
        client.onDisconnect = { [weak self] _ in self?.handleDisconnect() }
        supervisor.onSpawn = { [weak self] in self?.link = .connecting }
        connectTask = Task { [weak self] in await self?.connectLoop() }
    }

    func shutdown() {
        connectTask?.cancel()
        client.disconnect()
        supervisor.stop()
    }

    private func connectLoop() async {
        defer { connectTask = nil }
        link = .connecting
        var delay: Duration = .milliseconds(100)
        var waited: Duration = .zero
        while !Task.isCancelled {
            do {
                try client.connect(path: DaemonSupervisor.socketPath)
                try await client.call("events.subscribe")
                link = .connected
                supervisor.markHealthy()
                await refreshAll()
                return
            } catch {
                if !supervisor.isRunning, DaemonSupervisor.executableURL != nil {
                    supervisor.start()
                } else if DaemonSupervisor.executableURL == nil {
                    link = .unavailable("The openergy service binary is missing from the app.")
                }
                if waited > .seconds(8) {
                    link = .unavailable("The Openergy service is not responding. Retrying…")
                }
                try? await Task.sleep(for: delay)
                waited += delay
                delay = min(delay * 2, .seconds(2))
            }
        }
    }

    private func handleDisconnect() {
        link = .connecting
        logsSubscribed = false
        if connectTask == nil {
            connectTask = Task { [weak self] in await self?.connectLoop() }
        }
    }

    private func handleEvent(_ name: String, _ data: JSONValue) {
        switch name {
        case "status.changed":
            if let next = try? data.decode(DaemonStatus.self) { status = next }
        case "config.changed":
            if let next = try? OpenergyConfig.decode(fromResult: data) { config = next }
        case "discovery.changed":
            if let next = try? data.decode([DiscoveredPeer].self) { discovered = next }
        case "log.line":
            if let line = data.stringValue { appendLog(line) }
        default:
            break
        }
    }

    func refreshAll() async {
        guard !isDemo else { return }
        async let status = try? client.call("status.get").decode(DaemonStatus.self)
        async let config = try? OpenergyConfig.decode(fromResult: client.call("config.get"))
        async let found = try? client.call("discovery.list").decode([DiscoveredPeer].self)
        async let trust = try? client.call("trust.list").decode(TrustList.self)
        if let value = await status { self.status = value }
        if let value = await config { self.config = value }
        if let value = await found { self.discovered = value }
        if let value = await trust { self.trusted = value }
    }

    // MARK: Intents

    func setRole(_ role: Role) {
        mutateDemo { $0.status.role = role; $0.config.role = role }
        perform("config.patch", ["patch": ["role": .string(role.rawValue)]])
    }

    func patch(_ path: String, _ value: JSONValue) {
        mutateDemoConfig(path, value)
        perform("config.patch", ["patch": JSONValue.patch(path, value)])
    }

    func join(_ address: String) {
        let trimmed = address.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        mutateDemo { $0.config.client = .init(remote: trimmed, autoconnect: true); $0.config.role = .client; $0.status.role = .client }
        perform("config.patch", ["patch": ["role": "client", "client": ["remote": .string(trimmed), "autoconnect": true]]])
    }

    func reconnect() { perform("connection.restart") }
    func disconnect() { perform("connection.disconnect") }

    func approve(_ fingerprint: String) {
        mutateDemo { $0.status.pendingTrust?.removeAll { $0.fingerprint == fingerprint } }
        perform("trust.approve", ["fingerprint": .string(fingerprint)], refresh: true)
    }

    func reject(_ fingerprint: String) {
        mutateDemo { $0.status.pendingTrust?.removeAll { $0.fingerprint == fingerprint } }
        perform("trust.reject", ["fingerprint": .string(fingerprint)], refresh: true)
    }

    func revoke(_ fingerprint: String) {
        mutateDemo { model in
            model.trusted.servers?.removeAll { $0 == fingerprint }
            model.trusted.clients?.removeAll { $0 == fingerprint }
        }
        perform("trust.revoke", ["fingerprint": .string(fingerprint)], refresh: true)
    }

    func requestPermission(_ kind: PermissionKind) {
        perform("permissions.request", ["kind": .string(kind.rawValue)])
        NSWorkspace.shared.open(kind.settingsURL)
    }

    func importFromSynergy() {
        let path = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Preferences/Synergy").path
        Task {
            do {
                let result = try await client.call("config.import_synergy", ["path": .string(path), "apply": true])
                if let warnings = result["report"]?["warnings"], case .array(let items) = warnings, !items.isEmpty {
                    lastActionError = "Imported with notes: " + items.compactMap(\.stringValue).joined(separator: " ")
                }
                await refreshAll()
            } catch {
                lastActionError = error.localizedDescription
            }
        }
    }

    /// Saves the whole server layout after a drag. Rects use screen points.
    func saveLayout(_ screens: [OpenergyConfig.Screen]) {
        mutateDemo { $0.config.screens = screens }
        config.screens = screens
        let items: [JSONValue] = screens.map { screen in
            var object: [String: JSONValue] = ["name": .string(screen.name)]
            if let rect = screen.rect {
                object["rect"] = ["x": .number(rect.x.rounded()), "y": .number(rect.y.rounded()),
                                  "w": .number(rect.w.rounded()), "h": .number(rect.h.rounded())]
            }
            if let os = screen.os { object["os"] = .string(os) }
            return .object(object)
        }
        perform("config.patch", ["patch": ["screen": .array(items)]])
    }

    func setLaunchAtLogin(_ enabled: Bool) {
        do {
            if enabled { try SMAppService.mainApp.register() } else { try SMAppService.mainApp.unregister() }
        } catch {
            lastActionError = "Could not change the login item: \(error.localizedDescription)"
        }
        launchAtLogin = SMAppService.mainApp.status == .enabled
    }

    func clearActionError() { lastActionError = nil }

    func subscribeLogs(_ on: Bool) {
        guard !isDemo, on != logsSubscribed else { return }
        logsSubscribed = on
        Task {
            if on, let lines = try? await client.call("logs.tail", ["lines": 400]),
               case .array(let items) = lines {
                logLines = items.compactMap(\.stringValue)
            }
            _ = try? await client.call(on ? "logs.subscribe" : "logs.unsubscribe")
        }
    }

    func revealConfigFile() {
        let url = DaemonSupervisor.supportDirectory.appendingPathComponent("config.toml")
        NSWorkspace.shared.activateFileViewerSelecting([url])
    }

    // MARK: Plumbing

    private func perform(_ method: String, _ params: JSONValue = [:], refresh: Bool = false) {
        guard !isDemo else { return }
        Task {
            do {
                let result = try await client.call(method, params)
                if method == "config.patch" {
                    if let next = try? OpenergyConfig.decode(fromResult: result) { config = next }
                    // The daemon also pushes status.changed; fetching once keeps the UI in step even if
                    // that event is coalesced or arrives after the next user action.
                    if let next = try? await client.call("status.get").decode(DaemonStatus.self) { status = next }
                }
                lastActionError = nil
                if refresh { await refreshAll() }
            } catch {
                log.error("\(method) failed: \(error.localizedDescription)")
                lastActionError = error.localizedDescription
            }
        }
    }

    private func appendLog(_ line: String) {
        logLines.append(line)
        if logLines.count > 2000 { logLines.removeFirst(logLines.count - 2000) }
    }

    // Demo mode keeps the UI interactive without a daemon (screenshots, design review).
    private func mutateDemo(_ change: (inout DemoScenario) -> Void) {
        guard isDemo else { return }
        var scenario = DemoScenario(status: status, config: config, discovered: discovered, trusted: trusted, logs: logLines)
        change(&scenario)
        status = scenario.status
        config = scenario.config
        trusted = scenario.trusted
    }

    private func mutateDemoConfig(_ path: String, _ value: JSONValue) {
        guard isDemo else { return }
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        guard let data = try? encoder.encode(config),
              let current = try? JSONDecoder().decode(JSONValue.self, from: data),
              let next = try? current.merging(JSONValue.patch(path, value)).decode(OpenergyConfig.self)
        else { return }
        config = next
    }
}

extension JSONValue {
    /// RFC 7386 JSON merge patch.
    func merging(_ patch: JSONValue) -> JSONValue {
        guard case .object(let patchObject) = patch else { return patch }
        var result: [String: JSONValue] = [:]
        if case .object(let base) = self { result = base }
        for (key, value) in patchObject {
            if value == .null {
                result.removeValue(forKey: key)
            } else {
                result[key] = (result[key] ?? .null).merging(value)
            }
        }
        return .object(result)
    }
}
