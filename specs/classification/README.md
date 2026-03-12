# classification

Canonical taxonomy and classification map used by runtime discovery.

## Files

- `classification/classification-map.json`
- `classification/action-taxonomy.json`
- `classification/asset-taxonomy.json`
- `classification/risk-taxonomy.json`
- `classification/signal-taxonomy.json`

## Usage

- Discovery/classification logic resolves family/specialization hints from these taxonomies.
- These files are canonical source artifacts and are runtime-consumable.
- Classification remains separate from compliance attachment: classifying an action does not
  attach policy objects by itself.

## Authoring guardrail

Do not encode policy effects or authority semantics here.
Those belong to specialization/compliance/overlay/customer-pack layers.

## Composition boundary

Classification answers "what this action looks like".
Governance attachment answers "which governable objects are active for this workspace".
The two are intentionally separated and converge only during runtime resolution.
