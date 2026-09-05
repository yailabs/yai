//! Provider-independent cognitive capability bindings and execution planning.
//!
//! Semantic suitability is explicit YAI evidence. It is intentionally distinct
//! from mechanical `ProviderCapability`, and planning never dispatches provider
//! work. Case binding history is canonical in `transition`; plans and lanes are
//! deterministic derived values and continuations remain disposable hints.

use crate::effect::digest_bytes;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SEMANTIC_SUITABILITY_EVIDENCE_SCHEMA: &str = "yai.semantic_suitability_evidence.v1";
pub const CASE_COGNITIVE_BINDING_SCHEMA: &str = "yai.case_cognitive_binding.v1";
pub const COGNITIVE_REQUIREMENT_SCHEMA: &str = "yai.cognitive_capability_requirement.v1";
pub const COGNITIVE_EXECUTION_PLAN_SCHEMA: &str = "yai.cognitive_execution_plan.v1";
pub const COGNITIVE_PLANNER_VERSION: &str = "yai.cognitive_execution_planner.v1";
pub const MAX_SEMANTIC_EVIDENCE_REFS: usize = 32;
pub const MAX_COGNITIVE_BINDINGS_PER_CASE: usize = 64;

fn require_identifier(label: &str, value: &str, max: usize) -> Result<(), String> {
    if value.is_empty()
        || value.len() > max
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "._:-/".contains(ch))
    {
        return Err(format!("{label}_invalid"));
    }
    Ok(())
}

fn digest_of<T: Serialize>(value: &T, label: &str) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|bytes| digest_bytes(&bytes))
        .map_err(|error| format!("{label}_encode_failed: {error}"))
}

