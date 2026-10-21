import SwiftUI
import UniformTypeIdentifiers

/// One screen in the arrangement: this Mac or a remote computer.
struct LayoutItem: Identifiable, Equatable {
    var name: String
    var os: String?
    var size: CGSize
    var isLocal: Bool
    var peer: Peer?
    var rect: CGRect?

    var id: String { name }
    var isOnline: Bool { isLocal || peer?.connected == true }
}

extension AppModel {
    /// All screens the server knows: this Mac, configured screens and connected clients.
    var layoutItems: [LayoutItem] {
        var items: [String: LayoutItem] = [:]
        let local = localName
        let localSize = CGSize(width: status.localScreen?.w ?? 1512, height: status.localScreen?.h ?? 982)
        items[local] = LayoutItem(name: local, os: "macos", size: localSize, isLocal: true, peer: nil, rect: nil)

        for screen in config.screens ?? [] {
            var item = items[screen.name] ?? LayoutItem(name: screen.name, os: screen.os, size: CGSize(width: 1920, height: 1080),
                                                        isLocal: false, peer: nil, rect: nil)
            item.rect = screen.rect?.cgRect
            // A saved size is real data: keep it until the screen reports its live size.
            if !item.isLocal, let rect = screen.rect, rect.w > 0, rect.h > 0 {
                item.size = CGSize(width: rect.w, height: rect.h)
            }
            item.os = item.os ?? screen.os
            items[screen.name] = item
        }
        for peer in status.peers ?? [] where peer.role != "server" {
            var item = items[peer.name] ?? LayoutItem(name: peer.name, os: peer.os, size: CGSize(width: 1920, height: 1080),
                                                      isLocal: false, peer: nil, rect: nil)
            item.peer = peer
            item.os = peer.os ?? item.os
            if let w = peer.screen?.w, let h = peer.screen?.h, w > 0, h > 0 {
                item.size = CGSize(width: w, height: h)
            }
            items[peer.name] = item
        }
        // A placed screen keeps its position but always shows its real size.
        for (name, item) in items {
            if let rect = item.rect {
                items[name]?.rect = CGRect(origin: rect.origin, size: item.size)
            }
        }
        return items.values.sorted { lhs, rhs in
            if lhs.isLocal != rhs.isLocal { return lhs.isLocal }
            return lhs.name.localizedStandardCompare(rhs.name) == .orderedAscending
        }
    }
}

struct LayoutEditor: View {
    let model: AppModel
    @State private var selection: String?
    @State private var dragging: String?
    @State private var dragTranslation: CGSize = .zero
    @State private var frozenTransform: CanvasTransform?
    @State private var showInspector = false
    @FocusState private var canvasFocused: Bool

