import AppKit
import SwiftUI

extension EnvironmentValues {
    /// False while the hosting window is closed, hidden or fully covered.
    @Entry var windowVisible = true
}

extension View {
    /// Publishes the hosting window's visibility as `\.windowVisible`, so continuous
    /// animations stop when nobody can see them. The menu bar panel keeps its views
    /// alive while hidden, so without this its animations would run all day.
    func tracksWindowVisibility() -> some View {
        modifier(WindowVisibilityModifier())
    }
}

private struct WindowVisibilityModifier: ViewModifier {
    @State private var visible = true

    func body(content: Content) -> some View {
        content
            .environment(\.windowVisible, visible)
            .background(WindowVisibilityReader(visible: $visible))
    }
}

private struct WindowVisibilityReader: NSViewRepresentable {
    @Binding var visible: Bool

    func makeNSView(context: Context) -> ObservingView {
        let view = ObservingView()
        view.onChange = { visible = $0 }
        return view
    }

    func updateNSView(_ view: ObservingView, context: Context) {
        view.onChange = { visible = $0 }
    }

    final class ObservingView: NSView {
        var onChange: ((Bool) -> Void)?
        private var observer: NSObjectProtocol?

        override func viewDidMoveToWindow() {
            super.viewDidMoveToWindow()
            if let observer { NotificationCenter.default.removeObserver(observer) }
            observer = nil
            guard let window else { return report() }
            observer = NotificationCenter.default.addObserver(
                forName: NSWindow.didChangeOcclusionStateNotification, object: window, queue: .main
            ) { [weak self] _ in
                MainActor.assumeIsolated { self?.report() }
            }
            report()
        }

        private func report() {
            let visible = window?.occlusionState.contains(.visible) ?? false
            // SwiftUI state must not change during a view update, so publish on the next turn.
            DispatchQueue.main.async { [weak self] in
                MainActor.assumeIsolated { self?.onChange?(visible) }
            }
        }
    }
}
