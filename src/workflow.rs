use crate::telemetry::{
    ProgramBenchReproductionStep, ProgramBenchReproductionTelemetry,
    ProgramBenchReproductionTerminal,
};
use ldgr::harness_config::{parse_harness_config_json, parse_harness_config_toml, HarnessConfig};
use serde::Serialize;
use serde_json::Value;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_INSTANCE: &str = "sharkdp__hyperfine.327d5f4";
const IMAGE_TAG: &str = "task_cleanroom_v6";
const SETUP_DIRECTORY: &str = ".programbench";
const RUNS_DIRECTORY: &str = "programbench-runs";

#[derive(Clone, Debug)]
pub struct SetupOptions {
    pub benchmark_root: PathBuf,
    pub instance: String,
    pub harness: Option<String>,
    pub dry_run: bool,
}

#[derive(Clone, Debug)]
pub struct ReproduceOptions {
    pub benchmark_root: PathBuf,
    pub instance: String,
    pub harness: Option<String>,
    pub output_root: Option<PathBuf>,
    pub skip_evaluation: bool,
    pub dry_run: bool,
}

#[derive(Clone, Debug, Serialize)]
struct SetupPlan {
    schema_version: u32,
    benchmark_root: PathBuf,
    setup_root: PathBuf,
    results_root: PathBuf,
    instance: String,
    image: String,
    harness: String,
    agentctl_profile: String,
    execution_boundary: &'static str,
}

#[derive(Debug, Serialize)]
struct AttemptEnvironment<'a> {
    schema_version: u32,
    instance: &'a str,
    image: &'a str,
    harness: &'a str,
    agentctl_profile: &'a str,
    benchmark_root: &'a Path,
    run_root: &'a Path,
    workspace: &'a Path,
    limitations: [&'static str; 4],
}

#[derive(Clone, Debug, Serialize)]
struct EvaluationSummary {
    instance: String,
    artifact: PathBuf,
    tests: usize,
    passed: usize,
    failed: usize,
    other: usize,
    error_code: Option<String>,
    valid_artifact: bool,
}

pub fn setup(options: SetupOptions) -> Result<(), String> {
    let home = ldgr_home()?;
    let plan = prepare_setup(&options, &home, !options.dry_run)?;
    print_setup(&plan, options.dry_run);
    Ok(())
}

pub fn reproduce(options: ReproduceOptions) -> Result<(), String> {
    let mut telemetry = ProgramBenchReproductionTelemetry::begin_running();
    let result = reproduce_inner(&options, &mut telemetry);
    telemetry.finish(match &result {
        Ok(Some(summaries)) if summaries.iter().any(|summary| !summary.valid_artifact) => {
            ProgramBenchReproductionTerminal::CompletedNegative
        }
        Ok(Some(_)) => ProgramBenchReproductionTerminal::CompletedPositive,
        Ok(None) => ProgramBenchReproductionTerminal::CompletedInconclusive,
        Err(_) => ProgramBenchReproductionTerminal::OperationalFailure,
    });
    result.map(|_| ())
}

