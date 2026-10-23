import AppKit
import Sparkle

/// State is written only by real Sparkle delegates. No synthetic callbacks or user driver.
@MainActor
final class UpdaterVerificationSession {
    static let stopDomain = "dev.openergy.updater.fixture-stop"
    var checks = 0
    var feedLoaded = false
    var noUpdate = false
    var candidateVersion: String?
    var candidateStopped = false
    var cycleFinished = false
    var cycleError: NSError?
    var startupError: NSError?
}

/// An early command entrypoint: never constructs AppModel, a daemon, or a SwiftUI scene.
/// The caller must supply a disposable bundle and preference home under the private
/// output directory. HTTP is permitted only for an explicit literal loopback fixture.
@MainActor
enum UpdaterEndToEnd {
    private enum Expected: String { case current, candidate, failure, unconfigured }

    private struct Result: Encodable {
        let schemaVersion = 1
        let expected: String
        let outcome: String
        let passed: Bool
        let started: Bool
        let checks: Int
        let feedLoaded: Bool
        let cycleFinished: Bool
        let candidateVersion: String?
        let candidateStoppedBeforeDownload: Bool
        let errorDomain: String?
        let errorCode: Int?
        let elapsedSeconds: Double
        let limit = "Feed parsing and candidate selection only. Signed archive download, installation and relaunch are not exercised."
    }

