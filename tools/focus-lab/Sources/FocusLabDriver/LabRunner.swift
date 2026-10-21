import AppKit
import ApplicationServices
import FocusLabSupport

// The handler only sets a sig_atomic_t. Cleanup runs synchronously on main.
nonisolated(unsafe) private var interrupted: sig_atomic_t = 0

@MainActor
private func waitUntil(_ description: String, timeout: TimeInterval = 4, condition: () throws -> Bool) throws {
    let deadline = Date().addingTimeInterval(timeout)
    repeat {
        if interrupted != 0 { throw LabError("Interrupted; cleaning up lab-owned processes and pointer") }
        if try condition() { return }
        RunLoop.current.run(until: Date().addingTimeInterval(0.02))
    } while Date() < deadline
    throw LabError("Timed out: \(description)")
}

@MainActor
private final class FixtureProcess {
    let name: String
    let process = Process()
    let logURL: URL
    let commandURL: URL
    private var output: FileHandle?
    var records: [[String: Any]] { JSONFile.lines(logURL) }

    init(name: String, binary: URL, evidence: URL, arguments: [String]) throws {
        self.name = name
        logURL = evidence.appendingPathComponent("\(name).jsonl")
        commandURL = evidence.appendingPathComponent("\(name)-command.json")
        let outputURL = evidence.appendingPathComponent("\(name)-stderr.txt")
        guard FileManager.default.createFile(atPath: outputURL.path, contents: nil) else { throw LabError("Cannot create fixture stderr file") }
        output = try FileHandle(forWritingTo: outputURL)
        process.executableURL = binary
        process.arguments = ["--log", logURL.path, "--commands", commandURL.path] + arguments
        process.standardOutput = output
        process.standardError = output
        try process.run()
    }

    func ready() throws {
        try waitUntil("\(name) readiness") {
            guard self.process.isRunning else { throw LabError("\(self.name) exited before readiness; see fixture stderr") }
            return self.records.contains { $0["event"] as? String == "ready" }
        }
    }

    func command(_ command: String, fields: [String: Any] = [:]) throws -> [String: Any] {
        let id = UUID().uuidString
        var body = fields
        body["requestID"] = id
        body["command"] = command
        try JSONFile.write(body, to: commandURL)
        var response: [String: Any]?
        try waitUntil("\(name) command \(command)") {
            guard self.process.isRunning else { throw LabError("\(self.name) is no longer running") }
            response = self.records.last { $0["event"] as? String == "query" && $0["requestID"] as? String == id }
            return response != nil
        }
        return response!
    }

    func stop() -> [String: Any] {
        if process.isRunning {
            try? JSONFile.write(["requestID": UUID().uuidString, "command": "quit"], to: commandURL)
            let deadline = Date().addingTimeInterval(1)
            while process.isRunning && Date() < deadline { Thread.sleep(forTimeInterval: 0.02) }
            if process.isRunning {
                process.terminate()
                let deadline = Date().addingTimeInterval(1)
                while process.isRunning && Date() < deadline { Thread.sleep(forTimeInterval: 0.02) }
                if process.isRunning { kill(process.processIdentifier, SIGKILL) }
            }
            process.waitUntilExit()
        }
        try? output?.close()
        return ["name": name, "pid": process.processIdentifier, "terminated": !process.isRunning,
                "exitStatus": process.terminationStatus]
    }
}

@MainActor
final class LabRunner {
    let options: RunOptions
    private var fixtures: [String: FixtureProcess] = [:]
    private var injector: Injector?
    private var results: [[String: Any]] = []
    private var report: [String: Any] = [:]
    private var evidence: URL!
    private var visible: CGRect = .zero
    private var originalPointer: CGPoint?
    private var initialA: CGRect = .zero
    private var initialB: CGRect = .zero

    init(options: RunOptions) { self.options = options }

