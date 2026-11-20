//! Microphone capability negotiation and routing for the authenticated input session.
use super::{ConnId, Message, NetCommand, Priority, Role, Runtime};
use crate::audio::{Audio, StreamState, CAPTURE_QUEUE};
use openergy_config::MicrophoneCfg;
use openergy_ipc::{IpcError, MicrophoneInfo};
use openergy_net::{Fingerprint, TrustStore};
use openergy_protocol::MICROPHONE_OPTION;
use serde_json::{json, Value};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

#[derive(Default)]
pub(super) struct Peer {
    offered: bool,
    accepted: bool,
    receive_stream: u32,
}

#[derive(Default)]
pub(super) struct Sharing {
    audio: Audio,
    conn: Option<ConnId>,
    advertised: Option<u32>,
    last_stream: u32,
    status: MicrophoneInfo,
    controller_pid: Option<i64>,
    armed: Option<(String, String)>,
    trust_checked: Option<Instant>,
    peer_trusted: bool,
}

impl Sharing {
    pub(super) fn shutdown(&mut self) {
        self.audio.shutdown();
    }
    pub(super) fn new(cfg: &MicrophoneCfg, controller_pid: Option<i64>, enable_capture: bool) -> Self {
        let mut sharing = Self {
            controller_pid,
            ..Self::default()
        };
        if enable_capture && controller_pid.is_none() {
            sharing.armed = cfg.peer_fingerprint.clone().zip(cfg.input_device.clone());
        }
        sharing
    }

    pub(super) fn disarm(&mut self) {
        self.armed = None;
    }

    pub(super) fn invalidate_trust(&mut self) {
        self.trust_checked = None;
        self.peer_trusted = false;
    }
}

impl Runtime {
    pub(super) fn arm_microphone(&mut self, caller_pid: Option<i64>, params: &Value) -> Result<Value, IpcError> {
        if caller_pid.is_none() || caller_pid != self.microphone.controller_pid {
            return Err(IpcError::new(
                "permission_denied",
                "Start microphone sharing in the local Openergy app",
            ));
        }
        self.microphone.trust_checked = None;
        self.refresh_microphone_trust();
        if !self.microphone.peer_trusted
            || !self.enabled
            || self.suspended()
            || !self.cfg.tls.enabled
            || (self.cfg.role == Role::Server && !self.cfg.tls.require_client_trust)
        {
            return Err(IpcError::new(
                "permission_denied",
                "Connect with TLS and approve the selected microphone peer first",
            ));
        }
        let cfg = &self.cfg.microphone;
        let requested = params["peer_fingerprint"].as_str().zip(params["input_device"].as_str());
        let configured = cfg.peer_fingerprint.as_deref().zip(cfg.input_device.as_deref());
        if !requested
            .zip(configured)
            .is_some_and(|((peer, device), (expected_peer, expected_device))| {
                peer.eq_ignore_ascii_case(expected_peer) && device == expected_device
            })
        {
            return Err(IpcError::new(
                "invalid_params",
                "The configured microphone route changed. Review it and try again.",
            ));
        }
        self.microphone.armed = cfg.peer_fingerprint.clone().zip(cfg.input_device.clone());
        Ok(json!({"armed": true}))
    }

    fn refresh_microphone_trust(&mut self) {
        if self
            .microphone
            .trust_checked
            .is_some_and(|at| at.elapsed() < Duration::from_secs(1))
        {
            return;
        }
        self.microphone.trust_checked = Some(Instant::now());
        let trusted = self
            .cfg
            .microphone
            .peer_fingerprint
            .as_deref()
            .and_then(Fingerprint::parse)
            .is_some_and(|fp| {
                TrustStore::load(self.driver.trust_store().dir()).is_ok_and(|store| match self.cfg.role {
                    Role::Server => store.is_trusted_client(&fp),
                    Role::Client => store.is_trusted_server(&fp),
                    Role::Off => false,
                })
            });
        if self.microphone.peer_trusted && !trusted {
            self.microphone.disarm();
        }
        self.microphone.peer_trusted = trusted;
    }

