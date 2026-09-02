//! YAI - Rust operational data engine
//!
//! Purpose:
//!   Host Rust-side residue, projection, query, hot-state and record-plane
//!   modules.
//!
//! Ownership:
//!   Rust operational data engine module graph and public crate exports.
//!
//! Boundary:
//!   Does not own C ABI declarations or daemon process lifecycle.
//!
//! Status:
//!   active

pub mod admission;
pub mod case_policy;
pub mod compatibility;
pub mod context;
pub mod effect;
pub mod governance;
pub mod graph;
pub mod handoff;
pub mod journal;
pub mod memory;
pub mod projection;
pub mod provider_governance;
pub mod query;
pub mod reconcile;
pub mod record;
pub mod residency;
pub mod resource_control;
pub mod security;
pub mod store;
pub mod transition;
pub mod workflow;

#[cfg(test)]
mod tests {
    use crate::journal::Journal;
    use crate::projection::ProjectionSummary;
    use crate::record::{Record, RecordKind};

    #[test]
    fn append_record_and_build_projection_summary() {
        let mut journal = Journal::new();
        journal.append(Record::new(
            "record:attempt",
            RecordKind::Attempt,
            "op:file.write attempted",
        ));
        journal.append(Record::new(
            "record:receipt",
            RecordKind::Receipt,
            "receipt:blocked",
        ));

        let projection = ProjectionSummary::from_journal("operator", &journal);

        assert_eq!(journal.count(), 2);
        assert_eq!(projection.source_record_count, 2);
        assert!(projection.summary.contains("operator"));
    }

    #[test]
    fn parse_jsonl_and_build_projection_summary() {
        let record = Record::from_parts(
            "record:receipt",
            "case:new2-persistent",
            RecordKind::Receipt,
            "subject:repo-test",
            "op:file-write-test",
            "decision:new1",
            "receipt:file-write-blocked",
            "receipt:blocked",
        );
        let journal = Journal::from_jsonl_str(&record.to_jsonl());
        let projection = ProjectionSummary::from_journal("operator", &journal);

        assert_eq!(journal.count(), 1);
        assert_eq!(projection.case_ref, "case:new2-persistent");
        assert_eq!(projection.receipt_count, 1);
        assert_eq!(projection.subject_count, 1);
    }

    #[test]
    fn build_control_projection_summary() {
        let mut journal = Journal::new();
        journal.append(Record::from_parts(
            "record:rule",
            "case:new3-control",
            RecordKind::PolicyRule,
            "subject:repo-test",
            "",
            "",
            "",
            "rule:mutative_operation_requires_review",
        ));
        journal.append(Record::from_parts(
            "record:gate",
            "case:new3-control",
            RecordKind::GateResult,
            "subject:repo-test",
            "op:file-write-test",
            "",
            "",
            "gate:operation require_review",
        ));
        journal.append(Record::from_parts(
            "record:decision",
            "case:new3-control",
            RecordKind::Decision,
            "subject:repo-test",
            "op:file-write-test",
            "decision:new3",
            "",
            "decision:require_review",
        ));

        let projection = ProjectionSummary::from_journal("control", &journal);

        assert_eq!(projection.source_record_count, 3);
        assert_eq!(projection.policy_rule_count, 1);
        assert_eq!(projection.gate_count, 1);
        assert_eq!(projection.decision_count, 1);
        assert!(projection.summary.contains("rules:1"));
    }

