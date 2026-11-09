//! Read-only discovery in a run-owned session. See tools/linux-dev/wayland-discover.py.
#[cfg(not(target_os = "linux"))]
fn main() {
    use std::io::Write;
    let _ = writeln!(std::io::stderr(), "wayland_probe requires Linux");
    std::process::exit(2);
}

#[cfg(target_os = "linux")]
fn main() {
    use std::io::Write;
    let (report, success) = probe::run();
    if writeln!(std::io::stdout().lock(), "{report}").is_err() || !success {
        std::process::exit(1);
    }
}

#[cfg(target_os = "linux")]
mod probe {
    use serde_json::{json, Value};
    use std::{
        collections::BTreeMap,
        env, fs,
        io::{Read, Write},
        os::unix::fs::{FileTypeExt, MetadataExt},
        path::{Path, PathBuf},
        process::{Command, Stdio},
        sync::mpsc,
        time::Duration,
    };
    use wayland_client::{
        delegate_noop,
        protocol::{wl_keyboard, wl_output, wl_registry, wl_seat},
        Connection, Dispatch, QueueHandle, WEnum,
    };
    use wayland_protocols::xdg::xdg_output::zv1::client::{zxdg_output_manager_v1, zxdg_output_v1};

    type Result<T> = std::result::Result<T, &'static str>;
    const PORTAL: &str = "org.freedesktop.portal.Desktop";
    const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
    const RD: &str = "org.freedesktop.portal.RemoteDesktop";
    const IC: &str = "org.freedesktop.portal.InputCapture";

    pub fn run() -> (Value, bool) {
        let args: Vec<_> = env::args_os().skip(1).collect();
        let result = if args.len() != 1 || args.first().is_none_or(|arg| arg != "discover") {
            Err("usage: wayland_probe discover (isolated, read-only discovery only)")
        } else if env::var_os("WAYLAND_DISPLAY").is_none_or(|s| s.is_empty()) {
            Err("no Wayland session: WAYLAND_DISPLAY is not set")
        } else {
            isolation().and_then(|()| discover())
        };
        match result {
            Ok(report) => (report, true),
            Err(error) => {
                let _ = writeln!(std::io::stderr(), "wayland_probe: {error}");
                (
                    json!({"schema_version":1,"mode":"discover","status":"error","errors":[error],
                    "capability_matrix":{
                        "client":{"possible":"unknown","ready":false,"reasons":[error]},
                        "server":{"possible":"unknown","ready":false,"reasons":[error]}}}),
                    false,
                )
            }
        }
    }

    // This is a lab guard, not a security boundary against a caller forging the environment.
    // Validate exact private paths before opening any socket. Never fall back to host defaults.
    fn isolation() -> Result<()> {
        let root = PathBuf::from(env::var_os("OPENERGY_WAYLAND_PROBE_ROOT").ok_or("isolation_guard_missing")?);
        if env::var("OPENERGY_WAYLAND_PROBE_ISOLATED").as_deref() != Ok("1")
            || !root.is_absolute()
            || fs::canonicalize(&root).ok().as_ref() != Some(&root)
            || env::var_os("WAYLAND_SOCKET").is_some()
            || env::var_os("DISPLAY").is_some()
        {
            return Err("isolation_guard_invalid");
        }
        for (key, suffix) in [
            ("OPENERGY_WAYLAND_PROBE_ROOT", ""),
            ("XDG_RUNTIME_DIR", "runtime"),
            ("HOME", "home"),
            ("XDG_CONFIG_HOME", "config"),
            ("XDG_DATA_HOME", "data"),
            ("XDG_STATE_HOME", "state"),
            ("XDG_CACHE_HOME", "cache"),
        ] {
            let expected = root.join(suffix);
            let actual = PathBuf::from(env::var_os(key).ok_or("isolation_private_directory_missing")?);
            let meta = fs::symlink_metadata(&actual).map_err(|_| "isolation_private_directory_missing")?;
            // getuid has no preconditions and does not change process credentials.
            if actual != expected
                || !meta.is_dir()
                || meta.mode() & 0o777 != 0o700
                || meta.uid() != unsafe { libc::getuid() }
            {
                return Err("isolation_private_directory_invalid");
            }
        }
        if env::var("WAYLAND_DISPLAY").as_deref() != Ok("wayland-probe-0")
            || env::var("DBUS_SESSION_BUS_ADDRESS").ok() != Some(format!("unix:path={}/runtime/bus", root.display()))
        {
            return Err("isolation_private_socket_or_bus_identity_missing");
        }
        let bus = root.join("runtime/bus");
        if !owned_socket(&bus) {
            return Err("isolation_private_bus_socket_missing");
        }
        // A removed display is a connection failure, not a reason to try another display.
        if !owned_socket(&root.join("runtime/wayland-probe-0")) {
            return Err("wayland_connection_failed: private compositor socket missing");
        }
        Ok(())
    }

