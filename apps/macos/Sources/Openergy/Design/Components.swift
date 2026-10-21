import AppKit
import SwiftUI

/// A status light. Waiting states breathe slowly unless Reduce Motion is on.
struct StatusDot: View {
    var tone: AppModel.Tone
    var size: CGFloat = 8
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var breathe = false

    var body: some View {
        Circle()
            .fill(tone.color)
            .frame(width: size, height: size)
            .background(
                Circle()
                    .fill(tone.color.opacity(0.25))
                    .frame(width: size * 2.1, height: size * 2.1)
                    .opacity(tone == .neutral ? 0 : 1)
                    .scaleEffect(breathe ? 1 : 0.7)
            )
            .onAppear(perform: animate)
            .onChange(of: tone) { animate() }
            .accessibilityHidden(true)
    }

    private func animate() {
        guard !reduceMotion, tone == .waiting else {
            breathe = true
            return
        }
        breathe = false
        withAnimation(.easeInOut(duration: 1.1).repeatForever(autoreverses: true)) { breathe = true }
    }
}

/// Dot plus headline, used in the toolbar.
struct StatusPill: View {
    let model: AppModel

    var body: some View {
        HStack(spacing: 7) {
            StatusDot(tone: model.tone, size: 7)
            Text(model.headline)
                .font(.callout.weight(.medium))
                .lineLimit(1)
            if let rtt = model.connectedPeers.first?.rttMs, model.role == .client {
                Text(String(format: "%.1f ms", rtt))
                    .font(.callout.monospacedDigit())
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 4)
        .background(model.tone.color.opacity(0.12), in: Capsule())
        .accessibilityElement(children: .combine)
    }
}

/// SF Symbol for a peer's operating system.
struct OSSymbol: View {
    var os: String?
    var body: some View {
        Image(systemName: Self.name(for: os))
            .symbolRenderingMode(.hierarchical)
    }

    static func name(for os: String?) -> String {
        switch os?.lowercased() {
        case "macos", "mac", "darwin": "apple.logo"
        case "windows", "win32": "pc"
        case "linux": "server.rack"
        default: "desktopcomputer"
        }
    }

    static func label(for os: String?) -> String {
        switch os?.lowercased() {
        case "macos", "mac", "darwin": "macOS"
        case "windows", "win32": "Windows"
        case "linux": "Linux"
        default: "Computer"
        }
    }
}

/// A SHA-256 fingerprint shown in readable groups, with a copy button.
struct FingerprintView: View {
    let fingerprint: String
    var compact = false
    @State private var copied = false

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Text(display)
                .font(.system(compact ? .caption : .callout, design: .monospaced))
                .foregroundStyle(.secondary)
                .textSelection(.enabled)
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 0)
            Button {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(fingerprint, forType: .string)
                copied = true
                Task { try? await Task.sleep(for: .seconds(1.5)); copied = false }
            } label: {
                Image(systemName: copied ? "checkmark" : "doc.on.doc")
                    .contentTransition(.symbolEffect(.replace))
            }
            .buttonStyle(.borderless)
            .help("Copy fingerprint")
            .accessibilityLabel("Copy fingerprint")
        }
    }

    /// "AB:CD:EF:…" becomes "ABCD EFGH …" in two lines of eight groups for easy comparison.
    private var display: String {
        let hex = fingerprint.replacingOccurrences(of: ":", with: "").uppercased()
        guard !compact else {
            return hex.count > 16 ? "\(hex.prefix(8)) … \(hex.suffix(8))" : hex
        }
        let groups = stride(from: 0, to: hex.count, by: 4).map { offset -> String in
            let start = hex.index(hex.startIndex, offsetBy: offset)
            let end = hex.index(start, offsetBy: 4, limitedBy: hex.endIndex) ?? hex.endIndex
            return String(hex[start..<end])
        }
        return stride(from: 0, to: groups.count, by: 8)
            .map { groups[$0..<min($0 + 8, groups.count)].joined(separator: " ") }
            .joined(separator: "\n")
    }
}

/// Off / Host / Join control.
struct RolePicker: View {
    let model: AppModel
    var labelsHidden = true

    var body: some View {
        Picker("Role", selection: Binding(get: { model.role }, set: { model.setRole($0) })) {
            ForEach(Role.allCases) { role in
                Text(role.title).tag(role)
            }
        }
        .pickerStyle(.segmented)
        .labelsHidden()
        .help("Off: not sharing. Host: share this Mac's keyboard and mouse. Join: use another computer's.")
    }
}

/// Formats byte counts the way Finder does.
enum ByteText {
    static func string(_ bytes: Double?) -> String {
        guard let bytes else { return "—" }
        return ByteCountFormatter.string(fromByteCount: Int64(bytes), countStyle: .memory)
    }
}

enum RelativeTime {
    static func string(unixMs: Double?) -> String {
        guard let unixMs else { return "" }
        let date = Date(timeIntervalSince1970: unixMs / 1000)
        return date.formatted(.relative(presentation: .named))
    }
}

/// Inline banner for warnings with one action.
struct Banner: View {
    enum Kind { case warning, error, info }
    var kind: Kind
    var title: String
    var message: String?
    var actionTitle: String?
    var action: (() -> Void)?

    var body: some View {
        HStack(alignment: .center, spacing: 12) {
            Image(systemName: symbol)
                .font(.title3)
                .foregroundStyle(color)
            VStack(alignment: .leading, spacing: 2) {
                Text(title).font(.callout.weight(.semibold))
                if let message {
                    Text(message).font(.callout).foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
            Spacer(minLength: 8)
            if let actionTitle, let action {
                Button(actionTitle, action: action)
                    .controlSize(.regular)
            }
        }
        .padding(12)
        .background(color.opacity(0.1), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        .overlay(RoundedRectangle(cornerRadius: 10, style: .continuous).strokeBorder(color.opacity(0.25)))
    }

    private var symbol: String {
        switch kind {
        case .warning: "exclamationmark.triangle.fill"
        case .error: "xmark.octagon.fill"
        case .info: "info.circle.fill"
        }
    }

    private var color: Color {
        switch kind {
        case .warning: .orange
        case .error: .red
        case .info: Brand.accent
        }
    }
}
