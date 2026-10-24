import AppKit
import Darwin
import Foundation
import MachO

/// A bundle-carried, fail-closed entry point for an explicitly prepared private
/// running-target fixture. This never checks for, downloads, or installs updates.
/// See docs/verification/private-update-bootstrap-2026-10-23/README.md.
enum PrivateUpdateFixture {
    private static let metadataKey = "OpenergyPrivateUpdateFixture"
    private static let idPrefix = "dev.openergy.updater-fixture."
    private static let internalArguments = ["--private-update-fixture", "--e2e"]

    private struct Refusal: Error { let reason: String }
    private static func require(_ condition: Bool, _ reason: String) throws {
        if !condition { throw Refusal(reason: reason) }
    }

    /// Called before all argument dispatch, singletons, and SwiftUI scenes.
    static func dispatchIfPresent() {
        let info = Bundle.main.infoDictionary ?? [:]
        let identifier = info["CFBundleIdentifier"] as? String ?? ""
        guard let raw = info[metadataKey] else {
            // Removing the metadata from a fixture must not enable normal startup.
            if identifier.hasPrefix(idPrefix) { refuse("missing metadata") }
            return
        }
        do {
            let context = try Context(raw: raw, info: info, identifier: identifier)
            let arguments = Array(ProcessInfo.processInfo.arguments.dropFirst())
            try require(arguments.isEmpty || arguments == internalArguments, "unexpected arguments")
            try context.installEnvironment()
            if arguments.isEmpty {
                // AppModel's existing --e2e guard skips all login integration.
                // Re-exec also puts the private home in place before CF/AppKit load.
                // Activation still requires metadata; these arguments cannot enable it.
                let strings = ([context.executable] + internalArguments).map { strdup($0) }
                var argv = strings + [nil]
                _ = argv.withUnsafeMutableBufferPointer { execv(context.executable, $0.baseAddress!) }
                strings.forEach { free($0) }
                throw Refusal(reason: "private re-exec failed")
            }
            let lock = try context.lockLaunch()
            defer { Darwin.close(lock) }
            MainActor.assumeIsolated { run(context) }
            exit(0)
        } catch let error as Refusal {
            refuse(error.reason)
        } catch {
            refuse("private prerequisite failed")
        }
    }

    private static func refuse(_ reason: String) -> Never {
        FileHandle.standardError.write(Data("Private update fixture refused: \(reason).\n".utf8))
        exit(78)
    }

    private struct Marker: Decodable {
        let schema: Int
        let fixtureID: String
        let root: String
        let nonce: String
        let port: Int
        let builds: [String]
        let createdAt: Double
        let expiresAt: Double
    }

    private struct Context {
        let root: String
        let identifier: String
        let nonce: String
        let port: Int
        let build: String
        let bundle: String
        let executable: String
        let daemon: String
        var state: String { root + "/state" }
        var config: String { state + "/config.toml" }
        var tls: String { state + "/tls" }
        var socket: String { state + "/control.sock" }
        var receipts: String { root + "/receipts" }

