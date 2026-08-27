//! YAI - record model
//!
//! Purpose:
//!   Define normalized operational records and their JSON encodings.
//!
//! Ownership:
//!   Record kind taxonomy and record envelope serialization used by the Rust
//!   engine.
//!
//! Boundary:
//!   Does not own LMDB storage, journal file IO or graph semantics.
//!
//! Status:
//!   active

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordKind {
    Case,
    SubjectBinding,
    Attempt,
    Decision,
    Receipt,
    Projection,
    PolicyRule,
    GateResult,
    DecisionBasis,
    Obligation,
    ReceiptRequirement,
    CarrierRequest,
    EffectReceipt,
    FilesystemReceipt,
    SubjectState,
    GraphEdge,
    Reconstruction,
    MemoryCandidate,
    Divergence,
    Reconciliation,
    ProjectionRequest,
    ProjectionResult,
    QueryResult,
    ProjectionRule,
    AuthorityScope,
    ModelInterpretation,
    CaseDomain,
    CaseAttachment,
    CaseBinding,
    InteractionThread,
    InteractionTurn,
    ParticipantViewFrame,
    ReviewRequest,
    ReviewDecision,
    ControlPending,
}

impl RecordKind {
    pub const ALL: [Self; 35] = [
        Self::Case,
        Self::SubjectBinding,
        Self::Attempt,
        Self::Decision,
        Self::Receipt,
        Self::Projection,
        Self::PolicyRule,
        Self::GateResult,
        Self::DecisionBasis,
        Self::Obligation,
        Self::ReceiptRequirement,
        Self::CarrierRequest,
        Self::EffectReceipt,
        Self::FilesystemReceipt,
        Self::SubjectState,
        Self::GraphEdge,
        Self::Reconstruction,
        Self::MemoryCandidate,
        Self::Divergence,
        Self::Reconciliation,
        Self::ProjectionRequest,
        Self::ProjectionResult,
        Self::QueryResult,
        Self::ProjectionRule,
        Self::AuthorityScope,
        Self::ModelInterpretation,
        Self::CaseDomain,
        Self::CaseAttachment,
        Self::CaseBinding,
        Self::InteractionThread,
        Self::InteractionTurn,
        Self::ParticipantViewFrame,
        Self::ReviewRequest,
        Self::ReviewDecision,
        Self::ControlPending,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Case => "case",
            Self::SubjectBinding => "subject_binding",
            Self::Attempt => "attempt",
            Self::Decision => "decision",
            Self::Receipt => "receipt",
            Self::Projection => "projection",
            Self::PolicyRule => "policy_rule",
            Self::GateResult => "gate_result",
            Self::DecisionBasis => "decision_basis",
            Self::Obligation => "obligation",
            Self::ReceiptRequirement => "receipt_requirement",
            Self::CarrierRequest => "carrier_request",
            Self::EffectReceipt => "effect_receipt",
            Self::FilesystemReceipt => "filesystem_receipt",
            Self::SubjectState => "subject_state",
            Self::GraphEdge => "graph_edge",
            Self::Reconstruction => "reconstruction",
            Self::MemoryCandidate => "memory_candidate",
            Self::Divergence => "divergence",
            Self::Reconciliation => "reconciliation",
            Self::ProjectionRequest => "projection_request",
            Self::ProjectionResult => "projection_result",
            Self::QueryResult => "query_result",
            Self::ProjectionRule => "projection_rule",
            Self::AuthorityScope => "authority_scope",
            Self::ModelInterpretation => "model_interpretation",
            Self::CaseDomain => "case_domain",
            Self::CaseAttachment => "case_attachment",
            Self::CaseBinding => "case_binding",
            Self::InteractionThread => "interaction_thread",
            Self::InteractionTurn => "interaction_turn",
            Self::ParticipantViewFrame => "participant_view_frame",
            Self::ReviewRequest => "review_request",
            Self::ReviewDecision => "review_decision",
            Self::ControlPending => "control_pending",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "case" => Some(Self::Case),
            "subject_binding" => Some(Self::SubjectBinding),
            "attempt" => Some(Self::Attempt),
            "decision" => Some(Self::Decision),
            "receipt" => Some(Self::Receipt),
            "projection" => Some(Self::Projection),
            "policy_rule" => Some(Self::PolicyRule),
            "gate_result" => Some(Self::GateResult),
            "decision_basis" => Some(Self::DecisionBasis),
            "obligation" => Some(Self::Obligation),
            "receipt_requirement" => Some(Self::ReceiptRequirement),
            "carrier_request" => Some(Self::CarrierRequest),
            "effect_receipt" => Some(Self::EffectReceipt),
            "filesystem_receipt" => Some(Self::FilesystemReceipt),
            "subject_state" => Some(Self::SubjectState),
            "graph_edge" => Some(Self::GraphEdge),
            "reconstruction" => Some(Self::Reconstruction),
            "memory_candidate" => Some(Self::MemoryCandidate),
            "divergence" => Some(Self::Divergence),
            "reconciliation" => Some(Self::Reconciliation),
            "projection_request" => Some(Self::ProjectionRequest),
            "projection_result" => Some(Self::ProjectionResult),
            "query_result" => Some(Self::QueryResult),
            "projection_rule" => Some(Self::ProjectionRule),
            "authority_scope" => Some(Self::AuthorityScope),
            "model_interpretation" => Some(Self::ModelInterpretation),
            "case_domain" => Some(Self::CaseDomain),
            "case_attachment" => Some(Self::CaseAttachment),
            "case_binding" => Some(Self::CaseBinding),
            "interaction_thread" => Some(Self::InteractionThread),
            "interaction_turn" => Some(Self::InteractionTurn),
            "participant_view_frame" => Some(Self::ParticipantViewFrame),
            "review_request" => Some(Self::ReviewRequest),
            "review_decision" => Some(Self::ReviewDecision),
            "control_pending" => Some(Self::ControlPending),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    pub id: String,
    pub case_ref: String,
    pub kind: RecordKind,
    pub subject_ref: String,
    pub attempt_id: String,
    pub decision_id: String,
    pub receipt_id: String,
    pub summary: String,
}

impl Record {
    pub fn new(id: impl Into<String>, kind: RecordKind, summary: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            case_ref: String::new(),
            kind,
            subject_ref: String::new(),
            attempt_id: String::new(),
            decision_id: String::new(),
            receipt_id: String::new(),
            summary: summary.into(),
        }
    }

