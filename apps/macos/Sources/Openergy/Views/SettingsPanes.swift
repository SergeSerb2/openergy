import Darwin
import SwiftUI

// MARK: Bindings into the daemon config

extension AppModel {
    func bool(_ path: String, _ get: @escaping (OpenergyConfig) -> Bool?, default fallback: Bool) -> Binding<Bool> {
        Binding(get: { get(self.config) ?? fallback }, set: { self.patch(path, .bool($0)) })
    }

    func number(_ path: String, _ get: @escaping (OpenergyConfig) -> Double?, default fallback: Double) -> Binding<Double> {
        Binding(get: { get(self.config) ?? fallback }, set: { self.patch(path, .number($0)) })
    }
}

// MARK: Connection

struct ConnectionPane: View {
    let model: AppModel
    @State private var name = ""
    @State private var address = ""
    @State private var port = ""

    var body: some View {
        Form {
            Section {
                LabeledContent("Mode") {
                    Text("\(model.role.title) · \(model.role.summary)").foregroundStyle(.secondary)
                }
                TextField("Computer name", text: $name, prompt: Text(AppModel.computerName))
                    .onSubmit { model.renameLocal(to: name) }
            } header: {
                Text("This Mac")
            } footer: {
                FooterText("Other computers see this Mac by its name. With Synergy, the name must match the screen name in the server's layout.")
            }

            switch model.role {
            case .client: joinSection
            case .server: hostSection
            case .off: EmptyView()
            }

            Section {
                Button {
                    model.importFromSynergy()
                } label: {
                    Label("Import Settings from Synergy…", systemImage: "square.and.arrow.down")
                }
            } header: {
                Text("Moving from Synergy")
            } footer: {
                FooterText("Reads the layout, server address, clipboard and security settings from Synergy 3. Your Synergy files stay unchanged.")
            }
        }
        .formStyle(.grouped)
        .onAppear(perform: load)
        .onChange(of: model.config) { load() }
    }

    private func load() {
        name = model.config.name ?? model.status.name ?? ""
        address = model.config.client?.remote ?? ""
        port = String(model.config.server?.port ?? 24800)
    }

    private var joinSection: some View {
        Section {
            HStack {
                TextField("Server address", text: $address, prompt: Text("192.168.1.20 or name.local"))
                    .onSubmit { model.join(address) }
                Button("Join") { model.join(address) }
                    .disabled(address.trimmingCharacters(in: .whitespaces).isEmpty)
            }
            LabeledContent("Status") {
                HStack(spacing: 6) {
                    StatusDot(tone: model.tone, size: 6)
                    Text(model.headline)
                }
            }
            if let error = model.status.lastError?.message, model.state != .connected {
                Text(error).font(.callout).foregroundStyle(.secondary)
            }
            Toggle("Connect automatically", isOn: model.bool("client.autoconnect", { $0.client?.autoconnect }, default: true))
            HStack {
                Button("Reconnect") { model.reconnect() }
                Button("Disconnect") { model.disconnect() }
                    .disabled(model.connectedPeers.isEmpty)
            }
            if !model.discovered.isEmpty {
                ForEach(model.discovered) { peer in
                    HStack {
                        Image(systemName: "dot.radiowaves.left.and.right").foregroundStyle(Brand.accent)
                        VStack(alignment: .leading) {
                            Text(peer.name)
                            Text(peer.address).font(.caption).foregroundStyle(.secondary)
                        }
                        Spacer()
                        if model.connectedPeers.contains(where: { $0.name == peer.name }) {
                            Text("Connected").foregroundStyle(.secondary)
                        } else {
                            Button("Join") {
                                address = peer.address
                                model.join(peer.address)
                            }
                        }
                    }
                }
            }
        } header: {
            Text("Server")
        } footer: {
            FooterText("The server is the computer with the keyboard and mouse. Openergy works with Synergy, Deskflow and Openergy servers.")
        }
    }