    #[test]
    fn build_authority_projection_summary() {
        let mut journal = Journal::new();
        journal.append(Record::from_parts(
            "record:case-domain",
            "case:authority",
            RecordKind::CaseDomain,
            "subject:none",
            "",
            "",
            "",
            "case_domain scope:manual domain:nested_case",
        ));
        journal.append(Record::from_parts(
            "record:case-attachment",
            "case:authority",
            RecordKind::CaseAttachment,
            "subject:llm-provider",
            "",
            "",
            "",
            "case_attachment:model kind:model posture:bound",
        ));
        journal.append(Record::from_parts(
            "record:case-binding",
            "case:authority",
            RecordKind::CaseBinding,
            "subject:none",
            "",
            "",
            "",
            "case_binding:model kind:model posture:bound",
        ));
        journal.append(Record::from_parts(
            "record:projection-rule",
            "case:authority",
            RecordKind::ProjectionRule,
            "subject:llm-provider",
            "",
            "",
            "",
            "projection_rule:model-context-only",
        ));
        journal.append(Record::from_parts(
            "record:authority-scope",
            "case:authority",
            RecordKind::AuthorityScope,
            "subject:llm-provider",
            "",
            "",
            "",
            "authority_scope:model may:claim_proposal",
        ));
        journal.append(Record::from_parts(
            "record:model-interpretation",
            "case:authority",
            RecordKind::ModelInterpretation,
            "subject:llm-provider",
            "attempt:model-prompt",
            "",
            "",
            "model_interpretation:observed authority:not_authoritative_state",
        ));

        let projection = ProjectionSummary::from_journal("model", &journal);

        assert_eq!(projection.case_domain_count, 1);
        assert_eq!(projection.case_attachment_count, 1);
        assert_eq!(projection.case_binding_count, 1);
        assert_eq!(projection.projection_rule_count, 1);
        assert_eq!(projection.authority_scope_count, 1);
        assert_eq!(projection.model_interpretation_count, 1);
        assert!(projection.summary.contains("case_domains:1"));
        assert!(projection.summary.contains("case_attachments:1"));
        assert!(projection.summary.contains("case_bindings:1"));
        assert!(projection.summary.contains("projection_rules:1"));
        assert!(projection.summary.contains("authority_scopes:1"));
        assert!(projection.summary.contains("model_interpretations:1"));
    }

    #[test]
    fn build_filesystem_projection_summary() {
        let mut journal = Journal::new();
        journal.append(Record::from_parts(
            "record:fs-receipt",
            "case:new4-filesystem",
            RecordKind::FilesystemReceipt,
            "subject:file-input",
            "op:fs-read",
            "decision:allow",
            "receipt:fs-read",
            "effect:fs.read status:observed hash:abc",
        ));
        journal.append(Record::from_parts(
            "record:subject-state",
            "case:new4-filesystem",
            RecordKind::SubjectState,
            "subject:file-input",
            "",
            "",
            "receipt:fs-read",
            "state:observed hash:abc",
        ));

        let projection = ProjectionSummary::from_journal("filesystem", &journal);

        assert_eq!(projection.source_record_count, 2);
        assert_eq!(projection.filesystem_receipt_count, 1);
        assert_eq!(projection.subject_state_count, 1);
        assert_eq!(projection.effect_count, 1);
    }

    #[test]
    fn build_graph_projection_summary() {
        let mut journal = Journal::new();
        journal.append(Record::from_parts(
            "record:edge-case-subject",
            "case:new5-graph",
            RecordKind::GraphEdge,
            "subject:repo-test",
            "",
            "",
            "",
            "edge:case_binds_subject from:case:new5-graph to:subject:repo-test",
        ));
        journal.append(Record::from_parts(
            "record:edge-decision-op",
            "case:new5-graph",
            RecordKind::GraphEdge,
            "subject:repo-test",
            "op:fs-write",
            "decision:new5",
            "",
            "edge:decision_controls_op from:decision:new5 to:op:fs-write",
        ));

        let graph = crate::graph::GraphSummary::from_summaries(
            journal
                .records()
                .iter()
                .map(|record| record.summary.as_str()),
        );
        let projection = ProjectionSummary::from_journal("graph", &journal);

        assert_eq!(graph.graph_edges, 2);
        assert_eq!(graph.case_binds_subject, 1);
        assert_eq!(graph.decision_controls_op, 1);
        assert_eq!(projection.graph_edge_count, 2);
    }

    #[test]
    fn build_memory_projection_summary() {
        let mut journal = Journal::new();
        journal.append(Record::from_parts(
            "record:memory-candidate",
            "case:new6-memory",
            RecordKind::MemoryCandidate,
            "subject:repo-test",
            "op:fs-write-memory",
            "decision:new6",
            "receipt:fs-write-blocked",
            "memory:decision scope:subject freshness:fresh confidence:high",
        ));

        let memory = crate::memory::MemorySummary::from_journal(&journal);
        let projection = ProjectionSummary::from_journal("memory", &journal);

        assert_eq!(memory.memory_candidates, 1);
        assert_eq!(memory.decision, 1);
        assert_eq!(projection.memory_candidate_count, 1);
        assert_eq!(projection.decision_memory_candidate_count, 1);
    }

