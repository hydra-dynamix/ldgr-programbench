#![cfg(all(target_os = "linux", target_arch = "x86_64"))]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

fn executable(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn reproduce_prepares_invokes_configured_harness_evaluates_and_verifies() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("home");
    let root = temp.path().join("benchmark");
    let bin = temp.path().join("bin");
    let task = temp.path().join("task");
    fs::create_dir_all(home.join(".ldgr")).unwrap();
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&task).unwrap();
    fs::write(
        task.join("README.md"),
        "Implement the documented fixture.\n",
    )
    .unwrap();
    fs::write(
        home.join(".ldgr/config.json"),
        r#"{
          "schema_version": 1,
          "default_harness": "pi",
          "selected_harnesses": ["pi", "codex"],
          "installed": []
        }"#,
    )
    .unwrap();

    executable(
        &bin.join("docker"),
        r#"#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  version|pull|run|rm) exit 0 ;;
  exec) /usr/bin/tar --exclude=./executable -C "$FAKE_TASK_ROOT" -czf - . ;;
  *) exit 0 ;;
esac
"#,
    );
    executable(
        &bin.join("agentctl"),
        r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == config ]]; then exit 0; fi
workspace=""
while (($#)); do
  if [[ "$1" == --cwd ]]; then workspace="$2"; shift 2; else shift; fi
done
printf 'success\n' > "$workspace/.ldgr/fake-run-status"
printf '#!/usr/bin/env bash\nprintf fixture\\n' > "$workspace/compile.sh"
chmod +x "$workspace/compile.sh"
printf 'candidate prepared by configured harness\n'
"#,
    );
    executable(
        &bin.join("uvx"),
        r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "${2:-}" == --help ]]; then exit 0; fi
if [[ "${2:-}" == eval ]]; then
  run_root="$3"
  for instance_root in "$run_root"/*; do
    [[ -d "$instance_root" && -f "$instance_root/submission.tar.gz" ]] || continue
    instance="$(basename "$instance_root")"
    printf '{"test_results":[{"status":"passed"},{"status":"failure"}],"error_code":null}\n' > "$instance_root/$instance.eval.json"
  done
fi
"#,
    );
    executable(
        &bin.join("ldgr"),
        r#"#!/usr/bin/env bash
set -euo pipefail
db=""
while (($#)); do
  case "$1" in
    --db) db="$2"; shift 2 ;;
    --artifact-root) shift 2 ;;
    init) mkdir -p "$(dirname "$db")"; : > "$db"; exit 0 ;;
    work) exit 0 ;;
    run)
      if [[ "${2:-}" == start ]]; then printf 'started run 1\n'; fi
      if [[ "${2:-}" == show ]]; then
        status="$(cat .ldgr/fake-run-status 2>/dev/null || printf running)"
        printf '{"status":"%s"}\n' "$status"
      fi
      if [[ "${2:-}" == close && -f .ldgr/fake-run-status ]]; then
        printf 'error: run 1 is already success\n' >&2
        exit 1
      fi
      exit 0
      ;;
    artifact|validation|observation|observe|decision) exit 0 ;;
    *) shift ;;
  esac
done
"#,
    );

    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new(env!("CARGO_BIN_EXE_ldgr-programbench"))
        .args(["reproduce", "--benchmark-root"])
        .arg(&root)
        .env("HOME", &home)
        .env("PATH", &path)
        .env("FAKE_TASK_ROOT", &task)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("setup: task image already available:"));
    assert!(stdout.contains("harness: pi"));
    assert!(stdout.contains("agentctl profile: ldgr-loop-pi"));
    assert!(stdout.contains("tests: 2 passed=1 failed=1 other=0"));
    assert!(stdout.contains("already closed by harness [success]"));
    assert!(stdout.contains("Attempt complete"));

    let run_root = fs::read_dir(root.join("programbench-runs"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let instance_root = run_root.join("sharkdp__hyperfine.327d5f4");
    assert!(instance_root.join("submission.tar.gz").is_file());
    assert!(instance_root
        .join("sharkdp__hyperfine.327d5f4.eval.json")
        .is_file());
    assert!(instance_root.join("workspace/.ldgr/ldgr.db").is_file());

    let verified = Command::new(env!("CARGO_BIN_EXE_ldgr-programbench"))
        .args(["verify", "--benchmark-root"])
        .arg(&root)
        .env("HOME", &home)
        .env("PATH", &path)
        .output()
        .unwrap();
    assert!(verified.status.success());
    assert!(String::from_utf8(verified.stdout)
        .unwrap()
        .contains("verification=complete artifacts=1"));
}
