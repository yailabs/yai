```yai-policy-json
{
  "schema": "yai.policy_source_input.v4",
  "policy_key": "engineering",
  "source_version": "1",
  "owner_ref": "organization:engineering",
  "source_origin": {
    "source_system": "enterprise-policy",
    "source_uri": "enterprise://engineering/policy/1"
  },
  "validity": {
    "mode": "unbounded"
  },
  "rules": [
    {
      "kind": "operation_restriction",
      "rule_id": "read",
      "operation_kind": "filesystem.read",
      "resource_kind": "filesystem",
      "effect": "allow",
      "reason": "Read only admitted source"
    },
    {
      "kind": "operation_restriction",
      "rule_id": "search",
      "operation_kind": "filesystem.search",
      "resource_kind": "filesystem",
      "effect": "deny",
      "reason": "Bulk source search is prohibited"
    },
    {
      "kind": "operation_restriction",
      "rule_id": "write",
      "operation_kind": "filesystem.write",
      "resource_kind": "filesystem",
      "effect": "allow",
      "reason": "Reviewed engineering changes only"
    },
    {
      "kind": "review_requirement",
      "rule_id": "review",
      "operation_kind": "filesystem.write",
      "resource_kind": "filesystem",
      "required": true,
      "reason": "Human approval is required"
    },
    {
      "kind": "authority_requirement",
      "rule_id": "proposer",
      "operation_kind": "filesystem.write",
      "resource_kind": "filesystem",
      "subject": "proposer",
      "required_role": "operation-proposer",
      "reason": "A bound proposer is required"
    },
    {
      "kind": "evidence_obligation",
      "rule_id": "audit",
      "operation_kind": "filesystem.write",
      "resource_kind": "filesystem",
      "obligation": "audit_reason",
      "reason": "Reviewer must explain approval"
    }
  ]
}
```
