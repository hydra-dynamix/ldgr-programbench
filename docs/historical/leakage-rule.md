# Frozen leakage adjudication rule

Status: frozen for the historical ProgramBench reproduction campaign on 2026-07-12.

A run is `valid_non_cleanroom` only when its recorded implementation lineage contains no upstream source, evaluator test source, golden payload, test-name/environment branch, copied installed implementation, or wrapper/delegation to the reference; its candidate is independently authored; and its result is backed by a retained submission, eval JSON, LDGR/provenance record, and validator-visible host workflow.

A run is `invalid_source_leakage` when direct artifacts establish any prohibited source or evaluator-aware behavior. Moving or deleting the offending code later does not restore the historical run.

A run is `unresolved` when the retained evidence cannot establish either condition. Absence of a detected marker is not proof of independence. Uncertainty never rounds up to valid.

This rule is intentionally stricter than the historical archive labels. The four valid runs are evidence of long-running LDGR-assisted task execution only. They are not official submissions, clean-room results, independent benchmark estimates, or a general performance score.
