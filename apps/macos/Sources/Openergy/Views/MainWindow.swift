import SwiftUI

enum Pane: String, CaseIterable, Identifiable, Hashable {
    case layout, connection, input, clipboard, security, permissions, advanced, activity
    var id: String { rawValue }

    var title: String {
        switch self {
        case .layout: "Layout"
        case .connection: "Connection"
        case .input: "Keyboard & Mouse"
        case .clipboard: "Clipboard"
        case .security: "Security"
        case .permissions: "Permissions"
        case .advanced: "Advanced"
        case .activity: "Activity"
        }
    }

    var symbol: String {
        switch self {
        case .layout: "rectangle.3.group"
        case .connection: "point.3.connected.trianglepath.dotted"
        case .input: "keyboard"
        case .clipboard: "doc.on.clipboard"
        case .security: "lock.shield"
        case .permissions: "hand.raised"
        case .advanced: "slider.horizontal.3"
        case .activity: "list.bullet.rectangle"
        }
    }
}

struct MainWindow: View {
    let model: AppModel
    var initialPane: Pane?
    @State private var pane: Pane?

    init(model: AppModel, initialPane: Pane? = nil) {
        self.model = model
        self.initialPane = initialPane
        // Start on the requested pane so no other pane (and its side effects) renders first.
        _pane = State(initialValue: initialPane ?? .layout)
    }
    @SceneStorage("pane") private var storedPane = Pane.layout.rawValue

    var body: some View {
        NavigationSplitView {
            List(selection: $pane) {
                Section {
                    sidebarRow(.layout)
                    sidebarRow(.connection)
                }
                Section("Settings") {
                    sidebarRow(.input)
                    sidebarRow(.clipboard)
                    sidebarRow(.security)
                    sidebarRow(.permissions)
                    sidebarRow(.advanced)
                }
                Section {
                    sidebarRow(.activity)
                }
            }
            .navigationSplitViewColumnWidth(min: 190, ideal: 210, max: 260)
        } detail: {
            Group {
                if model.isReady || pane == .activity || pane == .permissions {
                    detail(for: pane ?? .layout)
                } else {
                    ServiceStartingView(model: model)
                }
            }
                .frame(minWidth: 560, minHeight: 460)
                .navigationTitle((pane ?? .layout).title)
                .toolbar {
                    ToolbarItemGroup(placement: .primaryAction) {
                        if model.role != .off || model.link != .connected {
                            StatusPill(model: model)
                        }
                        RolePicker(model: model)
                            .frame(width: 180)
                    }
                }
                .safeAreaInset(edge: .bottom) { actionErrorBar }
        }
        .sheet(item: trustBinding) { pending in
            TrustApprovalSheet(model: model, pending: pending)
        }
        .onAppear {
            if let initialPane {
                pane = initialPane
            } else if !model.missingPermissions.isEmpty {
                pane = .permissions
            } else {
                pane = Pane(rawValue: storedPane) ?? .layout
            }
        }
        .onChange(of: pane) { _, next in storedPane = (next ?? .layout).rawValue }
    }

    private func sidebarRow(_ item: Pane) -> some View {
        Label {
            HStack {
                Text(item.title)
                Spacer()
                if item == .permissions, !model.missingPermissions.isEmpty {
                    Image(systemName: "exclamationmark.circle.fill")
                        .foregroundStyle(.orange)
                        .accessibilityLabel("Needs attention")
                }
                if item == .security, !model.pendingTrust.isEmpty {
                    Image(systemName: "exclamationmark.circle.fill")
                        .foregroundStyle(.orange)
                        .accessibilityLabel("Approval needed")
                }
            }
        } icon: {
            Image(systemName: item.symbol)
        }
        .tag(item)
    }

    @ViewBuilder
    private func detail(for pane: Pane) -> some View {
        switch pane {
        case .layout: LayoutPane(model: model)
        case .connection: ConnectionPane(model: model)
        case .input: InputPane(model: model)
        case .clipboard: ClipboardPane(model: model)
        case .security: SecurityPane(model: model)
        case .permissions: PermissionsPane(model: model)
        case .advanced: AdvancedPane(model: model)
        case .activity: ActivityPane(model: model)
        }
    }

    private var trustBinding: Binding<PendingTrust?> {
        Binding(get: { model.pendingTrust.first }, set: { _ in })
    }

    @ViewBuilder
    private var actionErrorBar: some View {
        if let notice = model.notice, model.lastActionError == nil {
            Banner(kind: .info, title: "Note", message: notice, actionTitle: "OK") { model.clearNotice() }
                .padding(12)
                .transition(.move(edge: .bottom).combined(with: .opacity))
        }
        if let message = model.lastActionError {
            Banner(kind: .error, title: "Something went wrong", message: message, actionTitle: "Dismiss") {
                model.clearActionError()
            }
            .padding(12)
            .transition(.move(edge: .bottom).combined(with: .opacity))
        }
    }
}


/// Shown while the app waits for the background service, so no action can fail early.
struct ServiceStartingView: View {
    let model: AppModel

    var body: some View {
        VStack(spacing: 12) {
            if case .unavailable(let message) = model.link {
                Image(systemName: "exclamationmark.triangle").font(.system(size: 28)).foregroundStyle(.orange)
                Text("The Openergy service is not running").font(.headline)
                Text(message).font(.callout).foregroundStyle(.secondary).multilineTextAlignment(.center)
            } else {
                ProgressView()
                Text("Starting Openergy…").font(.headline)
            }
        }
        .padding(32)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