    fn microphone_authorized(&self, conn: ConnId) -> bool {
        self.enabled
            && !self.stop.load(Ordering::Relaxed)
            && !self.suspended()
            && self.platform != "virtual"
            && self.microphone.peer_trusted
            && self.cfg.tls.enabled
            && (self.cfg.role != Role::Server || self.cfg.tls.require_client_trust)
            && self.connections.get(&conn).is_some_and(|peer| {
                peer.tls
                    && peer.certificate_verified
                    && peer
                        .fingerprint
                        .as_ref()
                        .zip(self.cfg.microphone.peer_fingerprint.as_ref())
                        .is_some_and(|(actual, selected)| actual.eq_ignore_ascii_case(selected))
            })
    }

    pub(super) fn offer_microphone(&mut self, conn: ConnId, message: &mut Message) {
        if let Message::SetOptions { options } = message {
            self.refresh_microphone_trust();
            // Only the expressly selected bound identity sees the DSOP marker.
            // Legacy peers receive the original options unless explicitly selected.
            if self.cfg.role == Role::Server && self.microphone_authorized(conn) && self.machine.peer_ready(conn) {
                options.push((MICROPHONE_OPTION, 1));
                if let Some(peer) = self.connections.get_mut(&conn) {
                    peer.microphone.offered = true;
                }
            }
        }
    }

    pub(super) fn handle_microphone_message(&mut self, conn: ConnId, message: Message) -> Option<Message> {
        if matches!(message, Message::SetOptions { .. } | Message::MicrophoneReady { .. }) {
            self.refresh_microphone_trust();
        }
        match message {
            Message::SetOptions { ref options }
                if self.cfg.role == Role::Client
                    && options.contains(&(MICROPHONE_OPTION, 1))
                    && self.microphone_authorized(conn) =>
            {
                if let Some(peer) = self.connections.get_mut(&conn) {
                    peer.microphone.offered = true;
                    peer.microphone.accepted = true;
                }
                Some(message)
            }
            Message::MicrophoneReady { stream } => {
                if self.microphone_authorized(conn) && self.machine.peer_ready(conn) {
                    if let Some(peer) = self.connections.get_mut(&conn).filter(|p| p.microphone.offered) {
                        if peer.microphone.receive_stream != stream {
                            self.driver.cancel_audio(conn);
                        }
                        peer.microphone.accepted = true;
                        peer.microphone.receive_stream = stream;
                    }
                }
                None
            }
            Message::MicrophoneAudio { stream, pcm } => {
                if self.microphone.conn == Some(conn)
                    && stream != 0
                    && self.microphone.advertised == Some(stream)
                    && self.microphone_authorized(conn)
                    && self.machine.peer_ready(conn)
                    && self
                        .microphone
                        .audio
                        .output_matches(self.cfg.microphone.output_device.as_deref())
                {
                    self.microphone.audio.play(pcm);
                }
                None
            }
            other => Some(other),
        }
    }

