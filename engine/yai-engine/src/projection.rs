use crate::compatibility::{legacy_summary_is, legacy_summary_value};
use crate::journal::Journal;
use crate::record::RecordKind;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionSummary {
    pub consumer: String,
    pub case_ref: String,
    pub summary: String,
    pub source_record_count: usize,
    pub receipt_count: usize,
    pub subject_count: usize,
    pub decision_count: usize,
    pub policy_rule_count: usize,
    pub gate_count: usize,
    pub obligation_count: usize,
    pub receipt_requirement_count: usize,
    pub filesystem_receipt_count: usize,
    pub subject_state_count: usize,
    pub effect_count: usize,
    pub graph_edge_count: usize,
    pub reconstruction_count: usize,
    pub memory_candidate_count: usize,
    pub operational_memory_candidate_count: usize,
    pub decision_memory_candidate_count: usize,
    pub subject_memory_candidate_count: usize,
    pub error_memory_candidate_count: usize,
    pub recovery_memory_candidate_count: usize,
    pub divergence_count: usize,
    pub reconciliation_count: usize,
    pub critical_divergence_count: usize,
    pub warning_divergence_count: usize,
    pub projection_request_count: usize,
    pub projection_result_count: usize,
    pub operator_projection_count: usize,
    pub model_projection_count: usize,
    pub audit_projection_count: usize,
    pub limited_projection_count: usize,
    pub query_result_count: usize,
    pub projection_rule_count: usize,
    pub authority_scope_count: usize,
    pub model_interpretation_count: usize,
    pub case_domain_count: usize,
    pub case_attachment_count: usize,
    pub case_binding_count: usize,
    pub interaction_thread_count: usize,
    pub interaction_turn_count: usize,
    pub participant_view_frame_count: usize,
}

