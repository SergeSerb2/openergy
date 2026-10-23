import AppKit
import Observation
import Sparkle

/// One standard Sparkle user driver for the normal app lifetime. Construction is inert:
/// only AppDelegate's normal-launch callback starts it, never a view or `.shared` access.
@MainActor
@Observable
final class AppUpdater: NSObject, SPUUpdaterDelegate {
    static let shared = AppUpdater()

    private(set) var started = false
    private(set) var canCheckForUpdates = false
    private(set) var automaticallyChecksForUpdates = false
    private(set) var automaticallyDownloadsUpdates = false
    private(set) var allowsAutomaticUpdates = false
    private(set) var status = "Updates have not started."

    @ObservationIgnored private var controller: SPUStandardUpdaterController?
    @ObservationIgnored private var observations: [NSKeyValueObservation] = []
    @ObservationIgnored private var configuration: Configuration?
    @ObservationIgnored private var attemptedStart = false
    @ObservationIgnored private let verification: UpdaterVerificationSession?

    init(verification: UpdaterVerificationSession? = nil) {
        self.verification = verification
        super.init()
    }

    func startAfterNormalLaunch() {
        let arguments = ProcessInfo.processInfo.arguments
        let excluded = ["--snapshot", "--e2e", "--updater-e2e", "--open-at-login", "--demo", "--offscreen"]
        guard !excluded.contains(where: arguments.contains),
              ProcessInfo.processInfo.environment["OPENERGY_DEMO"] == nil else {
            status = "Updates are disabled in this mode."
            return
        }
        startConfiguredUpdater(allowLoopbackFixture: false)
    }

    /// Available only to the early, isolated command entrypoint. Uses the same controller,
    /// configuration checks, observations and delegates as normal startup.
    func startForVerification(allowLoopbackFixture: Bool) {
        guard verification != nil else { return }
        startConfiguredUpdater(allowLoopbackFixture: allowLoopbackFixture)
    }

    private func startConfiguredUpdater(allowLoopbackFixture: Bool) {
        guard !attemptedStart else { return }
        attemptedStart = true
        do {
            configuration = try Configuration(bundle: .main, allowLoopbackFixture: allowLoopbackFixture)
            let controller = SPUStandardUpdaterController(startingUpdater: false, updaterDelegate: self, userDriverDelegate: nil)
            self.controller = controller
            let updater = controller.updater
            // Sparkle documents these properties as main-thread-only and KVO compliant.
            // Read all values together rather than retaining an independent UserDefaults copy.
            observations = [
                updater.observe(\.canCheckForUpdates, options: [.new]) { [weak self] _, _ in
                    MainActor.assumeIsolated { self?.refreshSettings() }
                },
                updater.observe(\.sessionInProgress, options: [.new]) { [weak self] _, _ in
                    MainActor.assumeIsolated { self?.refreshSettings() }
                },
                updater.observe(\.automaticallyChecksForUpdates, options: [.new]) { [weak self] _, _ in
                    MainActor.assumeIsolated { self?.refreshSettings() }
                },
                updater.observe(\.automaticallyDownloadsUpdates, options: [.new]) { [weak self] _, _ in
                    MainActor.assumeIsolated { self?.refreshSettings() }
                },
                updater.observe(\.allowsAutomaticUpdates, options: [.new]) { [weak self] _, _ in
                    MainActor.assumeIsolated { self?.refreshSettings() }
                },
            ]
            // Calling the throwing core API retains the standard controller/user driver,
            // while reporting configuration failures inline instead of showing error dialogs.
            try updater.start()
            started = true
            status = "Sparkle checks for updates using your preferences."
            refreshSettings()
        } catch {
            status = "Updates unavailable. \(error.localizedDescription)"
            verification?.startupError = (error as NSError)
            observations.removeAll()
            controller = nil
            configuration = nil
            refreshSettings()
        }
    }

    func checkForUpdates() {
        guard started, let controller, controller.updater.canCheckForUpdates else { return }
        controller.checkForUpdates(nil)
        refreshSettings()
    }

    func setAutomaticallyChecksForUpdates(_ enabled: Bool) {
        guard started, let updater = controller?.updater else { return }
        updater.automaticallyChecksForUpdates = enabled
        refreshSettings()
    }

    func setAutomaticallyDownloadsUpdates(_ enabled: Bool) {
        guard started, let updater = controller?.updater, updater.allowsAutomaticUpdates else { return }
        updater.automaticallyDownloadsUpdates = enabled
        refreshSettings()
    }

