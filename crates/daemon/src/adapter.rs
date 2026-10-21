//! Keep owner-crate construction APIs at one integration boundary.
use openergy_config::{Config, Role};
use openergy_net::{MdnsConfig, NetConfig};
use openergy_platform::virtual_backend::{self, VirtualControl, VirtualOptions};
use openergy_platform::{Backend, PlatformError};
use std::{path::Path, time::Duration};

pub fn net_config(config: &Config, dir: &Path, virtual_platform: bool) -> NetConfig {
    NetConfig {
        tls_enabled: config.tls.enabled,
        tls_dir: dir.join("tls"),
        identity_name: config.name.clone(),
        require_client_trust: config.tls.require_client_trust,
        held_timeout: Duration::from_secs(30),
        max_frame_len: openergy_protocol::MAX_FRAME_LEN,
        mdns: (!virtual_platform && std::env::var("OPENERGY_DISABLE_DISCOVERY").as_deref() != Ok("1")).then(|| {
            MdnsConfig {
                advertise: (config.role == Role::Server).then(|| (config.name.clone(), config.server.port)),
                browse: true,
                loopback_only: false,
            }
        }),
    }
}
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub enum Notice {
    Permission,
    CaptureError,
    CaptureStopped,
    Displays,
    Sleep,
    Wake,
    Lock,
    Unlock,
    NotificationError,
}
pub type Notices = Box<dyn FnMut() -> Vec<Notice> + Send>;
pub struct Platform {
    pub backend: Backend,
    pub control: Option<VirtualControl>,
    pub notices: Notices,
}
fn parts(backend: Backend, control: Option<VirtualControl>) -> Platform {
    Platform {
        backend,
        control,
        notices: Box::new(Vec::new),
    }
}
pub fn platform(name: &str, _cfg: &Config) -> Result<Platform, PlatformError> {
    match name {
        "virtual" => {
            let (backend, control) = virtual_backend::create(VirtualOptions {
                screen_w: 1920,
                screen_h: 1080,
                hid_seed: 100,
                double_click_ms: 500,
            });
            Ok(parts(backend, Some(control)))
        }
        #[cfg(target_os = "macos")]
        "macos" => {
            use openergy_platform_macos::{ClickNumbering, MacNotice, MacOptions};
            let click_numbering = match _cfg.macos.click_numbering {
                openergy_config::ClickNumbering::Native => ClickNumbering::Native,
                openergy_config::ClickNumbering::Legacy => ClickNumbering::Legacy,
            };
            let (backend, rx) = openergy_platform_macos::create_with_notices(MacOptions {
                click_numbering,
                lines_per_notch: 3.0,
            })?;
            let notices: Notices = Box::new(move || {
                rx.try_iter()
                    .map(|n| match n {
                        MacNotice::CapturePermissionProblem => Notice::Permission,
                        MacNotice::CaptureTranslationProblem => Notice::CaptureError,
                        MacNotice::CaptureStopped => Notice::CaptureStopped,
                        MacNotice::DisplaysChanged => Notice::Displays,
                        MacNotice::Sleeping => Notice::Sleep,
                        MacNotice::Woke => Notice::Wake,
                        MacNotice::ScreenLocked => Notice::Lock,
                        MacNotice::ScreenUnlocked => Notice::Unlock,
                        MacNotice::NotificationSetupFailed => Notice::NotificationError,
                    })
                    .collect()
            });
            Ok(Platform {
                backend,
                control: None,
                notices,
            })
        }
        #[cfg(target_os = "linux")]
        "x11" => openergy_platform_linux::create(openergy_platform_linux::X11Options {
            display: std::env::var("DISPLAY").ok(),
        })
        .map(|b| parts(b, None)),
        #[cfg(target_os = "windows")]
        "windows" => {
            openergy_platform_windows::create(openergy_platform_windows::WindowsOptions {}).map(|b| parts(b, None))
        }
        _ => Err(PlatformError::Failed(format!(
            "platform {name} is unavailable on this OS"
        ))),
    }
}

