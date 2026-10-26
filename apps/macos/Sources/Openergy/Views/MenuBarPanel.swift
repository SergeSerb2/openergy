import SwiftUI

/// The panel that opens from the menu bar icon.
struct MenuBarPanel: View {
    let model: AppModel
    var updater: AppUpdater? = nil
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            header
            if model.role != .off { MiniMap(model: model).frame(height: 78) }
            RolePicker(model: model)
            peers
            attention
            Divider().padding(.vertical, 2)
            HStack {
                Label("Share clipboard", systemImage: "doc.on.clipboard")
                    .font(.callout)
                Spacer()
                Toggle("Share clipboard", isOn: model.bool("clipboard.enabled", { $0.clipboard?.enabled }, default: true))
                    .labelsHidden()
                    .toggleStyle(.switch)
                    .controlSize(.small)
            }
            .padding(.horizontal, 8)
            Divider().padding(.vertical, 2)
            VStack(spacing: 0) {
                MenuRowButton(title: "Open Openergy…", shortcut: "⌘,") { openMain() }
                MenuRowButton(title: "Check for Updates…") { updater?.checkForUpdates() }
                    .disabled(updater?.canCheckForUpdates != true)
                MenuRowButton(title: "Quit Openergy", shortcut: "⌘Q") { NSApp.terminate(nil) }
                    .keyboardShortcut("q")
            }
        }
        .padding(12)
        .frame(width: 320)
        .tint(Brand.accent)
    }

    private var header: some View {
        HStack(spacing: 10) {
            AppIconView(size: 30)
            VStack(alignment: .leading, spacing: 1) {
                Text("Openergy").font(.headline)
                Text(model.headline).font(.callout).foregroundStyle(.secondary).lineLimit(1)
            }
            Spacer()
            StatusDot(tone: model.tone)
                .padding(.trailing, 4)
        }
    }

    @ViewBuilder
    private var peers: some View {
        if model.role == .client, model.connectedPeers.isEmpty {
            HStack(alignment: .top, spacing: 9) {
                StatusDot(tone: model.tone, size: 7).padding(.top, 5)
                VStack(alignment: .leading, spacing: 2) {
                    Text(model.headline).font(.callout.weight(.medium))
                    Text(model.detail).font(.caption).foregroundStyle(.secondary).lineLimit(2)
                }
                Spacer()
                Button("Retry") { model.reconnect() }.controlSize(.small)
            }
            .padding(8)
            .background(.quaternary.opacity(0.5), in: RoundedRectangle(cornerRadius: 9, style: .continuous))
        }
        ForEach(model.connectedPeers) { peer in
            HStack(spacing: 9) {
                StatusDot(tone: .good, size: 7)
                OSSymbol(os: peer.os).foregroundStyle(.secondary)
                VStack(alignment: .leading, spacing: 1) {
                    Text(peer.name).font(.callout.weight(.medium))
                    Text([peer.address?.components(separatedBy: ":").first, peer.tls?.enabled == true ? peer.tls?.version : "Not encrypted"]
                        .compactMap { $0 }.joined(separator: " · "))
                        .font(.caption).foregroundStyle(.secondary)
                }
                Spacer()
                if let rtt = peer.rttMs {
                    Text(String(format: "%.1f ms", rtt))
                        .font(.caption.monospacedDigit())
                        .foregroundStyle(Brand.good)
                }
            }
            .padding(8)
            .background(.quaternary.opacity(0.5), in: RoundedRectangle(cornerRadius: 9, style: .continuous))
        }
        if model.role == .server, model.connectedPeers.isEmpty {
            Text("Waiting for computers to join \(LocalAddresses.ipv4().first ?? "this Mac")")
                .font(.callout)
                .foregroundStyle(.secondary)
                .padding(.horizontal, 6)
        }
    }

    @ViewBuilder
    private var attention: some View {
        if let pending = model.pendingTrust.first {
            Button { openMain() } label: {
                HStack {
                    Image(systemName: "lock.shield").foregroundStyle(.orange)
                    Text("\(pending.name ?? "A computer") wants to connect").font(.callout)
                    Spacer()
                    Text("Review").font(.callout.weight(.semibold)).foregroundStyle(Brand.accent)
                }
                .padding(8)
                .background(Color.orange.opacity(0.12), in: RoundedRectangle(cornerRadius: 9, style: .continuous))
            }
            .buttonStyle(.plain)
        }
        if !model.missingPermissions.isEmpty {
            Button { openMain() } label: {
                HStack {
                    Image(systemName: "hand.raised.fill").foregroundStyle(.orange)
                    Text("Allow \(model.missingPermissions.map(\.title).joined(separator: " and "))").font(.callout)
                    Spacer()
                    Image(systemName: "chevron.right").font(.caption).foregroundStyle(.secondary)
                }
                .padding(8)
                .background(Color.orange.opacity(0.12), in: RoundedRectangle(cornerRadius: 9, style: .continuous))
            }
            .buttonStyle(.plain)
        }
        if case .unavailable(let message) = model.link {
            Text(message).font(.caption).foregroundStyle(.red).padding(.horizontal, 6)
        }
    }

    private func openMain() {
        openWindow(id: WindowID.main)
        NSApp.activate()
    }
}

