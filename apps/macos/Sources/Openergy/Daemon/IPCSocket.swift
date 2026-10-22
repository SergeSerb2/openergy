import Darwin
import Foundation

enum IPCError: LocalizedError, Sendable {
    case socketPathTooLong
    case connectFailed(Int32)
    case writeFailed(Int32)
    case frameTooLarge(UInt32)
    case closed
    case timeout(String)
    case remote(code: String, message: String)
    case badResponse

    var errorDescription: String? {
        switch self {
        case .socketPathTooLong: "The control socket path is too long."
        case .connectFailed(let code): "Could not reach the Openergy service (\(String(cString: strerror(code))))."
        case .writeFailed(let code): "Could not send to the Openergy service (\(String(cString: strerror(code))))."
        case .frameTooLarge(let size): "The Openergy service sent an oversized message (\(size) bytes)."
        case .closed: "The connection to the Openergy service closed."
        case .timeout(let method): "The Openergy service did not answer \(method) in time."
        case .remote(_, let message): message
        case .badResponse: "The Openergy service sent a response the app does not understand."
        }
    }
}

/// One connection to the daemon's control socket.
/// Framing: u32 big-endian length, then that many bytes of UTF-8 JSON.
final class IPCSocket: @unchecked Sendable {
    static let maxFrame: UInt32 = 16 * 1024 * 1024

    private let fd: Int32
    private let lock = NSLock()
    private let writeLock = NSLock()
    private let writes = DispatchQueue(label: "dev.openergy.ipc-writer", qos: .userInitiated)
    private var isClosed = false
    private var descriptorClosed = false

    init(path: String) throws {
        var address = sockaddr_un()
        address.sun_family = sa_family_t(AF_UNIX)
        let capacity = MemoryLayout.size(ofValue: address.sun_path)
        guard !path.isEmpty, !path.utf8.contains(0), path.utf8.count < capacity else { throw IPCError.socketPathTooLong }

        let fd = socket(AF_UNIX, SOCK_STREAM, 0)
        guard fd >= 0 else { throw IPCError.connectFailed(errno) }

        withUnsafeMutableBytes(of: &address.sun_path) { buffer in
            buffer.initializeMemory(as: UInt8.self, repeating: 0)
            _ = path.utf8CString.withUnsafeBytes { source in
                memcpy(buffer.baseAddress!, source.baseAddress!, min(source.count, capacity - 1))
            }
        }

        var noSigPipe: Int32 = 1
        setsockopt(fd, SOL_SOCKET, SO_NOSIGPIPE, &noSigPipe, socklen_t(MemoryLayout<Int32>.size))
        _ = fcntl(fd, F_SETFD, FD_CLOEXEC)
        _ = fcntl(fd, F_SETFL, O_NONBLOCK)
        // Bound stalled writes even when a connected daemon stops draining IPC.
        var writeTimeout = timeval(tv_sec: 2, tv_usec: 0)
        setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &writeTimeout, socklen_t(MemoryLayout<timeval>.size))

        let result = withUnsafePointer(to: &address) { pointer in
            pointer.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                Darwin.connect(fd, $0, socklen_t(MemoryLayout<sockaddr_un>.size))
            }
        }
        var connectError: Int32 = result == 0 ? 0 : errno
        if connectError == EINPROGRESS {
            var descriptor = pollfd(fd: fd, events: Int16(POLLOUT), revents: 0)
            let ready = poll(&descriptor, 1, 2000)
            if ready > 0 {
                var size = socklen_t(MemoryLayout<Int32>.size)
                if getsockopt(fd, SOL_SOCKET, SO_ERROR, &connectError, &size) != 0 { connectError = errno }
            } else {
                connectError = ready == 0 ? ETIMEDOUT : errno
            }
        }
        guard connectError == 0 else {
            Darwin.close(fd)
            throw IPCError.connectFailed(connectError)
        }
        _ = fcntl(fd, F_SETFL, 0)
        self.fd = fd
    }

    deinit {
        if !descriptorClosed { Darwin.close(fd) }
    }

    /// Enqueue in caller order, without blocking the UI actor on a slow daemon.
    func send(_ payload: Data, completion: @escaping @Sendable (Error?) -> Void) {
        writes.async { [self] in
            do { try write(payload); completion(nil) }
            catch { completion(error) }
        }
    }

    private func write(_ payload: Data) throws {
        guard payload.count <= Int(Self.maxFrame) else {
            throw IPCError.frameTooLarge(UInt32(clamping: payload.count))
        }
        var length = UInt32(payload.count).bigEndian
        var frame = Data(bytes: &length, count: 4)
        frame.append(payload)

        writeLock.lock()
        defer { writeLock.unlock() }
        let closed = lock.withLock { isClosed }
        guard !closed else { throw IPCError.closed }
        try frame.withUnsafeBytes { buffer in
            var offset = 0
            while offset < buffer.count {
                let written = Darwin.write(fd, buffer.baseAddress! + offset, buffer.count - offset)
                if written < 0 {
                    if errno == EINTR { continue }
                    throw IPCError.writeFailed(errno)
                }
                if written == 0 { throw IPCError.closed }
                offset += written
            }
        }
    }

    /// Reads frames on a background thread until the socket closes.
    func startReading(
        onFrame: @escaping @Sendable (Data) -> Void,
        onClose: @escaping @Sendable (Error?) -> Void
    ) {
        let thread = Thread { [self] in
            do {
                while true {
                    let header = try readExactly(4)
                    let length = header.withUnsafeBytes { $0.loadUnaligned(as: UInt32.self).bigEndian }
                    guard length <= Self.maxFrame else { throw IPCError.frameTooLarge(length) }
                    onFrame(try readExactly(Int(length)))
                }
            } catch {
                // Only the reader closes the descriptor, after it has stopped reading, and under the
                // lock: a new socket can then never reuse the number while it is still in use here.
                close()
                writeLock.withLock {
                    lock.withLock {
                        Darwin.close(fd)
                        descriptorClosed = true
                    }
                }
                onClose(error)
            }
        }
        thread.name = "dev.openergy.ipc-reader"
        thread.qualityOfService = .userInitiated
        thread.start()
    }

    /// Stops both directions; the reader thread wakes up, exits and closes the descriptor.
    func close() {
        lock.lock()
        defer { lock.unlock() }
        guard !isClosed else { return }
        isClosed = true
        shutdown(fd, SHUT_RDWR)
    }

    private func readExactly(_ count: Int) throws -> Data {
        var data = Data(count: count)
        var offset = 0
        while offset < count {
            let received = data.withUnsafeMutableBytes { buffer in
                Darwin.read(fd, buffer.baseAddress! + offset, count - offset)
            }
            if received == 0 { throw IPCError.closed }
            if received < 0 {
                if errno == EINTR { continue }
                throw IPCError.closed
            }
            offset += received
        }
        return data
    }
}