fn reproduce_inner(
    options: &ReproduceOptions,
    telemetry: &mut ProgramBenchReproductionTelemetry,
) -> Result<Option<Vec<EvaluationSummary>>, String> {
    let home = ldgr_home()?;
    let setup_options = SetupOptions {
        benchmark_root: options.benchmark_root.clone(),
        instance: options.instance.clone(),
        harness: options.harness.clone(),
        dry_run: options.dry_run,
    };
    let plan = prepare_setup(&setup_options, &home, !options.dry_run)?;
    telemetry.record_step(ProgramBenchReproductionStep::InputsPrepared);

    let timestamp = unix_timestamp()?;
    let run_root = options
        .output_root
        .as_deref()
        .map(|path| resolve_under(&plan.benchmark_root, path))
        .unwrap_or_else(|| plan.results_root.join(timestamp.to_string()));
    let instance_root = run_root.join(&plan.instance);
    let workspace = instance_root.join("workspace");
    let prompt_path = instance_root.join("PROMPT.md");
    let agent_stdout = instance_root.join("agent.stdout.log");
    let agent_stderr = instance_root.join("agent.stderr.log");
    let submission = instance_root.join("submission.tar.gz");
    let eval_artifact = instance_root.join(format!("{}.eval.json", plan.instance));
    let container_name = format!(
        "ldgr-programbench-{}-{}-{}",
        std::process::id(),
        timestamp,
        safe_slug(&plan.instance)
    );

    fs::create_dir_all(&workspace)
        .map_err(|error| format!("create {}: {error}", workspace.display()))?;
    write_json(
        &run_root.join("environment.json"),
        &AttemptEnvironment {
            schema_version: 1,
            instance: &plan.instance,
            image: &plan.image,
            harness: &plan.harness,
            agentctl_profile: &plan.agentctl_profile,
            benchmark_root: &plan.benchmark_root,
            run_root: &run_root,
            workspace: &workspace,
            limitations: [
                "on_host",
                "validator_visible",
                "not_cleanroom",
                "not_official_submission",
            ],
        },
    )?;
    write_text(
        &prompt_path,
        &attempt_prompt(&plan.instance, &workspace, &run_root),
    )?;

    println!("ProgramBench attempt");
    println!("  instance: {}", plan.instance);
    println!("  harness: {}", plan.harness);
    println!("  agentctl profile: {}", plan.agentctl_profile);
    println!("  workspace: {}", workspace.display());
    println!("  results: {}", run_root.display());

    if options.dry_run {
        println!("  mode: dry-run (no dependencies, containers, or harnesses executed)");
        println!("  reference: docker run {}", plan.image);
        println!(
            "  harness command: agentctl run {} --cwd {} --prompt-file {}",
            plan.agentctl_profile,
            workspace.display(),
            prompt_path.display()
        );
        if !options.skip_evaluation {
            println!(
                "  evaluation command: uvx programbench eval {} --filter '^{}$' --force",
                run_root.display(),
                regex_literal(&plan.instance)
            );
        }
        return Ok(None);
    }

    telemetry.record_step(ProgramBenchReproductionStep::ReproductionPrepared);
    initialize_ldgr(&workspace)?;
    let ledger = start_ldgr_attempt(&workspace, &plan, timestamp)?;

    start_reference_container(&plan, &workspace, &container_name)?;
    let attempt_result = (|| {
        extract_reference_material(&container_name, &workspace)?;
        write_reference_wrapper(&workspace.join("executable"), &container_name)?;
        let output = Command::new("agentctl")
            .arg("run")
            .arg(&plan.agentctl_profile)
            .arg("--cwd")
            .arg(&workspace)
            .arg("--prompt-file")
            .arg(&prompt_path)
            .output()
            .map_err(|error| format!("launch configured harness through agentctl: {error}"))?;
        write_bytes(&agent_stdout, &output.stdout)?;
        write_bytes(&agent_stderr, &output.stderr)?;
        package_submission(&workspace, &submission)?;
        Ok::<Output, String>(output)
    })();
    let cleanup = Command::new("docker")
        .args(["rm", "-f", &container_name])
        .output();
    if let Ok(output) = cleanup {
        if !output.status.success() {
            eprintln!(
                "warning: failed to remove reference container {}: {}",
                container_name,
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
    }
    let agent_output = attempt_result?;
    telemetry.record_step(ProgramBenchReproductionStep::AttemptsRecorded);

    println!(
        "  harness exit: {}",
        agent_output
            .status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "signal".to_string())
    );
    println!("  submission: {}", submission.display());

    let summaries = if options.skip_evaluation {
        println!("  evaluation: skipped by --skip-evaluation");
        None
    } else {
        let eval = Command::new("uvx")
            .arg("programbench")
            .arg("eval")
            .arg(&run_root)
            .arg("--filter")
            .arg(format!("^{}$", regex_literal(&plan.instance)))
            .arg("--force")
            .output()
            .map_err(|error| format!("launch ProgramBench evaluator with uvx: {error}"))?;
        write_bytes(&instance_root.join("evaluation.stdout.log"), &eval.stdout)?;
        write_bytes(&instance_root.join("evaluation.stderr.log"), &eval.stderr)?;
        if !eval.status.success() && !eval_artifact.is_file() {
            finish_ldgr_attempt(
                &workspace,
                &ledger,
                &prompt_path,
                &agent_stdout,
                &agent_stderr,
                &submission,
                None,
                false,
            )?;
            return Err(format!(
                "ProgramBench evaluation failed before producing {}: {}",
                eval_artifact.display(),
                String::from_utf8_lossy(&eval.stderr).trim()
            ));
        }
        let summaries = summarize_results(&run_root)?;
        print_summaries(&run_root, &summaries);
        Some(summaries)
    };

    finish_ldgr_attempt(
        &workspace,
        &ledger,
        &prompt_path,
        &agent_stdout,
        &agent_stderr,
        &submission,
        eval_artifact.is_file().then_some(eval_artifact.as_path()),
        true,
    )?;
    telemetry.record_step(ProgramBenchReproductionStep::EvidenceFinalized);
    println!(
        "  LDGR ledger: {}",
        workspace.join(".ldgr/ldgr.db").display()
    );
    println!("Attempt complete. Verify again with:");
    println!(
        "  ldgr programbench verify --results {}",
        run_root.display()
    );
    Ok(summaries)
}

pub fn verify(benchmark_root: &Path, results: Option<&Path>, json: bool) -> Result<(), String> {
    let root = resolve_results_root(benchmark_root, results)?;
    let summaries = summarize_results(&root)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&summaries).map_err(|error| error.to_string())?
        );
    } else {
        print_summaries(&root, &summaries);
        println!("verification=complete artifacts={}", summaries.len());
    }
    Ok(())
}

