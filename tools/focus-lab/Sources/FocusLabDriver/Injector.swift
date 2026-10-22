import AppKit
import CoreGraphics
import FocusLabSupport

enum MouseButton: String, CaseIterable {
    case left, right, middle
    var cgButton: CGMouseButton {
        switch self { case .left: .left; case .right: .right; case .middle: .center }
    }
    func eventType(_ action: Primitive) -> CGEventType {
        switch (action, self) {
        case (.move, _): .mouseMoved
        case (.down, .left): .leftMouseDown
        case (.up, .left): .leftMouseUp
        case (.drag, .left): .leftMouseDragged
        case (.down, .right): .rightMouseDown
        case (.up, .right): .rightMouseUp
        case (.drag, .right): .rightMouseDragged
        case (.down, .middle): .otherMouseDown
        case (.up, .middle): .otherMouseUp
        case (.drag, .middle): .otherMouseDragged
        }
    }
}

enum Primitive: String { case move, down, up, drag }

@MainActor
final class Injector {
    private struct Click {
        var time: TimeInterval
        var location: CGPoint
        var count: Int64
        var canContinue = true
    }
    let kind: String
    private let command: [String]?
    private var lastNumber: Int64 = 0
    private var pressedNumber: [MouseButton: Int64] = [:]
    private var pressedCount: [MouseButton: Int64] = [:]
    private var previousClick: [MouseButton: Click] = [:]
    private var lastPoint: CGPoint?
    private(set) var held: Set<MouseButton> = []
    private(set) var records: [[String: Any]] = []

    init(kind: String, template: String?) throws {
        self.kind = kind
        if kind == "cmd" {
            guard let template else { throw LabError("cmd requires --cmd-template") }
            let tokens = try Self.tokenize(template)
            guard let executable = tokens.first, executable.hasPrefix("/"),
                  ["{action}", "{button}", "{x}", "{y}"].allSatisfy({ template.contains($0) }) else {
                throw LabError("--cmd-template requires an absolute executable path and {action} {button} {x} {y}")
            }
            command = tokens
        } else {
            guard ["legacy", "numbered"].contains(kind) else { throw LabError("Unknown injector: \(kind)") }
            command = nil
        }
    }

