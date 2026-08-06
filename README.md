# ldgr-programbench

`ldgr-programbench` prepares and runs a real, bounded ProgramBench attempt with
the harness selected in the user's LDGR configuration. It creates a usable
workspace, invokes the harness through `agentctl`, packages the candidate,
evaluates it with ProgramBench, and retains the attempt as LDGR evidence.

It is an on-host, validator-visible demonstration. It is not an official
ProgramBench submission, a clean-room run, an independent evaluation, or a
score suitable for general model ranking.

## Quick start

Install the signed adapter, enter the directory that should own the run, and use
the defaults:

```sh
ldgr adapter install programbench
mkdir -p ~/repos/programbench
cd ~/repos/programbench

ldgr programbench setup
ldgr programbench reproduce
ldgr programbench verify
ldgr programbench report
```

`reproduce` defaults to:

- the current directory as `--benchmark-root`;
- `sharkdp__hyperfine.327d5f4` as one bounded demonstration task;
- `default_harness` from `~/.ldgr/config.toml` or `~/.ldgr/config.json`;
- `./programbench-runs/<timestamp>` for output;
- ProgramBench evaluation after the harness returns.

The singular `--benchmark-root` spelling is canonical. The plural
`--benchmarks-root` spelling is an accepted compatibility alias:

```sh
ldgr programbench reproduce --benchmark-root ~/repos/programbench
ldgr programbench reproduce --benchmarks-root ~/repos/programbench
```

Use `--harness codex` (or another harness already selected in the user config)
to override the configured default. Use `--dry-run` to write and display the
complete run plan without downloading task images or invoking the harness.

## Command responsibilities

- `setup` checks Docker, `uvx`, `agentctl`, and the selected harness; installs
  ProgramBench through `uvx`; pulls the task image; and writes local setup files.
- `reproduce` performs a new attempt. It does not require a previous submission,
  evaluation artifact, or historical archive.
- `verify` validates and summarizes the newest local evaluation result, or the
  run selected with `--results`.
- `report` renders the same evidence with the interpretation boundary attached.
- `custody` is the explicitly separate legacy command for checking retained
  historical artifacts.

See [`docs/programbench-run.md`](docs/programbench-run.md) for the installed run
guide. Historical classifications remain available under `docs/historical/`,
but they no longer define or block the normal reproduction workflow.

## Numerical sequence protocol

When LDGR Core numerical telemetry is explicitly enabled, the reproduction
command queues only the ProgramBench reproduction state machine
`/sequences/programbench-reproduction/v1` through Core. Adapter states represent
inputs prepared, reproduction prepared, attempt recorded, and evidence
finalized. Benchmark names, paths, commands, logs, prompts, and result contents
are not encoded or transmitted.