        init(raw: Any, info: [String: Any], identifier: String) throws {
            try require(getuid() == geteuid() && getuid() != 0, "unexpected user")
            try require(identifier.hasPrefix(idPrefix), "fixture bundle identifier required")
            let suffix = String(identifier.dropFirst(idPrefix.count))
            try require(Self.isUUID(suffix), "unique UUID identifier required")
            guard let metadata = raw as? [String: Any],
                  Set(metadata.keys) == Set(["Schema", "Root", "Nonce", "Port"]),
                  let schema = metadata["Schema"] as? NSNumber,
                  String(cString: schema.objCType) != "c", schema.stringValue == "1",
                  let root = metadata["Root"] as? String,
                  let nonce = metadata["Nonce"] as? String,
                  let number = metadata["Port"] as? NSNumber,
                  String(cString: number.objCType) != "c",
                  number.doubleValue == Double(number.intValue),
                  let build = info["CFBundleVersion"] as? String else {
                throw Refusal(reason: "invalid metadata schema")
            }
            try require(Self.isUUID(nonce) && nonce != suffix, "independent nonce required")
            try require((25000...65535).contains(number.intValue), "private port required")
            try require(Self.isBuild(build), "invalid bundle build")
            try require(root.hasPrefix("/") && !root.utf8.contains(0) && root.utf8.count < 512,
                        "absolute private root required")
            try require(root == URL(fileURLWithPath: root).standardizedFileURL.path,
                        "noncanonical root")
            let parent = URL(fileURLWithPath: root).deletingLastPathComponent().path
            // Do not trust TMPDIR supplied by the launching process.
            var buffer = [CChar](repeating: 0, count: Int(PATH_MAX))
            let length = confstr(_CS_DARWIN_USER_TEMP_DIR, &buffer, buffer.count)
            try require(length > 0 && length <= buffer.count, "user temp root unavailable")
            let userTemp = try Self.realPath(String(cString: buffer))
            try require(parent == "/private/tmp" || parent == userTemp, "temporary root required")
            try require(URL(fileURLWithPath: root).lastPathComponent == "oe-updater-fixture-" + suffix,
                        "root must match unique identifier")
            try Self.directory(root)
            self.root = root
            self.identifier = identifier
            self.nonce = nonce
            self.port = number.intValue
            self.build = build
            self.bundle = root + "/target/Openergy.app"
            self.executable = bundle + "/Contents/MacOS/Openergy"
            self.daemon = bundle + "/Contents/Helpers/openergy"

            let markerPath = root + "/fixture.json"
            let marker = try JSONDecoder().decode(Marker.self, from: Self.readPrivate(markerPath, limit: 4096))
            let now = Date().timeIntervalSince1970
            try require(marker.schema == 1 && marker.fixtureID == identifier && marker.root == root
                        && marker.nonce == nonce && marker.port == port, "marker mismatch")
            try require(marker.createdAt.isFinite && marker.expiresAt.isFinite
                        && marker.createdAt <= now && marker.expiresAt > now
                        && marker.expiresAt > marker.createdAt
                        && marker.expiresAt - marker.createdAt <= 21_600, "marker expired or invalid")
            try require(marker.builds.count == 2 && Set(marker.builds).count == 2
                        && marker.builds.allSatisfy(Self.isBuild) && marker.builds.contains(build),
                        "build not allowed by marker")
            for path in [root + "/target", state, tls, root + "/home", root + "/tmp", receipts] {
                try Self.directory(path)
            }
            try require(try Self.realPath(Bundle.main.bundleURL.path) == bundle, "unexpected bundle path")
            try require(info["CFBundleExecutable"] as? String == "Openergy", "unexpected executable name")
            try require(try Self.realPath(Bundle.main.executableURL?.path ?? "") == executable,
                        "bundle executable mismatch")
            var binaryPath = [CChar](repeating: 0, count: Int(PATH_MAX))
            var size = UInt32(binaryPath.count)
            try require(_NSGetExecutablePath(&binaryPath, &size) == 0, "process executable unavailable")
            try require(try Self.realPath(String(cString: binaryPath)) == executable,
                        "process executable mismatch")
            try Self.executableFile(executable)
            try Self.executableFile(daemon)
            try Self.bundleTree(bundle)
            // No old daemon may be attached, even if its socket is private.
            try Self.absent(socket)
            try require(socket.utf8.count < 104, "private socket path too long")
            for path in [state, root + "/home", root + "/tmp", receipts] {
                try Self.privateTree(path)
            }
            try require(try Self.readPrivate(config, limit: 4096) == Data(Self.safeConfig(port: port, nonce: nonce).utf8),
                        "private config does not match fixed safe config")
            _ = try Self.readPrivate(root + "/launch.lock", limit: 0)
        }

        private static func isUUID(_ value: String) -> Bool {
            UUID(uuidString: value)?.uuidString.lowercased() == value
        }

        private static func isBuild(_ value: String) -> Bool {
            (1...9).contains(value.utf8.count) && value.utf8.allSatisfy { (48...57).contains($0) }
                && value.first != "0"
        }

