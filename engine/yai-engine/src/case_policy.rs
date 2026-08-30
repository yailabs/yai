//! Case-native policy bindings and deterministic EffectivePolicy materialization.
//!
//! Canonical binding history is carried by Transition payloads. This module
//! owns binding integrity and the rebuildable composition algorithm; it never
//! evaluates an Operation or emits authority, Decisions, Grants, or effects.

use crate::effect::digest_bytes;
use crate::governance::{
    AuthoritySubject, EvidenceObligationKind, NormalizedPolicyRule, PolicyArtifact, PolicyEffect,
    PolicyLineage, PolicyValidityContract,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CASE_POLICY_BINDING_SCHEMA: &str = "yai.case_policy_binding.v2";
pub const CASE_POLICY_BINDING_SCHEMA_V1: &str = "yai.case_policy_binding.v1";
pub const EFFECTIVE_POLICY_SCHEMA: &str = "yai.effective_policy.v3";
pub const EFFECTIVE_POLICY_SCHEMA_V2: &str = "yai.effective_policy.v2";
pub const EFFECTIVE_POLICY_SCHEMA_V1: &str = "yai.effective_policy.v1";
pub const POLICY_MATERIALIZER_VERSION: &str = "yai.policy_materializer.v3";
pub const POLICY_MATERIALIZER_VERSION_V2: &str = "yai.policy_materializer.v2";
pub const POLICY_MATERIALIZER_VERSION_V1: &str = "yai.policy_materializer.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CasePolicyBinding {
    pub schema: String,
    pub binding_id: String,
    pub integrity_digest: String,
    pub case_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    pub lineage_id: String,
    pub owner_ref: String,
    pub policy_key: String,
    pub artifact_id: String,
    pub artifact_version: String,
    pub source_id: String,
    pub source_digest: String,
    pub policy_ir_digest: String,
    pub publication_event_id: String,
    pub publication_event_sequence: u64,
    pub bound_at_case_generation: u64,
    pub actor_ref: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaces_binding_id: Option<String>,
}

#[derive(Serialize)]
struct BindingDigestMaterial<'a> {
    schema: &'a str,
    case_id: &'a str,
    lineage_id: &'a str,
    owner_ref: &'a str,
    policy_key: &'a str,
    artifact_id: &'a str,
    artifact_version: &'a str,
    source_id: &'a str,
    source_digest: &'a str,
    policy_ir_digest: &'a str,
    publication_event_id: &'a str,
    publication_event_sequence: u64,
    bound_at_case_generation: u64,
    actor_ref: &'a str,
    reason: &'a str,
    replaces_binding_id: &'a Option<String>,
}

