# Fast Search reference characterization

This is an opt-in external reference experiment, not a YAI inference runtime or
provider registration. The production path has no public System Model producer;
`fast` SEND currently uses qualified standard Recall/W and says so.

The observed source release was [Laya v0.3.10](https://github.com/NandhaKishorM/laya/tree/002e3b8f6c387a4d0bcefefc5dd7a3f8e07c0454)
at tag commit `002e3b8f6c387a4d0bcefefc5dd7a3f8e07c0454`; the live upstream HEAD
was `2c6c16baf3ea3149948777937d5005a7c7fba425`. The model repository was
pinned to [revision `5e7b2b1b8ca2ecdd3f2322d94069c9b6ce7e844b`](https://huggingface.co/convaiinnovations/laya/tree/5e7b2b1b8ca2ecdd3f2322d94069c9b6ce7e844b).
Exact downloaded weights, installed package and caches stayed outside Git.

| Checkpoint | SHA-256 of `model.safetensors` | Bytes | CPU load | Peak process RSS | Warm median |
|---|---|---:|---:|---:|---:|
| root/English | `891102d372688fc2a094dac56a384bc537b87c63f21f9f3dac0be2b7cbc8d86c` | 842,609,210 | 1,810 ms | 3,103,588 KiB | 282 ms |
| multilingual | `9d628fd971b700382ac6f65920a86f149777b2e748e0c955fb3b19695aa8f204` | 643,835,514 | 3,043 ms | 2,638,104 KiB | 122 ms |

Observation: 2026-09-24, run `fast-search-0d1437-final` orders 3–5 in
[raw command evidence](evidence.jsonl), CPU-only (CUDA unavailable), four
Torch/BLAS threads, Laya package 0.3.10. The English checkpoint warned that an invalid/out-of-range
temperature was clamped; neither its output confidence nor the multilingual
output is accepted as calibrated correctness probability by YAI.

Six real YAI task captures came from the independent
[`semantic-working-state-sufficiency`](../../tests/characterization/semantic-working-state-sufficiency/test_sufficiency.py)
fixture with its unchanged oracle: documentary knowledge, documentary/operational
contradiction, temporal-causal explanation, 256-change distractor pressure,
revoked evidence and a 73-source pressure case. Each was compiled by ordinary
Recall/W and passed through the exact `semantic.fast_search.prepare` Application
operation/CLI adapter. The final candidate descriptors contained 4–9 disclosed
choices. Standard Recall/W held all required evidence in each case, had zero
forbidden disclosure and did not infer a false causal relation.

The root/English model chose deterministic fallback in all six cases. This
preserved standard evidence but avoided no proven primary-model calls. The
multilingual model chose a resident group in all six, but **none** of those
single groups contained all task-required evidence (the 73-source case held
one of two required selectors). The YAI qualification layer prevented hidden
candidates from entering either input; that is not evidence of learned
disclosure awareness. No model quality, navigation-step reduction or long-horizon
speedup is established. Current production adoption is therefore withheld.

Observed model decision latency over the six actual tasks was 250–421 ms for
root/English and 116–232 ms for multilingual. Current qualified Fast Search
preparation took about 949–1,099 ms on the ordinary/256-change cases and
9,301 ms on the 73-source case. Eight-way batch timing was 1,942 ms and
958 ms respectively. Synthetic candidate-count probes at 2/8/16/32 choices
took root/English 163/261/287/275 ms and multilingual 52/93/127/121 ms;
these are descriptive CPU measurements, not an SLA. No usable GPU was present,
so GPU memory and latency are unmeasured. Language suitability outside these
English YAI tasks, including Italian, remains unqualified.

To reproduce, set `YAI_FAST_SEARCH_CAPTURE_DIR` to a disposable directory and
run the independent sufficiency suite with the current `./yai` build. Then run
`laya_reference.py --captures DIR --model PINNED_LOCAL_CHECKPOINT
--expected-sha256 HASH --model-revision REV` with the external Laya 0.3.10
environment. This runner reads only captured disclosed candidate metadata,
returns relative scores, and never writes a Case. The public producer seam,
calibration and equal-compute generative comparison remain open.
