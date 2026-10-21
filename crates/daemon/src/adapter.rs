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
        mdns: (!virtual_platform).then(|| MdnsConfig {
            advertise: (config.role == Role::Server).then(|| (config.name.clone(), config.server.port)),
            browse: true,
            loopback_only: false,
        }),
    }
}
pub fn platform(name: &str, _cfg: &Config) -> Result<(Backend, Option<VirtualControl>), PlatformError> {
    match name {
        "virtual" => {
            let (b, c) = virtual_backend::create(VirtualOptions {
                screen_w: 1920,
                screen_h: 1080,
                hid_seed: 100,
                double_click_ms: 500,
            });
            Ok((b, Some(c)))
        }
        #[cfg(target_os = "macos")]
        "macos" => Err(PlatformError::NotImplemented),
        #[cfg(target_os = "linux")]
        "x11" => openergy_platform_linux::create(openergy_platform_linux::X11Options {
            display: std::env::var("DISPLAY").ok(),
        })
        .map(|b| (b, None)),
        #[cfg(target_os = "windows")]
        "windows" => openergy_platform_windows::create(openergy_platform_windows::WindowsOptions {}).map(|b| (b, None)),
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
            let request: openergy_platform::InjectRequest =
                serde_json::from_value(params.clone()).map_err(|e| invalid(e.to_string()))?;
            serde_json::to_value(control.inject(request.into_events()).map_err(failed)?)
                .map_err(|e| invalid(e.to_string()))
        }
        m::VIRTUAL_RECORDED => {
            let request: virtual_backend::RecordedRequest =
                serde_json::from_value(params.clone()).map_err(|e| invalid(e.to_string()))?;
            serde_json::to_value(control.ledger(request.since)).map_err(|e| invalid(e.to_string()))
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
            if params.get("summary").and_then(Value::as_bool) == Some(true) {
                let snapshot = control.clipboard_get();
                Ok(
                    json!({"stamp":snapshot.stamp,"own_write":snapshot.own_write,"last_own_write_stamp":snapshot.last_own_write_stamp,
                    "bytes":snapshot.data.total_bytes(),"formats":snapshot.data.formats.iter().map(|f|f.kind.as_str()).collect::<Vec<_>>()}),
                )
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