    #[test]
    fn build_reconcile_projection_summary() {
        let mut journal = Journal::new();
        journal.append(Record::from_parts(
            "divergence:denied-but-executed",
            "case:new7-reconcile",
            RecordKind::Divergence,
            "subject:repo-test",
            "op:dangerous-write",
            "decision:deny-write",
            "receipt:executed-conflict",
            "divergence:denied_but_executed severity:critical",
        ));
        journal.append(Record::from_parts(
            "reconciliation:denied-but-executed",
            "case:new7-reconcile",
            RecordKind::Reconciliation,
            "subject:repo-test",
            "",
            "",
            "",
            "reconcile:requires_review divergence:divergence:denied-but-executed",
        ));

        let reconcile = crate::reconcile::ReconcileSummary::from_journal(&journal);
        let projection = ProjectionSummary::from_journal("reconcile", &journal);

        assert_eq!(reconcile.divergences, 1);
        assert_eq!(reconcile.reconciliations, 1);
        assert_eq!(reconcile.critical, 1);
        assert_eq!(projection.divergence_count, 1);
        assert_eq!(projection.reconciliation_count, 1);
    }

    #[test]
    fn build_hardened_projection_summary() {
        let mut journal = Journal::new();
        journal.append(Record::from_parts(
            "projection-request:operator",
            "case:new8-projection",
            RecordKind::ProjectionRequest,
            "subject:repo-test",
            "",
            "",
            "",
            "projection_request:projection-request:operator consumer:operator kind:operator_summary scope:case",
        ));
        journal.append(Record::from_parts(
            "projection-result:operator",
            "case:new8-projection",
            RecordKind::ProjectionResult,
            "subject:repo-test",
            "",
            "",
            "",
            "projection_result:projection-result:operator consumer:operator kind:operator_summary redaction:none freshness:fresh source_records:4 source_receipts:1 source_memory:1 source_divergences:0",
        ));
        journal.append(Record::from_parts(
            "projection-request:model",
            "case:new8-projection",
            RecordKind::ProjectionRequest,
            "subject:repo-test",
            "",
            "",
            "",
            "projection_request:projection-request:model consumer:model kind:model_context scope:case",
        ));
        journal.append(Record::from_parts(
            "projection-result:model",
            "case:new8-projection",
            RecordKind::ProjectionResult,
            "subject:repo-test",
            "",
            "",
            "",
            "projection_result:projection-result:model consumer:model kind:model_context redaction:summary_only freshness:fresh source_records:4 source_receipts:1 source_memory:1 source_divergences:0",
        ));

        let projection = ProjectionSummary::from_journal("projection", &journal);

        assert_eq!(projection.projection_request_count, 2);
        assert_eq!(projection.projection_result_count, 2);
        assert_eq!(projection.operator_projection_count, 1);
        assert_eq!(projection.model_projection_count, 1);
        assert_eq!(projection.limited_projection_count, 1);
    }

    #[test]
    fn query_records_by_kind_and_case() {
        let mut journal = Journal::new();
        journal.append(Record::from_parts(
            "record:receipt",
            "case:new9-query",
            RecordKind::Receipt,
            "subject:repo-test",
            "op:query",
            "decision:query",
            "receipt:query",
            "receipt:blocked",
        ));
        journal.append(Record::from_parts(
            "record:memory",
            "case:new9-query",
            RecordKind::MemoryCandidate,
            "subject:repo-test",
            "op:query",
            "decision:query",
            "receipt:query",
            "memory:decision",
        ));
        journal.append(Record::from_parts(
            "record:other-case",
            "case:other",
            RecordKind::Receipt,
            "subject:repo-test",
            "",
            "",
            "",
            "receipt:other",
        ));

        let receipt_filter = crate::query::QueryFilter {
            record_kind: Some(RecordKind::Receipt),
            ..Default::default()
        };
        let case_filter = crate::query::QueryFilter {
            case_ref: Some("case:new9-query".to_string()),
            ..Default::default()
        };

        let receipts = crate::query::QueryResult::scan(&journal, &receipt_filter);
        let case_records = crate::query::QueryResult::scan(&journal, &case_filter);

        assert_eq!(receipts.records, 3);
        assert_eq!(receipts.matched, 2);
        assert_eq!(case_records.matched, 2);
        assert!(!case_records.truncated);
    }
}