    static func run(arguments: [String]) -> Never {
        let usage = "usage: CFFIXED_USER_HOME=<private-config-dir> Openergy --updater-e2e <private-output-dir> --updater-config <private-config-dir> --updater-feed <loopback-feed-url> --updater-expect current|candidate|failure|unconfigured [--updater-timeout 15]\n"
        var options: [String: String] = [:]
        let allowed: Set<String> = ["--updater-e2e", "--updater-config", "--updater-feed", "--updater-expect", "--updater-timeout"]
        var index = 1
        while index < arguments.count {
            let flag = arguments[index]
            guard allowed.contains(flag), options[flag] == nil, index + 1 < arguments.count,
                  !arguments[index + 1].hasPrefix("--") else { fail(usage) }
            options[flag] = arguments[index + 1]
            index += 2
        }
        guard let output = options["--updater-e2e"], let config = options["--updater-config"],
              let feed = options["--updater-feed"], let expected = options["--updater-expect"].flatMap(Expected.init),
              let timeout = Double(options["--updater-timeout"] ?? "15"), timeout.isFinite, (1...30).contains(timeout),
              let fixture = URLComponents(string: feed), ["http", "https"].contains(fixture.scheme ?? ""),
              ["127.0.0.1", "::1", "[::1]"].contains(fixture.host ?? ""),
              fixture.user == nil, fixture.password == nil, fixture.fragment == nil,
              fixture.port.map({ (1...65535).contains($0) }) ?? true,
              let outputURL = privateDirectory(output), let configURL = privateDirectory(config),
              isInside(configURL, outputURL),
              let fixedHome = ProcessInfo.processInfo.environment["CFFIXED_USER_HOME"], fixedHome.hasPrefix("/"),
              URL(fileURLWithPath: fixedHome).standardizedFileURL.resolvingSymlinksInPath().path == configURL.path,
              URL(fileURLWithPath: NSHomeDirectory()).resolvingSymlinksInPath().path == configURL.path,
              isInside(Bundle.main.bundleURL.resolvingSymlinksInPath(), outputURL),
              let identifier = Bundle.main.bundleIdentifier,
              identifier.hasPrefix("dev.openergy.updater-e2e."),
              let defaultsDomain = Bundle.main.object(forInfoDictionaryKey: "SUDefaultsDomain") as? String,
              defaultsDomain.hasPrefix("/"),
              URL(fileURLWithPath: defaultsDomain).lastPathComponent == "sparkle-preferences",
              URL(fileURLWithPath: defaultsDomain).deletingLastPathComponent().standardizedFileURL.resolvingSymlinksInPath().path == configURL.path,
              !FileManager.default.fileExists(atPath: defaultsDomain + ".plist"),
              !FileManager.default.fileExists(atPath: outputURL.appendingPathComponent("updater-result.json").path)
        else { fail(usage + "Requires owned mode-0700 temporary directories, a fresh result path, a temporary dev.openergy.updater-e2e.* bundle inside output-dir, matching CFFIXED_USER_HOME, and SUDefaultsDomain=<resolved-config-dir>/sparkle-preferences.\n") }

        if expected == .unconfigured {
            // Exercise the same strict production configuration startup path. It must
            // reject the bundle before a controller, permission prompt or request exists.
            guard (try? AppUpdater.Configuration(bundle: .main, allowLoopbackFixture: false)) == nil else {
                fail("The unconfigured case requires a bundle rejected by production configuration validation.\n")
            }
        } else {
            guard let configuration = try? AppUpdater.Configuration(bundle: .main, allowLoopbackFixture: true),
                  configuration.feedURL.absoluteString == feed else {
                fail("The private bundle must contain the exact loopback fixture SUFeedURL and a valid SUPublicEDKey.\n")
            }
        }

        // Private command preferences only; never touch the production app's domain.
        // Prevent scheduled checks, profiling and permission UI while the real probe runs.
        UserDefaults.standard.setVolatileDomain([
            "SUEnableAutomaticChecks": false, "SUAutomaticallyUpdate": false, "SUSendProfileInfo": false,
        ], forName: UserDefaults.argumentDomain)
        NSApplication.shared.setActivationPolicy(.prohibited)
        let session = UpdaterVerificationSession()
        let updater = AppUpdater(verification: session)
        let began = ProcessInfo.processInfo.systemUptime
        updater.startForVerification(allowLoopbackFixture: expected != .unconfigured)
        if updater.started { updater.checkFixtureInformation() }

        // An absolute monotonic deadline bounds even a stalled server. Keep the main
        // runloop alive for Sparkle's actual NSURLSession/parser/delegate machinery.
        while ProcessInfo.processInfo.systemUptime - began < timeout {
            if session.cycleFinished { break }
            if !updater.started, ProcessInfo.processInfo.systemUptime - began >= 0.5 { break }
            RunLoop.current.run(until: Date().addingTimeInterval(0.02))
        }

        let outcome: String
        if !updater.started { outcome = "unconfigured-no-request" }
        else if !session.cycleFinished { outcome = "deadline-exceeded" }
        else if session.candidateStopped,
                session.cycleError?.domain == UpdaterVerificationSession.stopDomain {
            outcome = "candidate-observed-installation-not-exercised"
        } else if session.noUpdate { outcome = "no-update" }
        else if session.cycleError != nil { outcome = "feed-failure" }
        else { outcome = "unexpected-cycle-result" }

        let passed: Bool
        switch expected {
        case .unconfigured: passed = outcome == "unconfigured-no-request" && session.checks == 0
        case .current: passed = outcome == "no-update" && session.feedLoaded && session.checks == 1
        case .candidate: passed = outcome == "candidate-observed-installation-not-exercised" && session.feedLoaded && session.checks == 1
        case .failure: passed = outcome == "feed-failure" && session.checks == 1
        }
        let error = session.cycleError ?? session.startupError
        let result = Result(expected: expected.rawValue, outcome: outcome, passed: passed, started: updater.started,
                            checks: session.checks, feedLoaded: session.feedLoaded, cycleFinished: session.cycleFinished,
                            candidateVersion: session.candidateVersion, candidateStoppedBeforeDownload: session.candidateStopped,
                            errorDomain: error?.domain, errorCode: error?.code,
                            elapsedSeconds: ProcessInfo.processInfo.systemUptime - began)
        UserDefaults(suiteName: defaultsDomain)?.synchronize()
        do {
            let encoder = JSONEncoder()
            encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
            try encoder.encode(result).write(to: outputURL.appendingPathComponent("updater-result.json"), options: .atomic)
        } catch { fail("Could not write updater result.\n", code: 1) }
        print("\(passed ? "PASS" : "FAIL") \(outcome)")
        withExtendedLifetime(updater) { exit(passed ? 0 : 1) }
    }

    private static func privateDirectory(_ path: String) -> URL? {
        guard path.hasPrefix("/") else { return nil }
        let url = URL(fileURLWithPath: path).standardizedFileURL.resolvingSymlinksInPath()
        let temporaryRoots = [URL(fileURLWithPath: "/private/tmp").resolvingSymlinksInPath(), FileManager.default.temporaryDirectory.resolvingSymlinksInPath()]
        guard temporaryRoots.contains(where: { isInside(url, $0) }),
              let attributes = try? FileManager.default.attributesOfItem(atPath: url.path),
              attributes[.type] as? FileAttributeType == .typeDirectory,
              (attributes[.ownerAccountID] as? NSNumber)?.uint32Value == getuid(),
              ((attributes[.posixPermissions] as? NSNumber)?.intValue ?? 0) & 0o777 == 0o700 else { return nil }
        return url
    }

    private static func isInside(_ child: URL, _ parent: URL) -> Bool {
        child.path.hasPrefix(parent.path.hasSuffix("/") ? parent.path : parent.path + "/")
    }

    private static func fail(_ message: String, code: Int32 = 64) -> Never {
        FileHandle.standardError.write(Data(message.utf8))
        exit(code)
    }
}
