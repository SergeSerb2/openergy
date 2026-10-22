import AppKit
// The SDK imports the immutable AX option key as a mutable C global. All use
// here is on main; retain Swift 6 isolation for our own state.
@preconcurrency import ApplicationServices
import FocusLabSupport

struct RunOptions {
    var injector = "numbered"
    var template: String?
    var report: String?
    var bundles: String?
    var acknowledged = false
    var dryRun = false

    init(_ arguments: [String]) throws {
        var index = 0
        var seen: Set<String> = []
        while index < arguments.count {
            let key = arguments[index]
            guard seen.insert(key).inserted else { throw LabError("Duplicate option: \(key)") }
            switch key {
            case "--dry-run": dryRun = true
            case "--i-understand-this-moves-the-pointer": acknowledged = true
            case "--injector", "--cmd-template", "--report", "--bundles":
                guard index + 1 < arguments.count else { throw LabError("Missing value for \(key)") }
                index += 1
                let value = arguments[index]
                switch key {
                case "--injector": injector = value
                case "--cmd-template": template = value
                case "--report": report = value
                default: bundles = value
                }
            default: throw LabError("Unknown run option: \(key)")
            }
            index += 1
        }
        guard ["legacy", "numbered", "cmd"].contains(injector) else { throw LabError("--injector must be legacy, numbered, or cmd") }
        if injector == "cmd", template == nil { throw LabError("cmd requires --cmd-template") }
        if injector != "cmd", template != nil { throw LabError("--cmd-template requires --injector cmd") }
        if !dryRun {
            guard acknowledged else { throw LabError("Refusing to inject: add --i-understand-this-moves-the-pointer on a dedicated test Mac") }
            guard seen.contains("--injector") else { throw LabError("Live runs require --injector") }
            guard let report, !report.isEmpty else { throw LabError("Live runs require --report <json>") }
        }
    }
}

@MainActor
enum Doctor {
    static func permissions() -> [String: Bool] {
        let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: false] as CFDictionary
        return ["accessibility": AXIsProcessTrustedWithOptions(options),
                "postEvents": CGPreflightPostEventAccess(), "listenEvents": CGPreflightListenEventAccess()]
    }
    static func printStatus() {
        let status = permissions()
        print("Accessibility (AX, prompt=false): \(status["accessibility"]! ? "granted" : "not granted")")
        print("Post events (CGPreflightPostEventAccess): \(status["postEvents"]! ? "granted" : "not granted")")
        print("Input Monitoring (CGPreflightListenEventAccess): \(status["listenEvents"]! ? "granted" : "not granted")")
        print("Read-only check. No prompts, event taps, fixture launch, or input injection.")
    }
}

enum ScenarioPlan {
    static let steps: [(String, String, String)] = [
        ("S7", "FIRST: fresh injector; B front; click A's content button", "A active + key + raised; button action; first down recorded"),
        ("S1", "B front; click exposed content button of inactive A", "A active + key + raised; button action"),
        ("S2", "B front; click inactive A's title bar", "A active"),
        ("S3", "B front; drag A's title bar (+120,+80), 10 steps", "A frame delta (+120,+80), tolerance 2 pt"),
        ("S4", "B front; open status menu; click item over A if possible", "Item action; menu closes; frontmost app is not A"),
        ("S5", "B front; open menu; click empty A content outside menu", "Menu closes; no button action through menu; activation recorded only"),
        ("S6", "A front; two content clicks within doubleClickInterval", "A mouseDown log contains clickCount 2"),
    ]
    static func dryRun(_ options: RunOptions) throws {
        print("DRY RUN — no apps launched, no screen reads, no events posted, no pointer/focus changes.")
        print("Injector: \(options.injector); CG global point coordinates; native events post at kCGHIDEventTap.")
        print("SETUP  Capture pointer; launch A/B/menu binaries; await logs; position overlapping A behind B.")
        for (id, action, expected) in steps { print("\(id)  \(action)\n    EXPECT: \(expected)") }
        print("CLEANUP  Release held buttons; restore pointer; terminate only lab-owned fixtures; retain JSONL evidence.")
        print("Live run requires --injector, --report, and --i-understand-this-moves-the-pointer.")
        if let path = options.report {
            let url = URL(fileURLWithPath: path)
            try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
            try JSONFile.write(["dryRun": true, "injector": options.injector,
                                "scenarios": steps.map { ["id": $0.0, "action": $0.1, "expected": $0.2] }], to: url)
        }
    }
}

@main
@MainActor
struct DriverMain {
    static let help = """
    focus-lab — macOS synthetic-click activation verification lab (macOS 14+)

    focus-lab --help
    focus-lab doctor
    focus-lab run --dry-run [--injector legacy|numbered|cmd] [--report <json>]
    focus-lab run --injector legacy|numbered|cmd --report <json>
      --i-understand-this-moves-the-pointer [--bundles <dist directory>]
      [--cmd-template '<absolute executable> {action} {button} {x} {y}']

    Live run launches GUI apps, changes focus, and moves the pointer. Use a test Mac.
    S7 runs first in a fresh driver process, before any synthetic click. Then S1–S6 run.
    cmd argv actions: move/down/up/drag; buttons: left/right/middle; x/y: CG global points.
    Templates support quotes and backslash escapes, with no shell expansion.
    doctor and dry-run never request TCC permissions. dry-run never launches apps.
    Exit: 0 = pass/observation or safe command; 1 = failed/error; 2 = usage/refusal.
    """

    static func main() {
        let arguments = Array(CommandLine.arguments.dropFirst())
        if arguments.isEmpty || arguments.contains("--help") || arguments == ["help"] { print(help); return }
        do {
            switch arguments.first {
            case "doctor":
                guard arguments.count == 1 else { throw LabError("doctor accepts no options") }
                Doctor.printStatus()
            case "run":
                let options = try RunOptions(Array(arguments.dropFirst()))
                if options.dryRun { try ScenarioPlan.dryRun(options); return }
                let success = try LabRunner(options: options).run()
                if !success { exit(1) }
            default: throw LabError("Unknown command. Use focus-lab --help")
            }
        } catch {
            FileHandle.standardError.write(Data("focus-lab: \(error)\n".utf8))
            exit(2)
        }
    }
}
