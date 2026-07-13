# ldgr-programbench Public Scope

`ldgr-programbench` is the narrow Apache-2.0 public proof slice for showing how LDGR can be used in ProgramBench-facing evaluation workflows without publishing commercial benchmark automation.

## Current implementation state

The repository now contains the open `ldgr-programbench` adapter binary, canonical `ldgr programbench` domain, typed harness resources, custody verifier, frozen historical classifications, on-host reproduction command, and bounded report command. Commercial benchmark automation remains outside this repository.

This keeps the public proof slice honest: anything documented here must be runnable with open LDGR surfaces or explicitly marked as planned.

## Relationship to neighboring components

- `ldgr-core` owns the generic ledger, loop lifecycle, artifacts, observations, validations, decisions, and adapter dispatch primitives.
- `ldgr-research` owns open research/readiness workflow primitives such as programs, experiments, facts, validation records, readiness audits, and adapter-neutral research context.
- `ldgr-recall` is advisory memory/retrieval support. It may help select or render prior evidence, but fresh LDGR state and validation remain authoritative.
- `ldgr-programbench` should be a thin public proof layer over those open surfaces: examples, fixtures, scripts, and documentation that demonstrate ProgramBench-shaped use of LDGR.
- Future commercial `ldgr-bench` owns closed or paid benchmark-product behavior, private corpora/workloads, entitlement checks, proprietary automation, and commercial release mechanics.

## Intended public examples and scripts

The public slice should prefer small, inspectable assets that can be run without private services or licenses:

1. Example ProgramBench-shaped LDGR project initialization using `ldgr` and, when useful, `ldgr research`.
2. Example work-item decomposition for a benchmark task with explicit observations, artifacts, validations, and decisions.
3. Minimal fixtures representing non-sensitive benchmark inputs/outputs.
4. Scripts that convert public fixture results into LDGR artifacts or validation records.
5. Smoke checks proving that the documented examples still run against current public LDGR commands.
6. Documentation explaining how ProgramBench proof evidence maps onto LDGR records.

The adapter is a convenience wrapper around inspectable public flows and contains no hidden scoring or entitlement policy.

## Explicit exclusions

`ldgr-programbench` must not contain:

- commercial license, entitlement, subscription, renewal, customer, or activation logic;
- private benchmark suites, private datasets, private scoring policies, or proprietary ProgramBench workloads;
- release-signing keys, customer license schemas, or enforcement internals;
- closed `ldgr-bench` automation or code paths presented as public examples;
- Core changes that make LDGR reject unrestricted third-party or open adapters;
- claims that advisory recall output is authoritative without fresh validation.

Commercial enforcement and proprietary benchmark execution belong in future closed/commercial `ldgr-bench` or private support crates, not in this public proof slice.

## Validation expectations

Public ProgramBench materials should be accepted only when they have evidence that matches their maturity level:

- Documentation-only changes: validate links, command names, and current implementation state against the repository contents and current LDGR help where practical.
- Example scripts: run the script on public fixtures and record the command plus output artifact.
- Adapter/profile additions: include smoke tests for installation/discovery/profile application through public LDGR surfaces.
- Benchmark-result examples: include deterministic fixture inputs, generated LDGR artifacts, and validation records explaining pass/fail criteria.
- Any use of `ldgr-recall`: label it advisory and pair it with authoritative LDGR observations/artifacts/validations before making claims.

A future public release should not claim ProgramBench coverage beyond the fixtures and workflows that were actually validated.

## First concrete public slice

The first useful increment should be a fixture-backed example workflow:

1. initialize an LDGR project;
2. create one ProgramBench-shaped work item;
3. attach a small public fixture artifact;
4. record one validation result derived from that fixture;
5. close the run with a bounded decision;
6. document the exact commands in this repository.

That increment is intentionally smaller than `ldgr-bench`: it demonstrates proof mechanics and evidence shape, not commercial benchmark operations.