pub fn report(benchmark_root: &Path, results: Option<&Path>, json: bool) -> Result<(), String> {
    let root = resolve_results_root(benchmark_root, results)?;
    let summaries = summarize_results(&root)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "results_root": root,
                "summaries": summaries,
                "limitations": ["on_host", "validator_visible", "not_cleanroom", "not_official_submission"]
            }))
            .map_err(|error| error.to_string())?
        );
        return Ok(());
    }
    println!("# ProgramBench attempt report\n");
    println!("Results: `{}`\n", root.display());
    for summary in summaries {
        println!(
            "- `{}`: {}/{} tests passed{}",
            summary.instance,
            summary.passed,
            summary.tests,
            summary
                .error_code
                .as_deref()
                .map(|code| format!("; evaluator error `{code}`"))
                .unwrap_or_default()
        );
    }
    println!("\nThis was an on-host, validator-visible LDGR demonstration. It is not an official submission, clean-room result, independent score, or general model ranking.");
    Ok(())
}

fn prepare_setup(
    options: &SetupOptions,
    ldgr_home: &Path,
    perform_preflight: bool,
) -> Result<SetupPlan, String> {
    validate_instance(&options.instance)?;
    fs::create_dir_all(&options.benchmark_root).map_err(|error| {
        format!(
            "create benchmark root {}: {error}",
            options.benchmark_root.display()
        )
    })?;
    let benchmark_root = fs::canonicalize(&options.benchmark_root).map_err(|error| {
        format!(
            "resolve benchmark root {}: {error}",
            options.benchmark_root.display()
        )
    })?;
    let config = load_harness_config(ldgr_home)?;
    let harness = resolve_harness(&config, options.harness.as_deref())?;
    let plan = SetupPlan {
        schema_version: 1,
        setup_root: benchmark_root.join(SETUP_DIRECTORY),
        results_root: benchmark_root.join(RUNS_DIRECTORY),
        image: image_for_instance(&options.instance),
        agentctl_profile: format!("ldgr-loop-{harness}"),
        benchmark_root,
        instance: options.instance.clone(),
        harness,
        execution_boundary: "on-host, validator-visible, not an official or clean-room submission",
    };
    fs::create_dir_all(&plan.setup_root)
        .map_err(|error| format!("create {}: {error}", plan.setup_root.display()))?;
    fs::create_dir_all(&plan.results_root)
        .map_err(|error| format!("create {}: {error}", plan.results_root.display()))?;
    write_json(&plan.setup_root.join("setup.json"), &plan)?;
    write_text(&plan.setup_root.join("README.md"), &setup_readme(&plan))?;
    if perform_preflight {
        preflight(&plan)?;
    }
    Ok(plan)
}