#[derive(Serialize)]
struct TenantBindingDigestMaterial<'a> {
    schema: &'a str,
    case_id: &'a str,
    tenant_id: &'a str,
    principal_id: &'a str,
    lineage_id: &'a str,
    owner_ref: &'a str,
    policy_key: &'a str,
    artifact_id: &'a str,
    artifact_version: &'a str,
    source_id: &'a str,
    source_digest: &'a str,
    policy_ir_digest: &'a str,
    publication_event_id: &'a str,
    publication_event_sequence: u64,
    bound_at_case_generation: u64,
    actor_ref: &'a str,
    reason: &'a str,
    replaces_binding_id: &'a Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn build_case_policy_binding(
    case_id: &str,
    artifact: &PolicyArtifact,
    publication_event_id: &str,
    publication_event_sequence: u64,
    bound_at_case_generation: u64,
    actor_ref: &str,
    reason: &str,
    replaces_binding_id: Option<String>,
) -> Result<CasePolicyBinding, String> {
    artifact.validate()?;
    let lineage_id = artifact.lineage().identity();
    let reason = normalize_text(reason);
    let schema = if artifact.tenant_id.is_some() {
        CASE_POLICY_BINDING_SCHEMA
    } else {
        CASE_POLICY_BINDING_SCHEMA_V1
    };
    let encoded = if let Some(tenant_id) = artifact.tenant_id.as_deref() {
        if actor_ref != actor_ref.trim() || !actor_ref.starts_with("principal:") {
            return Err("tenant_policy_binding_requires_authenticated_principal".to_string());
        }
        serde_json::to_vec(&TenantBindingDigestMaterial {
            schema,
            case_id,
            tenant_id,
            principal_id: actor_ref,
            lineage_id: &lineage_id,
            owner_ref: &artifact.owner_ref,
            policy_key: &artifact.policy_key,
            artifact_id: &artifact.artifact_id,
            artifact_version: &artifact.artifact_version,
            source_id: &artifact.source_id,
            source_digest: &artifact.source_digest,
            policy_ir_digest: &artifact.policy_ir.ir_digest,
            publication_event_id,
            publication_event_sequence,
            bound_at_case_generation,
            actor_ref,
            reason: &reason,
            replaces_binding_id: &replaces_binding_id,
        })
    } else {
        serde_json::to_vec(&BindingDigestMaterial {
            schema,
            case_id,
            lineage_id: &lineage_id,
            owner_ref: &artifact.owner_ref,
            policy_key: &artifact.policy_key,
            artifact_id: &artifact.artifact_id,
            artifact_version: &artifact.artifact_version,
            source_id: &artifact.source_id,
            source_digest: &artifact.source_digest,
            policy_ir_digest: &artifact.policy_ir.ir_digest,
            publication_event_id,
            publication_event_sequence,
            bound_at_case_generation,
            actor_ref,
            reason: &reason,
            replaces_binding_id: &replaces_binding_id,
        })
    }
    .map_err(|error| format!("case_policy_binding_digest_encode_failed: {error}"))?;
    let integrity_digest = digest_bytes(&encoded);
    let binding = CasePolicyBinding {
        schema: schema.to_string(),
        binding_id: format!("case-policy-binding:{}", digest_prefix(&integrity_digest)),
        integrity_digest,
        case_id: case_id.to_string(),
        tenant_id: artifact.tenant_id.clone(),
        principal_id: artifact.tenant_id.as_ref().map(|_| actor_ref.to_string()),
        lineage_id,
        owner_ref: artifact.owner_ref.clone(),
        policy_key: artifact.policy_key.clone(),
        artifact_id: artifact.artifact_id.clone(),
        artifact_version: artifact.artifact_version.clone(),
        source_id: artifact.source_id.clone(),
        source_digest: artifact.source_digest.clone(),
        policy_ir_digest: artifact.policy_ir.ir_digest.clone(),
        publication_event_id: publication_event_id.to_string(),
        publication_event_sequence,
        bound_at_case_generation,
        actor_ref: actor_ref.to_string(),
        reason,
        replaces_binding_id,
    };
    binding.validate_integrity()?;
    Ok(binding)
}

impl CasePolicyBinding {
    pub fn lineage(&self) -> Result<PolicyLineage, String> {
        match &self.tenant_id {
            Some(tenant_id) => PolicyLineage::tenant(tenant_id, &self.owner_ref, &self.policy_key),
            None => PolicyLineage::new(&self.owner_ref, &self.policy_key),
        }
    }

