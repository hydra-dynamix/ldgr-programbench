# ldgr-programbench

`ldgr-programbench` is reserved for the public ProgramBench-facing LDGR integration slice.

The initial public release is intentionally small. Use the main LDGR tools directly while this repository grows concrete ProgramBench adapter commands, fixtures, and workflow documentation.

## Current scope

- Public home for future ProgramBench/LDGR integration work.
- Fixture-backed LDGR evidence example for one synthetic ProgramBench-shaped task.
- No separate adapter binary is released from this repository yet.
- See [docs/public-scope.md](docs/public-scope.md) for the public slice boundary, exclusions, and validation expectations.
- Start with [docs/examples/public-fixture-workflow.md](docs/examples/public-fixture-workflow.md) for exact commands, generated records, and smoke validation.

## Start with LDGR

```sh
cargo install --git https://github.com/hydra-dynamix/ldgr-core
cargo install --git https://github.com/hydra-dynamix/agentctl
```

Or use the integration installer:

```sh
git clone https://github.com/hydra-dynamix/ldgr
cd ldgr
./install.sh
```
