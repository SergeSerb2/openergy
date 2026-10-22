//! External E2E runner. No session/protocol linkage or native desktop access.
mod guardian;
mod model;
mod process;
mod runner;
mod steps;
mod util;
mod wire;

use model::{Failure, Options, Result, Scenario};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "__guardian") {
        guardian::guardian_main();
        return;
    }
    let external = !args.first().is_some_and(|s| s == "__parent" || s == "__watch");
    if external {
        if let Err(e) = guardian::init() {
            eprintln!("openergy-e2e: {e}");
            std::process::exit(2);
        }
    }
    let result = if args.first().is_some_and(|s| s == "__parent") {
        process::parent_main(&args[1..]).map(|()| true)
    } else if args.first().is_some_and(|s| s == "__watch") {
        process::watch_main(&args[1..]).map(|()| true)
    } else {
        run(&args)
    };
    if external {
        guardian::finish();
    }
    match result {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("openergy-e2e: {error}");
            std::process::exit(2);
        }
    }
}
fn run(args: &[String]) -> Result<bool> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| Failure::runner("cannot locate workspace"))?
        .to_path_buf();
    let binary = std::env::var_os("OPENERGY_BIN")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("openergy")))
        })
        .unwrap_or_else(|| root.join("target/release/openergy"));
    let mut options = Options {
        root: root.clone(),
        binary,
        port_range: (24990, 24999),
        lab_range: (24930, 24939),
        keep: false,
        scenario_dir: root.join("tools/e2e/scenarios"),
        pattern: "*".into(),
        report: None,
    };
    let command = args.first().map(String::as_str).unwrap_or("help");
    if ["help", "--help", "-h"].contains(&command) {
        println!("openergy-e2e run [--scenario GLOB] [--port-range A-B] [--lab-port-range A-B] [--keep] [--report PATH]\nopenergy-e2e list [--scenario GLOB]\nOptional: --scenario-dir DIR, --openergy PATH. Only virtual daemons and isolated Docker peers run.");
        return Ok(true);
    }
    if !["run", "list"].contains(&command) {
        return Err(Failure::runner("expected run or list; see --help"));
    }
    let mut i = 1;
    while i < args.len() {
        let flag = &args[i];
        if flag == "--keep" {
            options.keep = true;
            i += 1;
            continue;
        }
        i += 1;
        let value = args
            .get(i)
            .ok_or_else(|| Failure::runner(format!("{flag} needs a value")))?;
        match flag.as_str() {
            "--scenario" => options.pattern = value.clone(),
            "--port-range" => options.port_range = util::port_range(value)?,
            "--lab-port-range" => options.lab_range = util::port_range(value)?,
            "--scenario-dir" => options.scenario_dir = PathBuf::from(value),
            "--openergy" => options.binary = PathBuf::from(value),
            "--report" | "--json" => options.report = Some(PathBuf::from(value)),
            _ => {
                return Err(Failure::runner(format!(
                    "unknown option {flag}; --allow-desktop is forbidden"
                )))
            }
        }
        i += 1;
    }
    if options.lab_range.0 < 24900
        || (options.port_range.0..=options.port_range.1)
            .any(|p| (options.lab_range.0..=options.lab_range.1).contains(&p))
    {
        return Err(Failure::runner(
            "lab range must start at 24900 or above and not overlap local range",
        ));
    }
    options.binary = std::fs::canonicalize(&options.binary).unwrap_or(options.binary);
    let mut paths: Vec<_> = std::fs::read_dir(&options.scenario_dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    paths.sort();
    let mut scenarios = Vec::new();
    let mut names = std::collections::BTreeSet::new();
    for path in paths {
        let text = std::fs::read_to_string(&path)?;
        let scenario: Scenario =
            toml::from_str(&text).map_err(|e| Failure::runner(format!("{}: {e}", path.display())))?;
        scenario.validate()?;
        if scenario.steps.is_empty() || !names.insert(scenario.name.clone()) || scenario.name.len() > 80 {
            return Err(Failure::runner(
                "scenario names must be unique, at most 80 chars, and contain steps",
            ));
        }
        if util::glob(&options.pattern, &scenario.name)
            || util::glob(
                &options.pattern,
                &path.file_name().unwrap_or_default().to_string_lossy(),
            )
        {
            scenarios.push(scenario);
        }
    }
    if scenarios.is_empty() {
        return Err(Failure::runner("no scenarios match; nothing ran"));
    }
    if command == "list" {
        for s in scenarios {
            println!("{}\t{}\t{}", s.name, s.blocked_by.join(","), s.requirement);
        }
        return Ok(true);
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| Failure::runner(e.to_string()))?
        .as_millis();
    let temp_base = if cfg!(unix) {
        PathBuf::from("/tmp")
    } else {
        std::env::temp_dir()
    };
    let run_dir = temp_base.join(format!("oe7-{}-{stamp}", std::process::id()));
    util::private_dir(&run_dir)?;
    let mut reports = Vec::new();
    for (index, scenario) in scenarios.into_iter().enumerate() {
        println!("RUN {}", scenario.name);
        let report = runner::run_scenario(&options, scenario, run_dir.join(format!("s{index}")));
        let class = report.failure.as_ref().map(|e| match e.kind {
            "product" if !report.blocked_by.is_empty() => {
                format!("expected, blocked by {}", report.blocked_by.join("/"))
            }
            "product" => "product mismatch".into(),
            "runner" => "runner bug".into(),
            _ => "environment".into(),
        });
        println!(
            "{}: {}{} ({} ms)",
            report.scenario,
            report.outcome,
            class.as_ref().map(|c| format!(" ({c})")).unwrap_or_default(),
            report.elapsed_ms
        );
        if let Some(error) = &report.failure {
            println!("  {}", error);
        }
        if let Some(dir) = &report.artifact_dir {
            println!("  artifacts: {}", dir.display());
        }
        if report.failure.is_some() {
            for (child, lines) in &report.stderr_tails {
                if !lines.is_empty() {
                    println!("  stderr {child}:\n    {}", lines.join("\n    "));
                }
            }
        }
        reports.push(report);
    }
    let passed = reports.iter().filter(|r| r.outcome == "PASS").count();
    let data = serde_json::json!({"format_version":1,"binary":options.binary,"port_range":options.port_range,"lab_port_range":options.lab_range,"passed":passed,"failed":reports.len()-passed,"scenarios":reports});
    let report_path = options.report.unwrap_or_else(|| run_dir.join("report.json"));
    util::write_private(&report_path, &serde_json::to_vec_pretty(&data)?)?;
    println!(
        "{passed}/{} passed; JSON report: {}",
        reports.len(),
        report_path.display()
    );
    Ok(passed == reports.len())
}
