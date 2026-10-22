import SwiftUI

struct LayoutPane: View {
    let model: AppModel

    var body: some View {
        switch model.role {
        case .server: LayoutEditor(model: model)
        case .client: JoinOverview(model: model)
        case .off: OffOverview(model: model)
        }
    }
}

/// Details for the selected screen on the layout canvas.
struct ScreenInspector: View {
    let model: AppModel
    let item: LayoutItem?
    let placed: [LayoutItem]
    var unplace: (LayoutItem) -> Void

    var body: some View {
        if let item {
            Form {
                Section {
                    HStack(spacing: 10) {
                        OSSymbol(os: item.os).font(.title2)
                        VStack(alignment: .leading, spacing: 2) {
                            Text(item.name).font(.headline)
                            Text(item.isLocal ? "This Mac" : OSSymbol.label(for: item.os))
                                .font(.callout).foregroundStyle(.secondary)
                        }
                    }
                    LabeledContent("Status") {
                        HStack(spacing: 6) {
                            StatusDot(tone: item.isOnline ? .good : .neutral, size: 6)
                            Text(item.isOnline ? "Connected" : "Offline")
                        }
                    }
                    LabeledContent("Screen", value: "\(Int(item.size.width)) × \(Int(item.size.height))")
                    LabeledContent("Neighbours", value: neighbours(of: item))
                }
                if let peer = item.peer {
                    Section("Connection") {
                        if let address = peer.address { LabeledContent("Address", value: address) }
                        if let since = peer.sinceUnixMs, peer.connected == true {
                            LabeledContent("Connected", value: RelativeTime.string(unixMs: since))
                        }
                        if let rtt = peer.rttMs { LabeledContent("Latency", value: String(format: "%.1f ms", rtt)) }
                        if let proto = peer.protocol { LabeledContent("Protocol", value: proto.label) }
                    }
                    Section("Security") {
                        LabeledContent("Encryption", value: peer.tls?.enabled == true ? (peer.tls?.version ?? "TLS") : "Off")
                        if let fingerprint = peer.tls?.fingerprint {
                            LabeledContent("Trusted", value: peer.tls?.trusted == true ? "Yes" : "No")
                            FingerprintView(fingerprint: fingerprint, compact: true)
                        }
                    }
                }
                if !item.isLocal, item.rect != nil {
                    Section {
                        Button("Remove from Layout", role: .destructive) { unplace(item) }
                    } footer: {
                        FooterText("The computer stays trusted. It can join again when you place it.")
                    }
                }
            }
            .formStyle(.grouped)
        } else {
            VStack(spacing: 10) {
                Image(systemName: "cursorarrow.click.2")
                    .font(.system(size: 30))
                    .foregroundStyle(.tertiary)
                Text("Select a screen")
                    .font(.headline)
                Text("Drag screens so their edges touch. The pointer moves across touching edges.")
                    .font(.callout)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
            }
            .padding(24)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }

    private func neighbours(of item: LayoutItem) -> String {
        guard let rect = item.rect else { return "Not placed" }
        let tolerance = LayoutGeometry.edgeTolerance
        var names: [String] = []
        for other in placed where other.name != item.name {
            guard let o = other.rect else { continue }
            let sharesY = min(rect.maxY, o.maxY) - max(rect.minY, o.minY) > tolerance
            let sharesX = min(rect.maxX, o.maxX) - max(rect.minX, o.minX) > tolerance
            if abs(rect.maxX - o.minX) <= tolerance, sharesY { names.append("\(other.name) (right)") }
            if abs(o.maxX - rect.minX) <= tolerance, sharesY { names.append("\(other.name) (left)") }
            if abs(rect.maxY - o.minY) <= tolerance, sharesX { names.append("\(other.name) (below)") }
            if abs(o.maxY - rect.minY) <= tolerance, sharesX { names.append("\(other.name) (above)") }
        }
        return names.isEmpty ? "None" : names.joined(separator: ", ")
    }
}

/// Client role: this Mac uses another computer's keyboard and mouse.
struct JoinOverview: View {
    let model: AppModel
    @State private var address = ""