fn preflight(plan: &SetupPlan) -> Result<(), String> {
    if std::env::consts::OS != "linux" || std::env::consts::ARCH != "x86_64" {
        return Err(format!(
            "ProgramBench task images require a Linux x86_64 host; detected {} {}. Use `setup --dry-run` to inspect the plan without execution",
            std::env::consts::OS,
            std::env::consts::ARCH
        ));
    }
    run_checked(
        Command::new("docker").arg("version"),
        "Docker is unavailable",
    )?;
    run_checked(
        Command::new("agentctl").args(["config", "check"]),
        "agentctl configuration is unavailable",
    )?;
    run_checked(
        Command::new("uvx").args(["programbench", "--help"]),
        "ProgramBench could not be installed/resolved through uvx",
    )?;
    run_checked(
        Command::new("docker").args(["pull", "--platform", "linux/amd64", &plan.image]),
        "ProgramBench task image could not be pulled",
    )?;
    Ok(())
}

fn print_setup(plan: &SetupPlan, dry_run: bool) {
    println!(
        "ProgramBench setup {}",
        if dry_run { "plan" } else { "complete" }
    );
    println!("  benchmark root: {}", plan.benchmark_root.display());
    println!("  instance: {}", plan.instance);
    println!("  task image: {}", plan.image);
    println!("  harness: {}", plan.harness);
    println!("  agentctl profile: {}", plan.agentctl_profile);
    println!("  runs: {}", plan.results_root.display());
    println!("  boundary: {}", plan.execution_boundary);
    if dry_run {
        println!("  downloads/execution: skipped");
    }
}

fn load_harness_config(ldgr_home: &Path) -> Result<HarnessConfig, String> {
    let toml_path = ldgr_home.join("config.toml");
    if toml_path.is_file() {
        let text = fs::read_to_string(&toml_path)
            .map_err(|error| format!("read {}: {error}", toml_path.display()))?;
        return parse_harness_config_toml(&text)
            .map_err(|error| format!("parse {}: {error:#}", toml_path.display()));
    }
    let json_path = ldgr_home.join("config.json");
    if json_path.is_file() {
        let text = fs::read_to_string(&json_path)
            .map_err(|error| format!("read {}: {error}", json_path.display()))?;
        return parse_harness_config_json(&text)
            .map_err(|error| format!("parse {}: {error:#}", json_path.display()));
    }
    Err(format!(
        "LDGR harness config is missing under {}; run `ldgr install` first",
        ldgr_home.display()
    ))
}

fn resolve_harness(config: &HarnessConfig, requested: Option<&str>) -> Result<String, String> {
    let selected = requested
        .map(str::to_owned)
        .or_else(|| config.default_harness.clone())
        .or_else(|| {
            (config.selected_harnesses.len() == 1).then(|| config.selected_harnesses[0].clone())
        })
        .ok_or_else(|| {
            "no default harness is configured; run `ldgr install` or pass --harness <name>"
                .to_string()
        })?;
    if !config
        .selected_harnesses
        .iter()
        .any(|harness| harness == &selected)
    {
        return Err(format!(
            "harness `{selected}` is not selected in the LDGR user config (selected: {})",
            config.selected_harnesses.join(", ")
        ));
    }
    Ok(selected)
}

