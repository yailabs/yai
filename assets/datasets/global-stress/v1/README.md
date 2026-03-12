# Global Stress Dataset — v1

This folder contains a **stress-test dataset** for YAI and Mind parity runs.

## Contents

- `prompts.csv` — the test case matrix (v1, 500 cases)
- `seed/episodic_events.jsonl` — sample episodic events (seed)
- `seed/semantic_nodes.jsonl` — sample semantic nodes (seed)
- `seed/semantic_edges.jsonl` — sample semantic edges (seed)
- `manifest.json` — dataset manifest (checksums + metadata)
- `YAI_Global_Stress_Test_v1.xlsx` — spreadsheet view of `prompts.csv`

## Execution modes

- **YAI-attached**: run the prompt through planes (control.sock/root.sock, engine, etc.)
- **Mind-standalone**: run Mind as an app (agentic wrapper) and compare outcomes/evidence.

## Notes

Seed files are intentionally minimal and deterministic (generated), meant to bootstrap graph/memory components.
