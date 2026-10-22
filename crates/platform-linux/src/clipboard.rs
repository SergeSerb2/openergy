//! ICCCM selections with a hidden owner window and bidirectional INCR transfers.
use crate::{
    image,
    native::{connect, drain, error, wait, wake},
};
use openergy_platform::{ClipboardBackend, ClipboardData, ClipboardFormat, FormatKind, PlatformError};
use std::{
    collections::BTreeMap,
    os::unix::net::UnixStream,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, SyncSender},
        Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use x11rb::{
    connection::Connection,
    protocol::{
        xfixes::{ConnectionExt as _, SelectionEventMask},
        xproto::{
            self, AtomEnum, ConnectionExt as _, CreateWindowAux, EventMask, PropMode, Property, SelectionNotifyEvent,
            SelectionRequestEvent, WindowClass,
        },
        Event,
    },
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
    COPY_DEPTH_FROM_PARENT, CURRENT_TIME, NONE,
};
const LIMIT: usize = 256 * 1024 * 1024;
const CHUNK: usize = 64 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    /// The only selection routed through the v1 shared Backend (clipboard id 0).
    Clipboard,
    /// Compiled local support for a post-v1 extension; v1 does not route clipboard id 1.
    Primary,
}
impl Selection {
    fn index(self) -> usize {
        if self == Self::Clipboard {
            0
        } else {
            1
        }
    }
}
#[derive(Clone)]
pub struct X11Clipboard {
    inner: Arc<Inner>,
    selection: Selection,
}
struct Inner {
    tx: SyncSender<Command>,
    wake: UnixStream,
    stamps: Arc<[AtomicU64; 2]>,
    thread: std::sync::Mutex<Option<JoinHandle<()>>>,
}
enum Command {
    Read(Selection, usize, SyncSender<Result<ClipboardData, PlatformError>>),
    Write(Selection, ClipboardData, SyncSender<Result<(), PlatformError>>),
    Stop,
}
impl X11Clipboard {
    pub fn connect(display: Option<&str>, selection: Selection) -> Result<Self, PlatformError> {
        let worker = Worker::new(display)?;
        let (tx, rx) = mpsc::sync_channel(16);
        let (w, mut r) = UnixStream::pair().map_err(error)?;
        w.set_nonblocking(true).map_err(error)?;
        r.set_nonblocking(true).map_err(error)?;
        let stamps = worker.stamps.clone();
        let thread = std::thread::Builder::new()
            .name("oe-x11-clipboard".into())
            .spawn(move || {
                let mut worker = worker;
                let result = (|| -> Result<(), PlatformError> {
                    loop {
                        while let Ok(cmd) = rx.try_recv() {
                            match cmd {
                                Command::Stop => return Ok(()),
                                Command::Read(selection, max, reply) => {
                                    let result = worker.read(selection, max.min(LIMIT));
                                    let _ = reply.send(result);
                                }
                                Command::Write(selection, data, reply) => {
                                    let _ = reply.send(worker.write(selection, data));
                                }
                            }
                        }
                        while let Some(ev) = worker.c.poll_for_event().map_err(error)? {
                            worker.event(ev)?;
                        }
                        worker.expire_transfers();
                        worker.c.flush().map_err(error)?;
                        wait(&worker.c, &r, if worker.outgoing.is_empty() { -1 } else { 1000 })?;
                        drain(&mut r);
                    }
                })();
                if let Err(e) = result {
                    log::error!("X11 clipboard worker stopped: {e}");
                }
            })
            .map_err(error)?;
        Ok(Self {
            inner: Arc::new(Inner {
                tx,
                wake: w,
                stamps,
                thread: std::sync::Mutex::new(Some(thread)),
            }),
            selection,
        })
    }
    /// Select a local selection without opening another worker.
    /// PRIMARY is a post-v1 extension; `create` routes only CLIPBOARD through the shared backend.
    pub fn for_selection(&self, selection: Selection) -> Self {
        Self {
            inner: self.inner.clone(),
            selection,
        }
    }
    pub fn read_selection(&self, selection: Selection, max_bytes: usize) -> Result<ClipboardData, PlatformError> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.inner
            .tx
            .send(Command::Read(selection, max_bytes, tx))
            .map_err(error)?;
        wake(&self.inner.wake);
        rx.recv().map_err(error)?
    }
    pub fn write_selection(&self, selection: Selection, data: &ClipboardData) -> Result<(), PlatformError> {
        if data.total_bytes() > LIMIT {
            return Err(error("X11 clipboard exceeds 256 MiB safety cap"));
        }
        let (tx, rx) = mpsc::sync_channel(1);
        self.inner
            .tx
            .send(Command::Write(selection, data.clone(), tx))
            .map_err(error)?;
        wake(&self.inner.wake);
        rx.recv().map_err(error)?
    }
}
impl ClipboardBackend for X11Clipboard {
    fn change_stamp(&self) -> u64 {
        self.inner.stamps[self.selection.index()].load(Ordering::Acquire)
    }
    fn read(&mut self, max_bytes: usize) -> Result<ClipboardData, PlatformError> {
        self.read_selection(self.selection, max_bytes)
    }
    fn write(&mut self, data: &ClipboardData) -> Result<(), PlatformError> {
        self.write_selection(self.selection, data)
    }
}
impl Drop for Inner {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Stop);
        wake(&self.wake);
        if let Ok(h) = self.thread.get_mut() {
            if let Some(h) = h.take() {
                let _ = h.join();
            }
        }
    }
}
struct Atoms {
    selections: [u32; 2],
    targets: u32,
    utf8: u32,
    plain: u32,
    text: u32,
    html: u32,
    png: u32,
    bmp: u32,
    incr: u32,
    property: u32,
}
impl Atoms {
    fn new(c: &RustConnection) -> Result<Self, PlatformError> {
        let atom = |name: &[u8]| -> Result<u32, PlatformError> {
            Ok(c.intern_atom(false, name).map_err(error)?.reply().map_err(error)?.atom)
        };
        Ok(Self {
            selections: [atom(b"CLIPBOARD")?, AtomEnum::PRIMARY.into()],
            targets: atom(b"TARGETS")?,
            utf8: atom(b"UTF8_STRING")?,
            plain: atom(b"text/plain;charset=utf-8")?,
            text: atom(b"text/plain")?,
            html: atom(b"text/html")?,
            png: atom(b"image/png")?,
            bmp: atom(b"image/bmp")?,
            incr: atom(b"INCR")?,
            property: atom(b"_OPENERGY_SELECTION")?,
        })
    }
}
struct Transfer {
    target: u32,
    bytes: Arc<Vec<u8>>,
    offset: usize,
    deadline: Instant,
}
struct Worker {
    c: RustConnection,
    window: u32,
    atoms: Atoms,
    stamps: Arc<[AtomicU64; 2]>,
    data: [BTreeMap<u32, Arc<Vec<u8>>>; 2],
    outgoing: BTreeMap<(u32, u32), Transfer>,
}
impl Worker {
    fn new(display: Option<&str>) -> Result<Self, PlatformError> {
        let (c, root) = connect(display)?;
        let atoms = Atoms::new(&c)?;
        let window = c.generate_id().map_err(error)?;
        c.create_window(
            COPY_DEPTH_FROM_PARENT,
            window,
            root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE),
        )
        .map_err(error)?
        .check()
        .map_err(error)?;
        c.xfixes_query_version(2, 0).map_err(error)?.reply().map_err(error)?;
        for selection in atoms.selections {
            c.xfixes_select_selection_input(
                window,
                selection,
                SelectionEventMask::SET_SELECTION_OWNER
                    | SelectionEventMask::SELECTION_WINDOW_DESTROY
                    | SelectionEventMask::SELECTION_CLIENT_CLOSE,
            )
            .map_err(error)?
            .check()
            .map_err(error)?;
        }
        c.flush().map_err(error)?;
        Ok(Self {
            c,
            window,
            atoms,
            stamps: Arc::new([AtomicU64::new(0), AtomicU64::new(0)]),
            data: [BTreeMap::new(), BTreeMap::new()],
            outgoing: BTreeMap::new(),
        })
    }
    fn write(&mut self, selection: Selection, data: ClipboardData) -> Result<(), PlatformError> {
        let mut targets = BTreeMap::new();
        for format in data.formats {
            match format.kind {
                FormatKind::TextUtf8 => {
                    std::str::from_utf8(&format.bytes).map_err(error)?;
                    let value = Arc::new(format.bytes);
                    for atom in [self.atoms.utf8, self.atoms.plain, self.atoms.text] {
                        targets.insert(atom, value.clone());
                    }
                }
                FormatKind::HtmlUtf8 => {
                    std::str::from_utf8(&format.bytes).map_err(error)?;
                    targets.insert(self.atoms.html, Arc::new(format.bytes));
                }
                FormatKind::Dib => {
                    let bmp = image::dib_to_bmp(&format.bytes)?;
                    let png = image::dib_to_png(&format.bytes)?;
                    targets.insert(self.atoms.bmp, Arc::new(bmp));
                    targets.entry(self.atoms.png).or_insert_with(|| Arc::new(png));
                }
                FormatKind::Png => {
                    let dib = image::png_to_dib(&format.bytes, LIMIT)?;
                    targets.insert(self.atoms.bmp, Arc::new(image::dib_to_bmp(&dib)?));
                    targets.insert(self.atoms.png, Arc::new(format.bytes));
                }
            }
        }
        self.c
            .set_selection_owner(self.window, self.atoms.selections[selection.index()], CURRENT_TIME)
            .map_err(error)?
            .check()
            .map_err(error)?;
        if self
            .c
            .get_selection_owner(self.atoms.selections[selection.index()])
            .map_err(error)?
            .reply()
            .map_err(error)?
            .owner
            != self.window
        {
            return Err(error("X11 selection ownership was refused"));
        }
        self.data[selection.index()] = targets;
        self.stamps[selection.index()].fetch_add(1, Ordering::AcqRel);
        Ok(())
    }
    fn request(&mut self, request: SelectionRequestEvent) -> Result<(), PlatformError> {
        let property = if request.property == NONE {
            request.target
        } else {
            request.property
        };
        let index = self.atoms.selections.iter().position(|s| *s == request.selection);
        let mut accepted = false;
        if let Some(index) = index.filter(|_| !self.outgoing.contains_key(&(request.requestor, property))) {
            if request.target == self.atoms.targets {
                let mut atoms: Vec<u32> = self.data[index].keys().copied().collect();
                atoms.push(self.atoms.targets);
                self.c
                    .change_property32(PropMode::REPLACE, request.requestor, property, AtomEnum::ATOM, &atoms)
                    .map_err(error)?
                    .check()
                    .map_err(error)?;
                accepted = true;
            } else if let Some(bytes) = self.data[index].get(&request.target).cloned() {
                if bytes.len() <= CHUNK {
                    self.c
                        .change_property8(PropMode::REPLACE, request.requestor, property, request.target, &bytes)
                        .map_err(error)?
                        .check()
                        .map_err(error)?;
                    accepted = true;
                } else if self.outgoing.len() < 16
                    && bytes.len() <= LIMIT
                    && self.outgoing.values().map(|t| t.bytes.len()).sum::<usize>() <= LIMIT.saturating_sub(bytes.len())
                {
                    self.c
                        .change_window_attributes(
                            request.requestor,
                            &xproto::ChangeWindowAttributesAux::new()
                                .event_mask(EventMask::PROPERTY_CHANGE | EventMask::STRUCTURE_NOTIFY),
                        )
                        .map_err(error)?
                        .check()
                        .map_err(error)?;
                    self.c
                        .change_property32(
                            PropMode::REPLACE,
                            request.requestor,
                            property,
                            self.atoms.incr,
                            &[bytes.len() as u32],
                        )
                        .map_err(error)?
                        .check()
                        .map_err(error)?;
                    self.outgoing.insert(
                        (request.requestor, property),
                        Transfer {
                            target: request.target,
                            bytes,
                            offset: 0,
                            deadline: Instant::now() + Duration::from_secs(10),
                        },
                    );
                    accepted = true;
                }
            }
        }
        self.c
            .send_event(
                false,
                request.requestor,
                EventMask::NO_EVENT,
                SelectionNotifyEvent {
                    response_type: xproto::SELECTION_NOTIFY_EVENT,
                    sequence: 0,
                    time: request.time,
                    requestor: request.requestor,
                    selection: request.selection,
                    target: request.target,
                    property: if accepted { property } else { NONE },
                },
            )
            .map_err(error)?
            .check()
            .map_err(error)?;
        self.c.flush().map_err(error)
    }
    fn event(&mut self, event: Event) -> Result<(), PlatformError> {
        match event {
            Event::SelectionRequest(e) => {
                // A requestor may disappear at any time; do not kill the owner.
                if let Err(e) = self.request(e) {
                    log::debug!("X11 selection request ended: {e}");
                }
            }
            Event::XfixesSelectionNotify(e) => {
                // write() advances the stamp before acknowledging its caller.
                // Counting the queued notification again races own-write loop
                // prevention and makes our own selection appear externally new.
                if e.owner == self.window {
                    return Ok(());
                }
                if let Some(index) = self
                    .atoms
                    .selections
                    .iter()
                    .position(|selection| *selection == e.selection)
                {
                    self.stamps[index].fetch_add(1, Ordering::AcqRel);
                }
            }
            Event::SelectionClear(e) => {
                if let Some(index) = self.atoms.selections.iter().position(|s| *s == e.selection) {
                    if self
                        .c
                        .get_selection_owner(e.selection)
                        .map_err(error)?
                        .reply()
                        .map_err(error)?
                        .owner
                        != self.window
                    {
                        self.data[index].clear();
                    }
                }
            }
            Event::PropertyNotify(e) if e.state == Property::DELETE => {
                let key = (e.window, e.atom);
                if let Some(transfer) = self.outgoing.get_mut(&key) {
                    let end = (transfer.offset + CHUNK).min(transfer.bytes.len());
                    let result = self
                        .c
                        .change_property8(
                            PropMode::REPLACE,
                            e.window,
                            e.atom,
                            transfer.target,
                            &transfer.bytes[transfer.offset..end],
                        )
                        .map_err(error)
                        .and_then(|c| c.check().map_err(error));
                    let done = end == transfer.offset || result.is_err();
                    transfer.offset = end;
                    transfer.deadline = Instant::now() + Duration::from_secs(10);
                    if done {
                        self.finish_transfer(key);
                    }
                    self.c.flush().map_err(error)?;
                }
            }
            Event::DestroyNotify(e) => {
                self.outgoing.retain(|(window, _), _| *window != e.window);
            }
            _ => {}
        }
        Ok(())
    }
    fn finish_transfer(&mut self, key: (u32, u32)) {
        self.outgoing.remove(&key);
        if !self.outgoing.keys().any(|(window, _)| *window == key.0) {
            // Event selection is per client. Stop observing a foreign window
            // once its last transfer ends, without changing its owner's mask.
            if let Ok(cookie) = self.c.change_window_attributes(
                key.0,
                &xproto::ChangeWindowAttributesAux::new().event_mask(EventMask::NO_EVENT),
            ) {
                let _ = cookie.check();
            }
        }
    }
    fn expire_transfers(&mut self) {
        let expired: Vec<_> = self
            .outgoing
            .iter()
            .filter(|(_, t)| t.deadline <= Instant::now())
            .map(|(key, _)| *key)
            .collect();
        for key in expired {
            self.finish_transfer(key);
        }
    }
    fn next(&mut self, deadline: Instant) -> Result<Event, PlatformError> {
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err(error("X11 selection transfer timed out"));
            }
            if let Some(e) = self.c.poll_for_event().map_err(error)? {
                return Ok(e);
            }
            // The event worker is the only reader. Use the X socket directly while a request is active.
            use std::os::fd::AsRawFd;
            let mut fd = libc::pollfd {
                fd: self.c.stream().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // Safety: fd is valid for the duration of this synchronous poll call.
            let result = unsafe { libc::poll(&mut fd, 1, left.as_millis().min(i32::MAX as u128) as i32) };
            if result < 0 && std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted {
                return Err(error(std::io::Error::last_os_error()));
            }
        }
    }
    fn convert(
        &mut self,
        selection: u32,
        target: u32,
        max: usize,
    ) -> Result<Option<xproto::GetPropertyReply>, PlatformError> {
        // A separate requestor window confines late SelectionNotify/INCR chunks
        // to this conversion. Destroying it also cancels a refused/failed read.
        // Reusing the owner's property lets an aborted INCR corrupt later reads.
        let window = self.c.generate_id().map_err(error)?;
        self.c
            .create_window(
                COPY_DEPTH_FROM_PARENT,
                window,
                self.window,
                0,
                0,
                1,
                1,
                0,
                WindowClass::INPUT_OUTPUT,
                0,
                &CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE),
            )
            .map_err(error)?
            .check()
            .map_err(error)?;
        let result = self.convert_on(window, selection, target, max);
        let cleanup = self
            .c
            .destroy_window(window)
            .map_err(error)
            .and_then(|c| c.check().map_err(error));
        match result {
            Err(e) => Err(e),
            Ok(reply) => {
                cleanup?;
                Ok(reply)
            }
        }
    }
    fn convert_on(
        &mut self,
        window: u32,
        selection: u32,
        target: u32,
        max: usize,
    ) -> Result<Option<xproto::GetPropertyReply>, PlatformError> {
        self.c
            .delete_property(window, self.atoms.property)
            .map_err(error)?
            .check()
            .map_err(error)?;
        self.c
            .convert_selection(window, selection, target, self.atoms.property, CURRENT_TIME)
            .map_err(error)?
            .check()
            .map_err(error)?;
        self.c.flush().map_err(error)?;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match self.next(deadline)? {
                Event::SelectionNotify(e)
                    if e.requestor == window && e.selection == selection && e.target == target =>
                {
                    if e.property == NONE {
                        return Ok(None);
                    }
                    if e.property != self.atoms.property {
                        return Err(error("X11 selection returned an unexpected property"));
                    }
                    break;
                }
                e => self.event(e)?,
            }
        }
        let mut reply = self.property(window, max.max(4))?;
        let expected_type = if target == self.atoms.targets {
            u32::from(AtomEnum::ATOM)
        } else {
            target
        };
        let expected_format = if target == self.atoms.targets { 32 } else { 8 };
        if reply.type_ != self.atoms.incr {
            if reply.type_ == NONE {
                return Err(PlatformError::ClipboardUnavailable);
            }
            if reply.type_ != expected_type || reply.format != expected_format {
                return Err(error("X11 selection returned an invalid property type or format"));
            }
            if reply.value.len() > max {
                return Err(PlatformError::ClipboardTooLarge {
                    bytes: reply.value.len(),
                    limit: max,
                });
            }
            return Ok(Some(reply));
        }
        // xclip 0.13 omits the advisory byte count. Treat that as unknown,
        // while rejecting malformed widths and multiple counts.
        if reply.format != 32 || reply.value_len > 1 {
            return Err(error("X11 INCR header has an invalid byte count"));
        }
        let advertised = reply.value32().and_then(|mut values| values.next()).unwrap_or(0) as usize;
        if advertised > LIMIT {
            return Err(PlatformError::ClipboardTooLarge {
                bytes: advertised,
                limit: max,
            });
        }
        let mut rejected = (advertised > max).then_some(advertised);
        let mut received = 0usize;
        let mut bytes = Vec::new();
        loop {
            match self.next(deadline)? {
                Event::PropertyNotify(e)
                    if e.window == window && e.atom == self.atoms.property && e.state == Property::NEW_VALUE =>
                {
                    // Some owners (including xclip) cannot serve another read
                    // until INCR completes. Drain an oversized transfer without
                    // retaining its contents, under the same deadline and cap.
                    let chunk = match self.property(
                        window,
                        if rejected.is_some() {
                            CHUNK
                        } else {
                            max.saturating_sub(bytes.len())
                        },
                    ) {
                        Ok(chunk) => chunk,
                        Err(PlatformError::ClipboardTooLarge { bytes: size, .. }) => {
                            received = received.saturating_add(size);
                            if received > LIMIT {
                                return Err(PlatformError::ClipboardTooLarge {
                                    bytes: received,
                                    limit: max,
                                });
                            }
                            rejected = Some(advertised.max(received));
                            bytes.clear();
                            continue;
                        }
                        Err(e) => return Err(e),
                    };
                    // The initial INCR property's NEW_VALUE event may still be
                    // queued after SelectionNotify and after we deleted it.
                    if chunk.type_ == NONE {
                        continue;
                    }
                    if chunk.type_ != expected_type || chunk.format != expected_format {
                        return Err(error("X11 INCR chunk changed property type or format"));
                    }
                    if chunk.value.is_empty() {
                        if let Some(size) = rejected {
                            return Err(PlatformError::ClipboardTooLarge {
                                bytes: size.max(received),
                                limit: max,
                            });
                        }
                        reply.type_ = chunk.type_;
                        reply.format = expected_format;
                        reply.value = bytes;
                        reply.bytes_after = 0;
                        reply.value_len = (reply.value.len() / (usize::from(expected_format) / 8)) as u32;
                        return Ok(Some(reply));
                    }
                    received = received.saturating_add(chunk.value.len());
                    if received > LIMIT {
                        return Err(PlatformError::ClipboardTooLarge {
                            bytes: received,
                            limit: max,
                        });
                    }
                    if rejected.is_none() {
                        bytes.extend_from_slice(&chunk.value);
                    }
                }
                e => self.event(e)?,
            }
        }
    }
    fn property(&self, window: u32, max: usize) -> Result<xproto::GetPropertyReply, PlatformError> {
        let reply = self
            .c
            .get_property(
                true,
                window,
                self.atoms.property,
                AtomEnum::ANY,
                0,
                (max / 4 + 1).min(u32::MAX as usize) as u32,
            )
            .map_err(error)?
            .reply()
            .map_err(error)?;
        if reply.value.len() > max || reply.bytes_after != 0 {
            let _ = self.c.delete_property(window, self.atoms.property);
            let _ = self.c.flush();
            return Err(PlatformError::ClipboardTooLarge {
                bytes: reply.value.len().saturating_add(reply.bytes_after as usize),
                limit: max,
            });
        }
        self.c.flush().map_err(error)?;
        Ok(reply)
    }
    fn owner(&self, selection: u32) -> Result<u32, PlatformError> {
        Ok(self
            .c
            .get_selection_owner(selection)
            .map_err(error)?
            .reply()
            .map_err(error)?
            .owner)
    }
    fn pending_events(&mut self) -> Result<(), PlatformError> {
        while let Some(event) = self.c.poll_for_event().map_err(error)? {
            self.event(event)?;
        }
        Ok(())
    }
    fn check_snapshot(&mut self, index: usize, owner: u32, stamp: u64) -> Result<(), PlatformError> {
        let current = self.owner(self.atoms.selections[index])?;
        // The owner query is a server round trip. Process notifications queued
        // before its reply, including a same-owner reacquisition of new data.
        self.pending_events()?;
        if current != owner || self.stamps[index].load(Ordering::Acquire) != stamp {
            return Err(PlatformError::ClipboardUnavailable);
        }
        Ok(())
    }
    fn read(&mut self, selection: Selection, max: usize) -> Result<ClipboardData, PlatformError> {
        let index = selection.index();
        let selection = self.atoms.selections[index];
        let owner = self.owner(selection)?;
        self.pending_events()?;
        let stamp = self.stamps[index].load(Ordering::Acquire);
        if owner == NONE || (owner == self.window && self.data[index].is_empty()) {
            // No owner is a clear. We also know that our own empty target map
            // came from an explicit write(empty), rather than unsupported data.
            self.check_snapshot(index, owner, stamp)?;
            return Ok(ClipboardData::default());
        }
        let targets = self
            .convert(selection, self.atoms.targets, 65536)?
            .and_then(|r| r.value32().map(|v| v.collect::<Vec<_>>()))
            .unwrap_or_else(|| vec![self.atoms.utf8]);
        let mut data = ClipboardData::default();
        for (kind, choices) in [
            (
                FormatKind::TextUtf8,
                vec![self.atoms.utf8, self.atoms.plain, self.atoms.text],
            ),
            (FormatKind::HtmlUtf8, vec![self.atoms.html]),
            (FormatKind::Png, vec![self.atoms.png, self.atoms.bmp]),
        ] {
            if let Some(target) = choices.into_iter().find(|t| targets.contains(t)) {
                let reply = self
                    .convert(selection, target, max.saturating_sub(data.total_bytes()))?
                    .ok_or(PlatformError::ClipboardUnavailable)?;
                if reply.format != 8 {
                    return Err(error("X11 selection has invalid format"));
                }
                let (kind, bytes) = if target == self.atoms.bmp {
                    (FormatKind::Dib, image::bmp_to_dib(&reply.value)?)
                } else if kind == FormatKind::Png {
                    (
                        FormatKind::Dib,
                        image::png_to_dib(&reply.value, max.saturating_sub(data.total_bytes()))?,
                    )
                } else {
                    std::str::from_utf8(&reply.value).map_err(error)?;
                    (kind, reply.value)
                };
                if bytes.len() > max.saturating_sub(data.total_bytes()) {
                    return Err(PlatformError::ClipboardTooLarge {
                        bytes: data.total_bytes().saturating_add(bytes.len()),
                        limit: max,
                    });
                }
                data.formats.push(ClipboardFormat { kind, bytes });
            }
        }
        self.check_snapshot(index, owner, stamp)?;
        if data.formats.is_empty() {
            return Err(PlatformError::ClipboardUnavailable);
        }
        Ok(data)
    }
}