    func checkFixtureInformation() {
        guard verification != nil, started, let updater = controller?.updater,
              !updater.sessionInProgress else { return }
        // Sparkle's documented probe still fetches/parses the feed and calls the real
        // delegates, but never offers an installation or shows the standard update UI.
        updater.checkForUpdateInformation()
    }

    private func refreshSettings() {
        guard started, let updater = controller?.updater else {
            canCheckForUpdates = false
            automaticallyChecksForUpdates = false
            automaticallyDownloadsUpdates = false
            allowsAutomaticUpdates = false
            return
        }
        canCheckForUpdates = updater.canCheckForUpdates
        automaticallyChecksForUpdates = updater.automaticallyChecksForUpdates
        automaticallyDownloadsUpdates = updater.automaticallyDownloadsUpdates
        allowsAutomaticUpdates = updater.allowsAutomaticUpdates
    }

    // Always return the validated bundle URL. Sparkle's legacy SUFeedURL preference
    // must not override the application's baked feed, including in development.
    func feedURLString(for updater: SPUUpdater) -> String? { configuration?.feedURL.absoluteString }

    func updater(_ updater: SPUUpdater, mayPerform updateCheck: SPUUpdateCheck) throws {
        guard configuration != nil else { throw Configuration.failure("This build has no update configuration.") }
        if let verification {
            guard updateCheck == .updateInformation else {
                throw Configuration.failure("Only an information check is allowed in updater verification.")
            }
            verification.checks += 1
        }
    }

    func updaterShouldPromptForPermissionToCheck(forUpdates updater: SPUUpdater) -> Bool { verification == nil }

    func updater(_ updater: SPUUpdater, didFinishLoading appcast: SUAppcast) {
        verification?.feedLoaded = true
    }

    func updater(_ updater: SPUUpdater, shouldProceedWithUpdate item: SUAppcastItem, updateCheck: SPUUpdateCheck) throws {
        guard let verification else { return }
        verification.candidateVersion = String(item.versionString.prefix(256))
        verification.candidateStopped = true
        // Documented veto: Sparkle neither presents, downloads nor installs this item.
        throw NSError(domain: UpdaterVerificationSession.stopDomain, code: 1, userInfo: [
            NSLocalizedDescriptionKey: "Candidate observed; installation not exercised.",
        ])
    }

    func updaterDidNotFindUpdate(_ updater: SPUUpdater, error: Error) {
        verification?.noUpdate = true
    }

    func updater(_ updater: SPUUpdater, didFinishUpdateCycleFor updateCheck: SPUUpdateCheck, error: Error?) {
        verification?.cycleError = error.map { $0 as NSError }
        verification?.cycleFinished = true
        refreshSettings()
    }

    struct Configuration {
        let feedURL: URL

        init(bundle: Bundle, allowLoopbackFixture: Bool) throws {
            guard let feed = bundle.object(forInfoDictionaryKey: "SUFeedURL") as? String,
                  !feed.isEmpty,
                  feed.rangeOfCharacter(from: .whitespacesAndNewlines.union(.controlCharacters)) == nil,
                  let url = URL(string: feed, encodingInvalidCharacters: false),
                  let parts = URLComponents(url: url, resolvingAgainstBaseURL: false),
                  let host = parts.host, !host.isEmpty,
                  parts.user == nil, parts.password == nil, parts.fragment == nil,
                  parts.port.map({ (1...65535).contains($0) }) ?? true else {
                throw Self.failure("This build needs a valid HTTPS update feed and an Ed25519 public key.")
            }
            let loopback = ["127.0.0.1", "::1", "[::1]"].contains(host)
            guard parts.scheme?.lowercased() == "https"
                    || (allowLoopbackFixture && parts.scheme == "http" && loopback) else {
                throw Self.failure("The update feed must use HTTPS.")
            }
            guard let key = bundle.object(forInfoDictionaryKey: "SUPublicEDKey") as? String,
                  let decoded = Data(base64Encoded: key), decoded.count == 32,
                  decoded.base64EncodedString() == key else {
                throw Self.failure("This build needs a valid 32-byte Ed25519 update public key.")
            }
            feedURL = url
        }

        static func failure(_ message: String) -> NSError {
            NSError(domain: "dev.openergy.updater.configuration", code: 1, userInfo: [NSLocalizedDescriptionKey: message])
        }
    }
}
