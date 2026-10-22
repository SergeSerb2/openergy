import AppKit
import SwiftUI

/// End-to-end check of the app against a real daemon, with no visible windows.
///
/// Requires explicit OPENERGY_CONFIG, OPENERGY_CONFIG_DIR, OPENERGY_TLS_DIR,
/// OPENERGY_SOCKET and OPENERGY_DAEMON_ARGS="--platform virtual", all under output-dir.
/// Use OPENERGY_E2E_NO_RENDER=1 to exercise the model without starting AppKit.
///
/// It connects through the real `AppModel` and IPC client, drives the same intents the
/// UI buttons call, waits for the daemon to reflect each change in `status`/`config`,
/// renders the panes offscreen, and exits 0 only if every step passed.
@MainActor
enum AppEndToEnd {
    private struct Step: Encodable {
        var name: String
        var passed: Bool
        var detail: String
    }

    static func run(outputDirectory: URL) {
        // E2E daemons must never be allowed to host on the live Synergy port.
        setenv("OPENERGY_ALLOW_PORT_24800", "0", 1)
        let arguments = ProcessInfo.processInfo.arguments
        func value(_ flag: String) -> String? {
            arguments.firstIndex(of: flag).flatMap { arguments.indices.contains($0 + 1) ? arguments[$0 + 1] : nil }
        }
        guard let joinAddress = value("--join"), let hostPort = value("--host-port").flatMap(Int.init),
              validPort(hostPort), let remotePort = joinAddress.split(separator: ":").last.flatMap({ Int($0) }),
              joinAddress == "127.0.0.1:\(remotePort)", validPort(remotePort),
              isolatedPaths(under: outputDirectory) else {
            FileHandle.standardError.write(Data("E2E requires a private temporary output directory, explicit isolated config/socket/TLS paths, --platform virtual, and explicit loopback --join and --host-port (never 24800).\n".utf8))
            exit(64)
        }
        let noRender = ProcessInfo.processInfo.environment["OPENERGY_E2E_NO_RENDER"] == "1"
        if !noRender { NSApplication.shared.setActivationPolicy(.prohibited) }
        do { try FileManager.default.createDirectory(at: outputDirectory, withIntermediateDirectories: true, attributes: [.posixPermissions: 0o700]) }
        catch {
            FileHandle.standardError.write(Data("Could not create E2E output directory: \(error.localizedDescription)\n".utf8))
            exit(1)
        }

        Task { @MainActor in
            let model = AppModel(demo: nil)
            var steps: [Step] = []

            func record(_ name: String, _ passed: Bool, _ detail: String) {
                steps.append(Step(name: name, passed: passed, detail: detail))
                print("\(passed ? "PASS" : "FAIL")  \(name) — \(detail)")
            }

            model.start()
            let connected = await waitUntil(seconds: 15) { model.link == .connected && model.status.daemon?.platform == "virtual" }
            record("connect", connected, "link=\(model.link) daemon=\(model.status.daemon?.version ?? "none") pid=\(model.status.daemon?.pid.map(String.init) ?? "-")")
            guard connected else { finish(steps, outputDirectory, model: model) ; return }

            render(model, .layout, "01-layout-initial", outputDirectory)
            render(model, .advanced, "02-advanced", outputDirectory)

            model.setRole(.off)
            let off = await waitUntil(seconds: 10) { model.status.role == .off && model.config.role == .off }
            record("role off", off, "status.role=\(model.status.role?.rawValue ?? "nil") config.role=\(model.config.role?.rawValue ?? "nil") error=\(model.lastActionError ?? "none")")

            model.join(joinAddress)
            let joined = await waitUntil(seconds: 10) {
                model.status.role == .client && model.config.client?.remote == joinAddress
                    && [.connecting, .reconnecting, .connected, .error].contains(model.state)
            }
            record("join \(joinAddress)", joined,
                   "role=\(model.status.role?.rawValue ?? "nil") state=\(model.state.rawValue) remote=\(model.config.client?.remote ?? "nil") error=\(model.status.lastError?.message ?? "none")")
            if arguments.contains("--require-peer") {
                let peer = await waitUntil(seconds: 10) { model.state == .connected && !model.connectedPeers.isEmpty }
                record("real peer connected", peer, "state=\(model.state.rawValue) peers=\(model.connectedPeers.count)")
                model.disconnect()
                let disconnected = await waitUntil(seconds: 10) { model.state == .idle && model.connectedPeers.isEmpty }
                record("disconnect peer", disconnected, "state=\(model.state.rawValue) peers=\(model.connectedPeers.count)")
                model.join(joinAddress)
                let rejoined = await waitUntil(seconds: 10) { model.state == .connected && !model.connectedPeers.isEmpty }
                record("rejoin same address", rejoined, "state=\(model.state.rawValue) peers=\(model.connectedPeers.count)")
            }
            render(model, .layout, "03-layout-joining", outputDirectory)
            render(model, .connection, "04-connection-joining", outputDirectory)

            model.patch("clipboard.enabled", false)
            let clipboardOff = await waitUntil(seconds: 10) { model.config.clipboard?.enabled == false }
            record("clipboard off", clipboardOff, "config.clipboard.enabled=\(String(describing: model.config.clipboard?.enabled))")
            model.patch("clipboard.enabled", true)
            let clipboardOn = await waitUntil(seconds: 10) { model.config.clipboard?.enabled == true }
            record("clipboard on", clipboardOn, "config.clipboard.enabled=\(String(describing: model.config.clipboard?.enabled))")
            render(model, .clipboard, "05-clipboard", outputDirectory)

            // Never bind the real Synergy port in tests: require an explicit loopback host port.
            model.patch("server", ["port": .number(Double(hostPort)), "listen": .string("127.0.0.1:\(hostPort)")])
            let port = await waitUntil(seconds: 10) { model.config.server?.port == hostPort }
            record("test port", port, "server=\(model.config.server?.listen ?? "nil")")
            model.setRole(.server)
            let hosting = await waitUntil(seconds: 10) { model.status.role == .server && model.state == .listening }
            record("role server", hosting, "role=\(model.status.role?.rawValue ?? "nil") state=\(model.state.rawValue) listen=\(model.status.server?.listen ?? "nil")")
            render(model, .security, "07-security", outputDirectory)

            // Layout: an imported screen with extra fields, then a drag that puts it LEFT of this
            // Mac (a negative x before the app normalises the layout).
            let local = model.localName
            model.patch("screen", .array([
                ["name": .string(local), "rect": ["x": 0, "y": 0, "w": 1512, "h": 982]],
                ["name": "e2e-pc", "aliases": ["e2e-pc-alias"], "links": ["left": .string(local), "ranges": [
                    ["edge": "top", "from_range": [0, 50], "to": .string(local), "to_range": [50, 100]],
                ]],
                 "rect": ["x": 1512, "y": 0, "w": 1920, "h": 1080]],
            ]))
            let seeded = await waitUntil(seconds: 10) { model.config.screens?.contains { $0.name == "e2e-pc" } == true }
            record("layout seeded", seeded, "screens=\(model.config.screens?.map(\.name) ?? []) error=\(model.lastActionError ?? "none")")
            model.saveLayout([
                .init(name: local, rect: .init(x: 0, y: 0, w: 1512, h: 982), os: "macos"),
                .init(name: "e2e-pc", rect: .init(x: -1920, y: -49, w: 1920, h: 1080), os: "windows"),
            ])
            let leftOf = await waitUntil(seconds: 10) {
                let rects = Dictionary(uniqueKeysWithValues: (model.config.screens ?? []).compactMap { s in s.rect.map { (s.name, $0) } })
                return rects["e2e-pc"]?.x == 0 && rects[local]?.x == 1920
                    && model.lastActionError == nil
            }
            record("layout left of this Mac", leftOf,
                   "rects=\((model.config.screens ?? []).map { "\($0.name)@\(Int($0.rect?.x ?? -1)),\(Int($0.rect?.y ?? -1))" }) error=\(model.lastActionError ?? "none")")
            let kept = await waitUntil(seconds: 5) {
                model.config.rawScreens.contains { $0["name"]?.stringValue == "e2e-pc" && $0["aliases"] != nil }
            }
            record("layout keeps imported fields", kept, "raw e2e-pc=\(model.config.rawScreens.first { $0["name"]?.stringValue == "e2e-pc" }.map { "\($0)" } ?? "missing")")
            let linked = await waitUntil(seconds: 10) {
                (model.config.derivedLinks ?? []).contains { $0.from == local && $0.edge == "left" && $0.to == "e2e-pc" }
            }
            record("derived links arrive", linked, "links=\((model.config.derivedLinks ?? []).map { "\($0.from).\($0.edge)→\($0.to)" })")
            render(model, .layout, "06-layout-hosting", outputDirectory)

            // Live logs reach the Activity pane.
            model.subscribeLogs(true)
            let tail = await waitUntil(seconds: 10) { !model.logLines.isEmpty }
            let before = model.logLines.count
            model.renameLocal(to: "e2e-mac")
            let renamed = await waitUntil(seconds: 10) {
                model.config.name == "e2e-mac" && model.config.screens?.contains { $0.name == "e2e-mac" && $0.rect != nil } == true
                    && model.config.screens?.contains { $0.name == local } == false
            }
            record("rename keeps layout entry", renamed, "name=\(model.config.name ?? "nil") screens=\(model.config.screens?.map(\.name) ?? [])")
            let renamedLinks = model.config.rawScreens.first { $0["name"]?.stringValue == "e2e-pc" }?["links"]?["left"]?.stringValue
            let rangedLinks = model.config.rawScreens.first { $0["name"]?.stringValue == "e2e-pc" }?["links"]?["ranges"]
            let rangedTarget: String?
            if case .array(let ranges)? = rangedLinks { rangedTarget = ranges.first?["to"]?.stringValue }
            else { rangedTarget = nil }
            record("rename keeps explicit links", renamedLinks == "e2e-mac" && rangedTarget == "e2e-mac",
                   "direct=\(renamedLinks ?? "missing") ranged=\(rangedTarget ?? "missing")")
            let live = await waitUntil(seconds: 10) { model.logLines.count > before }
            record("live log lines", tail && live, "tail=\(before) now=\(model.logLines.count) last=\(model.logLines.last.map { LogLine($0).display } ?? "none")")
            render(model, .activity, "08-activity", outputDirectory)
            model.subscribeLogs(false)

            let saved = model.config.screens ?? []
            model.saveLayout(saved.map { screen in
                var screen = screen
                if screen.name == "e2e-pc" { screen.rect?.w = 0 }
                return screen
            })
            let rejectedLayout = await waitUntil(seconds: 10) {
                model.lastActionError != nil && model.config.screens == saved
            }
            record("rejected layout rolls back", rejectedLayout, "error=\(model.lastActionError ?? "none")")
            model.clearActionError()

            // Join is two IPC requests; Off must run after both, not between them.
            model.join(joinAddress)
            model.setRole(.off)
            let ordered = await waitUntil(seconds: 10) { model.status.role == .off && model.config.role == .off && model.lastActionError == nil }
            try? await Task.sleep(for: .milliseconds(300))
            record("join then off preserves intent", ordered && model.status.role == .off, "role=\(model.role.rawValue)")

            // Exercise real EOF/reconnect and supervised restart without touching an external daemon.
            if let pid = model.supervisedDaemonPID {
                let probe = DaemonClient()
                do {
                    try await probe.connect(path: DaemonSupervisor.socketPath)
                    let stopped = kill(pid, SIGSTOP) == 0
                    if stopped {
                        // A frame larger than the socket buffer stalls a real write. The UI
                        // actor must still run its timer and close that session by its deadline.
                        let clock = ContinuousClock()
                        let start = clock.now
                        let request = Task {
                            try await probe.call("status.get", ["padding": .string(String(repeating: "x", count: 1_000_000))], timeout: .milliseconds(300))
                        }
                        try? await Task.sleep(for: .milliseconds(50))
                        let responsive = clock.now - start < .seconds(1)
                        var failed = false
                        do { _ = try await request.value } catch { failed = true }
                        let bounded = clock.now - start < .seconds(2)
                        _ = kill(pid, SIGCONT)
                        record("stalled IPC remains responsive", responsive && bounded && failed,
                               "responsive=\(responsive) bounded=\(bounded) requestFailed=\(failed)")
                    } else { record("stalled IPC setup", false, "could not stop owned child") }
                } catch { record("stalled IPC setup", false, error.localizedDescription) }
                probe.disconnect()
                let signalled = kill(pid, SIGKILL) == 0
                let cleared = await waitUntil(seconds: 3) { !model.isReady && model.connectedPeers.isEmpty && model.status.daemon == nil }
                let recovered = await waitUntil(seconds: 15) { model.isReady && model.supervisedDaemonPID != pid && model.status.daemon?.platform == "virtual" }
                record("owned daemon crash recovers", signalled && cleared && recovered,
                       "cleared=\(cleared) recovered=\(recovered)")
            }

            model.setRole(.off)
            let offAgain = await waitUntil(seconds: 10) { model.status.role == .off }
            record("role off again", offAgain, "role=\(model.status.role?.rawValue ?? "nil")")

            do {
                let fixture = DaemonSupervisor.supportDirectory.appendingPathComponent("synergy-fixture", isDirectory: true)
                try FileManager.default.createDirectory(at: fixture, withIntermediateDirectories: true)
                try """
                section: screens
                    e2e-import:
                end
                section: options
                    address = 127.0.0.1:\(hostPort)
                    e2eUnsupportedSetting = true
                end
                """.write(to: fixture.appendingPathComponent("synergy.conf"), atomically: true, encoding: .utf8)
                model.clearNotice()
                model.importFromSynergy(from: fixture)
                let warned = await waitUntil(seconds: 10) {
                    model.notice?.hasPrefix("Imported with notes:") == true && model.notice?.contains("e2eUnsupportedSetting") == true
                }
                record("import warning notice", warned, "notice=\(model.notice ?? "none")")
                let empty = DaemonSupervisor.supportDirectory.appendingPathComponent("empty-import", isDirectory: true)
                try FileManager.default.createDirectory(at: empty, withIntermediateDirectories: true)
                model.importFromSynergy(from: empty)
                let rejected = await waitUntil(seconds: 10) { model.lastActionError != nil }
                record("empty import reports error", rejected, "error=\(model.lastActionError ?? "none")")
            } catch {
                record("import fixture", false, error.localizedDescription)
            }

            finish(steps, outputDirectory, model: model)
        }
        if noRender { CFRunLoopRun() } else { NSApplication.shared.run() }
    }

