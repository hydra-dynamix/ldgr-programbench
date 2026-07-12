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

Install the bundle directly for local development with `ldgr-programbench adapter install --install-root <path>`. Core’s adapter installer is the canonical distribution path.
