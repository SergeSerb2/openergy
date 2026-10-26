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
        if let config = ProcessInfo.processInfo.environment["OPENERGY_CONFIG"] {
            return URL(fileURLWithPath: config).deletingLastPathComponent()
        }
        if let directory = ProcessInfo.processInfo.environment["OPENERGY_CONFIG_DIR"] {
            return URL(fileURLWithPath: directory, isDirectory: true)
        }
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        return base.appendingPathComponent("Openergy", isDirectory: true)
    }

    static var configURL: URL {
        supportDirectory.appendingPathComponent(
            ProcessInfo.processInfo.environment["OPENERGY_CONFIG"].map { URL(fileURLWithPath: $0).lastPathComponent }
                ?? "config.toml")
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
    var processIdentifier: Int32? { isRunning ? process?.processIdentifier : nil }
    /// After the first start, only the supervisor restarts the daemon (with its own backoff).
    private(set) var hasStarted = false

    func start() {
        guard process == nil, restartTask == nil else { return }
        guard let executable = Self.executableURL else {
            log.error("No openergy daemon binary found next to the app")
            return
        }
        stopping = false
        hasStarted = true

        let process = Process()
        process.executableURL = executable
        // OPENERGY_DAEMON_ARGS adds flags for development and E2E, e.g. "--platform virtual".
        let extra = ProcessInfo.processInfo.environment["OPENERGY_DAEMON_ARGS"]?
            .split(separator: " ").map(String.init) ?? []
        process.arguments = ["daemon", "--parent-pid", String(getpid()), "--socket", Self.socketPath,
                             "--config", Self.configURL.path] + extra
        // Native daemon startup owns the default-port policy. Preserve explicit
        // overrides and leave the guard active for virtual development runs.
        process.environment = ProcessInfo.processInfo.environment
        process.standardInput = FileHandle.nullDevice
        process.standardOutput = FileHandle.nullDevice
        // Keep the daemon's stderr (startup failures, aborts) next to its own log files.
        process.terminationHandler = { [weak self] finished in
            let status = finished.terminationStatus
            let reason = finished.terminationReason
            Task { @MainActor in self?.handleExit(process: finished, status: status, reason: reason) }
        }

        do {
            try FileManager.default.createDirectory(
                at: Self.supportDirectory,
                withIntermediateDirectories: true,
                attributes: [.posixPermissions: 0o700]
            )
            let stderr = Self.stderrLog()
            defer { try? stderr?.close() }
            process.standardError = stderr ?? FileHandle.nullDevice
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
        restartTask = nil
        hasStarted = false
        guard let process else { return }
        guard process.isRunning else { self.process = nil; return }
        process.terminate()
        let deadline = Date().addingTimeInterval(1.5)
        while process.isRunning, Date() < deadline {
            RunLoop.current.run(until: Date().addingTimeInterval(0.05))
        }
        if process.isRunning { kill(process.processIdentifier, SIGKILL) }
        // The identity check in handleExit prevents a queued callback from clearing a new child.
        self.process = nil
    }

    private static func stderrLog() -> FileHandle? {
        let url = supportDirectory.appendingPathComponent("daemon-stderr.log")
        if !FileManager.default.fileExists(atPath: url.path) {
            FileManager.default.createFile(atPath: url.path, contents: nil, attributes: [.posixPermissions: 0o600])
        }
        // Start fresh when the file grows past 1 MB.
        if let size = try? FileManager.default.attributesOfItem(atPath: url.path)[.size] as? Int, size > 1_000_000 {
            try? Data().write(to: url)
        }
        guard let handle = try? FileHandle(forWritingTo: url) else { return nil }
        handle.seekToEndOfFile()
        return handle
    }

    private func handleExit(process finished: Process, status: Int32, reason: Process.TerminationReason) {
        guard process === finished else { return }
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
            self.restartTask = nil
            self.start()
        }
    }

    /// Call after a healthy connection so the next crash restarts quickly again.
    func markHealthy() {
        restartDelay = .milliseconds(500)
    }
}
