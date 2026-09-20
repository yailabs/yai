//! Provider-independent cognitive capability bindings and execution planning.
//!
//! Semantic suitability is explicit YAI evidence. It is intentionally distinct
//! from mechanical `ProviderCapability`, and planning never dispatches provider
//! work. Case binding history is canonical in `transition`; plans and lanes are
//! deterministic derived values and continuations remain disposable hints.

use crate::effect::digest_bytes;
use crate::provider_governance::ProviderRealizationShape;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SEMANTIC_SUITABILITY_EVIDENCE_SCHEMA: &str = "yai.semantic_suitability_evidence.v1";
pub const CASE_COGNITIVE_BINDING_SCHEMA: &str = "yai.case_cognitive_binding.v1";
pub const CASE_COGNITIVE_BINDING_SCHEMA_V2: &str = "yai.case_cognitive_binding.v2";
pub const MAX_COGNITIVE_CANDIDATES: usize = 8;
pub const COGNITIVE_REQUIREMENT_SCHEMA: &str = "yai.cognitive_capability_requirement.v1";
pub const COGNITIVE_EXECUTION_PLAN_SCHEMA: &str = "yai.cognitive_execution_plan.v2";
pub const COGNITIVE_PLANNER_VERSION: &str = "yai.cognitive_execution_planner.v2";
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
    /// v1 is pinned. In v2 the fields above name the FIRST preference, not
    /// the execution target; only a derived plan names the selected target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_policy: Option<CognitiveTargetPolicy>,
    pub provider_binding_id_at_bind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaces_binding_id: Option<String>,
    pub bound_by_principal_id: String,
    pub bound_at_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveTargetCandidate {
    pub target_id: String,
    pub target_digest: String,
    pub semantic_evidence_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CognitiveTargetPolicy {
    OrderedEligible {
        alternatives: Vec<CognitiveTargetCandidate>,
    },
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
            target_policy: None,
            provider_binding_id_at_bind: provider_binding_id_at_bind.to_string(),
            replaces_binding_id,
            bound_by_principal_id: bound_by_principal_id.to_string(),
            bound_at_generation,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != CASE_COGNITIVE_BINDING_SCHEMA
            && self.schema != CASE_COGNITIVE_BINDING_SCHEMA_V2
        {
            return Err("unsupported_case_cognitive_binding_schema".to_string());
        }
        let mut rebuilt = Self::new(
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
        if let Some(CognitiveTargetPolicy::OrderedEligible { alternatives }) = &self.target_policy {
            rebuilt = rebuilt.with_ordered_alternatives(alternatives.clone())?;
        }
        if rebuilt != *self {
            return Err("case_cognitive_binding_integrity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn candidates(&self) -> Vec<CognitiveTargetCandidate> {
        let mut result = vec![CognitiveTargetCandidate {
            target_id: self.target_id.clone(),
            target_digest: self.target_digest.clone(),
            semantic_evidence_id: self.semantic_evidence_id.clone(),
        }];
        if let Some(CognitiveTargetPolicy::OrderedEligible { alternatives }) = &self.target_policy {
            result.extend(alternatives.iter().cloned());
        }
        result
    }

    pub fn candidate(&self, target_id: &str) -> Option<CognitiveTargetCandidate> {
        self.candidates()
            .into_iter()
            .find(|candidate| candidate.target_id == target_id)
    }

    pub fn with_ordered_alternatives(
        mut self,
        alternatives: Vec<CognitiveTargetCandidate>,
    ) -> Result<Self, String> {
        self.validate()?;
        if self.target_policy.is_some()
            || alternatives.is_empty()
            || alternatives.len() >= MAX_COGNITIVE_CANDIDATES
        {
            return Err("cognitive_candidate_policy_bounds_invalid".to_string());
        }
        let mut ids = BTreeSet::from([self.target_id.clone()]);
        for candidate in &alternatives {
            for value in [
                &candidate.target_id,
                &candidate.target_digest,
                &candidate.semantic_evidence_id,
            ] {
                require_identifier("cognitive_candidate", value, 256)?;
            }
            if !ids.insert(candidate.target_id.clone()) {
                return Err("cognitive_candidate_duplicate".to_string());
            }
        }
        self.schema = CASE_COGNITIVE_BINDING_SCHEMA_V2.to_string();
        self.target_policy = Some(CognitiveTargetPolicy::OrderedEligible { alternatives });
        self.integrity_digest = digest_of(
            &(&self.schema, &self.integrity_digest, &self.target_policy),
            "cognitive_policy_identity",
        )?;
        self.binding_id = short_identity("case-cognitive-binding", &self.integrity_digest);
        Ok(self)
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
    pub execution_evidence: Option<CognitiveExecutionEvidence>,
}

/// Exact identities plus effective operational posture, never runtime residency.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CognitiveExecutionEvidence {
    pub provider_envelope_id: Option<String>,
    pub qualification_id: Option<String>,
    pub trust_id: Option<String>,
    pub health_digest: String,
    pub operational_exclusion: Option<CognitiveCandidateExclusion>,
    pub shapes: Vec<ProviderRealizationShape>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveCandidateExclusion {
    TargetMissingOrChanged,
    ProviderEnvelopeMismatch,
    SemanticEvidenceMissingOrStale,
    RequiredSemanticSuitabilityMissing,
    MechanicalQualificationMissingOrStale,
    MechanicalShapeUnsupported,
    TrustNotApproved,
    CircuitOpen,
    ProviderUnavailable,
    CredentialUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CognitiveCandidateAssessment {
    pub binding_id: String,
    pub role: CognitiveBindingRole,
    pub preference: usize,
    pub target_id: String,
    pub target_digest: String,
    pub binding_evidence_id: String,
    pub required_evidence_id: Option<String>,
    pub execution_evidence: Option<CognitiveExecutionEvidence>,
    pub exclusions: Vec<CognitiveCandidateExclusion>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CognitiveArbitration {
    pub requirement: CognitiveCapabilityRequirement,
    pub required_shape: Option<ProviderRealizationShape>,
    pub evidence_snapshot_digest: String,
    pub candidates: Vec<CognitiveCandidateAssessment>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arbitration: Option<CognitiveArbitration>,
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

pub fn cognitive_execution_lane_id(
    case_id: &str,
    participant_id: &str,
    binding: &CaseCognitiveBinding,
) -> Result<String, String> {
    if binding.target_policy.is_some() {
        return Err("arbitrated_lane_requires_exact_target".to_string());
    }
    binding_lane_base(case_id, participant_id, binding)
}

fn binding_lane_base(
    case_id: &str,
    participant_id: &str,
    binding: &CaseCognitiveBinding,
) -> Result<String, String> {
    binding.validate()?;
    if binding.case_id != case_id || binding.participant_id != participant_id {
        return Err("cognitive_lane_binding_scope_mismatch".to_string());
    }
    let material = if binding.role == CognitiveBindingRole::Primary {
        format!(
            "primary\0{}\0{}\0{}",
            case_id, participant_id, binding.binding_id
        )
    } else {
        format!(
            "auxiliary\0{}\0{}\0{}\0{}",
            case_id,
            participant_id,
            binding.capability.as_str(),
            binding.binding_id
        )
    };
    Ok(short_identity(
        "cognitive-lane",
        &digest_of(&material, "cognitive_lane_identity")?,
    ))
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
        arbitration: None,
        provider_realization: ProviderRealizationPosture::DeferredToExecutionAdapter,
        provider_execution: ProviderExecutionPosture::NotPerformed,
    })
}

/// Select first eligible preference, never a provider/model-name score.
/// Snapshot construction belongs to the existing governance owner; this is pure.
pub fn plan_cognitive_execution(
    snapshot: &CognitivePlanningSnapshot,
    requirement: &CognitiveCapabilityRequirement,
) -> Result<CognitiveExecutionPlan, String> {
    plan_cognitive_execution_for_shape(snapshot, requirement, None)
}

pub fn cognitive_execution_lane_for_target(
    case_id: &str,
    participant_id: &str,
    binding: &CaseCognitiveBinding,
    target_id: &str,
) -> Result<String, String> {
    let base = binding_lane_base(case_id, participant_id, binding)?;
    let candidate = binding
        .candidate(target_id)
        .ok_or_else(|| "cognitive_lane_target_not_in_policy".to_string())?;
    if binding.target_policy.is_none() {
        return Ok(base);
    }
    Ok(short_identity(
        "cognitive-lane",
        &digest_of(&(base, candidate), "cognitive_arbitrated_lane")?,
    ))
}

fn assess_binding(
    snapshot: &CognitivePlanningSnapshot,
    binding: &CaseCognitiveBinding,
    requirement: &CognitiveCapabilityRequirement,
    shape: Option<&ProviderRealizationShape>,
) -> Vec<CognitiveCandidateAssessment> {
    binding
        .candidates()
        .into_iter()
        .enumerate()
        .map(|(preference, candidate)| {
            use CognitiveCandidateExclusion::*;
            let mut assessment = CognitiveCandidateAssessment {
                binding_id: binding.binding_id.clone(),
                role: binding.role.clone(),
                preference,
                target_id: candidate.target_id.clone(),
                target_digest: candidate.target_digest.clone(),
                binding_evidence_id: candidate.semantic_evidence_id.clone(),
                required_evidence_id: None,
                execution_evidence: None,
                exclusions: Vec::new(),
            };
            let target = snapshot.targets.iter().find(|target| {
                target.target_id == candidate.target_id
                    && target.target_digest == candidate.target_digest
            });
            let Some(target) = target else {
                assessment.exclusions.push(TargetMissingOrChanged);
                return assessment;
            };
            if !target.provider_envelope_admitted {
                assessment.exclusions.push(ProviderEnvelopeMismatch);
            }
            let evidence = exact_evidence(
                target,
                &binding.capability,
                Some(&candidate.semantic_evidence_id),
            )
            .filter(|evidence| evidence.tenant_id == snapshot.tenant_id);
            if evidence.is_none() {
                assessment.exclusions.push(SemanticEvidenceMissingOrStale);
            }
            let required = if binding.capability == requirement.capability {
                evidence
            } else {
                exact_evidence(target, &requirement.capability, None)
                    .filter(|evidence| evidence.tenant_id == snapshot.tenant_id)
            };
            assessment.required_evidence_id = required.map(|item| item.evidence_id.clone());
            if required.is_none() {
                assessment
                    .exclusions
                    .push(RequiredSemanticSuitabilityMissing);
            }
            if !target.mechanically_qualified {
                assessment
                    .exclusions
                    .push(MechanicalQualificationMissingOrStale);
            }
            if !target.trust_approved {
                assessment.exclusions.push(TrustNotApproved);
            }
            if let Some(shape) = shape {
                if !target
                    .execution_evidence
                    .as_ref()
                    .is_some_and(|e| e.shapes.contains(shape))
                {
                    assessment.exclusions.push(MechanicalShapeUnsupported);
                }
            }
            if let Some(evidence) = &target.execution_evidence {
                if let Some(reason) = &evidence.operational_exclusion {
                    assessment.exclusions.push(reason.clone());
                }
            }
            assessment.execution_evidence = target.execution_evidence.clone();
            assessment
        })
        .collect()
}

pub fn plan_cognitive_execution_for_shape(
    snapshot: &CognitivePlanningSnapshot,
    requirement: &CognitiveCapabilityRequirement,
    shape: Option<&ProviderRealizationShape>,
) -> Result<CognitiveExecutionPlan, String> {
    requirement.validate()?;
    validate_active_cognitive_bindings(&snapshot.active_bindings)?;
    if snapshot.case_id != requirement.case_id
        || snapshot.participant_id != requirement.participant_id
    {
        return Err("cognitive_planning_scope_mismatch".to_string());
    }
    let primary = snapshot.active_bindings.iter().find(|b| {
        b.participant_id == snapshot.participant_id && b.role == CognitiveBindingRole::Primary
    });
    let auxiliary = snapshot.active_bindings.iter().find(|b| {
        b.participant_id == snapshot.participant_id
            && b.role == CognitiveBindingRole::Auxiliary
            && b.capability == requirement.capability
    });
    let mut assessments = Vec::new();
    for binding in primary.into_iter().chain(auxiliary) {
        if binding.case_id != snapshot.case_id || binding.tenant_id != snapshot.tenant_id {
            return Err("cognitive_binding_scope_mismatch".to_string());
        }
        assessments.extend(assess_binding(snapshot, binding, requirement, shape));
    }
    // Auxiliary execution never invents a missing primary semantic slot.
    let primary_viable = assessments.iter().any(|a| {
        a.role == CognitiveBindingRole::Primary
            && a.exclusions.iter().all(|reason| {
                matches!(
                    reason,
                    CognitiveCandidateExclusion::RequiredSemanticSuitabilityMissing
                        | CognitiveCandidateExclusion::MechanicalShapeUnsupported
                )
            })
    });
    let selected = primary
        .filter(|_| primary_viable)
        .and_then(|_| assessments.iter().find(|a| a.exclusions.is_empty()));
    let mut plan = if let Some(selected) = selected {
        let binding = snapshot
            .active_bindings
            .iter()
            .find(|b| b.binding_id == selected.binding_id)
            .unwrap();
        let is_primary = binding.role == CognitiveBindingRole::Primary;
        seal_plan(
            snapshot,
            requirement,
            if is_primary {
                CognitivePlanRoute::Native
            } else {
                CognitivePlanRoute::Derived
            },
            if is_primary {
                CognitivePlanRole::Primary
            } else {
                CognitivePlanRole::Auxiliary
            },
            Some(binding.binding_id.clone()),
            Some(selected.target_id.clone()),
            selected.required_evidence_id.clone(),
            Some(cognitive_execution_lane_for_target(
                &snapshot.case_id,
                &snapshot.participant_id,
                binding,
                &selected.target_id,
            )?),
            None,
        )?
    } else {
        // Preserve v1 diagnostic distinctions for pinned callers.
        use CognitiveCandidateExclusion as E;
        use CognitivePlanUnresolvedReason::*;
        let reason = if primary.is_none() {
            PrimaryBindingMissing
        } else if let Some(first) = assessments.first() {
            if first.exclusions.contains(&E::TargetMissingOrChanged)
                || first
                    .exclusions
                    .contains(&E::SemanticEvidenceMissingOrStale)
            {
                PrimaryBindingStale
            } else if first.exclusions.contains(&E::ProviderEnvelopeMismatch) {
                PrimaryTargetNotAdmitted
            } else if first
                .exclusions
                .contains(&E::MechanicalQualificationMissingOrStale)
            {
                PrimaryProviderQualificationMissing
            } else if first.exclusions.contains(&E::TrustNotApproved) {
                PrimaryTrustNotApproved
            } else if requirement.capability == CognitiveCapability::PrimaryConversation {
                PrimarySuitabilityMissing
            } else if auxiliary.is_none() {
                AuxiliaryBindingMissing
            } else {
                let last = assessments
                    .iter()
                    .find(|a| a.role == CognitiveBindingRole::Auxiliary)
                    .unwrap();
                if last.exclusions.contains(&E::ProviderEnvelopeMismatch) {
                    AuxiliaryTargetNotAdmitted
                } else if last.exclusions.contains(&E::TrustNotApproved) {
                    AuxiliaryTrustNotApproved
                } else if last
                    .exclusions
                    .contains(&E::MechanicalQualificationMissingOrStale)
                {
                    AuxiliaryProviderQualificationMissing
                } else if last.exclusions.contains(&E::TargetMissingOrChanged) {
                    AuxiliaryBindingStale
                } else {
                    AuxiliarySuitabilityMissing
                }
            }
        } else {
            PrimaryBindingMissing
        };
        unresolved(snapshot, requirement, reason)?
    };
    let snapshot_digest = digest_of(
        &(
            &snapshot.tenant_id,
            &snapshot.case_id,
            &snapshot.participant_id,
            &requirement.capability,
            shape,
            &assessments,
        ),
        "cognitive_arbitration_snapshot",
    )?;
    let arbitration = CognitiveArbitration {
        requirement: requirement.clone(),
        required_shape: shape.cloned(),
        evidence_snapshot_digest: snapshot_digest,
        candidates: assessments,
    };
    plan.integrity_digest = digest_of(
        &(&plan.integrity_digest, &arbitration),
        "cognitive_arbitration_plan",
    )?;
    plan.plan_id = short_identity("cognitive-plan", &plan.integrity_digest);
    plan.arbitration = Some(arbitration);
    Ok(plan)
}

/// Shared I03 contract: purpose cannot be changed to evade prior-delivery checks.
pub fn cognitive_provider_requirement(
    cognitive: &CognitiveCapabilityRequirement,
) -> Result<crate::provider_governance::ProviderRequirement, String> {
    use crate::provider_governance::{
        CapabilityProvenance, ProviderCapability, ProviderCapabilityRequirement,
        ProviderRequirement,
    };
    cognitive.validate()?;
    ProviderRequirement::new(
        &format!("cognitive_realization:{}", cognitive.requirement_id),
        vec![
            ProviderCapabilityRequirement {
                capability: ProviderCapability::ChatText,
                minimum_provenance: CapabilityProvenance::Qualified,
            },
            ProviderCapabilityRequirement {
                capability: ProviderCapability::ModelExactAddressing,
                minimum_provenance: CapabilityProvenance::Qualified,
            },
        ],
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

// Cognitive Decision Plane contracts are derived execution meaning. They are
// deliberately separate from canonical control::Decision and DecisionBasis:
// a score can inform a later proposal, but cannot authorize or mutate a Case.
pub const COGNITIVE_DECISION_REQUEST_SCHEMA: &str = "yai.cognitive_decision_request.v1";
pub const COGNITIVE_DECISION_OUTPUT_SCHEMA: &str = "yai.cognitive_decision_output.v1";
pub const COGNITIVE_DECISION_DISTRIBUTION_SCHEMA: &str = "yai.cognitive_decision_distribution.v1";
pub const COGNITIVE_DECISION_QUALIFICATION_SCHEMA: &str = "yai.cognitive_decision_qualification.v1";
pub const MAX_DECISION_CANDIDATES: usize = 32;
pub const MAX_DECISION_REFS_PER_CANDIDATE: usize = 16;
pub const MAX_DECISION_PRODUCER_EVIDENCE_REFS: usize = 16;

fn require_bounded_text(label: &str, value: &str, max: usize) -> Result<(), String> {
    if value.is_empty() || value.len() > max || value.chars().any(char::is_control) {
        return Err(format!("{label}_invalid"));
    }
    Ok(())
}

/// One finite semantic alternative. `candidate_kind` is namespaced rather than
/// a closed enum so YAI does not freeze every future choice family in v1.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveDecisionCandidate {
    pub candidate_id: String,
    pub candidate_kind: String,
    pub description: String,
    #[serde(default)]
    pub semantic_refs: Vec<String>,
}

impl CognitiveDecisionCandidate {
    fn normalized(mut self) -> Result<Self, String> {
        require_identifier("cognitive_decision_candidate", &self.candidate_id, 256)?;
        require_identifier(
            "cognitive_decision_candidate_kind",
            &self.candidate_kind,
            128,
        )?;
        require_bounded_text(
            "cognitive_decision_candidate_description",
            &self.description,
            512,
        )?;
        self.semantic_refs.sort();
        self.semantic_refs.dedup();
        if self.semantic_refs.len() > MAX_DECISION_REFS_PER_CANDIDATE
            || self.semantic_refs.iter().any(|reference| {
                reference.is_empty()
                    || reference.len() > 512
                    || reference.chars().any(char::is_control)
            })
        {
            return Err("cognitive_decision_candidate_refs_invalid".to_string());
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveDecisionBudget {
    pub max_candidates: usize,
    pub max_result_bytes: usize,
    pub max_compute_millis: u64,
}

impl CognitiveDecisionBudget {
    pub fn validate(&self) -> Result<(), String> {
        if !(2..=MAX_DECISION_CANDIDATES).contains(&self.max_candidates)
            || self.max_result_bytes == 0
            || self.max_result_bytes > 1024 * 1024
            || self.max_compute_millis == 0
            || self.max_compute_millis > 600_000
        {
            return Err("cognitive_decision_budget_invalid".to_string());
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct CognitiveDecisionRequestIdentity<'a> {
    schema: &'a str,
    case_id: &'a str,
    case_generation: u64,
    participant_id: &'a str,
    task_id: &'a str,
    working_state_id: &'a str,
    decision_kind: &'a str,
    candidates: &'a [CognitiveDecisionCandidate],
    budget: &'a CognitiveDecisionBudget,
}

/// Exact W/task-bound semantic request. It contains no producer/runtime shape
/// and possession of it is never effect authority.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveDecisionRequest {
    pub schema: String,
    pub request_id: String,
    pub case_id: String,
    pub case_generation: u64,
    pub participant_id: String,
    pub task_id: String,
    pub working_state_id: String,
    pub decision_kind: String,
    pub candidates: Vec<CognitiveDecisionCandidate>,
    pub budget: CognitiveDecisionBudget,
}

impl CognitiveDecisionRequest {
    pub fn new(
        working: &crate::semantic_state::SemanticWorkingState,
        decision_kind: impl Into<String>,
        candidates: Vec<CognitiveDecisionCandidate>,
        budget: CognitiveDecisionBudget,
    ) -> Result<Self, String> {
        working.validate_refresh_envelope()?;
        budget.validate()?;
        let decision_kind = decision_kind.into();
        require_identifier("cognitive_decision_kind", &decision_kind, 128)?;
        if candidates.len() < 2 || candidates.len() > budget.max_candidates {
            return Err("cognitive_decision_candidate_bound".to_string());
        }
        let mut candidates = candidates
            .into_iter()
            .map(CognitiveDecisionCandidate::normalized)
            .collect::<Result<Vec<_>, _>>()?;
        candidates.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
        if candidates
            .windows(2)
            .any(|pair| pair[0].candidate_id == pair[1].candidate_id)
        {
            return Err("cognitive_decision_candidate_id_duplicate".to_string());
        }
        // An abstract next-step candidate may have no refs. Once it names Case
        // material, every ref must already be resident in this qualified W.
        // Hidden, foreign and merely deferred identifiers fail identically.
        if candidates.iter().any(|candidate| {
            candidate
                .semantic_refs
                .iter()
                .any(|reference| !working.contains_resident_reference(reference))
        }) {
            return Err("cognitive_decision_candidate_reference_unavailable".to_string());
        }
        let task_id = working.semantic_task_id()?;
        let participant_id = working.request().scope.participant_id.clone();
        let identity = CognitiveDecisionRequestIdentity {
            schema: COGNITIVE_DECISION_REQUEST_SCHEMA,
            case_id: working.case_id(),
            case_generation: working.generation(),
            participant_id: &participant_id,
            task_id: &task_id,
            working_state_id: working.id(),
            decision_kind: &decision_kind,
            candidates: &candidates,
            budget: &budget,
        };
        let digest = digest_of(&identity, "cognitive_decision_request_identity")?;
        Ok(Self {
            schema: COGNITIVE_DECISION_REQUEST_SCHEMA.to_string(),
            request_id: short_identity("cognitive-decision-request", &digest),
            case_id: working.case_id().to_string(),
            case_generation: working.generation(),
            participant_id,
            task_id,
            working_state_id: working.id().to_string(),
            decision_kind,
            candidates,
            budget,
        })
    }

    pub fn validate_against(
        &self,
        working: &crate::semantic_state::SemanticWorkingState,
    ) -> Result<(), String> {
        let rebuilt = Self::new(
            working,
            self.decision_kind.clone(),
            self.candidates.clone(),
            self.budget.clone(),
        )?;
        if rebuilt != *self {
            return Err("cognitive_decision_request_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

/// Exact producer/capability identity supplied by the execution boundary. The
/// optional model/composition/profile/state identities do not grant authority.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveDecisionProducerIdentity {
    pub capability_id: String,
    pub producer_id: String,
    pub producer_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_generation: Option<String>,
    pub evidence_refs: Vec<String>,
}

impl CognitiveDecisionProducerIdentity {
    pub fn validate(&self) -> Result<(), String> {
        for (label, value) in [
            ("cognitive_decision_capability", self.capability_id.as_str()),
            ("cognitive_decision_producer", self.producer_id.as_str()),
            (
                "cognitive_decision_producer_version",
                self.producer_version.as_str(),
            ),
        ] {
            require_identifier(label, value, 256)?;
        }
        for (label, value) in [
            ("cognitive_decision_model", &self.model_id),
            ("cognitive_decision_composition", &self.composition_id),
            ("cognitive_decision_profile", &self.profile_id),
            (
                "cognitive_decision_state_generation",
                &self.state_generation,
            ),
        ] {
            if let Some(value) = value {
                require_identifier(label, value, 256)?;
            }
        }
        if self.evidence_refs.is_empty()
            || self.evidence_refs.len() > MAX_DECISION_PRODUCER_EVIDENCE_REFS
        {
            return Err("cognitive_decision_producer_evidence_invalid".to_string());
        }
        let mut evidence = self.evidence_refs.clone();
        evidence.sort();
        if evidence.windows(2).any(|pair| pair[0] == pair[1])
            || evidence.iter().any(|reference| {
                reference.is_empty()
                    || reference.len() > 512
                    || reference.chars().any(char::is_control)
            })
        {
            return Err("cognitive_decision_producer_evidence_invalid".to_string());
        }
        Ok(())
    }
}

/// Mechanical meaning of the numeric vector. Normalization alone never claims
/// empirical calibration or correctness confidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CognitiveScoreSemantics {
    RawScore {
        measure: String,
    },
    RelativeCandidateProbability {
        normalization: String,
    },
    CalibratedProbability {
        calibration_artifact_id: String,
        calibration_scope: String,
    },
}

impl CognitiveScoreSemantics {
    fn validate(&self) -> Result<(), String> {
        match self {
            Self::RawScore { measure } => {
                require_identifier("cognitive_decision_raw_measure", measure, 128)
            }
            Self::RelativeCandidateProbability { normalization } => {
                require_identifier("cognitive_decision_normalization", normalization, 128)
            }
            Self::CalibratedProbability {
                calibration_artifact_id,
                calibration_scope,
            } => {
                require_identifier(
                    "cognitive_decision_calibration_artifact",
                    calibration_artifact_id,
                    256,
                )?;
                require_identifier(
                    "cognitive_decision_calibration_scope",
                    calibration_scope,
                    256,
                )
            }
        }
    }

    fn requires_distribution(&self) -> bool {
        !matches!(self, Self::RawScore { .. })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveScoreDirection {
    HigherIsPreferred,
    LowerIsPreferred,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveDecisionScore {
    pub candidate_id: String,
    /// Fixed-point numerator under `score_scale`; raw scores may be signed.
    pub value: i64,
}

/// Optional producer-declared metric kept distinct from candidate scores and
/// from calibrated correctness probability.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveUncertaintyMetric {
    pub metric_id: String,
    pub value: u64,
    pub scale: u64,
    pub interpretation: String,
}

impl CognitiveUncertaintyMetric {
    fn validate(&self) -> Result<(), String> {
        require_identifier(
            "cognitive_decision_uncertainty_metric",
            &self.metric_id,
            128,
        )?;
        require_bounded_text(
            "cognitive_decision_uncertainty_interpretation",
            &self.interpretation,
            256,
        )?;
        if self.scale == 0 {
            return Err("cognitive_decision_uncertainty_scale_invalid".to_string());
        }
        Ok(())
    }
}

/// Producer output before YAI interpretation. This is not a shared YVEX ABI and
/// cannot be converted to a canonical Decision or effect directly.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveDecisionOutput {
    pub schema: String,
    pub request_id: String,
    pub producer: CognitiveDecisionProducerIdentity,
    pub score_semantics: CognitiveScoreSemantics,
    pub score_direction: CognitiveScoreDirection,
    pub score_scale: u64,
    pub scores: Vec<CognitiveDecisionScore>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<CognitiveUncertaintyMetric>,
}

#[derive(Serialize)]
struct CognitiveDecisionDistributionIdentity<'a> {
    schema: &'a str,
    request_id: &'a str,
    working_state_id: &'a str,
    producer: &'a CognitiveDecisionProducerIdentity,
    score_semantics: &'a CognitiveScoreSemantics,
    score_direction: &'a CognitiveScoreDirection,
    score_scale: u64,
    scores: &'a [CognitiveDecisionScore],
    uncertainty: &'a Option<CognitiveUncertaintyMetric>,
}

/// Qualified, non-authoritative interpretation of an exact finite score set.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveDecisionDistribution {
    pub schema: String,
    pub distribution_id: String,
    pub request: CognitiveDecisionRequest,
    pub producer: CognitiveDecisionProducerIdentity,
    pub score_semantics: CognitiveScoreSemantics,
    pub score_direction: CognitiveScoreDirection,
    pub score_scale: u64,
    pub scores: Vec<CognitiveDecisionScore>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<CognitiveUncertaintyMetric>,
}

impl CognitiveDecisionDistribution {
    pub fn qualify(
        request: &CognitiveDecisionRequest,
        mut output: CognitiveDecisionOutput,
    ) -> Result<Self, String> {
        if output.schema != COGNITIVE_DECISION_OUTPUT_SCHEMA
            || output.request_id != request.request_id
        {
            return Err("cognitive_decision_output_request_mismatch".to_string());
        }
        output.producer.validate()?;
        output.score_semantics.validate()?;
        if let CognitiveScoreSemantics::CalibratedProbability {
            calibration_artifact_id,
            ..
        } = &output.score_semantics
        {
            if !output
                .producer
                .evidence_refs
                .contains(calibration_artifact_id)
            {
                return Err("cognitive_decision_calibration_evidence_missing".to_string());
            }
        }
        if output.score_scale == 0 || output.score_scale > 1_000_000_000 {
            return Err("cognitive_decision_score_scale_invalid".to_string());
        }
        if output.score_semantics.requires_distribution()
            && output.score_direction != CognitiveScoreDirection::HigherIsPreferred
        {
            return Err("cognitive_decision_probability_direction_invalid".to_string());
        }
        output
            .scores
            .sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
        if output.scores.len() != request.candidates.len()
            || output
                .scores
                .windows(2)
                .any(|pair| pair[0].candidate_id == pair[1].candidate_id)
            || output
                .scores
                .iter()
                .zip(&request.candidates)
                .any(|(score, candidate)| score.candidate_id != candidate.candidate_id)
        {
            return Err("cognitive_decision_scores_not_candidate_complete".to_string());
        }
        if output.score_semantics.requires_distribution() {
            if output
                .scores
                .iter()
                .any(|score| score.value < 0 || score.value as u64 > output.score_scale)
                || output
                    .scores
                    .iter()
                    .map(|score| i128::from(score.value))
                    .sum::<i128>()
                    != i128::from(output.score_scale)
            {
                return Err("cognitive_decision_probability_distribution_invalid".to_string());
            }
        }
        if let Some(uncertainty) = &output.uncertainty {
            uncertainty.validate()?;
            if uncertainty.value > uncertainty.scale {
                return Err("cognitive_decision_uncertainty_value_invalid".to_string());
            }
        }
        let encoded = serde_json::to_vec(&output)
            .map_err(|error| format!("cognitive_decision_output_encode_failed: {error}"))?;
        if encoded.len() > request.budget.max_result_bytes {
            return Err("cognitive_decision_result_budget_exceeded".to_string());
        }
        let identity = CognitiveDecisionDistributionIdentity {
            schema: COGNITIVE_DECISION_DISTRIBUTION_SCHEMA,
            request_id: &request.request_id,
            working_state_id: &request.working_state_id,
            producer: &output.producer,
            score_semantics: &output.score_semantics,
            score_direction: &output.score_direction,
            score_scale: output.score_scale,
            scores: &output.scores,
            uncertainty: &output.uncertainty,
        };
        let digest = digest_of(&identity, "cognitive_decision_distribution_identity")?;
        Ok(Self {
            schema: COGNITIVE_DECISION_DISTRIBUTION_SCHEMA.to_string(),
            distribution_id: short_identity("cognitive-decision-distribution", &digest),
            request: request.clone(),
            producer: output.producer,
            score_semantics: output.score_semantics,
            score_direction: output.score_direction,
            score_scale: output.score_scale,
            scores: output.scores,
            uncertainty: output.uncertainty,
        })
    }

    pub fn preferred_candidate_id(&self) -> Result<&str, String> {
        let mut ranked = self.scores.iter().collect::<Vec<_>>();
        ranked.sort_by(|left, right| {
            let order = left.value.cmp(&right.value);
            let order = match self.score_direction {
                CognitiveScoreDirection::HigherIsPreferred => order.reverse(),
                CognitiveScoreDirection::LowerIsPreferred => order,
            };
            order.then_with(|| left.candidate_id.cmp(&right.candidate_id))
        });
        let [first, rest @ ..] = ranked.as_slice() else {
            return Err("cognitive_decision_distribution_empty".to_string());
        };
        if rest
            .first()
            .is_some_and(|second| second.value == first.value)
        {
            return Err("cognitive_decision_preference_tied".to_string());
        }
        Ok(&first.candidate_id)
    }
}

/// Typed application result. Measurements are deliberately outside semantic
/// distribution identity and carry no authority.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveDecisionQualification {
    pub schema: String,
    pub distribution: CognitiveDecisionDistribution,
    pub current_requalification_us: u128,
    pub output_bytes: usize,
}

// The frontier is a derived, model-free input to the Decision Plane. It names
// only opportunities already established by typed Case owners; it neither
// scores them nor makes them executable.
pub const COGNITIVE_DECISION_FRONTIER_REQUEST_SCHEMA: &str =
    "yai.cognitive_decision_frontier_request.v1";
pub const COGNITIVE_DECISION_FRONTIER_SCHEMA: &str = "yai.cognitive_decision_frontier.v1";
pub const COGNITIVE_DECISION_FRONTIER_QUALIFICATION_SCHEMA: &str =
    "yai.cognitive_decision_frontier_qualification.v1";
pub const COGNITIVE_DECISION_FRONTIER_PROFILE: &str = "yai.current_typed_opportunities.v1";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveDecisionCandidateRequirement {
    Required,
    Optional,
}

/// Exact typed reason why a candidate exists. No variant is inferred from
/// arbitrary prose. Workflow variants are qualified by the existing Workflow
/// resolver; task Resources and paging references are qualified by W.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CognitiveDecisionCandidateOrigin {
    WorkflowReadyWork {
        workflow_definition_id: String,
        workflow_binding_id: String,
        effective_topology_digest: String,
        node_id: String,
        node_kind: String,
        topological_rank: usize,
    },
    WorkflowResolvableProgress {
        workflow_definition_id: String,
        workflow_binding_id: String,
        effective_topology_digest: String,
        node_id: String,
        node_kind: String,
        resolution_reason: String,
        evidence_refs: Vec<String>,
    },
    TaskResource {
        resource_ref: String,
        exact_required: bool,
    },
    DeferredSemanticGroup {
        reference_id: String,
        group_entry_id: String,
        family: String,
        evidence_digest: String,
        mandatory_task_dependency: bool,
    },
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CognitiveDecisionCandidateSubject<'a> {
    WorkflowReadyWork {
        workflow_definition_id: &'a str,
        workflow_binding_id: &'a str,
        effective_topology_digest: &'a str,
        node_id: &'a str,
    },
    WorkflowResolvableProgress {
        workflow_definition_id: &'a str,
        workflow_binding_id: &'a str,
        effective_topology_digest: &'a str,
        node_id: &'a str,
    },
    TaskResource {
        resource_ref: &'a str,
    },
    DeferredSemanticGroup {
        reference_id: &'a str,
    },
}

impl CognitiveDecisionCandidateOrigin {
    fn requirement(&self) -> CognitiveDecisionCandidateRequirement {
        match self {
            Self::WorkflowReadyWork { .. } | Self::WorkflowResolvableProgress { .. } => {
                CognitiveDecisionCandidateRequirement::Required
            }
            Self::TaskResource { exact_required, .. } => {
                if *exact_required {
                    CognitiveDecisionCandidateRequirement::Required
                } else {
                    CognitiveDecisionCandidateRequirement::Optional
                }
            }
            Self::DeferredSemanticGroup {
                mandatory_task_dependency,
                ..
            } => {
                if *mandatory_task_dependency {
                    CognitiveDecisionCandidateRequirement::Required
                } else {
                    CognitiveDecisionCandidateRequirement::Optional
                }
            }
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            Self::WorkflowReadyWork { .. } | Self::WorkflowResolvableProgress { .. } => 0,
            Self::TaskResource { .. } => 1,
            Self::DeferredSemanticGroup { .. } => 2,
        }
    }

    fn candidate_material(&self) -> (&'static str, &'static str, Vec<String>) {
        match self {
            Self::WorkflowReadyWork { .. } => (
                "yai.frontier.workflow-ready-work.v1",
                "Advance an exact ready Workflow work item",
                Vec::new(),
            ),
            Self::WorkflowResolvableProgress { .. } => (
                "yai.frontier.workflow-resolvable-progress.v1",
                "Advance exact mechanically resolvable Workflow progression",
                Vec::new(),
            ),
            Self::TaskResource { resource_ref, .. } => (
                "yai.frontier.inspect-task-resource.v1",
                "Inspect an exact task-bound Resource",
                vec![resource_ref.clone()],
            ),
            Self::DeferredSemanticGroup { .. } => (
                "yai.frontier.expand-semantic-group.v1",
                "Expand one exact deferred semantic group",
                Vec::new(),
            ),
        }
    }

    fn candidate_subject(&self) -> CognitiveDecisionCandidateSubject<'_> {
        match self {
            Self::WorkflowReadyWork {
                workflow_definition_id,
                workflow_binding_id,
                effective_topology_digest,
                node_id,
                ..
            } => CognitiveDecisionCandidateSubject::WorkflowReadyWork {
                workflow_definition_id,
                workflow_binding_id,
                effective_topology_digest,
                node_id,
            },
            Self::WorkflowResolvableProgress {
                workflow_definition_id,
                workflow_binding_id,
                effective_topology_digest,
                node_id,
                ..
            } => CognitiveDecisionCandidateSubject::WorkflowResolvableProgress {
                workflow_definition_id,
                workflow_binding_id,
                effective_topology_digest,
                node_id,
            },
            Self::TaskResource { resource_ref, .. } => {
                CognitiveDecisionCandidateSubject::TaskResource { resource_ref }
            }
            Self::DeferredSemanticGroup { reference_id, .. } => {
                CognitiveDecisionCandidateSubject::DeferredSemanticGroup { reference_id }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveDecisionFrontierRequest {
    pub schema: String,
    pub request_id: String,
    pub case_id: String,
    pub case_generation: u64,
    pub participant_id: String,
    pub task_id: String,
    pub working_state_id: String,
    pub profile: String,
    pub max_candidates: usize,
}

#[derive(Serialize)]
struct CognitiveDecisionFrontierRequestIdentity<'a> {
    schema: &'a str,
    case_id: &'a str,
    case_generation: u64,
    participant_id: &'a str,
    task_id: &'a str,
    working_state_id: &'a str,
    profile: &'a str,
    max_candidates: usize,
}

impl CognitiveDecisionFrontierRequest {
    pub fn new(
        working: &crate::semantic_state::SemanticWorkingState,
        max_candidates: usize,
    ) -> Result<Self, String> {
        working.validate_refresh_envelope()?;
        if !(2..=MAX_DECISION_CANDIDATES).contains(&max_candidates) {
            return Err("cognitive_decision_frontier_bound_invalid".to_string());
        }
        let task_id = working.semantic_task_id()?;
        let participant_id = working.request().scope.participant_id.clone();
        let identity = CognitiveDecisionFrontierRequestIdentity {
            schema: COGNITIVE_DECISION_FRONTIER_REQUEST_SCHEMA,
            case_id: working.case_id(),
            case_generation: working.generation(),
            participant_id: &participant_id,
            task_id: &task_id,
            working_state_id: working.id(),
            profile: COGNITIVE_DECISION_FRONTIER_PROFILE,
            max_candidates,
        };
        let digest = digest_of(&identity, "cognitive_decision_frontier_request_identity")?;
        Ok(Self {
            schema: COGNITIVE_DECISION_FRONTIER_REQUEST_SCHEMA.to_string(),
            request_id: short_identity("cognitive-decision-frontier-request", &digest),
            case_id: working.case_id().to_string(),
            case_generation: working.generation(),
            participant_id,
            task_id,
            working_state_id: working.id().to_string(),
            profile: COGNITIVE_DECISION_FRONTIER_PROFILE.to_string(),
            max_candidates,
        })
    }

    pub fn validate_against(
        &self,
        working: &crate::semantic_state::SemanticWorkingState,
    ) -> Result<(), String> {
        if Self::new(working, self.max_candidates)? != *self {
            return Err("cognitive_decision_frontier_request_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveDecisionFrontierCandidate {
    pub candidate: CognitiveDecisionCandidate,
    pub requirement: CognitiveDecisionCandidateRequirement,
    pub origins: Vec<CognitiveDecisionCandidateOrigin>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CognitiveDecisionFrontierCompleteness {
    CompleteAtTypedProfile,
    OptionalCandidatesOmitted { omitted_count: usize },
}

#[derive(Serialize)]
struct CognitiveDecisionFrontierIdentity<'a> {
    schema: &'a str,
    request: &'a CognitiveDecisionFrontierRequest,
    candidates: &'a [CognitiveDecisionFrontierCandidate],
    completeness: &'a CognitiveDecisionFrontierCompleteness,
    visible_candidate_count: usize,
    required_candidate_count: usize,
    optional_candidate_count: usize,
    omitted_optional_candidates: usize,
    visible_origin_count: usize,
}

/// Reconstructible current frontier. Candidate order is canonical
/// representation only; membership and typed origins define meaning.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveDecisionFrontier {
    pub schema: String,
    pub frontier_id: String,
    pub request: CognitiveDecisionFrontierRequest,
    pub candidates: Vec<CognitiveDecisionFrontierCandidate>,
    pub completeness: CognitiveDecisionFrontierCompleteness,
    pub visible_candidate_count: usize,
    pub required_candidate_count: usize,
    pub optional_candidate_count: usize,
    pub omitted_optional_candidates: usize,
    pub visible_origin_count: usize,
}

impl CognitiveDecisionFrontier {
    pub(crate) fn derive(
        request: &CognitiveDecisionFrontierRequest,
        working: &crate::semantic_state::SemanticWorkingState,
        mut workflow_origins: Vec<CognitiveDecisionCandidateOrigin>,
    ) -> Result<Self, String> {
        request.validate_against(working)?;
        let mut origins = Vec::new();
        origins.append(&mut workflow_origins);

        let required_refs = working
            .request()
            .required_refs
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        for resource_ref in &working.request().resource_refs {
            if working.contains_resident_reference(resource_ref) {
                origins.push(CognitiveDecisionCandidateOrigin::TaskResource {
                    resource_ref: resource_ref.clone(),
                    exact_required: required_refs.contains(resource_ref),
                });
            }
        }

        let resident = working
            .resident_page_references()
            .into_iter()
            .collect::<BTreeSet<_>>();
        for reference in working.page_references() {
            if !resident.contains(&reference.reference_id) {
                origins.push(CognitiveDecisionCandidateOrigin::DeferredSemanticGroup {
                    reference_id: reference.reference_id.clone(),
                    group_entry_id: reference.group_entry_id.clone(),
                    family: reference.family.clone(),
                    evidence_digest: reference.evidence_digest.clone(),
                    mandatory_task_dependency: reference.mandatory_task_dependency,
                });
            }
        }

        origins.sort();
        origins.dedup();
        let visible_origin_count = origins.len();
        let mut candidates = BTreeMap::<String, CognitiveDecisionFrontierCandidate>::new();
        for origin in origins {
            let (candidate_kind, description, semantic_refs) = origin.candidate_material();
            let subject_digest = digest_of(
                &origin.candidate_subject(),
                "cognitive_decision_frontier_candidate_subject",
            )?;
            let candidate_id = short_identity("cognitive-frontier-candidate", &subject_digest);
            let requirement = origin.requirement();
            let entry = candidates.entry(candidate_id.clone()).or_insert_with(|| {
                CognitiveDecisionFrontierCandidate {
                    candidate: CognitiveDecisionCandidate {
                        candidate_id,
                        candidate_kind: candidate_kind.to_string(),
                        description: description.to_string(),
                        semantic_refs,
                    },
                    requirement: requirement.clone(),
                    origins: Vec::new(),
                }
            });
            if requirement == CognitiveDecisionCandidateRequirement::Required {
                entry.requirement = CognitiveDecisionCandidateRequirement::Required;
            }
            entry.origins.push(origin);
            entry.origins.sort();
            entry.origins.dedup();
        }

        let mut required = Vec::new();
        let mut optional = Vec::new();
        for candidate in candidates.into_values() {
            if candidate.requirement == CognitiveDecisionCandidateRequirement::Required {
                required.push(candidate);
            } else {
                optional.push(candidate);
            }
        }
        if required.len() > request.max_candidates {
            return Err("cognitive_decision_frontier_required_bound_exceeded".to_string());
        }
        optional.sort_by(|left, right| {
            let left_precedence = left.origins.iter().map(|origin| origin.precedence()).min();
            let right_precedence = right.origins.iter().map(|origin| origin.precedence()).min();
            left_precedence.cmp(&right_precedence).then(
                left.candidate
                    .candidate_id
                    .cmp(&right.candidate.candidate_id),
            )
        });
        let visible_candidate_count = required.len() + optional.len();
        let required_candidate_count = required.len();
        let optional_candidate_count = optional.len();
        let optional_capacity = request.max_candidates - required.len();
        let omitted_optional_candidates = optional.len().saturating_sub(optional_capacity);
        required.extend(optional.into_iter().take(optional_capacity));
        if required.len() < 2 {
            return Err("cognitive_decision_frontier_insufficient_candidates".to_string());
        }
        required.sort_by(|left, right| {
            left.candidate
                .candidate_id
                .cmp(&right.candidate.candidate_id)
        });
        let completeness = if omitted_optional_candidates == 0 {
            CognitiveDecisionFrontierCompleteness::CompleteAtTypedProfile
        } else {
            CognitiveDecisionFrontierCompleteness::OptionalCandidatesOmitted {
                omitted_count: omitted_optional_candidates,
            }
        };
        let identity = CognitiveDecisionFrontierIdentity {
            schema: COGNITIVE_DECISION_FRONTIER_SCHEMA,
            request,
            candidates: &required,
            completeness: &completeness,
            visible_candidate_count,
            required_candidate_count,
            optional_candidate_count,
            omitted_optional_candidates,
            visible_origin_count,
        };
        let digest = digest_of(&identity, "cognitive_decision_frontier_identity")?;
        Ok(Self {
            schema: COGNITIVE_DECISION_FRONTIER_SCHEMA.to_string(),
            frontier_id: short_identity("cognitive-decision-frontier", &digest),
            request: request.clone(),
            candidates: required,
            completeness,
            visible_candidate_count,
            required_candidate_count,
            optional_candidate_count,
            omitted_optional_candidates,
            visible_origin_count,
        })
    }

    pub fn decision_candidates(&self) -> Vec<CognitiveDecisionCandidate> {
        self.candidates
            .iter()
            .map(|item| item.candidate.clone())
            .collect()
    }
}

/// Typed application result. Measurements are not part of frontier identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveDecisionFrontierQualification {
    pub schema: String,
    pub frontier: CognitiveDecisionFrontier,
    pub current_requalification_us: u128,
    pub candidate_generation_us: u128,
    pub origin_count: usize,
    pub output_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arbitration_never_infers_capability_and_reports_exact_exclusions() {
        let a = evidence(
            "provider-target:whisper-best",
            "sha256:a",
            CognitiveCapability::PrimaryConversation,
        );
        let b = evidence(
            "provider-target:plain",
            "sha256:b",
            CognitiveCapability::PrimaryConversation,
        );
        let pinned = binding(
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            &a.target_id,
            &a.target_digest,
            &a.evidence_id,
            2,
            None,
        );
        let ordered = pinned
            .with_ordered_alternatives(vec![CognitiveTargetCandidate {
                target_id: b.target_id.clone(),
                target_digest: b.target_digest.clone(),
                semantic_evidence_id: b.evidence_id.clone(),
            }])
            .unwrap();
        let mut snapshot = CognitivePlanningSnapshot {
            tenant_id: "tenant:test".into(),
            case_id: "case:test".into(),
            participant_id: "participant:model".into(),
            case_generation: 3,
            active_bindings: vec![ordered],
            targets: vec![
                target(&a.target_id, &a.target_digest, vec![a.clone()]),
                target(&b.target_id, &b.target_digest, vec![b.clone()]),
            ],
        };
        let req = CognitiveCapabilityRequirement::new(
            "case:test",
            "participant:model",
            CognitiveCapability::PrimaryConversation,
            "source:test",
        )
        .unwrap();
        snapshot.targets[0].semantic_evidence.clear();
        let plan = plan_cognitive_execution(&snapshot, &req).unwrap();
        assert_eq!(
            plan.selected_target_id.as_deref(),
            Some("provider-target:plain")
        );
        assert!(plan.arbitration.as_ref().unwrap().candidates[0]
            .exclusions
            .contains(&CognitiveCandidateExclusion::SemanticEvidenceMissingOrStale));
        snapshot.targets[0].semantic_evidence.push(a);
        snapshot.targets[0].mechanically_qualified = false;
        let plan = plan_cognitive_execution(&snapshot, &req).unwrap();
        assert_eq!(
            plan.selected_target_id.as_deref(),
            Some("provider-target:plain")
        );
        assert!(plan.arbitration.as_ref().unwrap().candidates[0]
            .exclusions
            .contains(&CognitiveCandidateExclusion::MechanicalQualificationMissingOrStale));
        snapshot.targets[0].target_digest = "sha256:replaced".into();
        let plan = plan_cognitive_execution(&snapshot, &req).unwrap();
        assert!(plan.arbitration.as_ref().unwrap().candidates[0]
            .exclusions
            .contains(&CognitiveCandidateExclusion::TargetMissingOrChanged));
        assert!(CognitiveCapability::parse("whisper").is_err());
        let mut historical_plan = serde_json::to_value(&plan).unwrap();
        historical_plan["schema"] = serde_json::json!("yai.cognitive_execution_plan.v1");
        historical_plan["planner_version"] =
            serde_json::json!("yai.cognitive_execution_planner.v1");
        historical_plan
            .as_object_mut()
            .unwrap()
            .remove("arbitration");
        let historical: CognitiveExecutionPlan = serde_json::from_value(historical_plan).unwrap();
        assert!(historical.arbitration.is_none());
        let mut unknown = serde_json::to_value(&snapshot.active_bindings[0]).unwrap();
        unknown["target_policy"]["kind"] = serde_json::json!("learned_best");
        assert!(serde_json::from_value::<CaseCognitiveBinding>(unknown).is_err());
    }

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
            execution_evidence: None,
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
            arbitration: None,
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
