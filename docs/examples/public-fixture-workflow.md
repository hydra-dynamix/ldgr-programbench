# Public fixture LDGR workflow example

This is the first concrete public `ldgr-programbench` proof slice described in
[`docs/public-scope.md`](../public-scope.md). It demonstrates LDGR evidence shape
for a ProgramBench-like task without invoking commercial `ldgr-bench` behavior.

## Boundary

This example uses only:

- the public fixture at `fixtures/tiny-addition/input.json`;
- the public helper `scripts/validate-tiny-addition.py`;
- core `ldgr` commands.

It does not read private benchmark suites, run ProgramBench evaluation, launch
cleanrooms, require commercial licenses, or call `ldgr-bench`.

## One-command smoke

From the repository root:

```sh
./scripts/run-public-fixture-smoke.sh
```

Or provide an explicit scratch project directory:

```sh
./scripts/run-public-fixture-smoke.sh /tmp/ldgr-programbench-public-smoke
```

The smoke script installs a local `ldgr-bench` sentinel that fails if invoked,
then runs the fixture workflow through public `ldgr` commands. A successful run
therefore proves the example is independent of commercial `ldgr-bench` behavior.
Run-94 smoke evidence is recorded in
[`public-fixture-smoke-evidence.md`](public-fixture-smoke-evidence.md).

## Exact LDGR commands

The script automates the following commands in a fresh scratch directory. Replace
`/path/to/ldgr-programbench` with this repository checkout.

```sh
REPO=/path/to/ldgr-programbench
WORKDIR=$(mktemp -d)
cd "$WORKDIR"

ldgr init
ldgr work create programbench-tiny-addition \
  --title "Validate public tiny-addition fixture" \
  --description "Record LDGR evidence for the synthetic public ProgramBench-shaped tiny-addition fixture."
RUN_ID=$(ldgr run start programbench-tiny-addition --command "public fixture smoke" \
  | awk '/started run / {print $3; exit}')

mkdir -p fixtures artifacts
cp "$REPO/fixtures/tiny-addition/input.json" fixtures/tiny-addition-input.json
python3 "$REPO/scripts/validate-tiny-addition.py" \
  --input fixtures/tiny-addition-input.json \
  --output artifacts/tiny-addition-validation.json

ldgr artifact add "$RUN_ID" \
  --kind json \
  --path fixtures/tiny-addition-input.json \
  --description "Public synthetic ProgramBench-shaped tiny-addition fixture input."
ldgr artifact add "$RUN_ID" \
  --kind json \
  --path artifacts/tiny-addition-validation.json \
  --description "Generated validation artifact derived from the public tiny-addition fixture."
ldgr validation record "$RUN_ID" \
  --outcome pass \
  --command "python3 scripts/validate-tiny-addition.py --input fixtures/tiny-addition-input.json --output artifacts/tiny-addition-validation.json" \
  --rationale "All fixture cases matched expected values and the public boundary declares no private benchmark or ldgr-bench requirement."
ldgr observe "$RUN_ID" \
  --body "Public tiny-addition fixture generated LDGR artifact and validation records using only core ldgr commands; commercial ldgr-bench was not invoked."
ldgr run close "$RUN_ID" \
  --status success \
  --outcome stop \
  --rationale "The bounded public fixture proof slice generated fixture artifact, validation artifact, validation record, and close decision without commercial benchmark behavior."
```

## Generated validation artifact

The fixture validator writes `artifacts/tiny-addition-validation.json`:

```json
{
  "case_results": [
    {
      "actual": 5,
      "expected": 5,
      "name": "positive integers",
      "passed": true
    },
    {
      "actual": 7,
      "expected": 7,
      "name": "includes zero",
      "passed": true
    },
    {
      "actual": 3,
      "expected": 3,
      "name": "negative integer",
      "passed": true
    }
  ],
  "commercial_boundary": {
    "ldgr_bench_invoked": false,
    "ldgr_bench_required": false,
    "private_benchmark_material_used": false
  },
  "fixture_id": "tiny-addition-001",
  "schema": "ldgr-programbench.public-validation.v1",
  "validation": {
    "boundary_passed": true,
    "case_count": 3,
    "outcome": "pass",
    "passed_case_count": 3
  }
}
```

## LDGR records produced

A successful smoke creates:

1. work item `programbench-tiny-addition`;
2. one run for the public fixture smoke;
3. an artifact record for `fixtures/tiny-addition-input.json`;
4. an artifact record for `artifacts/tiny-addition-validation.json`;
5. one `pass` validation record with the validator command and rationale;
6. one observation recording the no-`ldgr-bench` boundary;
7. a closed run with `success` / `stop` decision.

This is intentionally smaller than commercial `ldgr-bench`: it proves public LDGR
evidence mechanics for one toy fixture only.