    var body: some View {
        let items = model.layoutItems
        let placed = items.filter { $0.rect != nil }
        let unplaced = items.filter { $0.rect == nil }

        VStack(spacing: 0) {
            GeometryReader { proxy in
                let transform = frozenTransform ?? fittedTransform(placed, size: proxy.size)
                ZStack(alignment: .topLeading) {
                    CanvasBackground()
                        .contentShape(Rectangle())
                        .onTapGesture { selection = nil }

                    linkBars(placed, transform: transform)

                    if let ghost = ghostRect(placed, transform: transform) {
                        let frame = transform.toView(ghost)
                        RoundedRectangle(cornerRadius: 10, style: .continuous)
                            .strokeBorder(Brand.accent.opacity(0.8), style: StrokeStyle(lineWidth: 1.5, dash: [5, 4]))
                            .frame(width: frame.width, height: frame.height)
                            .offset(x: frame.minX, y: frame.minY)
                            .allowsHitTesting(false)
                    }

                    ForEach(placed) { item in
                        tile(item, placed: placed, transform: transform)
                    }

                    if placed.count <= 1 {
                        hint
                            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottom)
                            .padding(.bottom, 24)
                            .allowsHitTesting(false)
                    }
                }
                .coordinateSpace(.named("canvas"))
                .dropDestination(for: String.self) { names, location in
                    guard let name = names.first, let item = items.first(where: { $0.name == name }) else { return false }
                    place(item, at: transform.toLayout(location), placed: placed)
                    return true
                }
            }
            .focusable()
            .focused($canvasFocused)
            .focusEffectDisabled()
            .onKeyPress(keys: [.leftArrow, .rightArrow, .upArrow, .downArrow]) { press in
                nudge(press.key, placed: placed) ? .handled : .ignored
            }
            .onKeyPress(.tab) {
                // Tab walks through the placed screens so the arrow keys work without a mouse.
                let names = placed.map(\.name)
                guard !names.isEmpty else { return .ignored }
                let next = selection.flatMap { names.firstIndex(of: $0) }.map { ($0 + 1) % names.count } ?? 0
                selection = names[next]
                return .handled
            }
            .onKeyPress(.delete) {
                guard let selection, let item = placed.first(where: { $0.name == selection }), !item.isLocal else { return .ignored }
                unplace(item, placed: placed)
                return .handled
            }

            if !unplaced.isEmpty {
                Divider()
                UnplacedTray(items: unplaced) { item in
                    place(item, at: nil, placed: placed)
                }
            }
        }
        .task(id: placed.contains(where: \.isLocal)) {
            // A server always owns a screen in its layout: put this Mac on the canvas first.
            // Wait briefly so a config that is still arriving (an import, another window) wins.
            guard model.layoutItems.contains(where: { $0.isLocal && $0.rect == nil }) else { return }
            try? await Task.sleep(for: .milliseconds(300))
            guard !Task.isCancelled, model.role == .server else { return }
            let current = model.layoutItems
            if let local = current.first(where: \.isLocal), local.rect == nil {
                place(local, at: nil, placed: current.filter { $0.rect != nil })
            }
        }
        .inspector(isPresented: $showInspector) {
            ScreenInspector(model: model, item: items.first { $0.name == selection }, placed: placed) { item in
                unplace(item, placed: placed)
            }
            .inspectorColumnWidth(min: 230, ideal: 260, max: 340)
        }
        .toolbar {
            ToolbarItem {
                Button {
                    showInspector.toggle()
                } label: {
                    Label("Inspector", systemImage: "sidebar.trailing")
                }
                .help("Show or hide screen details")
            }
        }
    }

    // MARK: Tiles

    private func tile(_ item: LayoutItem, placed: [LayoutItem], transform: CanvasTransform) -> some View {
        let rect = item.rect ?? .zero
        let isDragging = dragging == item.name
        let frame = transform.toView(rect)
            .offsetBy(dx: isDragging ? dragTranslation.width : 0, dy: isDragging ? dragTranslation.height : 0)
        let isActive = model.status.activeScreen == item.name
        let cursor: CGPoint? = {
            guard isActive, let x = model.status.cursor?.x, let y = model.status.cursor?.y,
                  item.size.width > 0, item.size.height > 0 else { return nil }
            return CGPoint(x: x / item.size.width, y: y / item.size.height)
        }()

        return ScreenTile(item: item, isActive: isActive, isSelected: selection == item.name, cursor: cursor)
            .frame(width: frame.width, height: frame.height)
            .offset(x: frame.minX, y: frame.minY)
            .zIndex(isDragging ? 2 : (selection == item.name ? 1 : 0))
            .scaleEffect(isDragging ? 1.02 : 1)
            .shadow(color: .black.opacity(isDragging ? 0.35 : 0), radius: 14, y: 8)
            .onTapGesture {
                selection = item.name
                canvasFocused = true
                showInspector = true
            }
            .gesture(
                DragGesture(minimumDistance: 3, coordinateSpace: .named("canvas"))
                    .onChanged { value in
                        if dragging == nil {
                            dragging = item.name
                            selection = item.name
                            frozenTransform = transform
                        }
                        dragTranslation = value.translation
                    }
                    .onEnded { value in
                        let moved = rect.offsetBy(dx: value.translation.width / transform.scale,
                                                  dy: value.translation.height / transform.scale)
                        commit(item.name, proposed: moved, placed: placed, transform: transform)
                    }
            )
            .accessibilityElement(children: .combine)
            .accessibilityLabel("\(item.name), \(item.isLocal ? "this Mac" : (item.isOnline ? "connected" : "offline"))")
            .accessibilityHint("Drag to arrange, or use the actions to move it beside another screen.")
            .accessibilityAddTraits(selection == item.name ? [.isButton, .isSelected] : .isButton)
            .accessibilityAction { selection = item.name; showInspector = true }
            .accessibilityAction(named: "Move Left") { selection = item.name; _ = nudge(.leftArrow, placed: placed) }
            .accessibilityAction(named: "Move Right") { selection = item.name; _ = nudge(.rightArrow, placed: placed) }
            .accessibilityAction(named: "Move Up") { selection = item.name; _ = nudge(.upArrow, placed: placed) }
            .accessibilityAction(named: "Move Down") { selection = item.name; _ = nudge(.downArrow, placed: placed) }
            .accessibilityActions {
                if !item.isLocal {
                    Button("Remove from Layout") { unplace(item, placed: placed) }
                }
            }
    }

    private func linkBars(_ placed: [LayoutItem], transform: CanvasTransform) -> some View {
        let rects = placed.filter { $0.name != dragging }.compactMap(\.rect)
        let contacts = LayoutGeometry.contacts(rects)
        return ForEach(contacts, id: \.self) { contact in
            let thickness: CGFloat = 4
            let inset: CGFloat = 6
            let frame: CGRect = switch contact.axis {
            case .vertical:
                CGRect(x: contact.position * transform.scale + transform.offset.x - thickness / 2,
                       y: contact.start * transform.scale + transform.offset.y + inset,
                       width: thickness,
                       height: max(0, (contact.end - contact.start) * transform.scale - inset * 2))
            case .horizontal:
                CGRect(x: contact.start * transform.scale + transform.offset.x + inset,
                       y: contact.position * transform.scale + transform.offset.y - thickness / 2,
                       width: max(0, (contact.end - contact.start) * transform.scale - inset * 2),
                       height: thickness)
            }
            Capsule()
                .fill(Brand.linkGradient)
                .shadow(color: Brand.accent.opacity(0.8), radius: 6)
                .frame(width: frame.width, height: frame.height)
                .offset(x: frame.minX, y: frame.minY)
                .zIndex(3)
                .allowsHitTesting(false)
                .transition(.opacity)
        }
    }

    private var hint: some View {
        Label("Drag a computer from the bar below onto the layout. Edges that touch are where the pointer crosses.",
              systemImage: "hand.draw")
            .font(.callout)
            .foregroundStyle(.secondary)
            .padding(.horizontal, 14)
            .padding(.vertical, 8)
            .background(.regularMaterial, in: Capsule())
    }

    // MARK: Geometry

    private func fittedTransform(_ placed: [LayoutItem], size: CGSize) -> CanvasTransform {
        let bounds = placed.compactMap(\.rect).reduce(nil) { (partial: CGRect?, rect) in partial?.union(rect) ?? rect }
            ?? CGRect(x: 0, y: 0, width: 1512, height: 982)
        // Leave room around the layout so a screen can be dropped on any side.
        let padded = bounds.insetBy(dx: -bounds.width * 0.1, dy: -bounds.height * 0.25)
        return CanvasTransform.fitting(padded, in: size, padding: 28)
    }

    private func ghostRect(_ placed: [LayoutItem], transform: CanvasTransform) -> CGRect? {
        guard let dragging, let item = placed.first(where: { $0.name == dragging }), let rect = item.rect else { return nil }
        let moved = rect.offsetBy(dx: dragTranslation.width / transform.scale, dy: dragTranslation.height / transform.scale)
        let others = placed.filter { $0.name != dragging }.compactMap(\.rect)
        return LayoutGeometry.snap(moved, to: others, alignThreshold: 14 / transform.scale)
    }

    private func commit(_ name: String, proposed: CGRect, placed: [LayoutItem], transform: CanvasTransform) {
        let others = placed.filter { $0.name != name }.compactMap(\.rect)
        let snapped = LayoutGeometry.snap(proposed, to: others, alignThreshold: 14 / transform.scale)
        withAnimation(.spring(response: 0.32, dampingFraction: 0.82)) {
            dragging = nil
            dragTranslation = .zero
            save(placed, setting: name, to: snapped)
        }
        frozenTransform = nil
    }

    private func place(_ item: LayoutItem, at point: CGPoint?, placed: [LayoutItem]) {
        let rects = placed.filter { $0.name != item.name }.compactMap(\.rect)
        var rect = LayoutGeometry.freeSpot(for: item.size, beside: rects)
        if let point {
            rect.origin = CGPoint(x: point.x - item.size.width / 2, y: point.y - item.size.height / 2)
        }
        let snapped = LayoutGeometry.snap(rect, to: rects, alignThreshold: item.size.height * 0.1)
        withAnimation(.spring(response: 0.32, dampingFraction: 0.82)) {
            save(placed, setting: item.name, to: snapped, extra: item)
        }
        selection = item.name
    }

    private func unplace(_ item: LayoutItem, placed: [LayoutItem]) {
        var screens = placed.filter { $0.name != item.name }
            .map { OpenergyConfig.Screen(name: $0.name, rect: $0.rect.map(OpenergyConfig.LayoutRect.init), os: $0.os) }
        screens.append(OpenergyConfig.Screen(name: item.name, rect: nil, os: item.os))
        // Keep screens the config already lists without a position.
        for screen in model.config.screens ?? [] where !screens.contains(where: { $0.name == screen.name }) {
            screens.append(.init(name: screen.name, rect: nil, os: screen.os))
        }
        withAnimation(.smooth) { model.saveLayout(screens) }
        selection = nil
    }

    private func nudge(_ key: KeyEquivalent, placed: [LayoutItem]) -> Bool {
        guard let selection, let item = placed.first(where: { $0.name == selection }), let rect = item.rect else { return false }
        let others = placed.filter { $0.name != selection }.compactMap(\.rect)
        guard !others.isEmpty else { return false }
        let delta: CGSize = switch key {
        case .leftArrow: CGSize(width: -rect.width, height: 0)
        case .rightArrow: CGSize(width: rect.width, height: 0)
        case .upArrow: CGSize(width: 0, height: -rect.height)
        default: CGSize(width: 0, height: rect.height)
        }
        let snapped = LayoutGeometry.snap(rect.offsetBy(dx: delta.width, dy: delta.height), to: others, alignThreshold: rect.height * 0.1)
        withAnimation(.spring(response: 0.32, dampingFraction: 0.82)) {
            save(placed, setting: selection, to: snapped)
        }
        return true
    }

    private func save(_ placed: [LayoutItem], setting name: String, to rect: CGRect, extra: LayoutItem? = nil) {
        var screens = placed.map { item in
            OpenergyConfig.Screen(name: item.name, rect: (item.name == name ? rect : item.rect).map(OpenergyConfig.LayoutRect.init), os: item.os)
        }
        if let extra, !screens.contains(where: { $0.name == extra.name }) {
            screens.append(.init(name: extra.name, rect: OpenergyConfig.LayoutRect(rect), os: extra.os))
        }
        // Keep unplaced screens that the config already knows about.
        for screen in model.config.screens ?? [] where !screens.contains(where: { $0.name == screen.name }) {
            screens.append(.init(name: screen.name, rect: nil, os: screen.os))
        }
        model.saveLayout(screens)
    }
}

