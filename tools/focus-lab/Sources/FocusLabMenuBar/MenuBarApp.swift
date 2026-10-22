import AppKit
import FocusLabSupport

/// Public AppKit has no NSMenu.rect(for:) API. A custom view gives us actual
/// on-screen bounds, without AX scraping or private APIs. Selection is handled
/// by this view and closes the native tracking menu through cancelTracking().
@MainActor
private final class MenuItemView: NSView {
    let itemID: Int
    let label: String
    var choose: (@MainActor (Int) -> Void)?
    init(itemID: Int, label: String) {
        self.itemID = itemID; self.label = label
        super.init(frame: NSRect(x: 0, y: 0, width: 230, height: 30))
    }
    required init?(coder: NSCoder) { nil }
    override var acceptsFirstResponder: Bool { true }
    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
    override func draw(_ dirtyRect: NSRect) {
        (label as NSString).draw(at: NSPoint(x: 14, y: 7), withAttributes: [
            .font: NSFont.menuFont(ofSize: 14), .foregroundColor: NSColor.labelColor,
        ])
    }
    override func mouseDown(with event: NSEvent) { }
    override func mouseUp(with event: NSEvent) {
        if bounds.contains(convert(event.locationInWindow, from: nil)) { choose?(itemID) }
    }
}

@MainActor
private final class MenuDelegate: NSObject, NSApplicationDelegate, NSMenuDelegate {
    let args: FixtureArguments
    let log: EventLog
    var statusItem: NSStatusItem!
    let menu = NSMenu(title: "FocusLab")
    var itemViews: [MenuItemView] = []
    var inbox: CommandInbox?
    var timer: Timer?
    var menuOpen = false
    var lastGeometry = ""

    init(args: FixtureArguments, log: EventLog) { self.args = args; self.log = log }

    func applicationDidFinishLaunching(_ notification: Notification) {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        statusItem.button?.title = "FLab"
        menu.delegate = self
        menu.autoenablesItems = false
        for id in 1...3 {
            let item = NSMenuItem(title: "Lab item \(id)", action: #selector(itemChosen(_:)), keyEquivalent: "")
            item.tag = id
            item.target = self
            let view = MenuItemView(itemID: id, label: item.title)
            view.choose = { [weak self, weak item] _ in
                guard let self, let item else { return }
                NSApp.sendAction(item.action!, to: self, from: item)
                self.menu.cancelTracking()
            }
            item.view = view
            itemViews.append(view)
            menu.addItem(item)
        }
        statusItem.menu = menu
        do {
            inbox = CommandInbox(path: try args.require("--commands")) { [weak self] in self?.command($0) }
            inbox?.start()
        } catch { log.append("fatal", ["message": String(describing: error)]); NSApp.terminate(nil); return }
        log.append("start", ["bundleID": Bundle.main.bundleIdentifier ?? "unbundled"])
        let timer = Timer(timeInterval: 0.04, repeats: true) { [weak self] _ in
            MainActor.assumeIsolated { self?.publishGeometry() }
        }
        self.timer = timer
        RunLoop.main.add(timer, forMode: .common)
        RunLoop.main.add(timer, forMode: .eventTracking)
        log.append("ready", state())
    }

    @objc func itemChosen(_ item: NSMenuItem) { log.append("menuItemAction", ["itemID": item.tag, "title": item.title]) }
    func menuWillOpen(_ menu: NSMenu) { menuOpen = true; lastGeometry = ""; log.append("menuOpen") }
    func menuDidClose(_ menu: NSMenu) { menuOpen = false; log.append("menuClose") }

    private func viewRect(_ view: NSView) -> CGRect? {
        guard let window = view.window, window.isVisible, !view.isHiddenOrHasHiddenAncestor else { return nil }
        return ScreenCoordinates.cgRect(window.convertToScreen(view.convert(view.bounds, to: nil)))
    }

    private func state() -> [String: Any] {
        var result: [String: Any] = ["menuOpen": menuOpen, "coordinateSystem": "CG global points; top-left origin on primary display"]
        if let button = statusItem.button, let frame = viewRect(button) {
            result["statusButtonFrame"] = ScreenCoordinates.rectJSON(frame)
            result["statusButtonCenter"] = ScreenCoordinates.pointJSON(CGPoint(x: frame.midX, y: frame.midY))
        }
        if menuOpen {
            if let window = itemViews.first?.window, window.isVisible {
                result["menuFrame"] = ScreenCoordinates.rectJSON(ScreenCoordinates.cgRect(window.frame))
            }
            result["items"] = itemViews.compactMap { view -> [String: Any]? in
                guard let frame = viewRect(view), frame.width > 0, frame.height > 0 else { return nil }
                return ["itemID": view.itemID, "title": view.label, "frame": ScreenCoordinates.rectJSON(frame),
                        "center": ScreenCoordinates.pointJSON(CGPoint(x: frame.midX, y: frame.midY))]
            }
        }
        return result
    }

    private func publishGeometry() {
        guard menuOpen else { return }
        let geometry = state()
        guard let items = geometry["items"] as? [[String: Any]], items.count == 3,
              let data = try? JSONSerialization.data(withJSONObject: geometry, options: [.sortedKeys]),
              let signature = String(data: data, encoding: .utf8), signature != lastGeometry else { return }
        lastGeometry = signature
        log.append("menuGeometry", geometry)
    }

    private func command(_ command: [String: Any]) {
        if command["command"] as? String == "quit" { NSApp.terminate(nil); return }
        if command["command"] as? String == "closeMenu" { menu.cancelTracking() }
        var response = state()
        response["requestID"] = command["requestID"]
        log.append("query", response)
    }
}

@main
@MainActor
struct MenuBarMain {
    static func main() {
        do {
            let args = try FixtureArguments(Array(CommandLine.arguments.dropFirst()))
            let log = try EventLog(path: args.require("--log"))
            let app = NSApplication.shared
            app.setActivationPolicy(.accessory)
            let delegate = MenuDelegate(args: args, log: log)
            app.delegate = delegate
            withExtendedLifetime(delegate) { app.run() }
        } catch {
            FileHandle.standardError.write(Data("FocusLabMenuBar: \(error)\n".utf8))
            exit(1)
        }
    }
}
