# Tiny addition public fixture

This fixture is a synthetic ProgramBench-shaped input for demonstrating LDGR evidence mechanics.
It is intentionally not a private benchmark task and does not require or invoke `ldgr-bench`.

The fixture contains:

- one small task prompt;
- a candidate source string;
- deterministic case results with `expected` and `actual` values;
- an explicit public-boundary marker.

Use `scripts/validate-tiny-addition.py` to derive a validation artifact from `input.json`.
