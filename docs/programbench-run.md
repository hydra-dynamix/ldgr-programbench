# Run ProgramBench with LDGR

The adapter runs one bounded ProgramBench attempt with the harness selected in
the user's LDGR configuration. It prepares the task, invokes the harness through
`agentctl`, packages the candidate, evaluates it, and records an LDGR ledger.

From the directory that should own the run:

```sh
ldgr programbench setup
ldgr programbench reproduce
ldgr programbench verify
ldgr programbench report
```

Defaults:

- benchmark root: the current directory;
- task: `sharkdp__hyperfine.327d5f4`;
- harness: `default_harness` from `~/.ldgr/config.toml` or `config.json`;
- results: `./programbench-runs/<timestamp>`;
- evaluation: enabled.

Use `--benchmark-root` to choose another project directory. The common plural
spelling `--benchmarks-root` is accepted as an alias. Use `--dry-run` to inspect
the complete plan without downloads, containers, or harness execution.

The run is on-host and validator-visible. It is useful as an LDGR workflow
demonstration, but it is not an official ProgramBench submission, a clean-room
result, an independent score, or a general model ranking.
