# Primary source research and limits

Public source was read through web read tools only on 2026-10-23. No remote source was downloaded by shell, imported, compiled, or executed. These references establish source semantics, not the deployed kernel's exact behavior during a historical bind.

## Mio pinned by both original lockfiles

Both original `Cargo.lock` files pin `mio` **1.2.4**, package checksum `1788edb87fdc09c7e26304471e2f5be8cdefb1b6930d6e3985fc02ff53bf86ee`. The public revision read was tag **v1.2.4**. Public tag files were not independently compared with the compiled crate archive.

- [TCP listener](https://github.com/tokio-rs/mio/blob/v1.2.4/src/net/tcp/listener.rs): `TcpListener::bind` creates an owned socket, enables address reuse on Unix, binds and listens. `from_std` wraps an already owned listener. `accept` wraps a new accepted stream. It does not clone the listening descriptor.
- [Unix TCP wrapper](https://github.com/tokio-rs/mio/blob/v1.2.4/src/sys/unix/tcp.rs): `new_for_addr`, `bind`, `set_reuseaddr`, and `accept` show the syscall routes. Darwin accept creates a new descriptor and sets close-on-exec. No port-reuse option is added here.
- [Unix socket constructor](https://github.com/tokio-rs/mio/blob/v1.2.4/src/sys/unix/net.rs): `new_socket` sets Darwin nonblocking, close-on-exec, and no-SIGPIPE behavior. It does not enable address or port reuse for outbound sockets.
- [TCP stream](https://github.com/tokio-rs/mio/blob/v1.2.4/src/net/tcp/stream.rs): the stream owns its inner standard-library socket; its shutdown forwards to that socket. Drop releases the descriptor. That is not a wait for kernel TCP-state disposal.

## Rust bound to the saved compiler source revision

A's saved verbose compiler record reports **1.99.1**, commit **58cf8f9c35b355199727627d07ae34a4e2846d67**. B's saved preparation reports 1.99.1, but not its full compiler commit. The same-revision comparison for B is therefore limited to the reported toolchain version.

- [Standard TCP wrapper at that commit](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/library/std/src/sys/net/connection/socket/mod.rs): standard `TcpListener::bind` enables address reuse on non-Windows platforms before bind/listen. `accept` returns a distinct stream. This explains why A's competing standard listener is different from its explicit non-reusing `bound_socket` helper.
- [Unix socket implementation](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/library/std/src/sys/net/connection/socket/unix.rs): shutdown maps `Both` to `SHUT_RDWR`; descriptor duplication is a separate operation. The reviewed product/fixture path does not invoke it.
- [OwnedFd](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/library/std/src/os/fd/owned.rs): `Drop` calls close and ignores its return. Thus a source-level drop is neither a saved close-success receipt nor a kernel-endpoint-disposal receipt. This does not establish that close returned an error in A.

## Python helper comparator

B's traceback identifies an Xcode Python **3.9** library path, not an exact CPython patch/source revision. **v3.9.6** below is a named public comparator; it is not claimed as deployed provenance.

- [socket.py at v3.9.6](https://github.com/python/cpython/blob/v3.9.6/Lib/socket.py): `socket()` without arguments selects IPv4, stream, protocol zero. The context manager closes it. The constructor does not enable address reuse. `create_server` is a different helper and is not called by B.
- [socketmodule.c at v3.9.6](https://github.com/python/cpython/blob/v3.9.6/Modules/socketmodule.c): `sock_bind` performs address conversion and the bind syscall, and exposes its error. B's saved Errno 48 is more direct evidence than inferring errno from lsof. No option readback was archived for the Python probe.

## Pinned Apple kernel comparator

Revision: **xnu-12377.1.9** in Apple's source distribution. Both saved runs report macOS **27.2 / 26B5112a**. Neither saved record establishes the matching deployed XNU revision, build flags, or network-policy implementation. No mapping from this public tag to those deployed kernels is asserted. This comparator supports a possible mechanism and the limit on the oracle; it cannot supply missing historical state.

- [in_pcb.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-12377.1.9/bsd/netinet/in_pcb.c): `in_pcbbind` changes local lookup when neither reuse flag is set. `in_pcblookup_local` with wildcard matching can include connected local endpoints; without it, the lookup seeks an unconnected endpoint. A conflicting local PCB can return EADDRINUSE without a listening process. Cleanup of a dying PCB has conditions; a close call alone is not that condition. The file also has other EADDRINUSE routes, including namespace reservation. It cannot identify the route taken in A or B. A row with only a matching **remote** port is not a match of the bind's local port in this lookup.
- [tcp_usrreq.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-12377.1.9/bsd/netinet/tcp_usrreq.c): detach can initiate a disconnect that finishes later. `tcp_usrclosed` moves an established connection toward FIN_WAIT_1 and a CLOSE_WAIT connection toward LAST_ACK. This supports A's retained-passive-close candidate. It does not prove that A was in either state at bind, or that B's application shutdown order selected an active closer.
- [uipc_socket2.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-12377.1.9/bsd/kern/uipc_socket2.c): `sonewconn_internal` copies listener socket options to a newly created accepted socket, apart from the listen flag. With the provenance limit above, this explains the expected option difference between accepted A raw sockets and product Mio sockets. Actual accepted-socket options were not measured in either archive.

No fixed TIME_WAIT duration, instant kernel cleanup guarantee, universal restart result, or deployed-kernel bug is inferred from these sources.
