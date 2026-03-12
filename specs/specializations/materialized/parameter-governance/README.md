# scientific.parameter-governance

Scientific specialization for parameter mutation governance.

## Focus
- safe mutation of sensitive parameters
- explicit handling of locked parameters
- publication-impacting parameter change control

## Baseline semantics
- missing parameter hash/lock trace: `deny`
- governed mutation path: `review_required`
- evidence: lock trace, parameter diff, approval chain
