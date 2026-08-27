//! Canonical committed-transition and materialized CaseState semantics.
//!
//! `Transition` is immutable historical authority. `CaseState` is a
//! deterministic, rebuildable reduction of a Case's ordered transitions.
//! Provider output and review outcomes are represented as typed payloads; the
//! optional summary is presentation material and is never read by the reducer.

use serde::{Deserialize, Serialize};

pub const TRANSITION_SCHEMA: &str = "yai.transition.v1";
pub const CASE_STATE_SCHEMA: &str = "yai.case_state.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub schema: String,
    pub transition_id: String,
    pub case_id: String,
    pub sequence: u64,
    pub committed_at_unix_ms: u64,
    pub source: TransitionSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<TransitionScope>,
    #[serde(default)]
    pub causal_refs: Vec<String>,
    pub payload: TransitionPayload,
    #[serde(default)]
    pub provenance: Vec<TransitionProvenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingTransition {
    pub transition_id: String,
    pub case_id: String,
    pub expected_generation: u64,
    pub source: TransitionSource,
    pub scope: Option<TransitionScope>,
    pub causal_refs: Vec<String>,
    pub payload: TransitionPayload,
    pub provenance: Vec<TransitionProvenance>,
    pub summary: Option<String>,
}

impl PendingTransition {
    pub fn new(
        transition_id: impl Into<String>,
        case_id: impl Into<String>,
        expected_generation: u64,
        source: TransitionSource,
        payload: TransitionPayload,
    ) -> Self {
        Self {
            transition_id: transition_id.into(),
            case_id: case_id.into(),
            expected_generation,
            source,
            scope: None,
            causal_refs: Vec::new(),
            payload,
            provenance: Vec::new(),
            summary: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransitionSource {
    pub component: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
}

impl TransitionSource {
    pub fn component(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            participant_id: None,
            source_ref: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransitionScope {
    pub case_id: String,
    #[serde(default)]
    pub participant_refs: Vec<String>,
    #[serde(default)]
    pub resource_refs: Vec<String>,
    #[serde(default)]
    pub policy_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransitionProvenance {
    pub origin_schema: String,
    pub origin_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_record_id: Option<String>,
    pub promotion: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum TransitionPayload {
    CaseOpened {
        lifecycle: CaseLifecycle,
    },
    ParticipantBound {
        participant_id: String,
        role: String,
    },
    ParticipantAdmitted {
        participant_id: String,
        consumer: String,
        view_kind: String,
    },
    ProviderAttached {
        participant_id: String,
        provider_kind: String,
        base_url: String,
        model_id: String,
        credential_ref: String,
    },
    ProviderInvocationStarted {
        invocation_id: String,
        participant_id: String,
        provider_kind: String,
        model_id: String,
    },
    ProviderResultRecorded {
        result_id: String,
        invocation_id: String,
        provider_kind: String,
        model_id: String,
        output: String,
    },
    ModelInterpretationRecorded {
        interpretation_id: String,
        result_id: String,
        authority: InterpretationAuthority,
    },
    ReviewRequested {
        review: ReviewState,
    },
    ReviewResolved {
        review_id: String,
        attempt_id: String,
        resolution: ReviewResolution,
        reason: String,
        decision_ref: String,
        receipt_ref: String,
        carrier_attempted: bool,
        execution_performed: bool,
    },
}

impl TransitionPayload {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::CaseOpened { .. } => "case_opened",
            Self::ParticipantBound { .. } => "participant_bound",
            Self::ParticipantAdmitted { .. } => "participant_admitted",
            Self::ProviderAttached { .. } => "provider_attached",
            Self::ProviderInvocationStarted { .. } => "provider_invocation_started",
            Self::ProviderResultRecorded { .. } => "provider_result_recorded",
            Self::ModelInterpretationRecorded { .. } => "model_interpretation_recorded",
            Self::ReviewRequested { .. } => "review_requested",
            Self::ReviewResolved { .. } => "review_resolved",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseLifecycle {
    Open,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpretationAuthority {
    NonAuthoritative,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewResolution {
    PendingOperator,
    Approved,
    Denied,
    Deferred,
    Quarantined,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CaseState {
    pub schema: String,
    pub case_id: String,
    pub generation: u64,
    pub lifecycle: CaseLifecycle,
    #[serde(default)]
    pub participants: Vec<ParticipantState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderAttachmentState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_provider_invocation: Option<ProviderInvocationState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_provider_result: Option<ProviderResultState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_model_interpretation: Option<ModelInterpretationState>,
    #[serde(default)]
    pub reviews: Vec<ReviewState>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ParticipantState {
    pub participant_id: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub admitted_views: Vec<AdmittedView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdmittedView {
    pub consumer: String,
    pub view_kind: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderAttachmentState {
    pub participant_id: String,
    pub provider_kind: String,
    pub base_url: String,
    pub model_id: String,
    pub credential_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderInvocationState {
    pub invocation_id: String,
    pub participant_id: String,
    pub provider_kind: String,
    pub model_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderResultState {
    pub result_id: String,
    pub invocation_id: String,
    pub provider_kind: String,
    pub model_id: String,
    pub output_chars: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelInterpretationState {
    pub interpretation_id: String,
    pub result_id: String,
    pub authority: InterpretationAuthority,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReviewState {
    pub review_id: String,
    pub attempt_id: String,
    pub requested_by_participant: String,
    pub target_participant: String,
    pub reviewer_participant: String,
    pub operation_kind: String,
    pub carrier_family: String,
    pub target_display: String,
    pub sandbox_path: String,
    pub target_path: String,
    pub policy_reason: String,
    pub status: ReviewResolution,
    pub carrier_attempted: bool,
    pub execution_performed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_ref: Option<String>,
}

impl CaseState {
    pub fn new(case_id: impl Into<String>, lifecycle: CaseLifecycle) -> Self {
        Self {
            schema: CASE_STATE_SCHEMA.to_string(),
            case_id: case_id.into(),
            generation: 0,
            lifecycle,
            participants: Vec::new(),
            provider: None,
            last_provider_invocation: None,
            last_provider_result: None,
            last_model_interpretation: None,
            reviews: Vec::new(),
        }
    }

    pub fn reduce(&self, transition: &Transition) -> Result<Self, String> {
        transition.validate()?;
        if self.case_id != transition.case_id {
            return Err("case_state_case_mismatch".to_string());
        }
        if transition.sequence != self.generation + 1 {
            return Err(format!(
                "case_sequence_mismatch: expected={} actual={}",
                self.generation + 1,
                transition.sequence
            ));
        }

        let mut next = self.clone();
        match &transition.payload {
            TransitionPayload::CaseOpened { lifecycle } => {
                if transition.sequence != 1 {
                    return Err("case_opened_must_be_first_transition".to_string());
                }
                next.lifecycle = lifecycle.clone();
            }
            TransitionPayload::ParticipantBound {
                participant_id,
                role,
            } => {
                let participant = upsert_participant(&mut next.participants, participant_id);
                push_unique(&mut participant.roles, role);
            }
            TransitionPayload::ParticipantAdmitted {
                participant_id,
                consumer,
                view_kind,
            } => {
                let participant = upsert_participant(&mut next.participants, participant_id);
                let admitted = AdmittedView {
                    consumer: consumer.clone(),
                    view_kind: view_kind.clone(),
                };
                if !participant.admitted_views.contains(&admitted) {
                    participant.admitted_views.push(admitted);
                }
            }
            TransitionPayload::ProviderAttached {
                participant_id,
                provider_kind,
                base_url,
                model_id,
                credential_ref,
            } => {
                if !next
                    .participants
                    .iter()
                    .any(|participant| participant.participant_id == *participant_id)
                {
                    return Err("provider_participant_not_bound".to_string());
                }
                next.provider = Some(ProviderAttachmentState {
                    participant_id: participant_id.clone(),
                    provider_kind: provider_kind.clone(),
                    base_url: base_url.clone(),
                    model_id: model_id.clone(),
                    credential_ref: credential_ref.clone(),
                });
            }
            TransitionPayload::ProviderInvocationStarted {
                invocation_id,
                participant_id,
                provider_kind,
                model_id,
            } => {
                let Some(provider) = next.provider.as_ref() else {
                    return Err("provider_not_attached".to_string());
                };
                if provider.participant_id != *participant_id
                    || provider.provider_kind != *provider_kind
                    || provider.model_id != *model_id
                {
                    return Err("provider_invocation_attachment_mismatch".to_string());
                }
                next.last_provider_invocation = Some(ProviderInvocationState {
                    invocation_id: invocation_id.clone(),
                    participant_id: participant_id.clone(),
                    provider_kind: provider_kind.clone(),
                    model_id: model_id.clone(),
                });
            }
            TransitionPayload::ProviderResultRecorded {
                result_id,
                invocation_id,
                provider_kind,
                model_id,
                output,
            } => {
                let Some(invocation) = next.last_provider_invocation.as_ref() else {
                    return Err("provider_result_without_invocation".to_string());
                };
                if invocation.invocation_id != *invocation_id
                    || invocation.provider_kind != *provider_kind
                    || invocation.model_id != *model_id
                {
                    return Err("provider_result_invocation_mismatch".to_string());
                }
                next.last_provider_result = Some(ProviderResultState {
                    result_id: result_id.clone(),
                    invocation_id: invocation_id.clone(),
                    provider_kind: provider_kind.clone(),
                    model_id: model_id.clone(),
                    output_chars: output.chars().count(),
                });
            }
            TransitionPayload::ModelInterpretationRecorded {
                interpretation_id,
                result_id,
                authority,
            } => {
                let Some(result) = next.last_provider_result.as_ref() else {
                    return Err("interpretation_without_provider_result".to_string());
                };
                if result.result_id != *result_id {
                    return Err("interpretation_result_mismatch".to_string());
                }
                next.last_model_interpretation = Some(ModelInterpretationState {
                    interpretation_id: interpretation_id.clone(),
                    result_id: result_id.clone(),
                    authority: authority.clone(),
                });
            }
            TransitionPayload::ReviewRequested { review } => {
                if next
                    .reviews
                    .iter()
                    .any(|existing| existing.review_id == review.review_id)
                {
                    return Err("review_already_exists".to_string());
                }
                next.reviews.push(review.clone());
            }
            TransitionPayload::ReviewResolved {
                review_id,
                attempt_id,
                resolution,
                decision_ref,
                receipt_ref,
                carrier_attempted,
                execution_performed,
                ..
            } => {
                let Some(review) = next
                    .reviews
                    .iter_mut()
                    .find(|review| review.review_id == *review_id)
                else {
                    return Err("review_not_found".to_string());
                };
                if review.attempt_id != *attempt_id {
                    return Err("review_attempt_mismatch".to_string());
                }
                review.status = resolution.clone();
                review.decision_ref = Some(decision_ref.clone());
                review.receipt_ref = Some(receipt_ref.clone());
                review.carrier_attempted = *carrier_attempted;
                review.execution_performed = *execution_performed;
            }
        }
        next.generation = transition.sequence;
        Ok(next)
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|error| format!("case_state_encode_failed: {error}"))
    }

    pub fn from_json(value: &str) -> Result<Self, String> {
        let state: Self = serde_json::from_str(value)
            .map_err(|error| format!("case_state_decode_failed: {error}"))?;
        if state.schema != CASE_STATE_SCHEMA {
            return Err(format!("unsupported_case_state_schema: {}", state.schema));
        }
        Ok(state)
    }
}

impl Transition {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != TRANSITION_SCHEMA {
            return Err(format!("unsupported_transition_schema: {}", self.schema));
        }
        require_value("transition_id", &self.transition_id)?;
        require_value("case_id", &self.case_id)?;
        require_value("source.component", &self.source.component)?;
        if self.sequence == 0 {
            return Err("transition_sequence_must_be_positive".to_string());
        }
        if let Some(scope) = &self.scope {
            if scope.case_id != self.case_id {
                return Err("transition_scope_case_mismatch".to_string());
            }
        }
        for causal_ref in &self.causal_refs {
            require_value("causal_ref", causal_ref)?;
        }
        match &self.payload {
            TransitionPayload::CaseOpened { .. } => {
                if self.sequence != 1 {
                    return Err("case_opened_must_be_first_transition".to_string());
                }
            }
            TransitionPayload::ParticipantBound {
                participant_id,
                role,
            } => {
                require_value("participant_id", participant_id)?;
                require_value("role", role)?;
            }
            TransitionPayload::ParticipantAdmitted {
                participant_id,
                consumer,
                view_kind,
            } => {
                require_value("participant_id", participant_id)?;
                require_value("consumer", consumer)?;
                require_value("view_kind", view_kind)?;
            }
            TransitionPayload::ProviderAttached {
                participant_id,
                provider_kind,
                base_url,
                model_id,
                credential_ref,
            } => {
                require_value("participant_id", participant_id)?;
                require_value("provider_kind", provider_kind)?;
                require_value("base_url", base_url)?;
                require_value("model_id", model_id)?;
                require_value("credential_ref", credential_ref)?;
            }
            TransitionPayload::ProviderInvocationStarted {
                invocation_id,
                participant_id,
                provider_kind,
                model_id,
            } => {
                require_value("invocation_id", invocation_id)?;
                require_value("participant_id", participant_id)?;
                require_value("provider_kind", provider_kind)?;
                require_value("model_id", model_id)?;
            }
            TransitionPayload::ProviderResultRecorded {
                result_id,
                invocation_id,
                provider_kind,
                model_id,
                ..
            } => {
                require_value("result_id", result_id)?;
                require_value("invocation_id", invocation_id)?;
                require_value("provider_kind", provider_kind)?;
                require_value("model_id", model_id)?;
                require_causal_ref(&self.causal_refs, invocation_id, "provider_invocation")?;
            }
            TransitionPayload::ModelInterpretationRecorded {
                interpretation_id,
                result_id,
                ..
            } => {
                require_value("interpretation_id", interpretation_id)?;
                require_value("result_id", result_id)?;
                require_causal_ref(&self.causal_refs, result_id, "provider_result")?;
            }
            TransitionPayload::ReviewRequested { review } => {
                review.validate()?;
                require_causal_ref(&self.causal_refs, &review.attempt_id, "review_attempt")?;
            }
            TransitionPayload::ReviewResolved {
                review_id,
                attempt_id,
                decision_ref,
                receipt_ref,
                ..
            } => {
                require_value("review_id", review_id)?;
                require_value("attempt_id", attempt_id)?;
                require_value("decision_ref", decision_ref)?;
                require_value("receipt_ref", receipt_ref)?;
                require_causal_ref(&self.causal_refs, review_id, "review_request")?;
                require_causal_ref(&self.causal_refs, attempt_id, "review_attempt")?;
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|error| format!("transition_encode_failed: {error}"))
    }

    pub fn from_json(value: &str) -> Result<Self, String> {
        let transition: Self = serde_json::from_str(value)
            .map_err(|error| format!("transition_decode_failed: {error}"))?;
        transition.validate()?;
        Ok(transition)
    }
}

impl ReviewState {
    fn validate(&self) -> Result<(), String> {
        require_value("review_id", &self.review_id)?;
        require_value("attempt_id", &self.attempt_id)?;
        require_value("requested_by_participant", &self.requested_by_participant)?;
        require_value("target_participant", &self.target_participant)?;
        require_value("reviewer_participant", &self.reviewer_participant)?;
        require_value("operation_kind", &self.operation_kind)?;
        require_value("carrier_family", &self.carrier_family)?;
        require_value("target_display", &self.target_display)?;
        require_value("sandbox_path", &self.sandbox_path)?;
        require_value("target_path", &self.target_path)?;
        require_value("policy_reason", &self.policy_reason)
    }
}

pub fn replay_case(case_id: &str, transitions: &[Transition]) -> Result<CaseState, String> {
    if transitions.is_empty() {
        return Err("cannot_replay_empty_case_history".to_string());
    }
    let first = &transitions[0];
    if first.case_id != case_id {
        return Err("replay_case_mismatch".to_string());
    }
    let TransitionPayload::CaseOpened { lifecycle } = &first.payload else {
        return Err("case_history_must_start_with_case_opened".to_string());
    };
    let mut state = CaseState::new(case_id, lifecycle.clone());
    for transition in transitions {
        state = state.reduce(transition)?;
    }
    Ok(state)
}

fn upsert_participant<'a>(
    participants: &'a mut Vec<ParticipantState>,
    participant_id: &str,
) -> &'a mut ParticipantState {
    if let Some(index) = participants
        .iter()
        .position(|participant| participant.participant_id == participant_id)
    {
        return &mut participants[index];
    }
    participants.push(ParticipantState {
        participant_id: participant_id.to_string(),
        roles: Vec::new(),
        admitted_views: Vec::new(),
    });
    participants.last_mut().expect("participant just inserted")
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

fn require_value(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        Err(format!("missing_required_field: {field}"))
    } else {
        Ok(())
    }
}

fn require_causal_ref(refs: &[String], required: &str, role: &str) -> Result<(), String> {
    if refs.iter().any(|reference| reference == required) {
        Ok(())
    } else {
        Err(format!("missing_causal_reference: {role}:{required}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transition(sequence: u64, payload: TransitionPayload) -> Transition {
        Transition {
            schema: TRANSITION_SCHEMA.to_string(),
            transition_id: format!("transition:{sequence}"),
            case_id: "case:test".to_string(),
            sequence,
            committed_at_unix_ms: sequence,
            source: TransitionSource::component("test"),
            scope: None,
            causal_refs: Vec::new(),
            payload,
            provenance: Vec::new(),
            summary: Some("presentation only".to_string()),
        }
    }

    #[test]
    fn transition_roundtrip_and_unknown_field_policy() {
        let value = transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        );
        let encoded = value.to_json().expect("encode transition");
        assert_eq!(Transition::from_json(&encoded).expect("decode"), value);

        let with_unknown = encoded.replacen('{', "{\"future_field\":\"ignored\",", 1);
        assert_eq!(
            Transition::from_json(&with_unknown).expect("additive field accepted"),
            value
        );
    }

    #[test]
    fn version_and_unknown_kind_are_rejected() {
        let encoded = transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        )
        .to_json()
        .expect("encode");
        assert!(
            Transition::from_json(&encoded.replace(TRANSITION_SCHEMA, "yai.transition.v2"))
                .unwrap_err()
                .contains("unsupported_transition_schema")
        );
        assert!(
            Transition::from_json(&encoded.replace("case_opened", "future_kind"))
                .unwrap_err()
                .contains("transition_decode_failed")
        );
    }

    #[test]
    fn closure_is_mechanical_not_summary_based() {
        let mut result = transition(
            2,
            TransitionPayload::ProviderResultRecorded {
                result_id: "result:1".to_string(),
                invocation_id: "invocation:1".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:1".to_string(),
                output: "hello".to_string(),
            },
        );
        result.summary = Some("complete:true invocation:wrong".to_string());
        assert!(result
            .validate()
            .unwrap_err()
            .contains("missing_causal_reference"));
        result.causal_refs.push("invocation:1".to_string());
        result.validate().expect("typed closure is complete");
    }
}