    private var hostSection: some View {
        Section {
            TextField("Port", text: $port)
                .onSubmit {
                    if let value = Int(port), (1...65535).contains(value) { model.patch("server.port", .number(Double(value))) }
                }
            LabeledContent("Join with") {
                VStack(alignment: .trailing, spacing: 2) {
                    let port = model.config.server?.port ?? 24800
                    let addresses = LocalAddresses.joinable()
                    if addresses.isEmpty {
                        Text("Not connected to a network").foregroundStyle(.secondary)
                    }
                    ForEach(addresses, id: \.self) { ip in
                        Text(port == 24800 ? ip : "\(ip):\(port)").textSelection(.enabled).font(.body.monospacedDigit())
                    }
                }
            }
            LabeledContent("Status") {
                HStack(spacing: 6) {
                    StatusDot(tone: model.tone, size: 6)
                    Text(model.headline)
                }
            }
            ForEach(model.connectedPeers) { peer in
                LabeledContent(peer.name) {
                    Text([peer.address, peer.rttMs.map { String(format: "%.1f ms", $0) }].compactMap { $0 }.joined(separator: " · "))
                        .foregroundStyle(.secondary)
                }
            }
        } header: {
            Text("Hosting")
        } footer: {
            FooterText("On the other computer, choose Join and enter one of these addresses. Synergy and Deskflow clients can connect too.")
        }
    }
}

enum LocalAddresses {
    /// Addresses another computer on the LAN can use: Wi-Fi and Ethernet (en*), not bridges,
    /// VPN tunnels or virtual machine networks.
    static func joinable() -> [String] {
        ipv4(interfaces: { $0.hasPrefix("en") })
    }

    static func ipv4(interfaces include: (String) -> Bool = { _ in true }) -> [String] {
        var result: [String] = []
        var pointer: UnsafeMutablePointer<ifaddrs>?
        guard getifaddrs(&pointer) == 0, let first = pointer else { return [] }
        defer { freeifaddrs(pointer) }
        for entry in sequence(first: first, next: { $0.pointee.ifa_next }) {
            let flags = Int32(entry.pointee.ifa_flags)
            guard let address = entry.pointee.ifa_addr, address.pointee.sa_family == UInt8(AF_INET),
                  flags & IFF_UP != 0, flags & IFF_LOOPBACK == 0,
                  include(String(cString: entry.pointee.ifa_name)) else { continue }
            var host = [CChar](repeating: 0, count: Int(NI_MAXHOST))
            if getnameinfo(address, socklen_t(address.pointee.sa_len), &host, socklen_t(host.count), nil, 0, NI_NUMERICHOST) == 0 {
                result.append(String(decoding: host.prefix { $0 != 0 }.map { UInt8(bitPattern: $0) }, as: UTF8.self))
            }
        }
        return result
    }
}

// MARK: Keyboard & Mouse

struct InputPane: View {
    let model: AppModel

    var body: some View {
        Form {
            Section("Pointer") {
                LabeledContent("Scroll speed") {
                    HStack {
                        Image(systemName: "tortoise").foregroundStyle(.secondary)
                        CommitSlider(value: model.config.input?.scrollSensitivity ?? 1, range: 0.25...3, step: 0.25) {
                            model.patch("input.scroll_sensitivity", .number($0))
                        }
                        .frame(width: 200)
                        Image(systemName: "hare").foregroundStyle(.secondary)
                    }
                }
                Toggle("Reverse scroll direction", isOn: model.bool("input.invert_scroll_y", { $0.input?.invertScrollY }, default: false))
                Toggle(isOn: model.bool("input.relative_mouse", { $0.input?.relativeMouse }, default: false)) {
                    Text("Relative mouse movement")
                    Text("Sends motion instead of positions. Use it for games and 3D apps.")
                }
            }

            Section("Keyboard") {
                Toggle(isOn: model.bool("input.sync_language", { $0.input?.syncLanguage }, default: true)) {
                    Text("Match keyboard layout")
                    Text("Switches this Mac's input source to the language used on the other computer.")
                }
            }

            Section {
                ValuePicker(title: "Wait at the edge",
                            value: model.number("switch.delay_ms", { $0.switch?.delayMs }, default: 0),
                            options: [("Off", 0), ("150 ms", 150), ("250 ms", 250), ("500 ms", 500)],
                            custom: { "\(Int($0)) ms" })
                ValuePicker(title: "Double-tap the edge",
                            value: model.number("switch.double_tap_ms", { $0.switch?.doubleTapMs }, default: 0),
                            options: [("Off", 0), ("Within 250 ms", 250), ("Within 400 ms", 400)],
                            custom: { "Within \(Int($0)) ms" })
                LabeledContent("Blocked corners") {
                    DeadCornersPicker(model: model)
                }
            } header: {
                Text("Switching Screens")
            } footer: {
                FooterText("These settings stop the pointer from leaving a screen by accident. They apply when this Mac hosts.")
            }

            Section("While Sharing") {
                Toggle(isOn: model.bool("input.prevent_sleep", { $0.input?.preventSleep }, default: false)) {
                    Text("Keep displays awake")
                    Text("Stops this Mac's display from sleeping while another computer uses it.")
                }
                if let hotkey = model.config.hotkeys?.lockToScreen {
                    LabeledContent("Lock pointer to screen", value: HotkeyText.display(hotkey))
                }
            }
        }
        .formStyle(.grouped)
    }
}

