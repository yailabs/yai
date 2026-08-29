//! Policy-driven, operation-specific admission.
//!
//! This is the single boundary that combines a Ready EffectivePolicy with the
//! mechanical resource envelope, Case-bound participant roles and canonical
//! evidence. It emits immutable decision material; it does not own policy
//! authoring/materialization, review persistence, or carrier execution.

use crate::case_policy::{EffectivePolicy, EffectivePolicyRule, EffectiveRuleProvenance};
use crate::effect::{
    build_policy_decision, digest_bytes, path_within_prefix, Decision, DecisionOutcome, Operation,
    OperationKind, OperationOrigin,
};
use crate::governance::{AuthoritySubject, EvidenceObligationKind, PolicyEffect};
use crate::transition::{
    CaseState, ResourceAttachmentState, ReviewAction, ReviewActionKind, ReviewResolution,
    ReviewState, REVIEW_REQUEST_SCHEMA,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const DECISION_BASIS_SCHEMA: &str = "yai.decision_basis.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MechanicalPosture {
    Satisfied,
    TargetOutsideAttachment,
    PayloadExceedsAttachment,
    AttachmentMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationRestrictionPosture {
    ExplicitAllow,
    ExplicitDeny,
    NoApplicableAllow,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationStatus {
    Satisfied,
    MissingDeny,
    MissingRequiresReview,
    RequiredAtExecution,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObligationEvaluation {
    pub obligation: EvidenceObligationKind,
    pub status: ObligationStatus,
    pub evidence_refs: Vec<String>,
    pub contributing_rules: Vec<EffectiveRuleProvenance>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityEvaluation {
    pub subject: AuthoritySubject,
    pub participant_id: Option<String>,
    pub required_roles: Vec<String>,
    pub observed_roles: Vec<String>,
    pub eligible_participant_ids: Vec<String>,
    pub satisfied: bool,
    pub contributing_rules: Vec<EffectiveRuleProvenance>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionEvidenceRequirement {
    PreObservation,
    PostObservation,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionBasis {
    pub schema: String,
    pub basis_id: String,
    pub integrity_digest: String,
    pub case_id: String,
    pub evaluated_case_generation: u64,
    pub operation_id: String,
    pub operation_digest: String,
    pub operation_kind: String,
    pub proposer_participant_id: String,
    pub resource_attachment_id: String,
    pub resource_kind: String,
    pub effective_policy_id: String,
    pub effective_policy_digest: String,
    pub materializer_version: String,
    pub policy_binding_refs: Vec<String>,
    pub policy_artifact_refs: Vec<String>,
    pub matched_rule_refs: Vec<String>,
    pub contributing_provenance: Vec<EffectiveRuleProvenance>,
    pub mechanical_posture: MechanicalPosture,
    pub operation_restriction: OperationRestrictionPosture,
    pub review_required: bool,
    pub authority: Vec<AuthorityEvaluation>,
    pub obligations: Vec<ObligationEvaluation>,
    pub final_posture: DecisionOutcome,
    pub final_reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_action_ref: Option<String>,
}

#[derive(Serialize)]
struct DecisionBasisDigestMaterial<'a> {
    schema: &'a str,
    case_id: &'a str,
    evaluated_case_generation: u64,
    operation_id: &'a str,
    operation_digest: &'a str,
    operation_kind: &'a str,
    proposer_participant_id: &'a str,
    resource_attachment_id: &'a str,
    resource_kind: &'a str,
    effective_policy_id: &'a str,
    effective_policy_digest: &'a str,
    materializer_version: &'a str,
    policy_binding_refs: &'a [String],
    policy_artifact_refs: &'a [String],
    matched_rule_refs: &'a [String],
    contributing_provenance: &'a [EffectiveRuleProvenance],
    mechanical_posture: &'a MechanicalPosture,
    operation_restriction: &'a OperationRestrictionPosture,
    review_required: bool,
    authority: &'a [AuthorityEvaluation],
    obligations: &'a [ObligationEvaluation],
    final_posture: &'a DecisionOutcome,
    final_reason: &'a str,
    review_action_ref: &'a Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvidenceContext {
    pub canonical_provider_result_id: Option<String>,
    pub canonical_provider_invocation_id: Option<String>,
    pub review_action_id: Option<String>,
    pub review_reason: Option<String>,
}

pub fn evaluate_filesystem_admission(
    operation: &Operation,
    state: &CaseState,
    resource: &ResourceAttachmentState,
    effective_policy: &EffectivePolicy,
    evidence: &EvidenceContext,
) -> Result<Decision, String> {
    operation.validate()?;
    if state.case_id != operation.case_id
        || state.generation == 0
        || resource.attachment_id != operation.resource_attachment_id
        || effective_policy.case_id != operation.case_id
    {
        return Err("admission_case_operation_input_mismatch".to_string());
    }

    let selector_matches = |operation_kind: &str, resource_kind: &Option<String>| {
        operation_kind == "filesystem.write"
            && resource_kind
                .as_deref()
                .is_none_or(|kind| kind == "filesystem")
    };
    let mechanical_posture = if resource.attachment_id != operation.resource_attachment_id {
        MechanicalPosture::AttachmentMismatch
    } else if !path_within_prefix(
        &resource.allowed_write_prefix,
        &operation.filesystem_write.relative_path,
    ) {
        MechanicalPosture::TargetOutsideAttachment
    } else if operation.filesystem_write.content_bytes > resource.max_write_bytes {
        MechanicalPosture::PayloadExceedsAttachment
    } else {
        MechanicalPosture::Satisfied
    };

    let mut allow = false;
    let mut deny = false;
    let mut review_required = false;
    let mut matched_rule_refs = BTreeSet::new();
    let mut provenance = Vec::new();
    let mut proposer_roles = BTreeSet::new();
    let mut reviewer_roles = BTreeSet::new();
    let mut proposer_contributions = Vec::new();
    let mut reviewer_contributions = Vec::new();
    let mut obligation_rules = Vec::new();

    for rule in &effective_policy.rules {
        match rule {
            EffectivePolicyRule::OperationRestriction {
                operation_kind,
                resource_kind,
                effect,
                contributions,
                ..
            } if selector_matches(operation_kind, resource_kind) => {
                allow |= *effect == PolicyEffect::Allow;
                deny |= *effect == PolicyEffect::Deny;
                add_contributions(contributions, &mut matched_rule_refs, &mut provenance);
            }
            EffectivePolicyRule::ReviewRequirement {
                operation_kind,
                resource_kind,
                required,
                contributions,
                ..
            } if selector_matches(operation_kind, resource_kind) => {
                review_required |= *required;
                add_contributions(contributions, &mut matched_rule_refs, &mut provenance);
            }
            EffectivePolicyRule::EvidenceObligation {
                operation_kind,
                resource_kind,
                obligation,
                contributions,
                ..
            } if selector_matches(operation_kind, resource_kind) => {
                obligation_rules.push((obligation.clone(), contributions.clone()));
                add_contributions(contributions, &mut matched_rule_refs, &mut provenance);
            }
            EffectivePolicyRule::AuthorityRequirement {
                operation_kind,
                resource_kind,
                subject,
                required_roles,
                contributions,
                ..
            } if selector_matches(operation_kind, resource_kind) => {
                match subject {
                    AuthoritySubject::Proposer => {
                        proposer_roles.extend(required_roles.iter().cloned());
                        proposer_contributions.extend(contributions.iter().cloned());
                    }
                    AuthoritySubject::Reviewer => {
                        reviewer_roles.extend(required_roles.iter().cloned());
                        reviewer_contributions.extend(contributions.iter().cloned());
                    }
                }
                add_contributions(contributions, &mut matched_rule_refs, &mut provenance);
            }
            _ => {}
        }
    }

    sort_provenance(&mut provenance);
    sort_provenance(&mut proposer_contributions);
    sort_provenance(&mut reviewer_contributions);
    let participant = state
        .participants
        .iter()
        .find(|participant| participant.participant_id == operation.participant_id);
    let observed_proposer_roles = participant
        .map(|participant| sorted_unique(participant.roles.clone()))
        .unwrap_or_default();
    let required_proposer_roles = proposer_roles.into_iter().collect::<Vec<_>>();
    let proposer_satisfied = participant.is_some()
        && required_proposer_roles
            .iter()
            .all(|role| observed_proposer_roles.contains(role));
    let reviewer_required_roles = reviewer_roles.into_iter().collect::<Vec<_>>();
    let eligible_reviewers = eligible_participants(state, &reviewer_required_roles);
    let mut authority = vec![AuthorityEvaluation {
        subject: AuthoritySubject::Proposer,
        participant_id: Some(operation.participant_id.clone()),
        required_roles: required_proposer_roles,
        observed_roles: observed_proposer_roles,
        eligible_participant_ids: Vec::new(),
        satisfied: proposer_satisfied,
        contributing_rules: proposer_contributions,
    }];
    if !reviewer_required_roles.is_empty() || review_required {
        authority.push(AuthorityEvaluation {
            subject: AuthoritySubject::Reviewer,
            participant_id: None,
            required_roles: reviewer_required_roles,
            observed_roles: Vec::new(),
            satisfied: !eligible_reviewers.is_empty(),
            eligible_participant_ids: eligible_reviewers,
            contributing_rules: reviewer_contributions,
        });
    }

    let mut obligations = obligation_rules
        .into_iter()
        .map(|(obligation, contributions)| {
            let (status, evidence_refs) = match obligation {
                EvidenceObligationKind::SourceProvenance => {
                    source_provenance_status(operation, evidence)
                }
                EvidenceObligationKind::AuditReason => {
                    if let (Some(action_id), Some(reason)) =
                        (&evidence.review_action_id, &evidence.review_reason)
                    {
                        if !reason.trim().is_empty() {
                            (ObligationStatus::Satisfied, vec![action_id.clone()])
                        } else {
                            (ObligationStatus::MissingRequiresReview, Vec::new())
                        }
                    } else {
                        (ObligationStatus::MissingRequiresReview, Vec::new())
                    }
                }
                EvidenceObligationKind::PreObservation => {
                    (ObligationStatus::RequiredAtExecution, Vec::new())
                }
                EvidenceObligationKind::PostObservation => {
                    (ObligationStatus::RequiredAtExecution, Vec::new())
                }
            };
            ObligationEvaluation {
                obligation,
                status,
                evidence_refs,
                contributing_rules: contributions,
            }
        })
        .collect::<Vec<_>>();
    obligations.sort_by_key(|obligation| format!("{:?}", obligation.obligation));
    let audit_needs_review = obligations
        .iter()
        .any(|obligation| obligation.status == ObligationStatus::MissingRequiresReview);
    review_required |= audit_needs_review;

    let operation_restriction = if deny {
        OperationRestrictionPosture::ExplicitDeny
    } else if allow {
        OperationRestrictionPosture::ExplicitAllow
    } else {
        OperationRestrictionPosture::NoApplicableAllow
    };
    let (final_posture, final_reason) = if mechanical_posture != MechanicalPosture::Satisfied {
        (DecisionOutcome::Deny, "resource_mechanical_envelope_denied")
    } else if !proposer_satisfied {
        (DecisionOutcome::Deny, "proposer_not_case_role_eligible")
    } else if operation_restriction == OperationRestrictionPosture::ExplicitDeny {
        (DecisionOutcome::Deny, "applicable_policy_deny")
    } else if operation_restriction == OperationRestrictionPosture::NoApplicableAllow {
        (DecisionOutcome::Deny, "no_applicable_allow_rule")
    } else if obligations
        .iter()
        .any(|obligation| obligation.status == ObligationStatus::MissingDeny)
    {
        (DecisionOutcome::Deny, "required_admission_evidence_missing")
    } else if review_required
        && authority
            .iter()
            .find(|evaluation| evaluation.subject == AuthoritySubject::Reviewer)
            .is_none_or(|evaluation| !evaluation.satisfied)
    {
        (DecisionOutcome::Deny, "eligible_reviewer_not_available")
    } else if review_required {
        (
            DecisionOutcome::RequireReview,
            "policy_requires_human_review",
        )
    } else {
        (DecisionOutcome::Allow, "policy_admission_satisfied")
    };

    let basis = seal_basis(DecisionBasis {
        schema: DECISION_BASIS_SCHEMA.to_string(),
        basis_id: String::new(),
        integrity_digest: String::new(),
        case_id: operation.case_id.clone(),
        evaluated_case_generation: state.generation,
        operation_id: operation.operation_id.clone(),
        operation_digest: operation.operation_digest.clone(),
        operation_kind: operation_kind_name(&operation.kind).to_string(),
        proposer_participant_id: operation.participant_id.clone(),
        resource_attachment_id: resource.attachment_id.clone(),
        resource_kind: "filesystem".to_string(),
        effective_policy_id: effective_policy.effective_policy_id.clone(),
        effective_policy_digest: effective_policy.semantic_digest.clone(),
        materializer_version: effective_policy.materializer_version.clone(),
        policy_binding_refs: sorted_unique(effective_policy.binding_ids.clone()),
        policy_artifact_refs: sorted_unique(effective_policy.artifact_ids.clone()),
        matched_rule_refs: matched_rule_refs.into_iter().collect(),
        contributing_provenance: provenance,
        mechanical_posture,
        operation_restriction,
        review_required,
        authority,
        obligations,
        final_posture,
        final_reason: final_reason.to_string(),
        review_action_ref: evidence.review_action_id.clone(),
    })?;
    build_policy_decision(operation, basis, final_reason)
}

pub fn resolve_policy_review_decision(
    operation: &Operation,
    state: &CaseState,
    resource: &ResourceAttachmentState,
    effective_policy: &EffectivePolicy,
    review: &ReviewState,
    action: &ReviewAction,
) -> Result<Decision, String> {
    action.validate_integrity()?;
    if review.schema != REVIEW_REQUEST_SCHEMA
        || review.case_id != state.case_id
        || review.operation_id != operation.operation_id
        || review.operation_digest != operation.operation_digest
        || review.resource_attachment_id != resource.attachment_id
        || review.latest_action_id.as_deref() != Some(action.action_id.as_str())
        || review.decision_basis_id.is_empty()
        || review.effective_policy_id != effective_policy.effective_policy_id
        || review.effective_policy_digest != effective_policy.semantic_digest
        || action.review_id != review.review_id
        || action.operation_id != operation.operation_id
        || action.case_id != operation.case_id
        || !reviewer_is_eligible(state, review, &action.reviewer_participant_id)
        || state.last_operation.as_ref().is_none_or(|current| {
            current.operation_id != operation.operation_id
                || current.operation_digest != operation.operation_digest
        })
    {
        return Err("review_policy_basis_stale_or_ineligible".to_string());
    }
    let evidence = EvidenceContext {
        canonical_provider_result_id: match &operation.origin {
            OperationOrigin::ProviderResult {
                provider_result_id, ..
            } => Some(provider_result_id.clone()),
            _ => None,
        },
        canonical_provider_invocation_id: match &operation.origin {
            OperationOrigin::ProviderResult {
                provider_invocation_id,
                ..
            } => Some(provider_invocation_id.clone()),
            _ => None,
        },
        review_action_id: Some(action.action_id.clone()),
        review_reason: Some(action.reason.clone()),
    };
    let mut decision =
        evaluate_filesystem_admission(operation, state, resource, effective_policy, &evidence)?;
    let basis = decision
        .decision_basis
        .as_mut()
        .ok_or_else(|| "review_effective_decision_basis_missing".to_string())?;
    match (&action.action, &review.status) {
        (ReviewActionKind::Approve, ReviewResolution::Approved) => {
            if basis.operation_restriction != OperationRestrictionPosture::ExplicitAllow
                || !basis.admission_obligations_satisfied()
            {
                return Err("review_approval_cannot_override_policy_or_evidence".to_string());
            }
            basis.review_required = false;
            basis.final_posture = DecisionOutcome::Allow;
            basis.final_reason = "eligible_human_review_approved".to_string();
        }
        (ReviewActionKind::Deny, ReviewResolution::Denied) => {
            basis.review_required = false;
            basis.final_posture = DecisionOutcome::Deny;
            basis.final_reason = "eligible_human_review_denied".to_string();
        }
        (ReviewActionKind::Defer, _) => {
            return Err("deferred_review_has_no_effective_decision".to_string())
        }
        _ => return Err("review_action_state_mismatch".to_string()),
    }
    *basis = seal_basis(basis.clone())?;
    build_policy_decision(operation, basis.clone(), &basis.final_reason)
}

pub fn build_policy_review_request(
    operation: &Operation,
    decision: &Decision,
    current_case_generation: u64,
) -> Result<ReviewState, String> {
    decision.validate_integrity()?;
    let basis = decision
        .decision_basis
        .as_ref()
        .ok_or_else(|| "policy_review_decision_basis_missing".to_string())?;
    if decision.outcome != DecisionOutcome::RequireReview
        || current_case_generation != decision.decided_at_case_generation + 1
    {
        return Err("review_request_requires_committed_policy_review_decision".to_string());
    }
    let reviewer_roles = basis
        .authority
        .iter()
        .find(|evaluation| evaluation.subject == AuthoritySubject::Reviewer)
        .map(|evaluation| evaluation.required_roles.clone())
        .unwrap_or_default();
    if reviewer_roles.is_empty() {
        return Err("policy_review_requires_reviewer_roles".to_string());
    }
    ReviewState {
        review_id: String::new(),
        schema: REVIEW_REQUEST_SCHEMA.to_string(),
        integrity_digest: String::new(),
        case_id: operation.case_id.clone(),
        operation_id: operation.operation_id.clone(),
        operation_digest: operation.operation_digest.clone(),
        initial_decision_id: decision.decision_id.clone(),
        decision_basis_id: basis.basis_id.clone(),
        decision_basis_digest: basis.integrity_digest.clone(),
        effective_policy_id: basis.effective_policy_id.clone(),
        effective_policy_digest: basis.effective_policy_digest.clone(),
        policy_binding_refs: basis.policy_binding_refs.clone(),
        policy_artifact_refs: basis.policy_artifact_refs.clone(),
        required_reviewer_roles: reviewer_roles,
        resource_attachment_id: operation.resource_attachment_id.clone(),
        normalized_target: operation.filesystem_write.relative_path.clone(),
        created_at_generation: current_case_generation,
        latest_action_id: None,
        effective_decision_id: None,
        attempt_id: String::new(),
        requested_by_participant: operation.participant_id.clone(),
        target_participant: String::new(),
        reviewer_participant: String::new(),
        operation_kind: String::new(),
        carrier_family: String::new(),
        target_display: String::new(),
        sandbox_path: String::new(),
        target_path: String::new(),
        policy_reason: decision.reason.clone(),
        status: ReviewResolution::Pending,
        carrier_attempted: false,
        execution_performed: false,
        decision_ref: None,
        receipt_ref: None,
    }
    .seal_policy_integrity()
}

pub fn reviewer_is_eligible(state: &CaseState, review: &ReviewState, participant_id: &str) -> bool {
    state
        .participants
        .iter()
        .find(|participant| participant.participant_id == participant_id)
        .is_some_and(|participant| {
            !review.required_reviewer_roles.is_empty()
                && review
                    .required_reviewer_roles
                    .iter()
                    .all(|role| participant.roles.contains(role))
        })
}

impl DecisionBasis {
    pub fn validate_integrity(&self) -> Result<(), String> {
        if self.schema != DECISION_BASIS_SCHEMA
            || self.case_id.is_empty()
            || self.operation_id.is_empty()
            || self.operation_digest.is_empty()
            || self.resource_attachment_id.is_empty()
            || self.effective_policy_id.is_empty()
            || self.effective_policy_digest.is_empty()
            || self.policy_binding_refs.is_empty()
            || self.policy_artifact_refs.is_empty()
            || self.final_reason.is_empty()
        {
            return Err("invalid_decision_basis_contract".to_string());
        }
        let digest = basis_digest(self)?;
        if self.integrity_digest != digest
            || self.basis_id != format!("decision-basis:{}", digest_suffix(&digest))
        {
            return Err("decision_basis_integrity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn admission_obligations_satisfied(&self) -> bool {
        self.obligations.iter().all(|obligation| {
            matches!(
                obligation.status,
                ObligationStatus::Satisfied | ObligationStatus::RequiredAtExecution
            )
        })
    }

    pub fn execution_evidence_requirements(&self) -> Vec<ExecutionEvidenceRequirement> {
        let mut values = self
            .obligations
            .iter()
            .filter_map(|obligation| match obligation.obligation {
                EvidenceObligationKind::PreObservation
                    if obligation.status == ObligationStatus::RequiredAtExecution =>
                {
                    Some(ExecutionEvidenceRequirement::PreObservation)
                }
                EvidenceObligationKind::PostObservation
                    if obligation.status == ObligationStatus::RequiredAtExecution =>
                {
                    Some(ExecutionEvidenceRequirement::PostObservation)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        values.sort();
        values.dedup();
        values
    }
}

fn seal_basis(mut basis: DecisionBasis) -> Result<DecisionBasis, String> {
    basis.basis_id.clear();
    basis.integrity_digest.clear();
    basis.policy_binding_refs = sorted_unique(basis.policy_binding_refs);
    basis.policy_artifact_refs = sorted_unique(basis.policy_artifact_refs);
    basis.matched_rule_refs = sorted_unique(basis.matched_rule_refs);
    basis.contributing_provenance.sort_by(|left, right| {
        (&left.artifact_id, &left.policy_ir_rule_id)
            .cmp(&(&right.artifact_id, &right.policy_ir_rule_id))
    });
    let digest = basis_digest(&basis)?;
    basis.basis_id = format!("decision-basis:{}", digest_suffix(&digest));
    basis.integrity_digest = digest;
    basis.validate_integrity()?;
    Ok(basis)
}

fn basis_digest(basis: &DecisionBasis) -> Result<String, String> {
    serde_json::to_vec(&DecisionBasisDigestMaterial {
        schema: &basis.schema,
        case_id: &basis.case_id,
        evaluated_case_generation: basis.evaluated_case_generation,
        operation_id: &basis.operation_id,
        operation_digest: &basis.operation_digest,
        operation_kind: &basis.operation_kind,
        proposer_participant_id: &basis.proposer_participant_id,
        resource_attachment_id: &basis.resource_attachment_id,
        resource_kind: &basis.resource_kind,
        effective_policy_id: &basis.effective_policy_id,
        effective_policy_digest: &basis.effective_policy_digest,
        materializer_version: &basis.materializer_version,
        policy_binding_refs: &basis.policy_binding_refs,
        policy_artifact_refs: &basis.policy_artifact_refs,
        matched_rule_refs: &basis.matched_rule_refs,
        contributing_provenance: &basis.contributing_provenance,
        mechanical_posture: &basis.mechanical_posture,
        operation_restriction: &basis.operation_restriction,
        review_required: basis.review_required,
        authority: &basis.authority,
        obligations: &basis.obligations,
        final_posture: &basis.final_posture,
        final_reason: &basis.final_reason,
        review_action_ref: &basis.review_action_ref,
    })
    .map(|value| digest_bytes(&value))
    .map_err(|error| format!("decision_basis_digest_encode_failed: {error}"))
}

fn source_provenance_status(
    operation: &Operation,
    evidence: &EvidenceContext,
) -> (ObligationStatus, Vec<String>) {
    match &operation.origin {
        OperationOrigin::ProviderResult {
            provider_result_id,
            provider_invocation_id,
        } if evidence.canonical_provider_result_id.as_deref() == Some(provider_result_id)
            && evidence.canonical_provider_invocation_id.as_deref()
                == Some(provider_invocation_id) =>
        {
            (
                ObligationStatus::Satisfied,
                vec![provider_result_id.clone(), provider_invocation_id.clone()],
            )
        }
        _ => (ObligationStatus::MissingDeny, Vec::new()),
    }
}

fn eligible_participants(state: &CaseState, required_roles: &[String]) -> Vec<String> {
    if required_roles.is_empty() {
        return Vec::new();
    }
    let mut result = state
        .participants
        .iter()
        .filter(|participant| {
            required_roles
                .iter()
                .all(|role| participant.roles.contains(role))
        })
        .map(|participant| participant.participant_id.clone())
        .collect::<Vec<_>>();
    result.sort();
    result
}

fn add_contributions(
    contributions: &[EffectiveRuleProvenance],
    rule_refs: &mut BTreeSet<String>,
    provenance: &mut Vec<EffectiveRuleProvenance>,
) {
    for contribution in contributions {
        rule_refs.insert(format!(
            "{}#{}",
            contribution.artifact_id, contribution.policy_ir_rule_id
        ));
        provenance.push(contribution.clone());
    }
}

fn sort_provenance(values: &mut Vec<EffectiveRuleProvenance>) {
    values.sort_by(|left, right| {
        (&left.artifact_id, &left.policy_ir_rule_id, &left.fact_refs).cmp(&(
            &right.artifact_id,
            &right.policy_ir_rule_id,
            &right.fact_refs,
        ))
    });
    values.dedup();
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn operation_kind_name(kind: &OperationKind) -> &'static str {
    match kind {
        OperationKind::FilesystemWrite => "filesystem.write",
    }
}

fn digest_suffix(digest: &str) -> &str {
    let suffix = digest.strip_prefix("sha256:").unwrap_or(digest);
    &suffix[..suffix.len().min(32)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_policy::{EffectivePolicy, EffectivePolicyRule};
    use crate::effect::{
        issue_policy_execution_grant, normalize_filesystem_write_candidate, NormalizationContext,
        DECISION_SCHEMA, EXECUTION_GRANT_SCHEMA,
    };
    use crate::transition::{
        build_review_action, CaseLifecycle, OperationState, ParticipantState, ResourceKind,
        ReviewActionKind, ReviewRequirement,
    };

    fn resource() -> ResourceAttachmentState {
        ResourceAttachmentState {
            attachment_id: "workspace".to_string(),
            kind: ResourceKind::Filesystem,
            allowed_write_prefix: "allowed".to_string(),
            max_write_bytes: 4096,
            policy_id: "compatibility-only-policy".to_string(),
            policy_owner_participant_id: "participant:legacy-owner".to_string(),
            review_requirement: ReviewRequirement::RequireReview,
        }
    }

    fn state() -> CaseState {
        let mut state = CaseState::new("case:admission", CaseLifecycle::Open);
        state.generation = 7;
        state.participants = vec![
            ParticipantState {
                participant_id: "participant:model".to_string(),
                roles: vec!["proposer".to_string()],
                admitted_views: Vec::new(),
            },
            ParticipantState {
                participant_id: "participant:reviewer-a".to_string(),
                roles: vec!["reviewer".to_string()],
                admitted_views: Vec::new(),
            },
            ParticipantState {
                participant_id: "participant:reviewer-b".to_string(),
                roles: vec!["reviewer".to_string()],
                admitted_views: Vec::new(),
            },
        ];
        state.resources = vec![resource()];
        state
    }

    fn operation_fixture(state: &CaseState, relative_path: &str) -> Operation {
        let resource = resource();
        normalize_filesystem_write_candidate(
            &serde_json::json!({
                "schema": crate::effect::OPERATION_PROPOSAL_SCHEMA,
                "operation":"filesystem.write",
                "resource":"workspace",
                "path":relative_path,
                "content":"governed content"
            })
            .to_string(),
            &NormalizationContext {
                case_id: &state.case_id,
                participant_id: "participant:model",
                provider_result_id: "provider-result:canonical",
                provider_invocation_id: "provider-invocation:canonical",
                case_generation: state.generation,
                resource: &resource,
            },
        )
        .unwrap()
    }

    fn provenance(rule: &str) -> Vec<EffectiveRuleProvenance> {
        vec![EffectiveRuleProvenance {
            binding_id: "binding:one".to_string(),
            artifact_id: "artifact:one".to_string(),
            policy_ir_rule_id: rule.to_string(),
            source_id: "source:one".to_string(),
            fact_refs: vec![format!("fact:{rule}")],
            source_locations: vec!["$.rules[0]".to_string()],
        }]
    }

    fn policy(effect: Option<PolicyEffect>, review: bool) -> EffectivePolicy {
        let mut rules = Vec::new();
        if let Some(effect) = effect {
            rules.push(EffectivePolicyRule::OperationRestriction {
                operation_kind: "filesystem.write".to_string(),
                resource_kind: Some("filesystem".to_string()),
                effect,
                resolution: "test".to_string(),
                contributions: provenance("operation"),
            });
        }
        rules.push(EffectivePolicyRule::AuthorityRequirement {
            operation_kind: "filesystem.write".to_string(),
            resource_kind: Some("filesystem".to_string()),
            subject: AuthoritySubject::Proposer,
            required_roles: vec!["proposer".to_string()],
            resolution: "all_of".to_string(),
            contributions: provenance("proposer"),
        });
        rules.push(EffectivePolicyRule::AuthorityRequirement {
            operation_kind: "filesystem.write".to_string(),
            resource_kind: Some("filesystem".to_string()),
            subject: AuthoritySubject::Reviewer,
            required_roles: vec!["reviewer".to_string()],
            resolution: "all_of".to_string(),
            contributions: provenance("reviewer"),
        });
        if review {
            rules.push(EffectivePolicyRule::ReviewRequirement {
                operation_kind: "filesystem.write".to_string(),
                resource_kind: Some("filesystem".to_string()),
                required: true,
                resolution: "required".to_string(),
                contributions: provenance("review"),
            });
        }
        rules.push(EffectivePolicyRule::EvidenceObligation {
            operation_kind: "filesystem.write".to_string(),
            resource_kind: Some("filesystem".to_string()),
            obligation: EvidenceObligationKind::SourceProvenance,
            resolution: "required".to_string(),
            contributions: provenance("source"),
        });
        rules.push(EffectivePolicyRule::EvidenceObligation {
            operation_kind: "filesystem.write".to_string(),
            resource_kind: Some("filesystem".to_string()),
            obligation: EvidenceObligationKind::PostObservation,
            resolution: "required".to_string(),
            contributions: provenance("post"),
        });
        EffectivePolicy {
            schema: crate::case_policy::EFFECTIVE_POLICY_SCHEMA.to_string(),
            effective_policy_id: "effective-policy:one".to_string(),
            semantic_digest: "sha256:effective-one".to_string(),
            case_id: "case:admission".to_string(),
            materializer_version: crate::case_policy::POLICY_MATERIALIZER_VERSION.to_string(),
            binding_ids: vec!["binding:one".to_string()],
            artifact_ids: vec!["artifact:one".to_string()],
            input_rule_count: rules.len(),
            rules,
            merged_rule_count: 0,
            resolved_conflict_count: 0,
        }
    }

    fn evidence() -> EvidenceContext {
        EvidenceContext {
            canonical_provider_result_id: Some("provider-result:canonical".to_string()),
            canonical_provider_invocation_id: Some("provider-invocation:canonical".to_string()),
            review_action_id: None,
            review_reason: None,
        }
    }

    #[test]
    fn explicit_allow_is_policy_bound_and_resource_legacy_fields_are_inert() {
        let state = state();
        let operation = operation_fixture(&state, "allowed/result.txt");
        let policy = policy(Some(PolicyEffect::Allow), false);
        let first =
            evaluate_filesystem_admission(&operation, &state, &resource(), &policy, &evidence())
                .unwrap();
        assert_eq!(first.schema, DECISION_SCHEMA);
        assert_eq!(first.outcome, DecisionOutcome::Allow);
        let mut changed_legacy = resource();
        changed_legacy.policy_id = "policy:changed-only-legacy".to_string();
        changed_legacy.policy_owner_participant_id = "participant:anyone".to_string();
        changed_legacy.review_requirement = ReviewRequirement::Automatic;
        let second = evaluate_filesystem_admission(
            &operation,
            &state,
            &changed_legacy,
            &policy,
            &evidence(),
        )
        .unwrap();
        assert_eq!(first.decision_digest, second.decision_digest);
        let grant = issue_policy_execution_grant(&operation, &first, state.generation + 1).unwrap();
        assert_eq!(grant.schema, EXECUTION_GRANT_SCHEMA);
        assert_eq!(
            grant.effective_policy_id.as_deref(),
            Some("effective-policy:one")
        );
        assert!(grant.require_pre_observation && grant.require_post_observation);
        assert_eq!(
            grant.execution_evidence_requirements,
            vec![ExecutionEvidenceRequirement::PostObservation]
        );
    }

    #[test]
    fn deny_no_match_resource_violation_and_missing_role_fail_closed() {
        let mut state = state();
        let operation = operation_fixture(&state, "allowed/result.txt");
        let deny = evaluate_filesystem_admission(
            &operation,
            &state,
            &resource(),
            &policy(Some(PolicyEffect::Deny), false),
            &evidence(),
        )
        .unwrap();
        assert_eq!(deny.outcome, DecisionOutcome::Deny);
        assert_eq!(deny.reason, "applicable_policy_deny");
        let no_match = evaluate_filesystem_admission(
            &operation,
            &state,
            &resource(),
            &policy(None, false),
            &evidence(),
        )
        .unwrap();
        assert_eq!(no_match.reason, "no_applicable_allow_rule");
        let outside = operation_fixture(&state, "outside/result.txt");
        let resource_deny = evaluate_filesystem_admission(
            &outside,
            &state,
            &resource(),
            &policy(Some(PolicyEffect::Allow), false),
            &evidence(),
        )
        .unwrap();
        assert_eq!(resource_deny.reason, "resource_mechanical_envelope_denied");
        state.participants[0].roles.clear();
        let role_deny = evaluate_filesystem_admission(
            &operation,
            &state,
            &resource(),
            &policy(Some(PolicyEffect::Allow), false),
            &evidence(),
        )
        .unwrap();
        assert_eq!(role_deny.reason, "proposer_not_case_role_eligible");
    }

    #[test]
    fn review_eligibility_and_source_provenance_are_mechanical() {
        let state = state();
        let operation = operation_fixture(&state, "allowed/review.txt");
        let policy = policy(Some(PolicyEffect::Allow), true);
        let decision =
            evaluate_filesystem_admission(&operation, &state, &resource(), &policy, &evidence())
                .unwrap();
        assert_eq!(decision.outcome, DecisionOutcome::RequireReview);
        let reviewer = decision
            .decision_basis
            .as_ref()
            .unwrap()
            .authority
            .iter()
            .find(|evaluation| evaluation.subject == AuthoritySubject::Reviewer)
            .unwrap();
        assert_eq!(
            reviewer.eligible_participant_ids,
            vec!["participant:reviewer-a", "participant:reviewer-b"]
        );
        let review = build_policy_review_request(&operation, &decision, state.generation + 1)
            .expect("build integrity-bound review request");
        review
            .validate_policy_integrity()
            .expect("review request integrity");
        let mut tampered_review = review;
        tampered_review.required_reviewer_roles = vec!["invented-role".to_string()];
        assert!(tampered_review
            .validate_policy_integrity()
            .unwrap_err()
            .contains("policy_review_request_integrity_mismatch"));
        let forged = evaluate_filesystem_admission(
            &operation,
            &state,
            &resource(),
            &policy,
            &EvidenceContext::default(),
        )
        .unwrap();
        assert_eq!(forged.outcome, DecisionOutcome::Deny);
        assert_eq!(forged.reason, "required_admission_evidence_missing");
    }

    #[test]
    fn audit_reason_requires_real_review_action_evidence() {
        let mut state = state();
        let operation = operation_fixture(&state, "allowed/audited.txt");
        let mut policy = policy(Some(PolicyEffect::Allow), false);
        policy.rules.push(EffectivePolicyRule::EvidenceObligation {
            operation_kind: "filesystem.write".to_string(),
            resource_kind: Some("filesystem".to_string()),
            obligation: EvidenceObligationKind::AuditReason,
            resolution: "required".to_string(),
            contributions: provenance("audit"),
        });
        policy.input_rule_count = policy.rules.len();
        let initial =
            evaluate_filesystem_admission(&operation, &state, &resource(), &policy, &evidence())
                .expect("evaluate missing audit reason");
        assert_eq!(initial.outcome, DecisionOutcome::RequireReview);
        assert!(initial
            .decision_basis
            .as_ref()
            .unwrap()
            .obligations
            .iter()
            .any(
                |obligation| obligation.obligation == EvidenceObligationKind::AuditReason
                    && obligation.status == ObligationStatus::MissingRequiresReview
            ));
        let mut review = build_policy_review_request(&operation, &initial, state.generation + 1)
            .expect("request human rationale");
        let action = build_review_action(
            &review,
            &state.case_id,
            "participant:reviewer-a",
            ReviewActionKind::Approve,
            "ticket SEC-42 authorizes this exact write",
            state.generation + 1,
            "local_cli_claimed_participant",
        )
        .expect("record typed rationale");
        review.latest_action_id = Some(action.action_id.clone());
        review.status = ReviewResolution::Approved;
        state.last_operation = Some(OperationState {
            operation_id: operation.operation_id.clone(),
            operation_digest: operation.operation_digest.clone(),
            participant_id: operation.participant_id.clone(),
            resource_attachment_id: operation.resource_attachment_id.clone(),
            relative_path: operation.filesystem_write.relative_path.clone(),
            intended_content_digest: operation.filesystem_write.content_digest.clone(),
            origin: operation.origin.clone(),
            recorded_at_generation: state.generation,
        });
        let effective = resolve_policy_review_decision(
            &operation,
            &state,
            &resource(),
            &policy,
            &review,
            &action,
        )
        .expect("approve with real audit evidence");
        assert_eq!(effective.outcome, DecisionOutcome::Allow);
        let audit = effective
            .decision_basis
            .as_ref()
            .unwrap()
            .obligations
            .iter()
            .find(|obligation| obligation.obligation == EvidenceObligationKind::AuditReason)
            .unwrap();
        assert_eq!(audit.status, ObligationStatus::Satisfied);
        assert_eq!(audit.evidence_refs, vec![action.action_id]);
    }

    #[test]
    fn decision_basis_and_grant_integrity_detect_tampering() {
        let state = state();
        let operation = operation_fixture(&state, "allowed/result.txt");
        let decision = evaluate_filesystem_admission(
            &operation,
            &state,
            &resource(),
            &policy(Some(PolicyEffect::Allow), false),
            &evidence(),
        )
        .unwrap();
        let mut tampered = decision.clone();
        tampered
            .decision_basis
            .as_mut()
            .unwrap()
            .effective_policy_id = "effective-policy:forged".to_string();
        assert_eq!(
            tampered.validate_integrity(),
            Err("decision_basis_integrity_mismatch".to_string())
        );
        let mut grant =
            issue_policy_execution_grant(&operation, &decision, state.generation + 1).unwrap();
        grant.effective_policy_digest = Some("sha256:forged".to_string());
        assert_eq!(
            grant.validate_integrity(),
            Err("execution_grant_integrity_mismatch".to_string())
        );
    }
}
