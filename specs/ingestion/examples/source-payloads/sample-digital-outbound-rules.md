---
source_name: Sample Digital Outbound Rules
domain: digital
specialization: remote-publication
---

- RULE review_required action=publication target=external_sink rationale=outbound_requires_review
- RULE sink_restricted sink=external_untrusted severity=high
- RULE evidence_required evidence=destination_trace
- RULE evidence_required evidence=review_record
- RULE contract_required contract=approved_publication
- RULE authority_escalation role=risk_controller
- RULE publication_restriction mode=deny_without_contract
- RULE distribution_restriction mode=quarantine_untrusted_sink