fn short_identity(prefix: &str, digest: &str) -> String {
    let material = digest.strip_prefix("sha256:").unwrap_or(digest);
    format!("{prefix}:{}", &material[..material.len().min(32)])
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveCapability {
    PrimaryConversation,
    SpeechToText,
    ImageUnderstanding,
}

impl CognitiveCapability {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::PrimaryConversation => "primary_conversation",
            Self::SpeechToText => "speech_to_text",
            Self::ImageUnderstanding => "image_understanding",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "primary_conversation" => Ok(Self::PrimaryConversation),
            "speech_to_text" => Ok(Self::SpeechToText),
            "image_understanding" => Ok(Self::ImageUnderstanding),
            _ => Err("unknown_cognitive_capability".to_string()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticEvidencePosture {
    /// An authenticated Tenant owner supplied evidence references. This is not
    /// a mechanically executed semantic qualification.
    OperatorAttested,
    /// Deterministic repository qualification evidence used by executable
    /// tests. It does not imply external model execution.
    DeterministicFixture,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticSuitabilityEvidence {
    pub schema: String,
    pub evidence_id: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub target_id: String,
    pub target_digest: String,
    pub capability: CognitiveCapability,
    pub posture: SemanticEvidencePosture,
    pub suite_id: String,
    pub run_id: String,
    pub provenance_refs: Vec<String>,
    pub qualification_source: String,
    pub recorded_by_principal_id: String,
    pub recorded_at_unix_ms: u64,
}

#[derive(Serialize)]
struct SemanticEvidenceIdentity<'a> {
    schema: &'a str,
    tenant_id: &'a str,
    target_id: &'a str,
    target_digest: &'a str,
    capability: &'a CognitiveCapability,
    posture: &'a SemanticEvidencePosture,
    suite_id: &'a str,
    run_id: &'a str,
    provenance_refs: &'a [String],
    qualification_source: &'a str,
    recorded_by_principal_id: &'a str,
    recorded_at_unix_ms: u64,
}

impl SemanticSuitabilityEvidence {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        target_id: &str,
        target_digest: &str,
        capability: CognitiveCapability,
        posture: SemanticEvidencePosture,
        suite_id: &str,
        run_id: &str,
        mut provenance_refs: Vec<String>,
        qualification_source: &str,
        recorded_by_principal_id: &str,
        recorded_at_unix_ms: u64,
    ) -> Result<Self, String> {
        require_identifier("semantic_evidence_tenant", tenant_id, 256)?;
        require_identifier("semantic_evidence_target", target_id, 256)?;
        require_identifier("semantic_evidence_target_digest", target_digest, 128)?;
        require_identifier("semantic_evidence_suite", suite_id, 128)?;
        require_identifier("semantic_evidence_run", run_id, 256)?;
        require_identifier(
            "semantic_evidence_qualification_source",
            qualification_source,
            128,
        )?;
        require_identifier("semantic_evidence_principal", recorded_by_principal_id, 256)?;
        provenance_refs.sort();
        provenance_refs.dedup();
        if provenance_refs.is_empty() || provenance_refs.len() > MAX_SEMANTIC_EVIDENCE_REFS {
            return Err("semantic_evidence_refs_bounds_invalid".to_string());
        }
        for value in &provenance_refs {
            require_identifier("semantic_evidence_ref", value, 256)?;
        }
        let identity = SemanticEvidenceIdentity {
            schema: SEMANTIC_SUITABILITY_EVIDENCE_SCHEMA,
            tenant_id,
            target_id,
            target_digest,
            capability: &capability,
            posture: &posture,
            suite_id,
            run_id,
            provenance_refs: &provenance_refs,
            qualification_source,
            recorded_by_principal_id,
            recorded_at_unix_ms,
        };
        let integrity_digest = digest_of(&identity, "semantic_evidence_identity")?;
        Ok(Self {
            schema: SEMANTIC_SUITABILITY_EVIDENCE_SCHEMA.to_string(),
            evidence_id: short_identity("semantic-suitability", &integrity_digest),
            integrity_digest,
            tenant_id: tenant_id.to_string(),
            target_id: target_id.to_string(),
            target_digest: target_digest.to_string(),
            capability,
            posture,
            suite_id: suite_id.to_string(),
            run_id: run_id.to_string(),
            provenance_refs,
            qualification_source: qualification_source.to_string(),
            recorded_by_principal_id: recorded_by_principal_id.to_string(),
            recorded_at_unix_ms,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != SEMANTIC_SUITABILITY_EVIDENCE_SCHEMA {
            return Err("unsupported_semantic_suitability_evidence_schema".to_string());
        }
        let rebuilt = Self::new(
            &self.tenant_id,
            &self.target_id,
            &self.target_digest,
            self.capability.clone(),
            self.posture.clone(),
            &self.suite_id,
            &self.run_id,
            self.provenance_refs.clone(),
            &self.qualification_source,
            &self.recorded_by_principal_id,
            self.recorded_at_unix_ms,
        )?;
        if rebuilt != *self {
            return Err("semantic_suitability_evidence_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveBindingRole {
    Primary,
    Auxiliary,
}

impl CognitiveBindingRole {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Auxiliary => "auxiliary",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "primary" => Ok(Self::Primary),
            "auxiliary" => Ok(Self::Auxiliary),
            _ => Err("cognitive_binding_role_invalid".to_string()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CaseCognitiveBinding {
    pub schema: String,
    pub binding_id: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub case_id: String,
    pub participant_id: String,
    pub role: CognitiveBindingRole,
    pub capability: CognitiveCapability,
    pub target_id: String,
    pub target_digest: String,
    pub semantic_evidence_id: String,
    pub provider_binding_id_at_bind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaces_binding_id: Option<String>,
    pub bound_by_principal_id: String,
    pub bound_at_generation: u64,
}

#[derive(Serialize)]
struct CognitiveBindingIdentity<'a> {
    schema: &'a str,
    tenant_id: &'a str,
    case_id: &'a str,
    participant_id: &'a str,
    role: &'a CognitiveBindingRole,
    capability: &'a CognitiveCapability,
    target_id: &'a str,
    target_digest: &'a str,
    semantic_evidence_id: &'a str,
    provider_binding_id_at_bind: &'a str,
    replaces_binding_id: &'a Option<String>,
    bound_by_principal_id: &'a str,
    bound_at_generation: u64,
}

impl CaseCognitiveBinding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        case_id: &str,
        participant_id: &str,
        role: CognitiveBindingRole,
        capability: CognitiveCapability,
        target_id: &str,
        target_digest: &str,
        semantic_evidence_id: &str,
        provider_binding_id_at_bind: &str,
        replaces_binding_id: Option<String>,
        bound_by_principal_id: &str,
        bound_at_generation: u64,
    ) -> Result<Self, String> {
        if (role == CognitiveBindingRole::Primary
            && capability != CognitiveCapability::PrimaryConversation)
            || (role == CognitiveBindingRole::Auxiliary
                && capability == CognitiveCapability::PrimaryConversation)
        {
            return Err("cognitive_binding_role_capability_mismatch".to_string());
        }
        for (label, value) in [
            ("cognitive_binding_tenant", tenant_id),
            ("cognitive_binding_case", case_id),
            ("cognitive_binding_participant", participant_id),
            ("cognitive_binding_target", target_id),
            ("cognitive_binding_target_digest", target_digest),
            ("cognitive_binding_evidence", semantic_evidence_id),
            (
                "cognitive_binding_provider_envelope",
                provider_binding_id_at_bind,
            ),
            ("cognitive_binding_principal", bound_by_principal_id),
        ] {
            require_identifier(label, value, 256)?;
        }
        if let Some(value) = &replaces_binding_id {
            require_identifier("cognitive_binding_replaces", value, 256)?;
        }
        let identity = CognitiveBindingIdentity {
            schema: CASE_COGNITIVE_BINDING_SCHEMA,
            tenant_id,
            case_id,
            participant_id,
            role: &role,
            capability: &capability,
            target_id,
            target_digest,
            semantic_evidence_id,
            provider_binding_id_at_bind,
            replaces_binding_id: &replaces_binding_id,
            bound_by_principal_id,
            bound_at_generation,
        };
        let integrity_digest = digest_of(&identity, "cognitive_binding_identity")?;
        Ok(Self {
            schema: CASE_COGNITIVE_BINDING_SCHEMA.to_string(),
            binding_id: short_identity("case-cognitive-binding", &integrity_digest),
            integrity_digest,
            tenant_id: tenant_id.to_string(),
            case_id: case_id.to_string(),
            participant_id: participant_id.to_string(),
            role,
            capability,
            target_id: target_id.to_string(),
            target_digest: target_digest.to_string(),
            semantic_evidence_id: semantic_evidence_id.to_string(),
            provider_binding_id_at_bind: provider_binding_id_at_bind.to_string(),
            replaces_binding_id,
            bound_by_principal_id: bound_by_principal_id.to_string(),
            bound_at_generation,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != CASE_COGNITIVE_BINDING_SCHEMA {
            return Err("unsupported_case_cognitive_binding_schema".to_string());
        }
        let rebuilt = Self::new(
            &self.tenant_id,
            &self.case_id,
            &self.participant_id,
            self.role.clone(),
            self.capability.clone(),
            &self.target_id,
            &self.target_digest,
            &self.semantic_evidence_id,
            &self.provider_binding_id_at_bind,
            self.replaces_binding_id.clone(),
            &self.bound_by_principal_id,
            self.bound_at_generation,
        )?;
        if rebuilt != *self {
            return Err("case_cognitive_binding_integrity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn same_slot(&self, other: &Self) -> bool {
        self.participant_id == other.participant_id
            && self.role == other.role
            && (self.role == CognitiveBindingRole::Primary || self.capability == other.capability)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CognitiveCapabilityRequirement {
    pub schema: String,
    pub requirement_id: String,
    pub integrity_digest: String,
    pub case_id: String,
    pub participant_id: String,
    pub capability: CognitiveCapability,
    pub source_ref: String,
}

impl CognitiveCapabilityRequirement {
    pub fn new(
        case_id: &str,
        participant_id: &str,
        capability: CognitiveCapability,
        source_ref: &str,
    ) -> Result<Self, String> {
        require_identifier("cognitive_requirement_case", case_id, 256)?;
        require_identifier("cognitive_requirement_participant", participant_id, 256)?;
        require_identifier("cognitive_requirement_source", source_ref, 256)?;
        let material = (
            COGNITIVE_REQUIREMENT_SCHEMA,
            case_id,
            participant_id,
            &capability,
            source_ref,
        );
        let integrity_digest = digest_of(&material, "cognitive_requirement_identity")?;
        Ok(Self {
            schema: COGNITIVE_REQUIREMENT_SCHEMA.to_string(),
            requirement_id: short_identity("cognitive-requirement", &integrity_digest),
            integrity_digest,
            case_id: case_id.to_string(),
            participant_id: participant_id.to_string(),
            capability,
            source_ref: source_ref.to_string(),
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        let rebuilt = Self::new(
            &self.case_id,
            &self.participant_id,
            self.capability.clone(),
            &self.source_ref,
        )?;
        if self.schema != COGNITIVE_REQUIREMENT_SCHEMA || rebuilt != *self {
            return Err("cognitive_requirement_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CognitiveTargetSnapshot {
    pub target_id: String,
    pub target_digest: String,
    pub provider_envelope_admitted: bool,
    pub mechanically_qualified: bool,
    pub trust_approved: bool,
    pub semantic_evidence: Vec<SemanticSuitabilityEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CognitivePlanningSnapshot {
    pub tenant_id: String,
    pub case_id: String,
    pub participant_id: String,
    pub case_generation: u64,
    pub active_bindings: Vec<CaseCognitiveBinding>,
    pub targets: Vec<CognitiveTargetSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitivePlanRoute {
    Native,
    Derived,
    Unresolved,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitivePlanRole {
    Primary,
    Auxiliary,
    None,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitivePlanUnresolvedReason {
    PrimaryBindingMissing,
    PrimaryBindingStale,
    PrimaryTargetNotAdmitted,
    PrimaryProviderQualificationMissing,
    PrimaryTrustNotApproved,
    PrimarySuitabilityMissing,
    AuxiliaryBindingMissing,
    AuxiliaryBindingStale,
    AuxiliaryTargetNotAdmitted,
    AuxiliaryProviderQualificationMissing,
    AuxiliaryTrustNotApproved,
    AuxiliarySuitabilityMissing,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderExecutionPosture {
    NotPerformed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderRealizationPosture {
    DeferredToExecutionAdapter,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CognitiveExecutionPlan {
    pub schema: String,
    pub planner_version: String,
    pub plan_id: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub case_id: String,
    pub participant_id: String,
    pub case_generation: u64,
    pub requirement_id: String,
    pub capability: CognitiveCapability,
    pub route: CognitivePlanRoute,
    pub role: CognitivePlanRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_binding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_target_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_evidence_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_lane_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unresolved_reason: Option<CognitivePlanUnresolvedReason>,
    pub provider_realization: ProviderRealizationPosture,
    pub provider_execution: ProviderExecutionPosture,
}

#[derive(Serialize)]
struct PlanIdentity<'a> {
    planner_version: &'a str,
    tenant_id: &'a str,
    case_id: &'a str,
    participant_id: &'a str,
    case_generation: u64,
    requirement_id: &'a str,
    capability: &'a CognitiveCapability,
    route: &'a CognitivePlanRoute,
    role: &'a CognitivePlanRole,
    selected_binding_id: &'a Option<String>,
    selected_target_id: &'a Option<String>,
    semantic_evidence_id: &'a Option<String>,
    execution_lane_id: &'a Option<String>,
    unresolved_reason: &'a Option<CognitivePlanUnresolvedReason>,
}

fn execution_lane_id(
    snapshot: &CognitivePlanningSnapshot,
    binding: &CaseCognitiveBinding,
) -> Result<String, String> {
    let material = if binding.role == CognitiveBindingRole::Primary {
        format!(
            "primary\0{}\0{}\0{}",
            snapshot.case_id, snapshot.participant_id, binding.binding_id
        )
    } else {
        format!(
            "auxiliary\0{}\0{}\0{}\0{}",
            snapshot.case_id,
            snapshot.participant_id,
            binding.capability.as_str(),
            binding.binding_id
        )
    };
    Ok(short_identity(
        "cognitive-lane",
        &digest_of(&material, "cognitive_lane_identity")?,
    ))
}

fn target_for_binding<'a>(
    snapshot: &'a CognitivePlanningSnapshot,
    binding: &CaseCognitiveBinding,
) -> Option<&'a CognitiveTargetSnapshot> {
    snapshot.targets.iter().find(|target| {
        target.target_id == binding.target_id && target.target_digest == binding.target_digest
    })
}

fn exact_evidence<'a>(
    target: &'a CognitiveTargetSnapshot,
    capability: &CognitiveCapability,
    exact_id: Option<&str>,
) -> Option<&'a SemanticSuitabilityEvidence> {
    target
        .semantic_evidence
        .iter()
        .filter(|evidence| {
            evidence.validate().is_ok()
                && evidence.target_id == target.target_id
                && evidence.target_digest == target.target_digest
                && evidence.capability == *capability
                && exact_id.is_none_or(|value| value == evidence.evidence_id)
        })
        .min_by(|left, right| left.evidence_id.cmp(&right.evidence_id))
}

fn unresolved(
    snapshot: &CognitivePlanningSnapshot,
    requirement: &CognitiveCapabilityRequirement,
    reason: CognitivePlanUnresolvedReason,
) -> Result<CognitiveExecutionPlan, String> {
    seal_plan(
        snapshot,
        requirement,
        CognitivePlanRoute::Unresolved,
        CognitivePlanRole::None,
        None,
        None,
        None,
        None,
        Some(reason),
    )
}

#[allow(clippy::too_many_arguments)]
fn seal_plan(
    snapshot: &CognitivePlanningSnapshot,
    requirement: &CognitiveCapabilityRequirement,
    route: CognitivePlanRoute,
    role: CognitivePlanRole,
    selected_binding_id: Option<String>,
    selected_target_id: Option<String>,
    semantic_evidence_id: Option<String>,
    execution_lane_id: Option<String>,
    unresolved_reason: Option<CognitivePlanUnresolvedReason>,
) -> Result<CognitiveExecutionPlan, String> {
    let identity = PlanIdentity {
        planner_version: COGNITIVE_PLANNER_VERSION,
        tenant_id: &snapshot.tenant_id,
        case_id: &snapshot.case_id,
        participant_id: &snapshot.participant_id,
        case_generation: snapshot.case_generation,
        requirement_id: &requirement.requirement_id,
        capability: &requirement.capability,
        route: &route,
        role: &role,
        selected_binding_id: &selected_binding_id,
        selected_target_id: &selected_target_id,
        semantic_evidence_id: &semantic_evidence_id,
        execution_lane_id: &execution_lane_id,
        unresolved_reason: &unresolved_reason,
    };
    let integrity_digest = digest_of(&identity, "cognitive_plan_identity")?;
    Ok(CognitiveExecutionPlan {
        schema: COGNITIVE_EXECUTION_PLAN_SCHEMA.to_string(),
        planner_version: COGNITIVE_PLANNER_VERSION.to_string(),
        plan_id: short_identity("cognitive-plan", &integrity_digest),
        integrity_digest,
        tenant_id: snapshot.tenant_id.clone(),
        case_id: snapshot.case_id.clone(),
        participant_id: snapshot.participant_id.clone(),
        case_generation: snapshot.case_generation,
        requirement_id: requirement.requirement_id.clone(),
        capability: requirement.capability.clone(),
        route,
        role,
        selected_binding_id,
        selected_target_id,
        semantic_evidence_id,
        execution_lane_id,
        unresolved_reason,
        provider_realization: ProviderRealizationPosture::DeferredToExecutionAdapter,
        provider_execution: ProviderExecutionPosture::NotPerformed,
    })
}

/// Produces a semantic route only. It performs no provider selection,
/// transport, network access, content derivation, or model execution.
pub fn plan_cognitive_execution(
    snapshot: &CognitivePlanningSnapshot,
    requirement: &CognitiveCapabilityRequirement,
) -> Result<CognitiveExecutionPlan, String> {
    requirement.validate()?;
    if snapshot.case_id != requirement.case_id
        || snapshot.participant_id != requirement.participant_id
    {
        return Err("cognitive_planning_scope_mismatch".to_string());
    }
    if snapshot.active_bindings.len() > MAX_COGNITIVE_BINDINGS_PER_CASE {
        return Err("cognitive_binding_case_limit_exceeded".to_string());
    }
    let primary = snapshot.active_bindings.iter().find(|binding| {
        binding.participant_id == snapshot.participant_id
            && binding.role == CognitiveBindingRole::Primary
    });
    let Some(primary) = primary else {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::PrimaryBindingMissing,
        );
    };
    if primary.validate().is_err()
        || primary.tenant_id != snapshot.tenant_id
        || primary.case_id != snapshot.case_id
    {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::PrimaryBindingStale,
        );
    }
    let Some(primary_target) = target_for_binding(snapshot, primary) else {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::PrimaryBindingStale,
        );
    };
    if !primary_target.provider_envelope_admitted {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::PrimaryTargetNotAdmitted,
        );
    }
    if !primary_target.mechanically_qualified {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::PrimaryProviderQualificationMissing,
        );
    }
    if !primary_target.trust_approved {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::PrimaryTrustNotApproved,
        );
    }
    if exact_evidence(
        primary_target,
        &primary.capability,
        Some(&primary.semantic_evidence_id),
    )
    .is_none()
    {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::PrimaryBindingStale,
        );
    }
    if let Some(evidence) = exact_evidence(primary_target, &requirement.capability, None) {
        return seal_plan(
            snapshot,
            requirement,
            CognitivePlanRoute::Native,
            CognitivePlanRole::Primary,
            Some(primary.binding_id.clone()),
            Some(primary.target_id.clone()),
            Some(evidence.evidence_id.clone()),
            Some(execution_lane_id(snapshot, primary)?),
            None,
        );
    }
    let auxiliary = snapshot.active_bindings.iter().find(|binding| {
        binding.participant_id == snapshot.participant_id
            && binding.role == CognitiveBindingRole::Auxiliary
            && binding.capability == requirement.capability
    });
    let Some(auxiliary) = auxiliary else {
        let reason = if requirement.capability == CognitiveCapability::PrimaryConversation {
            CognitivePlanUnresolvedReason::PrimarySuitabilityMissing
        } else {
            CognitivePlanUnresolvedReason::AuxiliaryBindingMissing
        };
        return unresolved(snapshot, requirement, reason);
    };
    if auxiliary.validate().is_err()
        || auxiliary.tenant_id != snapshot.tenant_id
        || auxiliary.case_id != snapshot.case_id
    {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::AuxiliaryBindingStale,
        );
    }
    let Some(auxiliary_target) = target_for_binding(snapshot, auxiliary) else {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::AuxiliaryBindingStale,
        );
    };
    if !auxiliary_target.provider_envelope_admitted {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::AuxiliaryTargetNotAdmitted,
        );
    }
    if !auxiliary_target.mechanically_qualified {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::AuxiliaryProviderQualificationMissing,
        );
    }
    if !auxiliary_target.trust_approved {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::AuxiliaryTrustNotApproved,
        );
    }
    let Some(evidence) = exact_evidence(
        auxiliary_target,
        &requirement.capability,
        Some(&auxiliary.semantic_evidence_id),
    ) else {
        return unresolved(
            snapshot,
            requirement,
            CognitivePlanUnresolvedReason::AuxiliarySuitabilityMissing,
        );
    };
    seal_plan(
        snapshot,
        requirement,
        CognitivePlanRoute::Derived,
        CognitivePlanRole::Auxiliary,
        Some(auxiliary.binding_id.clone()),
        Some(auxiliary.target_id.clone()),
        Some(evidence.evidence_id.clone()),
        Some(execution_lane_id(snapshot, auxiliary)?),
        None,
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaneContinuationReference {
    pub execution_lane_id: String,
    pub target_id: String,
    pub runtime_id: String,
    pub opaque_reference: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaneContinuationPosture {
    NotProvidedSemanticReconstruction,
    Compatible,
    RejectedUnresolvedPlan,
    RejectedCrossLane,
    RejectedTargetMismatch,
}

pub fn assess_lane_continuation(
    plan: &CognitiveExecutionPlan,
    continuation: Option<&LaneContinuationReference>,
) -> LaneContinuationPosture {
    let Some(continuation) = continuation else {
        return LaneContinuationPosture::NotProvidedSemanticReconstruction;
    };
    let (Some(lane_id), Some(target_id)) = (
        plan.execution_lane_id.as_deref(),
        plan.selected_target_id.as_deref(),
    ) else {
        return LaneContinuationPosture::RejectedUnresolvedPlan;
    };
    if continuation.execution_lane_id != lane_id {
        return LaneContinuationPosture::RejectedCrossLane;
    }
    if continuation.target_id != target_id {
        return LaneContinuationPosture::RejectedTargetMismatch;
    }
    LaneContinuationPosture::Compatible
}

pub fn validate_active_cognitive_bindings(bindings: &[CaseCognitiveBinding]) -> Result<(), String> {
    if bindings.len() > MAX_COGNITIVE_BINDINGS_PER_CASE {
        return Err("cognitive_binding_case_limit_exceeded".to_string());
    }
    let mut slots = BTreeSet::new();
    for binding in bindings {
        binding.validate()?;
        let slot = if binding.role == CognitiveBindingRole::Primary {
            format!("{}:primary", binding.participant_id)
        } else {
            format!(
                "{}:auxiliary:{}",
                binding.participant_id,
                binding.capability.as_str()
            )
        };
        if !slots.insert(slot) {
            return Err("duplicate_active_cognitive_binding_slot".to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(
        target: &str,
        digest: &str,
        capability: CognitiveCapability,
    ) -> SemanticSuitabilityEvidence {
        SemanticSuitabilityEvidence::new(
            "tenant:test",
            target,
            digest,
            capability,
            SemanticEvidencePosture::DeterministicFixture,
            "suite:i02",
            "run:i02",
            vec!["fixture:i02".to_string()],
            "repository_fixture",
            "principal:test",
            1,
        )
        .unwrap()
    }

    fn binding(
        role: CognitiveBindingRole,
        capability: CognitiveCapability,
        target: &str,
        target_digest: &str,
        evidence_id: &str,
        generation: u64,
        replaces: Option<String>,
    ) -> CaseCognitiveBinding {
        CaseCognitiveBinding::new(
            "tenant:test",
            "case:test",
            "participant:model",
            role,
            capability,
            target,
            target_digest,
            evidence_id,
            "case-provider-binding:test",
            replaces,
            "principal:test",
            generation,
        )
        .unwrap()
    }

    fn target(
        target_id: &str,
        digest: &str,
        semantic_evidence: Vec<SemanticSuitabilityEvidence>,
    ) -> CognitiveTargetSnapshot {
        CognitiveTargetSnapshot {
            target_id: target_id.to_string(),
            target_digest: digest.to_string(),
            provider_envelope_admitted: true,
            mechanically_qualified: true,
            trust_approved: true,
            semantic_evidence,
        }
    }

    #[test]
    fn native_and_derived_plans_are_deterministic_without_execution() {
        let primary_conversation = evidence(
            "provider-target:primary",
            "sha256:primary",
            CognitiveCapability::PrimaryConversation,
        );
        let primary_image = evidence(
            "provider-target:primary",
            "sha256:primary",
            CognitiveCapability::ImageUnderstanding,
        );
        let stt = evidence(
            "provider-target:whisper-name-is-not-proof",
            "sha256:aux",
            CognitiveCapability::SpeechToText,
        );
        let primary = binding(
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            "provider-target:primary",
            "sha256:primary",
            &primary_conversation.evidence_id,
            4,
            None,
        );
        let auxiliary = binding(
            CognitiveBindingRole::Auxiliary,
            CognitiveCapability::SpeechToText,
            "provider-target:whisper-name-is-not-proof",
            "sha256:aux",
            &stt.evidence_id,
            5,
            None,
        );
        let snapshot = CognitivePlanningSnapshot {
            tenant_id: "tenant:test".to_string(),
            case_id: "case:test".to_string(),
            participant_id: "participant:model".to_string(),
            case_generation: 6,
            active_bindings: vec![primary, auxiliary],
            targets: vec![
                target(
                    "provider-target:primary",
                    "sha256:primary",
                    vec![primary_conversation, primary_image],
                ),
                target(
                    "provider-target:whisper-name-is-not-proof",
                    "sha256:aux",
                    vec![stt],
                ),
            ],
        };
        let native_requirement = CognitiveCapabilityRequirement::new(
            "case:test",
            "participant:model",
            CognitiveCapability::ImageUnderstanding,
            "turn:one",
        )
        .unwrap();
        let native = plan_cognitive_execution(&snapshot, &native_requirement).unwrap();
        assert_eq!(native.route, CognitivePlanRoute::Native);
        assert_eq!(native.role, CognitivePlanRole::Primary);
        assert_eq!(
            native,
            plan_cognitive_execution(&snapshot, &native_requirement).unwrap()
        );
        assert_eq!(
            native.provider_execution,
            ProviderExecutionPosture::NotPerformed
        );

        let derived_requirement = CognitiveCapabilityRequirement::new(
            "case:test",
            "participant:model",
            CognitiveCapability::SpeechToText,
            "turn:two",
        )
        .unwrap();
        let derived = plan_cognitive_execution(&snapshot, &derived_requirement).unwrap();
        assert_eq!(derived.route, CognitivePlanRoute::Derived);
        assert_eq!(derived.role, CognitivePlanRole::Auxiliary);
        assert_ne!(native.execution_lane_id, derived.execution_lane_id);
    }

    #[test]
    fn misleading_target_name_never_confers_suitability() {
        let conversation = evidence(
            "provider-target:whisper-vision-deepseek-bge",
            "sha256:names",
            CognitiveCapability::PrimaryConversation,
        );
        let primary = binding(
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            "provider-target:whisper-vision-deepseek-bge",
            "sha256:names",
            &conversation.evidence_id,
            2,
            None,
        );
        let snapshot = CognitivePlanningSnapshot {
            tenant_id: "tenant:test".to_string(),
            case_id: "case:test".to_string(),
            participant_id: "participant:model".to_string(),
            case_generation: 3,
            active_bindings: vec![primary],
            targets: vec![target(
                "provider-target:whisper-vision-deepseek-bge",
                "sha256:names",
                vec![conversation],
            )],
        };
        let requirement = CognitiveCapabilityRequirement::new(
            "case:test",
            "participant:model",
            CognitiveCapability::SpeechToText,
            "turn:audio",
        )
        .unwrap();
        let plan = plan_cognitive_execution(&snapshot, &requirement).unwrap();
        assert_eq!(plan.route, CognitivePlanRoute::Unresolved);
        assert_eq!(
            plan.unresolved_reason,
            Some(CognitivePlanUnresolvedReason::AuxiliaryBindingMissing)
        );
    }

    #[test]
    fn binding_replacement_changes_lane_and_replay_does_not() {
        let first_evidence = evidence(
            "provider-target:first",
            "sha256:first",
            CognitiveCapability::PrimaryConversation,
        );
        let first = binding(
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            "provider-target:first",
            "sha256:first",
            &first_evidence.evidence_id,
            2,
            None,
        );
        let second_evidence = evidence(
            "provider-target:second",
            "sha256:second",
            CognitiveCapability::PrimaryConversation,
        );
        let second = binding(
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            "provider-target:second",
            "sha256:second",
            &second_evidence.evidence_id,
            3,
            Some(first.binding_id.clone()),
        );
        let snapshot = |binding: CaseCognitiveBinding, evidence: SemanticSuitabilityEvidence| {
            CognitivePlanningSnapshot {
                tenant_id: "tenant:test".to_string(),
                case_id: "case:test".to_string(),
                participant_id: "participant:model".to_string(),
                case_generation: 4,
                targets: vec![target(
                    &binding.target_id,
                    &binding.target_digest,
                    vec![evidence],
                )],
                active_bindings: vec![binding],
            }
        };
        let requirement = CognitiveCapabilityRequirement::new(
            "case:test",
            "participant:model",
            CognitiveCapability::PrimaryConversation,
            "turn:one",
        )
        .unwrap();
        let first_plan =
            plan_cognitive_execution(&snapshot(first, first_evidence), &requirement).unwrap();
        let replay = plan_cognitive_execution(
            &snapshot(second.clone(), second_evidence.clone()),
            &requirement,
        )
        .unwrap();
        let replay_again =
            plan_cognitive_execution(&snapshot(second, second_evidence), &requirement).unwrap();
        assert_eq!(replay.execution_lane_id, replay_again.execution_lane_id);
        assert_ne!(first_plan.execution_lane_id, replay.execution_lane_id);
    }

    #[test]
    fn continuation_is_lane_and_target_scoped_but_disposable() {
        let mut plan = CognitiveExecutionPlan {
            schema: COGNITIVE_EXECUTION_PLAN_SCHEMA.to_string(),
            planner_version: COGNITIVE_PLANNER_VERSION.to_string(),
            plan_id: "cognitive-plan:test".to_string(),
            integrity_digest: "sha256:test".to_string(),
            tenant_id: "tenant:test".to_string(),
            case_id: "case:test".to_string(),
            participant_id: "participant:model".to_string(),
            case_generation: 1,
            requirement_id: "requirement:test".to_string(),
            capability: CognitiveCapability::SpeechToText,
            route: CognitivePlanRoute::Derived,
            role: CognitivePlanRole::Auxiliary,
            selected_binding_id: Some("binding:test".to_string()),
            selected_target_id: Some("target:test".to_string()),
            semantic_evidence_id: Some("evidence:test".to_string()),
            execution_lane_id: Some("lane:test".to_string()),
            unresolved_reason: None,
            provider_realization: ProviderRealizationPosture::DeferredToExecutionAdapter,
            provider_execution: ProviderExecutionPosture::NotPerformed,
        };
        let continuation = LaneContinuationReference {
            execution_lane_id: "lane:test".to_string(),
            target_id: "target:test".to_string(),
            runtime_id: "runtime:opaque".to_string(),
            opaque_reference: "not-canonical".to_string(),
        };
        assert_eq!(
            assess_lane_continuation(&plan, None),
            LaneContinuationPosture::NotProvidedSemanticReconstruction
        );
        assert_eq!(
            assess_lane_continuation(&plan, Some(&continuation)),
            LaneContinuationPosture::Compatible
        );
        plan.execution_lane_id = Some("lane:replacement".to_string());
        assert_eq!(
            assess_lane_continuation(&plan, Some(&continuation)),
            LaneContinuationPosture::RejectedCrossLane
        );
    }
}