    fn owned_socket(path: &Path) -> bool {
        fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_socket() && m.uid() == unsafe { libc::getuid() })
    }

    fn discover() -> Result<Value> {
        // A compositor can accept a connection and then stop responding. Keep the public
        // command bounded even during connect/roundtrip. No subprocess runs on this thread.
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("wayland-discovery".into())
            .spawn(move || {
                let _ = tx.send(wayland());
            })
            .map_err(|_| "wayland_worker_start_failed")?;
        let wayland = rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "wayland_discovery_timeout_or_worker_lost")??;
        let portals = portals();
        let matrix = matrix(&wayland, &portals);
        Ok(
            json!({"schema_version":1,"mode":"discover","status":"ok","authorization":"not_requested",
            "compositor":{"identity":"not_standardized_by_wayland",
                "desktop_environment_hint":env::var("XDG_CURRENT_DESKTOP").ok(),
                "session_desktop_hint":env::var("XDG_SESSION_DESKTOP").ok(),
                "session_type_hint":env::var("XDG_SESSION_TYPE").ok(),
                "identity_basis":"environment hints and vendor globals only; never used for capability gating"},
            "wayland":wayland,"portals":portals,
            "ei":ei_facts(),
            "capability_matrix":matrix}),
        )
    }

    fn ei_facts() -> Value {
        let socket = env::var_os("LIBEI_SOCKET");
        let fact = if let Some(socket) = &socket {
            let runtime = PathBuf::from(env::var_os("XDG_RUNTIME_DIR").unwrap_or_default());
            let path = runtime.join(socket);
            if path.parent().and_then(|p| fs::canonicalize(p).ok()).as_ref() != Some(&runtime) {
                "outside_private_runtime_not_inspected"
            } else if owned_socket(&path) {
                "private_socket_present_not_connected"
            } else {
                "private_socket_missing_or_not_owned"
            }
        } else {
            "not_configured"
        };
        json!({"socket_environment_present":socket.is_some(),"socket_fact":fact,
            "socket_connected":false,"protocol_version":null,"devices":null,"absolute_regions":null,
            "reason":"portal EIS FD requires authorization; no direct socket or negotiated EI capabilities probed"})
    }

    #[derive(Default)]
    struct State {
        globals: BTreeMap<u32, (String, u32)>,
        outputs: BTreeMap<u32, Value>,
        seats: BTreeMap<u32, Value>,
    }

    fn wayland() -> Result<Value> {
        let connection = Connection::connect_to_env().map_err(|_| "wayland_connection_failed")?;
        let mut queue = connection.new_event_queue();
        let qh = queue.handle();
        let registry = connection.display().get_registry(&qh, ());
        let mut state = State::default();
        queue.roundtrip(&mut state).map_err(|_| "wayland_registry_failed")?;
        let manager = state
            .globals
            .iter()
            .find(|(_, (name, _))| name == "zxdg_output_manager_v1")
            .map(|(&id, (_, v))| {
                registry.bind::<zxdg_output_manager_v1::ZxdgOutputManagerV1, _, _>(id, (*v).min(3), &qh, ())
            });
        let mut objects = Vec::new();
        let mut xdg_outputs = Vec::new();
        let mut seats = Vec::new();
        for (&id, (name, version)) in &state.globals {
            match name.as_str() {
                "wl_output" => {
                    let output = registry.bind::<wl_output::WlOutput, _, _>(id, (*version).min(4), &qh, id);
                    state.outputs.insert(
                        id,
                        json!({"global":id,"logical_position":null,"logical_size":null,
                        "logical_source":if manager.is_some(){"xdg_output"}else{"unavailable"},
                        "fractional_scale":null,"integer_scale":null,"name":null,"current_mode":null}),
                    );
                    if let Some(manager) = &manager {
                        xdg_outputs.push(manager.get_xdg_output(&output, &qh, id));
                    }
                    objects.push(output);
                }
                "wl_seat" => {
                    state.seats.insert(
                        id,
                        json!({"global":id,"name":null,"capabilities":null,
                        "keymap":null,"layout":"unknown; keymap contents are not read"}),
                    );
                    seats.push(registry.bind::<wl_seat::WlSeat, _, _>(id, (*version).min(9), &qh, id));
                }
                _ => {}
            }
        }
        queue
            .roundtrip(&mut state)
            .map_err(|_| "wayland_output_or_seat_failed")?;
        // get_keyboard requests made while handling seat capabilities need one more sync.
        queue.roundtrip(&mut state).map_err(|_| "wayland_keymap_failed")?;
        let globals: Vec<_> = state
            .globals
            .iter()
            .map(|(id, (interface, version))| json!({"global":id,"interface":interface,"version":version}))
            .collect();
        let mut relevant = serde_json::Map::new();
        for interface in [
            "zwp_virtual_keyboard_manager_v1",
            "zwlr_virtual_pointer_manager_v1",
            "zwlr_screencopy_manager_v1",
            "ext_image_copy_capture_manager_v1",
            "ext_output_image_capture_source_manager_v1",
            "wl_output",
            "zxdg_output_manager_v1",
            "wl_seat",
            "zwp_keyboard_shortcuts_inhibit_manager_v1",
            "zwp_relative_pointer_manager_v1",
            "zwp_pointer_constraints_v1",
            "wp_pointer_warp_v1",
            "ext_data_control_manager_v1",
            "zwlr_data_control_manager_v1",
        ] {
            let versions: Vec<_> = state
                .globals
                .values()
                .filter(|(name, _)| name == interface)
                .map(|(_, v)| *v)
                .collect();
            relevant.insert(
                interface.into(),
                json!({"advertised":!versions.is_empty(),"versions":versions}),
            );
        }
        Ok(json!({"globals":globals,"relevant_globals":relevant,
            "outputs":state.outputs.values().collect::<Vec<_>>(),"seats":state.seats.values().collect::<Vec<_>>(),
            "geometry_note":"xdg-output is supplemental logical geometry, not EI regions or IC zones; no pixel/scale inference",
            "keyboard_note":"seat/keymap metadata only; no key events or keymap contents retained"}))
    }

    impl Dispatch<wl_registry::WlRegistry, ()> for State {
        fn event(
            state: &mut Self,
            _: &wl_registry::WlRegistry,
            event: wl_registry::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            match event {
                wl_registry::Event::Global {
                    name,
                    interface,
                    version,
                } => {
                    state.globals.insert(name, (interface, version));
                }
                wl_registry::Event::GlobalRemove { name } => {
                    state.globals.remove(&name);
                    state.outputs.remove(&name);
                    state.seats.remove(&name);
                }
                _ => {}
            }
        }
    }

    impl Dispatch<wl_output::WlOutput, u32> for State {
        fn event(
            state: &mut Self,
            _: &wl_output::WlOutput,
            event: wl_output::Event,
            id: &u32,
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            let Some(output) = state.outputs.get_mut(id) else {
                return;
            };
            match event {
                wl_output::Event::Geometry {
                    x,
                    y,
                    physical_width,
                    physical_height,
                    make,
                    model,
                    transform,
                    ..
                } => {
                    output["core_geometry"] = json!({"x":x,"y":y,"physical_width_mm":physical_width,
                        "physical_height_mm":physical_height,"make":make,"model":model,"transform":format!("{transform:?}")});
                }
                wl_output::Event::Mode {
                    flags: WEnum::Value(flags),
                    width,
                    height,
                    refresh,
                } if flags.contains(wl_output::Mode::Current) => {
                    output["current_mode"] = json!({"width_px":width,"height_px":height,"refresh_millihz":refresh});
                }
                wl_output::Event::Scale { factor } => {
                    output["integer_scale"] = json!(factor);
                }
                wl_output::Event::Name { name } => {
                    output["name"] = json!(name);
                }
                wl_output::Event::Description { description } => {
                    output["description"] = json!(description);
                }
                _ => {}
            }
        }
    }

    impl Dispatch<zxdg_output_v1::ZxdgOutputV1, u32> for State {
        fn event(
            state: &mut Self,
            _: &zxdg_output_v1::ZxdgOutputV1,
            event: zxdg_output_v1::Event,
            id: &u32,
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            let Some(output) = state.outputs.get_mut(id) else {
                return;
            };
            match event {
                zxdg_output_v1::Event::LogicalPosition { x, y } => {
                    output["logical_position"] = json!({"x":x,"y":y});
                }
                zxdg_output_v1::Event::LogicalSize { width, height } => {
                    output["logical_size"] = json!({"width":width,"height":height});
                }
                zxdg_output_v1::Event::Name { name } => {
                    output["name"] = json!(name);
                }
                zxdg_output_v1::Event::Description { description } => {
                    output["description"] = json!(description);
                }
                _ => {}
            }
        }
    }
    delegate_noop!(State: ignore zxdg_output_manager_v1::ZxdgOutputManagerV1);

    impl Dispatch<wl_seat::WlSeat, u32> for State {
        fn event(
            state: &mut Self,
            proxy: &wl_seat::WlSeat,
            event: wl_seat::Event,
            id: &u32,
            _: &Connection,
            qh: &QueueHandle<Self>,
        ) {
            let Some(seat) = state.seats.get_mut(id) else { return };
            match event {
                wl_seat::Event::Name { name } => {
                    seat["name"] = json!(name);
                }
                wl_seat::Event::Capabilities {
                    capabilities: WEnum::Value(caps),
                } => {
                    if seat["capabilities"].is_null() && caps.contains(wl_seat::Capability::Keyboard) {
                        let _keyboard = proxy.get_keyboard(qh, *id);
                    }
                    seat["capabilities"] = json!({"keyboard":caps.contains(wl_seat::Capability::Keyboard),
                        "pointer":caps.contains(wl_seat::Capability::Pointer),"touch":caps.contains(wl_seat::Capability::Touch)});
                }
                _ => {}
            }
        }
    }

    impl Dispatch<wl_keyboard::WlKeyboard, u32> for State {
        fn event(
            state: &mut Self,
            _: &wl_keyboard::WlKeyboard,
            event: wl_keyboard::Event,
            id: &u32,
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            let Some(seat) = state.seats.get_mut(id) else { return };
            match event {
                // The OwnedFd is closed without mapping/reading the keymap. No focused surface exists.
                wl_keyboard::Event::Keymap { format, size, .. } => {
                    seat["keymap"] = json!({"format":format!("{format:?}"),"size_bytes":size});
                }
                wl_keyboard::Event::RepeatInfo { rate, delay } => {
                    seat["repeat_info"] = json!({"rate":rate,"delay_ms":delay});
                }
                _ => {}
            }
        }
    }

    // busctl is deliberately an external, read-only introspection tool, not a portal wrapper
    // that might create a session. GNU timeout bounds even the bus connection itself.
    fn bus(args: &[&str]) -> Result<Value> {
        let address = env::var("DBUS_SESSION_BUS_ADDRESS").map_err(|_| "session_bus_missing")?;
        let mut child = Command::new("timeout")
            .args([
                "--signal=KILL",
                "3s",
                "busctl",
                "--address",
                &address,
                "--auto-start=no",
                "--allow-interactive-authorization=no",
                "--timeout=2",
                "--json=short",
            ])
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "busctl_or_timeout_unavailable")?;
        let mut bytes = Vec::new();
        let read = child
            .stdout
            .take()
            .ok_or("busctl_stdout_unavailable")?
            .take(1_048_577)
            .read_to_end(&mut bytes);
        if read.is_err() || bytes.len() > 1_048_576 {
            let _ = child.kill();
            let _ = child.wait();
            return Err("busctl_reply_unreadable_or_too_large");
        }
        if !child.wait().map_err(|_| "busctl_wait_failed")?.success() {
            return Err("dbus_call_failed_or_timed_out");
        }
        serde_json::from_slice(&bytes).map_err(|_| "busctl_invalid_json")
    }

    fn first_data(value: &Value) -> Option<&Value> {
        value.get("data")?.as_array()?.first()
    }

    fn portals() -> Value {
        match portal_interfaces() {
            Ok(interfaces) => json!({"status":"available","auto_start":false,"interfaces":interfaces}),
            Err(reason) => json!({"status":reason,"auto_start":false,"interfaces":{}}),
        }
    }

    fn portal_interfaces() -> Result<Value> {
        let owner = bus(&[
            "call",
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "NameHasOwner",
            "s",
            PORTAL,
        ])?;
        match first_data(&owner).and_then(Value::as_bool) {
            Some(false) => return Err("portal_service_missing"),
            Some(true) => {}
            None => return Err("portal_owner_reply_invalid"),
        }
        let reply = bus(&[
            "call",
            PORTAL,
            PORTAL_PATH,
            "org.freedesktop.DBus.Introspectable",
            "Introspect",
        ])?;
        let xml = first_data(&reply)
            .and_then(Value::as_str)
            .ok_or("portal_introspection_reply_invalid")?;
        // D-Bus introspection normally includes a public DOCTYPE. roxmltree never
        // fetches external DTDs; allow that declaration without network resolution.
        let doc = roxmltree::Document::parse_with_options(
            xml,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                nodes_limit: 100_000,
                ..Default::default()
            },
        )
        .map_err(|_| "portal_introspection_xml_invalid")?;
        let mut interfaces = serde_json::Map::new();
        for name in [
            RD,
            IC,
            "org.freedesktop.portal.Clipboard",
            "org.freedesktop.portal.ScreenCast",
        ] {
            let node = doc
                .descendants()
                .find(|n| n.has_tag_name("interface") && n.attribute("name") == Some(name));
            let Some(node) = node else {
                interfaces.insert(name.into(), json!({"advertised":false}));
                continue;
            };
            let methods: Vec<_> = node
                .children()
                .filter(|n| n.has_tag_name("method"))
                .filter_map(|n| n.attribute("name"))
                .collect();
            let mut properties = serde_json::Map::new();
            for prop in ["version", "AvailableDeviceTypes", "SupportedCapabilities"] {
                if node
                    .children()
                    .any(|n| n.has_tag_name("property") && n.attribute("name") == Some(prop))
                {
                    let value = match bus(&["get-property", PORTAL, PORTAL_PATH, name, prop]) {
                        Ok(v) => json!({"status":"read","value":v.get("data")}),
                        Err(reason) => json!({"status":reason,"value":null}),
                    };
                    properties.insert(prop.into(), value);
                }
            }
            interfaces.insert(
                name.into(),
                json!({"advertised":true,"methods":methods,"properties":properties,"authorized":false}),
            );
        }
        Ok(Value::Object(interfaces))
    }

    fn matrix(wayland: &Value, portals: &Value) -> Value {
        let mut roles = serde_json::Map::new();
        for (role, interface, property, required_methods) in [
            (
                "client",
                RD,
                "AvailableDeviceTypes",
                &["CreateSession", "SelectDevices", "Start", "ConnectToEIS"][..],
            ),
            (
                "server",
                IC,
                "SupportedCapabilities",
                &["ConnectToEIS", "GetZones", "SetPointerBarriers", "Enable", "Release"][..],
            ),
        ] {
            let facts = &portals["interfaces"][interface];
            let mut gaps = Vec::new();
            let mut unknown = Vec::new();
            if portals["status"] != "available" {
                let status = portals["status"].as_str().unwrap_or("portal_discovery_failed");
                if status == "portal_service_missing" {
                    gaps.push("wayland_portal_missing".to_string());
                } else {
                    unknown.push(status.to_string());
                }
            } else if facts["advertised"] != true {
                gaps.push(format!("{interface}_missing"));
            } else {
                for method in required_methods {
                    if !has_method(facts, method) {
                        gaps.push(format!("{method}_missing"));
                    }
                }
                let version = facts["properties"]["version"]["value"].as_u64();
                if version.is_none() {
                    unknown.push("portal_version_unknown".into());
                } else if version == Some(0) || (role == "client" && version.is_some_and(|v| v < 2)) {
                    gaps.push("portal_version_too_old_for_selected_path".into());
                }
                if role == "server" {
                    if version.is_some_and(|v| v >= 2) {
                        for method in ["CreateSession2", "Start"] {
                            if !has_method(facts, method) {
                                gaps.push(format!("{method}_missing"));
                            }
                        }
                    } else if !has_method(facts, "CreateSession") {
                        gaps.push("CreateSession_missing".into());
                    }
                }
                match facts["properties"][property]["value"].as_u64() {
                    Some(bits) if bits & 3 != 3 => gaps.push("keyboard_and_pointer_not_both_advertised".into()),
                    None => unknown.push("keyboard_and_pointer_capabilities_unknown".into()),
                    _ => {}
                }
            }
            if wayland["outputs"].as_array().is_none_or(Vec::is_empty) {
                gaps.push("no_outputs".into());
            }
            let possible = if !gaps.is_empty() {
                "unsupported"
            } else if !unknown.is_empty() {
                "unknown"
            } else {
                "candidate_after_grant"
            };
            unknown.extend(
                if role == "client" {
                    vec![
                        "authorization_not_requested",
                        "EI_absolute_regions_and_pointer_keyboard_devices_unverified",
                        "repeat_locks_text_unverified",
                    ]
                } else {
                    vec![
                        "authorization_not_requested",
                        "EI_receiver_and_barriers_unverified",
                        "capture_contract_and_held_transition_acceptance_required",
                        "no_passive_global_input_or_inactive_local_hotkeys",
                        "no_immediate_grab_or_PassOnce",
                        "no_global_cursor_query_hide_or_guaranteed_warp",
                    ]
                }
                .into_iter()
                .map(str::to_string),
            );
            gaps.extend(unknown);
            roles.insert(role.into(), json!({"possible":possible,"ready":false,"reasons":gaps}));
        }
        let direct = ["zwp_virtual_keyboard_manager_v1", "zwlr_virtual_pointer_manager_v1"]
            .iter()
            .all(|name| wayland["relevant_globals"][name]["advertised"] == true);
        roles.insert("optional_direct_client".into(), json!({"possible":if direct {"candidate_later_opt_in"} else {"unsupported"},
            "ready":false,"reasons":["requires_virtual_keyboard_and_pointer_explicit_opt_in_and_native_acceptance",
                "absolute_mapping_and_policy_unverified; clipboard_requires_separate_data_control", "virtual_input_and_screencopy_do_not_supply_server_capture"]}));
        Value::Object(roles)
    }

    fn has_method(facts: &Value, method: &str) -> bool {
        facts["methods"]
            .as_array()
            .is_some_and(|methods| methods.iter().any(|m| m == method))
    }
}