/// A menu-like row with hover highlight.
struct MenuRowButton: View {
    var title: String
    var shortcut: String?
    var action: () -> Void
    @State private var hovering = false
    @Environment(\.isEnabled) private var isEnabled

    var body: some View {
        Button(action: action) {
            HStack {
                Text(title)
                Spacer()
                if let shortcut { Text(shortcut).foregroundStyle(.secondary) }
            }
            .font(.callout)
            .padding(.horizontal, 8)
            .padding(.vertical, 5)
            .background(hovering && isEnabled ? Color.primary.opacity(0.08) : .clear, in: RoundedRectangle(cornerRadius: 6, style: .continuous))
            .opacity(isEnabled ? 1 : 0.5)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { hovering = $0 }
    }
}

/// A small live picture of where the pointer is.
struct MiniMap: View {
    let model: AppModel

    var body: some View {
        GeometryReader { proxy in
            ZStack(alignment: .topLeading) {
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .fill(.quaternary.opacity(0.5))
                if model.role == .server {
                    serverMap(size: proxy.size)
                } else {
                    clientMap(size: proxy.size)
                }
            }
        }
        .accessibilityElement()
        .accessibilityLabel("Pointer is on \(model.status.activeScreen ?? model.localName)")
    }

    private func serverMap(size: CGSize) -> some View {
        let placed = model.layoutItems.filter { $0.rect != nil }
        let bounds = placed.compactMap(\.rect).reduce(nil) { (partial: CGRect?, rect) in partial?.union(rect) ?? rect }
            ?? CGRect(x: 0, y: 0, width: 1, height: 1)
        let transform = CanvasTransform.fitting(bounds, in: size, padding: 10, maxScale: 1)
        return ZStack(alignment: .topLeading) {
            ForEach(placed) { item in
                let frame = transform.toView(item.rect ?? .zero).insetBy(dx: 1, dy: 1)
                let active = model.status.activeScreen == item.name
                RoundedRectangle(cornerRadius: 4, style: .continuous)
                    .fill(active ? Brand.accent.opacity(0.28) : Color.primary.opacity(0.1))
                    .overlay(RoundedRectangle(cornerRadius: 4, style: .continuous)
                        .strokeBorder(active ? Brand.accent : Color.primary.opacity(0.15)))
                    .overlay {
                        if frame.width > 54, frame.height > 30 {
                            Text(item.name)
                                .font(.system(size: 9, weight: .medium))
                                .foregroundStyle(.secondary)
                                .lineLimit(1)
                                .padding(.horizontal, 4)
                                .frame(maxHeight: .infinity, alignment: .bottom)
                                .padding(.bottom, 3)
                        }
                    }
                    .frame(width: frame.width, height: frame.height)
                    .offset(x: frame.minX, y: frame.minY)
                if active, let x = model.status.cursor?.x, let y = model.status.cursor?.y, item.size.width > 0 {
                    Circle().fill(.white).frame(width: 6, height: 6).shadow(color: Brand.accentLight, radius: 4)
                        .position(x: frame.minX + frame.width * min(max(x / item.size.width, 0.05), 0.95),
                                  y: frame.minY + frame.height * min(max(y / item.size.height, 0.08), 0.92))
                }
            }
        }
    }

    private func clientMap(size: CGSize) -> some View {
        let server = model.connectedPeers.first
        let onThisMac = model.pointerOnThisMac
        return HStack(spacing: 10) {
            miniScreen(name: "This Mac", symbol: "apple.logo", active: server != nil && onThisMac)
            ConnectionLine(active: server != nil, waiting: server == nil).frame(width: 46, height: 14)
            miniScreen(name: server?.name ?? "Server", symbol: OSSymbol.name(for: server?.os),
                       active: server != nil && !onThisMac)
        }
        .frame(width: size.width, height: size.height)
    }

    private func miniScreen(name: String, symbol: String, active: Bool) -> some View {
        VStack(spacing: 3) {
            Image(systemName: symbol).font(.system(size: 14))
            Text(name).font(.caption2).lineLimit(1)
        }
        .frame(width: 92, height: 52)
        .background(active ? Brand.accent.opacity(0.25) : Color.primary.opacity(0.08), in: RoundedRectangle(cornerRadius: 6, style: .continuous))
        .overlay(RoundedRectangle(cornerRadius: 6, style: .continuous).strokeBorder(active ? Brand.accent : Color.primary.opacity(0.12)))
    }
}