/// Four corner toggles drawn as a small screen.
struct DeadCornersPicker: View {
    let model: AppModel
    private let corners = [("top_left", Alignment.topLeading), ("top_right", .topTrailing),
                           ("bottom_left", .bottomLeading), ("bottom_right", .bottomTrailing)]

    var body: some View {
        let selected = Set(model.config.switch?.deadCorners ?? [])
        ZStack {
            RoundedRectangle(cornerRadius: 6, style: .continuous)
                .strokeBorder(Color.primary.opacity(0.25))
                .frame(width: 96, height: 60)
            ForEach(corners, id: \.0) { corner, alignment in
                Button {
                    var next = selected
                    if next.contains(corner) { next.remove(corner) } else { next.insert(corner) }
                    model.patch("switch.dead_corners", .array(next.sorted().map { .string($0) }))
                } label: {
                    RoundedRectangle(cornerRadius: 3, style: .continuous)
                        .fill(selected.contains(corner) ? Brand.accent : Color.primary.opacity(0.12))
                        .frame(width: 18, height: 14)
                }
                .buttonStyle(.plain)
                .frame(width: 96, height: 60, alignment: alignment)
                .padding(3)
                .accessibilityLabel(corner.replacingOccurrences(of: "_", with: " ").capitalized)
                .accessibilityAddTraits(selected.contains(corner) ? .isSelected : [])
            }
        }
        .frame(width: 104, height: 68)
        .help("Click a corner to stop the pointer from switching screens there")
    }
}

enum HotkeyText {
    static func display(_ value: String) -> String {
        value.split(separator: "+").map { part -> String in
            switch part.lowercased() {
            case "ctrl", "control": "⌃"
            case "alt", "option": "⌥"
            case "shift": "⇧"
            case "cmd", "command", "super", "meta": "⌘"
            default: part.uppercased()
            }
        }.joined()
    }
}

// MARK: Clipboard

struct ClipboardPane: View {
    let model: AppModel

    var body: some View {
        Form {
            Section {
                Toggle(isOn: model.bool("clipboard.enabled", { $0.clipboard?.enabled }, default: true)) {
                    Text("Share clipboard")
                    Text("Copy on one computer and paste on another. Text, rich text and images.")
                }
                ValuePicker(title: "Largest item",
                            value: model.number("clipboard.max_bytes", { $0.clipboard?.maxBytes }, default: 64 * 1024 * 1024),
                            options: [("4 MB", 4 * 1024 * 1024), ("16 MB", 16 * 1024 * 1024),
                                      ("64 MB", 64 * 1024 * 1024), ("256 MB", 256 * 1024 * 1024)],
                            custom: { ByteText.string($0) })
                    .disabled(model.config.clipboard?.enabled == false)
            } footer: {
                FooterText("Large items move in the background, so typing and the pointer never wait. Synergy computers accept at most their own limit, often 4 MB.")
            }

            Section("Recent Activity") {
                if let transfer = model.status.clipboard?.lastTransfer {
                    LabeledContent(transfer.direction == "in" ? "Last received" : "Last sent") {
                        Text("\(ByteText.string(transfer.bytes)) · \((transfer.formats ?? []).joined(separator: ", ")) · \(RelativeTime.string(unixMs: transfer.unixMs))")
                            .foregroundStyle(.secondary)
                    }
                } else {
                    Text("Nothing shared yet").foregroundStyle(.secondary)
                }
                if let blocked = model.status.clipboard?.lastBlocked {
                    Banner(kind: .warning,
                           title: "Not sent: \(ByteText.string(blocked.bytes)) is over \(blocked.peer ?? "the other computer")'s limit",
                           message: "\(blocked.peer ?? "The other computer") accepts up to \(ByteText.string(blocked.limit)). Raise the limit in its settings to share larger items.")
                }
            }
        }
        .formStyle(.grouped)
    }
}