        private static func realPath(_ path: String) throws -> String {
            guard !path.isEmpty, !path.utf8.contains(0), let pointer = realpath(path, nil) else {
                throw Refusal(reason: "path cannot be resolved")
            }
            defer { free(pointer) }
            return String(cString: pointer)
        }

        private static func attributes(_ path: String) throws -> stat {
            var value = stat()
            try require(lstat(path, &value) == 0 && value.st_uid == getuid(), "path owner or type invalid")
            return value
        }

        private static func directory(_ path: String) throws {
            let value = try attributes(path)
            try require(value.st_mode & S_IFMT == S_IFDIR && value.st_mode & 0o7777 == 0o700,
                        "private directory must be mode 0700")
            try require(try realPath(path) == path, "directory symlink forbidden")
        }

        private static func executableFile(_ path: String) throws {
            let value = try attributes(path)
            try require(value.st_mode & S_IFMT == S_IFREG && value.st_nlink == 1
                        && value.st_mode & 0o7022 == 0 && value.st_mode & 0o100 != 0,
                        "unsafe executable")
            try require(try realPath(path) == path, "executable symlink forbidden")
        }

        private static func absent(_ path: String) throws {
            var value = stat()
            let result = lstat(path, &value)
            try require(result == -1 && errno == ENOENT, "private socket must be absent")
        }

        private static func readPrivate(_ path: String, limit: Int) throws -> Data {
            try require(try realPath(path) == path, "private file symlink forbidden")
            let fd = open(path, O_RDONLY | O_NOFOLLOW | O_CLOEXEC | O_NONBLOCK)
            try require(fd >= 0, "private file unavailable")
            defer { Darwin.close(fd) }
            var value = stat()
            try require(fstat(fd, &value) == 0 && value.st_uid == getuid() && value.st_nlink == 1
                        && value.st_mode & S_IFMT == S_IFREG && value.st_mode & 0o7777 == 0o600
                        && value.st_size >= 0 && value.st_size <= limit, "unsafe private file")
            var bytes = [UInt8](repeating: 0, count: limit + 1)
            var offset = 0
            while offset < bytes.count {
                let count = bytes.withUnsafeMutableBytes { Darwin.read(fd, $0.baseAddress! + offset, $0.count - offset) }
                if count < 0 && errno == EINTR { continue }
                try require(count >= 0, "private file read failed")
                if count == 0 { break }
                offset += count
            }
            try require(offset <= limit && offset == value.st_size, "private file changed or oversized")
            return Data(bytes.prefix(offset))
        }

        private static func privateTree(_ path: String) throws {
            try walk(path) { child in
                let value = try attributes(child)
                if value.st_mode & S_IFMT == S_IFDIR { try directory(child) }
                else {
                    try require(value.st_mode & S_IFMT == S_IFREG && value.st_nlink == 1
                                && value.st_mode & 0o7777 == 0o600, "unsafe writable-tree entry")
                    try require(try realPath(child) == child, "writable-tree symlink forbidden")
                }
            }
        }

        private static func bundleTree(_ path: String) throws {
            try walk(path) { child in
                let value = try attributes(child)
                let type = value.st_mode & S_IFMT
                try require([S_IFDIR, S_IFREG, S_IFLNK].contains(type), "unsafe bundle entry")
                // Darwin reports symlinks as mode 0777. Their resolved target,
                // rather than those synthetic mode bits, must stay in this bundle.
                if type != S_IFLNK { try require(value.st_mode & 0o7022 == 0, "writable bundle entry") }
                if type == S_IFREG { try require(value.st_nlink == 1, "bundle hard link forbidden") }
                let resolved = try realPath(child)
                try require(resolved == path || resolved.hasPrefix(path + "/"), "bundle symlink escapes")
            }
        }

        private static func walk(_ path: String, check: (String) throws -> Void) throws {
            try check(path)
            var failed = false
            guard let iterator = FileManager.default.enumerator(at: URL(fileURLWithPath: path),
                includingPropertiesForKeys: nil, options: [], errorHandler: { _, _ in failed = true; return false }) else {
                throw Refusal(reason: "tree unavailable")
            }
            var count = 0
            for case let child as URL in iterator {
                count += 1
                try require(count <= 8192 && child.path.utf8.count < 2048, "tree exceeds fixture bound")
                try check(child.path)
            }
            try require(!failed, "tree enumeration failed")
        }

