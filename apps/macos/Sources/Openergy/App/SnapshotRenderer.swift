import AppKit
import SwiftUI

/// Renders the main surfaces with demo data into PNG files, in windows that are never shown.
/// Usage: `Openergy --snapshot <directory>`. Used for design review; it never touches the desktop.
@MainActor
enum SnapshotRenderer {
    static func run(outputDirectory: URL) {
        let app = NSApplication.shared
        app.setActivationPolicy(.prohibited)
        try? FileManager.default.createDirectory(at: outputDirectory, withIntermediateDirectories: true)

        let only = ProcessInfo.processInfo.environment["OPENERGY_SNAPSHOT_ONLY"]
        let appearances: [(String, NSAppearance.Name)] = [("dark", .darkAqua), ("light", .aqua)]
        let windowJobs: [(String, String, Pane)] = [
            ("layout-server", "server", .layout),
            ("layout-client", "client", .layout),
            ("layout-off", "setup", .layout),
            ("layout-offline", "offline", .layout),
            ("connection-client", "client", .connection),
            ("connection-server", "server", .connection),
            ("input", "client", .input),
            ("clipboard", "client", .clipboard),
            ("security", "trust", .security),
            ("permissions", "setup", .permissions),
            ("advanced", "client", .advanced),
            ("activity", "client", .activity),
        ]

        for (appearanceName, appearance) in appearances {
            for (file, scenario, pane) in windowJobs where only.map({ file.contains($0) }) ?? true {
                let model = AppModel(demo: DemoScenario.named(scenario))
                render(MainWindow(model: model, initialPane: pane).tint(Brand.accent),
                       size: CGSize(width: 1040, height: 680), appearance: appearance, titled: true, title: pane.title,
                       to: outputDirectory.appendingPathComponent("\(file)-\(appearanceName).png"))
            }
            for scenario in ["client", "server", "setup", "trust", "offline"] where only.map({ "menu-\(scenario)".contains($0) }) ?? true {
                let model = AppModel(demo: DemoScenario.named(scenario))
                render(MenuBarPanel(model: model).background(.regularMaterial),
                       size: nil, appearance: appearance, titled: false,
                       to: outputDirectory.appendingPathComponent("menu-\(scenario)-\(appearanceName).png"))
            }
            if only.map({ "trust-sheet".contains($0) }) ?? true {
                let model = AppModel(demo: DemoScenario.named("trust"))
                if let pending = model.pendingTrust.first {
                    render(TrustApprovalSheet(model: model, pending: pending).background(.background),
                           size: nil, appearance: appearance, titled: false,
                           to: outputDirectory.appendingPathComponent("trust-sheet-\(appearanceName).png"))
                }
            }
        }
        render(AppIconView(size: 256).padding(16), size: nil, appearance: .darkAqua, titled: false,
               to: outputDirectory.appendingPathComponent("app-icon.png"))
        print("Snapshots written to \(outputDirectory.path)")
        exit(0)
    }

    static func renderWindow<V: View>(_ view: V, title: String, appearance: NSAppearance.Name, to url: URL) {
        render(view, size: CGSize(width: 1040, height: 680), appearance: appearance, titled: true, title: title, to: url)
    }

    private static func render<V: View>(_ view: V, size: CGSize?, appearance: NSAppearance.Name, titled: Bool, title: String = "", to url: URL) {
        let controller = NSHostingController(rootView: view)
        controller.sceneBridgingOptions = titled ? [.toolbars, .title] : []
        let window = NSWindow(contentViewController: controller)
        window.styleMask = titled ? [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView] : [.borderless]
        window.appearance = NSAppearance(named: appearance)
        window.isReleasedWhenClosed = false
        window.title = title
        let contentSize = size ?? controller.view.fittingSize
        window.setContentSize(contentSize)
        window.layoutIfNeeded()

        // Let SwiftUI lay out, resolve the toolbar and finish onAppear work.
        RunLoop.current.run(until: Date().addingTimeInterval(0.8))

        guard let target = titled ? window.contentView?.superview : window.contentView else { return }
        let bounds = target.bounds
        let scale: CGFloat = 2
        guard let rep = NSBitmapImageRep(
            bitmapDataPlanes: nil, pixelsWide: Int(bounds.width * scale), pixelsHigh: Int(bounds.height * scale),
            bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
            colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0
        ) else { return }
        rep.size = bounds.size
        NSAppearance(named: appearance)?.performAsCurrentDrawingAppearance {
            target.cacheDisplay(in: bounds, to: rep)
        }
        try? rep.representation(using: .png, properties: [:])?.write(to: url)
        // Detach the view so a closed snapshot window stops observing the model (E2E keeps
        // driving the same model after each render).
        window.contentViewController = nil
        window.close()
    }
}
