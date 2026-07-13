#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
workdir="${1:-$(mktemp -d)}"
mkdir -p "$workdir"
cd "$workdir"

# Guardrail: prove this public example does not invoke commercial ldgr-bench behavior.
mkdir -p .public-smoke-bin
cat > .public-smoke-bin/ldgr-bench <<'STUB'
#!/usr/bin/env bash
echo "ldgr-bench must not be invoked by the public fixture smoke" >&2
touch .ldgr-bench-invoked
exit 97
STUB
chmod +x .public-smoke-bin/ldgr-bench
export PATH="$PWD/.public-smoke-bin:$PATH"

ldgr init >/tmp/ldgr-programbench-init.out
ldgr work create programbench-tiny-addition \
  --title "Validate public tiny-addition fixture" \
  --description "Record LDGR evidence for the synthetic public ProgramBench-shaped tiny-addition fixture."
run_output="$(ldgr run start programbench-tiny-addition --command "public fixture smoke")"
printf '%s\n' "$run_output"
run_id="$(printf '%s\n' "$run_output" | awk '/started run / {print $3; exit}')"
if [[ -z "$run_id" ]]; then
  echo "failed to parse LDGR run id" >&2
  exit 1
fi

mkdir -p fixtures artifacts
cp "$repo_root/fixtures/tiny-addition/input.json" fixtures/tiny-addition-input.json
python3 "$repo_root/scripts/validate-tiny-addition.py" \
  --input fixtures/tiny-addition-input.json \
  --output artifacts/tiny-addition-validation.json

ldgr artifact add "$run_id" \
  --kind json \
  --path fixtures/tiny-addition-input.json \
  --description "Public synthetic ProgramBench-shaped tiny-addition fixture input."
ldgr artifact add "$run_id" \
  --kind json \
  --path artifacts/tiny-addition-validation.json \
  --description "Generated validation artifact derived from the public tiny-addition fixture."
ldgr validation record "$run_id" \
  --outcome pass \
  --command "python3 scripts/validate-tiny-addition.py --input fixtures/tiny-addition-input.json --output artifacts/tiny-addition-validation.json" \
  --rationale "All fixture cases matched expected values and the public boundary declares no private benchmark or ldgr-bench requirement."
ldgr observe "$run_id" \
  --body "Public tiny-addition fixture generated LDGR artifact and validation records using only core ldgr commands; commercial ldgr-bench was not invoked."
ldgr run close "$run_id" \
  --status success \
  --outcome stop \
  --rationale "The bounded public fixture proof slice generated fixture artifact, validation artifact, validation record, and close decision without commercial benchmark behavior."

if [[ -e .ldgr-bench-invoked ]]; then
  echo "FAIL: ldgr-bench was invoked" >&2
  exit 1
fi

printf '\nSmoke project: %s\n' "$workdir"
printf 'Generated validation artifact:\n'
cat artifacts/tiny-addition-validation.json
printf '\nLDGR status:\n'
ldgr status