fn ldgr_home() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("LDGR_HOME") {
        return Ok(PathBuf::from(path));
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".ldgr"))
        .or_else(|| {
            env::var_os("USERPROFILE")
                .map(PathBuf::from)
                .map(|home| home.join(".ldgr"))
        })
        .ok_or_else(|| "cannot locate the LDGR user config: HOME and USERPROFILE are unset".into())
}

fn start_reference_container(
    plan: &SetupPlan,
    workspace: &Path,
    container_name: &str,
) -> Result<(), String> {
    let mount = format!("{}:/candidate", workspace.display());
    run_checked(
        Command::new("docker").args([
            "run",
            "--detach",
            "--rm",
            "--platform",
            "linux/amd64",
            "--network",
            "none",
            "--name",
            container_name,
            "--volume",
            &mount,
            "--entrypoint",
            "sleep",
            &plan.image,
            "infinity",
        ]),
        "failed to start the ProgramBench reference container",
    )?;
    Ok(())
}

fn extract_reference_material(container: &str, workspace: &Path) -> Result<(), String> {
    let archive = Command::new("docker")
        .args([
            "exec",
            container,
            "tar",
            "--exclude=./executable",
            "-C",
            "/workspace",
            "-czf",
            "-",
            ".",
        ])
        .output()
        .map_err(|error| format!("extract ProgramBench task material: {error}"))?;
    if !archive.status.success() {
        return Err(format!(
            "extract ProgramBench task material: {}",
            String::from_utf8_lossy(&archive.stderr).trim()
        ));
    }
    let mut child = Command::new("tar")
        .args(["-xzf", "-", "-C"])
        .arg(workspace)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| format!("start local tar extraction: {error}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "local tar stdin was not available".to_string())?
        .write_all(&archive.stdout)
        .map_err(|error| format!("stream task material to local tar: {error}"))?;
    let status = child
        .wait()
        .map_err(|error| format!("wait for local tar extraction: {error}"))?;
    if !status.success() {
        return Err(format!("local tar extraction failed with {status}"));
    }
    Ok(())
}

fn write_reference_wrapper(path: &Path, container: &str) -> Result<(), String> {
    write_text(
        path,
        &format!(
            "#!/usr/bin/env bash\nexec docker exec -i --workdir /candidate {container} /workspace/executable \"$@\"\n"
        ),
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .map_err(|error| format!("make {} executable: {error}", path.display()))?;
    }
    Ok(())
}

fn package_submission(workspace: &Path, submission: &Path) -> Result<(), String> {
    run_checked(
        Command::new("tar")
            .args([
                "--exclude=./executable",
                "--exclude=./.ldgr",
                "--exclude=./.git",
                "-czf",
            ])
            .arg(submission)
            .arg("-C")
            .arg(workspace)
            .arg("."),
        "failed to package the ProgramBench candidate",
    )?;
    Ok(())
}

fn initialize_ldgr(workspace: &Path) -> Result<(), String> {
    let db = workspace.join(".ldgr/ldgr.db");
    let artifacts = workspace.join(".ldgr/artifacts");
    ldgr_command(workspace, &db, &artifacts, &["init"])?;
    Ok(())
}

struct LedgerRun {
    db: PathBuf,
    artifacts: PathBuf,
    run_id: String,
}

fn start_ldgr_attempt(
    workspace: &Path,
    plan: &SetupPlan,
    timestamp: u64,
) -> Result<LedgerRun, String> {
    let db = workspace.join(".ldgr/ldgr.db");
    let artifacts = workspace.join(".ldgr/artifacts");
    let slug = format!("programbench-{}-{timestamp}", safe_slug(&plan.instance));
    ldgr_command(
        workspace,
        &db,
        &artifacts,
        &[
            "work",
            "create",
            &slug,
            "--title",
            &format!("Attempt ProgramBench {}", plan.instance),
            "--description",
            "Use the configured LDGR harness to produce and evaluate one on-host ProgramBench candidate.",
        ],
    )?;
    let started = ldgr_command(
        workspace,
        &db,
        &artifacts,
        &[
            "run",
            "start",
            &slug,
            "--command",
            &format!("agentctl run {}", plan.agentctl_profile),
        ],
    )?;
    let run_id = started
        .split_whitespace()
        .find_map(|value| value.parse::<i64>().ok())
        .ok_or_else(|| format!("cannot parse LDGR run id from: {started}"))?
        .to_string();
    Ok(LedgerRun {
        db,
        artifacts,
        run_id,
    })
}

#[allow(clippy::too_many_arguments)]
fn finish_ldgr_attempt(
    workspace: &Path,
    ledger: &LedgerRun,
    prompt: &Path,
    stdout: &Path,
    stderr: &Path,
    submission: &Path,
    evaluation: Option<&Path>,
    completed: bool,
) -> Result<(), String> {
    for (path, kind, description) in [
        (prompt, "document", "Exact ProgramBench harness prompt."),
        (stdout, "log", "Configured harness standard output."),
        (stderr, "log", "Configured harness standard error."),
        (
            submission,
            "archive",
            "Packaged ProgramBench candidate source.",
        ),
    ] {
        if path.is_file() {
            ldgr_command(
                workspace,
                &ledger.db,
                &ledger.artifacts,
                &[
                    "artifact",
                    "add",
                    &ledger.run_id,
                    "--kind",
                    kind,
                    "--path",
                    path.to_str().ok_or("non-UTF8 artifact path")?,
                    "--description",
                    description,
                ],
            )?;
        }
    }
    if let Some(evaluation) = evaluation {
        ldgr_command(
            workspace,
            &ledger.db,
            &ledger.artifacts,
            &[
                "artifact",
                "add",
                &ledger.run_id,
                "--kind",
                "json",
                "--path",
                evaluation.to_str().ok_or("non-UTF8 evaluation path")?,
                "--description",
                "ProgramBench evaluator output.",
            ],
        )?;
    }
    ldgr_command(
        workspace,
        &ledger.db,
        &ledger.artifacts,
        &[
            "validation",
            "record",
            &ledger.run_id,
            "--outcome",
            if completed { "pass" } else { "error" },
            "--rationale",
            if completed {
                "The configured harness attempt was packaged and its available evaluation evidence was retained."
            } else {
                "The evaluator stopped before producing a readable result artifact."
            },
        ],
    )?;
    ldgr_command(
        workspace,
        &ledger.db,
        &ledger.artifacts,
        &[
            "run",
            "close",
            &ledger.run_id,
            "--status",
            if completed { "success" } else { "failed" },
            "--outcome",
            "stop",
            "--rationale",
            "The bounded ProgramBench demonstration attempt is complete; inspect verification evidence before interpreting the result.",
        ],
    )?;
    Ok(())
}

fn ldgr_command(
    current_dir: &Path,
    db: &Path,
    artifacts: &Path,
    args: &[&str],
) -> Result<String, String> {
    let output = Command::new("ldgr")
        .current_dir(current_dir)
        .arg("--db")
        .arg(db)
        .arg("--artifact-root")
        .arg(artifacts)
        .args(args)
        .output()
        .map_err(|error| format!("run ldgr {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!(
            "ldgr {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn summarize_results(root: &Path) -> Result<Vec<EvaluationSummary>, String> {
    if !root.is_dir() {
        return Err(format!(
            "results directory does not exist: {}",
            root.display()
        ));
    }
    let mut artifacts = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| format!("read {}: {error}", root.display()))? {
        let instance_dir = entry.map_err(|error| error.to_string())?.path();
        if !instance_dir.is_dir() {
            continue;
        }
        for candidate in fs::read_dir(&instance_dir)
            .map_err(|error| format!("read {}: {error}", instance_dir.display()))?
        {
            let path = candidate.map_err(|error| error.to_string())?.path();
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".eval.json"))
            {
                artifacts.push(path);
            }
        }
    }
    artifacts.sort();
    if artifacts.is_empty() {
        return Err(format!(
            "no <instance>/<instance>.eval.json files found under {}; run `ldgr programbench reproduce` first or pass --results",
            root.display()
        ));
    }
    artifacts
        .into_iter()
        .map(|artifact| summarize_artifact(&artifact))
        .collect()
}

