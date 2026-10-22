import Foundation

/// Request/response and event client for the daemon control socket.
@MainActor
final class DaemonClient {
    var onEvent: (String, JSONValue) -> Void = { _, _ in }
    var onDisconnect: (Error?) -> Void = { _ in }

    private var socket: IPCSocket?
    private var nextID: UInt64 = 1
    private var generation = UUID()
    private struct Pending {
        var continuation: CheckedContinuation<JSONValue, Error>
        var timeout: Task<Void, Never>
    }
    private var pending: [UInt64: Pending] = [:]

    var isConnected: Bool { socket != nil }

    /// `OPENERGY_IPC_TRACE=1` prints every request (development only; never contains secrets).
    private static let trace = ProcessInfo.processInfo.environment["OPENERGY_IPC_TRACE"] == "1"

    func connect(path: String) async throws {
        disconnect()
        let attempt = generation
        let socket = try await Task.detached(priority: .userInitiated) { try IPCSocket(path: path) }.value
        guard generation == attempt, !Task.isCancelled else {
            socket.close()
            throw CancellationError()
        }
        self.socket = socket
        socket.startReading(
            onFrame: { [weak self] data in
                // The reader emits onto one FIFO queue, preserving event/response/EOF order.
                DispatchQueue.main.async {
                    guard let self, self.socket === socket else { return }
                    self.handleFrame(data)
                }
            },
            onClose: { [weak self] error in
                DispatchQueue.main.async { self?.handleClose(socket: socket, error: error) }
            }
        )
    }

    func disconnect() {
        generation = UUID()
        guard let socket else { return }
        self.socket = nil
        socket.close()
        failPending(IPCError.closed)
    }

    @discardableResult
    func call(_ method: String, _ params: JSONValue = [:], timeout: Duration = .seconds(10)) async throws -> JSONValue {
        guard let socket else { throw IPCError.closed }
        let id = nextID
        // JSON numbers can represent every integer only up to 2^53 - 1.
        guard id <= 9_007_199_254_740_991 else { throw IPCError.badResponse }
        nextID += 1

        let request: JSONValue = ["id": .number(Double(id)), "method": .string(method), "params": params]
        let payload = try JSONEncoder().encode(request)
        if Self.trace { print("IPC → \(String(decoding: payload, as: UTF8.self).prefix(400))") }

        return try await withTaskCancellationHandler {
            try Task.checkCancellation()
            return try await withCheckedThrowingContinuation { continuation in
                let timer = Task { @MainActor [weak self] in
                    do { try await Task.sleep(for: timeout) } catch { return }
                    guard let self, self.pending[id] != nil else { return }
                    // A mutating request may still be in flight. Reset the session instead of
                    // treating a socket that no longer answers as a healthy connection.
                    self.handleClose(socket: socket, error: IPCError.timeout(method))
                }
                pending[id] = Pending(continuation: continuation, timeout: timer)
                socket.send(payload) { [weak self] error in
                    guard let error else { return }
                    Task { @MainActor in self?.handleClose(socket: socket, error: error) }
                }
            }
        } onCancel: {
            Task { @MainActor [weak self] in
                self?.finish(id, with: .failure(CancellationError()))
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
              rawID <= 9_007_199_254_740_991,
              let id = UInt64(exactly: rawID), pending[id] != nil else { return }

        if let error = message["error"], error != .null {
            finish(id, with: .failure(IPCError.remote(
                code: error["code"]?.stringValue ?? "error",
                message: error["message"]?.stringValue ?? "The Openergy service reported an error."
            )))
        } else {
            finish(id, with: .success(message["result"] ?? .null))
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
        for request in waiting.values {
            request.timeout.cancel()
            request.continuation.resume(throwing: error)
        }
    }

    private func finish(_ id: UInt64, with result: Result<JSONValue, Error>) {
        guard let request = pending.removeValue(forKey: id) else { return }
        request.timeout.cancel()
        request.continuation.resume(with: result)
    }
}
