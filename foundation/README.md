# foundation/

`foundation/` is the normative core of governance in the unified YAI
repository.

It defines first-order meaning and non-negotiable constraints for the YAI runtime ontology.
Primary ontology is:
- `core` (sovereign authority plane)
- `exec` (execution and external-effect plane)
- `brain` (cognitive plane)
- cross-cutting layers: `protocol`, `platform`, `support`

Historical labels (`boot`, `ingress`, `root`, `kernel`, `engine`, `mind`) are not primary normative domains.
They are migration aliases only.

## Normative sections

- `axioms/`: first principles of execution, authority, state, adaptability
- `invariants/`: mandatory system constraints that must hold continuously
- `boundaries/`: separation surfaces between planes and layers
- `terminology/`: controlled vocabulary and deprecation taxonomy
- `extensions/`: scoped subordinate normative extensions

## Conformance direction

All downstream normative surfaces must conform to this foundation:
- `runtime/`
- `contracts/`
- `model/registry/`
- `model/schema/`
- `control/assurance/`
- `packs/`

If downstream artifacts conflict with `foundation/*`, foundation prevails unless an explicit compatibility/deprecation rule exists.

## Rewrite discipline

Foundation documents must:
- define semantics, not filesystem layout
- use primary ontology terms (`core/exec/brain`) as canonical
- tag historical terms as migration aliases when used
- maintain stable IDs where possible (`A-*`, `I-*`, `L*`) while updating content

## Cross-file relationship

- Axioms define what the system is.
- Invariants define what must always remain true.
- Boundaries define where authority, execution, cognition, and shared layers are separated.
- Terminology prevents semantic drift across the repository.
