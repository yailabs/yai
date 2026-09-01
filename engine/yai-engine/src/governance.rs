//! Deterministic governance source intake and immutable policy artifacts.
//!
//! This module owns one compiler boundary: constrained JSON source bytes become
//! typed parsed facts, normalized policy IR and an immutable candidate artifact.
//! It does not bind policy to a Case, evaluate an Operation, resolve authority,
//! issue a Decision/Grant, or invoke a provider/carrier.

use crate::effect::digest_bytes;
use serde::de::{DeserializeSeed, Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const POLICY_SOURCE_INPUT_SCHEMA: &str = "yai.policy_source_input.v4";
pub const POLICY_SOURCE_INPUT_SCHEMA_V3: &str = "yai.policy_source_input.v3";
pub const POLICY_SOURCE_INPUT_SCHEMA_V2: &str = "yai.policy_source_input.v2";
pub const POLICY_SOURCE_INPUT_SCHEMA_V1: &str = "yai.policy_source_input.v1";
pub const POLICY_SOURCE_ARTIFACT_SCHEMA: &str = "yai.policy_source_artifact.v4";
pub const POLICY_SOURCE_ARTIFACT_SCHEMA_V3: &str = "yai.policy_source_artifact.v3";
pub const POLICY_SOURCE_ARTIFACT_SCHEMA_V2: &str = "yai.policy_source_artifact.v2";
pub const POLICY_SOURCE_ARTIFACT_SCHEMA_V1: &str = "yai.policy_source_artifact.v1";
pub const PARSED_POLICY_SCHEMA: &str = "yai.parsed_policy.v2";
pub const PARSED_POLICY_SCHEMA_V1: &str = "yai.parsed_policy.v1";
pub const POLICY_IR_SCHEMA: &str = "yai.policy_ir.v2";
pub const POLICY_IR_SCHEMA_V1: &str = "yai.policy_ir.v1";
pub const POLICY_ARTIFACT_SCHEMA: &str = "yai.policy_artifact.v5";
pub const POLICY_ARTIFACT_SCHEMA_V4: &str = "yai.policy_artifact.v4";
pub const POLICY_ARTIFACT_SCHEMA_V3: &str = "yai.policy_artifact.v3";
pub const POLICY_ARTIFACT_SCHEMA_V2: &str = "yai.policy_artifact.v2";
pub const POLICY_ARTIFACT_SCHEMA_V1: &str = "yai.policy_artifact.v1";
pub const POLICY_LIFECYCLE_EVENT_SCHEMA: &str = "yai.policy_lifecycle_event.v3";
pub const POLICY_LIFECYCLE_EVENT_SCHEMA_V2: &str = "yai.policy_lifecycle_event.v2";
pub const POLICY_LIFECYCLE_EVENT_SCHEMA_V1: &str = "yai.policy_lifecycle_event.v1";
pub const POLICY_COMPILER_VERSION: &str = "yai.policy_compiler.v2";
pub const POLICY_COMPILER_VERSION_V1: &str = "yai.policy_compiler.v1";
pub const POLICY_VALIDATOR_VERSION: &str = "yai.policy_validator.v2";
pub const POLICY_VALIDATOR_VERSION_V1: &str = "yai.policy_validator.v1";
pub const MAX_POLICY_SOURCE_BYTES: usize = 256 * 1024;
pub const MAX_POLICY_RULES: usize = 128;
pub const MAX_POLICY_JSON_DEPTH: usize = 32;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyValidityMode {
    #[default]
    Unbounded,
    Bounded,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyValidityContract {
    pub mode: PolicyValidityMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_after_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_unix_ms: Option<u64>,
}

impl PolicyValidityContract {
    pub fn validate(&self) -> Result<(), String> {
        match self.mode {
            PolicyValidityMode::Unbounded => {
                if self.valid_from_unix_ms.is_some()
                    || self.refresh_after_unix_ms.is_some()
                    || self.expires_at_unix_ms.is_some()
                {
                    return Err("unbounded_policy_validity_cannot_have_window".to_string());
                }
            }
            PolicyValidityMode::Bounded => {
                let valid_from = self
                    .valid_from_unix_ms
                    .ok_or_else(|| "bounded_policy_validity_requires_valid_from".to_string())?;
                let refresh_after = self
                    .refresh_after_unix_ms
                    .ok_or_else(|| "bounded_policy_validity_requires_refresh_after".to_string())?;
                let expires_at = self
                    .expires_at_unix_ms
                    .ok_or_else(|| "bounded_policy_validity_requires_expires_at".to_string())?;
                if valid_from > refresh_after || refresh_after > expires_at {
                    return Err("policy_validity_window_order_invalid".to_string());
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyLineage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub owner_ref: String,
    pub policy_key: String,
}

impl PolicyLineage {
    pub fn new(
        owner_ref: impl Into<String>,
        policy_key: impl Into<String>,
    ) -> Result<Self, String> {
        let lineage = Self {
            tenant_id: None,
            owner_ref: owner_ref.into(),
            policy_key: policy_key.into(),
        };
        lineage.validate()?;
        Ok(lineage)
    }

    pub fn tenant(
        tenant_id: impl Into<String>,
        organization_ref: impl Into<String>,
        policy_key: impl Into<String>,
    ) -> Result<Self, String> {
        let lineage = Self {
            tenant_id: Some(tenant_id.into()),
            owner_ref: organization_ref.into(),
            policy_key: policy_key.into(),
        };
        lineage.validate()?;
        Ok(lineage)
    }

    pub fn validate(&self) -> Result<(), String> {
        validate_identifier("owner_ref", &self.owner_ref, 160)?;
        if let Some(tenant_id) = &self.tenant_id {
            validate_identifier("tenant_id", tenant_id, 160)?;
            if !tenant_id.starts_with("tenant:") {
                return Err("policy_lineage_tenant_id_invalid".to_string());
            }
        }
        validate_identifier("policy_key", &self.policy_key, 160)
    }

    pub fn identity(&self) -> String {
        let digest = match &self.tenant_id {
            Some(tenant_id) => digest_serialized(&serde_json::json!({
                "schema": "yai.policy_lineage.v2",
                "tenant_id": tenant_id,
                "policy_key": self.policy_key,
            })),
            None => digest_serialized(&serde_json::json!({
                "owner_ref": self.owner_ref,
                "policy_key": self.policy_key,
            })),
        };
        format!("policy-lineage:{}", digest_suffix(&digest))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySourceOrigin {
    pub source_system: String,
    pub source_uri: String,
}

impl PolicySourceOrigin {
    fn validate(&self) -> Result<(), String> {
        validate_identifier("source_system", &self.source_system, 120)?;
        validate_source_uri(&self.source_uri)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySourceArtifact {
    pub schema: String,
    pub source_id: String,
    pub content_digest: String,
    pub source_format: String,
    pub policy_key: String,
    pub source_version: String,
    pub owner_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_origin: Option<PolicySourceOrigin>,
    #[serde(default)]
    pub validity: PolicyValidityContract,
    pub content_utf8: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceObligationKind {
    PreObservation,
    PostObservation,
    AuditReason,
    SourceProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthoritySubject {
    Proposer,
    Reviewer,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParsedPolicyFact {
    OperationRestriction {
        fact_id: String,
        rule_id: String,
        operation_kind: String,
        resource_kind: Option<String>,
        effect: PolicyEffect,
        reason: String,
        source_ref: String,
        source_location: String,
    },
    ReviewRequirement {
        fact_id: String,
        rule_id: String,
        operation_kind: String,
        resource_kind: Option<String>,
        required: bool,
        reason: String,
        source_ref: String,
        source_location: String,
    },
    EvidenceObligation {
        fact_id: String,
        rule_id: String,
        operation_kind: String,
        resource_kind: Option<String>,
        obligation: EvidenceObligationKind,
        reason: String,
        source_ref: String,
        source_location: String,
    },
    AuthorityRequirement {
        fact_id: String,
        rule_id: String,
        operation_kind: String,
        resource_kind: Option<String>,
        subject: AuthoritySubject,
        required_role: String,
        reason: String,
        source_ref: String,
        source_location: String,
    },
}

impl ParsedPolicyFact {
    pub fn fact_id(&self) -> &str {
        match self {
            Self::OperationRestriction { fact_id, .. }
            | Self::ReviewRequirement { fact_id, .. }
            | Self::EvidenceObligation { fact_id, .. }
            | Self::AuthorityRequirement { fact_id, .. } => fact_id,
        }
    }

    pub fn rule_id(&self) -> &str {
        match self {
            Self::OperationRestriction { rule_id, .. }
            | Self::ReviewRequirement { rule_id, .. }
            | Self::EvidenceObligation { rule_id, .. }
            | Self::AuthorityRequirement { rule_id, .. } => rule_id,
        }
    }

    pub fn source_location(&self) -> &str {
        match self {
            Self::OperationRestriction {
                source_location, ..
            }
            | Self::ReviewRequirement {
                source_location, ..
            }
            | Self::EvidenceObligation {
                source_location, ..
            }
            | Self::AuthorityRequirement {
                source_location, ..
            } => source_location,
        }
    }

    pub fn source_ref(&self) -> &str {
        match self {
            Self::OperationRestriction { source_ref, .. }
            | Self::ReviewRequirement { source_ref, .. }
            | Self::EvidenceObligation { source_ref, .. }
            | Self::AuthorityRequirement { source_ref, .. } => source_ref,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnresolvedPolicyItem {
    pub code: String,
    pub source_location: String,
    pub source_kind: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParsedPolicy {
    pub schema: String,
    pub compiler_version: String,
    pub source_id: String,
    pub parsed_digest: String,
    pub facts: Vec<ParsedPolicyFact>,
    pub unresolved: Vec<UnresolvedPolicyItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRuleProvenance {
    pub source_id: String,
    pub fact_refs: Vec<String>,
    pub source_locations: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NormalizedPolicyRule {
    OperationRestriction {
        rule_id: String,
        operation_kind: String,
        resource_kind: Option<String>,
        effect: PolicyEffect,
        reason: String,
        provenance: PolicyRuleProvenance,
    },
    ReviewRequirement {
        rule_id: String,
        operation_kind: String,
        resource_kind: Option<String>,
        required: bool,
        reason: String,
        provenance: PolicyRuleProvenance,
    },
    EvidenceObligation {
        rule_id: String,
        operation_kind: String,
        resource_kind: Option<String>,
        obligation: EvidenceObligationKind,
        reason: String,
        provenance: PolicyRuleProvenance,
    },
    AuthorityRequirement {
        rule_id: String,
        operation_kind: String,
        resource_kind: Option<String>,
        subject: AuthoritySubject,
        required_role: String,
        reason: String,
        provenance: PolicyRuleProvenance,
    },
}

impl NormalizedPolicyRule {
    pub fn rule_id(&self) -> &str {
        match self {
            Self::OperationRestriction { rule_id, .. }
            | Self::ReviewRequirement { rule_id, .. }
            | Self::EvidenceObligation { rule_id, .. }
            | Self::AuthorityRequirement { rule_id, .. } => rule_id,
        }
    }

    pub fn provenance(&self) -> &PolicyRuleProvenance {
        match self {
            Self::OperationRestriction { provenance, .. }
            | Self::ReviewRequirement { provenance, .. }
            | Self::EvidenceObligation { provenance, .. }
            | Self::AuthorityRequirement { provenance, .. } => provenance,
        }
    }

    fn provenance_mut(&mut self) -> &mut PolicyRuleProvenance {
        match self {
            Self::OperationRestriction { provenance, .. }
            | Self::ReviewRequirement { provenance, .. }
            | Self::EvidenceObligation { provenance, .. }
            | Self::AuthorityRequirement { provenance, .. } => provenance,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyConflict {
    pub code: String,
    pub selector: String,
    pub rule_refs: Vec<String>,
    pub source_fact_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyIr {
    pub schema: String,
    pub compiler_version: String,
    pub source_id: String,
    pub ir_digest: String,
    pub rules: Vec<NormalizedPolicyRule>,
    pub unresolved: Vec<UnresolvedPolicyItem>,
    pub conflicts: Vec<PolicyConflict>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyValidationStatus {
    Qualified,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyValidation {
    pub validator_version: String,
    pub status: PolicyValidationStatus,
    pub blockers: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyArtifact {
    pub schema: String,
    pub artifact_id: String,
    pub policy_key: String,
    pub artifact_version: String,
    pub owner_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_origin: Option<PolicySourceOrigin>,
    #[serde(default)]
    pub validity: PolicyValidityContract,
    pub source_id: String,
    pub source_digest: String,
    pub parsed: ParsedPolicy,
    pub policy_ir: PolicyIr,
    pub validation: PolicyValidation,
}

impl PolicyArtifact {
    pub fn lineage(&self) -> PolicyLineage {
        PolicyLineage {
            tenant_id: self.tenant_id.clone(),
            owner_ref: self.owner_ref.clone(),
            policy_key: self.policy_key.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyCompilation {
    pub source: PolicySourceArtifact,
    pub artifact: PolicyArtifact,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyIngestOutcome {
    pub source_created: bool,
    pub artifact_created: bool,
    pub view: PolicyArtifactView,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyLifecycleOutcome {
    pub changed: bool,
    pub view: PolicyArtifactView,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyLifecycleState {
    Candidate,
    Validated,
    Published,
    Superseded,
    Retired,
    Revoked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyLifecycleAction {
    CandidateRegistered,
    Validated,
    Published,
    Superseded,
    Retired,
    Revoked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyLifecycleEvent {
    pub schema: String,
    pub event_id: String,
    pub sequence: u64,
    pub artifact_id: String,
    pub action: PolicyLifecycleAction,
    pub prior_state: Option<PolicyLifecycleState>,
    pub next_state: PolicyLifecycleState,
    pub related_artifact_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    pub actor_ref: String,
    pub reason: String,
    pub committed_at_unix_ms: u64,
    pub integrity_digest: String,
}

pub(crate) struct PolicyLifecycleEventInput<'a> {
    pub artifact_id: &'a str,
    pub action: PolicyLifecycleAction,
    pub prior_state: Option<PolicyLifecycleState>,
    pub next_state: PolicyLifecycleState,
    pub related_artifact_id: Option<&'a str>,
    pub tenant_id: Option<&'a str>,
    pub principal_id: Option<&'a str>,
    pub actor_ref: &'a str,
    pub reason: &'a str,
    pub committed_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyArtifactView {
    pub artifact: PolicyArtifact,
    pub lifecycle: PolicyLifecycleState,
    pub runtime_consumable: bool,
    pub superseded_by: Option<String>,
    pub lifecycle_events: Vec<PolicyLifecycleEvent>,
}

impl PolicyArtifactView {
    pub fn validate(&self) -> Result<(), String> {
        self.artifact.validate()?;
        if self
            .lifecycle_events
            .iter()
            .any(|event| event.artifact_id != self.artifact.artifact_id)
        {
            return Err("policy_artifact_view_event_artifact_mismatch".to_string());
        }
        let derived = lifecycle_from_events(&self.lifecycle_events)?;
        if derived != self.lifecycle {
            return Err("policy_artifact_view_lifecycle_mismatch".to_string());
        }
        let expected_runtime = self.lifecycle == PolicyLifecycleState::Published
            && self.artifact.validation.status == PolicyValidationStatus::Qualified;
        if self.runtime_consumable != expected_runtime {
            return Err("policy_artifact_view_runtime_consumable_mismatch".to_string());
        }
        let expected_superseded_by = self
            .lifecycle_events
            .iter()
            .rev()
            .find(|event| event.action == PolicyLifecycleAction::Superseded)
            .and_then(|event| event.related_artifact_id.clone());
        if self.superseded_by != expected_superseded_by {
            return Err("policy_artifact_view_supersession_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicySourceDocument {
    schema: String,
    policy_key: String,
    source_version: String,
    owner_ref: String,
    #[serde(default)]
    source_origin: Option<PolicySourceOrigin>,
    #[serde(default)]
    validity: Option<PolicyValidityContract>,
    rules: Vec<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationRestrictionSourceRule {
    kind: String,
    rule_id: String,
    operation_kind: String,
    resource_kind: Option<String>,
    effect: PolicyEffect,
    reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewRequirementSourceRule {
    kind: String,
    rule_id: String,
    operation_kind: String,
    resource_kind: Option<String>,
    required: bool,
    reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceObligationSourceRule {
    kind: String,
    rule_id: String,
    operation_kind: String,
    resource_kind: Option<String>,
    obligation: EvidenceObligationKind,
    reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityRequirementSourceRule {
    kind: String,
    rule_id: String,
    operation_kind: String,
    resource_kind: Option<String>,
    subject: AuthoritySubject,
    required_role: String,
    reason: String,
}

#[derive(Serialize)]
struct ParsedDigestMaterial<'a> {
    schema: &'a str,
    compiler_version: &'a str,
    source_id: &'a str,
    facts: &'a [ParsedPolicyFact],
    unresolved: &'a [UnresolvedPolicyItem],
}

#[derive(Serialize)]
struct PolicyIrDigestMaterial<'a> {
    schema: &'a str,
    compiler_version: &'a str,
    source_id: &'a str,
    rules: &'a [NormalizedPolicyRule],
    unresolved: &'a [UnresolvedPolicyItem],
    conflicts: &'a [PolicyConflict],
}

#[derive(Serialize)]
struct PolicyArtifactDigestMaterialV5<'a> {
    tenant_id: &'a str,
    organization_ref: &'a str,
    schema: &'a str,
    policy_key: &'a str,
    artifact_version: &'a str,
    owner_ref: &'a str,
    source_origin: Option<&'a PolicySourceOrigin>,
    validity: &'a PolicyValidityContract,
    source_id: &'a str,
    source_digest: &'a str,
    parsed_digest: &'a str,
    ir_digest: &'a str,
    validation: &'a PolicyValidation,
}

#[derive(Serialize)]
struct PolicyArtifactDigestMaterialV4<'a> {
    schema: &'a str,
    policy_key: &'a str,
    artifact_version: &'a str,
    owner_ref: &'a str,
    source_origin: Option<&'a PolicySourceOrigin>,
    validity: &'a PolicyValidityContract,
    source_id: &'a str,
    source_digest: &'a str,
    parsed_digest: &'a str,
    ir_digest: &'a str,
    validation: &'a PolicyValidation,
}

#[derive(Serialize)]
struct PolicyArtifactDigestMaterialV2V3<'a> {
    schema: &'a str,
    policy_key: &'a str,
    artifact_version: &'a str,
    owner_ref: &'a str,
    source_origin: Option<&'a PolicySourceOrigin>,
    source_id: &'a str,
    source_digest: &'a str,
    parsed_digest: &'a str,
    ir_digest: &'a str,
    validation: &'a PolicyValidation,
}

#[derive(Serialize)]
struct PolicyArtifactDigestMaterialV1<'a> {
    schema: &'a str,
    policy_key: &'a str,
    artifact_version: &'a str,
    owner_ref: &'a str,
    source_id: &'a str,
    source_digest: &'a str,
    parsed_digest: &'a str,
    ir_digest: &'a str,
    validation: &'a PolicyValidation,
}

struct StrictJsonSeed {
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for StrictJsonSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        if self.depth > MAX_POLICY_JSON_DEPTH {
            return Err(D::Error::custom("policy_source_json_depth_exceeded"));
        }
        deserializer.deserialize_any(StrictJsonVisitor { depth: self.depth })
    }
}

struct StrictJsonVisitor {
    depth: usize,
}

impl<'de> Visitor<'de> for StrictJsonVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bounded JSON without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(Value::String(value.to_string()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        StrictJsonSeed { depth: self.depth }.deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut values: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result = Vec::new();
        while let Some(value) = values.next_element_seed(StrictJsonSeed {
            depth: self.depth + 1,
        })? {
            result.push(value);
        }
        Ok(Value::Array(result))
    }

    fn visit_map<A>(self, mut values: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut keys = BTreeSet::new();
        let mut result = serde_json::Map::new();
        while let Some(key) = values.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(A::Error::custom(format!("duplicate_json_key:{key}")));
            }
            let value = values.next_value_seed(StrictJsonSeed {
                depth: self.depth + 1,
            })?;
            result.insert(key, value);
        }
        Ok(Value::Object(result))
    }
}

pub(crate) fn parse_strict_json(bytes: &[u8]) -> Result<Value, String> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = StrictJsonSeed { depth: 0 }
        .deserialize(&mut deserializer)
        .map_err(|error| format!("policy_source_json_invalid: {error}"))?;
    deserializer
        .end()
        .map_err(|error| format!("policy_source_json_invalid: {error}"))?;
    Ok(value)
}

pub fn compile_policy_source(bytes: &[u8]) -> Result<PolicyCompilation, String> {
    if bytes.is_empty() {
        return Err("policy_source_empty".to_string());
    }
    if bytes.len() > MAX_POLICY_SOURCE_BYTES {
        return Err(format!(
            "policy_source_too_large: maximum={MAX_POLICY_SOURCE_BYTES} actual={}",
            bytes.len()
        ));
    }
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err("policy_source_utf8_bom_not_supported".to_string());
    }
    let content_utf8 = std::str::from_utf8(bytes)
        .map_err(|error| format!("policy_source_not_utf8: {error}"))?
        .to_string();
    let strict_value = parse_strict_json(bytes)?;
    let document: PolicySourceDocument = serde_json::from_value(strict_value)
        .map_err(|error| format!("policy_source_json_invalid: {error}"))?;
    if document.schema != POLICY_SOURCE_INPUT_SCHEMA
        && document.schema != POLICY_SOURCE_INPUT_SCHEMA_V3
        && document.schema != POLICY_SOURCE_INPUT_SCHEMA_V2
        && document.schema != POLICY_SOURCE_INPUT_SCHEMA_V1
    {
        return Err(format!(
            "unsupported_policy_source_schema: {}",
            document.schema
        ));
    }
    if (document.schema == POLICY_SOURCE_INPUT_SCHEMA
        || document.schema == POLICY_SOURCE_INPUT_SCHEMA_V3
        || document.schema == POLICY_SOURCE_INPUT_SCHEMA_V2)
        && document.source_origin.is_none()
    {
        return Err("policy_source_origin_required".to_string());
    }
    validate_identifier("policy_key", &document.policy_key, 160)?;
    validate_identifier("source_version", &document.source_version, 80)?;
    validate_identifier("owner_ref", &document.owner_ref, 160)?;
    if let Some(origin) = &document.source_origin {
        origin.validate()?;
    }
    let validity = match (&*document.schema, document.validity.clone()) {
        (POLICY_SOURCE_INPUT_SCHEMA, Some(validity)) => validity,
        (POLICY_SOURCE_INPUT_SCHEMA, None) => {
            return Err("policy_validity_contract_required".to_string())
        }
        (_, Some(_)) => return Err("legacy_policy_source_cannot_claim_v4_validity".to_string()),
        (_, None) => PolicyValidityContract::default(),
    };
    validity.validate()?;
    if document.rules.is_empty() {
        return Err("policy_source_requires_rules".to_string());
    }
    if document.rules.len() > MAX_POLICY_RULES {
        return Err(format!(
            "policy_source_rule_limit_exceeded: maximum={MAX_POLICY_RULES} actual={}",
            document.rules.len()
        ));
    }

    let source_digest = digest_bytes(bytes);
    let source_id = format!("policy-source:{}", digest_suffix(&source_digest));
    let source = PolicySourceArtifact {
        schema: match document.schema.as_str() {
            POLICY_SOURCE_INPUT_SCHEMA => POLICY_SOURCE_ARTIFACT_SCHEMA,
            POLICY_SOURCE_INPUT_SCHEMA_V3 => POLICY_SOURCE_ARTIFACT_SCHEMA_V3,
            POLICY_SOURCE_INPUT_SCHEMA_V2 => POLICY_SOURCE_ARTIFACT_SCHEMA_V2,
            _ => POLICY_SOURCE_ARTIFACT_SCHEMA_V1,
        }
        .to_string(),
        source_id: source_id.clone(),
        content_digest: source_digest.clone(),
        source_format: "constrained_json".to_string(),
        policy_key: document.policy_key.clone(),
        source_version: document.source_version.clone(),
        owner_ref: document.owner_ref.clone(),
        source_origin: document.source_origin.clone(),
        validity: validity.clone(),
        content_utf8,
    };

    let supports_authority_rules = matches!(
        document.schema.as_str(),
        POLICY_SOURCE_INPUT_SCHEMA | POLICY_SOURCE_INPUT_SCHEMA_V3
    );
    let parsed_schema = if supports_authority_rules {
        PARSED_POLICY_SCHEMA
    } else {
        PARSED_POLICY_SCHEMA_V1
    };
    let compiler_version = if supports_authority_rules {
        POLICY_COMPILER_VERSION
    } else {
        POLICY_COMPILER_VERSION_V1
    };
    let (facts, unresolved) =
        parse_policy_rules(&source_id, &document.rules, supports_authority_rules)?;
    let parsed_digest = digest_serialized(&ParsedDigestMaterial {
        schema: parsed_schema,
        compiler_version,
        source_id: &source_id,
        facts: &facts,
        unresolved: &unresolved,
    });
    let parsed = ParsedPolicy {
        schema: parsed_schema.to_string(),
        compiler_version: compiler_version.to_string(),
        source_id: source_id.clone(),
        parsed_digest,
        facts,
        unresolved,
    };
    let policy_ir = normalize_policy(&parsed)?;
    let validation = derive_policy_validation(&policy_ir);
    let artifact_schema = match document.schema.as_str() {
        POLICY_SOURCE_INPUT_SCHEMA => POLICY_ARTIFACT_SCHEMA_V4,
        POLICY_SOURCE_INPUT_SCHEMA_V3 => POLICY_ARTIFACT_SCHEMA_V3,
        POLICY_SOURCE_INPUT_SCHEMA_V2 => POLICY_ARTIFACT_SCHEMA_V2,
        _ => POLICY_ARTIFACT_SCHEMA_V1,
    };
    let artifact_digest = if artifact_schema == POLICY_ARTIFACT_SCHEMA_V4 {
        digest_serialized(&PolicyArtifactDigestMaterialV4 {
            schema: artifact_schema,
            policy_key: &document.policy_key,
            artifact_version: &document.source_version,
            owner_ref: &document.owner_ref,
            source_origin: document.source_origin.as_ref(),
            validity: &validity,
            source_id: &source_id,
            source_digest: &source_digest,
            parsed_digest: &parsed.parsed_digest,
            ir_digest: &policy_ir.ir_digest,
            validation: &validation,
        })
    } else if artifact_schema != POLICY_ARTIFACT_SCHEMA_V1 {
        digest_serialized(&PolicyArtifactDigestMaterialV2V3 {
            schema: artifact_schema,
            policy_key: &document.policy_key,
            artifact_version: &document.source_version,
            owner_ref: &document.owner_ref,
            source_origin: document.source_origin.as_ref(),
            source_id: &source_id,
            source_digest: &source_digest,
            parsed_digest: &parsed.parsed_digest,
            ir_digest: &policy_ir.ir_digest,
            validation: &validation,
        })
    } else {
        digest_serialized(&PolicyArtifactDigestMaterialV1 {
            schema: artifact_schema,
            policy_key: &document.policy_key,
            artifact_version: &document.source_version,
            owner_ref: &document.owner_ref,
            source_id: &source_id,
            source_digest: &source_digest,
            parsed_digest: &parsed.parsed_digest,
            ir_digest: &policy_ir.ir_digest,
            validation: &validation,
        })
    };
    let artifact = PolicyArtifact {
        schema: artifact_schema.to_string(),
        artifact_id: format!("policy-artifact:{}", digest_suffix(&artifact_digest)),
        policy_key: document.policy_key,
        artifact_version: document.source_version,
        owner_ref: document.owner_ref,
        tenant_id: None,
        organization_ref: None,
        source_origin: document.source_origin,
        validity,
        source_id,
        source_digest,
        parsed,
        policy_ir,
        validation,
    };
    let compilation = PolicyCompilation { source, artifact };
    compilation.validate()?;
    Ok(compilation)
}

/// Binds an already deterministic source compilation to the authenticated
/// Tenant ingest context. Source bytes remain globally content-addressed; the
/// authority-bearing artifact identity becomes Tenant-specific.
pub fn scope_policy_compilation(
    compilation: &PolicyCompilation,
    tenant_id: &str,
    organization_ref: &str,
) -> Result<PolicyCompilation, String> {
    compilation.validate()?;
    validate_identifier("tenant_id", tenant_id, 160)?;
    validate_identifier("organization_ref", organization_ref, 160)?;
    if !tenant_id.starts_with("tenant:") || !organization_ref.starts_with("organization:") {
        return Err("policy_security_scope_invalid".to_string());
    }
    if compilation.source.owner_ref != organization_ref {
        return Err("policy_source_owner_disagrees_with_authenticated_tenant".to_string());
    }
    if compilation.artifact.schema != POLICY_ARTIFACT_SCHEMA_V4 {
        return Err("only_v4_policy_compilation_can_enter_tenant_authority".to_string());
    }
    let mut artifact = compilation.artifact.clone();
    artifact.schema = POLICY_ARTIFACT_SCHEMA.to_string();
    artifact.tenant_id = Some(tenant_id.to_string());
    artifact.organization_ref = Some(organization_ref.to_string());
    let digest = digest_serialized(&PolicyArtifactDigestMaterialV5 {
        tenant_id,
        organization_ref,
        schema: &artifact.schema,
        policy_key: &artifact.policy_key,
        artifact_version: &artifact.artifact_version,
        owner_ref: &artifact.owner_ref,
        source_origin: artifact.source_origin.as_ref(),
        validity: &artifact.validity,
        source_id: &artifact.source_id,
        source_digest: &artifact.source_digest,
        parsed_digest: &artifact.parsed.parsed_digest,
        ir_digest: &artifact.policy_ir.ir_digest,
        validation: &artifact.validation,
    });
    artifact.artifact_id = format!("policy-artifact:{}", digest_suffix(&digest));
    let result = PolicyCompilation {
        source: compilation.source.clone(),
        artifact,
    };
    result.validate()?;
    Ok(result)
}

fn parse_policy_rules(
    source_id: &str,
    rules: &[Value],
    authority_rules_enabled: bool,
) -> Result<(Vec<ParsedPolicyFact>, Vec<UnresolvedPolicyItem>), String> {
    let mut facts = Vec::new();
    let mut unresolved = Vec::new();
    for (index, value) in rules.iter().enumerate() {
        let location = format!("$.rules[{index}]");
        let kind = value
            .as_object()
            .and_then(|object| object.get("kind"))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("policy_rule_kind_missing: {location}"))?;
        match kind {
            "operation_restriction" => {
                let rule: OperationRestrictionSourceRule = serde_json::from_value(value.clone())
                    .map_err(|error| format!("policy_rule_invalid: {location}: {error}"))?;
                validate_source_rule(
                    &rule.kind,
                    &rule.rule_id,
                    &rule.operation_kind,
                    rule.resource_kind.as_deref(),
                    &rule.reason,
                    &location,
                )?;
                let fact_id = fact_identity(source_id, index, value);
                facts.push(ParsedPolicyFact::OperationRestriction {
                    fact_id,
                    rule_id: rule.rule_id,
                    operation_kind: rule.operation_kind,
                    resource_kind: rule.resource_kind,
                    effect: rule.effect,
                    reason: normalize_reason(&rule.reason),
                    source_ref: source_id.to_string(),
                    source_location: location,
                });
            }
            "review_requirement" => {
                let rule: ReviewRequirementSourceRule = serde_json::from_value(value.clone())
                    .map_err(|error| format!("policy_rule_invalid: {location}: {error}"))?;
                validate_source_rule(
                    &rule.kind,
                    &rule.rule_id,
                    &rule.operation_kind,
                    rule.resource_kind.as_deref(),
                    &rule.reason,
                    &location,
                )?;
                let fact_id = fact_identity(source_id, index, value);
                facts.push(ParsedPolicyFact::ReviewRequirement {
                    fact_id,
                    rule_id: rule.rule_id,
                    operation_kind: rule.operation_kind,
                    resource_kind: rule.resource_kind,
                    required: rule.required,
                    reason: normalize_reason(&rule.reason),
                    source_ref: source_id.to_string(),
                    source_location: location,
                });
            }
            "evidence_obligation" => {
                let rule: EvidenceObligationSourceRule = serde_json::from_value(value.clone())
                    .map_err(|error| format!("policy_rule_invalid: {location}: {error}"))?;
                validate_source_rule(
                    &rule.kind,
                    &rule.rule_id,
                    &rule.operation_kind,
                    rule.resource_kind.as_deref(),
                    &rule.reason,
                    &location,
                )?;
                let fact_id = fact_identity(source_id, index, value);
                facts.push(ParsedPolicyFact::EvidenceObligation {
                    fact_id,
                    rule_id: rule.rule_id,
                    operation_kind: rule.operation_kind,
                    resource_kind: rule.resource_kind,
                    obligation: rule.obligation,
                    reason: normalize_reason(&rule.reason),
                    source_ref: source_id.to_string(),
                    source_location: location,
                });
            }
            "authority_requirement" if authority_rules_enabled => {
                let rule: AuthorityRequirementSourceRule = serde_json::from_value(value.clone())
                    .map_err(|error| format!("policy_rule_invalid: {location}: {error}"))?;
                validate_source_rule(
                    &rule.kind,
                    &rule.rule_id,
                    &rule.operation_kind,
                    rule.resource_kind.as_deref(),
                    &rule.reason,
                    &location,
                )?;
                validate_identifier("required_role", &rule.required_role, 160)?;
                let fact_id = fact_identity(source_id, index, value);
                facts.push(ParsedPolicyFact::AuthorityRequirement {
                    fact_id,
                    rule_id: rule.rule_id,
                    operation_kind: rule.operation_kind,
                    resource_kind: rule.resource_kind,
                    subject: rule.subject,
                    required_role: rule.required_role,
                    reason: normalize_reason(&rule.reason),
                    source_ref: source_id.to_string(),
                    source_location: location,
                });
            }
            unknown => unresolved.push(UnresolvedPolicyItem {
                code: "unsupported_rule_kind".to_string(),
                source_location: location,
                source_kind: unknown.to_string(),
                detail: "rule was preserved as unresolved and is not runtime consumable"
                    .to_string(),
            }),
        }
    }
    Ok((facts, unresolved))
}

fn normalize_policy(parsed: &ParsedPolicy) -> Result<PolicyIr, String> {
    let mut normalized = BTreeMap::<String, NormalizedPolicyRule>::new();
    let mut conflicts = Vec::new();
    let mut rule_ids = BTreeMap::<String, Vec<String>>::new();
    let mut outcomes = BTreeMap::<String, Vec<(String, String)>>::new();

    for fact in &parsed.facts {
        rule_ids
            .entry(fact.rule_id().to_string())
            .or_default()
            .push(fact.fact_id().to_string());
        let (semantic_key, selector_key, outcome_key, rule) = normalized_from_fact(fact)?;
        outcomes
            .entry(selector_key)
            .or_default()
            .push((outcome_key, fact.fact_id().to_string()));
        if let Some(existing) = normalized.get_mut(&semantic_key) {
            merge_provenance(existing.provenance_mut(), fact);
        } else {
            normalized.insert(semantic_key, rule);
        }
    }

    for (rule_id, facts) in rule_ids {
        if facts.len() > 1 {
            conflicts.push(PolicyConflict {
                code: "duplicate_rule_id".to_string(),
                selector: rule_id.clone(),
                rule_refs: vec![rule_id],
                source_fact_refs: facts,
            });
        }
    }

    for (selector, values) in outcomes {
        let distinct = values
            .iter()
            .map(|(value, _)| value)
            .collect::<BTreeSet<_>>();
        if distinct.len() > 1 {
            conflicts.push(PolicyConflict {
                code: "contradictory_policy_outcome".to_string(),
                selector,
                rule_refs: Vec::new(),
                source_fact_refs: values.into_iter().map(|(_, fact)| fact).collect(),
            });
        }
    }
    conflicts.sort_by(|left, right| {
        (&left.code, &left.selector, &left.source_fact_refs).cmp(&(
            &right.code,
            &right.selector,
            &right.source_fact_refs,
        ))
    });

    let mut rules = normalized.into_values().collect::<Vec<_>>();
    rules.sort_by(|left, right| {
        (
            left.rule_id(),
            serde_json::to_string(left).unwrap_or_default(),
        )
            .cmp(&(
                right.rule_id(),
                serde_json::to_string(right).unwrap_or_default(),
            ))
    });
    let (ir_schema, compiler_version) = if parsed.compiler_version == POLICY_COMPILER_VERSION {
        (POLICY_IR_SCHEMA, POLICY_COMPILER_VERSION)
    } else {
        (POLICY_IR_SCHEMA_V1, POLICY_COMPILER_VERSION_V1)
    };
    let ir_digest = digest_serialized(&PolicyIrDigestMaterial {
        schema: ir_schema,
        compiler_version,
        source_id: &parsed.source_id,
        rules: &rules,
        unresolved: &parsed.unresolved,
        conflicts: &conflicts,
    });
    Ok(PolicyIr {
        schema: ir_schema.to_string(),
        compiler_version: compiler_version.to_string(),
        source_id: parsed.source_id.clone(),
        ir_digest,
        rules,
        unresolved: parsed.unresolved.clone(),
        conflicts,
    })
}

fn normalized_from_fact(
    fact: &ParsedPolicyFact,
) -> Result<(String, String, String, NormalizedPolicyRule), String> {
    let provenance = PolicyRuleProvenance {
        source_id: match fact {
            ParsedPolicyFact::OperationRestriction { source_ref, .. }
            | ParsedPolicyFact::ReviewRequirement { source_ref, .. }
            | ParsedPolicyFact::EvidenceObligation { source_ref, .. }
            | ParsedPolicyFact::AuthorityRequirement { source_ref, .. } => source_ref.clone(),
        },
        fact_refs: vec![fact.fact_id().to_string()],
        source_locations: vec![fact.source_location().to_string()],
    };
    let result = match fact {
        ParsedPolicyFact::OperationRestriction {
            rule_id,
            operation_kind,
            resource_kind,
            effect,
            reason,
            ..
        } => {
            let selector = selector_key("operation_restriction", operation_kind, resource_kind);
            let outcome = format!("effect:{effect:?}");
            (
                format!("{selector}:{outcome}"),
                selector,
                outcome,
                NormalizedPolicyRule::OperationRestriction {
                    rule_id: rule_id.clone(),
                    operation_kind: operation_kind.clone(),
                    resource_kind: resource_kind.clone(),
                    effect: effect.clone(),
                    reason: reason.clone(),
                    provenance,
                },
            )
        }
        ParsedPolicyFact::ReviewRequirement {
            rule_id,
            operation_kind,
            resource_kind,
            required,
            reason,
            ..
        } => {
            let selector = selector_key("review_requirement", operation_kind, resource_kind);
            let outcome = format!("required:{required}");
            (
                format!("{selector}:{outcome}"),
                selector,
                outcome,
                NormalizedPolicyRule::ReviewRequirement {
                    rule_id: rule_id.clone(),
                    operation_kind: operation_kind.clone(),
                    resource_kind: resource_kind.clone(),
                    required: *required,
                    reason: reason.clone(),
                    provenance,
                },
            )
        }
        ParsedPolicyFact::EvidenceObligation {
            rule_id,
            operation_kind,
            resource_kind,
            obligation,
            reason,
            ..
        } => {
            let selector = selector_key("evidence_obligation", operation_kind, resource_kind);
            let outcome = format!("obligation:{obligation:?}");
            (
                format!("{selector}:{outcome}"),
                format!("{selector}:{outcome}"),
                outcome,
                NormalizedPolicyRule::EvidenceObligation {
                    rule_id: rule_id.clone(),
                    operation_kind: operation_kind.clone(),
                    resource_kind: resource_kind.clone(),
                    obligation: obligation.clone(),
                    reason: reason.clone(),
                    provenance,
                },
            )
        }
        ParsedPolicyFact::AuthorityRequirement {
            rule_id,
            operation_kind,
            resource_kind,
            subject,
            required_role,
            reason,
            ..
        } => {
            let selector = selector_key("authority_requirement", operation_kind, resource_kind);
            let outcome = format!("subject:{subject:?}:role:{required_role}");
            (
                format!("{selector}:{outcome}"),
                format!("{selector}:{outcome}"),
                outcome,
                NormalizedPolicyRule::AuthorityRequirement {
                    rule_id: rule_id.clone(),
                    operation_kind: operation_kind.clone(),
                    resource_kind: resource_kind.clone(),
                    subject: subject.clone(),
                    required_role: required_role.clone(),
                    reason: reason.clone(),
                    provenance,
                },
            )
        }
    };
    Ok(result)
}

fn selector_key(prefix: &str, operation_kind: &str, resource_kind: &Option<String>) -> String {
    format!(
        "{prefix}:operation={operation_kind}:resource={}",
        resource_kind.as_deref().unwrap_or("any")
    )
}

fn merge_provenance(provenance: &mut PolicyRuleProvenance, fact: &ParsedPolicyFact) {
    if !provenance
        .fact_refs
        .iter()
        .any(|item| item == fact.fact_id())
    {
        provenance.fact_refs.push(fact.fact_id().to_string());
    }
    if !provenance
        .source_locations
        .iter()
        .any(|item| item == fact.source_location())
    {
        provenance
            .source_locations
            .push(fact.source_location().to_string());
    }
    provenance.fact_refs.sort();
    provenance.source_locations.sort();
}

impl PolicyCompilation {
    pub fn validate(&self) -> Result<(), String> {
        self.source.validate()?;
        self.artifact.validate()?;
        if self.source.source_id != self.artifact.source_id
            || self.source.content_digest != self.artifact.source_digest
            || self.source.policy_key != self.artifact.policy_key
            || self.source.source_version != self.artifact.artifact_version
            || self.source.owner_ref != self.artifact.owner_ref
            || self.source.source_origin != self.artifact.source_origin
            || self.source.validity != self.artifact.validity
        {
            return Err("policy_compilation_source_artifact_mismatch".to_string());
        }
        Ok(())
    }
}

impl PolicySourceArtifact {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != POLICY_SOURCE_ARTIFACT_SCHEMA
            && self.schema != POLICY_SOURCE_ARTIFACT_SCHEMA_V3
            && self.schema != POLICY_SOURCE_ARTIFACT_SCHEMA_V2
            && self.schema != POLICY_SOURCE_ARTIFACT_SCHEMA_V1
        {
            return Err(format!(
                "unsupported_policy_source_artifact_schema: {}",
                self.schema
            ));
        }
        if self.source_format != "constrained_json" {
            return Err("unsupported_policy_source_format".to_string());
        }
        if self.content_utf8.len() > MAX_POLICY_SOURCE_BYTES {
            return Err("stored_policy_source_too_large".to_string());
        }
        validate_identifier("source_id", &self.source_id, 96)?;
        validate_identifier("policy_key", &self.policy_key, 160)?;
        validate_identifier("source_version", &self.source_version, 80)?;
        validate_identifier("owner_ref", &self.owner_ref, 160)?;
        match (&*self.schema, &self.source_origin) {
            (POLICY_SOURCE_ARTIFACT_SCHEMA, Some(origin)) => origin.validate()?,
            (POLICY_SOURCE_ARTIFACT_SCHEMA_V3, Some(origin)) => origin.validate()?,
            (POLICY_SOURCE_ARTIFACT_SCHEMA_V2, Some(origin)) => origin.validate()?,
            (POLICY_SOURCE_ARTIFACT_SCHEMA, None) => {
                return Err("policy_source_origin_required".to_string())
            }
            (POLICY_SOURCE_ARTIFACT_SCHEMA_V3, None) => {
                return Err("policy_source_origin_required".to_string())
            }
            (POLICY_SOURCE_ARTIFACT_SCHEMA_V2, None) => {
                return Err("policy_source_origin_required".to_string())
            }
            (POLICY_SOURCE_ARTIFACT_SCHEMA_V1, Some(_)) => {
                return Err("legacy_policy_source_cannot_claim_v2_origin".to_string())
            }
            (POLICY_SOURCE_ARTIFACT_SCHEMA_V1, None) => {}
            _ => unreachable!("schema checked above"),
        }
        let digest = digest_bytes(self.content_utf8.as_bytes());
        if digest != self.content_digest
            || self.source_id != format!("policy-source:{}", digest_suffix(&digest))
        {
            return Err("policy_source_artifact_digest_mismatch".to_string());
        }
        let value = parse_strict_json(self.content_utf8.as_bytes())?;
        let document: PolicySourceDocument = serde_json::from_value(value)
            .map_err(|error| format!("stored_policy_source_json_invalid: {error}"))?;
        let expected_input_schema = match self.schema.as_str() {
            POLICY_SOURCE_ARTIFACT_SCHEMA => POLICY_SOURCE_INPUT_SCHEMA,
            POLICY_SOURCE_ARTIFACT_SCHEMA_V3 => POLICY_SOURCE_INPUT_SCHEMA_V3,
            POLICY_SOURCE_ARTIFACT_SCHEMA_V2 => POLICY_SOURCE_INPUT_SCHEMA_V2,
            _ => POLICY_SOURCE_INPUT_SCHEMA_V1,
        };
        if document.schema != expected_input_schema
            || document.policy_key != self.policy_key
            || document.source_version != self.source_version
            || document.owner_ref != self.owner_ref
            || document.source_origin != self.source_origin
            || document.validity.clone().unwrap_or_default() != self.validity
        {
            return Err("policy_source_artifact_metadata_mismatch".to_string());
        }
        validate_identifier("policy_key", &document.policy_key, 160)?;
        validate_identifier("source_version", &document.source_version, 80)?;
        validate_identifier("owner_ref", &document.owner_ref, 160)?;
        if document.rules.is_empty() || document.rules.len() > MAX_POLICY_RULES {
            return Err("stored_policy_source_rule_count_invalid".to_string());
        }
        if let Some(origin) = &document.source_origin {
            origin.validate()?;
        }
        self.validity.validate()?;
        if self.schema == POLICY_SOURCE_ARTIFACT_SCHEMA && document.validity.is_none() {
            return Err("policy_validity_contract_required".to_string());
        }
        if self.schema != POLICY_SOURCE_ARTIFACT_SCHEMA && document.validity.is_some() {
            return Err("legacy_policy_source_cannot_claim_v4_validity".to_string());
        }
        parse_policy_rules(
            &self.source_id,
            &document.rules,
            matches!(
                self.schema.as_str(),
                POLICY_SOURCE_ARTIFACT_SCHEMA | POLICY_SOURCE_ARTIFACT_SCHEMA_V3
            ),
        )?;
        Ok(())
    }
}

impl PolicyArtifact {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != POLICY_ARTIFACT_SCHEMA
            && self.schema != POLICY_ARTIFACT_SCHEMA_V4
            && self.schema != POLICY_ARTIFACT_SCHEMA_V3
            && self.schema != POLICY_ARTIFACT_SCHEMA_V2
            && self.schema != POLICY_ARTIFACT_SCHEMA_V1
        {
            return Err(format!(
                "unsupported_policy_artifact_schema: {}",
                self.schema
            ));
        }
        validate_identifier("artifact_id", &self.artifact_id, 128)?;
        validate_identifier("policy_key", &self.policy_key, 160)?;
        validate_identifier("artifact_version", &self.artifact_version, 80)?;
        validate_identifier("owner_ref", &self.owner_ref, 160)?;
        match (&*self.schema, &self.tenant_id, &self.organization_ref) {
            (POLICY_ARTIFACT_SCHEMA, Some(tenant_id), Some(organization_ref)) => {
                validate_identifier("tenant_id", tenant_id, 160)?;
                validate_identifier("organization_ref", organization_ref, 160)?;
                if !tenant_id.starts_with("tenant:")
                    || !organization_ref.starts_with("organization:")
                    || organization_ref != &self.owner_ref
                {
                    return Err("policy_artifact_security_scope_invalid".to_string());
                }
            }
            (POLICY_ARTIFACT_SCHEMA, _, _) => {
                return Err("tenant_policy_artifact_security_scope_required".to_string())
            }
            (_, None, None) => {}
            _ => return Err("legacy_policy_artifact_cannot_claim_tenant_scope".to_string()),
        }
        self.lineage().validate()?;
        match (&*self.schema, &self.source_origin) {
            (POLICY_ARTIFACT_SCHEMA, Some(origin)) => origin.validate()?,
            (POLICY_ARTIFACT_SCHEMA_V4, Some(origin)) => origin.validate()?,
            (POLICY_ARTIFACT_SCHEMA_V3, Some(origin)) => origin.validate()?,
            (POLICY_ARTIFACT_SCHEMA_V2, Some(origin)) => origin.validate()?,
            (POLICY_ARTIFACT_SCHEMA, None) => {
                return Err("policy_artifact_source_origin_required".to_string())
            }
            (POLICY_ARTIFACT_SCHEMA_V4, None) => {
                return Err("policy_artifact_source_origin_required".to_string())
            }
            (POLICY_ARTIFACT_SCHEMA_V3, None) => {
                return Err("policy_artifact_source_origin_required".to_string())
            }
            (POLICY_ARTIFACT_SCHEMA_V2, None) => {
                return Err("policy_artifact_source_origin_required".to_string())
            }
            (POLICY_ARTIFACT_SCHEMA_V1, Some(_)) => {
                return Err("legacy_policy_artifact_cannot_claim_v2_origin".to_string())
            }
            (POLICY_ARTIFACT_SCHEMA_V1, None) => {}
            _ => unreachable!("schema checked above"),
        }
        validate_identifier("source_id", &self.source_id, 96)?;
        validate_sha256_digest("source_digest", &self.source_digest)?;
        if self.source_id != format!("policy-source:{}", digest_suffix(&self.source_digest)) {
            return Err("policy_artifact_source_identity_mismatch".to_string());
        }
        self.validity.validate()?;
        if self.schema != POLICY_ARTIFACT_SCHEMA
            && self.schema != POLICY_ARTIFACT_SCHEMA_V4
            && self.validity != PolicyValidityContract::default()
        {
            return Err("legacy_policy_artifact_cannot_claim_v4_validity".to_string());
        }
        let supports_authority_rules = matches!(
            self.schema.as_str(),
            POLICY_ARTIFACT_SCHEMA | POLICY_ARTIFACT_SCHEMA_V4 | POLICY_ARTIFACT_SCHEMA_V3
        );
        let expected_parsed = if supports_authority_rules {
            PARSED_POLICY_SCHEMA
        } else {
            PARSED_POLICY_SCHEMA_V1
        };
        let expected_ir = if supports_authority_rules {
            POLICY_IR_SCHEMA
        } else {
            POLICY_IR_SCHEMA_V1
        };
        let expected_compiler = if supports_authority_rules {
            POLICY_COMPILER_VERSION
        } else {
            POLICY_COMPILER_VERSION_V1
        };
        let expected_validator = if supports_authority_rules {
            POLICY_VALIDATOR_VERSION
        } else {
            POLICY_VALIDATOR_VERSION_V1
        };
        if self.parsed.schema != expected_parsed
            || self.policy_ir.schema != expected_ir
            || self.parsed.compiler_version != expected_compiler
            || self.policy_ir.compiler_version != expected_compiler
            || self.validation.validator_version != expected_validator
        {
            return Err("unsupported_policy_compiler_contract".to_string());
        }
        if self.source_id != self.parsed.source_id
            || self.source_id != self.policy_ir.source_id
            || self.policy_ir.unresolved != self.parsed.unresolved
        {
            return Err("policy_artifact_provenance_chain_mismatch".to_string());
        }
        if self.parsed.facts.iter().any(|fact| {
            fact.source_ref() != self.source_id || !fact.source_location().starts_with("$.rules[")
        }) {
            return Err("parsed_policy_source_provenance_invalid".to_string());
        }
        let parsed_digest = digest_serialized(&ParsedDigestMaterial {
            schema: &self.parsed.schema,
            compiler_version: &self.parsed.compiler_version,
            source_id: &self.parsed.source_id,
            facts: &self.parsed.facts,
            unresolved: &self.parsed.unresolved,
        });
        if parsed_digest != self.parsed.parsed_digest {
            return Err("parsed_policy_digest_mismatch".to_string());
        }
        if normalize_policy(&self.parsed)? != self.policy_ir {
            return Err("policy_ir_not_reproducible_from_parsed_facts".to_string());
        }
        let ir_digest = digest_serialized(&PolicyIrDigestMaterial {
            schema: &self.policy_ir.schema,
            compiler_version: &self.policy_ir.compiler_version,
            source_id: &self.policy_ir.source_id,
            rules: &self.policy_ir.rules,
            unresolved: &self.policy_ir.unresolved,
            conflicts: &self.policy_ir.conflicts,
        });
        if ir_digest != self.policy_ir.ir_digest {
            return Err("policy_ir_digest_mismatch".to_string());
        }
        if self.validation != derive_policy_validation(&self.policy_ir) {
            return Err("policy_validation_disposition_mismatch".to_string());
        }
        for rule in &self.policy_ir.rules {
            if rule.provenance().source_id != self.source_id
                || rule.provenance().fact_refs.is_empty()
                || rule.provenance().source_locations.is_empty()
                || rule.provenance().fact_refs.iter().any(|reference| {
                    !self
                        .parsed
                        .facts
                        .iter()
                        .any(|fact| fact.fact_id() == reference)
                })
            {
                return Err("policy_rule_provenance_invalid".to_string());
            }
        }
        let artifact_digest = if self.schema == POLICY_ARTIFACT_SCHEMA {
            digest_serialized(&PolicyArtifactDigestMaterialV5 {
                tenant_id: self
                    .tenant_id
                    .as_deref()
                    .ok_or("tenant_policy_artifact_security_scope_required")?,
                organization_ref: self
                    .organization_ref
                    .as_deref()
                    .ok_or("tenant_policy_artifact_security_scope_required")?,
                schema: &self.schema,
                policy_key: &self.policy_key,
                artifact_version: &self.artifact_version,
                owner_ref: &self.owner_ref,
                source_origin: self.source_origin.as_ref(),
                validity: &self.validity,
                source_id: &self.source_id,
                source_digest: &self.source_digest,
                parsed_digest: &self.parsed.parsed_digest,
                ir_digest: &self.policy_ir.ir_digest,
                validation: &self.validation,
            })
        } else if self.schema == POLICY_ARTIFACT_SCHEMA_V4 {
            digest_serialized(&PolicyArtifactDigestMaterialV4 {
                schema: &self.schema,
                policy_key: &self.policy_key,
                artifact_version: &self.artifact_version,
                owner_ref: &self.owner_ref,
                source_origin: self.source_origin.as_ref(),
                validity: &self.validity,
                source_id: &self.source_id,
                source_digest: &self.source_digest,
                parsed_digest: &self.parsed.parsed_digest,
                ir_digest: &self.policy_ir.ir_digest,
                validation: &self.validation,
            })
        } else if self.schema != POLICY_ARTIFACT_SCHEMA_V1 {
            digest_serialized(&PolicyArtifactDigestMaterialV2V3 {
                schema: &self.schema,
                policy_key: &self.policy_key,
                artifact_version: &self.artifact_version,
                owner_ref: &self.owner_ref,
                source_origin: self.source_origin.as_ref(),
                source_id: &self.source_id,
                source_digest: &self.source_digest,
                parsed_digest: &self.parsed.parsed_digest,
                ir_digest: &self.policy_ir.ir_digest,
                validation: &self.validation,
            })
        } else {
            digest_serialized(&PolicyArtifactDigestMaterialV1 {
                schema: &self.schema,
                policy_key: &self.policy_key,
                artifact_version: &self.artifact_version,
                owner_ref: &self.owner_ref,
                source_id: &self.source_id,
                source_digest: &self.source_digest,
                parsed_digest: &self.parsed.parsed_digest,
                ir_digest: &self.policy_ir.ir_digest,
                validation: &self.validation,
            })
        };
        if self.artifact_id != format!("policy-artifact:{}", digest_suffix(&artifact_digest)) {
            return Err("policy_artifact_identity_mismatch".to_string());
        }
        Ok(())
    }
}

impl PolicyLifecycleEvent {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != POLICY_LIFECYCLE_EVENT_SCHEMA
            && self.schema != POLICY_LIFECYCLE_EVENT_SCHEMA_V2
            && self.schema != POLICY_LIFECYCLE_EVENT_SCHEMA_V1
        {
            return Err(format!(
                "unsupported_policy_lifecycle_event_schema: {}",
                self.schema
            ));
        }
        if self.sequence == 0 {
            return Err("policy_lifecycle_sequence_must_be_positive".to_string());
        }
        validate_identifier("event_id", &self.event_id, 128)?;
        validate_identifier("artifact_id", &self.artifact_id, 128)?;
        validate_identifier("actor_ref", &self.actor_ref, 160)?;
        match (&*self.schema, &self.tenant_id, &self.principal_id) {
            (POLICY_LIFECYCLE_EVENT_SCHEMA, Some(tenant_id), Some(principal_id)) => {
                validate_identifier("tenant_id", tenant_id, 160)?;
                validate_identifier("principal_id", principal_id, 160)?;
                if !tenant_id.starts_with("tenant:")
                    || !principal_id.starts_with("principal:")
                    || &self.actor_ref != principal_id
                {
                    return Err("policy_lifecycle_security_provenance_invalid".to_string());
                }
            }
            (POLICY_LIFECYCLE_EVENT_SCHEMA, _, _) => {
                return Err("tenant_policy_lifecycle_security_provenance_required".to_string())
            }
            (_, None, None) => {}
            _ => return Err("legacy_policy_lifecycle_cannot_claim_security_scope".to_string()),
        }
        validate_reason(&self.reason, "policy_lifecycle_reason")?;
        validate_lifecycle_transition(self.prior_state.as_ref(), &self.action, &self.next_state)?;
        if self.schema == POLICY_LIFECYCLE_EVENT_SCHEMA_V1
            && self.action == PolicyLifecycleAction::Revoked
        {
            return Err("policy_revocation_requires_lifecycle_event_v2".to_string());
        }
        match (&self.action, &self.related_artifact_id) {
            (PolicyLifecycleAction::Superseded, Some(related)) => {
                validate_identifier("related_artifact_id", related, 128)?;
                if related == &self.artifact_id {
                    return Err("policy_supersession_cannot_reference_self".to_string());
                }
            }
            (PolicyLifecycleAction::Superseded, None) => {
                return Err("policy_supersession_requires_replacement".to_string())
            }
            (_, Some(_)) => return Err("policy_lifecycle_unexpected_related_artifact".to_string()),
            (_, None) => {}
        }
        let digest = lifecycle_event_digest(self);
        if self.integrity_digest != digest
            || self.event_id != format!("policy-event:{}", digest_suffix(&digest))
        {
            return Err("policy_lifecycle_event_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

pub fn derive_policy_validation(policy_ir: &PolicyIr) -> PolicyValidation {
    let mut blockers = Vec::new();
    if policy_ir.rules.is_empty() {
        blockers.push("no_supported_policy_rules".to_string());
    }
    blockers.extend(
        policy_ir
            .unresolved
            .iter()
            .map(|item| format!("unresolved:{}:{}", item.code, item.source_location)),
    );
    blockers.extend(
        policy_ir
            .conflicts
            .iter()
            .map(|conflict| format!("conflict:{}:{}", conflict.code, conflict.selector)),
    );
    PolicyValidation {
        validator_version: if policy_ir.compiler_version == POLICY_COMPILER_VERSION {
            POLICY_VALIDATOR_VERSION
        } else {
            POLICY_VALIDATOR_VERSION_V1
        }
        .to_string(),
        status: if blockers.is_empty() {
            PolicyValidationStatus::Qualified
        } else {
            PolicyValidationStatus::Blocked
        },
        blockers,
    }
}

pub fn lifecycle_from_events(
    events: &[PolicyLifecycleEvent],
) -> Result<PolicyLifecycleState, String> {
    let mut state = None;
    let mut previous_sequence = 0;
    for event in events {
        event.validate()?;
        if event.sequence <= previous_sequence {
            return Err("policy_lifecycle_sequence_not_strict".to_string());
        }
        if event.prior_state != state {
            return Err("policy_lifecycle_prior_state_mismatch".to_string());
        }
        state = Some(event.next_state.clone());
        previous_sequence = event.sequence;
    }
    state.ok_or_else(|| "policy_lifecycle_missing_candidate_event".to_string())
}

pub fn validate_lifecycle_transition(
    prior: Option<&PolicyLifecycleState>,
    action: &PolicyLifecycleAction,
    next: &PolicyLifecycleState,
) -> Result<(), String> {
    let valid = matches!(
        (prior, action, next),
        (
            None,
            PolicyLifecycleAction::CandidateRegistered,
            PolicyLifecycleState::Candidate
        ) | (
            Some(PolicyLifecycleState::Candidate),
            PolicyLifecycleAction::Validated,
            PolicyLifecycleState::Validated
        ) | (
            Some(PolicyLifecycleState::Validated),
            PolicyLifecycleAction::Published,
            PolicyLifecycleState::Published
        ) | (
            Some(PolicyLifecycleState::Published),
            PolicyLifecycleAction::Superseded,
            PolicyLifecycleState::Superseded
        ) | (
            Some(PolicyLifecycleState::Candidate),
            PolicyLifecycleAction::Retired,
            PolicyLifecycleState::Retired
        ) | (
            Some(PolicyLifecycleState::Validated),
            PolicyLifecycleAction::Retired,
            PolicyLifecycleState::Retired
        ) | (
            Some(PolicyLifecycleState::Published),
            PolicyLifecycleAction::Retired,
            PolicyLifecycleState::Retired
        ) | (
            Some(PolicyLifecycleState::Superseded),
            PolicyLifecycleAction::Retired,
            PolicyLifecycleState::Retired
        ) | (
            Some(PolicyLifecycleState::Published),
            PolicyLifecycleAction::Revoked,
            PolicyLifecycleState::Revoked
        ) | (
            Some(PolicyLifecycleState::Superseded),
            PolicyLifecycleAction::Revoked,
            PolicyLifecycleState::Revoked
        ) | (
            Some(PolicyLifecycleState::Retired),
            PolicyLifecycleAction::Revoked,
            PolicyLifecycleState::Revoked
        )
    );
    if valid {
        Ok(())
    } else {
        Err(format!(
            "invalid_policy_lifecycle_transition: prior={prior:?} action={action:?} next={next:?}"
        ))
    }
}

pub(crate) fn build_lifecycle_event(
    sequence: u64,
    input: PolicyLifecycleEventInput<'_>,
) -> Result<PolicyLifecycleEvent, String> {
    validate_identifier("actor_ref", input.actor_ref, 160)?;
    if let Some(related) = input.related_artifact_id {
        validate_identifier("related_artifact_id", related, 128)?;
    }
    validate_reason(input.reason, "policy_lifecycle_reason")?;
    validate_lifecycle_transition(input.prior_state.as_ref(), &input.action, &input.next_state)?;
    if input.tenant_id.is_some() != input.principal_id.is_some() {
        return Err("policy_lifecycle_security_context_incomplete".to_string());
    }
    let mut event = PolicyLifecycleEvent {
        schema: if input.tenant_id.is_some() {
            POLICY_LIFECYCLE_EVENT_SCHEMA
        } else {
            POLICY_LIFECYCLE_EVENT_SCHEMA_V2
        }
        .to_string(),
        event_id: String::new(),
        sequence,
        artifact_id: input.artifact_id.to_string(),
        action: input.action,
        prior_state: input.prior_state,
        next_state: input.next_state,
        related_artifact_id: input.related_artifact_id.map(ToString::to_string),
        tenant_id: input.tenant_id.map(ToString::to_string),
        principal_id: input.principal_id.map(ToString::to_string),
        actor_ref: input.actor_ref.to_string(),
        reason: normalize_reason(input.reason),
        committed_at_unix_ms: input.committed_at_unix_ms,
        integrity_digest: String::new(),
    };
    event.integrity_digest = lifecycle_event_digest(&event);
    event.event_id = format!("policy-event:{}", digest_suffix(&event.integrity_digest));
    event.validate()?;
    Ok(event)
}

fn lifecycle_event_digest(event: &PolicyLifecycleEvent) -> String {
    let mut value = serde_json::json!({
        "schema": if event.schema == POLICY_LIFECYCLE_EVENT_SCHEMA {
            POLICY_LIFECYCLE_EVENT_SCHEMA
        } else {
            POLICY_LIFECYCLE_EVENT_SCHEMA_V2
        },
        "sequence": event.sequence,
        "artifact_id": event.artifact_id,
        "action": event.action,
        "prior_state": event.prior_state,
        "next_state": event.next_state,
        "related_artifact_id": event.related_artifact_id,
        "actor_ref": event.actor_ref,
        "reason": event.reason,
        "committed_at_unix_ms": event.committed_at_unix_ms,
    });
    if event.schema == POLICY_LIFECYCLE_EVENT_SCHEMA {
        value["tenant_id"] = serde_json::json!(event.tenant_id);
        value["principal_id"] = serde_json::json!(event.principal_id);
    }
    digest_serialized(&value)
}

fn validate_source_rule(
    kind: &str,
    rule_id: &str,
    operation_kind: &str,
    resource_kind: Option<&str>,
    reason: &str,
    location: &str,
) -> Result<(), String> {
    validate_identifier("policy_rule_kind", kind, 80)
        .map_err(|error| format!("{error}: {location}"))?;
    validate_identifier("policy_rule_id", rule_id, 120)
        .map_err(|error| format!("{error}: {location}"))?;
    validate_identifier("policy_operation_kind", operation_kind, 120)
        .map_err(|error| format!("{error}: {location}"))?;
    if let Some(resource_kind) = resource_kind {
        validate_identifier("policy_resource_kind", resource_kind, 120)
            .map_err(|error| format!("{error}: {location}"))?;
    }
    validate_reason(reason, "policy_rule_reason").map_err(|error| format!("{error}: {location}"))
}

fn validate_identifier(field: &str, value: &str, maximum: usize) -> Result<(), String> {
    if value.is_empty() || value.len() > maximum {
        return Err(format!("invalid_{field}_length"));
    }
    if !value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'-' | b'_' | b'/')
    }) {
        return Err(format!("invalid_{field}_characters"));
    }
    Ok(())
}

fn validate_source_uri(value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
        return Err("invalid_source_uri".to_string());
    }
    if value.starts_with('/') || value.starts_with("file:") || !value.contains("://") {
        return Err("invalid_source_uri".to_string());
    }
    Ok(())
}

fn validate_sha256_digest(field: &str, value: &str) -> Result<(), String> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(format!("invalid_{field}"));
    };
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("invalid_{field}"));
    }
    Ok(())
}

fn validate_reason(value: &str, field: &str) -> Result<(), String> {
    let normalized = normalize_reason(value);
    if normalized.is_empty() || normalized.len() > 512 {
        return Err(format!("invalid_{field}"));
    }
    Ok(())
}

fn normalize_reason(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn fact_identity(source_id: &str, index: usize, value: &Value) -> String {
    let digest = digest_serialized(&serde_json::json!({
        "source_id": source_id,
        "source_location": format!("$.rules[{index}]"),
        "rule": value,
    }));
    format!("policy-fact:{}", digest_suffix(&digest))
}

fn digest_serialized(value: &impl Serialize) -> String {
    digest_bytes(&serde_json::to_vec(value).expect("policy digest material serializes"))
}

fn digest_suffix(value: &str) -> &str {
    value.strip_prefix("sha256:").unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_source(version: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": "organization.example.filesystem",
            "source_version": version,
            "owner_ref": "organization:example",
            "source_origin": {
                "source_system": "unit-test",
                "source_uri": "test://governance/example"
            },
            "validity": {"mode":"unbounded"},
            "rules": [
                {
                    "kind": "operation_restriction",
                    "rule_id": "deny-private",
                    "operation_kind": "filesystem.write",
                    "resource_kind": "filesystem",
                    "effect": "deny",
                    "reason": "private targets are not writable"
                },
                {
                    "kind": "review_requirement",
                    "rule_id": "review-workspace",
                    "operation_kind": "filesystem.write",
                    "resource_kind": "filesystem",
                    "required": true,
                    "reason": "writes require review"
                },
                {
                    "kind": "evidence_obligation",
                    "rule_id": "observe-post-state",
                    "operation_kind": "filesystem.write",
                    "resource_kind": "filesystem",
                    "obligation": "post_observation",
                    "reason": "a write needs observed consequence"
                }
            ]
        }))
        .expect("serialize fixture")
    }

    #[test]
    fn deterministic_compilation_has_typed_provenance_and_no_case_authority() {
        let first = compile_policy_source(&valid_source("1")).expect("compile valid source");
        let second = compile_policy_source(&valid_source("1")).expect("recompile valid source");
        assert_eq!(first, second);
        assert_eq!(
            first.artifact.validation.status,
            PolicyValidationStatus::Qualified
        );
        assert_eq!(first.artifact.parsed.facts.len(), 3);
        assert_eq!(first.artifact.policy_ir.rules.len(), 3);
        for rule in &first.artifact.policy_ir.rules {
            assert_eq!(rule.provenance().source_id, first.source.source_id);
            assert!(!rule.provenance().fact_refs.is_empty());
            assert!(!rule.provenance().source_locations.is_empty());
        }
        let encoded = serde_json::to_string(&first.artifact).expect("encode artifact");
        for forbidden in ["case_id", "decision_id", "grant_id", "provider_id"] {
            assert!(!encoded.contains(forbidden), "unexpected {forbidden}");
        }
        let mut wrong_source = first.artifact.clone();
        wrong_source.source_digest = format!("sha256:{}", "0".repeat(64));
        assert_eq!(
            wrong_source.validate(),
            Err("policy_artifact_source_identity_mismatch".to_string())
        );
        let mut wrong_fact_source = first.artifact.clone();
        if let ParsedPolicyFact::OperationRestriction { source_ref, .. } =
            &mut wrong_fact_source.parsed.facts[0]
        {
            *source_ref = "policy-source:other".to_string();
        }
        assert_eq!(
            wrong_fact_source.validate(),
            Err("parsed_policy_source_provenance_invalid".to_string())
        );
    }

    #[test]
    fn malformed_unknown_future_and_edited_sources_are_distinct_and_fail_closed() {
        assert!(compile_policy_source(br#"{"#)
            .unwrap_err()
            .contains("policy_source_json_invalid"));
        let future = br#"{"schema":"yai.policy_source_input.v99","policy_key":"p","source_version":"1","owner_ref":"o","rules":[{}]}"#;
        assert!(compile_policy_source(future)
            .unwrap_err()
            .contains("unsupported_policy_source_schema"));

        let first = compile_policy_source(&valid_source("1")).expect("version one");
        let second = compile_policy_source(&valid_source("2")).expect("version two");
        assert_ne!(first.source.source_id, second.source.source_id);
        assert_ne!(first.artifact.artifact_id, second.artifact.artifact_id);
        assert_eq!(first.artifact.policy_key, second.artifact.policy_key);
    }

    #[test]
    fn unsupported_and_conflicting_rules_cannot_qualify() {
        let unknown = serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": "organization.example.unknown",
            "source_version": "1",
            "owner_ref": "organization:example",
            "source_origin": {"source_system":"unit-test","source_uri":"test://governance/unknown"},
            "validity": {"mode":"unbounded"},
            "rules": [{"kind":"imagined_rule","meaning":"guess"}]
        }))
        .unwrap();
        let unknown = compile_policy_source(&unknown).expect("unknown retained as unresolved");
        assert_eq!(unknown.artifact.policy_ir.unresolved.len(), 1);
        assert_eq!(
            unknown.artifact.validation.status,
            PolicyValidationStatus::Blocked
        );

        let conflict = serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": "organization.example.conflict",
            "source_version": "1",
            "owner_ref": "organization:example",
            "source_origin": {"source_system":"unit-test","source_uri":"test://governance/conflict"},
            "validity": {"mode":"unbounded"},
            "rules": [
                {"kind":"review_requirement","rule_id":"review-a","operation_kind":"filesystem.write","resource_kind":"filesystem","required":true,"reason":"review"},
                {"kind":"review_requirement","rule_id":"review-b","operation_kind":"filesystem.write","resource_kind":"filesystem","required":false,"reason":"automatic"}
            ]
        }))
        .unwrap();
        let conflict = compile_policy_source(&conflict).expect("conflict remains inspectable");
        assert_eq!(conflict.artifact.policy_ir.conflicts.len(), 1);
        assert_eq!(
            conflict.artifact.validation.status,
            PolicyValidationStatus::Blocked
        );

        let operation_conflict = serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": "organization.example.operation-conflict",
            "source_version": "1",
            "owner_ref": "organization:example",
            "source_origin": {"source_system":"unit-test","source_uri":"test://governance/operation-conflict"},
            "validity": {"mode":"unbounded"},
            "rules": [
                {"kind":"operation_restriction","rule_id":"allow-a","operation_kind":"filesystem.write","resource_kind":"filesystem","effect":"allow","reason":"allow"},
                {"kind":"operation_restriction","rule_id":"deny-a","operation_kind":"filesystem.write","resource_kind":"filesystem","effect":"deny","reason":"deny"}
            ]
        }))
        .unwrap();
        let operation_conflict = compile_policy_source(&operation_conflict).unwrap();
        assert!(operation_conflict
            .artifact
            .policy_ir
            .conflicts
            .iter()
            .any(|conflict| conflict.code == "contradictory_policy_outcome"));
        assert_eq!(
            operation_conflict.artifact.validation.status,
            PolicyValidationStatus::Blocked
        );
    }

    #[test]
    fn lifecycle_is_small_explicit_and_integrity_bound() {
        let candidate = build_lifecycle_event(
            1,
            PolicyLifecycleEventInput {
                artifact_id: "policy-artifact:test",
                action: PolicyLifecycleAction::CandidateRegistered,
                prior_state: None,
                next_state: PolicyLifecycleState::Candidate,
                related_artifact_id: None,
                tenant_id: None,
                principal_id: None,
                actor_ref: "participant:operator",
                reason: "source ingested",
                committed_at_unix_ms: 1,
            },
        )
        .expect("candidate event");
        let validated = build_lifecycle_event(
            2,
            PolicyLifecycleEventInput {
                artifact_id: "policy-artifact:test",
                action: PolicyLifecycleAction::Validated,
                prior_state: Some(PolicyLifecycleState::Candidate),
                next_state: PolicyLifecycleState::Validated,
                related_artifact_id: None,
                tenant_id: None,
                principal_id: None,
                actor_ref: "participant:operator",
                reason: "deterministic qualification passed",
                committed_at_unix_ms: 2,
            },
        )
        .expect("validation event");
        assert_eq!(
            lifecycle_from_events(&[candidate.clone(), validated]),
            Ok(PolicyLifecycleState::Validated)
        );
        let mut tampered = candidate.clone();
        tampered.reason = "changed".to_string();
        assert!(tampered.validate().is_err());
        assert!(validate_lifecycle_transition(
            Some(&PolicyLifecycleState::Candidate),
            &PolicyLifecycleAction::Published,
            &PolicyLifecycleState::Published
        )
        .is_err());
        let another_candidate = build_lifecycle_event(
            3,
            PolicyLifecycleEventInput {
                artifact_id: "policy-artifact:test",
                action: PolicyLifecycleAction::CandidateRegistered,
                prior_state: None,
                next_state: PolicyLifecycleState::Candidate,
                related_artifact_id: None,
                tenant_id: None,
                principal_id: None,
                actor_ref: "participant:operator",
                reason: "invalid second candidate",
                committed_at_unix_ms: 3,
            },
        )
        .unwrap();
        assert_eq!(
            lifecycle_from_events(&[another_candidate.clone(), candidate.clone()]),
            Err("policy_lifecycle_sequence_not_strict".to_string())
        );
        assert_eq!(
            lifecycle_from_events(&[candidate.clone(), another_candidate]),
            Err("policy_lifecycle_prior_state_mismatch".to_string())
        );
        assert_eq!(
            lifecycle_from_events(&[candidate.clone(), candidate]),
            Err("policy_lifecycle_sequence_not_strict".to_string())
        );
    }

    #[test]
    fn strict_json_rejects_duplicate_keys_bom_depth_and_invalid_shapes() {
        for source in [
            br#"{"schema":"yai.policy_source_input.v2","policy_key":"a","policy_key":"b","source_version":"1","owner_ref":"o","source_origin":{"source_system":"test","source_uri":"test://a"},"rules":[]}"#.as_slice(),
            br#"{"schema":"yai.policy_source_input.v2","policy_key":"a","source_version":"1","owner_ref":"o","source_origin":{"source_system":"test","source_uri":"test://a"},"rules":[{"kind":"review_requirement","rule_id":"r","operation_kind":"filesystem.write","required":true,"required":false,"reason":"x"}]}"#.as_slice(),
        ] {
            assert!(compile_policy_source(source)
                .unwrap_err()
                .contains("duplicate_json_key"));
        }
        let mut bom = vec![0xef, 0xbb, 0xbf];
        bom.extend(valid_source("1"));
        assert_eq!(
            compile_policy_source(&bom).unwrap_err(),
            "policy_source_utf8_bom_not_supported"
        );
        assert!(compile_policy_source(&[0xff, 0xfe])
            .unwrap_err()
            .contains("not_utf8"));

        let nested = format!(
            "{{\"schema\":\"{POLICY_SOURCE_INPUT_SCHEMA}\",\"policy_key\":\"a\",\"source_version\":\"1\",\"owner_ref\":\"o\",\"source_origin\":{{\"source_system\":\"test\",\"source_uri\":\"test://a\"}},\"rules\":[{{\"kind\":\"future\",\"payload\":{} }}]}}",
            "[".repeat(MAX_POLICY_JSON_DEPTH + 2) + &"]".repeat(MAX_POLICY_JSON_DEPTH + 2)
        );
        assert!(compile_policy_source(nested.as_bytes())
            .unwrap_err()
            .contains("depth_exceeded"));

        let unknown_known_field = br#"{"schema":"yai.policy_source_input.v2","policy_key":"a","source_version":"1","owner_ref":"o","source_origin":{"source_system":"test","source_uri":"test://a"},"rules":[{"kind":"review_requirement","rule_id":"r","operation_kind":"filesystem.write","required":true,"reason":"x","guess":1}]}"#;
        assert!(compile_policy_source(unknown_known_field)
            .unwrap_err()
            .contains("unknown field"));
        let unicode_identifier = r#"{"schema":"yai.policy_source_input.v2","policy_key":"polıcy","source_version":"1","owner_ref":"o","source_origin":{"source_system":"test","source_uri":"test://a"},"rules":[{"kind":"future"}]}"#;
        assert_eq!(
            compile_policy_source(unicode_identifier.as_bytes()).unwrap_err(),
            "invalid_policy_key_characters"
        );
    }

    #[test]
    fn parser_limits_conflicts_and_equivalent_obligations_are_explicit() {
        assert_eq!(
            compile_policy_source(&vec![b' '; MAX_POLICY_SOURCE_BYTES + 1]).unwrap_err(),
            format!(
                "policy_source_too_large: maximum={MAX_POLICY_SOURCE_BYTES} actual={}",
                MAX_POLICY_SOURCE_BYTES + 1
            )
        );
        let too_many = serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": "limit",
            "source_version": "1",
            "owner_ref": "organization:test",
            "source_origin": {"source_system":"test","source_uri":"test://limit"},
            "validity": {"mode":"unbounded"},
            "rules": (0..=MAX_POLICY_RULES).map(|index| serde_json::json!({
                "kind":"review_requirement", "rule_id":format!("r{index}"),
                "operation_kind":"filesystem.write", "required":true, "reason":"valid"
            })).collect::<Vec<_>>()
        }))
        .unwrap();
        assert!(compile_policy_source(&too_many)
            .unwrap_err()
            .contains("rule_limit_exceeded"));

        let duplicate_rule = serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": "duplicate.rule",
            "source_version": "1",
            "owner_ref": "organization:test",
            "source_origin": {"source_system":"test","source_uri":"test://duplicate"},
            "validity": {"mode":"unbounded"},
            "rules": [
                {"kind":"evidence_obligation","rule_id":"same","operation_kind":"filesystem.write","obligation":"audit_reason","reason":"motivo è valido"},
                {"kind":"evidence_obligation","rule_id":"same","operation_kind":"filesystem.write","obligation":"audit_reason","reason":"motivo è valido"}
            ]
        })).unwrap();
        let compiled =
            compile_policy_source(&duplicate_rule).expect("duplicate remains inspectable");
        assert_eq!(compiled.artifact.policy_ir.rules.len(), 1);
        assert!(compiled
            .artifact
            .policy_ir
            .conflicts
            .iter()
            .any(|conflict| conflict.code == "duplicate_rule_id"));
        assert_eq!(
            compiled.artifact.validation.status,
            PolicyValidationStatus::Blocked
        );
    }

    #[test]
    fn source_origin_and_validator_are_integrity_bound_with_v1_read_compatibility() {
        let compiled = compile_policy_source(&valid_source("7")).expect("compile v2 source");
        let origin = compiled.source.source_origin.as_ref().expect("v2 origin");
        assert_eq!(origin.source_system, "unit-test");
        assert_eq!(
            compiled.artifact.source_origin,
            compiled.source.source_origin
        );
        let mut tampered_source = compiled.source.clone();
        tampered_source
            .source_origin
            .as_mut()
            .unwrap()
            .source_system = "changed-system".to_string();
        assert_eq!(
            tampered_source.validate(),
            Err("policy_source_artifact_metadata_mismatch".to_string())
        );

        let mut status = compiled.artifact.clone();
        status.validation.status = PolicyValidationStatus::Blocked;
        assert_eq!(
            status.validate(),
            Err("policy_validation_disposition_mismatch".to_string())
        );
        let mut blockers = compiled.artifact.clone();
        blockers.validation.blockers.push("fabricated".to_string());
        assert_eq!(
            blockers.validate(),
            Err("policy_validation_disposition_mismatch".to_string())
        );

        let legacy = serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA_V1,
            "policy_key": "legacy.policy",
            "source_version": "1",
            "owner_ref": "organization:legacy",
            "rules": [{"kind":"review_requirement","rule_id":"r","operation_kind":"filesystem.write","required":true,"reason":"legacy input remains readable"}]
        })).unwrap();
        let legacy = compile_policy_source(&legacy).expect("compile v1 compatibility source");
        assert_eq!(legacy.source.schema, POLICY_SOURCE_ARTIFACT_SCHEMA_V1);
        assert_eq!(legacy.artifact.schema, POLICY_ARTIFACT_SCHEMA_V1);
        assert!(legacy.source.source_origin.is_none());
        legacy.validate().expect("v1 compatibility validates");
    }

    #[test]
    fn identifier_reason_rule_count_and_ordering_contracts_are_exact() {
        let source = |policy_key: String, rules: Vec<Value>| {
            serde_json::to_vec(&serde_json::json!({
                "schema": POLICY_SOURCE_INPUT_SCHEMA,
                "policy_key": policy_key,
                "source_version": "1",
                "owner_ref": "organization:test",
                "source_origin": {"source_system":"test","source_uri":"test://bounds"},
                "validity": {"mode":"unbounded"},
                "rules": rules
            }))
            .unwrap()
        };
        let rule = |rule_id: &str, reason: String| {
            serde_json::json!({
                "kind":"review_requirement",
                "rule_id":rule_id,
                "operation_kind":"filesystem.write",
                "required":true,
                "reason":reason
            })
        };

        assert_eq!(
            compile_policy_source(&source("valid".to_string(), Vec::new())).unwrap_err(),
            "policy_source_requires_rules"
        );
        compile_policy_source(&source("a".repeat(160), vec![rule("r", "ok".to_string())]))
            .expect("maximum identifier accepted");
        assert_eq!(
            compile_policy_source(&source("a".repeat(161), vec![rule("r", "ok".to_string())]))
                .unwrap_err(),
            "invalid_policy_key_length"
        );
        assert!(compile_policy_source(&source(
            "valid".to_string(),
            vec![rule("", "ok".to_string())]
        ))
        .unwrap_err()
        .contains("invalid_policy_rule_id_length"));
        assert!(compile_policy_source(&source(
            "valid".to_string(),
            vec![rule("r", String::new())]
        ))
        .unwrap_err()
        .contains("invalid_policy_rule_reason"));
        assert!(compile_policy_source(&source(
            "valid".to_string(),
            vec![rule("r", "x".repeat(513))]
        ))
        .unwrap_err()
        .contains("invalid_policy_rule_reason"));
        assert!(compile_policy_source(br#"{"schema":"yai.policy_source_input.v2","policy_key":"a","source_version":"1","owner_ref":"o","source_origin":{"source_system":"test","source_uri":"test://a"},"rules":[{}]}"#)
            .unwrap_err()
            .contains("policy_rule_kind_missing"));
        assert!(compile_policy_source(br#"{"schema":"yai.policy_source_input.v2","policy_key":"a","source_version":"1","owner_ref":"o","source_origin":{"source_system":"test","source_uri":"test://a"},"rules":[{"kind":"future"}],"unexpected":true}"#)
            .unwrap_err()
            .contains("unknown field"));
        let local_origin = source("valid".to_string(), vec![rule("r", "ok".to_string())]);
        let local_origin = String::from_utf8(local_origin)
            .unwrap()
            .replace("test://bounds", "file:///tmp/policy.json");
        assert_eq!(
            compile_policy_source(local_origin.as_bytes()).unwrap_err(),
            "invalid_source_uri"
        );

        let original = compile_policy_source(&valid_source("order")).unwrap();
        let mut reordered: Value = serde_json::from_slice(&valid_source("order")).unwrap();
        reordered["rules"].as_array_mut().unwrap().reverse();
        let reordered = compile_policy_source(&serde_json::to_vec(&reordered).unwrap()).unwrap();
        assert_ne!(original.source.source_id, reordered.source.source_id);
        let ids = |compilation: &PolicyCompilation| {
            compilation
                .artifact
                .policy_ir
                .rules
                .iter()
                .map(|rule| rule.rule_id().to_string())
                .collect::<BTreeSet<_>>()
        };
        assert_eq!(ids(&original), ids(&reordered));
    }
}
#[test]
fn v3_authority_requirement_is_typed_provenanced_and_v2_remains_unchanged() {
    let source = serde_json::to_vec(&serde_json::json!({
        "schema": POLICY_SOURCE_INPUT_SCHEMA,
        "policy_key":"authority",
        "source_version":"1",
        "owner_ref":"organization:acme",
        "source_origin":{"source_system":"unit-test","source_uri":"test://authority/1"},
        "validity":{"mode":"unbounded"},
        "rules":[{
            "kind":"authority_requirement",
            "rule_id":"reviewer-role",
            "operation_kind":"filesystem.write",
            "resource_kind":"filesystem",
            "subject":"reviewer",
            "required_role":"policy-reviewer",
            "reason":"human approval requires a Case-bound role"
        }]
    }))
    .unwrap();
    let compilation = compile_policy_source(&source).unwrap();
    assert_eq!(compilation.artifact.schema, POLICY_ARTIFACT_SCHEMA_V4);
    assert_eq!(compilation.artifact.parsed.schema, PARSED_POLICY_SCHEMA);
    assert!(matches!(
        compilation.artifact.policy_ir.rules.as_slice(),
        [NormalizedPolicyRule::AuthorityRequirement {
            subject: AuthoritySubject::Reviewer,
            required_role,
            provenance,
            ..
        }] if required_role == "policy-reviewer"
            && provenance.source_id == compilation.source.source_id
    ));

    let legacy = serde_json::to_vec(&serde_json::json!({
        "schema": POLICY_SOURCE_INPUT_SCHEMA_V2,
        "policy_key":"authority",
        "source_version":"legacy",
        "owner_ref":"organization:acme",
        "source_origin":{"source_system":"unit-test","source_uri":"test://authority/legacy"},
        "rules":[{
            "kind":"authority_requirement",
            "rule_id":"reviewer-role",
            "operation_kind":"filesystem.write",
            "resource_kind":"filesystem",
            "subject":"reviewer",
            "required_role":"policy-reviewer",
            "reason":"not part of v2 semantics"
        }]
    }))
    .unwrap();
    let legacy = compile_policy_source(&legacy).unwrap();
    assert_eq!(legacy.artifact.schema, POLICY_ARTIFACT_SCHEMA_V2);
    assert_eq!(legacy.artifact.policy_ir.rules.len(), 0);
    assert_eq!(legacy.artifact.policy_ir.unresolved.len(), 1);
    assert_eq!(
        legacy.artifact.validation.status,
        PolicyValidationStatus::Blocked
    );
}

#[test]
fn wave11_policy_validity_contract_is_explicit_ordered_and_identity_bearing() {
    let source = |validity: serde_json::Value| {
        serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key":"temporal",
            "source_version":"1",
            "owner_ref":"organization:acme",
            "source_origin":{"source_system":"unit-test","source_uri":"test://temporal/1"},
            "validity": validity,
            "rules":[{
                "kind":"operation_restriction",
                "rule_id":"allow-write",
                "operation_kind":"filesystem.write",
                "resource_kind":"filesystem",
                "effect":"allow",
                "reason":"bounded authority test"
            }]
        }))
        .unwrap()
    };
    let bounded = compile_policy_source(&source(serde_json::json!({
        "mode":"bounded",
        "valid_from_unix_ms":100,
        "refresh_after_unix_ms":200,
        "expires_at_unix_ms":300
    })))
    .expect("ordered bounded validity");
    let unbounded = compile_policy_source(&source(serde_json::json!({"mode":"unbounded"})))
        .expect("explicit unbounded validity");
    assert_ne!(bounded.artifact.artifact_id, unbounded.artifact.artifact_id);
    assert_eq!(
        compile_policy_source(&source(serde_json::json!({
            "mode":"bounded",
            "valid_from_unix_ms":300,
            "refresh_after_unix_ms":200,
            "expires_at_unix_ms":100
        })))
        .unwrap_err(),
        "policy_validity_window_order_invalid"
    );
    let missing = serde_json::to_vec(&serde_json::json!({
        "schema": POLICY_SOURCE_INPUT_SCHEMA,
        "policy_key":"temporal",
        "source_version":"1",
        "owner_ref":"organization:acme",
        "source_origin":{"source_system":"unit-test","source_uri":"test://temporal/missing"},
        "rules":[]
    }))
    .unwrap();
    assert_eq!(
        compile_policy_source(&missing).unwrap_err(),
        "policy_validity_contract_required"
    );
}