    private static func validPort(_ port: Int) -> Bool { (1024...65535).contains(port) && port != 24800 }

    /// Resolve symlinks before checking containment, so a temp symlink cannot redirect
    /// config, logs, sockets or identity writes into the live installation.
    private static func isolatedPaths(under output: URL) -> Bool {
        func reject(_ reason: String) -> Bool {
            FileHandle.standardError.write(Data("Unsafe E2E settings: \(reason).\n".utf8))
            return false
        }
        let root = output.standardizedFileURL.resolvingSymlinksInPath()
        let temporaryRoots = [FileManager.default.temporaryDirectory, URL(fileURLWithPath: "/tmp")]
            .map { $0.standardizedFileURL.resolvingSymlinksInPath().path }
        guard temporaryRoots.contains(where: { root.path.hasPrefix($0 + "/") }) else { return reject("output must be inside a temporary directory") }
        guard let attributes = try? FileManager.default.attributesOfItem(atPath: root.path),
              let permissions = attributes[.posixPermissions] as? NSNumber,
              let owner = attributes[.ownerAccountID] as? NSNumber,
              permissions.intValue & 0o077 == 0, owner.uint32Value == getuid() else {
            return reject("output must be an existing private directory owned by this user")
        }
        let environment = ProcessInfo.processInfo.environment
        var urls: [String: URL] = [:]
        for key in ["OPENERGY_CONFIG", "OPENERGY_CONFIG_DIR", "OPENERGY_TLS_DIR", "OPENERGY_SOCKET"] {
            guard let path = environment[key], path.hasPrefix("/"), !path.utf8.contains(0) else { return reject("\(key) must be an absolute path") }
            let original = URL(fileURLWithPath: path).standardizedFileURL
            // Foundation does not consistently resolve /tmp aliases for a socket that
            // does not exist yet. Resolve the existing parent, then append its basename.
            guard (try? FileManager.default.destinationOfSymbolicLink(atPath: original.path)) == nil else {
                return reject("\(key) must not be a symbolic link")
            }
            let url = original.deletingLastPathComponent().resolvingSymlinksInPath()
                .appendingPathComponent(original.lastPathComponent)
            guard url.path.hasPrefix(root.path + "/") else { return reject("\(key) escapes the output directory") }
            urls[key] = url
        }
        guard let config = urls["OPENERGY_CONFIG"], let directory = urls["OPENERGY_CONFIG_DIR"],
              config.deletingLastPathComponent().path == directory.path else { return reject("config parent must match OPENERGY_CONFIG_DIR") }
        guard urls["OPENERGY_TLS_DIR"]?.path == directory.appendingPathComponent("tls").path else { return reject("TLS directory must be config-dir/tls") }
        guard environment["OPENERGY_DAEMON_ARGS"]?.split(separator: " ").map(String.init) == ["--platform", "virtual"] else { return reject("daemon arguments must be --platform virtual") }
        guard let socket = environment["OPENERGY_SOCKET"], socket.utf8.count < 104 else { return reject("socket path is too long") }
        guard FileManager.default.fileExists(atPath: config.path) else { return reject("explicit config file must exist") }
        return true
    }

