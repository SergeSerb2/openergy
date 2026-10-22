import AppKit
import Observation
import OSLog
import ServiceManagement
import SystemConfiguration

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
    /// Non-error information from the last action, e.g. config warnings or import notes.
    private(set) var notice: String?
    private(set) var launchAtLogin = SMAppService.mainApp.status == .enabled
    private(set) var launchAtLoginNeedsApproval = SMAppService.mainApp.status == .requiresApproval
    let isDemo: Bool

    static let shared = AppModel()

    /// The Mac's user-visible name, read once (Host.current() can block on the network).
    static let computerName: String = {
        if let name = SCDynamicStoreCopyComputerName(nil, nil) as String?, !name.isEmpty { return name }
        return ProcessInfo.processInfo.hostName.replacingOccurrences(of: ".local", with: "")
    }()

    private let client = DaemonClient()
    private let supervisor = DaemonSupervisor()
    private let log = Logger(subsystem: "dev.openergy.app", category: "model")
    private var connectTask: Task<Void, Never>?
    /// What the Activity pane wants; re-applied after every reconnect.
    private var logsWanted = false

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
    var isReady: Bool { link == .connected }

    var localName: String {
        if let name = status.name, !name.isEmpty { return name }
        if let name = config.name, !name.isEmpty { return name }
        return Self.computerName
    }

    /// True while the shared pointer is on this Mac. A client only learns this from its server,
    /// so with no server connected the pointer is, by definition, local.
    var pointerOnThisMac: Bool {
        guard role == .client, !connectedPeers.isEmpty else { return true }
        return status.activeScreen == localName
    }

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
            guard connectedPeers.isEmpty else { return "This Mac's keyboard and mouse" }
            let port = config.server?.port ?? 24800
            let address = LocalAddresses.joinable().first.map { port == 24800 ? $0 : "\($0):\(port)" }
            return address.map { "Other computers join \($0)" } ?? "Listening on port \(port)"
        case .client:
            if let peer = connectedPeers.first {
                return "\(peer.address ?? "") · \(peer.tls?.enabled == true ? (peer.tls?.version ?? "TLS") : "Not encrypted")"
            }
            // After Disconnect the daemon is idle: an old error would only confuse.
            if let error = status.lastError?.message, [.connecting, .reconnecting, .error].contains(state) {
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
        var lastError: Error?
        while !Task.isCancelled {
            do {
                try client.connect(path: DaemonSupervisor.socketPath)
                try await client.call("events.subscribe")
                link = .connected
                supervisor.markHealthy()
                await refreshAll()
                if logsWanted { await applyLogSubscription() }
                // The socket can close while refreshing; then keep looping instead of stopping here.
                if client.isConnected { return }
                link = .connecting
            } catch {
                lastError = error
                // Start the daemon once; after that the supervisor owns restarts and their backoff.
                if !supervisor.hasStarted, DaemonSupervisor.executableURL != nil {
                    supervisor.start()
                } else if DaemonSupervisor.executableURL == nil {
                    link = .unavailable("The openergy service binary is missing from the app.")
                }
                if waited > .seconds(8) {
                    let reason = (lastError as? IPCError)?.errorDescription ?? "The Openergy service is not responding."
                    link = .unavailable("\(reason) Retrying…")
                }
            }
            try? await Task.sleep(for: delay)
            waited += delay
            delay = min(delay * 2, .seconds(2))
        }
    }

    private func handleDisconnect() {
        link = .connecting
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
            if let line = data["line"]?.stringValue ?? data.stringValue { appendLog(line) }
        case "peer.untrusted":
            // Bring the approval into view right away; the status event may be coalesced.
            Task { if let next = try? await client.call("status.get").decode(DaemonStatus.self) { status = next } }
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

    /// Join a server. Works again after Disconnect, also with the same address.
    func join(_ address: String) {
        let trimmed = address.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        mutateDemo { $0.config.client = .init(remote: trimmed, autoconnect: true); $0.config.role = .client; $0.status.role = .client }
        guard !isDemo else { return }
        Task {
            do {
                let result = try await client.call("config.patch", ["patch": [
                    "role": "client", "client": ["remote": .string(trimmed), "autoconnect": true],
                ]])
                applyPatchResult(result)
                // Disconnect switches connecting off; connect without a host switches it on again.
                try await client.call("connection.connect")
                if let next = try? await client.call("status.get").decode(DaemonStatus.self) { status = next }
                lastActionError = nil
            } catch {
                lastActionError = error.localizedDescription
            }
        }
    }

    func reconnect() { perform("connection.restart") }
    func disconnect() { perform("connection.disconnect", refresh: true) }

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
                if case .array(let items)? = result["report"]?["warnings"], !items.isEmpty {
                    notice = "Imported with notes: " + items.compactMap(\.stringValue).joined(separator: " ")
                } else {
                    notice = "Imported your Synergy settings."
                }
                lastActionError = nil
                await refreshAll()
            } catch {
                lastActionError = error.localizedDescription
            }
        }
    }

    /// Saves the whole server layout after a drag. Rects use screen points.
    /// The daemon accepts only non-negative origins, so the layout is shifted to start at (0, 0);
    /// only relative positions matter for links.
    func saveLayout(_ screens: [OpenergyConfig.Screen]) {
        let placed = screens.compactMap(\.rect)
        let dx = placed.map(\.x).min() ?? 0
        let dy = placed.map(\.y).min() ?? 0
        let normalized = screens.map { screen -> OpenergyConfig.Screen in
            var screen = screen
            if var rect = screen.rect {
                rect.x = (rect.x - dx).rounded()
                rect.y = (rect.y - dy).rounded()
                rect.w = rect.w.rounded()
                rect.h = rect.h.rounded()
                screen.rect = rect
            }
            return screen
        }
        mutateDemo { $0.config.screens = normalized }
        config.screens = normalized
        sendScreens(normalized.map { ($0.name, $0) })
    }

    /// Renames this Mac. While hosting, its layout entry is renamed in the same patch so its
    /// position and links stay attached.
    func renameLocal(to newName: String) {
        let name = newName.trimmingCharacters(in: .whitespaces)
        guard !name.isEmpty, name != localName else { return }
        guard !isDemo else { return }
        let oldName = localName
        let screens = config.screens ?? []
        var patch: [String: JSONValue] = ["name": .string(name)]
        if screens.contains(where: { $0.name == oldName }) {
            let renamed = screens.map { screen -> (String, OpenergyConfig.Screen) in
                var copy = screen
                if screen.name == oldName { copy.name = name }
                return (screen.name, copy)
            }
            patch["screen"] = .array(screenObjects(renamed))
        }
        perform("config.patch", ["patch": .object(patch)])
    }

    func setLaunchAtLogin(_ enabled: Bool) {
        do {
            if enabled { try SMAppService.mainApp.register() } else { try SMAppService.mainApp.unregister() }
        } catch {
            lastActionError = "Could not change the login item: \(error.localizedDescription)"
        }
        refreshLoginItem()
        if launchAtLoginNeedsApproval { SMAppService.openSystemSettingsLoginItems() }
    }

    func refreshLoginItem() {
        launchAtLogin = SMAppService.mainApp.status == .enabled
        launchAtLoginNeedsApproval = SMAppService.mainApp.status == .requiresApproval
    }

    func clearActionError() { lastActionError = nil }
    func clearNotice() { notice = nil }

    func subscribeLogs(_ on: Bool) {
        guard !isDemo, on != logsWanted else { return }
        logsWanted = on
        Task { await applyLogSubscription() }
    }

    private func applyLogSubscription() async {
        if logsWanted, case .array(let items)? = try? await client.call("logs.tail", ["lines": 400]) {
            logLines = items.compactMap { $0["line"]?.stringValue ?? $0.stringValue }
        }
        _ = try? await client.call(logsWanted ? "logs.subscribe" : "logs.unsubscribe")
    }

    func revealConfigFile() {
        let url = DaemonSupervisor.supportDirectory.appendingPathComponent("config.toml")
        NSWorkspace.shared.activateFileViewerSelecting([url])
    }

    // MARK: Plumbing

    /// Builds `[[screen]]` objects from the daemon's raw entries (looked up by their current name),
    /// changing only `name` and `rect`, so imported aliases, links, modifiers and options survive.
    private func screenObjects(_ screens: [(String, OpenergyConfig.Screen)]) -> [JSONValue] {
        var raw: [String: [String: JSONValue]] = [:]
        for item in config.rawScreens {
            if let object = item.objectValue, let name = object["name"]?.stringValue { raw[name] = object }
        }
        return screens.map { originalName, screen in
            var object = raw[originalName] ?? [:]
            object["name"] = .string(screen.name)
            if let rect = screen.rect {
                object["rect"] = ["x": .number(rect.x), "y": .number(rect.y), "w": .number(rect.w), "h": .number(rect.h)]
            } else {
                object.removeValue(forKey: "rect")
            }
            if object["os"] == nil, let os = screen.os { object["os"] = .string(os) }
            return .object(object)
        }
    }

    private func sendScreens(_ screens: [(String, OpenergyConfig.Screen)]) {
        perform("config.patch", ["patch": ["screen": .array(screenObjects(screens))]])
    }

    private func applyPatchResult(_ result: JSONValue) {
        guard var next = try? OpenergyConfig.decode(fromResult: result) else { return }
        // `config.patch` answers without derived links; keep the last ones until config.changed.
        if next.derivedLinks == nil { next.derivedLinks = config.derivedLinks }
        config = next
        if case .array(let warnings)? = result["warnings"], !warnings.isEmpty {
            notice = warnings.compactMap(\.stringValue).joined(separator: " ")
        }
    }

    private func perform(_ method: String, _ params: JSONValue = [:], refresh: Bool = false) {
        guard !isDemo else { return }
        Task {
            do {
                let result = try await client.call(method, params)
                if method == "config.patch" {
                    applyPatchResult(result)
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
