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
    private(set) var launchAtLogin = false
    private(set) var launchAtLoginNeedsApproval = false
    let isDemo: Bool
    let isEndToEnd = ProcessInfo.processInfo.arguments.contains("--e2e")
    var allowsSystemIntegration: Bool { !isDemo && !isEndToEnd }

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
    private var actionTask: Task<Void, Never>?
    private var logSubscriptionTask: Task<Void, Never>?
    private var running = false
    private var session = UUID()
    private var statusRevision = UUID()
    private var configRevision = UUID()
    private var discoveryRevision = UUID()
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
        if allowsSystemIntegration { refreshLoginItem() }
    }

    // MARK: Derived state

    var role: Role { status.role ?? config.role ?? .off }
    var state: ConnectionState { status.state ?? .idle }
    var connectedPeers: [Peer] { (status.peers ?? []).filter { $0.connected == true } }
    var missingPermissions: [PermissionKind] { status.permissions?.missing(for: role) ?? [] }
    var pendingTrust: [PendingTrust] { status.pendingTrust ?? [] }
    var isReady: Bool { link == .connected }
    /// A PID is returned only for this model's own child, never an attached external daemon.
    var supervisedDaemonPID: Int32? { supervisor.processIdentifier }

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
        if link == .connecting { return .waiting }
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
        guard !isDemo, !running, connectTask == nil else { return }
        running = true
        client.onEvent = { [weak self] name, data in self?.handleEvent(name, data) }
        client.onDisconnect = { [weak self] _ in self?.handleDisconnect() }
        connectTask = Task { [weak self] in await self?.connectLoop() }
    }

    func shutdown() {
        running = false
        session = UUID()
        connectTask?.cancel()
        actionTask?.cancel()
        logSubscriptionTask?.cancel()
        client.disconnect()
        supervisor.stop()
        status = .empty
        link = .unavailable("The Openergy service has stopped.")
    }

    private func connectLoop() async {
        defer { connectTask = nil }
        link = .connecting
        var delay: Duration = .milliseconds(100)
        var waited: Duration = .zero
        var lastError: Error?
        while running, !Task.isCancelled {
            var reachedDaemon = false
            do {
                try await client.connect(path: DaemonSupervisor.socketPath)
                reachedDaemon = true
                try await client.call("events.subscribe")
                try await refreshCore()
                try Task.checkCancellation()
                // The socket can close while refreshing; then keep looping instead of stopping here.
                if running, client.isConnected {
                    link = .connected
                    supervisor.markHealthy()
                    if logsWanted { scheduleLogSubscription() }
                    return
                }
                link = .connecting
            } catch {
                guard running, !Task.isCancelled else { return }
                lastError = error
                client.disconnect()
                invalidateLiveState()
                // Start the daemon once; after that the supervisor owns restarts and their backoff.
                // A reachable socket with a failed handshake belongs to an existing daemon.
                if !reachedDaemon, !supervisor.hasStarted, DaemonSupervisor.executableURL != nil {
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
        guard running else { return }
        invalidateLiveState()
        link = .connecting
        if connectTask == nil {
            connectTask = Task { [weak self] in await self?.connectLoop() }
        }
    }

    private func invalidateLiveState() {
        session = UUID()
        status = .empty
        discovered = []
        trusted = TrustList()
    }

    private func handleEvent(_ name: String, _ data: JSONValue) {
        switch name {
        case "status.changed":
            if let next = try? data.decode(DaemonStatus.self) { status = next; statusRevision = UUID() }
        case "config.changed":
            if let next = try? OpenergyConfig.decode(fromResult: data) { config = next; configRevision = UUID() }
        case "discovery.changed":
            if let next = try? data.decode([DiscoveredPeer].self) { discovered = next; discoveryRevision = UUID() }
        case "log.line":
            if let line = data["line"]?.stringValue ?? data.stringValue { appendLog(line) }
        case "peer.untrusted":
            // Bring the approval into view right away; the status event may be coalesced.
            Task { try? await refreshStatus() }
        default:
            break
        }
    }

    func refreshAll() async {
        guard !isDemo else { return }
        try? await refreshCore()
    }

    private func refreshStatus() async throws {
        let current = session, revision = statusRevision
        let next = try await client.call("status.get").decode(DaemonStatus.self)
        try Task.checkCancellation()
        guard session == current, client.isConnected else { throw IPCError.closed }
        if statusRevision == revision { status = next; statusRevision = UUID() }
    }

    private func refreshCore() async throws {
        try await refreshStatus()
        let current = session, revision = configRevision
        let next = try await OpenergyConfig.decode(fromResult: client.call("config.get"))
        try Task.checkCancellation()
        guard session == current, client.isConnected else { throw IPCError.closed }
        if configRevision == revision { config = next; configRevision = UUID() }
        let foundRevision = discoveryRevision
        if let found = try? await client.call("discovery.list").decode([DiscoveredPeer].self),
           session == current, discoveryRevision == foundRevision {
            discovered = found
            discoveryRevision = UUID()
        }
        if let trust = try? await client.call("trust.list").decode(TrustList.self), session == current { trusted = trust }
        guard session == current, client.isConnected else { throw IPCError.closed }
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
        let trimmed = address.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        mutateDemo { $0.config.client = .init(remote: trimmed, autoconnect: true); $0.config.role = .client; $0.status.role = .client }
        guard !isDemo else { return }
        enqueueAction { [self] current in
            let revision = self.configRevision
            let result = try await client.call("config.patch", ["patch": [
                "role": "client", "client": ["remote": .string(trimmed), "autoconnect": true],
            ]])
            try checkSession(current)
            applyPatchResult(result, revision: revision)
            // Disconnect switches connecting off; connect without a host switches it on again.
            try await client.call("connection.connect")
            try checkSession(current)
            try await refreshCore()
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
        guard allowsSystemIntegration else { return }
        perform("permissions.request", ["kind": .string(kind.rawValue)])
        NSWorkspace.shared.open(kind.settingsURL)
    }

    func importFromSynergy(from directory: URL? = nil) {
        guard !isDemo else { return }
        let path: String
        if isEndToEnd {
            // E2E may import only an explicit fixture inside its already validated config dir.
            guard let directory else { return }
            let fixture = directory.standardizedFileURL.resolvingSymlinksInPath()
            let root = DaemonSupervisor.supportDirectory.standardizedFileURL.resolvingSymlinksInPath()
            guard fixture.path.hasPrefix(root.path + "/") else { return }
            path = fixture.path
        } else {
            path = (directory ?? FileManager.default.homeDirectoryForCurrentUser
                .appendingPathComponent("Library/Preferences/Synergy")).path
        }
        enqueueAction { [self] current in
            let result = try await client.call("config.import_synergy", ["path": .string(path), "apply": true])
            try checkSession(current)
            if case .array(let items)? = result["warnings"], !items.isEmpty {
                notice = "Imported with notes: " + items.compactMap(\.stringValue).joined(separator: " ")
            } else {
                notice = "Imported your Synergy settings."
            }
            await refreshAll()
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
        let name = newName.trimmingCharacters(in: .whitespacesAndNewlines)
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
            // Explicit imported links also refer to screen names. Rename their targets
            // atomically, otherwise the daemon rejects the whole layout as a dangling link.
            patch["screen"] = .array(screenObjects(renamed).map { item in
                guard var object = item.objectValue, var links = object["links"]?.objectValue else { return item }
                for edge in ["left", "right", "top", "bottom"] where links[edge]?.stringValue == oldName {
                    links[edge] = .string(name)
                }
                if case .array(let ranges)? = links["ranges"] {
                    links["ranges"] = .array(ranges.map { range in
                        guard var entry = range.objectValue, entry["to"]?.stringValue == oldName else { return range }
                        entry["to"] = .string(name)
                        return .object(entry)
                    })
                }
                object["links"] = .object(links)
                return .object(object)
            })
        }
        perform("config.patch", ["patch": .object(patch)])
    }

    func setLaunchAtLogin(_ enabled: Bool) {
        guard allowsSystemIntegration else { return }
        do {
            if enabled { try SMAppService.mainApp.register() } else { try SMAppService.mainApp.unregister() }
        } catch {
            lastActionError = "Could not change the login item: \(error.localizedDescription)"
        }
        refreshLoginItem()
        if launchAtLoginNeedsApproval { SMAppService.openSystemSettingsLoginItems() }
    }

    func refreshLoginItem() {
        guard allowsSystemIntegration else { return }
        launchAtLogin = SMAppService.mainApp.status == .enabled
        launchAtLoginNeedsApproval = SMAppService.mainApp.status == .requiresApproval
    }

    func clearActionError() { lastActionError = nil }
    func clearNotice() { notice = nil }

    func subscribeLogs(_ on: Bool) {
        guard !isDemo, on != logsWanted else { return }
        logsWanted = on
        scheduleLogSubscription()
    }

    private func scheduleLogSubscription() {
        logSubscriptionTask?.cancel()
        logSubscriptionTask = Task { await applyLogSubscription() }
    }

    private func applyLogSubscription() async {
        let current = session, wanted = logsWanted
        guard running, client.isConnected else { return }
        if wanted, case .array(let items)? = try? await client.call("logs.tail", ["lines": 400]),
           session == current, !Task.isCancelled {
            logLines = items.compactMap { $0["line"]?.stringValue ?? $0.stringValue }
        }
        guard session == current, !Task.isCancelled else { return }
        _ = try? await client.call(wanted ? "logs.subscribe" : "logs.unsubscribe")
    }

    func revealConfigFile() {
        guard allowsSystemIntegration else { return }
        let url = DaemonSupervisor.configURL
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

    private func applyPatchResult(_ result: JSONValue, revision: UUID) {
        guard var next = try? OpenergyConfig.decode(fromResult: result) else { return }
        // `config.patch` answers without derived links; keep the last ones until config.changed.
        if next.derivedLinks == nil { next.derivedLinks = config.derivedLinks }
        if configRevision == revision { config = next; configRevision = UUID() }
        if case .array(let warnings)? = result["warnings"], !warnings.isEmpty {
            notice = warnings.compactMap(\.stringValue).joined(separator: " ")
        }
    }

    private func perform(_ method: String, _ params: JSONValue = [:], refresh: Bool = false) {
        guard !isDemo else { return }
        enqueueAction { [self] current in
            let revision = configRevision
            let result = try await client.call(method, params)
            try checkSession(current)
            if method == "config.patch" {
                applyPatchResult(result, revision: revision)
                try await refreshCore()
            } else if refresh {
                try await refreshCore()
            }
        }
    }

    /// Preserve user intent order across multi-request actions such as Join then Off.
    private func enqueueAction(_ action: @escaping @MainActor (UUID) async throws -> Void) {
        let previous = actionTask, current = session
        actionTask = Task {
            await previous?.value
            guard running, session == current, !Task.isCancelled else { return }
            do {
                try await action(current)
                guard running, session == current, !Task.isCancelled else { return }
                lastActionError = nil
            } catch {
                guard running, session == current, !Task.isCancelled else { return }
                log.error("Action failed: \(error.localizedDescription)")
                lastActionError = error.localizedDescription
                // A rejected drag must return to the daemon's actual persisted layout.
                await refreshAll()
            }
        }
    }

    private func checkSession(_ current: UUID) throws {
        try Task.checkCancellation()
        guard running, session == current, client.isConnected else { throw IPCError.closed }
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