/// Dot grid with a soft brand glow, like a design canvas.
struct CanvasBackground: View {
    var body: some View {
        Canvas { context, size in
            let spacing: CGFloat = 18
            let dot = Path(ellipseIn: CGRect(x: 0, y: 0, width: 1.6, height: 1.6))
            var y: CGFloat = spacing / 2
            while y < size.height {
                var x: CGFloat = spacing / 2
                while x < size.width {
                    context.fill(dot.offsetBy(dx: x, dy: y), with: .color(.primary.opacity(0.09)))
                    x += spacing
                }
                y += spacing
            }
        }
        .background(
            RadialGradient(colors: [Brand.violet.opacity(0.10), .clear], center: .center, startRadius: 10, endRadius: 520)
        )
    }
}

/// A computer screen on the canvas.
struct ScreenTile: View {
    let item: LayoutItem
    var isActive: Bool
    var isSelected: Bool
    var cursor: CGPoint?
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        GeometryReader { proxy in
            let compact = proxy.size.height < 78
            ZStack {
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .fill(fill)
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .strokeBorder(border, lineWidth: isSelected || isActive ? 1.5 : 1)

                VStack(spacing: compact ? 2 : 6) {
                    OSSymbol(os: item.os)
                        .font(.system(size: compact ? 16 : 24, weight: .regular))
                        .foregroundStyle(item.isOnline ? .primary : .secondary)
                    Text(item.name)
                        .font(.system(size: compact ? 11 : 12.5, weight: .semibold))
                        .lineLimit(1)
                        .truncationMode(.middle)
                    if !compact {
                        Text(subtitle)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }
                }
                .padding(.horizontal, 8)

                if item.isLocal, proxy.size.height > 150 {
                    Text("THIS MAC")
                        .font(.system(size: 9, weight: .bold))
                        .tracking(0.6)
                        .foregroundStyle(Brand.accent)
                        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                        .padding(8)
                }

                Circle()
                    .fill(item.isOnline ? Brand.good : Color.secondary.opacity(0.5))
                    .frame(width: 7, height: 7)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
                    .padding(8)

                if let cursor {
                    Circle()
                        .fill(.white)
                        .frame(width: 7, height: 7)
                        .shadow(color: Brand.accentLight, radius: 5)
                        .position(x: min(max(cursor.x, 0.04), 0.96) * proxy.size.width,
                                  y: min(max(cursor.y, 0.06), 0.94) * proxy.size.height)
                        .animation(.linear(duration: 0.12), value: cursor)
                }
            }
        }
        .shadow(color: isActive ? Brand.accent.opacity(0.35) : .black.opacity(colorScheme == .dark ? 0.35 : 0.08),
                radius: isActive ? 14 : 8, y: isActive ? 0 : 4)
    }

    private var subtitle: String {
        if let peer = item.peer, peer.connected != true { return "Offline" }
        let size = "\(Int(item.size.width)) × \(Int(item.size.height))"
        return item.isLocal ? "This Mac · \(size)" : size
    }

    private var fill: LinearGradient {
        let dark = colorScheme == .dark
        if isActive {
            return LinearGradient(colors: dark ? [Color(white: 0.27), Color(red: 0.2, green: 0.14, blue: 0.24)]
                                               : [Color.white, Color(red: 0.98, green: 0.93, blue: 0.99)],
                                  startPoint: .topLeading, endPoint: .bottomTrailing)
        }
        return LinearGradient(colors: dark ? [Color(white: 0.24), Color(white: 0.17)] : [Color.white, Color(white: 0.95)],
                              startPoint: .topLeading, endPoint: .bottomTrailing)
    }

    private var border: Color {
        if isSelected { return Brand.accent }
        if isActive { return Brand.accent.opacity(0.6) }
        return Color.primary.opacity(colorScheme == .dark ? 0.16 : 0.12)
    }
}

