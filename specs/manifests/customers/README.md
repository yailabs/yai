# Customer Policy Packs

Canonical customer/workspace governance packs.

- Index: `manifests/customer-policy-packs.index.json`
- Schema: `grammar/schema/customer_policy_pack.v1.schema.json`
- Architecture model: `docs/architecture/customer-policy-pack-model.md`

These manifests compose existing compliance/overlay/specialization surfaces into
one customer-facing runtime input contract.

## Structure

- `examples/` runnable sample manifests
- `templates/` starter manifests for deterministic authoring
- `profiles/` starter authority/evidence profiles

## Validation

```bash
./tools/bin/yai-govern-ingest-validate
```
