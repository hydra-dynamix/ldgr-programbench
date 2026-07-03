# Public fixture smoke evidence

Run 94 validation command:

```sh
cd ldgr-programbench && ./scripts/run-public-fixture-smoke.sh /tmp/ldgr-programbench-public-smoke-run94
```

Observed result on 2026-07-03:

```text
created work item 1 programbench-tiny-addition
started run 1 for programbench-tiny-addition
pass artifacts/tiny-addition-validation.json
added artifact 1 submitted/run-1-1783095947970274930-tiny-addition-input.json
added artifact 2 submitted/run-1-1783095947974060724-tiny-addition-validation.json
Validation: 6
run_id: 1
outcome: pass
command: python3 scripts/validate-tiny-addition.py --input fixtures/tiny-addition-input.json --output artifacts/tiny-addition-validation.json
rationale: All fixture cases matched expected values and the public boundary declares no private benchmark or ldgr-bench requirement.
added observation 1
closed run 1 [success] and recorded decision 1 [stop] for programbench-tiny-addition
```

Generated validation artifact excerpt:

```json
{
  "fixture_id": "tiny-addition-001",
  "schema": "ldgr-programbench.public-validation.v1",
  "validation": {
    "boundary_passed": true,
    "case_count": 3,
    "outcome": "pass",
    "passed_case_count": 3
  },
  "commercial_boundary": {
    "ldgr_bench_invoked": false,
    "ldgr_bench_required": false,
    "private_benchmark_material_used": false
  }
}
```

LDGR status at the end of the scratch project showed `pending=0`, `running=0`,
`done=1`, latest validation `pass`, and latest decision `stop` for
`programbench-tiny-addition`.

The smoke script placed a failing `ldgr-bench` sentinel first on `PATH`; the run
completed and no `.ldgr-bench-invoked` marker was created, so the public example
ran without commercial `ldgr-bench` behavior.