/// Screens that are known but not placed yet.
struct UnplacedTray: View {
    let items: [LayoutItem]
    var place: (LayoutItem) -> Void

    var body: some View {
        HStack(spacing: 10) {
            Text("Not placed")
                .font(.callout)
                .foregroundStyle(.secondary)
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    ForEach(items) { item in
                        Button { place(item) } label: {
                            HStack(spacing: 6) {
                                OSSymbol(os: item.os)
                                Text(item.name)
                                Circle().fill(item.isOnline ? Brand.good : Color.secondary.opacity(0.5)).frame(width: 6, height: 6)
                            }
                            .font(.callout)
                            .padding(.horizontal, 10)
                            .padding(.vertical, 5)
                            .background(.quaternary.opacity(0.6), in: RoundedRectangle(cornerRadius: 7, style: .continuous))
                            .overlay(
                                RoundedRectangle(cornerRadius: 7, style: .continuous)
                                    .strokeBorder(Color.primary.opacity(0.2), style: StrokeStyle(lineWidth: 1, dash: [4, 3]))
                            )
                        }
                        .buttonStyle(.plain)
                        .draggable(item.name)
                        .contextMenu { Button("Place on Layout") { place(item) } }
                        .help("Drag onto the layout, or click to place it beside the others")
                        .accessibilityLabel("Place \(item.name) on the layout")
                    }
                }
            }
            Spacer(minLength: 0)
            Text("Drag onto the layout")
                .font(.caption)
                .foregroundStyle(.tertiary)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }
}
