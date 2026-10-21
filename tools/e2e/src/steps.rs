use crate::model::{Failure, Result, Step};
use crate::process::command_output;
use crate::runner::{ipc_response, isolated_env, Context};
use crate::util::{digest, fields_match, matches, summary, write_private};
use crate::wire;
use serde_json::{json, Value};
use std::process::Command;
use std::time::{Duration, Instant};

impl Context {
    pub fn execute(&mut self, step: &Step) -> Result<Value> {
        let id = step.string("node", "");
        let timeout = self.timeout(step);
        match step.op.as_str() {
            "wait_status" | "assert_status" => {
                let expected = step.fields.get("fields").cloned().unwrap_or_else(|| json!({}));
                self.poll_value(timeout, || {
                    let status = self.ipc(&id, "status.get", json!({}), Duration::from_millis(250))?;
                    Ok((fields_match(&status, &expected)?, status))
                })
            }
            "wait_connected" => {
                let peer = step.fields.get("peer").and_then(Value::as_str);
                self.poll_value(timeout, || {
                    let status = self.ipc(&id, "status.get", json!({}), Duration::from_millis(250))?;
                    let connected = if let Some(peer) = peer {
                        status
                            .get("peers")
                            .and_then(Value::as_array)
                            .is_some_and(|items| items.iter().any(|p| p["connected"] == true && p["name"] == peer))
                    } else {
                        status["state"] == "connected"
                            || status
                                .get("peers")
                                .and_then(Value::as_array)
                                .is_some_and(|items| items.iter().any(|p| p["connected"] == true))
                    };
                    Ok((connected, status))
                })
            }
            "inject" => {
                let events = step.value("events");
                if !events.is_array() {
                    return Err(Failure::runner("inject requires events array"));
                }
                let mut params = json!({"events":events});
                for key in ["seed", "now_ms", "hid_seed", "click_interval_ms", "click_distance"] {
                    if let Some(value) = step.fields.get(key) {
                        params[key] = value.clone();
                    }
                }
                self.ipc(&id, "virtual.inject", params, timeout)
            }
            "inject_burst" => {
                let batch_size = step.number("batch_size", 512).min(512) as usize;
                let repeats = step.number("repeat", 1000).min(1000);
                let events = vec![step.value("event"); batch_size];
                let deadline = Instant::now() + timeout;
                for batch in 0..repeats {
                    self.ipc(&id, "virtual.inject", json!({"events":events}), Duration::from_secs(1))?;
                    std::thread::sleep(Duration::from_millis(1));
                    let status = self.ipc(&id, "status.get", json!({}), Duration::from_secs(1))?;
                    if status["last_error"]["code"] == "network_queue_full" {
                        return Ok(json!({"queue_full_after_batches": batch + 1}));
                    }
                    if Instant::now() >= deadline {
                        break;
                    }
                }
                Err(Failure::product("input burst did not reach queue limit"))
            }
            "assert_recorded" | "assert_recorded_tail" => {
                let expect_value = step
                    .fields
                    .get("expect")
                    .or_else(|| step.fields.get("expect_tail"))
                    .ok_or_else(|| Failure::runner("ledger assertion needs expect array"))?;
                let expected = expect_value
                    .as_array()
                    .ok_or_else(|| Failure::runner("expect must be an array"))?;
                let tail = step.op == "assert_recorded_tail" || step.fields.contains_key("expect_tail");
                let since = step.number("since", 0);
                self.poll_value(timeout, || {
                    let actual = self
                        .ledger(&id, since, Duration::from_millis(250))?
                        .into_iter()
                        .filter(|r| since == 0 || r["sequence"].as_u64().is_some_and(|n| n > since))
                        .filter(|r| !step.flag("input_only") || r.get("event").is_some_and(Value::is_object))
                        .collect::<Vec<_>>();
                    let actual_matches = ordered(&actual, expected, tail, step.flag("exact"));
                    Ok((actual_matches, json!(actual)))
                })
            }
            "ledger_mark" => {
                let ledger = self.ledger(&id, 0, timeout)?;
                Ok(
                    json!({"sequence":ledger.iter().filter_map(|r| r["sequence"].as_u64()).max().unwrap_or(0),"count":ledger.len()}),
                )
            }
            "assert_numbering" => {
                let ledger = self.ledger(&id, step.number("since", 0), timeout)?;
                numbering(&ledger, step)?;
                Ok(json!({"records":ledger.len(),"numbering":"valid"}))
            }
            "clipboard_set" => {
                let data = self.clipboard_payload(step)?;
                self.ipc(&id, "virtual.clipboard", json!({"set":data}), timeout)
            }
            "clipboard_get" => self.ipc(&id, "virtual.clipboard", json!({"get":true}), timeout),
            "assert_clipboard" => self.poll_value(timeout, || {
                let data = self.ipc(
                    &id,
                    "virtual.clipboard",
                    json!({"get":true}),
                    timeout.min(Duration::from_secs(5)),
                )?;
                Ok((clipboard_matches(&data, step)?, clipboard_summary(&data)))
            }),
            "set_screens" => self.ipc(
                &id,
                "virtual.set_screens",
                json!({"rects":step.value("rects")}),
                timeout,
            ),
            "set_permissions" => self.ipc(
                &id,
                "virtual.set_permissions",
                step.fields.get("permissions").cloned().unwrap_or_else(|| json!({})),
                timeout,
            ),
            "config_patch" => {
                let params = json!({"patch":step.value("patch")});
                self.safe_config(&params["patch"])?;
                self.control(&id, "config.patch", params, step)
            }
            "ctl" => self.control(
                &id,
                step.text("method")?,
                step.fields.get("params").cloned().unwrap_or_else(|| json!({})),
                step,
            ),
            "subscribe" => {
                #[cfg(unix)]
                self.subscribe(&id, timeout)?;
                #[cfg(not(unix))]
                return Err(Failure::environment("event subscriptions require Unix"));
                Ok(json!({"subscribed":true}))
            }
            "wait_event" => {
                #[cfg(unix)]
                {
                    self.wait_event(&id, step)
                }
                #[cfg(not(unix))]
                {
                    Err(Failure::environment("event subscriptions require Unix"))
                }
            }
            "kill" | "pause" | "resume" => {
                let signal = match step.op.as_str() {
                    "pause" => "STOP",
                    "resume" => "CONT",
                    _ => step.fields.get("signal").and_then(Value::as_str).unwrap_or("KILL"),
                };
                self.state(&id)?
                    .process
                    .as_ref()
                    .ok_or_else(|| Failure::runner("signal requires an owned child"))?
                    .signal(signal)?;
                Ok(json!({"signal":signal}))
            }
            "start" | "stop" | "restart" => {
                if step.op != "start" {
                    self.stop(&id, !step.flag("force"))?;
                }
                if step.op != "stop" {
                    self.start(&id)?;
                }
                Ok(json!({"node":id,"action":step.op}))
            }
            "parent_exit" => {
                write_private(&self.state(&id)?.dir.join("parent.exit"), b"exit")?;
                Ok(json!({"parent_exit":true}))
            }
            "wait_exit" => {
                let expected = step.number("code", 0) as i32;
                if self.state(&id)?.spec.parent {
                    let dir = self.state(&id)?.dir.clone();
                    let pid = self
                        .state(&id)?
                        .pid
                        .ok_or_else(|| Failure::runner("missing daemon PID"))?;
                    // Reap the gate so the daemon's parent-pid watch sees it disappear.
                    self.state_mut(&id)?
                        .process
                        .as_mut()
                        .ok_or_else(|| Failure::runner("missing parent"))?
                        .wait(timeout)?;
                    let exit_file = dir.join("daemon.exit");
                    self.poll_value(timeout, || {
                        let code = std::fs::read_to_string(&exit_file)
                            .ok()
                            .and_then(|v| v.trim().parse::<i32>().ok());
                        Ok((code == Some(expected), json!({"pid":pid,"exit_code":code})))
                    })
                } else {
                    let status = self
                        .state_mut(&id)?
                        .process
                        .as_mut()
                        .ok_or_else(|| Failure::runner("wait_exit needs a child"))?
                        .wait(timeout)?;
                    if status.code() != Some(expected) {
                        return Err(Failure::product(format!("unexpected child exit status {status}")));
                    }
                    Ok(json!({"exit_code":expected}))
                }
            }
            "trust_crash_recovery" => {
                let dir = self.state(&id)?.dir.join("crash-trust");
                let tls = dir.join("tls");
                crate::util::private_dir(&tls)?;
                // A real populated trust file keeps the writer in its locked
                // read/merge interval long enough to observe kernel ownership.
                let mut records = String::new();
                for n in 0..8192 {
                    records.push_str(&format!("v2:sha256:{n:064x}\n"));
                }
                write_private(&tls.join("trusted-servers"), records.as_bytes())?;
                let lock = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(tls.join(".trust-lock"))?;
                let fingerprint = "ff".repeat(32);
                let mut command = Command::new(&self.options.binary);
                command.args(["trust", "approve", &fingerprint, "--dir"]).arg(&dir);
                isolated_env(&mut command, &dir, &dir.join("unused.sock"));
                let mut writer = crate::process::Process::spawn(&mut command, &dir, "interrupted-approve")?;
                let deadline = Instant::now() + timeout;
                loop {
                    match lock.try_lock() {
                        Ok(()) => {
                            lock.unlock()?;
                        }
                        Err(std::fs::TryLockError::WouldBlock) => break,
                        Err(std::fs::TryLockError::Error(error)) => return Err(error.into()),
                    }
                    if writer.poll()?.is_some() || Instant::now() >= deadline {
                        return Err(Failure::product("could not observe approval lock ownership"));
                    }
                    std::thread::sleep(Duration::from_micros(100));
                }
                writer.signal("KILL")?;
                writer.wait(Duration::from_secs(2))?;
                lock.try_lock()
                    .map_err(|e| Failure::product(format!("lock survived killed writer: {e}")))?;
                lock.unlock()?;
                let (code, _) = command_output(&mut command, &dir, "retry-approve", timeout)?;
                if code != 0 {
                    return Err(Failure::product("approval after killed writer failed"));
                }
                let pins = std::fs::read_to_string(tls.join("trusted-servers"))?;
                if !pins.contains(&fingerprint) {
                    return Err(Failure::product("retry approval did not save pin"));
                }
                Ok(json!({"killed_while_locked":true,"retry_approved":true}))
            }
            "raw_send" => {
                let data = wire::build(step)?;
                self.raw(&id)?.send(&data, step, timeout)?;
                Ok(json!({"sent_bytes":data.len(),"repeat":step.number("repeat",1)}))
            }
            "raw_expect" => {
                let data = wire::build(step)?;
                self.raw(&id)?
                    .expect(&data, timeout, step.flag("framed"), step.flag("scan"))?;
                Ok(json!({"expected_bytes":data.len()}))
            }
            "raw_expect_absent" => {
                let codes = step.value("codes");
                let codes = codes
                    .as_array()
                    .ok_or_else(|| Failure::runner("codes must be an array"))?;
                let codes = codes
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .map(str::to_owned)
                            .ok_or_else(|| Failure::runner("code must be a string"))
                    })
                    .collect::<Result<Vec<_>>>()?;
                self.raw(&id)?.expect_absent(&codes, timeout)?;
                Ok(json!({"absent": codes}))
            }
            "raw_expect_closed" => {
                self.raw(&id)?.expect_closed(timeout, step.flag("reject_bytes"))?;
                Ok(json!({"closed":true}))
            }
            "lab_drive" | "lab_pointer" | "lab_events_dump" | "lab_clipboard" | "lab_ctl" => {
                let mut args = match step.op.as_str() {
                    "lab_drive" => vec!["drive".into()],
                    "lab_pointer" => vec!["pointer".into()],
                    "lab_events_dump" => vec!["events".into(), "dump".into()],
                    "lab_clipboard" => vec!["clipboard".into(), step.string("action", "get")],
                    _ => Vec::new(),
                };
                if step.op == "lab_clipboard" && step.string("action", "get") == "set" {
                    args.push(step.text("text")?.into());
                }
                if let Some(values) = step.fields.get("args") {
                    for value in values
                        .as_array()
                        .ok_or_else(|| Failure::runner("lab args must be strings"))?
                    {
                        args.push(
                            value
                                .as_str()
                                .ok_or_else(|| Failure::runner("lab arg is not string"))?
                                .into(),
                        );
                    }
                }
                if step.op == "lab_ctl"
                    && !matches!(
                        args.first().map(String::as_str),
                        Some("fingerprint" | "events" | "status" | "logs")
                    )
                {
                    return Err(Failure::runner(
                        "lab_ctl allows fingerprint/events/status/logs; lifecycle uses node start/stop",
                    ));
                }
                let output = self.lab(&id, &args)?;
                if step.op == "lab_pointer" {
                    let parsed: serde_json::Map<String, Value> = output
                        .lines()
                        .filter_map(|line| {
                            let (key, value) = line.split_once('=')?;
                            Some((key.to_ascii_lowercase(), json!(value.parse::<i64>().ok()?)))
                        })
                        .collect();
                    let pointer = Value::Object(parsed);
                    if let Some(fields) = step.fields.get("fields") {
                        if !fields_match(&pointer, fields)? {
                            return Err(Failure::product("lab pointer field mismatch"));
                        }
                    }
                    return Ok(pointer);
                }
                if let Some(regex) = step.fields.get("regex").and_then(Value::as_str) {
                    self.assert_regex(&output, regex)?;
                }
                if let Some(text) = step.fields.get("expect_text").and_then(Value::as_str) {
                    if output.trim_end_matches('\n') != text {
                        return Err(Failure::product("lab text assertion mismatch (content redacted)"));
                    }
                }
                if step.op == "lab_events_dump" && step.flag("nonempty") && output.trim().is_empty() {
                    return Err(Failure::product("lab event recorder is empty"));
                }
                if step.op == "lab_ctl" && args.first().is_some_and(|s| s == "fingerprint") {
                    return Ok(
                        json!({"synergy":output.lines().next().unwrap_or_default(),"colon":output.lines().nth(1).unwrap_or_default()}),
                    );
                }
                Ok(json!({"output_bytes":output.len(),"sha256":digest(output.as_bytes())}))
            }
            "cli" => self.cli(step),
            "assert_file" => self.assert_file(step),
            "copy_identity" => {
                let source = self.state(step.text("source")?)?.dir.join("tls");
                let dest = self.state(&id)?.dir.join("tls");
                let mut copied = 0;
                for entry in std::fs::read_dir(source)? {
                    let entry = entry?;
                    if entry.file_type()?.is_file()
                        && (entry.path().extension().is_some_and(|s| s == "pem")
                            || matches!(entry.file_name().to_str(), Some("trusted-servers" | "trusted-clients")))
                    {
                        write_private(&dest.join(entry.file_name()), &std::fs::read(entry.path())?)?;
                        copied += 1;
                    }
                }
                if copied == 0 {
                    return Err(Failure::product("no disposable PEM identity files were generated"));
                }
                Ok(json!({"copied_identity_files":copied}))
            }
            "set_lab_trust" => {
                if self.state(&id)?.lab.is_some() {
                    return Err(Failure::runner("set_lab_trust requires a stopped lab node"));
                }
                let pin = step.text("fingerprint")?.trim().to_owned();
                self.state_mut(&id)?.spec.trust.insert("fingerprint".into(), vec![pin]);
                Ok(json!({"seeded":true}))
            }
            "set_lab_screens" => {
                if self.state(&id)?.lab.is_some() {
                    return Err(Failure::runner("set_lab_screens requires a stopped lab node"));
                }
                let file = self.confined(step.text("path")?)?;
                let text = std::fs::read_to_string(file)?;
                self.state_mut(&id)?.spec.screens = Some(text);
                Ok(json!({"layout_loaded":true}))
            }
            "write_file" => {
                let path = self.confined(step.text("path")?)?;
                let bytes = if step.fields.contains_key("fixture") {
                    let source = self.confined(step.text("fixture")?)?;
                    let text = std::fs::read_to_string(source)?;
                    self.expand(&json!(text))?
                        .as_str()
                        .unwrap_or_default()
                        .as_bytes()
                        .to_vec()
                } else {
                    step.text("text")?.as_bytes().to_vec()
                };
                write_private(&path, &bytes)?;
                Ok(json!({"bytes":bytes.len(),"sha256":digest(&bytes)}))
            }
            "fixture_dib" => {
                let bytes = dib8();
                let path = self.confined(step.text("path")?)?;
                write_private(&path, &bytes)?;
                Ok(json!({"bytes":bytes.len(),"sha256":digest(&bytes),"path":path}))
            }
            "sleep" => {
                let ms = step.number("ms", 1);
                if ms > 60_000 {
                    return Err(Failure::runner("sleep exceeds 60 s; use a bounded wait"));
                }
                std::thread::sleep(Duration::from_millis(ms));
                Ok(json!({"slept_ms":ms}))
            }
            "measure" => self.measure(&id, step),
            "assert_json" => {
                let value = step.value("actual");
                if step.fields.get("expect").is_some_and(|expect| !matches(&value, expect)) {
                    return Err(Failure::product("JSON assertion mismatch (content redacted)"));
                }
                if let Some(fields) = step.fields.get("fields") {
                    if !fields_match(&value, fields)? {
                        return Err(Failure::product("JSON field predicate mismatch"));
                    }
                }
                Ok(summary(&value))
            }
            _ => Err(Failure::runner(format!("unknown step {}", step.op))),
        }
    }
    pub fn poll_value<F>(&self, timeout: Duration, mut f: F) -> Result<Value>
    where
        F: FnMut() -> Result<(bool, Value)>,
    {
        let deadline = Instant::now() + timeout;
        let mut last = String::from("predicate did not match");
        loop {
            match f() {
                Ok((true, value)) => return Ok(value),
                Ok((false, _)) => {}
                Err(error) if error.kind == "runner" => return Err(error),
                Err(error) => last = error.to_string(),
            }
            if Instant::now() >= deadline {
                return Err(Failure::product(format!("bounded wait failed: {last}")));
            }
            std::thread::sleep(Duration::from_millis(15));
        }
    }
    fn control(&self, id: &str, method: &str, params: Value, step: &Step) -> Result<Value> {
        // Apply all IPC safety checks even when an error response is expected.
        if step.fields.contains_key("expect_error") {
            if method.starts_with("permissions.")
                || method.starts_with("connection.")
                || method == "config.set"
                || method == "config.import_synergy"
            {
                return Err(Failure::runner("error probes cannot bypass safety validation"));
            }
            if method == "config.patch" {
                self.safe_config(params.get("patch").unwrap_or(&Value::Null))?;
            }
            let response = ipc_response(&self.state(id)?.socket, method, params, self.timeout(step))?;
            let expected = step.text("expect_error")?;
            if response.pointer("/error/code").and_then(Value::as_str) != Some(expected) {
                return Err(Failure::product(format!(
                    "{method} did not return expected error code {expected}"
                )));
            }
            return Ok(response);
        }
        if method == "config.import_synergy" {
            let path = params
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| Failure::runner("import needs path"))?;
            self.confined(path)?;
            return Err(Failure::runner("use cli import into a confined file, validate it, then config.patch; live import can bypass port rails"));
        }
        let result = self.ipc(id, method, params, self.timeout(step))?;
        if let Some(expect) = step.fields.get("expect") {
            if !matches(&result, expect) {
                return Err(Failure::product(format!(
                    "{method} JSON expectation did not match (content redacted)"
                )));
            }
        }
        if let Some(fields) = step.fields.get("fields") {
            if !fields_match(&result, fields)? {
                return Err(Failure::product(format!("{method} field predicates did not match")));
            }
        }
        Ok(result)
    }
    fn cli(&mut self, step: &Step) -> Result<Value> {
        let id = step.text("node")?;
        let state = self.state(id)?;
        let dir = state.dir.clone();
        let mut args = Vec::new();
        for value in step
            .value("args")
            .as_array()
            .ok_or_else(|| Failure::runner("cli requires args array"))?
        {
            args.push(
                value
                    .as_str()
                    .ok_or_else(|| Failure::runner("CLI arg must be a string"))?
                    .to_owned(),
            );
        }
        if !matches!(
            args.first().map(String::as_str),
            Some("version" | "fingerprint" | "trust" | "import-synergy" | "export-synergy-conf")
        ) || args
            .iter()
            .any(|s| s == "--apply" || s.contains("24800") || s.starts_with("192.168."))
        {
            return Err(Failure::runner(
                "CLI command is outside the safe offline subcommand set",
            ));
        }
        if args[0] == "import-synergy" && !args.iter().any(|s| s == "--out") {
            return Err(Failure::runner("imports require an explicit confined --out"));
        }
        if args[0] == "trust" && args.get(1).is_some_and(|s| s == "import-synergy") {
            self.confined(
                args.get(2)
                    .ok_or_else(|| Failure::runner("trust import needs a confined directory"))?,
            )?;
        }
        if !matches!(args[0].as_str(), "version" | "import-synergy") && !args.iter().any(|s| s == "--config") {
            args.extend(["--config".into(), state.config.to_string_lossy().into_owned()]);
        }
        for pair in args.windows(2) {
            if ["--from", "--out", "--config"].contains(&pair[0].as_str()) {
                self.confined(&pair[1])?;
            }
        }
        let mut command = Command::new(&self.options.binary);
        command.args(&args);
        isolated_env(&mut command, &dir, &state.socket);
        self.commands += 1;
        let (code, stdout) = command_output(
            &mut command,
            &dir,
            &format!("cli-{}", self.commands),
            self.timeout(step),
        )?;
        if code != step.number("code", 0) as i32 {
            return Err(Failure::product(format!(
                "openergy {} returned exit code {code}",
                args[0]
            )));
        }
        if let Some(regex) = step.fields.get("regex").and_then(Value::as_str) {
            self.assert_regex(&stdout, regex)?;
        }
        if let Some(regex) = step.fields.get("output_regex").and_then(Value::as_str) {
            let stderr = std::fs::read_to_string(dir.join(format!("cli-{}.stderr", self.commands)))?;
            self.assert_regex(&format!("{stdout}\n{stderr}"), regex)?;
        }
        if let Some(path) = step.fields.get("stdout_path").and_then(Value::as_str) {
            write_private(&self.confined(path)?, stdout.as_bytes())?;
        }
        Ok(json!({"code":code,"stdout":stdout}))
    }
    pub fn assert_regex(&mut self, text: &str, regex: &str) -> Result<()> {
        // POSIX ERE via the real system tool keeps dependencies on the frozen allowlist.
        let input = self.dir.join("regex-input");
        write_private(&input, text.as_bytes())?;
        let mut command = Command::new("grep");
        command.args(["-E", "-q", "--", regex]).arg(&input);
        self.commands += 1;
        let (code, _) = command_output(
            &mut command,
            &self.dir,
            &format!("regex-{}", self.commands),
            Duration::from_secs(2),
        )?;
        let _ = std::fs::remove_file(input);
        match code {
            0 => Ok(()),
            1 => Err(Failure::product("file/output regex did not match")),
            _ => Err(Failure::runner("invalid POSIX extended regex")),
        }
    }
    fn assert_file(&mut self, step: &Step) -> Result<Value> {
        let path = self.confined(step.text("path")?)?;
        let exists = step.fields.get("exists").and_then(Value::as_bool).unwrap_or(true);
        if path.exists() != exists {
            return Err(Failure::product(format!(
                "artifact existence mismatch: {}",
                path.display()
            )));
        }
        if !exists {
            return Ok(json!({"exists":false}));
        }
        let metadata = std::fs::metadata(&path)?;
        #[cfg(unix)]
        if let Some(mode) = step.fields.get("mode").and_then(Value::as_str) {
            use std::os::unix::fs::PermissionsExt;
            let expected = u32::from_str_radix(mode.trim_start_matches("0o"), 8)
                .map_err(|_| Failure::runner("mode must be octal"))?;
            if metadata.permissions().mode() & 0o777 != expected {
                return Err(Failure::product("artifact mode mismatch"));
            }
        }
        if metadata.is_dir() {
            return Ok(json!({"exists":true,"directory":true}));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            if metadata.file_type().is_socket() {
                return Ok(json!({"exists":true,"socket":true}));
            }
        }
        if metadata.len() > 64 * 1024 * 1024 {
            return Err(Failure::runner("artifact exceeds 64 MiB read limit"));
        }
        let bytes = std::fs::read(&path)?;
        let hash = digest(&bytes);
        if step
            .fields
            .get("sha256")
            .and_then(Value::as_str)
            .is_some_and(|s| s != hash)
        {
            return Err(Failure::product("artifact SHA-256 mismatch"));
        }
        if let Some(regex) = step.fields.get("regex").and_then(Value::as_str) {
            self.assert_regex(
                std::str::from_utf8(&bytes).map_err(|_| Failure::product("regex artifact is not UTF-8"))?,
                regex,
            )?;
        }
        if let Some(expected) = step.fields.get("toml_expect") {
            let text = std::str::from_utf8(&bytes).map_err(|_| Failure::product("TOML artifact is not UTF-8"))?;
            let actual: Value =
                toml::from_str(text).map_err(|e| Failure::product(format!("artifact TOML parse: {e}")))?;
            self.safe_config(&actual)?;
            if !matches(&actual, expected) {
                return Err(Failure::product("TOML artifact expectation mismatch"));
            }
        }
        Ok(json!({"exists":true,"bytes":bytes.len(),"sha256":hash}))
    }
    fn clipboard_payload(&self, step: &Step) -> Result<Value> {
        if let Some(formats) = step.fields.get("formats") {
            return Ok(json!({"formats":formats}));
        }
        let bytes = if let Some(file) = step.fields.get("fixture").and_then(Value::as_str) {
            let path = self.confined(file)?;
            if std::fs::metadata(&path)?.len() > 16 * 1024 * 1024 {
                return Err(Failure::runner("clipboard fixture exceeds 16 MiB"));
            }
            std::fs::read(path)?
        } else {
            step.text("text")?.as_bytes().to_vec()
        };
        let kind = step.string("kind", "text_utf8");
        let format = if step.string("encoding", "bytes") == "base64" {
            json!({"kind":kind,"bytes_base64":base64(&bytes)})
        } else {
            if bytes.len() > 3 * 1024 * 1024 {
                return Err(Failure::runner(
                    "large clipboard fixture needs encoding=base64 to fit the 16 MiB IPC cap",
                ));
            }
            json!({"kind":kind,"bytes":bytes})
        };
        Ok(json!({"formats":[format]}))
    }
    fn measure(&mut self, id: &str, step: &Step) -> Result<Value> {
        if step.string("kind", "resources") == "latency" {
            let receiver = step.text("receiver")?;
            let count = step.number("count", 1000).clamp(1, 10000);
            let mut times = Vec::new();
            let mut sequence = self
                .ledger(receiver, 0, Duration::from_secs(1))?
                .iter()
                .filter_map(|v| v["sequence"].as_u64())
                .max()
                .unwrap_or(0);
            for index in 0..count {
                let x = 200 + (index % 1000) as i32;
                let y = 300 + (index / 1000) as i32;
                let start = Instant::now();
                self.ipc(
                    id,
                    "virtual.inject",
                    json!({"events":[{"type":"mouse_move_abs","x":x,"y":y}]}),
                    self.timeout(step),
                )?;
                let deadline = Instant::now() + self.timeout(step);
                loop {
                    let ledger = self.ledger(receiver, sequence, Duration::from_millis(200))?;
                    if let Some(record) = ledger.iter().find(|r| {
                        r["sequence"].as_u64().is_some_and(|s| s > sequence)
                            && r["event"]["type"] == "mouse_move_abs"
                            && r["event"]["x"] == x
                            && r["event"]["y"] == y
                    }) {
                        sequence = record["sequence"]
                            .as_u64()
                            .ok_or_else(|| Failure::product("ledger missing sequence"))?;
                        times.push(start.elapsed().as_secs_f64() * 1000.0);
                        break;
                    }
                    if Instant::now() >= deadline {
                        return Err(Failure::product("latency move never reached the receiver ledger"));
                    }
                    std::thread::sleep(Duration::from_micros(100));
                }
            }
            times.sort_by(f64::total_cmp);
            let percentile = |p: f64| {
                times[((times.len() as f64 * p).ceil() as usize)
                    .saturating_sub(1)
                    .min(times.len() - 1)]
            };
            let result = json!({"kind":"capture_to_recorded_upper_bound","samples":times.len(),"p50_ms":percentile(0.50),"p95_ms":percentile(0.95),"p99_ms":percentile(0.99),"max_ms":times[times.len()-1]});
            self.measurements.push(result.clone());
            let max = step.fields.get("p99_ms").and_then(Value::as_f64).unwrap_or(5.0);
            if percentile(0.99) >= max {
                return Err(Failure::product("capture-to-recorded p99 exceeded budget"));
            }
            Ok(result)
        } else {
            let pid = self
                .state(id)?
                .pid
                .ok_or_else(|| Failure::runner("resource measure needs daemon PID"))?;
            let window = Duration::from_millis(step.number("window_ms", 60_000).clamp(50, 180_000));
            let first = self.ps_sample(pid)?;
            let started = Instant::now();
            let mut peak = first.0;
            while started.elapsed() < window {
                std::thread::sleep(Duration::from_millis(100).min(window.saturating_sub(started.elapsed())));
                peak = peak.max(self.ps_sample(pid)?.0);
            }
            let last = self.ps_sample(pid)?;
            let cpu = ((last.1 - first.1).max(0.0) / started.elapsed().as_secs_f64()) * 100.0;
            let result = json!({"kind":"resources","rss_peak_bytes":peak,"cpu_percent":cpu,"window_ms":started.elapsed().as_millis()});
            self.measurements.push(result.clone());
            let max_rss = step.number("rss_bytes", 30_000_000);
            let max_cpu = step.fields.get("cpu_percent").and_then(Value::as_f64).unwrap_or(0.1);
            if peak >= max_rss || cpu >= max_cpu {
                return Err(Failure::product("RSS/CPU window exceeded budget"));
            }
            Ok(result)
        }
    }
    fn ps_sample(&mut self, pid: u32) -> Result<(u64, f64)> {
        let mut command = Command::new("ps");
        command.args(["-o", "rss=,time=", "-p", &pid.to_string()]);
        self.commands += 1;
        let (code, output) = command_output(
            &mut command,
            &self.dir,
            &format!("ps-{}", self.commands),
            Duration::from_secs(2),
        )?;
        if code != 0 {
            return Err(Failure::product("daemon exited during resource measurement"));
        }
        let mut fields = output.split_whitespace();
        let rss = fields
            .next()
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| Failure::environment("cannot parse ps RSS"))?
            * 1024;
        let raw_time = fields
            .next()
            .ok_or_else(|| Failure::environment("missing ps CPU time"))?;
        let time = raw_time
            .split(':')
            .try_fold(0.0, |sum, s| s.parse::<f64>().map(|v| sum * 60.0 + v))
            .map_err(|_| Failure::environment("cannot parse ps CPU time"))?;
        Ok((rss, time))
    }
}
fn ordered(actual: &[Value], expected: &[Value], tail: bool, exact: bool) -> bool {
    if expected.is_empty() {
        return actual.is_empty();
    }
    if actual.len() < expected.len() {
        return false;
    }
    if exact {
        return actual.len() == expected.len() && actual.iter().zip(expected).all(|(a, e)| matches(a, e));
    }
    let mut next = 0;
    for (index, value) in actual.iter().enumerate() {
        if matches(value, &expected[next]) {
            next += 1;
            if next == expected.len() {
                return !tail || index + 1 == actual.len();
            }
        }
    }
    false
}
fn numbering(ledger: &[Value], step: &Step) -> Result<()> {
    let seed = step.number("seed", 0) as i64;
    let mut last = seed;
    let mut held = std::collections::BTreeMap::new();
    let mut states = Vec::new();
    let mut previous_sequence = None;
    for record in ledger {
        let sequence = record["sequence"]
            .as_u64()
            .ok_or_else(|| Failure::product("numbering ledger lacks sequence"))?;
        if previous_sequence.is_some_and(|previous| sequence <= previous) {
            return Err(Failure::product("ledger sequence is not strictly increasing"));
        }
        previous_sequence = Some(sequence);
        if record["release_all"] == true {
            held.clear();
            continue;
        }
        let event = &record["event"];
        let kind = event["type"].as_str().unwrap_or("");
        if kind == "mouse_down" {
            let button = event["button"]
                .as_u64()
                .ok_or_else(|| Failure::product("mouse down lacks button"))?;
            let number = record["click_number"]
                .as_i64()
                .ok_or_else(|| Failure::product("mouse down lacks click_number"))?;
            let state = record["click_state"]
                .as_i64()
                .ok_or_else(|| Failure::product("mouse down lacks click_state"))?;
            if number <= last || held.contains_key(&button) {
                return Err(Failure::product("click number did not rise or button was already down"));
            }
            last = number;
            states.push(state);
            held.insert(button, (number, state));
        } else if kind == "mouse_up" {
            let button = event["button"]
                .as_u64()
                .ok_or_else(|| Failure::product("mouse up lacks button"))?;
            let expected = held
                .remove(&button)
                .ok_or_else(|| Failure::product("unpaired mouse up"))?;
            if record["click_number"].as_i64() != Some(expected.0) || record["click_state"].as_i64() != Some(expected.1)
            {
                return Err(Failure::product("mouse up did not reuse its down number/state"));
            }
        } else if kind.starts_with("mouse_move")
            && !held.is_empty()
            && !held
                .values()
                .any(|(n, s)| record["click_number"].as_i64() == Some(*n) && record["click_state"].as_i64() == Some(*s))
        {
            return Err(Failure::product("drag did not reuse its down number/state"));
        }
    }
    if states.len() < step.number("min_downs", 1) as usize || (!step.flag("allow_held") && !held.is_empty()) {
        return Err(Failure::product(
            "numbering assertion has too few downs or unmatched buttons",
        ));
    }
    if let Some(expected) = step.fields.get("states") {
        if &json!(states) != expected {
            return Err(Failure::product("click-state sequence did not match"));
        }
    }
    Ok(())
}
fn clipboard_formats(value: &Value) -> Option<&Vec<Value>> {
    value
        .get("formats")
        .or_else(|| value.pointer("/data/formats"))
        .and_then(Value::as_array)
}
fn format_bytes(format: &Value) -> Result<Vec<u8>> {
    if let Some(bytes) = format.get("bytes").and_then(Value::as_array) {
        return bytes
            .iter()
            .map(|v| {
                v.as_u64()
                    .filter(|v| *v <= 255)
                    .map(|v| v as u8)
                    .ok_or_else(|| Failure::product("invalid clipboard byte"))
            })
            .collect();
    }
    if let Some(text) = format.get("bytes_base64").and_then(Value::as_str) {
        return decode_base64(text);
    }
    Err(Failure::product("clipboard format has no supported byte encoding"))
}
fn clipboard_matches(value: &Value, step: &Step) -> Result<bool> {
    let Some(formats) = clipboard_formats(value) else {
        return Ok(false);
    };
    if step.flag("empty") {
        return Ok(formats.is_empty());
    }
    let kind = step.string("kind", "text_utf8");
    let Some(format) = formats.iter().find(|v| v["kind"] == kind) else {
        return Ok(false);
    };
    let bytes = format_bytes(format)?;
    if let Some(text) = step.fields.get("text").and_then(Value::as_str) {
        if bytes != text.as_bytes() {
            return Ok(false);
        }
    }
    if let Some(hash) = step.fields.get("sha256").and_then(Value::as_str) {
        if digest(&bytes) != hash {
            return Ok(false);
        }
    }
    if let Some(size) = step.fields.get("bytes").and_then(Value::as_u64) {
        if bytes.len() as u64 != size {
            return Ok(false);
        }
    }
    Ok(true)
}
fn clipboard_summary(value: &Value) -> Value {
    json!({"formats":clipboard_formats(value).map(|items| items.iter().map(|f| {
        let bytes = format_bytes(f).unwrap_or_default();
        json!({"kind":f["kind"],"size":bytes.len(),"sha256":digest(&bytes)})
    }).collect::<Vec<_>>()).unwrap_or_default()})
}
/// 40-byte BITMAPINFOHEADER + exactly 8 MiB of deterministic BGRA pixels.
/// 2048*1024*4 pixels, bottom-up, BI_RGB, 32 bits; no BMP file header.
pub fn dib8() -> Vec<u8> {
    let pixels = 8 * 1024 * 1024;
    let mut bytes = vec![0; 40 + pixels];
    bytes[0..4].copy_from_slice(&40u32.to_le_bytes());
    bytes[4..8].copy_from_slice(&2048i32.to_le_bytes());
    bytes[8..12].copy_from_slice(&1024i32.to_le_bytes());
    bytes[12..14].copy_from_slice(&1u16.to_le_bytes());
    bytes[14..16].copy_from_slice(&32u16.to_le_bytes());
    bytes[20..24].copy_from_slice(&(pixels as u32).to_le_bytes());
    for (i, pixel) in bytes[40..].as_chunks_mut::<4>().0.iter_mut().enumerate() {
        pixel.copy_from_slice(&[(i % 251) as u8, ((i / 251) % 251) as u8, ((i * 17) % 251) as u8, 255]);
    }
    bytes
}
const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let (a, b, c) = (chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0));
        out.push(B64[(a >> 2) as usize] as char);
        out.push(B64[(((a & 3) << 4) | (b >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            B64[(((b & 15) << 2) | (c >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[(c & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}
fn decode_base64(text: &str) -> Result<Vec<u8>> {
    if !text.len().is_multiple_of(4) || text.len() > 24 * 1024 * 1024 {
        return Err(Failure::product("invalid clipboard base64 length"));
    }
    let mut out = Vec::new();
    for (index, chunk) in text.as_bytes().as_chunks::<4>().0.iter().enumerate() {
        let last = (index + 1) * 4 == text.len();
        let mut values = [0u8; 4];
        for i in 0..4 {
            if chunk[i] == b'=' && i >= 2 && last {
                continue;
            }
            values[i] = B64
                .iter()
                .position(|c| *c == chunk[i])
                .ok_or_else(|| Failure::product("invalid clipboard base64 digit"))? as u8;
        }
        if chunk[2] == b'=' && chunk[3] != b'=' {
            return Err(Failure::product("invalid clipboard base64 padding"));
        }
        out.push((values[0] << 2) | (values[1] >> 4));
        if chunk[2] != b'=' {
            out.push((values[1] << 4) | (values[2] >> 2));
        }
        if chunk[3] != b'=' {
            out.push((values[2] << 6) | values[3]);
        }
    }
    Ok(out)
}