    pub(super) fn update_microphone(&mut self) {
        self.refresh_microphone_trust();
        let cfg = &self.cfg.microphone;
        let eligible = |conn| {
            self.microphone_authorized(conn)
                && self.machine.peer_ready(conn)
                && self.connections.get(&conn).is_some_and(|peer| peer.microphone.accepted)
        };
        let conn = self
            .microphone
            .conn
            .filter(|&conn| eligible(conn))
            .or_else(|| self.connections.keys().copied().find(|&conn| eligible(conn)));
        let waker = self.driver.waker();
        let suspended = self.suspended();
        let sharing = &mut self.microphone;
        if suspended
            || self.cfg.role == Role::Off
            || !self.cfg.tls.enabled
            || (self.cfg.role == Role::Server && !self.cfg.tls.require_client_trust)
            || sharing.armed.as_ref().is_some_and(|(peer, device)| {
                !cfg.peer_fingerprint
                    .as_ref()
                    .is_some_and(|value| value.eq_ignore_ascii_case(peer))
                    || cfg.input_device.as_ref() != Some(device)
            })
        {
            sharing.disarm();
        }
        if sharing.conn != conn {
            if let Some(old) = sharing.conn {
                self.driver.cancel_audio(old);
                if sharing.advertised.is_some_and(|stream| stream != 0) && self.connections.contains_key(&old) {
                    let _ = self.driver.execute_checked(
                        NetCommand::Send {
                            conn: old,
                            msg: Message::MicrophoneReady { stream: 0 },
                            priority: Priority::Control,
                        },
                        &mut self.net_events,
                    );
                }
            }
            sharing.audio.configure(None, None, &waker);
            sharing.advertised = None;
            sharing.conn = conn;
        }

        let remote_stream = conn
            .and_then(|id| self.connections.get(&id))
            .map_or(0, |p| p.microphone.receive_stream);
        let input = cfg
            .input_device
            .as_deref()
            .filter(|_| sharing.armed.is_some() && remote_stream != 0);
        let output = cfg.output_device.as_deref().filter(|_| conn.is_some());
        let (input_changed, output_changed) = sharing.audio.configure(input, output, &waker);
        let (input_state, output_state) = sharing.audio.stream_states();
        if let Some(conn) = conn {
            if input_changed || !matches!(input_state, Some(StreamState::Ready)) {
                self.driver.cancel_audio(conn);
            }
            let ready = matches!(output_state, Some(StreamState::Ready));
            let advertised = sharing.advertised.unwrap_or(0);
            if sharing.advertised.is_none() || ready != (advertised != 0) || (ready && output_changed) {
                let stream = if ready {
                    sharing.last_stream = sharing.last_stream.wrapping_add(1).max(1);
                    sharing.last_stream
                } else {
                    0
                };
                if self
                    .driver
                    .execute_checked(
                        NetCommand::Send {
                            conn,
                            msg: Message::MicrophoneReady { stream },
                            priority: Priority::Control,
                        },
                        &mut self.net_events,
                    )
                    .is_ok()
                {
                    sharing.advertised = Some(stream);
                }
            }
            if remote_stream != 0 && matches!(input_state, Some(StreamState::Ready)) {
                for _ in 0..CAPTURE_QUEUE {
                    let Some(pcm) = sharing.audio.take_capture() else { break };
                    let _ = self.driver.execute_checked(
                        NetCommand::Send {
                            conn,
                            msg: Message::MicrophoneAudio {
                                stream: remote_stream,
                                pcm,
                            },
                            priority: Priority::Audio,
                        },
                        &mut self.net_events,
                    );
                }
            }
        }

        let (input_state, input_error) = if cfg.input_device.is_some() && sharing.armed.is_none() {
            ("needs_consent".into(), None)
        } else {
            describe(
                input_state,
                cfg.input_device.is_some(),
                conn.is_some(),
                "sharing",
                "waiting_for_receiver",
            )
        };
        let (output_state, output_error) = describe(
            output_state,
            cfg.output_device.is_some(),
            conn.is_some(),
            "ready",
            "starting",
        );
        let status = MicrophoneInfo {
            peer_connected: conn.is_some(),
            input_state,
            output_state,
            input_error,
            output_error,
        };
        if status != sharing.status {
            sharing.status = status;
            self.status_dirty = true;
        }
    }

    pub(super) fn microphone_status(&self) -> MicrophoneInfo {
        self.microphone.status.clone()
    }
}

fn describe(
    state: Option<StreamState>,
    configured: bool,
    connected: bool,
    ready: &str,
    idle: &str,
) -> (String, Option<String>) {
    let state = match state {
        Some(StreamState::Starting) => "starting",
        Some(StreamState::Ready) => ready,
        Some(StreamState::Failed(error)) => return ("error".into(), Some(error)),
        None if !configured => "disabled",
        None if !connected => "waiting_for_peer",
        None => idle,
    };
    (state.into(), None)
}