    var body: some View {
        let server = model.connectedPeers.first
        ScrollView {
            VStack(spacing: 28) {
                HStack(spacing: 0) {
                    MachineCard(symbol: "apple.logo", name: model.localName, caption: "This Mac", online: true,
                                highlighted: server == nil || pointerOnThisMac)
                    ConnectionLine(active: server != nil, waiting: model.state == .connecting || model.state == .reconnecting)
                        .frame(width: 140, height: 40)
                    MachineCard(symbol: OSSymbol.name(for: server?.os),
                                name: server?.name ?? serverLabel,
                                caption: server != nil ? "Keyboard and mouse" : "Server",
                                online: server != nil, highlighted: server != nil && !pointerOnThisMac)
                }
                .padding(.top, 36)

                VStack(spacing: 6) {
                    Text(model.headline).font(.title2.weight(.semibold))
                    Text(model.detail).font(.callout).foregroundStyle(.secondary)
                        .multilineTextAlignment(.center)
                }

                if let server {
                    Grid(alignment: .leading, horizontalSpacing: 28, verticalSpacing: 10) {
                        GridRow {
                            stat("Latency", server.rttMs.map { String(format: "%.1f ms", $0) } ?? "—")
                            stat("Encryption", server.tls?.enabled == true ? (server.tls?.version ?? "TLS") : "Off")
                            stat("Protocol", server.protocol?.label ?? "—")
                            stat("Connected", RelativeTime.string(unixMs: server.sinceUnixMs))
                        }
                    }
                    .padding(16)
                    .background(.quaternary.opacity(0.4), in: RoundedRectangle(cornerRadius: 12, style: .continuous))

                    Label("\(server.name) arranges the screens. Change the layout on \(server.name).", systemImage: "info.circle")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                    Button("Disconnect") { model.disconnect() }
                } else {
                    joinForm
                }
            }
            .frame(maxWidth: 640)
            .frame(maxWidth: .infinity)
            .padding(24)
        }
        .background(CanvasBackground().ignoresSafeArea())
        .onAppear { address = model.config.client?.remote ?? "" }
    }

    private var pointerOnThisMac: Bool { model.pointerOnThisMac }

    private var serverLabel: String {
        model.config.client?.remote ?? "Choose a server"
    }

    private func stat(_ title: String, _ value: String) -> some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(title).font(.caption).foregroundStyle(.secondary)
            Text(value).font(.callout.monospacedDigit().weight(.medium))
        }
    }

    private var joinForm: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                TextField("Server address, for example 192.168.1.20", text: $address)
                    .textFieldStyle(.roundedBorder)
                    .onSubmit { model.join(address) }
                Button(model.state == .connecting || model.state == .reconnecting ? "Retry Now" : "Join") {
                    model.join(address)
                }
                .keyboardShortcut(.defaultAction)
                .disabled(address.trimmingCharacters(in: .whitespaces).isEmpty)
            }
            if !model.discovered.isEmpty {
                Text("Nearby").font(.caption.weight(.semibold)).foregroundStyle(.secondary)
                ForEach(model.discovered) { peer in
                    HStack {
                        Image(systemName: "dot.radiowaves.left.and.right").foregroundStyle(Brand.accent)
                        VStack(alignment: .leading, spacing: 1) {
                            Text(peer.name)
                            Text(peer.address).font(.caption).foregroundStyle(.secondary)
                        }
                        Spacer()
                        Button("Join") {
                            address = peer.address
                            model.join(peer.address)
                        }
                    }
                    .padding(.vertical, 2)
                }
            }
        }
        .padding(16)
        .background(.quaternary.opacity(0.4), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }
}

