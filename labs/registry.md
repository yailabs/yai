# Labs Registry

Status: lab-local evidence registry; not architectural authority.

This registry lists the maintained lab/test evidence surfaces. “Canonical” in
this file means reproducible within `labs/`, not canonical YAI semantics.

Canonical numbering:

```text
Test 1 - filesystem-loop
Test 2 - context-residency
```

| Lab | Purpose | README | Runbook | Notebook | Run script | Prompt catalog | Pack fixture | Latest run |
|---|---|---|---|---|---|---|---|---|
| filesystem-loop | Filesystem-loop command, policy-boundary, prompt-catalog and model-context validation. | `filesystem-loop/README.md` | `filesystem-loop/runbook.md` | `filesystem-loop/notebook.ipynb` | `filesystem-loop/run.sh` | `filesystem-loop/prompts.json` | `filesystem-loop/pack-fixture/` | `filesystem-loop/runs/20260526-real-fsloop/` |
| external-runtime | Local provider reachability, endpoint status, prompt-catalog and runtime smoke evidence. | `external-runtime/README.md` | `external-runtime/runbook.md` | `external-runtime/notebook.ipynb` | `external-runtime/run.sh` | `external-runtime/prompts.json` | not defined | `external-runtime/runs/20260526-canon6-provider-probe/` |
| context-residency | Test 2 case-native benchmark matrix: naked no-context, raw-context, mini-RAG, YAI case-bound and logical residency; direct provider calls are limited to naked baselines. | `context-residency/README.md` | `context-residency/runbook.md` | not defined | `context-residency/run.sh` | `context-residency/prompts.json` | `context-residency/fixtures/` | generated under `context-residency/runs/` |
