#!/usr/bin/env python3
"""Validate the public tiny-addition fixture and emit an LDGR artifact JSON."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path, help="Fixture input JSON")
    parser.add_argument("--output", required=True, type=Path, help="Validation artifact JSON to write")
    args = parser.parse_args()

    fixture = json.loads(args.input.read_text(encoding="utf-8"))
    cases = fixture.get("candidate_result", {}).get("cases", [])
    case_results = []
    for case in cases:
        expected = case.get("expected")
        actual = case.get("actual")
        case_results.append(
            {
                "name": case.get("name", "unnamed"),
                "expected": expected,
                "actual": actual,
                "passed": actual == expected,
            }
        )

    boundary = fixture.get("public_boundary", {})
    boundary_passed = (
        boundary.get("contains_private_benchmark") is False
        and boundary.get("requires_commercial_ldgr_bench") is False
    )
    all_cases_passed = bool(case_results) and all(result["passed"] for result in case_results)
    passed = all_cases_passed and boundary_passed

    artifact = {
        "schema": "ldgr-programbench.public-validation.v1",
        "fixture_id": fixture.get("fixture_id"),
        "validation": {
            "outcome": "pass" if passed else "fail",
            "case_count": len(case_results),
            "passed_case_count": sum(1 for result in case_results if result["passed"]),
            "boundary_passed": boundary_passed,
        },
        "case_results": case_results,
        "commercial_boundary": {
            "ldgr_bench_required": False,
            "ldgr_bench_invoked": False,
            "private_benchmark_material_used": False,
        },
    }

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(artifact, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"{artifact['validation']['outcome']} {args.output}")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