fn summarize_artifact(path: &Path) -> Result<EvaluationSummary, String> {
    let value: Value = serde_json::from_slice(
        &fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?,
    )
    .map_err(|error| format!("parse {}: {error}", path.display()))?;
    let tests = value
        .get("test_results")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} has no test_results array", path.display()))?;
    let passed = tests
        .iter()
        .filter(|test| test.get("status").and_then(Value::as_str) == Some("passed"))
        .count();
    let failed = tests
        .iter()
        .filter(|test| {
            matches!(
                test.get("status").and_then(Value::as_str),
                Some("failure" | "failed")
            )
        })
        .count();
    let error_code = value
        .get("error_code")
        .and_then(Value::as_str)
        .map(str::to_owned);
    Ok(EvaluationSummary {
        instance: path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string(),
        artifact: path.to_path_buf(),
        tests: tests.len(),
        passed,
        failed,
        other: tests.len().saturating_sub(passed + failed),
        valid_artifact: error_code.is_none(),
        error_code,
    })
}

fn print_summaries(root: &Path, summaries: &[EvaluationSummary]) {
    println!("ProgramBench verification");
    println!("  results: {}", root.display());
    for summary in summaries {
        println!("  instance: {}", summary.instance);
        println!(
            "    tests: {} passed={} failed={} other={}",
            summary.tests, summary.passed, summary.failed, summary.other
        );
        println!(
            "    evaluator: {}",
            summary.error_code.as_deref().unwrap_or("complete")
        );
        println!("    artifact: {}", summary.artifact.display());
    }
}