    pub fn validate_integrity(&self) -> Result<(), String> {
        if self.schema != CASE_POLICY_BINDING_SCHEMA && self.schema != CASE_POLICY_BINDING_SCHEMA_V1
        {
            return Err(format!(
                "unsupported_case_policy_binding_schema: {}",
                self.schema
            ));
        }
        match (&*self.schema, &self.tenant_id, &self.principal_id) {
            (CASE_POLICY_BINDING_SCHEMA, Some(tenant_id), Some(principal_id))
                if tenant_id.starts_with("tenant:") && principal_id.starts_with("principal:") => {}
            (CASE_POLICY_BINDING_SCHEMA, _, _) => {
                return Err("tenant_case_policy_binding_security_scope_required".to_string())
            }
            (CASE_POLICY_BINDING_SCHEMA_V1, None, None) => {}
            _ => return Err("legacy_case_policy_binding_cannot_claim_tenant_scope".to_string()),
        }
        for (name, value) in [
            ("binding_id", self.binding_id.as_str()),
            ("case_id", self.case_id.as_str()),
            ("artifact_id", self.artifact_id.as_str()),
            ("artifact_version", self.artifact_version.as_str()),
            ("source_id", self.source_id.as_str()),
            ("source_digest", self.source_digest.as_str()),
            ("policy_ir_digest", self.policy_ir_digest.as_str()),
            ("publication_event_id", self.publication_event_id.as_str()),
            ("actor_ref", self.actor_ref.as_str()),
            ("reason", self.reason.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("case_policy_binding_{name}_missing"));
            }
        }
        if self.publication_event_sequence == 0 || self.bound_at_case_generation == 0 {
            return Err("case_policy_binding_generation_or_event_sequence_invalid".to_string());
        }
        let lineage = self.lineage()?;
        if lineage.identity() != self.lineage_id {
            return Err("case_policy_binding_lineage_identity_mismatch".to_string());
        }
        let encoded =
            if let (Some(tenant_id), Some(principal_id)) = (&self.tenant_id, &self.principal_id) {
                serde_json::to_vec(&TenantBindingDigestMaterial {
                    schema: &self.schema,
                    case_id: &self.case_id,
                    tenant_id,
                    principal_id,
                    lineage_id: &self.lineage_id,
                    owner_ref: &self.owner_ref,
                    policy_key: &self.policy_key,
                    artifact_id: &self.artifact_id,
                    artifact_version: &self.artifact_version,
                    source_id: &self.source_id,
                    source_digest: &self.source_digest,
                    policy_ir_digest: &self.policy_ir_digest,
                    publication_event_id: &self.publication_event_id,
                    publication_event_sequence: self.publication_event_sequence,
                    bound_at_case_generation: self.bound_at_case_generation,
                    actor_ref: &self.actor_ref,
                    reason: &self.reason,
                    replaces_binding_id: &self.replaces_binding_id,
                })
            } else {
                serde_json::to_vec(&BindingDigestMaterial {
                    schema: &self.schema,
                    case_id: &self.case_id,
                    lineage_id: &self.lineage_id,
                    owner_ref: &self.owner_ref,
                    policy_key: &self.policy_key,
                    artifact_id: &self.artifact_id,
                    artifact_version: &self.artifact_version,
                    source_id: &self.source_id,
                    source_digest: &self.source_digest,
                    policy_ir_digest: &self.policy_ir_digest,
                    publication_event_id: &self.publication_event_id,
                    publication_event_sequence: self.publication_event_sequence,
                    bound_at_case_generation: self.bound_at_case_generation,
                    actor_ref: &self.actor_ref,
                    reason: &self.reason,
                    replaces_binding_id: &self.replaces_binding_id,
                })
            }
            .map_err(|error| format!("case_policy_binding_digest_encode_failed: {error}"))?;
        let digest = digest_bytes(&encoded);
        if digest != self.integrity_digest
            || self.binding_id != format!("case-policy-binding:{}", digest_prefix(&digest))
        {
            return Err("case_policy_binding_integrity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn matches_artifact(&self, artifact: &PolicyArtifact) -> Result<(), String> {
        self.validate_integrity()?;
        artifact.validate()?;
        if self.artifact_id != artifact.artifact_id
            || self.artifact_version != artifact.artifact_version
            || self.owner_ref != artifact.owner_ref
            || self.policy_key != artifact.policy_key
            || self.source_id != artifact.source_id
            || self.source_digest != artifact.source_digest
            || self.policy_ir_digest != artifact.policy_ir.ir_digest
            || self.tenant_id != artifact.tenant_id
        {
            return Err("case_policy_binding_artifact_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormativeReadiness {
    Unconfigured,
    Ready,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PolicyCatalogDrift {
    Current,
    Superseded { current_artifact_id: String },
    Retired,
    Revoked,
    NoCurrentPublishedArtifact,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyValidityPosture {
    Valid,
    NotYetValid,
    RefreshRequired,
    Stale,
    Expired,
    Revoked,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BindingValidity {
    pub binding_id: String,
    pub lineage_id: String,
    pub artifact_id: String,
    pub contract: PolicyValidityContract,
    pub posture: PolicyValidityPosture,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoke_event_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveRuleProvenance {
    pub binding_id: String,
    pub artifact_id: String,
    pub policy_ir_rule_id: String,
    pub source_id: String,
    pub fact_refs: Vec<String>,
    pub source_locations: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EffectivePolicyRule {
    OperationRestriction {
        operation_kind: String,
        resource_kind: Option<String>,
        effect: PolicyEffect,
        resolution: String,
        contributions: Vec<EffectiveRuleProvenance>,
    },
    ReviewRequirement {
        operation_kind: String,
        resource_kind: Option<String>,
        required: bool,
        resolution: String,
        contributions: Vec<EffectiveRuleProvenance>,
    },
    EvidenceObligation {
        operation_kind: String,
        resource_kind: Option<String>,
        obligation: EvidenceObligationKind,
        resolution: String,
        contributions: Vec<EffectiveRuleProvenance>,
    },
    AuthorityRequirement {
        operation_kind: String,
        resource_kind: Option<String>,
        subject: AuthoritySubject,
        required_roles: Vec<String>,
        resolution: String,
        contributions: Vec<EffectiveRuleProvenance>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectivePolicy {
    pub schema: String,
    pub effective_policy_id: String,
    pub semantic_digest: String,
    pub case_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub materializer_version: String,
    pub binding_ids: Vec<String>,
    pub artifact_ids: Vec<String>,
    pub input_rule_count: usize,
    pub rules: Vec<EffectivePolicyRule>,
    pub merged_rule_count: usize,
    pub resolved_conflict_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectivePolicyInput {
    pub binding: CasePolicyBinding,
    pub artifact: PolicyArtifact,
    pub drift: PolicyCatalogDrift,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormativeStatus {
    pub case_id: String,
    pub readiness: NormativeReadiness,
    #[serde(default = "unavailable_validity")]
    pub validity: PolicyValidityPosture,
    #[serde(default)]
    pub authority_time_unix_ms: u64,
    #[serde(default)]
    pub observed_wall_time_unix_ms: u64,
    #[serde(default)]
    pub persisted_authority_floor_unix_ms: u64,
    #[serde(default)]
    pub binding_validity: BTreeMap<String, BindingValidity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_policy: Option<EffectivePolicy>,
    pub missing: Vec<String>,
    pub blocking_conflicts: Vec<String>,
    pub catalog_drift: BTreeMap<String, PolicyCatalogDrift>,
}

fn unavailable_validity() -> PolicyValidityPosture {
    PolicyValidityPosture::Unavailable
}

#[derive(Serialize)]
struct EffectiveDigestMaterial<'a> {
    schema: &'a str,
    case_id: &'a str,
    materializer_version: &'a str,
    binding_ids: &'a [String],
    artifact_ids: &'a [String],
    rules: &'a [EffectivePolicyRule],
}

#[derive(Serialize)]
struct TenantEffectiveDigestMaterial<'a> {
    schema: &'a str,
    case_id: &'a str,
    tenant_id: &'a str,
    materializer_version: &'a str,
    binding_ids: &'a [String],
    artifact_ids: &'a [String],
    rules: &'a [EffectivePolicyRule],
}

pub fn materialize_effective_policy(
    case_id: &str,
    mut inputs: Vec<EffectivePolicyInput>,
) -> NormativeStatus {
    if inputs.is_empty() {
        return NormativeStatus {
            case_id: case_id.to_string(),
            readiness: NormativeReadiness::Unconfigured,
            validity: PolicyValidityPosture::Unavailable,
            authority_time_unix_ms: 0,
            observed_wall_time_unix_ms: 0,
            persisted_authority_floor_unix_ms: 0,
            binding_validity: BTreeMap::new(),
            effective_policy: None,
            missing: Vec::new(),
            blocking_conflicts: Vec::new(),
            catalog_drift: BTreeMap::new(),
        };
    }
    inputs.sort_by(|left, right| left.binding.lineage_id.cmp(&right.binding.lineage_id));
    let mut missing = Vec::new();
    let mut conflicts = Vec::new();
    let mut seen_lineages = BTreeSet::new();
    let mut drift = BTreeMap::new();
    let tenant_ids = inputs
        .iter()
        .map(|input| input.binding.tenant_id.clone())
        .collect::<BTreeSet<_>>();
    if tenant_ids.len() != 1 {
        conflicts.push("effective_policy_mixed_security_domains".to_string());
    }
    for input in &inputs {
        if input.binding.case_id != case_id {
            missing.push(format!(
                "binding_case_mismatch:{}",
                input.binding.binding_id
            ));
        }
        if !seen_lineages.insert(input.binding.lineage_id.clone()) {
            conflicts.push(format!(
                "duplicate_active_policy_lineage:{}",
                input.binding.lineage_id
            ));
        }
        if let Err(error) = input.binding.matches_artifact(&input.artifact) {
            missing.push(format!("{}:{error}", input.binding.binding_id));
        }
        drift.insert(input.binding.lineage_id.clone(), input.drift.clone());
    }
    if !missing.is_empty() || !conflicts.is_empty() {
        missing.sort();
        conflicts.sort();
        return NormativeStatus {
            case_id: case_id.to_string(),
            readiness: NormativeReadiness::Blocked,
            validity: PolicyValidityPosture::Unavailable,
            authority_time_unix_ms: 0,
            observed_wall_time_unix_ms: 0,
            persisted_authority_floor_unix_ms: 0,
            binding_validity: BTreeMap::new(),
            effective_policy: None,
            missing,
            blocking_conflicts: conflicts,
            catalog_drift: drift,
        };
    }

    #[derive(Default)]
    struct OpGroup {
        operation: String,
        resource: Option<String>,
        effects: Vec<PolicyEffect>,
        contributions: Vec<EffectiveRuleProvenance>,
    }
    #[derive(Default)]
    struct ReviewGroup {
        operation: String,
        resource: Option<String>,
        required: Vec<bool>,
        contributions: Vec<EffectiveRuleProvenance>,
    }
    #[derive(Default)]
    struct EvidenceGroup {
        operation: String,
        resource: Option<String>,
        obligation: Option<EvidenceObligationKind>,
        contributions: Vec<EffectiveRuleProvenance>,
    }
    #[derive(Default)]
    struct AuthorityGroup {
        operation: String,
        resource: Option<String>,
        subject: Option<AuthoritySubject>,
        roles: BTreeSet<String>,
        contributions: Vec<EffectiveRuleProvenance>,
    }
    let mut operations = BTreeMap::<String, OpGroup>::new();
    let mut reviews = BTreeMap::<String, ReviewGroup>::new();
    let mut evidence = BTreeMap::<String, EvidenceGroup>::new();
    let mut authority = BTreeMap::<String, AuthorityGroup>::new();
    let mut input_rule_count = 0usize;
    for input in &inputs {
        for rule in &input.artifact.policy_ir.rules {
            input_rule_count += 1;
            let provenance = contribution(&input.binding, rule);
            match rule {
                NormalizedPolicyRule::OperationRestriction {
                    operation_kind,
                    resource_kind,
                    effect,
                    ..
                } => {
                    let key = selector("operation", operation_kind, resource_kind, None);
                    let group = operations.entry(key).or_default();
                    group.operation = operation_kind.clone();
                    group.resource = resource_kind.clone();
                    group.effects.push(effect.clone());
                    group.contributions.push(provenance);
                }
                NormalizedPolicyRule::ReviewRequirement {
                    operation_kind,
                    resource_kind,
                    required,
                    ..
                } => {
                    let key = selector("review", operation_kind, resource_kind, None);
                    let group = reviews.entry(key).or_default();
                    group.operation = operation_kind.clone();
                    group.resource = resource_kind.clone();
                    group.required.push(*required);
                    group.contributions.push(provenance);
                }
                NormalizedPolicyRule::EvidenceObligation {
                    operation_kind,
                    resource_kind,
                    obligation,
                    ..
                } => {
                    let obligation_key = format!("{obligation:?}");
                    let key = selector(
                        "evidence",
                        operation_kind,
                        resource_kind,
                        Some(&obligation_key),
                    );
                    let group = evidence.entry(key).or_default();
                    group.operation = operation_kind.clone();
                    group.resource = resource_kind.clone();
                    group.obligation = Some(obligation.clone());
                    group.contributions.push(provenance);
                }
                NormalizedPolicyRule::AuthorityRequirement {
                    operation_kind,
                    resource_kind,
                    subject,
                    required_role,
                    ..
                } => {
                    let subject_key = format!("{subject:?}");
                    let key = selector(
                        "authority",
                        operation_kind,
                        resource_kind,
                        Some(&subject_key),
                    );
                    let group = authority.entry(key).or_default();
                    group.operation = operation_kind.clone();
                    group.resource = resource_kind.clone();
                    group.subject = Some(subject.clone());
                    group.roles.insert(required_role.clone());
                    group.contributions.push(provenance);
                }
            }
        }
    }
    let mut rules = Vec::new();
    let mut resolved_conflicts = 0usize;
    for (_, mut group) in operations {
        sort_contributions(&mut group.contributions);
        let has_deny = group.effects.contains(&PolicyEffect::Deny);
        let has_allow = group.effects.contains(&PolicyEffect::Allow);
        if has_deny && has_allow {
            resolved_conflicts += 1;
        }
        rules.push(EffectivePolicyRule::OperationRestriction {
            operation_kind: group.operation,
            resource_kind: group.resource,
            effect: if has_deny {
                PolicyEffect::Deny
            } else {
                PolicyEffect::Allow
            },
            resolution: if has_deny && has_allow {
                "deny_dominates_allow_under_yai.policy_materializer.v1".to_string()
            } else {
                "identical_posture_provenance_merged".to_string()
            },
            contributions: group.contributions,
        });
    }
    for (_, mut group) in reviews {
        sort_contributions(&mut group.contributions);
        let required = group.required.iter().any(|value| *value);
        let mixed = required && group.required.iter().any(|value| !*value);
        if mixed {
            resolved_conflicts += 1;
        }
        rules.push(EffectivePolicyRule::ReviewRequirement {
            operation_kind: group.operation,
            resource_kind: group.resource,
            required,
            resolution: if mixed {
                "review_required_dominates_not_required_under_yai.policy_materializer.v1"
                    .to_string()
            } else {
                "identical_posture_provenance_merged".to_string()
            },
            contributions: group.contributions,
        });
    }
    for (_, mut group) in evidence {
        sort_contributions(&mut group.contributions);
        rules.push(EffectivePolicyRule::EvidenceObligation {
            operation_kind: group.operation,
            resource_kind: group.resource,
            obligation: group.obligation.expect("group populated from a rule"),
            resolution: "obligation_set_union_with_provenance".to_string(),
            contributions: group.contributions,
        });
    }
    for (_, mut group) in authority {
        sort_contributions(&mut group.contributions);
        rules.push(EffectivePolicyRule::AuthorityRequirement {
            operation_kind: group.operation,
            resource_kind: group.resource,
            subject: group.subject.expect("group populated from a rule"),
            required_roles: group.roles.into_iter().collect(),
            resolution: "required_roles_compose_all_of_under_yai.policy_materializer.v2"
                .to_string(),
            contributions: group.contributions,
        });
    }
    rules.sort_by_key(|rule| serde_json::to_string(rule).unwrap_or_default());
    let binding_ids = inputs
        .iter()
        .map(|item| item.binding.binding_id.clone())
        .collect::<Vec<_>>();
    let artifact_ids = inputs
        .iter()
        .map(|item| item.artifact.artifact_id.clone())
        .collect::<Vec<_>>();
    let tenant_id = inputs
        .first()
        .and_then(|input| input.binding.tenant_id.clone());
    let (schema, materializer_version, encoded) = if let Some(tenant_id) = tenant_id.as_deref() {
        (
            EFFECTIVE_POLICY_SCHEMA,
            POLICY_MATERIALIZER_VERSION,
            serde_json::to_vec(&TenantEffectiveDigestMaterial {
                schema: EFFECTIVE_POLICY_SCHEMA,
                case_id,
                tenant_id,
                materializer_version: POLICY_MATERIALIZER_VERSION,
                binding_ids: &binding_ids,
                artifact_ids: &artifact_ids,
                rules: &rules,
            })
            .expect("serializable material"),
        )
    } else {
        (
            EFFECTIVE_POLICY_SCHEMA_V2,
            POLICY_MATERIALIZER_VERSION_V2,
            serde_json::to_vec(&EffectiveDigestMaterial {
                schema: EFFECTIVE_POLICY_SCHEMA_V2,
                case_id,
                materializer_version: POLICY_MATERIALIZER_VERSION_V2,
                binding_ids: &binding_ids,
                artifact_ids: &artifact_ids,
                rules: &rules,
            })
            .expect("serializable material"),
        )
    };
    let semantic_digest = digest_bytes(&encoded);
    let output_count = rules.len();
    NormativeStatus {
        case_id: case_id.to_string(),
        readiness: NormativeReadiness::Ready,
        validity: PolicyValidityPosture::Unavailable,
        authority_time_unix_ms: 0,
        observed_wall_time_unix_ms: 0,
        persisted_authority_floor_unix_ms: 0,
        binding_validity: BTreeMap::new(),
        effective_policy: Some(EffectivePolicy {
            schema: schema.to_string(),
            effective_policy_id: format!("effective-policy:{}", digest_prefix(&semantic_digest)),
            semantic_digest,
            case_id: case_id.to_string(),
            tenant_id,
            materializer_version: materializer_version.to_string(),
            binding_ids,
            artifact_ids,
            input_rule_count,
            rules,
            merged_rule_count: input_rule_count.saturating_sub(output_count),
            resolved_conflict_count: resolved_conflicts,
        }),
        missing,
        blocking_conflicts: conflicts,
        catalog_drift: drift,
    }
}

fn contribution(
    binding: &CasePolicyBinding,
    rule: &NormalizedPolicyRule,
) -> EffectiveRuleProvenance {
    EffectiveRuleProvenance {
        binding_id: binding.binding_id.clone(),
        artifact_id: binding.artifact_id.clone(),
        policy_ir_rule_id: rule.rule_id().to_string(),
        source_id: rule.provenance().source_id.clone(),
        fact_refs: rule.provenance().fact_refs.clone(),
        source_locations: rule.provenance().source_locations.clone(),
    }
}

fn sort_contributions(values: &mut [EffectiveRuleProvenance]) {
    values.sort_by(|left, right| {
        (&left.artifact_id, &left.policy_ir_rule_id, &left.fact_refs).cmp(&(
            &right.artifact_id,
            &right.policy_ir_rule_id,
            &right.fact_refs,
        ))
    });
}

fn selector(
    prefix: &str,
    operation: &str,
    resource: &Option<String>,
    suffix: Option<&str>,
) -> String {
    format!(
        "{prefix}:{operation}:{}:{}",
        resource.as_deref().unwrap_or("any"),
        suffix.unwrap_or("")
    )
}

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn digest_prefix(value: &str) -> &str {
    let suffix = value.strip_prefix("sha256:").unwrap_or(value);
    &suffix[..suffix.len().min(32)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::{compile_policy_source, POLICY_SOURCE_INPUT_SCHEMA};

    fn artifact(owner: &str, key: &str, version: &str, effect: &str) -> PolicyArtifact {
        let source = serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": key,
            "source_version": version,
            "owner_ref": owner,
            "source_origin": {"source_system":"unit-test","source_uri":format!("test://{owner}/{key}/{version}")},
            "validity": {"mode":"unbounded"},
            "rules": [{
                "kind":"operation_restriction",
                "rule_id":format!("rule-{key}-{version}"),
                "operation_kind":"filesystem.write",
                "resource_kind":"filesystem",
                "effect":effect,
                "reason":"deterministic unit policy"
            }]
        })).unwrap();
        compile_policy_source(&source).unwrap().artifact
    }

    fn binding(case: &str, artifact: &PolicyArtifact, generation: u64) -> CasePolicyBinding {
        build_case_policy_binding(
            case,
            artifact,
            &format!("policy-event:{}", artifact.artifact_id),
            generation,
            generation,
            "participant:operator",
            "bind for materialization",
            None,
        )
        .unwrap()
    }

    #[test]
    fn binding_integrity_rejects_tampering() {
        let policy = artifact("organization:acme", "security", "1", "deny");
        let mut value = binding("case:integrity", &policy, 2);
        value.artifact_version = "tampered".to_string();
        assert_eq!(
            value.validate_integrity(),
            Err("case_policy_binding_integrity_mismatch".to_string())
        );
    }

    #[test]
    fn duplicate_active_lineage_is_a_blocking_materialization_conflict() {
        let first = artifact("organization:acme", "security", "1", "allow");
        let second = artifact("organization:acme", "security", "2", "deny");
        let status = materialize_effective_policy(
            "case:conflict",
            vec![
                EffectivePolicyInput {
                    binding: binding("case:conflict", &first, 2),
                    artifact: first,
                    drift: PolicyCatalogDrift::Superseded {
                        current_artifact_id: second.artifact_id.clone(),
                    },
                },
                EffectivePolicyInput {
                    binding: binding("case:conflict", &second, 3),
                    artifact: second,
                    drift: PolicyCatalogDrift::Current,
                },
            ],
        );
        assert_eq!(status.readiness, NormativeReadiness::Blocked);
        assert_eq!(status.blocking_conflicts.len(), 1);
        assert!(status.effective_policy.is_none());
    }
}
