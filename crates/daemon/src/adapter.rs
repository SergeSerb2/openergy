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
pub fn platform(name: &str, cfg: &Config) -> Result<(Backend, Option<VirtualControl>), PlatformError> {
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
        "macos" => openergy_platform_macos::create(openergy_platform_macos::MacOptions {
            click_numbering: cfg.macos.click_numbering,
            lines_per_notch: 3.0,
        })
        .map(|b| (b, None)),
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
    _control: &VirtualControl,
    _method: &str,
    _params: &serde_json::Value,
) -> Result<serde_json::Value, openergy_ipc::IpcError> {
    Err(openergy_ipc::IpcError::new(
        "virtual_api_pending",
        "VirtualControl methods have not landed yet",
    ))
}
