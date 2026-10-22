import AppKit
import Foundation

public struct LabError: Error, CustomStringConvertible {
    public let description: String
    public init(_ description: String) { self.description = description }
}

public enum JSONFile {
    public static func write(_ object: [String: Any], to url: URL) throws {
        let data = try JSONSerialization.data(withJSONObject: object, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: url, options: .atomic)
    }

    public static func read(_ url: URL) -> [String: Any]? {
        guard let data = try? Data(contentsOf: url) else { return nil }
        return (try? JSONSerialization.jsonObject(with: data)) as? [String: Any]
    }

    public static func lines(_ url: URL) -> [[String: Any]] {
        guard let data = try? Data(contentsOf: url), let text = String(data: data, encoding: .utf8) else { return [] }
        // Ignore a final partial record while the fixture is writing it.
        return text.components(separatedBy: "\n").dropLast().compactMap {
            guard let data = $0.data(using: .utf8) else { return nil }
            return (try? JSONSerialization.jsonObject(with: data)) as? [String: Any]
        }
    }
}

@MainActor
public final class EventLog {
    private let handle: FileHandle
    private var sequence = 0

    public init(path: String) throws {
        let url = URL(fileURLWithPath: path)
        try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        if !FileManager.default.fileExists(atPath: path) {
            guard FileManager.default.createFile(atPath: path, contents: nil) else { throw LabError("Cannot create log: \(path)") }
        }
        handle = try FileHandle(forWritingTo: url)
        try handle.seekToEnd()
    }

    public func append(_ event: String, _ fields: [String: Any] = [:]) {
        sequence += 1
        var record = fields
        record["event"] = event
        record["sequence"] = sequence
        record["time"] = Date().timeIntervalSince1970
        record["uptime"] = ProcessInfo.processInfo.systemUptime
        record["pid"] = ProcessInfo.processInfo.processIdentifier
        do {
            var data = try JSONSerialization.data(withJSONObject: record, options: [.sortedKeys])
            data.append(0x0A)
            try handle.write(contentsOf: data)
        } catch {
            FileHandle.standardError.write(Data("FocusLab log error: \(error)\n".utf8))
            // A missing log must become a driver timeout, never fabricated evidence.
        }
    }
}

/// A single atomic JSON command file, with a unique requestID for each command.
/// The timer also runs in menu tracking mode. Responses go into the event log.
@MainActor
public final class CommandInbox {
    private let url: URL
    private var lastID: String?
    private var timer: Timer?
    private let receive: @MainActor ([String: Any]) -> Void

    public init(path: String, receive: @escaping @MainActor ([String: Any]) -> Void) {
        url = URL(fileURLWithPath: path)
        self.receive = receive
    }

    public func start() {
        let timer = Timer(timeInterval: 0.04, repeats: true) { [weak self] _ in
            MainActor.assumeIsolated { self?.poll() }
        }
        self.timer = timer
        RunLoop.main.add(timer, forMode: .common)
        RunLoop.main.add(timer, forMode: .eventTracking)
    }

    private func poll() {
        guard let command = JSONFile.read(url), let id = command["requestID"] as? String, id != lastID else { return }
        lastID = id
        receive(command)
    }
}

@MainActor
public enum ScreenCoordinates {
    // NSScreen.screens[0] owns the menu bar. Do not use .main: it follows focus.
    public static var primaryTop: CGFloat { NSScreen.screens.first?.frame.maxY ?? 0 }
    public static func cgPoint(_ point: NSPoint) -> CGPoint { CGPoint(x: point.x, y: primaryTop - point.y) }
    public static func cgRect(_ rect: NSRect) -> CGRect {
        CGRect(x: rect.minX, y: primaryTop - rect.maxY, width: rect.width, height: rect.height)
    }
    public static func nsRect(_ rect: CGRect) -> NSRect {
        NSRect(x: rect.minX, y: primaryTop - rect.maxY, width: rect.width, height: rect.height)
    }
    public static func pointJSON(_ point: CGPoint) -> [String: Double] { ["x": point.x, "y": point.y] }
    public static func rectJSON(_ rect: CGRect) -> [String: Double] {
        ["x": rect.minX, "y": rect.minY, "width": rect.width, "height": rect.height]
    }
    public static func point(_ object: Any?) -> CGPoint? {
        guard let values = object as? [String: Any], let x = values["x"] as? Double, let y = values["y"] as? Double else { return nil }
        return CGPoint(x: x, y: y)
    }
    public static func rect(_ object: Any?) -> CGRect? {
        guard let values = object as? [String: Any], let x = values["x"] as? Double, let y = values["y"] as? Double,
              let width = values["width"] as? Double, let height = values["height"] as? Double else { return nil }
        return CGRect(x: x, y: y, width: width, height: height)
    }
}

public struct FixtureArguments {
    public var values: [String: String] = [:]
    public init(_ arguments: [String]) throws {
        var index = 0
        while index < arguments.count {
            let key = arguments[index]
            guard key.hasPrefix("--"), index + 1 < arguments.count else { throw LabError("Expected --option value; got \(key)") }
            guard values[key] == nil else { throw LabError("Duplicate option: \(key)") }
            values[key] = arguments[index + 1]
            index += 2
        }
    }
    public func require(_ key: String) throws -> String {
        guard let value = values[key], !value.isEmpty else { throw LabError("Missing \(key)") }
        return value
    }
    public func frame() throws -> CGRect {
        let text = values["--frame"] ?? "80,140,460,340"
        let parts = text.split(separator: ",").compactMap { Double($0) }
        guard parts.count == 4, parts.allSatisfy(\.isFinite), parts[2] >= 400, parts[3] >= 300 else {
            throw LabError("--frame requires x,y,width,height in CG screen points; minimum size 400x300")
        }
        return CGRect(x: parts[0], y: parts[1], width: parts[2], height: parts[3])
    }
}
