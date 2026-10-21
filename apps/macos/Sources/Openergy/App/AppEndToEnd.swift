import AppKit
import SwiftUI

/// End-to-end check of the app against a real daemon, with no visible windows.
///
///   OPENERGY_SOCKET=/tmp/x.sock Openergy --e2e <output-dir> [--join host:port] [--host-port N]
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
        NSApplication.shared.setActivationPolicy(.prohibited)
        // E2E daemons must never be allowed to host on the live Synergy port.
        setenv("OPENERGY_ALLOW_PORT_24800", "0", 1)
        try? FileManager.default.createDirectory(at: outputDirectory, withIntermediateDirectories: true)
        let arguments = ProcessInfo.processInfo.arguments
        let joinAddress = arguments.firstIndex(of: "--join").flatMap { arguments.indices.contains($0 + 1) ? arguments[$0 + 1] : nil }
            ?? "127.0.0.1:24999"
        let hostPort = arguments.firstIndex(of: "--host-port")
            .flatMap { arguments.indices.contains($0 + 1) ? Int(arguments[$0 + 1]) : nil } ?? 24992

        Task { @MainActor in
            let model = AppModel(demo: nil)
            var steps: [Step] = []

            func record(_ name: String, _ passed: Bool, _ detail: String) {
                steps.append(Step(name: name, passed: passed, detail: detail))
                print("\(passed ? "PASS" : "FAIL")  \(name) — \(detail)")
            }

            model.start()
            let connected = await waitUntil(seconds: 15) { model.link == .connected && model.status.daemon?.version != nil }
            record("connect", connected, "link=\(model.link) daemon=\(model.status.daemon?.version ?? "none") pid=\(model.status.daemon?.pid.map(String.init) ?? "-")")
            guard connected else { finish(steps, outputDirectory) ; return }

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
            render(model, .layout, "03-layout-joining", outputDirectory)
            render(model, .connection, "04-connection-joining", outputDirectory)

            model.patch("clipboard.enabled", false)
            let clipboardOff = await waitUntil(seconds: 10) { model.config.clipboard?.enabled == false }
            record("clipboard off", clipboardOff, "config.clipboard.enabled=\(String(describing: model.config.clipboard?.enabled))")
            model.patch("clipboard.enabled", true)
            let clipboardOn = await waitUntil(seconds: 10) { model.config.clipboard?.enabled == true }
            record("clipboard on", clipboardOn, "config.clipboard.enabled=\(String(describing: model.config.clipboard?.enabled))")
            render(model, .clipboard, "05-clipboard", outputDirectory)

            // Never bind the real Synergy port in tests: host on loopback, on --host-port (default 24992).
            model.patch("server", ["port": .number(Double(hostPort)), "listen": .string("127.0.0.1:\(hostPort)")])
            let port = await waitUntil(seconds: 10) { model.config.server?.port == hostPort }
            record("test port", port, "server=\(model.config.server?.listen ?? "nil")")
            model.setRole(.server)
            let hosting = await waitUntil(seconds: 10) { model.status.role == .server && [.listening, .error].contains(model.state) }
            record("role server", hosting, "role=\(model.status.role?.rawValue ?? "nil") state=\(model.state.rawValue) listen=\(model.status.server?.listen ?? "nil")")
            render(model, .layout, "06-layout-hosting", outputDirectory)
            render(model, .security, "07-security", outputDirectory)

            model.setRole(.off)
            let offAgain = await waitUntil(seconds: 10) { model.status.role == .off }
            record("role off again", offAgain, "role=\(model.status.role?.rawValue ?? "nil")")

            finish(steps, outputDirectory)
        }
        NSApplication.shared.run()
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
        SnapshotRenderer.renderWindow(MainWindow(model: model, initialPane: pane).tint(Brand.accent),
                                      title: pane.title, appearance: .darkAqua,
                                      to: directory.appendingPathComponent("\(name).png"))
    }

    private static func finish(_ steps: [Step], _ directory: URL) {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        if let data = try? encoder.encode(steps) {
            try? data.write(to: directory.appendingPathComponent("report.json"))
        }
        let passed = !steps.isEmpty && steps.allSatisfy(\.passed)
        print(passed ? "E2E PASSED (\(steps.count) steps)" : "E2E FAILED")
        exit(passed ? 0 : 1)
    }
}