fn resolve_results_root(benchmark_root: &Path, results: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(results) = results {
        return Ok(resolve_under(benchmark_root, results));
    }
    let runs = resolve_under(benchmark_root, Path::new(RUNS_DIRECTORY));
    let mut candidates = fs::read_dir(&runs)
        .map_err(|error| {
            format!(
                "no local ProgramBench runs found under {}: {error}",
                runs.display()
            )
        })?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let modified = entry.metadata().ok()?.modified().ok()?;
            Some((modified, entry.path()))
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(modified, _)| *modified);
    candidates
        .pop()
        .map(|(_, path)| path)
        .ok_or_else(|| format!("no local ProgramBench runs found under {}", runs.display()))
}

fn setup_readme(plan: &SetupPlan) -> String {
    format!(
        "# LDGR ProgramBench workspace\n\nConfigured instance: `{}`\n\nConfigured harness: `{}` via agentctl profile `{}`.\n\nRun from the benchmark root:\n\n```sh\nldgr programbench reproduce\nldgr programbench verify\nldgr programbench report\n```\n\nThe adapter prepares a reference task container, invokes the user-configured harness on host, packages the candidate, evaluates it with ProgramBench, and records an LDGR ledger. It is not an official or clean-room submission.\n",
        plan.instance, plan.harness, plan.agentctl_profile
    )
}

fn attempt_prompt(instance: &str, workspace: &Path, run_root: &Path) -> String {
    format!(
        "You are the user's configured LDGR harness running one bounded, on-host ProgramBench attempt.\n\nInstance: {instance}\nWorkspace: {}\nEvidence root: {}\n\nThe workspace contains the task documentation and an executable-only black-box reference at `./executable`. Build a new, original implementation from observed behavior. Do not obtain upstream source, wrap the reference executable, decompile it, or use tracing tools against it.\n\nUse LDGR as the work ledger: begin with `ldgr status` and record useful observations as you work. Inspect the bundled documentation and exercise `./executable` through its normal interface. Create an executable `./compile.sh` that builds your candidate as `./executable`. Test the candidate you produce. Do not delete `.ldgr`; the adapter will retain it as run evidence and exclude it from the ProgramBench submission archive.\n\nThis is a validator-visible on-host demonstration, not an official submission or clean-room result. Finish when you have produced the strongest bounded attempt available.\n",
        workspace.display(),
        run_root.display()
    )
}

