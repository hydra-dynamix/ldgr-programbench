mod telemetry;
mod workflow;

use clap::{Args, Parser, Subcommand};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const SOURCE_MANIFEST: &str = include_str!("../docs/historical/source-manifest.json");
const CLASSIFICATIONS: &str = include_str!("../docs/historical/classifications.json");
const LEAKAGE_RULE: &str = include_str!("../docs/historical/leakage-rule.md");
const CONTRACT: &str = include_str!("../docs/historical/reproduction-contract.md");
const CLEANROOM_BOUNDARY: &str = include_str!("../docs/historical/cleanroom-boundary.md");
const RUN_GUIDE: &str = include_str!("../docs/programbench-run.md");
const ADAPTER: &str = include_str!("../adapter.toml");
const ADAPTER_COMPATIBILITY: &str = include_str!("../adapter-compatibility.json");
const RESOURCES: &str = include_str!("../adapter-resources.json");

const ROOT_HELP: &str = "Examples:\n  ldgr programbench setup\n  ldgr programbench reproduce\n  ldgr programbench reproduce --benchmark-root ~/repos/programbench --instance sharkdp__hyperfine.327d5f4\n  ldgr programbench verify\n  ldgr programbench report\n\n`reproduce` runs one bounded ProgramBench attempt by default. It prepares the benchmark workspace, invokes the default harness from ~/.ldgr/config.toml through agentctl, packages the candidate, runs ProgramBench evaluation, and records LDGR evidence. This is an on-host demonstration, not an official or clean-room leaderboard submission.";

#[derive(Parser, Debug)]
#[command(
    name = "ldgr-programbench",
    version,
    about = "Run and verify an on-host ProgramBench attempt with the user's LDGR harness",
    long_about = "Prepare a real ProgramBench task, invoke the user's configured LDGR harness through agentctl, evaluate the resulting candidate, and retain the run as LDGR evidence. Defaults are intentionally bounded to one task. The result is an on-host demonstration, not an official or clean-room leaderboard submission.",
    after_help = ROOT_HELP,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Prepare the local tools, task image, folders, and run plan.
    #[command(
        after_help = "Examples:\n  ldgr programbench setup\n  ldgr programbench setup --benchmark-root ~/repos/programbench\n  ldgr programbench setup --harness codex\n  ldgr programbench setup --dry-run"
    )]
    Setup(SetupArgs),

    /// Prepare, run, package, and evaluate one ProgramBench attempt.
    #[command(
        after_help = "Examples:\n  ldgr programbench reproduce\n  ldgr programbench reproduce --benchmark-root ~/repos/programbench\n  ldgr programbench reproduce --benchmarks-root ~/repos/programbench\n  ldgr programbench reproduce --instance sharkdp__hyperfine.327d5f4 --harness codex\n  ldgr programbench reproduce --dry-run\n\nThe singular --benchmark-root spelling is canonical; --benchmarks-root is an accepted compatibility alias. When --harness is omitted, the command reads default_harness from ~/.ldgr/config.toml or ~/.ldgr/config.json."
    )]
    Reproduce(ReproduceArgs),

    /// Validate and summarize evaluation artifacts from a completed attempt.
    #[command(
        after_help = "Examples:\n  ldgr programbench verify\n  ldgr programbench verify --results ./programbench-runs/1786000000\n  ldgr programbench verify --json\n\nWithout --results, the newest directory under <benchmark-root>/programbench-runs is used."
    )]
    Verify(ResultsArgs),

    /// Render a readable report for the newest or selected attempt.
    #[command(
        after_help = "Examples:\n  ldgr programbench report\n  ldgr programbench report --results ./programbench-runs/1786000000\n  ldgr programbench report --json"
    )]
    Report(ResultsArgs),

    /// Verify the frozen historical evidence archive (legacy custody operation).
    #[command(
        hide = true,
        after_help = "Example:\n  ldgr programbench custody --archive-root /path/to/20260613-archive\n\nThis command is intentionally separate from `reproduce`; a new attempt does not require a historical completed run."
    )]
    Custody {
        /// Root of the retained historical archive.
        #[arg(long, value_name = "DIR")]
        archive_root: PathBuf,
    },

    #[command(hide = true)]
    Adapter {
        #[command(subcommand)]
        command: AdapterAction,
    },
}

#[derive(Args, Clone, Debug)]
struct SetupArgs {
    /// Project directory that owns ProgramBench setup and run output.
    #[arg(
        long = "benchmark-root",
        visible_alias = "benchmarks-root",
        value_name = "DIR",
        default_value = "."
    )]
    benchmark_root: PathBuf,

    /// ProgramBench task instance to prepare.
    #[arg(long, value_name = "INSTANCE", default_value = workflow::DEFAULT_INSTANCE)]
    instance: String,

    /// Selected LDGR harness. Defaults to the user's configured default harness.
    #[arg(long, value_name = "HARNESS")]
    harness: Option<String>,

    /// Write and print the plan without downloading or executing dependencies.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Args, Clone, Debug)]
