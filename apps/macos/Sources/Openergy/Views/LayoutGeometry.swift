import CoreGraphics

/// Pure layout math for the screen arrangement editor. Units are screen points.
enum LayoutGeometry {
    struct Contact: Hashable {
        enum Axis { case vertical, horizontal }
        var axis: Axis
        /// For a vertical contact: x of the shared edge, and the y range. Horizontal: y and x range.
        var position: CGFloat
        var start: CGFloat
        var end: CGFloat
    }

    static let edgeTolerance: CGFloat = 1

    /// Shared edges between screens; each one is a place where the pointer can cross.
    static func contacts(_ rects: [CGRect]) -> [Contact] {
        var result: Set<Contact> = []
        for (i, a) in rects.enumerated() {
            for b in rects[(i + 1)...] {
                if abs(a.maxX - b.minX) <= edgeTolerance || abs(b.maxX - a.minX) <= edgeTolerance {
                    let x = abs(a.maxX - b.minX) <= edgeTolerance ? a.maxX : a.minX
                    let start = max(a.minY, b.minY), end = min(a.maxY, b.maxY)
                    if end - start > edgeTolerance { result.insert(.init(axis: .vertical, position: x, start: start, end: end)) }
                }
                if abs(a.maxY - b.minY) <= edgeTolerance || abs(b.maxY - a.minY) <= edgeTolerance {
                    let y = abs(a.maxY - b.minY) <= edgeTolerance ? a.maxY : a.minY
                    let start = max(a.minX, b.minX), end = min(a.maxX, b.maxX)
                    if end - start > edgeTolerance { result.insert(.init(axis: .horizontal, position: y, start: start, end: end)) }
                }
            }
        }
        return Array(result)
    }

    static func overlapsAny(_ rect: CGRect, _ others: [CGRect]) -> Bool {
        others.contains { other in
            let overlap = rect.intersection(other)
            return !overlap.isNull && overlap.width > edgeTolerance && overlap.height > edgeTolerance
        }
    }

    /// Moves `rect` to the nearest position where it touches another screen along an edge
    /// without overlapping any screen. Aligns top/bottom (or left/right) edges when close.
    static func snap(_ rect: CGRect, to others: [CGRect], alignThreshold: CGFloat) -> CGRect {
        guard !others.isEmpty else { return rect }
        var best: CGRect?
        var bestDistance = CGFloat.infinity

        func consider(_ candidate: CGRect) {
            guard !overlapsAny(candidate, others) else { return }
            let distance = hypot(candidate.minX - rect.minX, candidate.minY - rect.minY)
            if distance < bestDistance {
                best = candidate
                bestDistance = distance
            }
        }

        for other in others {
            let minimumShared = min(rect.height, other.height) * 0.2
            for x in [other.maxX, other.minX - rect.width] {
                let low = other.minY - rect.height + minimumShared
                let high = other.maxY - minimumShared
                var y = min(max(rect.minY, low), high)
                for target in [other.minY, other.maxY - rect.height, other.midY - rect.height / 2]
                where abs(y - target) <= alignThreshold {
                    y = target
                    break
                }
                consider(CGRect(x: x, y: y, width: rect.width, height: rect.height))
            }

            let minimumSharedX = min(rect.width, other.width) * 0.2
            for y in [other.maxY, other.minY - rect.height] {
                let low = other.minX - rect.width + minimumSharedX
                let high = other.maxX - minimumSharedX
                var x = min(max(rect.minX, low), high)
                for target in [other.minX, other.maxX - rect.width, other.midX - rect.width / 2]
                where abs(x - target) <= alignThreshold {
                    x = target
                    break
                }
                consider(CGRect(x: x, y: y, width: rect.width, height: rect.height))
            }
        }
        return best ?? rect
    }

    /// A free spot for a newly placed screen: to the right of everything, top-aligned.
    static func freeSpot(for size: CGSize, beside rects: [CGRect]) -> CGRect {
        guard let bounds = rects.reduce(nil, { (partial: CGRect?, rect) in partial?.union(rect) ?? rect }) else {
            return CGRect(origin: .zero, size: size)
        }
        return CGRect(x: bounds.maxX, y: bounds.minY, width: size.width, height: size.height)
    }
}

/// Maps layout points to view points with one uniform scale.
struct CanvasTransform: Equatable {
    var scale: CGFloat
    var offset: CGPoint

    static func fitting(_ bounds: CGRect, in size: CGSize, padding: CGFloat, maxScale: CGFloat = 0.2) -> CanvasTransform {
        guard bounds.width > 0, bounds.height > 0, size.width > 0, size.height > 0 else {
            return CanvasTransform(scale: 0.1, offset: CGPoint(x: size.width / 2, y: size.height / 2))
        }
        let scale = min((size.width - padding * 2) / bounds.width, (size.height - padding * 2) / bounds.height, maxScale)
        let offset = CGPoint(
            x: (size.width - bounds.width * scale) / 2 - bounds.minX * scale,
            y: (size.height - bounds.height * scale) / 2 - bounds.minY * scale
        )
        return CanvasTransform(scale: scale, offset: offset)
    }

    func toView(_ rect: CGRect) -> CGRect {
        CGRect(x: rect.minX * scale + offset.x, y: rect.minY * scale + offset.y,
               width: rect.width * scale, height: rect.height * scale)
    }

    func toView(_ point: CGPoint) -> CGPoint {
        CGPoint(x: point.x * scale + offset.x, y: point.y * scale + offset.y)
    }

    func toLayout(_ point: CGPoint) -> CGPoint {
        CGPoint(x: (point.x - offset.x) / scale, y: (point.y - offset.y) / scale)
    }
}