/// A large computer card used in the Join overview.
struct MachineCard: View {
    var symbol: String
    var name: String
    var caption: String
    var online: Bool
    var highlighted: Bool

    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: symbol)
                .font(.system(size: 30))
                .symbolRenderingMode(.hierarchical)
                .foregroundStyle(online ? .primary : .secondary)
            Text(name).font(.headline).lineLimit(1).truncationMode(.middle)
            Text(caption).font(.caption).foregroundStyle(.secondary)
        }
        .frame(width: 190, height: 124)
        .background(.background.opacity(0.7), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .strokeBorder(highlighted ? Brand.accent.opacity(0.7) : Color.primary.opacity(0.12), lineWidth: highlighted ? 1.5 : 1)
        )
        .shadow(color: highlighted ? Brand.accent.opacity(0.25) : .black.opacity(0.12), radius: 12, y: 4)
    }
}

/// The line between two machines. It flows when connected and pulses while connecting.
struct ConnectionLine: View {
    var active: Bool
    var waiting: Bool
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    var body: some View {
        TimelineView(.animation(paused: !(active || waiting) || reduceMotion)) { timeline in
            Canvas { context, size in
                let y = size.height / 2
                var line = Path()
                line.move(to: CGPoint(x: 6, y: y))
                line.addLine(to: CGPoint(x: size.width - 6, y: y))
                let phase = reduceMotion ? 0 : timeline.date.timeIntervalSinceReferenceDate * 28
                if active {
                    context.stroke(line, with: .color(Brand.accent.opacity(0.25)), style: StrokeStyle(lineWidth: 6, lineCap: .round))
                    context.stroke(line, with: .linearGradient(Gradient(colors: [Brand.accentLight, Brand.violet]),
                                                               startPoint: .zero, endPoint: CGPoint(x: size.width, y: 0)),
                                   style: StrokeStyle(lineWidth: 2.5, lineCap: .round, dash: [8, 6], dashPhase: -phase))
                } else {
                    let alpha = waiting ? 0.35 + 0.25 * sin(timeline.date.timeIntervalSinceReferenceDate * 3) : 0.25
                    context.stroke(line, with: .color(.secondary.opacity(alpha)),
                                   style: StrokeStyle(lineWidth: 2, lineCap: .round, dash: [3, 6]))
                }
            }
        }
        .accessibilityHidden(true)
    }
}

/// Role off: choose how to start.
struct OffOverview: View {
    let model: AppModel

    var body: some View {
        VStack(spacing: 26) {
            VStack(spacing: 8) {
                AppIconView(size: 64)
                Text("Share one keyboard and mouse").font(.title2.weight(.semibold))
                Text("Choose what this Mac does. You can change it at any time.")
                    .foregroundStyle(.secondary)
            }
            HStack(spacing: 16) {
                ChoiceCard(symbol: "keyboard", title: "Host",
                           text: "Use this Mac's keyboard and mouse to control other computers.",
                           action: "Host from This Mac") { model.setRole(.server) }
                ChoiceCard(symbol: "display.2", title: "Join",
                           text: "Control this Mac with another computer's keyboard and mouse.",
                           action: "Join a Computer") { model.setRole(.client) }
            }
            if FileManager.default.fileExists(atPath: NSHomeDirectory() + "/Library/Preferences/Synergy/synergy.conf") {
                Button {
                    model.importFromSynergy()
                } label: {
                    Label("Import Settings from Synergy", systemImage: "square.and.arrow.down")
                }
                .buttonStyle(.link)
            }
        }
        .padding(32)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(CanvasBackground().ignoresSafeArea())
    }
}

struct ChoiceCard: View {
    var symbol: String
    var title: String
    var text: String
    var action: String
    var perform: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Image(systemName: symbol)
                .font(.system(size: 26))
                .foregroundStyle(Brand.accent)
                .frame(height: 32, alignment: .leading)
            Text(title).font(.title3.weight(.semibold))
            Text(text).font(.callout).foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 4)
            Button(action, action: perform)
                .buttonStyle(.borderedProminent)
        }
        .padding(18)
        .frame(width: 250, height: 200, alignment: .topLeading)
        .background(.background.opacity(0.75), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(RoundedRectangle(cornerRadius: 14, style: .continuous).strokeBorder(Color.primary.opacity(0.1)))
        .shadow(color: .black.opacity(0.1), radius: 10, y: 4)
    }
}