    func send(_ action: Primitive, at point: CGPoint, button: MouseButton = .left) throws {
        guard point.x.isFinite, point.y.isFinite else { throw LabError("Non-finite pointer coordinate") }
        if action == .down, held.contains(button) { throw LabError("Duplicate mouse-down for \(button.rawValue)") }
        if action == .up || action == .drag {
            guard held.contains(button) else { throw LabError("\(action.rawValue) without mouse-down") }
        }
        if action == .move, !held.isEmpty { throw LabError("Use drag while a button is held") }

        var record: [String: Any] = ["action": action.rawValue, "button": button.rawValue,
                                     "point": ScreenCoordinates.pointJSON(point), "time": Date().timeIntervalSince1970]
        if let command {
            // Mark down before spawning: if the wrapper fails after posting it,
            // cleanup must still attempt to send a matching up.
            if action == .down { held.insert(button) }
            let replacements = ["{action}": action.rawValue, "{button}": button.rawValue,
                                "{x}": String(format: "%.3f", locale: Locale(identifier: "en_US_POSIX"), point.x),
                                "{y}": String(format: "%.3f", locale: Locale(identifier: "en_US_POSIX"), point.y)]
            let arguments = command.map { token in replacements.reduce(token) { $0.replacingOccurrences(of: $1.key, with: $1.value) } }
            let process = Process()
            process.executableURL = URL(fileURLWithPath: arguments[0])
            process.arguments = Array(arguments.dropFirst())
            process.standardOutput = FileHandle.nullDevice
            process.standardError = FileHandle.nullDevice
            do {
                try process.run()
                let deadline = Date().addingTimeInterval(3)
                while process.isRunning && Date() < deadline { Thread.sleep(forTimeInterval: 0.01) }
                if process.isRunning {
                    process.terminate()
                    Thread.sleep(forTimeInterval: 0.05)
                    if process.isRunning { kill(process.processIdentifier, SIGKILL) }
                    process.waitUntilExit()
                    throw LabError("External injector timed out: \(action.rawValue)")
                }
                guard process.terminationStatus == 0 else { throw LabError("External injector exited \(process.terminationStatus): \(action.rawValue)") }
            } catch {
                record["error"] = String(describing: error)
                records.append(record)
                throw error
            }
            record["eventNumber"] = "external injector owns event numbering"
        } else {
            let now = ProcessInfo.processInfo.systemUptime
            var number: Int64 = 0
            var count: Int64 = 0
            if action == .down {
                let seed = [CGEventType.leftMouseDown, .rightMouseDown, .otherMouseDown].reduce(Int64(0)) {
                    $0 + Int64(CGEventSource.counterForEventType(.hidSystemState, eventType: $1))
                }
                if kind == "numbered" {
                    let base = max(lastNumber, seed)
                    guard base < Int64.max else { throw LabError("Mouse event number exhausted") }
                    number = base + 1
                    lastNumber = number
                }
                if let previous = previousClick[button], previous.canContinue,
                   now - previous.time <= NSEvent.doubleClickInterval,
                   hypot(point.x - previous.location.x, point.y - previous.location.y) <= 4 {
                    count = previous.count + 1
                } else { count = 1 }
                previousClick[button] = Click(time: now, location: point, count: count)
                pressedNumber[button] = number
                pressedCount[button] = count
                record["hidSeed"] = seed
            } else if action == .drag || action == .up {
                number = pressedNumber[button] ?? 0
                count = pressedCount[button] ?? 0
                if let previous = previousClick[button], hypot(point.x - previous.location.x, point.y - previous.location.y) > 4 {
                    previousClick[button]?.canContinue = false
                }
            }
            guard let event = CGEvent(mouseEventSource: nil, mouseType: button.eventType(action),
                                      mouseCursorPosition: point, mouseButton: button.cgButton) else {
                throw LabError("CGEventCreateMouseEvent failed")
            }
            event.setIntegerValueField(.mouseEventClickState, value: count)
            if kind == "numbered", action != .move {
                event.setIntegerValueField(.mouseEventNumber, value: number)
            }
            if let previous = lastPoint, action == .move || action == .drag {
                event.setIntegerValueField(.mouseEventDeltaX, value: Int64((point.x - previous.x).rounded()))
                event.setIntegerValueField(.mouseEventDeltaY, value: Int64((point.y - previous.y).rounded()))
            }
            record["eventNumber"] = event.getIntegerValueField(.mouseEventNumber)
            record["clickState"] = count
            record["tap"] = "kCGHIDEventTap"
            event.post(tap: .cghidEventTap)
        }
        if action == .down { held.insert(button) }
        if action == .up { held.remove(button); pressedNumber.removeValue(forKey: button); pressedCount.removeValue(forKey: button) }
        lastPoint = point
        records.append(record)
    }

    func releaseAll() -> [String] {
        var errors: [String] = []
        for button in MouseButton.allCases where held.contains(button) {
            do { try send(.up, at: lastPoint ?? .zero, button: button) }
            catch { errors.append(String(describing: error)) }
        }
        return errors
    }

    /// Quote-aware argv parsing, not a shell. No expansion, pipes or evaluation.
    private static func tokenize(_ input: String) throws -> [String] {
        var tokens: [String] = [], token = ""
        var quote: Character?, escape = false, started = false
        for char in input {
            if escape { token.append(char); escape = false; started = true; continue }
            if char == "\\", quote != "'" { escape = true; started = true; continue }
            if let current = quote {
                if char == current { quote = nil } else { token.append(char) }
                continue
            }
            if char == "\"" || char == "'" { quote = char; started = true }
            else if char.isWhitespace {
                if started { tokens.append(token); token = ""; started = false }
            } else { token.append(char); started = true }
        }
        guard quote == nil, !escape else { throw LabError("Unclosed quote or escape in --cmd-template") }
        if started { tokens.append(token) }
        return tokens
    }
}
