// Draws the Openergy app icon at every size macOS needs and writes an .iconset folder.
// Usage: swift make-icon.swift <output.iconset>
import AppKit

let output = URL(fileURLWithPath: CommandLine.arguments.dropFirst().first ?? "AppIcon.iconset")
try? FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)

func color(_ hex: UInt32, _ alpha: CGFloat = 1) -> CGColor {
    CGColor(srgbRed: CGFloat((hex >> 16) & 0xFF) / 255, green: CGFloat((hex >> 8) & 0xFF) / 255,
            blue: CGFloat(hex & 0xFF) / 255, alpha: alpha)
}

/// Superellipse ("squircle") similar to the macOS icon shape.
func squircle(in rect: CGRect, exponent: CGFloat = 5) -> CGPath {
    let path = CGMutablePath()
    let a = rect.width / 2, b = rect.height / 2
    let center = CGPoint(x: rect.midX, y: rect.midY)
    let steps = 720
    for step in 0...steps {
        let t = CGFloat(step) / CGFloat(steps) * 2 * .pi
        let c = cos(t), s = sin(t)
        let x = center.x + a * (c >= 0 ? 1 : -1) * pow(abs(c), 2 / exponent)
        let y = center.y + b * (s >= 0 ? 1 : -1) * pow(abs(s), 2 / exponent)
        if step == 0 { path.move(to: CGPoint(x: x, y: y)) } else { path.addLine(to: CGPoint(x: x, y: y)) }
    }
    path.closeSubpath()
    return path
}

func drawIcon(pixels: Int) -> Data? {
    guard let context = CGContext(data: nil, width: pixels, height: pixels, bitsPerComponent: 8, bytesPerRow: 0,
                                  space: CGColorSpace(name: CGColorSpace.sRGB)!,
                                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else { return nil }
    let scale = CGFloat(pixels) / 1024
    context.scaleBy(x: scale, y: scale)

    let body = CGRect(x: 100, y: 100, width: 824, height: 824)
    let shape = squircle(in: body)

    // Soft drop shadow, as on other macOS icons.
    context.saveGState()
    context.setShadow(offset: CGSize(width: 0, height: -12), blur: 28, color: color(0x000000, 0.35))
    context.addPath(shape)
    context.setFillColor(color(0x9333EA))
    context.fillPath()
    context.restoreGState()

    context.saveGState()
    context.addPath(shape)
    context.clip()
    let gradient = CGGradient(colorsSpace: CGColorSpace(name: CGColorSpace.sRGB),
                              colors: [color(0xF0ABFC), color(0xD946EF), color(0x9333EA), color(0x4C1D95)] as CFArray,
                              locations: [0, 0.35, 0.75, 1])!
    context.drawLinearGradient(gradient, start: CGPoint(x: body.minX, y: body.maxY), end: CGPoint(x: body.maxX, y: body.minY), options: [])
    let sheen = CGGradient(colorsSpace: CGColorSpace(name: CGColorSpace.sRGB),
                           colors: [color(0xFFFFFF, 0.4), color(0xFFFFFF, 0)] as CFArray, locations: [0, 1])!
    let sheenCenter = CGPoint(x: body.minX + body.width * 0.25, y: body.maxY - body.height * 0.15)
    context.drawRadialGradient(sheen, startCenter: sheenCenter, startRadius: 0, endCenter: sheenCenter, endRadius: 760, options: [])
    context.restoreGState()

    // Two half rings around one core.
    let center = CGPoint(x: 512, y: 512)
    context.setLineCap(.round)
    context.setLineWidth(92)
    context.setStrokeColor(color(0xFFFFFF))
    context.addArc(center: center, radius: 240, startAngle: 100 * .pi / 180, endAngle: 260 * .pi / 180, clockwise: false)
    context.strokePath()
    context.setStrokeColor(color(0xFFFFFF, 0.5))
    context.addArc(center: center, radius: 240, startAngle: 80 * .pi / 180, endAngle: -80 * .pi / 180, clockwise: true)
    context.strokePath()
    context.setFillColor(color(0xFFFFFF))
    context.fillEllipse(in: CGRect(x: center.x - 46, y: center.y - 46, width: 92, height: 92))

    guard let image = context.makeImage() else { return nil }
    return NSBitmapImageRep(cgImage: image).representation(using: .png, properties: [:])
}

let sizes: [(String, Int)] = [
    ("icon_16x16", 16), ("icon_16x16@2x", 32), ("icon_32x32", 32), ("icon_32x32@2x", 64),
    ("icon_128x128", 128), ("icon_128x128@2x", 256), ("icon_256x256", 256), ("icon_256x256@2x", 512),
    ("icon_512x512", 512), ("icon_512x512@2x", 1024),
]
for (name, pixels) in sizes {
    guard let data = drawIcon(pixels: pixels) else { fatalError("Could not draw \(name)") }
    try data.write(to: output.appendingPathComponent("\(name).png"))
}
print("Wrote \(sizes.count) icon images to \(output.path)")
