import Foundation
import OSLog

/// Starts the bundled `openergy` daemon as a child process and restarts it if it crashes.
/// A child of the app shares the app's macOS privacy permissions, so the user grants
/// Accessibility and Input Monitoring once, to "Openergy".
@MainActor
final class DaemonSupervisor {
    private let log = Logger(subsystem: "dev.openergy.app", category: "daemon")
    private var process: Process?
    private var stopping = false
    private var restartDelay: Duration = .milliseconds(500)
    private var restartTask: Task<Void, Never>?

    /// Called after the supervisor (re)starts the daemon.
    var onSpawn: () -> Void = {}

    static var supportDirectory: URL {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        return base.appendingPathComponent("Openergy", isDirectory: true)
    }

    static var socketPath: String {
        if let override = ProcessInfo.processInfo.environment["OPENERGY_SOCKET"] { return override }
        return supportDirectory.appendingPathComponent("openergy.sock").path
    }

    /// The daemon binary: `Contents/Helpers/openergy` in the app bundle, or `OPENERGY_DAEMON`
    /// for development. (Not Contents/MacOS: on case-insensitive volumes `openergy` would
    /// collide with the app's own `Openergy` executable.)
    static var executableURL: URL? {
        if let override = ProcessInfo.processInfo.environment["OPENERGY_DAEMON"] {
            return URL(fileURLWithPath: override)
        }
        let bundled = Bundle.main.bundleURL.appendingPathComponent("Contents/Helpers/openergy")
        return FileManager.default.isExecutableFile(atPath: bundled.path) ? bundled : nil
    }

    var isRunning: Bool { process?.isRunning ?? false }

    func start() {
        guard process == nil else { return }
        guard let executable = Self.executableURL else {
            log.error("No openergy daemon binary found next to the app")
            return
        }
        stopping = false

        let process = Process()
        process.executableURL = executable
        // OPENERGY_DAEMON_ARGS adds flags for development and E2E, e.g. "--platform virtual".
        let extra = ProcessInfo.processInfo.environment["OPENERGY_DAEMON_ARGS"]?
            .split(separator: " ").map(String.init) ?? []
        process.arguments = ["daemon", "--parent-pid", String(getpid()), "--socket", Self.socketPath] + extra
        // The daemon refuses the standard Synergy port unless the real app asks for it,
        // so development and E2E runs can never collide with a live Synergy install.
        var environment = ProcessInfo.processInfo.environment
        if environment["OPENERGY_ALLOW_PORT_24800"] == nil {
            environment["OPENERGY_ALLOW_PORT_24800"] = "1"
        }
        process.environment = environment
        process.standardInput = FileHandle.nullDevice
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice
        process.terminationHandler = { [weak self] finished in
            let status = finished.terminationStatus
            let reason = finished.terminationReason
            Task { @MainActor in self?.handleExit(status: status, reason: reason) }
        }

        do {
            try FileManager.default.createDirectory(
                at: Self.supportDirectory,
                withIntermediateDirectories: true,
                attributes: [.posixPermissions: 0o700]
            )
            try process.run()
            self.process = process
            log.info("Started daemon pid \(process.processIdentifier)")
            onSpawn()
        } catch {
            log.error("Could not start daemon: \(error.localizedDescription)")
            scheduleRestart()
        }
    }

    /// Stops the child daemon. The daemon releases all keys and says goodbye to peers first.
    func stop() {
        stopping = true
        restartTask?.cancel()
        guard let process, process.isRunning else { return }
        process.terminate()
        let deadline = Date().addingTimeInterval(1.5)
        while process.isRunning, Date() < deadline {
            RunLoop.current.run(until: Date().addingTimeInterval(0.05))
        }
        if process.isRunning { kill(process.processIdentifier, SIGKILL) }
    }

    private func handleExit(status: Int32, reason: Process.TerminationReason) {
        process = nil
        guard !stopping else { return }
        log.error("Daemon exited (status \(status), reason \(reason.rawValue)); restarting")
        scheduleRestart()
    }

    private func scheduleRestart() {
        let delay = restartDelay
        restartDelay = min(restartDelay * 2, .seconds(30))
        restartTask?.cancel()
        restartTask = Task { @MainActor [weak self] in
            try? await Task.sleep(for: delay)
            guard let self, !Task.isCancelled, !self.stopping else { return }
            self.start()
        }
    }

    /// Call after a healthy connection so the next crash restarts quickly again.
    func markHealthy() {
        restartDelay = .milliseconds(500)
    }
}
