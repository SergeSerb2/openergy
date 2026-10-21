import Foundation

/// Request/response and event client for the daemon control socket.
@MainActor
final class DaemonClient {
    var onEvent: (String, JSONValue) -> Void = { _, _ in }
    var onDisconnect: (Error?) -> Void = { _ in }

    private var socket: IPCSocket?
    private var nextID: UInt64 = 1
    private var pending: [UInt64: CheckedContinuation<JSONValue, Error>] = [:]

    var isConnected: Bool { socket != nil }

    func connect(path: String) throws {
        disconnect()
        let socket = try IPCSocket(path: path)
        self.socket = socket
        socket.startReading(
            onFrame: { [weak self] data in
                Task { @MainActor in self?.handleFrame(data) }
            },
            onClose: { [weak self] error in
                Task { @MainActor in self?.handleClose(socket: socket, error: error) }
            }
        )
    }

    func disconnect() {
        guard let socket else { return }
        self.socket = nil
        socket.close()
        failPending(IPCError.closed)
    }

    @discardableResult
    func call(_ method: String, _ params: JSONValue = [:], timeout: Duration = .seconds(10)) async throws -> JSONValue {
        guard let socket else { throw IPCError.closed }
        let id = nextID
        nextID += 1

        let request: JSONValue = ["id": .number(Double(id)), "method": .string(method), "params": params]
        let payload = try JSONEncoder().encode(request)

        return try await withCheckedThrowingContinuation { continuation in
            pending[id] = continuation
            do {
                try socket.send(payload)
            } catch {
                pending.removeValue(forKey: id)?.resume(throwing: error)
                return
            }
            Task { @MainActor [weak self] in
                try? await Task.sleep(for: timeout)
                self?.pending.removeValue(forKey: id)?.resume(throwing: IPCError.timeout(method))
            }
        }
    }

    private func handleFrame(_ data: Data) {
        guard let message = try? JSONDecoder().decode(JSONValue.self, from: data) else { return }

        if let event = message["event"]?.stringValue {
            onEvent(event, message["data"] ?? .null)
            return
        }

        guard case .number(let rawID)? = message["id"],
              let continuation = pending.removeValue(forKey: UInt64(rawID)) else { return }

        if let error = message["error"], error != .null {
            continuation.resume(throwing: IPCError.remote(
                code: error["code"]?.stringValue ?? "error",
                message: error["message"]?.stringValue ?? "The Openergy service reported an error."
            ))
        } else {
            continuation.resume(returning: message["result"] ?? .null)
        }
    }

    private func handleClose(socket closedSocket: IPCSocket, error: Error?) {
        guard socket === closedSocket else { return }
        socket = nil
        closedSocket.close()
        failPending(error ?? IPCError.closed)
        onDisconnect(error)
    }

    private func failPending(_ error: Error) {
        let waiting = pending
        pending.removeAll()
        for continuation in waiting.values {
            continuation.resume(throwing: error)
        }
    }
}
