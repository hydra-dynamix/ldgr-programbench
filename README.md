# ldgr-programbench

`ldgr-programbench` is reserved for the public ProgramBench-facing LDGR integration slice.

The initial public release is intentionally small. Use the main LDGR tools directly while this repository grows concrete ProgramBench adapter commands, fixtures, and workflow documentation.

## Current scope

- Public home for future ProgramBench/LDGR integration work.
- Narrow benchmark-facing examples and scripts once they are ready.
- No separate adapter binary is released from this repository yet.

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
