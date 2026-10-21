import AppKit
import SwiftUI

enum Brand {
    /// Fuchsia from the app icon. Used for controls, links and the active screen.
    static let accent = Color(red: 0xC0 / 255, green: 0x26 / 255, blue: 0xD3 / 255)
    static let accentLight = Color(red: 0xF0 / 255, green: 0xAB / 255, blue: 0xFC / 255)
    static let violet = Color(red: 0x93 / 255, green: 0x33 / 255, blue: 0xEA / 255)
    static let deep = Color(red: 0x4C / 255, green: 0x1D / 255, blue: 0x95 / 255)

    static let iconGradient = LinearGradient(
        stops: [
            .init(color: accentLight, location: 0),
            .init(color: Color(red: 0xD9 / 255, green: 0x46 / 255, blue: 0xEF / 255), location: 0.35),
            .init(color: violet, location: 0.75),
            .init(color: deep, location: 1),
        ],
        startPoint: .topLeading,
        endPoint: .bottomTrailing
    )

    static let linkGradient = LinearGradient(
        colors: [Color(red: 0xE8 / 255, green: 0x79 / 255, blue: 0xF9 / 255), Color(red: 0x8B / 255, green: 0x5C / 255, blue: 0xF6 / 255)],
        startPoint: .top,
        endPoint: .bottom
    )

    static let good = Color(nsColor: .systemGreen)
    static let waiting = Color(nsColor: .systemOrange)
    static let bad = Color(nsColor: .systemRed)
}

extension AppModel.Tone {
    var color: Color {
        switch self {
        case .good: Brand.good
        case .waiting: Brand.waiting
        case .warning: Color(nsColor: .systemYellow)
        case .bad: Brand.bad
        case .neutral: Color.secondary
        }
    }
}

/// The Openergy mark: two half rings (two computers) around one core (one keyboard and mouse).
struct OpenergyGlyph: View {
    var left: Double = 1
    var right: Double = 0.5
    var core: Double = 1
    var color: Color = .white

    var body: some View {
        Canvas { context, size in
            let side = min(size.width, size.height)
            let center = CGPoint(x: size.width / 2, y: size.height / 2)
            let radius = side * 0.36
            let stroke = side * 0.14
            let style = StrokeStyle(lineWidth: stroke, lineCap: .round)

            var leftArc = Path()
            leftArc.addArc(center: center, radius: radius, startAngle: .degrees(-100), endAngle: .degrees(100), clockwise: true)
            context.stroke(leftArc, with: .color(color.opacity(left)), style: style)

            var rightArc = Path()
            rightArc.addArc(center: center, radius: radius, startAngle: .degrees(-80), endAngle: .degrees(80), clockwise: false)
            context.stroke(rightArc, with: .color(color.opacity(right)), style: style)

            let dot = side * 0.07
            context.fill(Path(ellipseIn: CGRect(x: center.x - dot, y: center.y - dot, width: dot * 2, height: dot * 2)),
                         with: .color(color.opacity(core)))
        }
    }
}

/// The app icon drawn in SwiftUI (used in the menu panel header and About).
struct AppIconView: View {
    var size: CGFloat = 64

    var body: some View {
        RoundedRectangle(cornerRadius: size * 0.225, style: .continuous)
            .fill(Brand.iconGradient)
            .overlay(
                RoundedRectangle(cornerRadius: size * 0.225, style: .continuous)
                    .fill(RadialGradient(colors: [.white.opacity(0.4), .clear], center: UnitPoint(x: 0.25, y: 0.15),
                                         startRadius: 0, endRadius: size))
            )
            .overlay(OpenergyGlyph(left: 1, right: 0.5, core: 1).padding(size * 0.14))
            .frame(width: size, height: size)
    }
}

enum MenuBarIcon {
    /// Template image for the status item. Half brightness shows state:
    /// both halves = connected, right half dim = waiting, both dim = off.
    static func image(for tone: AppModel.Tone, role: Role, headline: String = "") -> NSImage {
        let left: CGFloat = role == .off ? 0.35 : 1
        let right: CGFloat
        switch (role, tone) {
        case (.off, _): right = 0.35
        case (_, .good): right = 1
        default: right = 0.4
        }
        let image = NSImage(size: NSSize(width: 18, height: 18), flipped: false) { rect in
            let center = NSPoint(x: rect.midX, y: rect.midY)
            let radius: CGFloat = 6.2
            let width: CGFloat = 2.3

            let leftArc = NSBezierPath()
            leftArc.appendArc(withCenter: center, radius: radius, startAngle: 100, endAngle: 260, clockwise: false)
            leftArc.lineWidth = width
            leftArc.lineCapStyle = .round
            NSColor.black.withAlphaComponent(left).setStroke()
            leftArc.stroke()

            let rightArc = NSBezierPath()
            rightArc.appendArc(withCenter: center, radius: radius, startAngle: 80, endAngle: -80, clockwise: true)
            rightArc.lineWidth = width
            rightArc.lineCapStyle = .round
            NSColor.black.withAlphaComponent(right).setStroke()
            rightArc.stroke()

            let dot: CGFloat = 1.5
            NSColor.black.withAlphaComponent(left).setFill()
            NSBezierPath(ovalIn: NSRect(x: center.x - dot, y: center.y - dot, width: dot * 2, height: dot * 2)).fill()

            // Attention badge: a solid dot in the top-right corner, with a clear ring around it.
            if tone == .warning || tone == .bad {
                let badge = NSRect(x: rect.maxX - 6.5, y: rect.maxY - 6.5, width: 6, height: 6)
                NSGraphicsContext.current?.compositingOperation = .clear
                NSBezierPath(ovalIn: badge.insetBy(dx: -1.2, dy: -1.2)).fill()
                NSGraphicsContext.current?.compositingOperation = .sourceOver
                NSColor.black.setFill()
                NSBezierPath(ovalIn: badge).fill()
            }
            return true
        }
        image.isTemplate = true
        image.accessibilityDescription = headline.isEmpty ? "Openergy" : "Openergy, \(headline)"
        return image
    }
}
