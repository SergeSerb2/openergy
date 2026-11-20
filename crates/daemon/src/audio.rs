//! Local audio streams. Device setup runs off the input/network thread.
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, DeviceId, FromSample, SampleFormat, SizedSample, Stream, StreamConfig, SupportedBufferSize,
};
use mio::Waker;
use openergy_protocol::{MICROPHONE_BYTES, MICROPHONE_RATE};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

type Pcm = [u8; MICROPHONE_BYTES];
pub const CAPTURE_QUEUE: usize = 5;
type CaptureQueue = Arc<Mutex<VecDeque<(Instant, Pcm)>>>;

pub fn list_devices() -> Result<(), String> {
    let mut devices = Vec::new();
    let mut errors = Vec::new();
    for id in cpal::available_hosts() {
        let candidates = match cpal::host_from_id(id).and_then(|host| host.devices()) {
            Ok(devices) => devices,
            Err(error) => {
                errors.push(format!("{id}: {error}"));
                continue;
            }
        };
        for device in candidates {
            let (Ok(id), Ok(description)) = (device.id(), device.description()) else {
                continue;
            };
            let input = device.supports_input();
            let output = device.supports_output();
            if input || output {
                devices.push(serde_json::json!({
                    "id": id.to_string(), "name": description.name(), "input": input, "output": output
                }));
            }
        }
    }
    if devices.is_empty() && !errors.is_empty() {
        return Err(errors.join(", "));
    }
    println!("{}", serde_json::to_string_pretty(&devices).map_err(|e| e.to_string())?);
    Ok(())
}

enum DeviceIo {
    Capture(CaptureQueue),
    Playback(Receiver<Pcm>),
}

struct Request {
    id: String,
    io: DeviceIo,
    state: Arc<Mutex<StreamState>>,
    active: Arc<AtomicBool>,
    waker: Arc<Waker>,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub enum StreamState {
    #[default]
    Starting,
    Ready,
    Failed(String),
}

impl Request {
    fn open(self) -> Option<Stream> {
        let state = self.state.clone();
        let waker = self.waker.clone();
        let (stream, ready) = {
            let result = self.open_device();
            let mut status = state.lock().unwrap_or_else(|e| e.into_inner());
            match result {
                Ok(stream) => {
                    let ready = stream.is_some() && matches!(*status, StreamState::Starting);
                    if ready {
                        *status = StreamState::Ready;
                    }
                    (stream, ready)
                }
                Err(error) => {
                    *status = StreamState::Failed(error);
                    (None, false)
                }
            }
        };
        let _ = waker.wake();
        // Stream teardown can wait for a backend error callback using state.
        stream.filter(|_| ready)
    }

    fn open_device(self) -> Result<Option<Stream>, String> {
        let active = self.active.clone();
        if !active.load(Ordering::Acquire) {
            return Ok(None);
        }
        let id: DeviceId = self.id.parse::<DeviceId>().map_err(|e| e.to_string())?;
        let host = cpal::host_from_id(id.host()).map_err(|e| e.to_string())?;
        let device = host.device_by_id(&id).ok_or("Audio device is unavailable")?;
        let supported = match &self.io {
            DeviceIo::Capture(_) if device.supports_input() => device.default_input_config(),
            DeviceIo::Playback(_) if device.supports_output() => device.default_output_config(),
            _ => return Err("Audio device does not support the selected direction".into()),
        }
        .map_err(|e| e.to_string())?;
        let mut config = supported.config();
        if config.channels == 0 || !(8_000..=384_000).contains(&config.sample_rate) {
            return Err("Unsupported audio channel count or sample rate".into());
        }
        // Backends that publish a buffer range support explicit latency requests.
        if let SupportedBufferSize::Range { min, max } = *supported.buffer_size() {
            config.buffer_size = cpal::BufferSize::Fixed((config.sample_rate / 50).clamp(min, max));
        }
        if !active.load(Ordering::Acquire) {
            return Ok(None);
        }
        let stream = match supported.sample_format() {
            SampleFormat::I8 => self.build::<i8>(device, config),
            SampleFormat::I16 => self.build::<i16>(device, config),
            SampleFormat::I24 => self.build::<cpal::I24>(device, config),
            SampleFormat::I32 => self.build::<i32>(device, config),
            SampleFormat::U8 => self.build::<u8>(device, config),
            SampleFormat::U16 => self.build::<u16>(device, config),
            SampleFormat::U32 => self.build::<u32>(device, config),
            SampleFormat::F32 => self.build::<f32>(device, config),
            SampleFormat::F64 => self.build::<f64>(device, config),
            format => return Err(format!("Unsupported audio sample format {format}")),
        }
        .map_err(|e| e.to_string())?;
        if !active.load(Ordering::Acquire) {
            return Ok(None);
        }
        stream.play().map_err(|e| e.to_string())?;
        Ok(Some(stream))
    }

