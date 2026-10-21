import AppKit
import FocusLabSupport

@MainActor
private final class DeliveryButton: NSButton {
    // Click delivery and activation are measured separately. Do not consume the
    // first click merely because the window is inactive.
    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
}

@MainActor
private final class ContentView: NSView {
    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
}

@MainActor
private final class LoggingWindow: NSWindow {
    var log: EventLog?
    override func sendEvent(_ event: NSEvent) {
        if [.leftMouseDown, .rightMouseDown, .otherMouseDown].contains(event.type),
           let content = contentView,
           content.bounds.contains(content.convert(event.locationInWindow, from: nil)) {
            log?.append("mouseDown", [
                "clickCount": event.clickCount, "button": event.buttonNumber,
                "eventNumber": event.eventNumber,
                "location": ScreenCoordinates.pointJSON(ScreenCoordinates.cgPoint(convertPoint(toScreen: event.locationInWindow))),
                "windowLocation": ScreenCoordinates.pointJSON(event.locationInWindow),
            ])
        }
        super.sendEvent(event)
    }
}

@MainActor
private final class FixtureDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate {
    let args: FixtureArguments
    let log: EventLog
    var window: LoggingWindow!
    var button: DeliveryButton!
    var inbox: CommandInbox?

    init(args: FixtureArguments, log: EventLog) { self.args = args; self.log = log }

    func applicationDidFinishLaunching(_ notification: Notification) {
        do {
            window = LoggingWindow(contentRect: NSRect(x: 0, y: 0, width: 460, height: 320),
                                   styleMask: [.titled, .closable, .miniaturizable], backing: .buffered, defer: false)
            window.title = args.values["--title"] ?? "FocusLab Fixture"
            window.isReleasedWhenClosed = false
            window.delegate = self
            window.log = log
            window.setFrame(ScreenCoordinates.nsRect(try args.frame()), display: false)
            let content = ContentView(frame: NSRect(origin: .zero, size: window.contentLayoutRect.size))
            content.wantsLayer = true
            content.layer?.backgroundColor = try windowColor().withAlphaComponent(0.25).cgColor
            window.contentView = content

            button = DeliveryButton(frame: NSRect(x: 28, y: content.bounds.height - 145, width: 230, height: 100))
            button.title = "Click delivery target"
            button.bezelStyle = .regularSquare
            button.target = self
            button.action = #selector(buttonClicked)
            button.autoresizingMask = [.minYMargin]
            content.addSubview(button)
            let field = NSTextField(frame: NSRect(x: 28, y: 38, width: 270, height: 26))
            field.placeholderString = "Text input target"
            content.addSubview(field)
            let label = NSTextField(labelWithString: "Empty content point is below the button.")
            label.frame = NSRect(x: 28, y: 8, width: 350, height: 22)
            content.addSubview(label)

            inbox = CommandInbox(path: try args.require("--commands")) { [weak self] in self?.command($0) }
            inbox?.start()
            // Initial ordering is setup only. Tests never ask the target to
            // activate itself after an injected click.
            window.makeKeyAndOrderFront(nil)
            log.append("start", ["bundleID": Bundle.main.bundleIdentifier ?? "unbundled",
                                 "windowNumber": window.windowNumber, "title": window.title])
            log.append("ready", state())
        } catch {
            log.append("fatal", ["message": String(describing: error)])
            NSApp.terminate(nil)
        }
    }

    @objc func buttonClicked() { log.append("buttonAction", ["title": button.title]) }
    private func windowColor() throws -> NSColor {
        let text = args.values["--color"] ?? "blue"
        switch text.lowercased() {
        case "blue": return .systemBlue
        case "orange": return .systemOrange
        case "red": return .systemRed
        case "green": return .systemGreen
        default:
            guard text.count == 7, text.first == "#", let rgb = UInt32(text.dropFirst(), radix: 16) else {
                throw LabError("--color requires blue, orange, red, green, or #RRGGBB")
            }
            return NSColor(srgbRed: CGFloat((rgb >> 16) & 255) / 255, green: CGFloat((rgb >> 8) & 255) / 255,
                           blue: CGFloat(rgb & 255) / 255, alpha: 1)
        }
    }
    func applicationDidBecomeActive(_ notification: Notification) { log.append("appDidBecomeActive") }
    func applicationDidResignActive(_ notification: Notification) { log.append("appDidResignActive") }
    func windowDidBecomeKey(_ notification: Notification) { log.append("windowDidBecomeKey") }
    func windowDidResignKey(_ notification: Notification) { log.append("windowDidResignKey") }
    func windowDidBecomeMain(_ notification: Notification) { log.append("windowDidBecomeMain") }
    func windowDidMove(_ notification: Notification) { frameChanged() }
    func windowDidResize(_ notification: Notification) { frameChanged() }
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }

    private func frameChanged() {
        guard let window else { return }
        log.append("windowFrameChanged", ["frame": ScreenCoordinates.rectJSON(ScreenCoordinates.cgRect(window.frame))])
    }

    private func state() -> [String: Any] {
        let content = window.contentView!
        let buttonCenter = NSPoint(x: button.bounds.midX, y: button.bounds.midY)
        let screenButton = window.convertPoint(toScreen: button.convert(buttonCenter, to: nil))
        let empty = window.convertPoint(toScreen: content.convert(NSPoint(x: 60, y: 95), to: nil))
        let contentTop = window.convertPoint(toScreen: NSPoint(x: 0, y: content.bounds.maxY)).y
        let titlePoint = NSPoint(x: window.frame.midX, y: (contentTop + window.frame.maxY) / 2)
        return ["bundleID": Bundle.main.bundleIdentifier ?? "unbundled", "windowNumber": window.windowNumber,
                "active": NSApp.isActive, "key": window.isKeyWindow, "main": window.isMainWindow,
                "coordinateSystem": "CG global points; top-left origin on primary display",
                "frame": ScreenCoordinates.rectJSON(ScreenCoordinates.cgRect(window.frame)),
                "titleBarCenter": ScreenCoordinates.pointJSON(ScreenCoordinates.cgPoint(titlePoint)),
                "buttonCenter": ScreenCoordinates.pointJSON(ScreenCoordinates.cgPoint(screenButton)),
                "emptyContentPoint": ScreenCoordinates.pointJSON(ScreenCoordinates.cgPoint(empty))]
    }

    private func command(_ command: [String: Any]) {
        let id = command["requestID"] as? String ?? ""
        switch command["command"] as? String {
        case "query": break
        case "front":
            NSApp.activate()
            window.makeKeyAndOrderFront(nil)
        case "frame":
            if let frame = ScreenCoordinates.rect(command["frame"]) {
                window.setFrame(ScreenCoordinates.nsRect(frame), display: true)
            }
        case "quit": NSApp.terminate(nil); return
        default: log.append("commandError", ["requestID": id]); return
        }
        var response = state()
        response["requestID"] = id
        response["command"] = command["command"]
        log.append("query", response)
    }
}

@main
@MainActor
struct FixtureMain {
    static func main() {
        do {
            let args = try FixtureArguments(Array(CommandLine.arguments.dropFirst()))
            let log = try EventLog(path: args.require("--log"))
            let app = NSApplication.shared
            app.setActivationPolicy(.regular)
            let delegate = FixtureDelegate(args: args, log: log)
            app.delegate = delegate
            withExtendedLifetime(delegate) { app.run() }
        } catch {
            FileHandle.standardError.write(Data("FocusLabFixture: \(error)\n".utf8))
            exit(1)
        }
    }
}
