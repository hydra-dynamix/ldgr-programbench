# ldgr-programbench

`ldgr-programbench` is the open, narrow reproduction adapter for four historical LDGR-assisted ProgramBench runs. It verifies retained custody, reruns frozen submissions under the current on-host validator-visible harness, records raw and LDGR evidence, and produces bounded reports.

It is not an official benchmark submission, a clean-room run, an independent evaluation, or a score suitable for model ranking.

```sh
ldgr adapter install programbench
ldgr programbench verify --archive-root /path/to/20260613-archive
ldgr programbench reproduce \
  --archive-root /path/to/20260613-archive \
  --benchmarks-root /path/to/benchmarks \
  --output-root /path/to/new-evidence
ldgr programbench report --results /path/to/new-evidence/results.json
```

The four retained valid non-cleanroom runs are Hyperfine, Code Minimap, Brotli, and Nomino. Invalidated runs remain visible in the classification report.

For website and launch-copy wording, use the bounded claim in
[`docs/marketing-claim-boundary.md`](docs/marketing-claim-boundary.md). In
particular, these four historical reproductions must not be presented as an
official 4/200 leaderboard score or directly ranked against clean-room,
generic-harness results.

Install the bundle directly for local development with `ldgr-programbench adapter install --install-root <path>`. Core’s adapter installer is the canonical distribution path.

## Numerical sequence protocol

When LDGR Core numerical telemetry is explicitly enabled, the reproduction command queues only the ProgramBench reproduction state machine `/sequences/programbench-reproduction/v1` through Core. Adapter states are `8` custody-verified, `9` reproduction-prepared, `10` attempts-recorded, and `11` evidence-finalized. Normalized terminal codes keep the Core meanings: `3` completed-positive, `4` completed-negative, `5` completed-inconclusive, `6` operational-failure, and `7` cancelled.

The adapter uses Core's `LocalSequenceBuffer` only. Core persists a bare integer array such as `[0,1,8,9,10,11,4]`; benchmark names, target answers, repository data, archive paths, output paths, commands, logs, artifact descriptions, and reproduction outputs are not encoded or transmitted.
