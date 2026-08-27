# Experimental evidence

Authority: lab-local procedures, inputs, and captured results only.

`labs/` preserves reproducible experiments for filesystem behavior, external
runtime probing, and context-residency research. A lab may call its own
runbook, prompt catalog, notebook, or report “canonical”; that word is scoped
only to reproducing that experiment. No lab owns YAI architecture, semantic
definitions, implementation status, or roadmap ordering.

The lab registry and standards remain useful evidence organization. Results
must be read with their run manifest, conditions, limitations, and date. A
successful run proves only the observed configuration; synthetic or logical
estimates are labeled and cannot establish implementation capability.

Current authority is mapped in [`docs/index.md`](../docs/index.md). The frozen
context-residency run is design evidence for Projection/Residency separation;
it implements neither ContextFrame/ContextDelta nor provider KV continuation.
