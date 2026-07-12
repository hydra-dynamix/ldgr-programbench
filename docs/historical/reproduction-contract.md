# Historical reproduction contract

This is an on-host, validator-visible revalidation of four historical LDGR-assisted ProgramBench-shaped runs. It is not a migration, an official benchmark submission, or a clean-room replication.

Inputs are the frozen source manifest, classifications, retained submission archives, current ProgramBench harness selected by `--benchmarks-root`, and a caller-selected output root. The reproduction must record environment metadata, input digests, exact commands, stdout/stderr, exit status, current eval artifacts, and comparison with historical digests/results.

The validator is deliberately visible and `--force` evaluation feedback may be used. No result may be described as independent. Each run executes once per invocation; failures are retained and do not trigger automatic repair. Historical inputs are read-only. Output is written only beneath the configured output root.

Prohibited interpretations: official score, clean-room result, benchmark submission, evidence of general model superiority, evidence that invalid runs became valid, or evidence that current results are independent of historical validator feedback.
