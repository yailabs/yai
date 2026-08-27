//! Historical record compatibility boundary.
//!
//! Legacy JSONL and record-plane envelopes are accepted as input evidence.
//! Summary token parsing is deliberately confined here: callers may consume
//! explicitly named compatibility fields, but canonical transitions and
//! CaseState never derive semantics from summary text.

use crate::record::RecordKind;
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

pub const LEGACY_JSONL_SCHEMA: &str = "yai.store.record.v0";
pub const LEGACY_RECORD_PLANE_SCHEMA: &str = "yai.record.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LegacyPromotion {
    LosslessStructural,
    WithCompatibilityMetadata,
}

impl LegacyPromotion {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LosslessStructural => "losslessly_promoted",
            Self::WithCompatibilityMetadata => "promoted_with_compatibility_metadata",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyRecord {
    pub schema: String,
    pub record_id: String,
    pub case_ref: String,
    pub record_kind: RecordKind,
    pub subject_ref: String,
    pub attempt_id: String,
    pub decision_id: String,
    pub receipt_id: String,
    pub summary: String,
    pub source_ref: String,
    pub summary_fields: BTreeMap<String, String>,
    pub raw_json: String,
    pub promotion: LegacyPromotion,
}

impl LegacyRecord {
    pub fn compatibility_value(&self, key: &str) -> Option<&str> {
        self.summary_fields.get(key).map(String::as_str)
    }