    fn build<T>(self, device: Device, config: StreamConfig) -> Result<Stream, cpal::Error>
    where
        T: SizedSample + FromSample<f32>,
        f32: FromSample<T>,
    {
        let channels = usize::from(config.channels);
        let rate = config.sample_rate;
        let waker = self.waker.clone();
        let active = self.active.clone();
        let error = move |error: cpal::Error| {
            self.active.store(false, Ordering::Release);
            *self.state.lock().unwrap_or_else(|e| e.into_inner()) = StreamState::Failed(error.to_string());
            let _ = self.waker.wake();
        };
        match self.io {
            DeviceIo::Capture(queue) => {
                let mut pcm = [0; MICROPHONE_BYTES];
                let mut pos = 0;
                let mut phase = 0;
                let mut previous = 0.0;
                let input = move |samples: &[T], _: &cpal::InputCallbackInfo| {
                    if !active.load(Ordering::Acquire) {
                        return;
                    }
                    for frame in samples.chunks_exact(channels) {
                        let current = frame.iter().map(|s| s.to_sample::<f32>()).sum::<f32>() / channels as f32;
                        phase += MICROPHONE_RATE;
                        while phase >= rate {
                            phase -= rate;
                            let fraction = 1.0 - phase as f32 / MICROPHONE_RATE as f32;
                            let sample = previous + (current - previous) * fraction;
                            let sample = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                            pcm[pos..pos + 2].copy_from_slice(&sample.to_le_bytes());
                            pos += 2;
                            if pos == MICROPHONE_BYTES {
                                if let Ok(mut pending) = queue.try_lock() {
                                    if pending.len() == CAPTURE_QUEUE {
                                        pending.pop_front();
                                    }
                                    pending.push_back((Instant::now(), pcm));
                                    drop(pending);
                                    let _ = waker.wake();
                                }
                                pos = 0;
                            }
                        }
                        previous = current;
                    }
                };
                // CPAL device backends can quantize the callback size; the
                // packetizer above is independent of that size.
                device.build_input_stream(config, input, error, None)
            }
            DeviceIo::Playback(rx) => {
                let mut pcm = [0; MICROPHONE_BYTES];
                let mut pos = MICROPHONE_BYTES;
                let mut phase = 0;
                let mut previous = 0.0;
                let mut next = 0.0;
                device.build_output_stream(
                    config,
                    move |samples: &mut [T], _| {
                        if !active.load(Ordering::Acquire) {
                            samples.fill(T::from_sample(0.0));
                            return;
                        }
                        let mut empty = false;
                        for frame in samples.chunks_exact_mut(channels) {
                            while phase >= rate {
                                phase -= rate;
                                previous = next;
                                if pos == MICROPHONE_BYTES && !empty {
                                    if let Ok(packet) = rx.try_recv() {
                                        pcm = packet;
                                        pos = 0;
                                    } else {
                                        empty = true;
                                    }
                                }
                                next = if pos < MICROPHONE_BYTES {
                                    let bytes = [pcm[pos], pcm[pos + 1]];
                                    pos += 2;
                                    f32::from(i16::from_le_bytes(bytes)) / 32768.0
                                } else {
                                    0.0
                                };
                            }
                            let sample = previous + (next - previous) * (phase as f32 / rate as f32);
                            frame.fill(T::from_sample(sample));
                            phase += MICROPHONE_RATE;
                        }
                    },
                    error,
                    None,
                )
            }
        }
    }
}

enum Work {
    Stream(Request),
    Stop,
    Exit,
}

struct Worker {
    pending: Arc<Mutex<Option<Work>>>,
    thread: Option<JoinHandle<()>>,
    done: Receiver<()>,
}

impl Worker {
    fn start() -> std::io::Result<Self> {
        let pending = Arc::new(Mutex::new(None));
        let inbox = pending.clone();
        let (complete, done) = mpsc::channel();
        let thread = thread::Builder::new().name("openergy-audio".into()).spawn(move || {
            let mut stream: Option<Stream> = None;
            loop {
                let work = inbox.lock().unwrap_or_else(|e| e.into_inner()).take();
                match work {
                    Some(Work::Stream(request)) => {
                        drop(stream.take());
                        stream = request.open();
                    }
                    Some(Work::Stop) => drop(stream.take()),
                    Some(Work::Exit) => break,
                    None => thread::park(),
                }
            }
            drop(stream);
            let _ = complete.send(());
        })?;
        Ok(Self {
            pending,
            thread: Some(thread),
            done,
        })
    }

    fn submit(&self, work: Work) {
        *self.pending.lock().unwrap_or_else(|e| e.into_inner()) = Some(work);
        if let Some(thread) = &self.thread {
            thread.thread().unpark();
        }
    }