impl ProjectionSummary {
    pub fn from_journal(consumer: impl Into<String>, journal: &Journal) -> Self {
        let consumer = consumer.into();
        let source_record_count = journal.count();
        let case_ref = journal
            .records()
            .iter()
            .find(|record| !record.case_ref.is_empty())
            .map(|record| record.case_ref.clone())
            .unwrap_or_default();
        let receipt_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::Receipt)
            .count();
        let decision_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::Decision)
            .count();
        let policy_rule_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::PolicyRule)
            .count();
        let gate_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::GateResult)
            .count();
        let obligation_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::Obligation)
            .count();
        let receipt_requirement_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::ReceiptRequirement)
            .count();
        let filesystem_receipt_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::FilesystemReceipt)
            .count();
        let subject_state_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::SubjectState)
            .count();
        let effect_count = journal
            .records()
            .iter()
            .filter(|record| {
                matches!(
                    record.kind,
                    RecordKind::Receipt | RecordKind::EffectReceipt | RecordKind::FilesystemReceipt
                )
            })
            .count();
        let graph_edge_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::GraphEdge)
            .count();
        let reconstruction_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::Reconstruction)
            .count();
        let memory_records = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::MemoryCandidate)
            .collect::<Vec<_>>();
        let memory_candidate_count = memory_records.len();
        let operational_memory_candidate_count = memory_records
            .iter()
            .filter(|record| legacy_summary_is(&record.summary, "memory", "operational"))
            .count();
        let decision_memory_candidate_count = memory_records
            .iter()
            .filter(|record| legacy_summary_is(&record.summary, "memory", "decision"))
            .count();
        let subject_memory_candidate_count = memory_records
            .iter()
            .filter(|record| legacy_summary_is(&record.summary, "memory", "subject"))
            .count();
        let error_memory_candidate_count = memory_records
            .iter()
            .filter(|record| legacy_summary_is(&record.summary, "memory", "error"))
            .count();
        let recovery_memory_candidate_count = memory_records
            .iter()
            .filter(|record| legacy_summary_is(&record.summary, "memory", "recovery"))
            .count();
        let subject_count = journal
            .records()
            .iter()
            .filter(|record| !record.subject_ref.is_empty() && record.subject_ref != "subject:none")
            .map(|record| record.subject_ref.as_str())
            .collect::<BTreeSet<_>>()
            .len();
        let divergence_records = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::Divergence)
            .collect::<Vec<_>>();
        let divergence_count = divergence_records.len();
        let reconciliation_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::Reconciliation)
            .count();
        let critical_divergence_count = divergence_records
            .iter()
            .filter(|record| legacy_summary_is(&record.summary, "severity", "critical"))
            .count();
        let warning_divergence_count = divergence_records
            .iter()
            .filter(|record| legacy_summary_is(&record.summary, "severity", "warning"))
            .count();
        let projection_request_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::ProjectionRequest)
            .count();
        let projection_result_records = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::ProjectionResult)
            .collect::<Vec<_>>();
        let projection_result_count = projection_result_records.len();
        let operator_projection_count = projection_result_records
            .iter()
            .filter(|record| legacy_summary_is(&record.summary, "consumer", "operator"))
            .count();
        let model_projection_count = projection_result_records
            .iter()
            .filter(|record| legacy_summary_is(&record.summary, "consumer", "model"))
            .count();
        let audit_projection_count = projection_result_records
            .iter()
            .filter(|record| legacy_summary_is(&record.summary, "consumer", "audit"))
            .count();
        let limited_projection_count = projection_result_records
            .iter()
            .filter(|record| {
                matches!(
                    legacy_summary_value(&record.summary, "redaction").as_deref(),
                    Some("summary_only" | "refs_only" | "redacted" | "blocked")
                )
            })
            .count();
        let query_result_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::QueryResult)
            .count();
        let projection_rule_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::ProjectionRule)
            .count();
        let authority_scope_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::AuthorityScope)
            .count();
        let model_interpretation_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::ModelInterpretation)
            .count();
        let case_domain_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::CaseDomain)
            .count();
        let case_attachment_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::CaseAttachment)
            .count();
        let case_binding_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::CaseBinding)
            .count();
        let interaction_thread_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::InteractionThread)
            .count();
        let interaction_turn_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::InteractionTurn)
            .count();
        let participant_view_frame_count = journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::ParticipantViewFrame)
            .count();
        Self {
            summary: format!(
                "projection:{consumer} records:{source_record_count} case_domains:{case_domain_count} case_attachments:{case_attachment_count} case_bindings:{case_binding_count} interaction_threads:{interaction_thread_count} interaction_turns:{interaction_turn_count} participant_view_frames:{participant_view_frame_count} decisions:{decision_count} rules:{policy_rule_count} projection_rules:{projection_rule_count} authority_scopes:{authority_scope_count} model_interpretations:{model_interpretation_count} gates:{gate_count} obligations:{obligation_count} receipt_requirements:{receipt_requirement_count} filesystem_receipts:{filesystem_receipt_count} subject_states:{subject_state_count} effects:{effect_count} graph_edges:{graph_edge_count} reconstructions:{reconstruction_count} memory_candidates:{memory_candidate_count} divergences:{divergence_count} reconciliations:{reconciliation_count} projection_requests:{projection_request_count} projection_results:{projection_result_count} query_results:{query_result_count}"
            ),
            consumer,
            case_ref,
            source_record_count,
            receipt_count,
            subject_count,
            decision_count,
            policy_rule_count,
            gate_count,
            obligation_count,
            receipt_requirement_count,
            filesystem_receipt_count,
            subject_state_count,
            effect_count,
            graph_edge_count,
            reconstruction_count,
            memory_candidate_count,
            operational_memory_candidate_count,
            decision_memory_candidate_count,
            subject_memory_candidate_count,
            error_memory_candidate_count,
            recovery_memory_candidate_count,
            divergence_count,
            reconciliation_count,
            critical_divergence_count,
            warning_divergence_count,
            projection_request_count,
            projection_result_count,
            operator_projection_count,
            model_projection_count,
            audit_projection_count,
            limited_projection_count,
            query_result_count,
            projection_rule_count,
            authority_scope_count,
            model_interpretation_count,
            case_domain_count,
            case_attachment_count,
            case_binding_count,
            interaction_thread_count,
            interaction_turn_count,
            participant_view_frame_count,
        }
    }
}