        // Byte-for-byte contract, including the final LF. No arbitrary TOML is loaded.
        static func safeConfig(port: Int, nonce: String) -> String {
            """
            version = 1
            role = "off"
            name = "updater-fixture-\(nonce)"
            [server]
            listen = "127.0.0.1:\(port)"
            port = \(port)
            [client]
            remote = ""
            autoconnect = false
            [tls]
            enabled = false
            [clipboard]
            enabled = false

            """
        }

        func installEnvironment() throws {
            // Remove inherited development overrides. None of these can activate a fixture.
            for name in ProcessInfo.processInfo.environment.keys where name.hasPrefix("OPENERGY_") {
                try require(unsetenv(name) == 0, "could not clear inherited override")
            }
            let values = [
                "OPENERGY_CONFIG": config, "OPENERGY_CONFIG_DIR": state, "OPENERGY_TLS_DIR": tls,
                "OPENERGY_SOCKET": socket, "OPENERGY_DAEMON": daemon,
                "OPENERGY_DAEMON_ARGS": "--platform virtual", "OPENERGY_DISABLE_DISCOVERY": "1",
                "OPENERGY_ALLOW_PORT_24800": "0", "HOME": root + "/home",
                "CFFIXED_USER_HOME": root + "/home", "TMPDIR": root + "/tmp/",
                "XDG_CONFIG_HOME": state, "XDG_CACHE_HOME": root + "/home",
            ]
            for (name, value) in values { try require(setenv(name, value, 1) == 0, "could not set private environment") }
            umask(0o077)
            try require(chdir(root) == 0, "could not enter private root")
            errno = 0
            let originalNice = getpriority(PRIO_PROCESS, 0)
            try require(errno == 0 && originalNice <= 15, "nice 15 would require raising priority")
            try require(setpriority(PRIO_PROCESS, 0, 15) == 0 && getpriority(PRIO_PROCESS, 0) == 15,
                        "could not set absolute nice 15")
        }

        func lockLaunch() throws -> Int32 {
            let fd = open(root + "/launch.lock", O_RDWR | O_NOFOLLOW | O_CLOEXEC | O_NONBLOCK)
            try require(fd >= 0, "launch lock unavailable")
            var value = stat()
            guard fstat(fd, &value) == 0 && value.st_uid == getuid() && value.st_nlink == 1
                    && value.st_mode & S_IFMT == S_IFREG && value.st_mode & 0o7777 == 0o600
                    && flock(fd, LOCK_EX | LOCK_NB) == 0 else {
                Darwin.close(fd)
                throw Refusal(reason: "launch lock unsafe or already owned")
            }
            return fd
        }

        func writeReceipt(_ receipt: Receipt) throws {
            let bytes = try JSONEncoder().encode(receipt)
            try require(bytes.count <= 4096, "receipt exceeds bound")
            try Self.directory(receipts)
            let directory = open(receipts, O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC)
            try require(directory >= 0, "receipt directory unavailable")
            defer { Darwin.close(directory) }
            let name = "\(build)-\(getpid())-\(receipt.launchID)-\(receipt.event).json"
            let fd = openat(directory, name, O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC, 0o600)
            try require(fd >= 0, "receipt already exists or unavailable")
            defer { Darwin.close(fd) }
            var offset = 0
            while offset < bytes.count {
                let count = bytes.withUnsafeBytes { Darwin.write(fd, $0.baseAddress! + offset, $0.count - offset) }
                if count < 0 && errno == EINTR { continue }
                try require(count > 0, "receipt write failed")
                offset += count
            }
            try require(fsync(fd) == 0, "receipt sync failed")
        }
    }

    private struct Receipt: Encodable {
        let schema = 1
        let event: String
        let fixtureID: String
        let nonce: String
        let launchID: String
        let bundleBuild: String
        let bundlePath: String
        let executablePath: String
        let daemonPath: String
        let configPath: String
        let tlsPath: String
        let socketPath: String
        let appPID: Int32
        let appNice: Int32
        let ownedDaemonPID: Int32?
        let privateIPCReady: Bool
        let shutdownReturned: Bool
        let time: Double
    }

