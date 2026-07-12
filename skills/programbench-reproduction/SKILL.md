---
name: programbench-reproduction
description: Verify or reproduce the four bounded historical LDGR-assisted ProgramBench runs. Use when validating the published spiritual proof, checking custody hashes, rerunning under a current on-host validator-visible harness, or producing a bounded report.
---

# ProgramBench Reproduction

1. Run `ldgr programbench verify` against the configured historical archive before execution.
2. Run `ldgr programbench reproduce` with explicit archive, benchmarks, and output roots.
3. Preserve every failure and raw log. Do not repair candidates during reproduction.
4. Run `ldgr programbench report` and retain valid, invalid, and unresolved classifications.
5. Never call this an official score, submission, clean-room run, or independent benchmark.
