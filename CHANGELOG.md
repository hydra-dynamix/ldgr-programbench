# Changelog

## Unreleased

## [0.1.5] - 2026-08-06

### Fixed

- Preserve a terminal LDGR run status when the configured harness already
  closed the run instead of failing while attempting a duplicate close.
- Print and flush each setup preflight step, show dependency download output,
  and skip the task-image pull when the image is already available locally.

## [0.1.4] - 2026-08-06

### Added

- Add `setup` to prepare the local ProgramBench tools, task image, workspace,
  default folder layout, and user-configured harness plan.
- Add an end-to-end `reproduce` workflow that invokes the user's selected LDGR
  harness through `agentctl`, packages the candidate, evaluates it with
  ProgramBench, and records LDGR evidence.
- Add result-oriented `verify` and `report` commands with newest-run defaults,
  human-readable output, and JSON output.

### Changed

- Make `--benchmark-root` canonical and accept `--benchmarks-root` as a visible
  compatibility alias.
- Remove historical archives and completed result files from the normal
  reproduction inputs; historical digest checking now lives under `custody`.
- Default reproduction to the current directory, one bounded Hyperfine task,
  the user's configured default harness, and timestamped local output.

## [0.1.1] - 2026-07-05

### Added

- Add public tiny-addition fixture scaffolding, workflow documentation, and smoke-validation evidence for the open ProgramBench integration slice.

### Changed

- Clarify the public scope boundary: this repository does not release a separate adapter binary yet and does not require private benchmark material or `ldgr-bench`.
