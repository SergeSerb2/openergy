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
    Clipboard,
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
    stamp: Arc<AtomicU64>,
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
        r.set_nonblocking(true).map_err(error)?;
        let stamp = worker.stamp.clone();
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
                        worker.outgoing.retain(|_, t| t.deadline > Instant::now());
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
                stamp,
                thread: std::sync::Mutex::new(Some(thread)),
            }),
            selection,
        })
    }
    /// Select id 0 (CLIPBOARD) or id 1 (PRIMARY) without opening another worker.
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
        self.inner.stamp.load(Ordering::Acquire)
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
    stamp: Arc<AtomicU64>,
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
            stamp: Arc::new(AtomicU64::new(0)),
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
        self.stamp.fetch_add(1, Ordering::AcqRel);
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
        if let Some(index) = index {
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
                } else if self.outgoing.len() < 16 {
                    self.c
                        .change_window_attributes(
                            request.requestor,
                            &xproto::ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
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
            Event::XfixesSelectionNotify(_) => {
                self.stamp.fetch_add(1, Ordering::AcqRel);
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
                        self.outgoing.remove(&key);
                    }
                    self.c.flush().map_err(error)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn next(&mut self, deadline: Instant) -> Result<Event, PlatformError> {
        loop {
            if let Some(e) = self.c.poll_for_event().map_err(error)? {
                return Ok(e);
            }
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err(error("X11 selection transfer timed out"));
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
        self.c
            .delete_property(self.window, self.atoms.property)
            .map_err(error)?
            .check()
            .map_err(error)?;
        self.c
            .convert_selection(self.window, selection, target, self.atoms.property, CURRENT_TIME)
            .map_err(error)?
            .check()
            .map_err(error)?;
        self.c.flush().map_err(error)?;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match self.next(deadline)? {
                Event::SelectionNotify(e)
                    if e.requestor == self.window && e.selection == selection && e.target == target =>
                {
                    if e.property == NONE {
                        return Ok(None);
                    }
                    break;
                }
                e => self.event(e)?,
            }
        }
        let mut reply = self.property(max.max(4))?;
        if reply.type_ != self.atoms.incr {
            return Ok(Some(reply));
        }
        if reply
            .value32()
            .and_then(|mut v| v.next())
            .is_some_and(|n| n as usize > max)
        {
            return Err(error("X11 INCR selection exceeds caller limit"));
        }
        let mut bytes = Vec::new();
        loop {
            match self.next(deadline)? {
                Event::PropertyNotify(e)
                    if e.window == self.window && e.atom == self.atoms.property && e.state == Property::NEW_VALUE =>
                {
                    let chunk = self.property(max.saturating_sub(bytes.len()))?;
                    if chunk.type_ == NONE || chunk.type_ == self.atoms.incr {
                        continue;
                    }
                    if chunk.format != 8 {
                        return Err(error("Unsupported INCR property format"));
                    }
                    if chunk.value.is_empty() {
                        reply.type_ = chunk.type_;
                        reply.format = 8;
                        reply.value = bytes;
                        reply.bytes_after = 0;
                        reply.value_len = reply.value.len() as u32;
                        return Ok(Some(reply));
                    }
                    bytes.extend_from_slice(&chunk.value);
                }
                e => self.event(e)?,
            }
        }
    }
    fn property(&self, max: usize) -> Result<xproto::GetPropertyReply, PlatformError> {
        let reply = self
            .c
            .get_property(
                true,
                self.window,
                self.atoms.property,
                AtomEnum::ANY,
                0,
                (max / 4 + 1).min(u32::MAX as usize) as u32,
            )
            .map_err(error)?
            .reply()
            .map_err(error)?;
        if reply.value.len() > max || reply.bytes_after != 0 {
            let _ = self.c.delete_property(self.window, self.atoms.property);
            let _ = self.c.flush();
            return Err(error("X11 selection exceeds caller limit"));
        }
        self.c.flush().map_err(error)?;
        Ok(reply)
    }
    fn read(&mut self, selection: Selection, max: usize) -> Result<ClipboardData, PlatformError> {
        let selection = self.atoms.selections[selection.index()];
        if self
            .c
            .get_selection_owner(selection)
            .map_err(error)?
            .reply()
            .map_err(error)?
            .owner
            == NONE
        {
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
                if let Some(reply) = self.convert(selection, target, max.saturating_sub(data.total_bytes()))? {
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
                        return Err(error("Converted clipboard exceeds caller limit"));
                    }
                    data.formats.push(ClipboardFormat { kind, bytes });
                }
            }
        }
        Ok(data)
    }
}
