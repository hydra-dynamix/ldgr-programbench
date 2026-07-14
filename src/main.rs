use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SOURCE_MANIFEST: &str = include_str!("../docs/historical/source-manifest.json");
const CLASSIFICATIONS: &str = include_str!("../docs/historical/classifications.json");
const LEAKAGE_RULE: &str = include_str!("../docs/historical/leakage-rule.md");
const CONTRACT: &str = include_str!("../docs/historical/reproduction-contract.md");
const CLEANROOM_BOUNDARY: &str = include_str!("../docs/historical/cleanroom-boundary.md");
const ADAPTER: &str = include_str!("../adapter.toml");
const ADAPTER_DATABASE_CONTRACT: &str = include_str!("../adapter-database-contract.json");
const RESOURCES: &str = include_str!("../adapter-resources.json");
const SKILL: &str = include_str!("../skills/programbench-reproduction/SKILL.md");
const EXTENSION: &str = include_str!("../extensions/ldgr-programbench.ts");
const HARNESS_COMMAND: &str = include_str!("../commands/ldgr-programbench.md");

#[derive(Parser)]
#[command(
    name = "ldgr-programbench",
    about = "Bounded historical ProgramBench reproduction; not an official or clean-room benchmark"
)]
struct Cli {
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    Adapter {
        #[command(subcommand)]
        command: AdapterAction,
    },
    Verify {
        #[arg(long)]
        archive_root: PathBuf,
    },
    Reproduce {
        #[arg(long)]
        archive_root: PathBuf,
        #[arg(long)]
        benchmarks_root: PathBuf,
        #[arg(long)]
        output_root: PathBuf,
        #[arg(long)]
        runner: Option<String>,
    },
    Report {
        #[arg(long)]
        results: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum AdapterAction {
    Install {
        #[arg(long)]
        install_root: PathBuf,
        #[arg(long)]
        print_path: bool,
    },
}

#[derive(Deserialize)]
struct SourceManifest {
    artifacts: Vec<Artifact>,
}
#[derive(Deserialize)]
struct Artifact {
    path: String,
    sha256: String,
}
#[derive(Deserialize, Serialize)]
struct Classifications {
    runs: Vec<RunClass>,
    counts: Counts,
}
#[derive(Deserialize, Serialize)]
struct RunClass {
    instance: String,
    status: String,
    basis: Vec<String>,
}
#[derive(Deserialize, Serialize)]
struct Counts {
    valid_non_cleanroom: usize,
    invalid_source_leakage: usize,
    unresolved: usize,
}
#[derive(Serialize)]
struct ReproductionResult {
    instance: String,
    status: String,
    command: String,
    exit_code: Option<i32>,
    historical_eval_sha256: String,
    current_eval_sha256: Option<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match Cli::parse().command {
        Action::Adapter {
            command:
                AdapterAction::Install {
                    install_root,
                    print_path,
                },
        } => install(&install_root, print_path),
        Action::Verify { archive_root } => verify(&archive_root),
        Action::Reproduce {
            archive_root,
            benchmarks_root,
            output_root,
            runner,
        } => reproduce(
            &archive_root,
            &benchmarks_root,
            &output_root,
            runner.as_deref(),
        ),
        Action::Report { results, json } => report(results.as_deref(), json),
    }
}

fn write(path: &Path, body: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

fn install(root: &Path, print_path: bool) -> Result<(), String> {
    for (path, body) in [
        ("adapter.toml", ADAPTER),
        ("adapter-database-contract.json", ADAPTER_DATABASE_CONTRACT),
        ("adapter-resources.json", RESOURCES),
        ("skills/programbench-reproduction/SKILL.md", SKILL),
        ("extensions/ldgr-programbench.ts", EXTENSION),
        ("commands/ldgr-programbench.md", HARNESS_COMMAND),
        ("docs/historical/source-manifest.json", SOURCE_MANIFEST),
        ("docs/historical/classifications.json", CLASSIFICATIONS),
        ("docs/historical/leakage-rule.md", LEAKAGE_RULE),
        ("docs/historical/reproduction-contract.md", CONTRACT),
        ("docs/historical/cleanroom-boundary.md", CLEANROOM_BOUNDARY),
    ] {
        write(&root.join(path), body.as_bytes())?;
    }
    let manifest = root.join("adapter.toml");
    if print_path {
        println!("{}", manifest.display());
    } else {
        println!(
            "installed LDGR adapter `programbench`: {}",
            manifest.display()
        );
    }
    Ok(())
}

fn digest(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn verify(root: &Path) -> Result<(), String> {
    let manifest: SourceManifest =
        serde_json::from_str(SOURCE_MANIFEST).map_err(|e| e.to_string())?;
    let mut failed = Vec::new();
    for artifact in manifest.artifacts {
        let path = root.join(&artifact.path);
        match digest(&path) {
            Ok(actual) if actual == artifact.sha256 => {}
            Ok(actual) => failed.push(format!("{} digest {actual}", artifact.path)),
            Err(e) => failed.push(e),
        }
    }
    if failed.is_empty() {
        println!("custody=verified artifacts=12");
        Ok(())
    } else {
        Err(format!(
            "custody verification failed:\n{}",
            failed.join("\n")
        ))
    }
}

fn valid_runs() -> Result<Vec<RunClass>, String> {
    let classes: Classifications =
        serde_json::from_str(CLASSIFICATIONS).map_err(|e| e.to_string())?;
    Ok(classes
        .runs
        .into_iter()
        .filter(|r| r.status == "valid_non_cleanroom")
        .collect())
}

fn source_instance_dir(root: &Path, instance: &str) -> PathBuf {
    if instance == "google__brotli.b3dc9cc" {
        root.join("../20260613-clean-team-update/runs-main")
            .join(instance)
    } else {
        root.join("runs-main").join(instance)
    }
}

fn reproduce(
    archive: &Path,
    benchmarks: &Path,
    out: &Path,
    runner: Option<&str>,
) -> Result<(), String> {
    verify(archive)?;
    fs::create_dir_all(out.join("runs")).map_err(|e| e.to_string())?;
    let environment = serde_json::json!({"schema_version":1,"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"archive_root":archive,"benchmarks_root":benchmarks,"limitations":["on_host","validator_visible","not_cleanroom","not_official_submission"]});
    write(
        &out.join("environment.json"),
        serde_json::to_vec_pretty(&environment).unwrap().as_slice(),
    )?;
    let ldgr_db = out.join("ldgr/ldgr.db");
    let ldgr_artifacts = out.join("ldgr/artifacts");
    ldgr_command(&ldgr_db, &ldgr_artifacts, &["init"])?;
    let mut results = Vec::new();
    for run in valid_runs()? {
        let source = source_instance_dir(archive, &run.instance);
        let target = out.join("runs").join(&run.instance);
        fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        fs::copy(
            source.join("submission.tar.gz"),
            target.join("submission.tar.gz"),
        )
        .map_err(|e| e.to_string())?;
        let command_text = runner.map(str::to_owned).unwrap_or_else(|| {
            format!(
                "uv run programbench eval {} --filter ^{}$ --force",
                out.join("runs").display(),
                run.instance
            )
        });
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(&command_text)
            .current_dir(benchmarks)
            .env("LDGR_PROGRAMBENCH_INSTANCE", &run.instance)
            .env("LDGR_PROGRAMBENCH_OUTPUT", &target);
        let output = command
            .output()
            .map_err(|e| format!("execute runner: {e}"))?;
        write(&target.join("stdout.log"), &output.stdout)?;
        write(&target.join("stderr.log"), &output.stderr)?;
        let old_eval = source.join(format!("{}.eval.json", run.instance));
        let current_eval = target.join(format!("{}.eval.json", run.instance));
        results.push(ReproductionResult {
            instance: run.instance.clone(),
            status: if output.status.success() {
                "completed"
            } else {
                "failed"
            }
            .into(),
            command: command_text.clone(),
            exit_code: output.status.code(),
            historical_eval_sha256: digest(&old_eval)?,
            current_eval_sha256: current_eval
                .is_file()
                .then(|| digest(&current_eval))
                .transpose()?,
        });
        record_ldgr_run(
            &ldgr_db,
            &ldgr_artifacts,
            &run.instance,
            &command_text,
            output.status.success(),
            &target,
        )?;
    }
    write(
        &out.join("results.json"),
        serde_json::to_vec_pretty(&results).unwrap().as_slice(),
    )?;
    write(&out.join("limitations.md"), b"# Limitations\n\nThis reproduction is on-host and validator-visible. It is not an official score, submission, clean-room run, or independent benchmark.\n")?;
    println!(
        "reproduction_runs=4 results={}",
        out.join("results.json").display()
    );
    if results.iter().all(|r| r.status == "completed") {
        Ok(())
    } else {
        Err("one or more reproduction commands failed; raw evidence was retained".into())
    }
}

fn ldgr_command(db: &Path, artifacts: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("ldgr")
        .arg("--db")
        .arg(db)
        .arg("--artifact-root")
        .arg(artifacts)
        .args(args)
        .output()
        .map_err(|e| format!("run ldgr: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "ldgr {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn record_ldgr_run(
    db: &Path,
    artifacts: &Path,
    instance: &str,
    command: &str,
    passed: bool,
    target: &Path,
) -> Result<(), String> {
    let slug = format!("reproduce-{}", instance.replace(['_', '.'], "-"));
    let title = format!("Reproduce {instance}");
    ldgr_command(db, artifacts, &["work", "create", &slug, "--title", &title, "--description", "On-host validator-visible historical reproduction; not an official or clean-room benchmark."])?;
    let started = ldgr_command(
        db,
        artifacts,
        &["run", "start", &slug, "--command", command],
    )?;
    let run_id = started
        .split_whitespace()
        .find_map(|value| value.parse::<i64>().ok())
        .ok_or_else(|| format!("cannot parse run id from {started}"))?
        .to_string();
    for (path, description) in [
        (
            target.join("submission.tar.gz"),
            "frozen historical submission",
        ),
        (target.join("stdout.log"), "raw reproduction stdout"),
        (target.join("stderr.log"), "raw reproduction stderr"),
    ] {
        ldgr_command(
            db,
            artifacts,
            &[
                "artifact",
                "add",
                &run_id,
                "--path",
                path.to_str().ok_or("non-UTF8 artifact path")?,
                "--description",
                description,
            ],
        )?;
    }
    ldgr_command(
        db,
        artifacts,
        &[
            "validation",
            "record",
            &run_id,
            "--outcome",
            if passed { "pass" } else { "fail" },
            "--rationale",
            "Runner exit status recorded without post-hoc repair.",
        ],
    )?;
    ldgr_command(db, artifacts, &["run", "finish", &run_id, "--status", if passed { "success" } else { "failed" }, "--notes", "Historical reproduction only; prohibited interpretations are retained in limitations.md."])?;
    ldgr_command(
        db,
        artifacts,
        &[
            "decision",
            "record",
            &slug,
            "--outcome",
            "stop",
            "--rationale",
            "One frozen reproduction attempt completed; automatic repair is forbidden.",
        ],
    )?;
    Ok(())
}

fn report(results: Option<&Path>, json: bool) -> Result<(), String> {
    let classes: Classifications =
        serde_json::from_str(CLASSIFICATIONS).map_err(|e| e.to_string())?;
    if json {
        println!("{}", serde_json::to_string_pretty(&classes).unwrap());
        return Ok(());
    }
    println!("# ProgramBench historical reproduction\n\nValid non-cleanroom runs: {}\nInvalid source-leakage runs: {}\nUnresolved runs: {}", classes.counts.valid_non_cleanroom, classes.counts.invalid_source_leakage, classes.counts.unresolved);
    for run in classes.runs {
        println!("- `{}`: {}", run.instance, run.status);
    }
    if let Some(path) = results {
        println!("\nCurrent reproduction evidence: `{}`", path.display());
    }
    println!("\nProhibited interpretations: official score; benchmark submission; clean-room or independent result; general model ranking.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classification_count_is_exactly_four() {
        assert_eq!(valid_runs().unwrap().len(), 4);
    }
    #[test]
    fn report_language_is_bounded() {
        assert!(CONTRACT.contains(
            "not a migration, an official benchmark submission, or a clean-room replication"
        ));
    }
    #[test]
    fn installer_materializes_every_typed_resource() {
        let t = tempfile::tempdir().unwrap();
        install(t.path(), false).unwrap();
        for p in [
            "adapter.toml",
            "adapter-resources.json",
            "skills/programbench-reproduction/SKILL.md",
            "extensions/ldgr-programbench.ts",
            "commands/ldgr-programbench.md",
        ] {
            assert!(t.path().join(p).is_file(), "{p}");
        }
    }
}