struct ReproduceArgs {
    /// Project directory that owns ProgramBench setup and run output.
    #[arg(
        long = "benchmark-root",
        visible_alias = "benchmarks-root",
        value_name = "DIR",
        default_value = "."
    )]
    benchmark_root: PathBuf,

    /// ProgramBench task instance to attempt.
    #[arg(long, value_name = "INSTANCE", default_value = workflow::DEFAULT_INSTANCE)]
    instance: String,

    /// Selected LDGR harness. Defaults to the user's configured default harness.
    #[arg(long, value_name = "HARNESS")]
    harness: Option<String>,

    /// Run directory. Defaults to <benchmark-root>/programbench-runs/<timestamp>.
    #[arg(long, value_name = "DIR")]
    output_root: Option<PathBuf>,

    /// Package the harness attempt but skip ProgramBench evaluation.
    #[arg(long)]
    skip_evaluation: bool,

    /// Prepare and print the complete execution plan without launching the task or harness.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Args, Clone, Debug)]
struct ResultsArgs {
    /// Project directory containing the programbench-runs folder.
    #[arg(
        long = "benchmark-root",
        visible_alias = "benchmarks-root",
        value_name = "DIR",
        default_value = "."
    )]
    benchmark_root: PathBuf,

    /// Specific run directory. Defaults to the newest local run.
    #[arg(long, value_name = "DIR")]
    results: Option<PathBuf>,

    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand, Debug)]
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

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match Cli::parse().command {
        Action::Setup(args) => workflow::setup(workflow::SetupOptions {
            benchmark_root: args.benchmark_root,
            instance: args.instance,
            harness: args.harness,
            dry_run: args.dry_run,
        }),
        Action::Reproduce(args) => workflow::reproduce(workflow::ReproduceOptions {
            benchmark_root: args.benchmark_root,
            instance: args.instance,
            harness: args.harness,
            output_root: args.output_root,
            skip_evaluation: args.skip_evaluation,
            dry_run: args.dry_run,
        }),
        Action::Verify(args) => {
            workflow::verify(&args.benchmark_root, args.results.as_deref(), args.json)
        }
        Action::Report(args) => {
            workflow::report(&args.benchmark_root, args.results.as_deref(), args.json)
        }
        Action::Custody { archive_root } => verify_custody(&archive_root),
        Action::Adapter {
            command:
                AdapterAction::Install {
                    install_root,
                    print_path,
                },
        } => install(&install_root, print_path),
    }
}

fn write(path: &Path, body: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, body).map_err(|error| format!("write {}: {error}", path.display()))
}

fn install(root: &Path, print_path: bool) -> Result<(), String> {
    for (path, body) in [
        ("adapter.toml", ADAPTER),
        ("adapter-compatibility.json", ADAPTER_COMPATIBILITY),
        ("adapter-resources.json", RESOURCES),
        ("docs/programbench-run.md", RUN_GUIDE),
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
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn verify_custody(root: &Path) -> Result<(), String> {
    let manifest: SourceManifest =
        serde_json::from_str(SOURCE_MANIFEST).map_err(|error| error.to_string())?;
    let mut failed = Vec::new();
    for artifact in manifest.artifacts {
        let path = root.join(&artifact.path);
        match digest(&path) {
            Ok(actual) if actual == artifact.sha256 => {}
            Ok(actual) => failed.push(format!("{} digest {actual}", artifact.path)),
            Err(error) => failed.push(error),
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn singular_and_plural_benchmark_root_spellings_are_accepted() {
        for spelling in ["--benchmark-root", "--benchmarks-root"] {
            let parsed = Cli::try_parse_from([
                "ldgr-programbench",
                "reproduce",
                spelling,
                "/tmp/bench",
                "--dry-run",
            ])
            .unwrap();
            let Action::Reproduce(args) = parsed.command else {
                panic!("expected reproduce")
            };
            assert_eq!(args.benchmark_root, PathBuf::from("/tmp/bench"));
        }
    }

    #[test]
    fn reproduce_defaults_to_current_directory_and_a_bounded_instance() {
        let parsed = Cli::try_parse_from(["ldgr-programbench", "reproduce"]).unwrap();
        let Action::Reproduce(args) = parsed.command else {
            panic!("expected reproduce")
        };
        assert_eq!(args.benchmark_root, PathBuf::from("."));
        assert_eq!(args.instance, workflow::DEFAULT_INSTANCE);
        assert!(args.output_root.is_none());
        assert!(args.harness.is_none());
    }

    #[test]
    fn help_names_the_complete_workflow_and_examples() {
        let help = Cli::command().render_long_help().to_string();
        for expected in ["setup", "reproduce", "verify", "report", "Examples:"] {
            assert!(help.contains(expected), "missing {expected} from help");
        }
        assert!(!help.contains("archive-root"));
    }

    #[test]
    fn installer_materializes_run_guide_and_contract_files() {
        let temp = tempfile::tempdir().unwrap();
        install(temp.path(), false).unwrap();
        for path in [
            "adapter.toml",
            "adapter-compatibility.json",
            "adapter-resources.json",
            "docs/programbench-run.md",
            "docs/historical/reproduction-contract.md",
        ] {
            assert!(temp.path().join(path).is_file(), "missing {path}");
        }
        assert!(!temp.path().join("adapter-database-contract.json").exists());
    }
}