    func run() throws -> Bool {
        // This function is reachable only after argument parsing checks consent.
        let reportURL = URL(fileURLWithPath: options.report!).standardizedFileURL
        try FileManager.default.createDirectory(at: reportURL.deletingLastPathComponent(), withIntermediateDirectories: true)
        evidence = reportURL.deletingPathExtension().appendingPathExtension("evidence-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: evidence, withIntermediateDirectories: false)
        report = ["schemaVersion": 1, "injector": options.injector, "dryRun": false,
                  "startedAt": Date().timeIntervalSince1970, "macOS": try macOSVersion(),
                  "evidenceDirectory": evidence.path, "permissions": Doctor.permissions(),
                  "scenarioOrder": ScenarioPlan.steps.map(\.0),
                  "freshInjector": options.injector == "cmd"
                    ? "S7 is the driver's first click; external service freshness must be arranged by its operator"
                    : "S7 is the first click in this new driver process; lastNumber starts at zero",
                  "commandTemplate": options.template ?? NSNull()]

        interrupted = 0
        let oldINT = signal(SIGINT) { _ in interrupted = 1 }
        let oldTERM = signal(SIGTERM) { _ in interrupted = 1 }
        defer { signal(SIGINT, oldINT); signal(SIGTERM, oldTERM) }
        do {
            let permissions = Doctor.permissions()
            guard permissions["accessibility"] == true, permissions["postEvents"] == true else {
                throw LabError("Accessibility and Post Event access are required. Run doctor and grant access on the test Mac")
            }
            try setup()
            injector = try Injector(kind: options.injector, template: options.template)
            // No fixture setup posts mouse events. S7 is first, including for
            // WindowServer's stricter first-click-per-process behavior.
            scenario("S7") { try self.contentActivation(first: true) }
            scenario("S1") { try self.contentActivation(first: false) }
            scenario("S2") { try self.titleActivation() }
            scenario("S3") { try self.titleDrag() }
            scenario("S4") { try self.menuSelection() }
            scenario("S5") { try self.menuDismissal() }
            scenario("S6") { try self.doubleClick() }
        } catch {
            report["error"] = String(describing: error)
        }

        // Cleanup is independent of scenario success and also runs after Ctrl-C.
        var cleanup: [String: Any] = [:]
        if let injector {
            cleanup["releaseErrors"] = injector.releaseAll()
            if let originalPointer {
                do {
                    try injector.send(.move, at: originalPointer)
                    // CGEventPost is asynchronous. Allow WindowServer to apply
                    // the restore before checking the physical pointer state.
                    var observed: CGPoint?
                    let deadline = Date().addingTimeInterval(0.5)
                    repeat {
                        observed = CGEvent(source: nil)?.location
                        if let observed, hypot(observed.x - originalPointer.x, observed.y - originalPointer.y) <= 2 { break }
                        pause(0.02)
                    } while Date() < deadline
                    cleanup["pointerRestored"] = observed.map { hypot($0.x - originalPointer.x, $0.y - originalPointer.y) <= 2 } ?? false
                    cleanup["pointerObserved"] = observed.map(ScreenCoordinates.pointJSON) ?? [:]
                } catch { cleanup["pointerRestored"] = false; cleanup["pointerError"] = String(describing: error) }
            }
            report["injections"] = injector.records
        } else { cleanup["pointerRestored"] = "No injector was started; no pointer events posted" }
        cleanup["fixtures"] = fixtures.keys.sorted().compactMap { fixtures[$0]?.stop() }
        report["cleanup"] = cleanup
        let completedIDs = Set(results.compactMap { $0["id"] as? String })
        for (id, _, expected) in ScenarioPlan.steps where !completedIDs.contains(id) {
            results.append(["id": id, "status": "FAIL", "expected": expected,
                            "error": "Not run because setup failed or the run was interrupted"])
        }
        report["scenarios"] = results
        report["finishedAt"] = Date().timeIntervalSince1970
        let releaseErrors = cleanup["releaseErrors"] as? [String] ?? []
        let restored = cleanup["pointerRestored"] as? Bool ?? (injector == nil)
        let success = report["error"] == nil && results.allSatisfy { $0["status"] as? String == "PASS" } && releaseErrors.isEmpty && restored
        report["passed"] = success
        try JSONFile.write(report, to: reportURL)
        print("Scenario  Result  Expected")
        for result in results {
            print("\(result["id"]!)        \(result["status"]!)    \(result["expected"]!)")
            if let error = result["error"] { print("          \(error)") }
        }
        if let error = report["error"] { print("Run error: \(error)") }
        print("Report: \(reportURL.path)\nEvidence: \(evidence.path)")
        print("Pointer restored: \(cleanup["pointerRestored"] ?? false)")
        return success
    }

    private func setup() throws {
        guard let primary = NSScreen.screens.first else { throw LabError("No GUI display. Use a logged-in macOS desktop") }
        visible = ScreenCoordinates.cgRect(primary.visibleFrame)
        guard visible.width >= 900, visible.height >= 650 else { throw LabError("Test display needs at least 900x650 usable points") }
        guard let pointer = CGEvent(source: nil)?.location else { throw LabError("Cannot read original pointer location") }
        originalPointer = pointer
        report["originalPointer"] = ScreenCoordinates.pointJSON(pointer)
        report["visibleFrame"] = ScreenCoordinates.rectJSON(visible)
        let executable = URL(fileURLWithPath: CommandLine.arguments[0]).standardizedFileURL.resolvingSymlinksInPath()
        let bundles = options.bundles.map { URL(fileURLWithPath: $0) } ?? executable.deletingLastPathComponent()
        let paths = ["A": bundles.appendingPathComponent("FocusLab A.app/Contents/MacOS/FocusLabFixture"),
                     "B": bundles.appendingPathComponent("FocusLab B.app/Contents/MacOS/FocusLabFixture"),
                     "Menu": bundles.appendingPathComponent("FocusLabMenuBar.app/Contents/MacOS/FocusLabMenuBar")]
        for path in paths.values where !FileManager.default.isExecutableFile(atPath: path.path) {
            throw LabError("Missing fixture executable: \(path.path). Run build.sh or pass --bundles")
        }
        initialA = CGRect(x: visible.minX + 40, y: visible.minY + 80, width: 460, height: 340)
        initialB = CGRect(x: initialA.minX + 300, y: initialA.minY + 140, width: 400, height: 340)
        for name in ["A", "B", "Menu"] {
            var arguments: [String] = []
            if name != "Menu" {
                let frame = name == "A" ? initialA : initialB
                arguments = ["--title", "FocusLab \(name)", "--color", name == "A" ? "blue" : "orange", "--frame", frameArgument(frame)]
            }
            fixtures[name] = try FixtureProcess(name: name, binary: paths[name]!, evidence: evidence, arguments: arguments)
            try fixtures[name]!.ready()
        }
        try backgroundA()
        report["initialState"] = try allStates()
    }

    private func scenario(_ id: String, body: () throws -> [String: Any]) {
        guard interrupted == 0 else { return }
        let starts = fixtures.mapValues { $0.records.count }
        let injectionStart = injector!.records.count
        var result: [String: Any] = ["id": id, "expected": ScenarioPlan.steps.first { $0.0 == id }!.2,
                                     "startedAt": Date().timeIntervalSince1970]
        do {
            let evidence = try body()
            result["evidence"] = evidence
            let assertions = evidence["assertions"] as? [String: Bool] ?? [:]
            result["status"] = !assertions.isEmpty && assertions.values.allSatisfy { $0 } ? "PASS" : "FAIL"
        } catch {
            result["status"] = "FAIL"
            result["error"] = String(describing: error)
        }
        result["logs"] = fixtures.mapValues { fixture in Array(fixture.records.dropFirst(starts[fixture.name] ?? 0)) }
        result["injections"] = Array(injector!.records.dropFirst(injectionStart))
        result["frontmostAfter"] = frontmost()
        result["finishedAt"] = Date().timeIntervalSince1970
        results.append(result)
        print("\(id): \(result["status"]!)")
    }

    private func contentActivation(first: Bool) throws -> [String: Any] {
        try resetFrames()
        try backgroundA()
        let before = try allStates()
        let a = try fixture("A")
        let state = try a.command("query")
        let target = try point(state, "buttonCenter")
        try requireExposed(target)
        let start = a.records.count
        let injectionStart = injector!.records.count
        try click(target)
        settle()
        let after = try a.command("query")
        let logs = Array(a.records.dropFirst(start))
        var assertions = activeAssertions(after)
        assertions["buttonAction"] = has(logs, "buttonAction")
        assertions["didBecomeActive"] = has(logs, "appDidBecomeActive")
        assertions["didBecomeKey"] = has(logs, "windowDidBecomeKey")
        if first {
            assertions["noEarlierMouseDown"] = !injector!.records.prefix(injectionStart).contains { $0["action"] as? String == "down" }
            if options.injector == "numbered" {
                let down = injector!.records.dropFirst(injectionStart).first { $0["action"] as? String == "down" }
                let number = down?["eventNumber"] as? Int64
                let seed = down?["hidSeed"] as? Int64
                assertions["firstNumberAboveHIDSeed"] = number != nil && seed != nil && number! > seed!
            }
        }
        return ["assertions": assertions, "before": before, "after": after, "frontmost": frontmost(), "windowOrder": windowOrder()]
    }

    private func titleActivation() throws -> [String: Any] {
        try resetFrames(); try backgroundA()
        let state = try fixture("A").command("query")
        let target = try point(state, "titleBarCenter")
        try requireExposed(target)
        try click(target); settle()
        let after = try fixture("A").command("query")
        return ["assertions": activeAssertions(after), "before": state, "after": after]
    }

    private func titleDrag() throws -> [String: Any] {
        try resetFrames(); try backgroundA()
        let a = try fixture("A")
        let before = try a.command("query")
        let start = try point(before, "titleBarCenter")
        guard let oldFrame = ScreenCoordinates.rect(before["frame"]) else { throw LabError("A frame missing") }
        try requireExposed(start)
        try injector!.send(.move, at: start); pause(0.08)
        try injector!.send(.down, at: start); pause(0.12)
        // The window server anchors a title-bar drag at the FIRST drag event (verified on macOS 27
        // with every injector, including a Synergy-style one). A physical mouse starts with a
        // 1-2 pt movement, so start the same way; the window then follows the remaining motion.
        try injector!.send(.drag, at: CGPoint(x: start.x + 1, y: start.y)); pause(0.04)
        for step in 1...10 {
            try checkInterrupted()
            let next = CGPoint(x: start.x + Double(step) * 12, y: start.y + Double(step) * 8)
            try injector!.send(.drag, at: next); pause(0.04)
        }
        try injector!.send(.up, at: CGPoint(x: start.x + 120, y: start.y + 80)); settle()
        let after = try a.command("query")
        guard let newFrame = ScreenCoordinates.rect(after["frame"]) else { throw LabError("A frame missing after drag") }
        let dx = newFrame.minX - oldFrame.minX, dy = newFrame.minY - oldFrame.minY
        // Expected: the pointer's motion after the 1-pt drag start, i.e. (+119, +80) ± 2.
        return ["assertions": ["deltaX": abs(dx - 119) <= 2, "deltaY": abs(dy - 80) <= 2],
                "before": before, "after": after, "delta": ["x": dx, "y": dy], "steps": 10]
    }

    private func prepareMenu() throws -> [String: Any] {
        let menu = try fixture("Menu")
        _ = try menu.command("closeMenu")
        let state = try menu.command("query")
        let status = try point(state, "statusButtonCenter")
        let aFrame = CGRect(x: min(max(status.x - 230, visible.minX + 5), visible.maxX - 465),
                            y: visible.minY + 6, width: 460, height: 340)
        let bFrame = CGRect(x: aFrame.midX < visible.midX ? visible.maxX - 410 : visible.minX + 10,
                            y: visible.maxY - 350, width: 400, height: 340)
        _ = try fixture("A").command("frame", fields: ["frame": ScreenCoordinates.rectJSON(aFrame)])
        _ = try fixture("B").command("frame", fields: ["frame": ScreenCoordinates.rectJSON(bFrame)])
        try backgroundA()
        let start = menu.records.count
        try click(status)
        var geometry: [String: Any]?
        try waitUntil("native menu opening and item geometry") {
            let logs = Array(menu.records.dropFirst(start))
            geometry = logs.last { $0["event"] as? String == "menuGeometry" }
            return self.has(logs, "menuOpen") && geometry != nil
        }
        return geometry!
    }

    private func menuSelection() throws -> [String: Any] {
        let geometry = try prepareMenu()
        guard let items = geometry["items"] as? [[String: Any]], !items.isEmpty else { throw LabError("Menu item geometry unavailable") }
        let a = try fixture("A")
        var aState = try a.command("query")
        var aFrame = ScreenCoordinates.rect(aState["frame"])!
        var item = items.last { ScreenCoordinates.point($0["center"]).map { aFrame.contains($0) } ?? false } ?? items.last!
        var target = try point(item, "center")
        // Keep A inactive while placing its window under the selected menu item.
        if !aFrame.contains(target) {
            aFrame.origin.x = min(max(target.x - 230, visible.minX + 5), visible.maxX - 465)
            aFrame.origin.y = min(max(target.y - 80, visible.minY + 1), visible.maxY - 345)
            aState = try a.command("frame", fields: ["frame": ScreenCoordinates.rectJSON(aFrame)])
            aFrame = ScreenCoordinates.rect(aState["frame"])!
            item = items.last { ScreenCoordinates.point($0["center"]).map { aFrame.contains($0) } ?? false } ?? item
            target = try point(item, "center")
        }
        let menu = try fixture("Menu")
        let start = menu.records.count
        let before = frontmost()
        guard let beforePID = before["pid"] as? Int32, beforePID != a.process.processIdentifier,
              aState["active"] as? Bool == false else {
            throw LabError("S4 precondition failed: A must be inactive")
        }
        let aStart = a.records.count
        try click(target); settle()
        let logs = Array(menu.records.dropFirst(start))
        let selectedID = item["itemID"] as? Int
        let action = logs.contains { $0["event"] as? String == "menuItemAction" && $0["itemID"] as? Int == selectedID }
        let after = frontmost()
        let afterPID = after["pid"] as? Int32
        let afterA = try a.command("query")
        return ["assertions": ["selectedItemAction": action, "menuClosed": has(logs, "menuClose"),
                                "frontmostIsNotA": afterPID != nil && afterPID != a.process.processIdentifier,
                                "AStayedInactive": afterA["active"] as? Bool == false,
                                "noAButtonActionThroughMenu": !has(Array(a.records.dropFirst(aStart)), "buttonAction")],
                "menu": geometry, "chosenItem": item, "itemOverA": aFrame.contains(target), "A": aState,
                "frontmostBefore": before, "frontmostAfter": after]
    }

    private func menuDismissal() throws -> [String: Any] {
        let geometry = try prepareMenu()
        let a = try fixture("A"), menu = try fixture("Menu")
        let before = try a.command("query")
        let target = try point(before, "emptyContentPoint")
        guard before["active"] as? Bool == false else { throw LabError("S5 requires inactive A") }
        guard let menuFrame = ScreenCoordinates.rect(geometry["menuFrame"]), !menuFrame.insetBy(dx: -4, dy: -4).contains(target) else {
            throw LabError("Empty A content point is inside menu bounds; no safe outside click")
        }
        try requireExposed(target)
        let aStart = a.records.count, menuStart = menu.records.count
        try click(target); settle()
        let after = try a.command("query")
        let aLogs = Array(a.records.dropFirst(aStart)), menuLogs = Array(menu.records.dropFirst(menuStart))
        return ["assertions": ["menuClosed": has(menuLogs, "menuClose"), "noButtonActionThroughMenu": !has(aLogs, "buttonAction")],
                "activationIsObservationOnly": true, "before": before, "after": after, "menu": geometry,
                "observedFrontmost": frontmost(), "contentMouseDownObserved": has(aLogs, "mouseDown")]
    }

    private func doubleClick() throws -> [String: Any] {
        _ = try fixture("Menu").command("closeMenu")
        try resetFrames()
        try front("A")
        let a = try fixture("A")
        let state = try a.command("query")
        let target = try point(state, "emptyContentPoint")
        let start = a.records.count
        try click(target)
        pause(min(0.08, NSEvent.doubleClickInterval / 5))
        try click(target); settle()
        let logs = Array(a.records.dropFirst(start))
        return ["assertions": ["clickCount2": logs.contains { $0["event"] as? String == "mouseDown" && $0["clickCount"] as? Int == 2 }],
                "doubleClickInterval": NSEvent.doubleClickInterval, "target": ScreenCoordinates.pointJSON(target)]
    }

    private func backgroundA() throws {
        // Arm a known inactive target. No AX raise or event-based focus helper.
        try front("A"); try front("B")
        try waitUntil("A inactive and not key") {
            let state = try self.fixture("A").command("query")
            return state["active"] as? Bool == false && state["key"] as? Bool == false
        }
        pause(NSEvent.doubleClickInterval + 0.1)
    }

    /// SETUP ONLY: arrange which fixture is in front before a scenario. This never runs between a
    /// scenario's synthetic events, so it cannot help the click path under test.
    private func front(_ name: String) throws {
        let fixture = try fixture(name)
        let pid = fixture.process.processIdentifier
        func isFront() throws -> Bool {
            let state = try fixture.command("query")
            return NSWorkspace.shared.frontmostApplication?.processIdentifier == pid &&
                state["active"] as? Bool == true && state["key"] as? Bool == true
        }
        _ = try fixture.command("front")
        // Since macOS 14, an app's own NSApp.activate() is only a request; when the lab runs from a
        // background host (SSH, remote agent) the system can refuse it. Fall back to the
        // Accessibility frontmost attribute, which the driver's Accessibility grant allows.
        if (try? waitUntil("\(name) frontmost (self-activation)", timeout: 1.5, condition: isFront)) == nil {
            let app = AXUIElementCreateApplication(pid)
            AXUIElementSetAttributeValue(app, kAXFrontmostAttribute as CFString, kCFBooleanTrue)
            var windows: CFTypeRef?
            if AXUIElementCopyAttributeValue(app, kAXWindowsAttribute as CFString, &windows) == .success,
               let first = (windows as? [AXUIElement])?.first {
                AXUIElementSetAttributeValue(app, kAXMainWindowAttribute as CFString, first)
                AXUIElementPerformAction(first, kAXRaiseAction as CFString)
            }
            _ = try fixture.command("front")
        }
        try waitUntil("\(name) frontmost and key", condition: isFront)
    }

    private func resetFrames() throws {
        _ = try fixture("Menu").command("closeMenu")
        for (name, frame) in [("A", initialA), ("B", initialB)] {
            _ = try fixture(name).command("frame", fields: ["frame": ScreenCoordinates.rectJSON(frame)])
        }
    }

    private func requireExposed(_ point: CGPoint) throws {
        guard visible.contains(point) else { throw LabError("Target lies outside usable display") }
        let b = try fixture("B").command("query")
        guard let frame = ScreenCoordinates.rect(b["frame"]), !frame.contains(point) else {
            throw LabError("A's target is covered by B; refusing a misleading injection")
        }
    }

    private func activeAssertions(_ state: [String: Any]) -> [String: Bool] {
        let pid = fixtures["A"]!.process.processIdentifier
        return ["AActive": state["active"] as? Bool == true, "AKey": state["key"] as? Bool == true,
                "frontmostA": NSWorkspace.shared.frontmostApplication?.processIdentifier == pid,
                "ARaisedAboveB": windowOrder().first == Int(pid)]
    }

    private func windowOrder() -> [Int] {
        let pids = Set([fixtures["A"], fixtures["B"]].compactMap { $0.map { Int($0.process.processIdentifier) } })
        let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
        return windows.compactMap { row in
            guard let pid = row[kCGWindowOwnerPID as String] as? Int, pids.contains(pid),
                  row[kCGWindowLayer as String] as? Int == 0 else { return nil }
            return pid
        }
    }

    private func frontmost() -> [String: Any] {
        guard let app = NSWorkspace.shared.frontmostApplication else { return ["unknown": true] }
        return ["pid": app.processIdentifier, "bundleID": app.bundleIdentifier ?? "", "name": app.localizedName ?? ""]
    }
    private func fixture(_ name: String) throws -> FixtureProcess {
        guard let fixture = fixtures[name] else { throw LabError("Fixture \(name) not started") }; return fixture
    }
    private func allStates() throws -> [String: Any] {
        var states: [String: Any] = ["frontmost": frontmost(), "windowOrder": windowOrder()]
        for name in fixtures.keys.sorted() { states[name] = try fixtures[name]!.command("query") }
        return states
    }
    private func point(_ state: [String: Any], _ key: String) throws -> CGPoint {
        guard let point = ScreenCoordinates.point(state[key]) else { throw LabError("Missing geometry: \(key)") }; return point
    }
    private func has(_ logs: [[String: Any]], _ event: String) -> Bool { logs.contains { $0["event"] as? String == event } }
    private func click(_ point: CGPoint) throws {
        try checkInterrupted()
        try injector!.send(.move, at: point); pause(0.04)
        try injector!.send(.down, at: point); pause(min(0.025, NSEvent.doubleClickInterval / 10))
        try injector!.send(.up, at: point)
    }
    private func pause(_ duration: TimeInterval) { RunLoop.current.run(until: Date().addingTimeInterval(duration)) }
    private func settle() { pause(0.55) }
    private func checkInterrupted() throws { if interrupted != 0 { throw LabError("Interrupted") } }
    private func frameArgument(_ frame: CGRect) -> String { "\(frame.minX),\(frame.minY),\(frame.width),\(frame.height)" }
    private func macOSVersion() throws -> [String: String] {
        let process = Process(), pipe = Pipe()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/sw_vers")
        process.standardOutput = pipe
        try process.run()
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        process.waitUntilExit()
        guard process.terminationStatus == 0 else { throw LabError("sw_vers failed") }
        return ["sw_vers": String(data: data, encoding: .utf8) ?? "", "processInfo": ProcessInfo.processInfo.operatingSystemVersionString]
    }
}