    private static func waitUntil(seconds: Double, _ condition: @MainActor () -> Bool) async -> Bool {
        let deadline = Date().addingTimeInterval(seconds)
        while Date() < deadline {
            if condition() { return true }
            try? await Task.sleep(for: .milliseconds(100))
        }
        return condition()
    }

    private static func render(_ model: AppModel, _ pane: Pane, _ name: String, _ directory: URL) {
        // OPENERGY_E2E_NO_RENDER=1 measures the app's model/IPC footprint without window buffers.
        guard ProcessInfo.processInfo.environment["OPENERGY_E2E_NO_RENDER"] != "1" else { return }
        SnapshotRenderer.renderWindow(MainWindow(model: model, initialPane: pane).tint(Brand.accent),
                                      title: pane.title, appearance: .darkAqua,
                                      to: directory.appendingPathComponent("\(name).png"))
    }

    private static func finish(_ steps: [Step], _ directory: URL, model: AppModel) {
        model.shutdown()
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        do {
            try encoder.encode(steps).write(to: directory.appendingPathComponent("report.json"))
        } catch {
            FileHandle.standardError.write(Data("Could not save E2E report: \(error.localizedDescription)\n".utf8))
            exit(1)
        }
        let passed = !steps.isEmpty && steps.allSatisfy(\.passed)
        print(passed ? "E2E PASSED (\(steps.count) steps)" : "E2E FAILED")
        exit(passed ? 0 : 1)
    }
}
