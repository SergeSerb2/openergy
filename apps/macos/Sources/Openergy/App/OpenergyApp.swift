import AppKit
import ServiceManagement
import SwiftUI

@main
enum Main {
    static func main() {
        let arguments = ProcessInfo.processInfo.arguments
        if let index = arguments.firstIndex(of: "--snapshot"), arguments.indices.contains(index + 1) {
            MainActor.assumeIsolated {
                SnapshotRenderer.run(outputDirectory: URL(fileURLWithPath: arguments[index + 1]))
            }
            return
        }
        if let index = arguments.firstIndex(of: "--e2e"), arguments.indices.contains(index + 1) {
            MainActor.assumeIsolated {
                AppEndToEnd.run(outputDirectory: URL(fileURLWithPath: arguments[index + 1]))
            }
            return
        }
        if let index = arguments.firstIndex(of: "--open-at-login") {
            exit(LoginItemCommand.run(arguments.indices.contains(index + 1) ? arguments[index + 1] : "status"))
        }
        OpenergyApp.main()
    }
}

/// `Openergy --open-at-login on|off|status` sets the same login item as Settings → Open at
/// login, without opening a window, so scripted installs can enable it.
enum LoginItemCommand {
    static func run(_ value: String) -> Int32 {
        let service = SMAppService.mainApp
        do {
            switch value {
            case "on": if service.status != .enabled { try service.register() }
            case "off": if service.status != .notRegistered { try service.unregister() }
            case "status": break
            default:
                print("usage: Openergy --open-at-login on|off|status")
                return 64
            }
        } catch {
            FileHandle.standardError.write(Data("Could not change the login item: \(error.localizedDescription)\n".utf8))
            return 1
        }
        switch service.status {
        case .enabled: print("Open at login: on")
        case .requiresApproval: print("Open at login: allow Openergy in System Settings → General → Login Items")
        case .notRegistered: print("Open at login: off")
        default: print("Open at login: unavailable")
        }
        return service.status == .requiresApproval ? 2 : 0
    }
}

struct OpenergyApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var delegate
    @State private var model = AppModel.shared

    var body: some Scene {
        MenuBarExtra {
            MenuBarPanel(model: model)
        } label: {
            MenuBarLabel(model: model)
        }
        .menuBarExtraStyle(.window)

        Window("Openergy", id: WindowID.main) {
            MainWindow(model: model)
                .tint(Brand.accent)
        }
        .defaultSize(width: 960, height: 640)
        .windowResizability(.contentMinSize)
        .defaultLaunchBehavior(.suppressed)
        .commands {
            CommandGroup(replacing: .newItem) {}
            CommandGroup(replacing: .appSettings) {
                OpenMainWindowButton(title: "Settings…")
                    .keyboardShortcut(",", modifiers: .command)
            }
        }
    }
}

enum WindowID {
    static let main = "main"
}

/// The status item image. It also starts the model and opens the window on first run,
/// because the label is the one view SwiftUI renders at launch.
struct MenuBarLabel: View {
    let model: AppModel
    @Environment(\.openWindow) private var openWindow
    @AppStorage("didCompleteFirstRun") private var didCompleteFirstRun = false

    var body: some View {
        Image(nsImage: MenuBarIcon.image(for: model.tone, role: model.role, headline: model.headline))
            .task {
                model.start()
                if !didCompleteFirstRun {
                    didCompleteFirstRun = true
                    openWindow(id: WindowID.main)
                    NSApp.activate()
                }
            }
            // A held connection is dropped after 30 s, so bring the approval sheet forward now.
            .onChange(of: model.pendingTrust.first?.fingerprint) { _, fingerprint in
                guard fingerprint != nil else { return }
                openWindow(id: WindowID.main)
                NSApp.activate()
            }
    }
}

struct OpenMainWindowButton: View {
    var title: String
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        Button(title) {
            openWindow(id: WindowID.main)
            NSApp.activate()
        }
    }
}

final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { false }

    @MainActor
    func applicationWillTerminate(_ notification: Notification) {
        AppModel.shared.shutdown()
    }
}
