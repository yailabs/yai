# Overlays

Canonical overlay engine surfaces.

Runtime composition is driven by:

- regulatory overlays: `regulatory/*` + `regulatory/index/*`
- sector overlays: `sector/*` + `sector/index/*`
- contextual overlays: `contextual/*` + `contextual/index/*`

Core matrices:

- attachment: `overlay-attachment-matrix.json`
- precedence: `overlay-precedence-matrix.json`
- evidence: `overlay-evidence-matrix.json`

Generated runtime view:

- `index/overlay-compliance.runtime.v1.json`