// MARK: Security

struct SecurityPane: View {
    let model: AppModel

    var body: some View {
        Form {
            if !model.pendingTrust.isEmpty {
                Section("Waiting for Approval") {
                    ForEach(model.pendingTrust) { pending in
                        VStack(alignment: .leading, spacing: 8) {
                            Text("\(pending.name ?? pending.address ?? "A computer") wants to connect").font(.headline)
                            FingerprintView(fingerprint: pending.fingerprint)
                            HStack {
                                Button("Reject", role: .destructive) { model.reject(pending.fingerprint) }
                                Button("Trust") { model.approve(pending.fingerprint) }.buttonStyle(.borderedProminent)
                            }
                        }
                        .padding(.vertical, 4)
                    }
                }
            }

            Section {
                Toggle(isOn: model.bool("tls.enabled", { $0.tls?.enabled }, default: true)) {
                    Text("Encrypt connections")
                    Text("Uses TLS. Keep it on unless the other computer cannot use encryption.")
                }
                if let fingerprint = model.status.tls?.fingerprint {
                    VStack(alignment: .leading, spacing: 6) {
                        Text("This Mac's fingerprint")
                        FingerprintView(fingerprint: fingerprint)
                    }
                }
            } footer: {
                FooterText("A fingerprint identifies a computer. When a computer connects for the first time, compare its fingerprint on both screens before you trust it.")
            }

            Section("Trusted Computers") {
                let servers = model.trusted.servers ?? []
                let clients = model.trusted.clients ?? []
                if servers.isEmpty && clients.isEmpty {
                    Text("No trusted computers yet").foregroundStyle(.secondary)
                }
                ForEach(servers, id: \.self) { fingerprint in
                    trustedRow(fingerprint, kind: "Server")
                }
                ForEach(clients, id: \.self) { fingerprint in
                    trustedRow(fingerprint, kind: "Client")
                }
            }
        }
        .formStyle(.grouped)
    }

    private func trustedRow(_ fingerprint: String, kind: String) -> some View {
        let peer = (model.status.peers ?? []).first { $0.tls?.fingerprint?.uppercased() == fingerprint.uppercased() }
        return HStack(alignment: .top) {
            VStack(alignment: .leading, spacing: 4) {
                Text(peer?.name ?? kind).font(.headline)
                FingerprintView(fingerprint: fingerprint, compact: true)
            }
            Spacer()
            Button("Remove", role: .destructive) { model.revoke(fingerprint) }
        }
    }
}

// MARK: Permissions

struct PermissionsPane: View {
    let model: AppModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text("Let Openergy use your keyboard and mouse").font(.title2.weight(.semibold))
                Text("macOS asks you to allow two things. Openergy checks them every few seconds, so this page updates when you switch each one on.")
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
                ForEach(PermissionKind.allCases) { kind in
                    PermissionRow(model: model, kind: kind)
                }
                Text("Join needs Accessibility. Host needs both.")
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }
            .padding(28)
            .frame(maxWidth: 620, alignment: .leading)
            .frame(maxWidth: .infinity)
        }
    }
}

struct PermissionRow: View {
    let model: AppModel
    let kind: PermissionKind