    pub fn from_parts(
        id: impl Into<String>,
        case_ref: impl Into<String>,
        kind: RecordKind,
        subject_ref: impl Into<String>,
        attempt_id: impl Into<String>,
        decision_id: impl Into<String>,
        receipt_id: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            case_ref: case_ref.into(),
            kind,
            subject_ref: subject_ref.into(),
            attempt_id: attempt_id.into(),
            decision_id: decision_id.into(),
            receipt_id: receipt_id.into(),
            summary: summary.into(),
        }
    }

    pub fn to_jsonl(&self) -> String {
        format!(
            "{{\"schema\":\"yai.store.record.v0\",\"record_id\":\"{}\",\"case_ref\":\"{}\",\"record_kind\":\"{}\",\"subject_ref\":\"{}\",\"attempt_id\":\"{}\",\"decision_id\":\"{}\",\"receipt_id\":\"{}\",\"summary\":\"{}\"}}\n",
            escape_json(&self.id),
            escape_json(&self.case_ref),
            self.kind.as_str(),
            escape_json(&self.subject_ref),
            escape_json(&self.attempt_id),
            escape_json(&self.decision_id),
            escape_json(&self.receipt_id),
            escape_json(&self.summary)
        )
    }

    pub fn to_record_plane_json(&self, source_ref: &str) -> String {
        let subject_refs = if self.subject_ref.is_empty() || self.subject_ref == "subject:none" {
            String::new()
        } else {
            format!("\"{}\"", escape_json(&self.subject_ref))
        };
        format!(
            "{{\"schema\":\"yai.record.v1\",\"record_id\":\"{}\",\"record_kind\":\"{}\",\"case_ref\":\"{}\",\"subject_refs\":[{}],\"created_at_unix_ms\":0,\"source\":{{\"plane\":\"journal\",\"ref\":\"{}\"}},\"provenance_refs\":[],\"payload\":{{\"summary\":\"{}\",\"subject_ref\":\"{}\",\"attempt_id\":\"{}\",\"decision_id\":\"{}\",\"receipt_id\":\"{}\"}}}}",
            escape_json(&self.id),
            self.kind.as_str(),
            escape_json(&self.case_ref),
            subject_refs,
            escape_json(source_ref),
            escape_json(&self.summary),
            escape_json(&self.subject_ref),
            escape_json(&self.attempt_id),
            escape_json(&self.decision_id),
            escape_json(&self.receipt_id)
        )
    }

    pub fn from_jsonl(line: &str) -> Option<Self> {
        let schema = extract_json_string(line, "schema")?;
        if schema != "yai.store.record.v0" {
            return None;
        }
        let kind = RecordKind::from_str(&extract_json_string(line, "record_kind")?)?;
        Some(Self::from_parts(
            extract_json_string(line, "record_id")?,
            extract_json_string(line, "case_ref").unwrap_or_default(),
            kind,
            extract_json_string(line, "subject_ref").unwrap_or_default(),
            extract_json_string(line, "attempt_id").unwrap_or_default(),
            extract_json_string(line, "decision_id").unwrap_or_default(),
            extract_json_string(line, "receipt_id").unwrap_or_default(),
            extract_json_string(line, "summary").unwrap_or_default(),
        ))
    }
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn extract_json_string(line: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\":\"");
    let start = line.find(&pattern)? + pattern.len();
    let mut result = String::new();
    let mut chars = line[start..].chars();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => return Some(result),
            '\\' => {
                let escaped = chars.next()?;
                match escaped {
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    other => result.push(other),
                }
            }
            other => result.push(other),
        }
    }
    None
}
