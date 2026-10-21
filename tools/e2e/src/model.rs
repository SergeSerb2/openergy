use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Failure>;

#[derive(Clone, Debug, Serialize)]
pub struct Failure {
    pub kind: &'static str,
    pub message: String,
}
impl Failure {
    pub fn runner(message: impl Into<String>) -> Self {
        Self {
            kind: "runner",
            message: message.into(),
        }
    }
    pub fn product(message: impl Into<String>) -> Self {
        Self {
            kind: "product",
            message: message.into(),
        }
    }
    pub fn environment(message: impl Into<String>) -> Self {
        Self {
            kind: "environment",
            message: message.into(),
        }
    }
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}
impl From<std::io::Error> for Failure {
    fn from(e: std::io::Error) -> Self {
        Self::runner(e.to_string())
    }
}
impl From<serde_json::Error> for Failure {
    fn from(e: serde_json::Error) -> Self {
        Self::runner(e.to_string())
    }
}
fn yes() -> bool {
    true
}
fn default_kind() -> String {
    "openergy".into()
}
fn default_role() -> String {
    "off".into()
}
fn virtual_platform() -> String {
    "virtual".into()
}
fn timeout() -> u64 {
    5000
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub name: String,
    pub requirement: String,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default = "timeout")]
    pub timeout_ms: u64,
    #[serde(default, rename = "node")]
    pub nodes: Vec<Node>,
    #[serde(default, rename = "step")]
    pub steps: Vec<Step>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default = "default_role")]
    pub role: String,
    pub name: Option<String>,
    #[serde(default = "virtual_platform")]
    pub platform: String,
    pub remote: Option<String>,
    pub port: Option<u16>,
    #[serde(default)]
    pub tls: bool,
    #[serde(default)]
    pub require_client_trust: bool,
    #[serde(default = "yes")]
    pub autostart: bool,
    #[serde(default)]
    pub parent: bool,
    pub peer: Option<String>,
    pub protocol: Option<String>,
    pub position: Option<String>,
    pub screens: Option<String>,
    #[serde(default)]
    pub config: BTreeMap<String, Value>,
    #[serde(default)]
    pub trust: BTreeMap<String, Vec<String>>,
}
#[derive(Clone, Deserialize)]
pub struct Step {
    #[serde(rename = "do")]
    pub op: String,
    #[serde(flatten)]
    pub fields: BTreeMap<String, Value>,
}
impl Step {
    pub fn value(&self, key: &str) -> Value {
        self.fields.get(key).cloned().unwrap_or(Value::Null)
    }
    pub fn text(&self, key: &str) -> Result<&str> {
        self.fields
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| Failure::runner(format!("{} requires string {key}", self.op)))
    }
    pub fn string(&self, key: &str, default: &str) -> String {
        self.fields.get(key).and_then(Value::as_str).unwrap_or(default).into()
    }
    pub fn number(&self, key: &str, default: u64) -> u64 {
        self.fields.get(key).and_then(Value::as_u64).unwrap_or(default)
    }
    pub fn flag(&self, key: &str) -> bool {
        self.fields.get(key).and_then(Value::as_bool).unwrap_or(false)
    }
}
#[derive(Clone)]
pub struct Options {
    pub root: PathBuf,
    pub binary: PathBuf,
    pub port_range: (u16, u16),
    pub lab_range: (u16, u16),
    pub keep: bool,
    pub scenario_dir: PathBuf,
    pub pattern: String,
    pub report: Option<PathBuf>,
}
#[derive(Serialize)]
pub struct ScenarioReport {
    pub scenario: String,
    pub requirement: String,
    pub outcome: &'static str,
    pub blocked_by: Vec<String>,
    pub failed_step: Option<usize>,
    pub elapsed_ms: u128,
    pub failure: Option<Failure>,
    pub artifact_dir: Option<PathBuf>,
    pub stderr_tails: BTreeMap<String, Vec<String>>,
    pub measurements: Vec<Value>,
}

impl Scenario {
    /// Validate every step before starting children, including steps after a blocker.
    pub fn validate(&self) -> Result<()> {
        for step in &self.steps {
            let required: &[&str] = match step.op.as_str() {
                "wait_status" | "assert_status" => &["node", "fields"],
                "wait_connected"
                | "assert_numbering"
                | "ledger_mark"
                | "clipboard_get"
                | "kill"
                | "pause"
                | "resume"
                | "stop"
                | "start"
                | "restart"
                | "subscribe"
                | "parent_exit"
                | "wait_exit"
                | "lab_pointer"
                | "lab_events_dump"
                | "trust_crash_recovery" => &["node"],
                "inject" => &["node", "events"],
                "assert_recorded" | "assert_recorded_tail" => &["node"],
                "clipboard_set" | "assert_clipboard" => &["node"],
                "set_screens" => &["node", "rects"],
                "set_permissions" => &["node", "permissions"],
                "config_patch" => &["node", "patch"],
                "ctl" => &["node", "method"],
                "wait_event" => &["node", "event"],
                "raw_send" | "raw_expect" | "raw_expect_closed" | "raw_expect_absent" => &["node"],
                "lab_drive" | "lab_ctl" | "cli" => &["node", "args"],
                "lab_clipboard" => &["node"],
                "assert_file" | "write_file" | "fixture_dib" => &["path"],
                "copy_identity" => &["node", "source"],
                "set_lab_trust" => &["node", "fingerprint"],
                "set_lab_screens" => &["node", "path"],
                "sleep" => &["ms"],
                "measure" => &["node"],
                "assert_json" => &["actual"],
                other => return Err(Failure::runner(format!("{}: unknown step {other}", self.name))),
            };
            for field in required {
                if !step.fields.contains_key(*field) {
                    return Err(Failure::runner(format!("{}: {} requires {field}", self.name, step.op)));
                }
            }
            if let Some(id) = step.fields.get("node").and_then(Value::as_str) {
                if !self.nodes.iter().any(|n| n.id == id) {
                    return Err(Failure::runner(format!("{}: unknown node {id}", self.name)));
                }
            }
            for key in [
                "timeout_ms",
                "since",
                "seed",
                "now_ms",
                "hid_seed",
                "repeat",
                "ms",
                "window_ms",
                "rss_bytes",
                "count",
                "min_downs",
            ] {
                if let Some(value) = step.fields.get(key) {
                    if value.as_u64().is_none() && !value.as_str().is_some_and(|s| s.starts_with("${")) {
                        return Err(Failure::runner(format!(
                            "{}: {} must be an unsigned integer",
                            self.name, key
                        )));
                    }
                }
            }
            if matches!(step.op.as_str(), "raw_send" | "raw_expect")
                && !step.fields.contains_key("hex")
                && !step.fields.contains_key("builder")
            {
                return Err(Failure::runner(format!(
                    "{}: raw bytes require hex or builder",
                    self.name
                )));
            }
        }
        Ok(())
    }
}