    var body: some View {
        let state = state
        HStack(spacing: 14) {
            Image(systemName: kind == .accessibility ? "accessibility" : "keyboard")
                .font(.system(size: 18))
                .foregroundStyle(state.isSatisfied ? Brand.good : .orange)
                .frame(width: 36, height: 36)
                .background((state.isSatisfied ? Brand.good : Color.orange).opacity(0.14),
                            in: RoundedRectangle(cornerRadius: 9, style: .continuous))
            VStack(alignment: .leading, spacing: 2) {
                Text(kind.title).font(.headline)
                Text(kind.purpose).font(.callout).foregroundStyle(.secondary)
            }
            Spacer()
            if state.isSatisfied {
                Label("Allowed", systemImage: "checkmark.circle.fill")
                    .foregroundStyle(Brand.good)
                    .font(.callout.weight(.semibold))
            } else {
                Button("Open System Settings") { model.requestPermission(kind) }
                    .buttonStyle(.borderedProminent)
            }
        }
        .padding(14)
        .background(.quaternary.opacity(0.45), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private var state: PermissionState {
        switch kind {
        case .accessibility: model.status.permissions?.accessibility ?? .unknown
        case .inputMonitoring: model.status.permissions?.inputMonitoring ?? .unknown
        }
    }
}

// MARK: Advanced

struct AdvancedPane: View {
    let model: AppModel

    var body: some View {
        Form {
            Section {
                Picker("Click handling", selection: Binding(
                    get: { model.config.macos?.clickNumbering ?? "native" },
                    set: { model.patch("macos.click_numbering", .string($0)) }
                )) {
                    Text("Numbered clicks (recommended)").tag("native")
                    Text("Legacy, like Synergy").tag("legacy")
                }
            } header: {
                Text("Window Focus")
            } footer: {
                FooterText("macOS 27 brings a window to the front only when each click carries its own number. Numbered clicks fix the Synergy bug where a click in a window did not activate it.")
            }

            Section("General") {
                Toggle("Open at login", isOn: Binding(get: { model.launchAtLogin }, set: { model.setLaunchAtLogin($0) }))
                    .onAppear { model.refreshLoginItem() }
                if model.launchAtLoginNeedsApproval {
                    Text("Allow Openergy in System Settings → General → Login Items.")
                        .font(.callout)
                        .foregroundStyle(.orange)
                }
                Button("Show Configuration File in Finder") { model.revealConfigFile() }
            }

            Section("Service") {
                LabeledContent("Version", value: model.status.daemon?.version ?? "—")
                LabeledContent("Memory", value: ByteText.string(model.status.stats?.rssBytes))
                if let uptime = model.status.daemon?.uptimeS {
                    LabeledContent("Running for", value: Duration.seconds(uptime).formatted(.units(allowed: [.hours, .minutes, .seconds], width: .abbreviated)))
                }
                if let pid = model.status.daemon?.pid {
                    LabeledContent("Process", value: "openergy (\(pid))")
                }
                Button("Restart Connection") { model.reconnect() }
            }
        }
        .formStyle(.grouped)
    }
}

// MARK: Activity

struct ActivityPane: View {
    let model: AppModel
    @State private var filter = ""
    @State private var warningsOnly = false

    var body: some View {
        let lines = model.logLines.map(LogLine.init).filter { line in
            (filter.isEmpty || line.text.localizedCaseInsensitiveContains(filter))
                && (!warningsOnly || line.level == .warning || line.level == .error)
        }
        VStack(spacing: 0) {
            HStack {
                TextField("Filter", text: $filter)
                    .textFieldStyle(.roundedBorder)
                    .frame(maxWidth: 260)
                Toggle("Warnings and errors only", isOn: $warningsOnly)
                    .toggleStyle(.checkbox)
                Spacer()
                Button {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(lines.map(\.display).joined(separator: "\n"), forType: .string)
                } label: {
                    Label("Copy", systemImage: "doc.on.doc")
                }
            }
            .padding(10)
            Divider()
            ScrollViewReader { proxy in
                List(Array(lines.enumerated()), id: \.offset) { index, line in
                    Text(line.display)
                        .font(.system(.caption, design: .monospaced))
                        .foregroundStyle(line.level.color)
                        .textSelection(.enabled)
                        .id(index)
                }
                .listStyle(.plain)
                .onChange(of: lines.count) { _, count in
                    if count > 0 { proxy.scrollTo(count - 1, anchor: .bottom) }
                }
            }
        }
        .onAppear { model.subscribeLogs(true) }
        .onDisappear { model.subscribeLogs(false) }
    }

}

/// One daemon log line, e.g. "1792621985143 INFO openergy::runtime: connected".
struct LogLine {
    enum Level {
        case error, warning, info, debug
        var color: Color {
            switch self {
            case .error: .red
            case .warning: .orange
            case .info: .primary
            case .debug: .secondary
            }
        }
    }

    let text: String
    let level: Level
    let display: String

    init(_ text: String) {
        self.text = text
        let parts = text.split(separator: " ", maxSplits: 2, omittingEmptySubsequences: true)
        let token = parts.count > 1 ? parts[1].uppercased() : ""
        let level: Level = switch token {
        case "ERROR": .error
        case "WARN", "WARNING": .warning
        case "DEBUG", "TRACE": .debug
        default: text.range(of: " error ", options: .caseInsensitive) != nil ? .error
            : (text.range(of: " warn", options: .caseInsensitive) != nil ? .warning : .info)
        }
        self.level = level
        // Daemon lines start with Unix milliseconds; show local time instead.
        if let first = parts.first, let ms = Double(first), ms > 1_000_000_000_000, parts.count == 3 {
            let time = Date(timeIntervalSince1970: ms / 1000).formatted(.dateTime.hour().minute().second())
            display = "\(time) \(parts[1]) \(parts[2])"
        } else {
            display = text
        }
    }
}

// MARK: Trust approval

struct TrustApprovalSheet: View {
    let model: AppModel
    let pending: PendingTrust

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack(spacing: 12) {
                Image(systemName: "lock.shield")
                    .font(.system(size: 28))
                    .foregroundStyle(Brand.accent)
                VStack(alignment: .leading, spacing: 2) {
                    Text("Trust \(pending.name ?? "this computer")?").font(.title3.weight(.semibold))
                    if let address = pending.address {
                        Text(address).foregroundStyle(.secondary)
                    }
                }
            }
            Text("Compare this fingerprint with the one shown on the other computer. Trust it only if they match.")
                .fixedSize(horizontal: false, vertical: true)
            FingerprintView(fingerprint: pending.fingerprint)
                .padding(12)
                .background(.quaternary.opacity(0.5), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
            HStack {
                Spacer()
                Button("Don't Trust", role: .cancel) { model.reject(pending.fingerprint) }
                    .keyboardShortcut(.cancelAction)
                // No default (Return) shortcut: trusting must be a deliberate click.
                Button("Trust") { model.approve(pending.fingerprint) }
            }
        }
        .padding(24)
        .frame(width: 460)
    }
}

/// Form section footer, aligned with the section content.
struct FooterText: View {
    var text: String
    init(_ text: String) { self.text = text }
    var body: some View {
        Text(text)
            .multilineTextAlignment(.leading)
            .frame(maxWidth: .infinity, alignment: .leading)
    }
}


/// A picker for numeric settings. A value set elsewhere (for example by a Synergy import)
/// that matches no option is shown as its own entry instead of a blank picker.
struct ValuePicker: View {
    let title: String
    @Binding var value: Double
    let options: [(String, Double)]
    let custom: (Double) -> String

    var body: some View {
        Picker(title, selection: $value) {
            ForEach(options, id: \.1) { label, option in
                Text(label).tag(option)
            }
            if !options.contains(where: { $0.1 == value }) {
                Text(custom(value)).tag(value)
            }
        }
    }
}

/// A slider that saves once, when the drag ends, so responses for older values cannot pull
/// the knob back while it moves.
struct CommitSlider: View {
    let value: Double
    let range: ClosedRange<Double>
    let step: Double
    let commit: (Double) -> Void
    @State private var draft: Double?

    var body: some View {
        Slider(value: Binding(get: { draft ?? value }, set: { draft = $0 }), in: range, step: step) { editing in
            if !editing, let final = draft {
                commit(final)
                draft = nil
            }
        }
    }
}
