# External Runtime Runbook

Status: canonical CLI procedure.

This runbook is the complete terminal procedure for the external-runtime lab.
The notebook is a guided human demo; provider probing and report generation
belong to `run.sh` and `labs/shared/bin/`.

## Prerequisites

Run commands from the repository root:

```bash
pwd
test -f labs/external-runtime/README.md
test -x labs/external-runtime/run.sh
python3 -m json.tool labs/external-runtime/prompts.json >/tmp/yai-external-prompts.json
```

Provider defaults:

```bash
export YAI_PROVIDER_HOST="${YAI_PROVIDER_HOST:-127.0.0.1}"
export YAI_PROVIDER_PORT="${YAI_PROVIDER_PORT:-43117}"
export YAI_PROVIDER_MODEL="${YAI_PROVIDER_MODEL:-qwen-local}"
```

Do not put secrets or endpoint credentials in notebooks, reports or prompt
catalogs.

## Prompt Listing

Prompt text authority is `labs/external-runtime/prompts.json`.

```bash
labs/external-runtime/run.sh --list-prompts
```

## Provider Probe Run

Run exactly one provider prompt:

```bash
labs/external-runtime/run.sh \
  --slug canon6-provider-probe \
  --prompt-id local-provider-ready
```

The run script resolves the prompt from `prompts.json`, calls
`labs/shared/bin/probe-local-provider.sh`, and stores request payload, curl
metadata, provider responses and summary under run `assets/`.

Expected run package:

```text
labs/external-runtime/runs/YYYYMMDD-canon6-provider-probe/
  transcript.md
  report.md
  manifest.json
  metrics.json
  assets/
```

If the provider is unavailable, keep the generated run. The report should state
provider unavailable and must not claim provider readiness.

## Structural Smoke Fallback

Use this when no local provider should be contacted:

```bash
labs/external-runtime/run.sh \
  --slug structural-smoke \
  --command "true"
```

This is structural evidence only. It is not a provider benchmark and not a
model-quality evaluation.

## Report and Figure Regeneration

After a run exists, compose report-first tables and figures:

```bash
python3 labs/shared/bin/generate-run-figures.py \
  --run-dir labs/external-runtime/runs/<run-dir> \
  --update-report \
  --overwrite
```

Generated figures are supporting evidence inside `report.md`. The final human
artifact is `report.md`; `transcript.md` is raw chronological execution
evidence; `metrics.json` is machine-readable data; `assets/` contains
attachments.

## Output Inspection

```bash
ls labs/external-runtime/runs/<run-dir>
sed -n '1,200p' labs/external-runtime/runs/<run-dir>/transcript.md
sed -n '1,220p' labs/external-runtime/runs/<run-dir>/report.md
python3 -m json.tool labs/external-runtime/runs/<run-dir>/manifest.json
python3 -m json.tool labs/external-runtime/runs/<run-dir>/metrics.json
find labs/external-runtime/runs/<run-dir>/assets -maxdepth 2 -type f | sort
```

## Validation

```bash
python3 -m json.tool labs/external-runtime/notebook.ipynb
bash -n labs/external-runtime/run.sh
bash -n labs/shared/bin/probe-local-provider.sh
tools/checks/check-lab-notebooks.sh
tools/checks/check-labs-layout.sh
tools/checks/check-lab-run-contract.sh
make check-lab-notebooks
make check-labs
make check-lab-runs
make check-docs
```

## Failure Interpretation

### Repeatable real-provider latency sampling

For a locally installed llama.cpp or vLLM server, use the same explicit prompt
and output cap through the generic OpenAI-compatible transport. Bind servers to
loopback, run one model server at a time, and retain their exact launch arguments,
runtime version, model revision/digest, precision, context size and GPU posture.
Installation or a valid HTTP response alone does not qualify YAI execution.

```bash
python3 labs/shared/bin/benchmark-provider-http.py \
  --endpoint http://127.0.0.1:43117/v1 \
  --model '<exact served model>' \
  --runtime '<runtime and version>' \
  --weights '<revision/digest and precision>' \
  --samples 5 \
  --output build/evidence/providers/<unique-run-id>
```

The output directory must be new. It contains the manifest, unedited response
bodies and provider-reported usage for each request, plus aggregate nonstreaming
request latency with the first warmup excluded. Missing usage remains missing;
words are not tokens. This does not measure TTFT, pure decode throughput, model
correctness, or YAI overhead. Different precision/quantization is an explicit
comparison limitation, not an equal-compute result. Follow direct transport
sampling with the relevant real YAI product flow in an isolated qualification
profile; retain those results separately from deterministic fixture evidence.

- HTTP failures are endpoint evidence, not provider-quality conclusions.
- Missing provider timing, token usage or response content must be reported as
  `Not measured`.
- A structural smoke run does not attach a provider.
- Do not infer hardware benchmark, throughput, production readiness or model
  quality from this lab.

## Boundary

External-runtime reports local provider evidence only. Provider endpoint status,
curl timing and request/response artifacts are evidence for that run. They are
not a benchmark unless a benchmark run records explicit quantitative
measurements and limitations.