    fn join_until(&mut self, deadline: Instant) {
        if !matches!(
            self.done
                .recv_timeout(deadline.saturating_duration_since(Instant::now())),
            Err(mpsc::RecvTimeoutError::Timeout)
        ) {
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.submit(Work::Exit);
    }
}

struct Port<T> {
    id: String,
    channel: T,
    state: Arc<Mutex<StreamState>>,
    active: Arc<AtomicBool>,
    failure_handled: bool,
}

impl<T> Drop for Port<T> {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Release);
    }
}

struct Endpoint<T> {
    worker: Option<Worker>,
    port: Option<Port<T>>,
}

impl<T> Default for Endpoint<T> {
    fn default() -> Self {
        Self {
            worker: None,
            port: None,
        }
    }
}

impl<T> Endpoint<T> {
    fn set(&mut self, id: Option<&str>, waker: &Arc<Waker>, make_io: impl FnOnce() -> (T, DeviceIo)) -> bool {
        if self.port.as_ref().map(|port| port.id.as_str()) == id {
            return false;
        }
        drop(self.port.take());
        self.port = id.map(|id| {
            let (channel, io) = make_io();
            let state = Arc::new(Mutex::new(StreamState::Starting));
            let active = Arc::new(AtomicBool::new(true));
            let request = Request {
                id: id.into(),
                io,
                state: state.clone(),
                active: active.clone(),
                waker: waker.clone(),
            };
            let worker = match &self.worker {
                Some(worker) => Ok(worker),
                None => Worker::start().map(|worker| self.worker.insert(worker) as &Worker),
            };
            match worker {
                Ok(worker) => worker.submit(Work::Stream(request)),
                Err(error) => *state.lock().unwrap_or_else(|e| e.into_inner()) = StreamState::Failed(error.to_string()),
            }
            Port {
                id: id.into(),
                channel,
                state,
                active,
                failure_handled: false,
            }
        });
        if self.port.is_none() {
            if let Some(worker) = &self.worker {
                worker.submit(Work::Stop);
            }
        }
        true
    }

    fn state(&self) -> Option<StreamState> {
        self.port
            .as_ref()
            .map(|port| port.state.lock().unwrap_or_else(|e| e.into_inner()).clone())
    }

    fn stop_failed(&mut self) -> Option<StreamState> {
        let state = self.state();
        if matches!(state, Some(StreamState::Failed(_))) {
            if let Some(port) = self.port.as_mut().filter(|port| !port.failure_handled) {
                port.failure_handled = true;
                if let Some(worker) = &self.worker {
                    worker.submit(Work::Stop);
                }
            }
        }
        state
    }
}

#[derive(Default)]
pub struct Audio {
    input: Endpoint<CaptureQueue>,
    output: Endpoint<SyncSender<Pcm>>,
}

impl Audio {
    pub fn configure(&mut self, input: Option<&str>, output: Option<&str>, waker: &Arc<Waker>) -> (bool, bool) {
        let input_changed = self.input.set(input, waker, || {
            let queue = Arc::new(Mutex::new(VecDeque::with_capacity(CAPTURE_QUEUE)));
            (queue.clone(), DeviceIo::Capture(queue))
        });
        let output_changed = self.output.set(output, waker, || {
            let (tx, rx) = mpsc::sync_channel(4);
            (tx, DeviceIo::Playback(rx))
        });
        (input_changed, output_changed)
    }

    pub fn stream_states(&mut self) -> (Option<StreamState>, Option<StreamState>) {
        (self.input.stop_failed(), self.output.stop_failed())
    }

    pub fn take_capture(&self) -> Option<Vec<u8>> {
        let mut queue = self.input.port.as_ref()?.channel.lock().ok()?;
        for _ in 0..CAPTURE_QUEUE {
            let (captured, pcm) = queue.pop_front()?;
            if captured.elapsed() <= Duration::from_millis(80) {
                drop(queue);
                return Some(Vec::from(pcm));
            }
        }
        None
    }

    pub fn shutdown(&mut self) {
        drop(self.input.port.take());
        drop(self.output.port.take());
        let workers = [self.input.worker.take(), self.output.worker.take()];
        let deadline = Instant::now() + Duration::from_millis(250);
        for worker in workers.iter().flatten() {
            worker.submit(Work::Exit);
        }
        for mut worker in workers.into_iter().flatten() {
            worker.join_until(deadline);
        }
    }

    pub fn output_matches(&self, id: Option<&str>) -> bool {
        self.output.port.as_ref().is_some_and(|port| {
            Some(port.id.as_str()) == id
                && port.active.load(Ordering::Acquire)
                && matches!(self.output.state(), Some(StreamState::Ready))
        })
    }

    pub fn play(&self, pcm: Vec<u8>) {
        if let Some((port, pcm)) = self.output.port.as_ref().zip(Pcm::try_from(pcm).ok()) {
            let _ = port.channel.try_send(pcm);
        }
    }
}