pub fn virtual_request(
    control: &VirtualControl,
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, openergy_ipc::IpcError> {
    use openergy_ipc::{methods as m, IpcError};
    use serde_json::{json, Value};
    let invalid = |e: String| IpcError::new("invalid_params", e);
    let failed = |e: PlatformError| IpcError::new("virtual_failed", e.to_string());
    match method {
        m::VIRTUAL_INJECT => {
            let object = params
                .as_object()
                .ok_or_else(|| invalid("expected virtual injection object".into()))?;
            if object.keys().any(|k| {
                ![
                    "events",
                    "seed",
                    "hid_seed",
                    "now_ms",
                    "click_interval_ms",
                    "click_distance",
                ]
                .contains(&k.as_str())
            }) {
                return Err(invalid("unknown virtual injection parameter".into()));
            }
            let request: openergy_platform::InjectRequest =
                serde_json::from_value(json!({"events":params.get("events").cloned().unwrap_or(json!([]))}))
                    .map_err(|e| invalid(e.to_string()))?;
            let seed = params
                .get("seed")
                .or_else(|| params.get("hid_seed"))
                .map(|v| v.as_i64().ok_or_else(|| invalid("seed must fit i64".into())))
                .transpose()?;
            let now = params
                .get("now_ms")
                .map(|v| v.as_u64().ok_or_else(|| invalid("now_ms must fit u64".into())))
                .transpose()?;
            let interval = params
                .get("click_interval_ms")
                .map(|v| {
                    v.as_u64()
                        .ok_or_else(|| invalid("click_interval_ms must fit u64".into()))
                })
                .transpose()?;
            let distance = params
                .get("click_distance")
                .map(|v| {
                    v.as_f64()
                        .ok_or_else(|| invalid("click_distance must be a finite number".into()))
                })
                .transpose()?;
            control
                .set_numbering_controls(now, interval, distance)
                .map_err(failed)?;
            if let Some(seed) = seed {
                control.set_hid_seed(seed);
            }
            let events = request.into_events();
            if events.is_empty() {
                return Ok(json!({"delivered":0,"passed":0,"swallowed":0}));
            }
            serde_json::to_value(control.inject(events).map_err(failed)?).map_err(|e| invalid(e.to_string()))
        }
        m::VIRTUAL_RECORDED => {
            let mut fields = params.clone();
            let audit = fields
                .as_object_mut()
                .and_then(|fields| fields.remove("audit"))
                .map(|audit| audit.as_bool().ok_or_else(|| invalid("audit must be a boolean".into())))
                .transpose()?
                .unwrap_or(false);
            let request: virtual_backend::RecordedRequest =
                serde_json::from_value(fields).map_err(|e| invalid(e.to_string()))?;
            let mut ledger = control.ledger(request.since);
            if !audit {
                ledger.events.retain(|record| {
                    matches!(
                        record.detail,
                        virtual_backend::RecordKind::Sink | virtual_backend::RecordKind::ReleaseAll
                    )
                });
            }
            serde_json::to_value(ledger).map_err(|e| invalid(e.to_string()))
        }
        m::VIRTUAL_CLIPBOARD => {
            if let Some(data) = params.get("set") {
                let data = crate::clipboard_json::decode(data)?;
                control.clipboard_set(data).map_err(failed)?;
            } else if let Some(text) = params.get("text").and_then(Value::as_str) {
                control
                    .clipboard_set(openergy_platform::ClipboardData {
                        formats: vec![openergy_platform::ClipboardFormat {
                            kind: openergy_platform::FormatKind::TextUtf8,
                            bytes: text.as_bytes().to_vec(),
                        }],
                    })
                    .map_err(failed)?;
            } else if let Some(size) = params.get("dib_bytes").and_then(Value::as_u64) {
                if size > 64 * 1024 * 1024 {
                    return Err(invalid("dib_bytes exceeds 64 MiB".into()));
                }
                control
                    .clipboard_set(openergy_platform::ClipboardData {
                        formats: vec![openergy_platform::ClipboardFormat {
                            kind: openergy_platform::FormatKind::Dib,
                            bytes: vec![0; size as usize],
                        }],
                    })
                    .map_err(failed)?;
            }
            let wrote =
                params.get("set").is_some() || params.get("text").is_some() || params.get("dib_bytes").is_some();
            if params.get("summary").and_then(Value::as_bool) == Some(true)
                || (wrote && params.get("get").and_then(Value::as_bool) != Some(true))
            {
                serde_json::to_value(control.clipboard_summary()).map_err(|e| invalid(e.to_string()))
            } else {
                Ok(crate::clipboard_json::encode(control.clipboard_get()))
            }
        }
        m::VIRTUAL_SET_SCREENS => {
            let request: virtual_backend::ScreensRequest =
                serde_json::from_value(params.clone()).map_err(|e| invalid(e.to_string()))?;
            control.set_screens(request.rects).map_err(failed)?;
            Ok(json!({"ok":true}))
        }
        m::VIRTUAL_SET_PERMISSIONS => {
            let permissions = serde_json::from_value(params.clone()).map_err(|e| invalid(e.to_string()))?;
            control.set_permissions(permissions);
            Ok(json!({"ok":true}))
        }
        _ => Err(IpcError::new("unknown_method", format!("unknown method {method}"))),
    }
}
