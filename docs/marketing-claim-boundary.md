# ProgramBench marketing claim boundary

This document is the copy contract for public `ldgr` materials. It separates
the retained Hydra-Dynamix result from the official ProgramBench leaderboard so
that the benchmark can demonstrate task difficulty without implying an
apples-to-apples ranking.

## Supported result claim

Use:

> In a historical validator-visible, non-clean-room ProgramBench campaign,
> ldgr-assisted workflows completed four retained tasks: Hyperfine, Code
> Minimap, Brotli, and Nomino.

The retained evaluation artifacts contain only passing test results:

| ProgramBench instance | Passing tests | Eval SHA-256 |
| --- | ---: | --- |
| `sharkdp__hyperfine.327d5f4` | 298/298 | `6250b56d7b915f5be1dae87beef6ab977e9720224a14d397b45db70aad426c1e` |
| `wfxr__code-minimap.0ddeea5` | 369/369 | `06112ad6a8f4a86f874f82dfc22809747f71facfb714c363a1fed1174f6b9c71` |
| `google__brotli.b3dc9cc` | 606/606 | `7f71d5f94191f555175fff22b7f106d3c2c2a68fc9fe47b4042c7de4b68f2b18` |
| `yaa110__nomino.f892499` | 338/338 | `7211d98f62d49c504224820bd99e6dcba0ebcb5685ccfd499b906d43b30555df` |

The frozen adjudication rule classifies these four runs as
`valid_non_cleanroom`. It separately classifies seven other historical runs as
invalid because retained evidence establishes source or evaluator leakage.
Invalid runs are not included in the public result.

## Difficulty context

ProgramBench asks an agent to recreate a complete program from an executable
and documentation, without source access, internet access, or decompilation. A
candidate resolves an instance only when it passes every hidden behavioral
test.

On 2026-07-24, the official 200-task leaderboard reported a best resolved rate
of 0.5% for each of the leading GPT 5.5 configurations: one fully resolved
instance. This dated figure may be used only to establish that ProgramBench is
difficult. Link directly to <https://programbench.com/> and update both the
date and value before publishing or reusing the comparison.

## Required disclosure

Keep the following disclosure adjacent to the result, not hidden in legal
copy:

> Historical, validator-visible, non-clean-room reproduction; not an official
> ProgramBench submission or directly comparable to its generic-harness
> leaderboard.

The Hydra-Dynamix campaign used task-specific, long-running workflows and
validator-visible iteration. The official leaderboard uses a single generic
mini-SWE-agent harness across all 200 tasks in an isolated environment. Those
conditions measure different things.

## Prohibited claims

Do not say or imply:

- `ldgr` has an official ProgramBench score of 4/200;
- `ldgr` is four times better than OpenAI or any model;
- the four runs are clean-room, independent, or accepted leaderboard
  submissions;
- `ldgr` or a particular model generally solves ProgramBench;
- any of the seven invalidated historical runs contributes to the result.

## Evidence and reproduction

The authoritative local evidence chain is:

1. `docs/historical/leakage-rule.md` — frozen validity rule;
2. `docs/historical/classifications.json` — four valid and seven invalid runs;
3. `docs/historical/source-manifest.json` — retained artifact custody and
   digests;
4. `docs/historical/reproduction-contract.md` — bounded on-host reproduction
   conditions;
5. `ldgr programbench verify`, `reproduce`, and `report` — verification,
   re-execution, and report surfaces.

The retained archives and benchmark harness are deliberately not copied into
this public repository. Their roots are caller-supplied to the reproduction
commands.