    @MainActor
    private final class Lifecycle: NSObject, NSApplicationDelegate {
        let context: Context
        let model = AppModel(demo: nil)
        let launchID = UUID().uuidString.lowercased()
        var readiness: Task<Void, Never>?
        var ready = false
        var lastDaemonPID: Int32?
        var failed = false
        var finished = false

        init(_ context: Context) { self.context = context }

        func applicationDidFinishLaunching(_ notification: Notification) {
            model.start()
            readiness = Task { @MainActor [self] in
                let clock = ContinuousClock()
                let deadline = clock.now.advanced(by: .seconds(15))
                while !Task.isCancelled && clock.now < deadline {
                    if let pid = model.supervisedDaemonPID { lastDaemonPID = pid }
                    if model.isReady {
                        guard let pid = model.supervisedDaemonPID, model.status.daemon?.pid == pid,
                              model.status.daemon?.platform == "virtual", model.status.role == .off,
                              model.status.name == "updater-fixture-\(context.nonce)",
                              model.config.name == "updater-fixture-\(context.nonce)",
                              model.config.role == .off, model.config.client?.remote == "",
                              model.config.client?.autoconnect == false, model.config.tls?.enabled == false,
                              model.config.clipboard?.enabled == false, model.config.server?.port == context.port,
                              model.config.server?.listen == "127.0.0.1:\(context.port)" else {
                            fail(); return
                        }
                        do { try context.writeReceipt(receipt("ready", shutdown: false)) }
                        catch { fail(); return }
                        ready = true
                        return
                    }
                    try? await Task.sleep(for: .milliseconds(50))
                }
                if !Task.isCancelled { fail() }
            }
        }

        private func fail() {
            failed = true
            NSApplication.shared.terminate(nil)
        }

        func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply { .terminateNow }

        func applicationWillTerminate(_ notification: Notification) {
            guard !finished else { return }
            finished = true
            failed = failed || !ready
            readiness?.cancel()
            lastDaemonPID = model.supervisedDaemonPID ?? lastDaemonPID
            model.shutdown()
            // No child-exit claim: stop() can send SIGKILL without joining.
            // The external controller must observe PID/start identity/path itself.
            do { try context.writeReceipt(receipt(failed ? "failure" : "exit", shutdown: true)) }
            catch { refuse("exit receipt failed") }
            if failed { refuse("private lifecycle did not become ready") }
        }

        private func receipt(_ event: String, shutdown: Bool) -> Receipt {
            Receipt(event: event, fixtureID: context.identifier, nonce: context.nonce, launchID: launchID,
                    bundleBuild: context.build, bundlePath: context.bundle, executablePath: context.executable,
                    daemonPath: context.daemon, configPath: context.config, tlsPath: context.tls,
                    socketPath: context.socket, appPID: getpid(), appNice: getpriority(PRIO_PROCESS, 0),
                    ownedDaemonPID: lastDaemonPID,
                    privateIPCReady: event == "ready", shutdownReturned: shutdown,
                    time: Date().timeIntervalSince1970)
        }
    }

    @MainActor
    private static func run(_ context: Context) {
        // Internal arguments reuse the existing production integration guard only.
        // No AppEndToEnd, AppUpdater, SwiftUI scene, or login command is invoked.
        let application = NSApplication.shared
        guard application.setActivationPolicy(.prohibited) else { refuse("headless activation policy failed") }
        let lifecycle = Lifecycle(context)
        guard lifecycle.model.isEndToEnd && !lifecycle.model.allowsSystemIntegration else {
            refuse("system integration guard failed")
        }
        application.delegate = lifecycle
        withExtendedLifetime(lifecycle) { application.run() }
        // NSApplication normally exits after applicationWillTerminate. A bare
        // stop/run-loop return is not a successful lifecycle or a relaunch proof.
        if !lifecycle.finished {
            lifecycle.failed = true
            lifecycle.applicationWillTerminate(Notification(name: NSApplication.willTerminateNotification))
        }
    }
}