    pub fn compatibility_value_or<'a>(&'a self, key: &str, fallback: &'a str) -> &'a str {
        self.compatibility_value(key).unwrap_or(fallback)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpaqueLegacyPayload {
    pub schema: String,
    pub record_id: Option<String>,
    pub record_kind: Option<String>,
    pub reason: String,
    pub raw_json: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MalformedLegacyPayload {
    pub reason: String,
    pub raw_json: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LegacyDecodeOutcome {
    Promoted(LegacyRecord),
    PreservedOpaque(OpaqueLegacyPayload),
    RejectedMalformed(MalformedLegacyPayload),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LegacyCorpusReport {
    pub lines_total: usize,
    pub losslessly_promoted: usize,
    pub promoted_with_metadata: usize,
    pub preserved_opaque: usize,
    pub rejected_malformed: usize,
    pub repeated_record_ids: usize,
    pub entries: Vec<LegacyCorpusEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyCorpusEntry {
    pub line_number: usize,
    pub disposition: String,
    pub schema: String,
    pub record_id: String,
    pub record_kind: String,
    pub reason: String,
    pub raw_json: String,
}

pub fn inspect_legacy_jsonl(contents: &str) -> LegacyCorpusReport {
    let mut report = LegacyCorpusReport::default();
    let mut seen_ids = HashSet::new();
    for (index, raw_json) in contents.lines().enumerate() {
        let line_number = index + 1;
        report.lines_total += 1;
        let entry = match decode_legacy_record(raw_json) {
            LegacyDecodeOutcome::Promoted(record) => {
                if !seen_ids.insert(record.record_id.clone()) {
                    report.repeated_record_ids += 1;
                }
                match record.promotion {
                    LegacyPromotion::LosslessStructural => report.losslessly_promoted += 1,
                    LegacyPromotion::WithCompatibilityMetadata => {
                        report.promoted_with_metadata += 1
                    }
                }
                LegacyCorpusEntry {
                    line_number,
                    disposition: record.promotion.as_str().to_string(),
                    schema: record.schema,
                    record_id: record.record_id,
                    record_kind: record.record_kind.as_str().to_string(),
                    reason: String::new(),
                    raw_json: raw_json.to_string(),
                }
            }
            LegacyDecodeOutcome::PreservedOpaque(payload) => {
                report.preserved_opaque += 1;
                if let Some(record_id) = payload.record_id.as_ref() {
                    if !seen_ids.insert(record_id.clone()) {
                        report.repeated_record_ids += 1;
                    }
                }
                LegacyCorpusEntry {
                    line_number,
                    disposition: "preserved_opaque".to_string(),
                    schema: payload.schema,
                    record_id: payload.record_id.unwrap_or_default(),
                    record_kind: payload.record_kind.unwrap_or_default(),
                    reason: payload.reason,
                    raw_json: payload.raw_json,
                }
            }
            LegacyDecodeOutcome::RejectedMalformed(payload) => {
                report.rejected_malformed += 1;
                LegacyCorpusEntry {
                    line_number,
                    disposition: "rejected_malformed".to_string(),
                    schema: String::new(),
                    record_id: String::new(),
                    record_kind: String::new(),
                    reason: payload.reason,
                    raw_json: payload.raw_json,
                }
            }
        };
        report.entries.push(entry);
    }
    report
}

pub fn decode_legacy_record(raw_json: &str) -> LegacyDecodeOutcome {
    let value: Value = match serde_json::from_str(raw_json) {
        Ok(value) => value,
        Err(error) => {
            return LegacyDecodeOutcome::RejectedMalformed(MalformedLegacyPayload {
                reason: format!("invalid_json: {error}"),
                raw_json: raw_json.to_string(),
            });
        }
    };
    let Some(object) = value.as_object() else {
        return malformed(raw_json, "root_is_not_object");
    };
    let schema = string_member(object.get("schema")).unwrap_or_default();
    if schema != LEGACY_JSONL_SCHEMA && schema != LEGACY_RECORD_PLANE_SCHEMA {
        return LegacyDecodeOutcome::PreservedOpaque(OpaqueLegacyPayload {
            schema,
            record_id: string_member(object.get("record_id")),
            record_kind: string_member(object.get("record_kind")),
            reason: "unsupported_or_future_schema".to_string(),
            raw_json: raw_json.to_string(),
        });
    }

    let record_id = string_member(object.get("record_id")).unwrap_or_default();
    let case_ref = string_member(object.get("case_ref")).unwrap_or_default();
    let record_kind_name = string_member(object.get("record_kind")).unwrap_or_default();
    if record_id.is_empty() || case_ref.is_empty() || record_kind_name.is_empty() {
        return malformed(raw_json, "missing_required_envelope_field");
    }
    let Some(record_kind) = RecordKind::from_str(&record_kind_name) else {
        return LegacyDecodeOutcome::PreservedOpaque(OpaqueLegacyPayload {
            schema,
            record_id: Some(record_id),
            record_kind: Some(record_kind_name),
            reason: "unknown_future_record_kind".to_string(),
            raw_json: raw_json.to_string(),
        });
    };

    let (subject_ref, attempt_id, decision_id, receipt_id, summary, source_ref) =
        if schema == LEGACY_JSONL_SCHEMA {
            (
                string_member(object.get("subject_ref")).unwrap_or_default(),
                string_member(object.get("attempt_id")).unwrap_or_default(),
                string_member(object.get("decision_id")).unwrap_or_default(),
                string_member(object.get("receipt_id")).unwrap_or_default(),
                string_member(object.get("summary")).unwrap_or_default(),
                String::new(),
            )
        } else {
            let Some(payload) = object.get("payload").and_then(Value::as_object) else {
                return malformed(raw_json, "record_plane_payload_missing");
            };
            let source_ref = object
                .get("source")
                .and_then(Value::as_object)
                .and_then(|source| string_member(source.get("ref")))
                .unwrap_or_default();
            (
                string_member(payload.get("subject_ref")).unwrap_or_default(),
                string_member(payload.get("attempt_id")).unwrap_or_default(),
                string_member(payload.get("decision_id")).unwrap_or_default(),
                string_member(payload.get("receipt_id")).unwrap_or_default(),
                string_member(payload.get("summary")).unwrap_or_default(),
                source_ref,
            )
        };
    let summary_fields = parse_legacy_summary_fields(&summary);
    let promotion = if summary_fields.is_empty() {
        LegacyPromotion::LosslessStructural
    } else {
        LegacyPromotion::WithCompatibilityMetadata
    };
    LegacyDecodeOutcome::Promoted(LegacyRecord {
        schema,
        record_id,
        case_ref,
        record_kind,
        subject_ref,
        attempt_id,
        decision_id,
        receipt_id,
        summary,
        source_ref,
        summary_fields,
        raw_json: raw_json.to_string(),
        promotion,
    })
}

pub fn parse_legacy_summary_fields(summary: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    for token in summary.split_whitespace() {
        let Some((key, value)) = token.split_once(':') else {
            continue;
        };
        if key.is_empty() || value.is_empty() {
            continue;
        }
        fields
            .entry(key.to_string())
            .or_insert_with(|| value.to_string());
    }
    fields
}

pub fn legacy_summary_value(summary: &str, key: &str) -> Option<String> {
    parse_legacy_summary_fields(summary).remove(key)
}

pub fn legacy_summary_is(summary: &str, key: &str, expected: &str) -> bool {
    legacy_summary_value(summary, key).as_deref() == Some(expected)
}

pub fn legacy_summary_has_marker(summary: &str, marker: &str) -> bool {
    summary.split_whitespace().any(|token| token == marker)
}

fn malformed(raw_json: &str, reason: &str) -> LegacyDecodeOutcome {
    LegacyDecodeOutcome::RejectedMalformed(MalformedLegacyPayload {
        reason: reason.to_string(),
        raw_json: raw_json.to_string(),
    })
}

fn string_member(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Record;

    const C_RECORD_KINDS: [&str; 32] = [
        "case",
        "subject_binding",
        "attempt",
        "decision",
        "receipt",
        "projection",
        "policy_rule",
        "gate_result",
        "decision_basis",
        "obligation",
        "receipt_requirement",
        "carrier_request",
        "effect_receipt",
        "filesystem_receipt",
        "subject_state",
        "graph_edge",
        "reconstruction",
        "memory_candidate",
        "divergence",
        "reconciliation",
        "projection_request",
        "projection_result",
        "query_result",
        "projection_rule",
        "authority_scope",
        "model_interpretation",
        "case_domain",
        "case_attachment",
        "case_binding",
        "interaction_thread",
        "interaction_turn",
        "participant_view_frame",
    ];

    #[test]
    fn corpus_reads_all_rust_legacy_kinds() {
        for (index, kind) in RecordKind::ALL.into_iter().enumerate() {
            let record = Record::from_parts(
                format!("record:{index}"),
                "case:corpus",
                kind.clone(),
                "subject:corpus",
                "attempt:corpus",
                "decision:corpus",
                "receipt:corpus",
                "status:observed optional:value",
            );
            let LegacyDecodeOutcome::Promoted(decoded) =
                decode_legacy_record(record.to_jsonl().trim())
            else {
                panic!("Rust legacy kind {} was not promoted", kind.as_str());
            };
            assert_eq!(decoded.record_kind, kind);
            assert_eq!(decoded.compatibility_value("status"), Some("observed"));
        }
    }

    #[test]
    fn corpus_freezes_all_c_legacy_kinds_and_drift() {
        for kind in C_RECORD_KINDS {
            assert!(
                RecordKind::from_str(kind).is_some(),
                "C kind {kind} drifted"
            );
        }
        let rust_only: Vec<_> = RecordKind::ALL
            .iter()
            .map(RecordKind::as_str)
            .filter(|kind| !C_RECORD_KINDS.contains(kind))
            .collect();
        assert_eq!(
            rust_only,
            vec!["review_request", "review_decision", "control_pending"]
        );
    }

    #[test]
    fn record_plane_optional_fields_and_source_are_readable() {
        let record = Record::from_parts(
            "record:plane",
            "case:plane",
            RecordKind::ModelInterpretation,
            "subject:none",
            "",
            "",
            "",
            "",
        );
        let raw = record.to_record_plane_json("journal.jsonl#9");
        let LegacyDecodeOutcome::Promoted(decoded) = decode_legacy_record(&raw) else {
            panic!("record plane was not promoted");
        };
        assert_eq!(decoded.source_ref, "journal.jsonl#9");
        assert_eq!(decoded.subject_ref, "subject:none");
        assert_eq!(decoded.promotion, LegacyPromotion::LosslessStructural);
    }

    #[test]
    fn unknown_future_kind_and_schema_are_preserved_opaque() {
        let future_kind = r#"{"schema":"yai.store.record.v0","record_id":"future:1","case_ref":"case:future","record_kind":"future_kind"}"#;
        assert!(matches!(
            decode_legacy_record(future_kind),
            LegacyDecodeOutcome::PreservedOpaque(OpaqueLegacyPayload { reason, .. })
                if reason == "unknown_future_record_kind"
        ));
        let future_schema = r#"{"schema":"yai.store.record.v9","record_id":"future:2","case_ref":"case:future","record_kind":"case"}"#;
        assert!(matches!(
            decode_legacy_record(future_schema),
            LegacyDecodeOutcome::PreservedOpaque(OpaqueLegacyPayload { reason, .. })
                if reason == "unsupported_or_future_schema"
        ));
    }

    #[test]
    fn malformed_and_missing_required_fields_are_rejected() {
        assert!(matches!(
            decode_legacy_record("{bad"),
            LegacyDecodeOutcome::RejectedMalformed(_)
        ));
        assert!(matches!(
            decode_legacy_record(r#"{"schema":"yai.store.record.v0","record_kind":"case"}"#),
            LegacyDecodeOutcome::RejectedMalformed(_)
        ));
    }

    #[test]
    fn repeated_ids_remain_separate_input_events() {
        let first = r#"{"schema":"yai.store.record.v0","record_id":"record:repeat","case_ref":"case:repeat","record_kind":"review_request","summary":"status:pending_operator"}"#;
        let second = r#"{"schema":"yai.store.record.v0","record_id":"record:repeat","case_ref":"case:repeat","record_kind":"review_request","summary":"status:approved"}"#;
        let outcomes = [decode_legacy_record(first), decode_legacy_record(second)];
        let statuses: Vec<_> = outcomes
            .iter()
            .map(|outcome| match outcome {
                LegacyDecodeOutcome::Promoted(record) => {
                    record.compatibility_value("status").unwrap_or("")
                }
                _ => "",
            })
            .collect();
        assert_eq!(statuses, vec!["pending_operator", "approved"]);
    }

    #[test]
    fn corpus_report_classifies_without_collapsing_repeated_ids() {
        let contents = concat!(
            "{\"schema\":\"yai.store.record.v0\",\"record_id\":\"record:repeat\",\"case_ref\":\"case:repeat\",\"record_kind\":\"case\",\"summary\":\"\"}\n",
            "{\"schema\":\"yai.store.record.v0\",\"record_id\":\"record:repeat\",\"case_ref\":\"case:repeat\",\"record_kind\":\"review_request\",\"summary\":\"status:pending_operator\"}\n",
            "{\"schema\":\"yai.store.record.v0\",\"record_id\":\"future:1\",\"case_ref\":\"case:repeat\",\"record_kind\":\"future_kind\"}\n",
            "{bad\n"
        );
        let report = inspect_legacy_jsonl(contents);
        assert_eq!(report.lines_total, 4);
        assert_eq!(report.losslessly_promoted, 1);
        assert_eq!(report.promoted_with_metadata, 1);
        assert_eq!(report.preserved_opaque, 1);
        assert_eq!(report.rejected_malformed, 1);
        assert_eq!(report.repeated_record_ids, 1);
        assert_eq!(report.entries.len(), 4);
    }
}