fn image_for_instance(instance: &str) -> String {
    format!(
        "programbench/{}:{IMAGE_TAG}",
        instance.replace("__", "_1776_")
    )
}

fn validate_instance(instance: &str) -> Result<(), String> {
    if instance.is_empty()
        || !instance
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_.-".contains(character))
        || !instance.contains("__")
    {
        return Err(format!(
            "invalid ProgramBench instance `{instance}`; expected an ID such as {DEFAULT_INSTANCE}"
        ));
    }
    Ok(())
}

fn resolve_under(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn safe_slug(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn regex_literal(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| {
            if ".+*?()|[]{}^$\\".contains(character) {
                vec!['\\', character]
            } else {
                vec![character]
            }
        })
        .collect()
}

fn unix_timestamp() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| format!("system clock is before Unix epoch: {error}"))
}

fn run_checked(command: &mut Command, context: &str) -> Result<Output, String> {
    let rendered = format!("{command:?}");
    let output = command
        .output()
        .map_err(|error| format!("{context}: {rendered}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{context}: {rendered}\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output)
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    write_bytes(
        path,
        &serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?,
    )
}

fn write_text(path: &Path, value: &str) -> Result<(), String> {
    write_bytes(path, value.as_bytes())
}

fn write_bytes(path: &Path, value: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    fs::write(path, value).map_err(|error| format!("write {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_default_harness_drives_agentctl_profile() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(
            temp.path().join("config.json"),
            r#"{
              "schema_version": 1,
              "default_harness": "pi",
              "selected_harnesses": ["pi", "codex"],
              "installed": []
            }"#,
        )
        .unwrap();
        let root = tempfile::tempdir().unwrap();
        let plan = prepare_setup(
            &SetupOptions {
                benchmark_root: root.path().to_path_buf(),
                instance: DEFAULT_INSTANCE.to_string(),
                harness: None,
                dry_run: true,
            },
            temp.path(),
            false,
        )
        .unwrap();
        assert_eq!(plan.harness, "pi");
        assert_eq!(plan.agentctl_profile, "ldgr-loop-pi");
        assert!(plan.setup_root.join("setup.json").is_file());
    }

    #[test]
    fn harness_override_must_be_selected_by_the_user() {
        let config = parse_harness_config_json(
            r#"{
              "schema_version": 1,
              "default_harness": "pi",
              "selected_harnesses": ["pi"],
              "installed": []
            }"#,
        )
        .unwrap();
        assert!(resolve_harness(&config, Some("codex"))
            .unwrap_err()
            .contains("not selected"));
    }

    #[test]
    fn evaluation_summary_counts_real_programbench_status_values() {
        let temp = tempfile::tempdir().unwrap();
        let instance = temp.path().join(DEFAULT_INSTANCE);
        fs::create_dir_all(&instance).unwrap();
        let artifact = instance.join(format!("{DEFAULT_INSTANCE}.eval.json"));
        fs::write(
            &artifact,
            r#"{
              "test_results": [
                {"status":"passed"},
                {"status":"failure"},
                {"status":"skipped"}
              ],
              "error_code": null
            }"#,
        )
        .unwrap();
        let summary = summarize_results(temp.path()).unwrap().remove(0);
        assert_eq!(
            (summary.tests, summary.passed, summary.failed, summary.other),
            (3, 1, 1, 1)
        );
        assert!(summary.valid_artifact);
    }

    #[test]
    fn image_name_matches_programbench_convention() {
        assert_eq!(
            image_for_instance(DEFAULT_INSTANCE),
            "programbench/sharkdp_1776_hyperfine.327d5f4:task_cleanroom_v6"
        );
    }

    #[test]
    fn instance_filter_is_a_literal_anchored_match() {
        assert_eq!(
            regex_literal(DEFAULT_INSTANCE),
            "sharkdp__hyperfine\\.327d5f4"
        );
    }
}
