//! Typed same-Tenant Case handoff protocol material.
//!
//! Handoff moves bounded work information between two Case histories. It does
//! not move Principal roles, policy, grants, resources, provider attachments,
//! effects, or arbitrary Case context. Canonical ownership remains with the
//! source and target Case transitions; any process/choreography view is
//! derived from their exact cross-references.

use crate::effect::digest_bytes;
use serde::{Deserialize, Serialize};

pub const HANDOFF_OFFER_SCHEMA: &str = "yai.handoff_offer.v1";
pub const HANDOFF_ACCEPTANCE_SCHEMA: &str = "yai.handoff_acceptance.v1";
pub const HANDOFF_DECLINE_SCHEMA: &str = "yai.handoff_decline.v1";
pub const HANDOFF_RESULT_SCHEMA: &str = "yai.handoff_result.v1";
pub const HANDOFF_RECONCILIATION_SCHEMA: &str = "yai.handoff_reconciliation.v1";
pub const MAX_HANDOFF_DATA_BYTES: usize = 16 * 1024;
pub const MAX_HANDOFF_EVIDENCE_REFS: usize = 32;
pub const MAX_HANDOFF_ROLE_REQUIREMENTS: usize = 16;
pub const MAX_HANDOFF_IDENTIFIER_BYTES: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffDataKind {
    Text,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffData {
    pub kind: HandoffDataKind,
    pub value: String,
}

impl HandoffData {
    pub fn validate(&self) -> Result<(), String> {
        if self.value.is_empty() || self.value.len() > MAX_HANDOFF_DATA_BYTES {
            return Err("handoff_data_bounds_invalid".to_string());
        }
        if self.kind == HandoffDataKind::Json {
            crate::governance::parse_strict_json(self.value.as_bytes())
                .map_err(|error| format!("handoff_json_invalid: {error}"))?;
        }
        Ok(())
    }

    pub fn digest(&self) -> String {
        digest_bytes(self.value.as_bytes())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffOutcome {
    Succeeded,
    Failed,
    Cancelled,
    Declined,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffOffer {
    pub schema: String,
    pub handoff_id: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub source_case_id: String,
    pub target_case_id: String,
    pub source_binding_id: String,
    pub source_node_id: String,
    pub request: HandoffData,
    #[serde(default)]
    pub required_target_roles: Vec<String>,
    pub offered_by_principal_id: String,
    pub offered_at_generation: u64,
    pub offered_at_unix_ms: u64,
}

impl HandoffOffer {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        tenant_id: &str,
        source_case_id: &str,
        target_case_id: &str,
        source_binding_id: &str,
        source_node_id: &str,
        request: HandoffData,
        mut required_target_roles: Vec<String>,
        offered_by_principal_id: &str,
        offered_at_generation: u64,
        offered_at_unix_ms: u64,
    ) -> Result<Self, String> {
        required_target_roles.sort();
        required_target_roles.dedup();
        let material = serde_json::json!({
            "schema": HANDOFF_OFFER_SCHEMA,
            "tenant_id": tenant_id,
            "source_case_id": source_case_id,
            "target_case_id": target_case_id,
            "source_binding_id": source_binding_id,
            "source_node_id": source_node_id,
            "request": request,
            "required_target_roles": required_target_roles,
            "offered_by_principal_id": offered_by_principal_id,
            "offered_at_generation": offered_at_generation,
            "offered_at_unix_ms": offered_at_unix_ms,
        });
        let integrity_digest = digest_json(&material)?;
        let result = Self {
            schema: HANDOFF_OFFER_SCHEMA.to_string(),
            handoff_id: format!("handoff:{}", digest_component(&integrity_digest)),
            integrity_digest,
            tenant_id: tenant_id.to_string(),
            source_case_id: source_case_id.to_string(),
            target_case_id: target_case_id.to_string(),
            source_binding_id: source_binding_id.to_string(),
            source_node_id: source_node_id.to_string(),
            request,
            required_target_roles,
            offered_by_principal_id: offered_by_principal_id.to_string(),
            offered_at_generation,
            offered_at_unix_ms,
        };
        result.validate()?;
        Ok(result)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.request.validate()?;
        if self.schema != HANDOFF_OFFER_SCHEMA
            || !self.tenant_id.starts_with("tenant:")
            || !self.source_case_id.starts_with("case:")
            || !self.target_case_id.starts_with("case:")
            || self.source_case_id == self.target_case_id
            || !bounded_identifier(&self.source_binding_id)
            || !bounded_identifier(&self.source_node_id)
            || !self.offered_by_principal_id.starts_with("principal:")
            || !bounded_identifier(&self.handoff_id)
            || !bounded_identifier(&self.tenant_id)
            || !bounded_identifier(&self.source_case_id)
            || !bounded_identifier(&self.target_case_id)
            || !bounded_identifier(&self.offered_by_principal_id)
            || self.required_target_roles.len() > MAX_HANDOFF_ROLE_REQUIREMENTS
            || self
                .required_target_roles
                .iter()
                .any(|role| !bounded_identifier(role))
        {
            return Err("handoff_offer_contract_invalid".to_string());
        }
        let mut roles = self.required_target_roles.clone();
        roles.sort();
        roles.dedup();
        if roles != self.required_target_roles {
            return Err("handoff_offer_roles_not_canonical".to_string());
        }
        let material = serde_json::json!({
            "schema": self.schema,
            "tenant_id": self.tenant_id,
            "source_case_id": self.source_case_id,
            "target_case_id": self.target_case_id,
            "source_binding_id": self.source_binding_id,
            "source_node_id": self.source_node_id,
            "request": self.request,
            "required_target_roles": self.required_target_roles,
            "offered_by_principal_id": self.offered_by_principal_id,
            "offered_at_generation": self.offered_at_generation,
            "offered_at_unix_ms": self.offered_at_unix_ms,
        });
        let digest = digest_json(&material)?;
        if digest != self.integrity_digest
            || self.handoff_id != format!("handoff:{}", digest_component(&digest))
        {
            return Err("handoff_offer_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffAcceptance {
    pub schema: String,
    pub acceptance_id: String,
    pub handoff_id: String,
    pub source_case_id: String,
    pub target_case_id: String,
    pub accepted_by_principal_id: String,
    pub accepted_by_participant_id: String,
    pub accepted_at_generation: u64,
    pub accepted_at_unix_ms: u64,
}

impl HandoffAcceptance {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        offer: &HandoffOffer,
        principal_id: &str,
        participant_id: &str,
        accepted_at_generation: u64,
        accepted_at_unix_ms: u64,
    ) -> Result<Self, String> {
        let material = serde_json::json!({
            "schema": HANDOFF_ACCEPTANCE_SCHEMA,
            "handoff_id": offer.handoff_id,
            "source_case_id": offer.source_case_id,
            "target_case_id": offer.target_case_id,
            "accepted_by_principal_id": principal_id,
            "accepted_by_participant_id": participant_id,
            "accepted_at_generation": accepted_at_generation,
            "accepted_at_unix_ms": accepted_at_unix_ms,
        });
        let digest = digest_json(&material)?;
        let result = Self {
            schema: HANDOFF_ACCEPTANCE_SCHEMA.to_string(),
            acceptance_id: format!("handoff-acceptance:{}", digest_component(&digest)),
            handoff_id: offer.handoff_id.clone(),
            source_case_id: offer.source_case_id.clone(),
            target_case_id: offer.target_case_id.clone(),
            accepted_by_principal_id: principal_id.to_string(),
            accepted_by_participant_id: participant_id.to_string(),
            accepted_at_generation,
            accepted_at_unix_ms,
        };
        result.validate()?;
        Ok(result)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != HANDOFF_ACCEPTANCE_SCHEMA
            || !self.handoff_id.starts_with("handoff:")
            || !self.source_case_id.starts_with("case:")
            || !self.target_case_id.starts_with("case:")
            || !self.accepted_by_principal_id.starts_with("principal:")
            || !self.accepted_by_participant_id.starts_with("participant:")
            || !self.acceptance_id.starts_with("handoff-acceptance:")
            || !bounded_identifier(&self.acceptance_id)
            || !bounded_identifier(&self.handoff_id)
            || !bounded_identifier(&self.source_case_id)
            || !bounded_identifier(&self.target_case_id)
            || !bounded_identifier(&self.accepted_by_principal_id)
            || !bounded_identifier(&self.accepted_by_participant_id)
        {
            return Err("handoff_acceptance_contract_invalid".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffDecline {
    pub schema: String,
    pub decline_id: String,
    pub handoff_id: String,
    pub source_case_id: String,
    pub target_case_id: String,
    pub declined_by_principal_id: String,
    pub declined_by_participant_id: String,
    pub reason: String,
    pub declined_at_generation: u64,
    pub declined_at_unix_ms: u64,
}

impl HandoffDecline {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        offer: &HandoffOffer,
        principal_id: &str,
        participant_id: &str,
        reason: &str,
        generation: u64,
        now_unix_ms: u64,
    ) -> Result<Self, String> {
        if reason.is_empty() || reason.len() > MAX_HANDOFF_DATA_BYTES {
            return Err("handoff_decline_reason_bounds_invalid".to_string());
        }
        let material = serde_json::json!({
            "schema": HANDOFF_DECLINE_SCHEMA,
            "handoff_id": offer.handoff_id,
            "source_case_id": offer.source_case_id,
            "target_case_id": offer.target_case_id,
            "declined_by_principal_id": principal_id,
            "declined_by_participant_id": participant_id,
            "reason": reason,
            "declined_at_generation": generation,
            "declined_at_unix_ms": now_unix_ms,
        });
        let digest = digest_json(&material)?;
        let value = Self {
            schema: HANDOFF_DECLINE_SCHEMA.to_string(),
            decline_id: format!("handoff-decline:{}", digest_component(&digest)),
            handoff_id: offer.handoff_id.clone(),
            source_case_id: offer.source_case_id.clone(),
            target_case_id: offer.target_case_id.clone(),
            declined_by_principal_id: principal_id.to_string(),
            declined_by_participant_id: participant_id.to_string(),
            reason: reason.to_string(),
            declined_at_generation: generation,
            declined_at_unix_ms: now_unix_ms,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != HANDOFF_DECLINE_SCHEMA
            || !self.decline_id.starts_with("handoff-decline:")
            || !self.handoff_id.starts_with("handoff:")
            || !self.source_case_id.starts_with("case:")
            || !self.target_case_id.starts_with("case:")
            || !self.declined_by_principal_id.starts_with("principal:")
            || !self.declined_by_participant_id.starts_with("participant:")
            || self.reason.is_empty()
            || self.reason.len() > MAX_HANDOFF_DATA_BYTES
            || !bounded_identifier(&self.decline_id)
            || !bounded_identifier(&self.handoff_id)
            || !bounded_identifier(&self.source_case_id)
            || !bounded_identifier(&self.target_case_id)
            || !bounded_identifier(&self.declined_by_principal_id)
            || !bounded_identifier(&self.declined_by_participant_id)
        {
            return Err("handoff_decline_contract_invalid".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffResult {
    pub schema: String,
    pub result_id: String,
    pub handoff_id: String,
    pub acceptance_id: String,
    pub source_case_id: String,
    pub target_case_id: String,
    pub outcome: HandoffOutcome,
    pub result: HandoffData,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub recorded_by_principal_id: String,
    pub recorded_by_participant_id: String,
    pub recorded_at_generation: u64,
    pub recorded_at_unix_ms: u64,
}

impl HandoffResult {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        acceptance: &HandoffAcceptance,
        outcome: HandoffOutcome,
        result: HandoffData,
        mut evidence_refs: Vec<String>,
        principal_id: &str,
        participant_id: &str,
        generation: u64,
        now_unix_ms: u64,
    ) -> Result<Self, String> {
        result.validate()?;
        evidence_refs.sort();
        evidence_refs.dedup();
        if evidence_refs.len() > MAX_HANDOFF_EVIDENCE_REFS {
            return Err("handoff_result_evidence_bounds_invalid".to_string());
        }
        let material = serde_json::json!({
            "schema": HANDOFF_RESULT_SCHEMA,
            "handoff_id": acceptance.handoff_id,
            "acceptance_id": acceptance.acceptance_id,
            "source_case_id": acceptance.source_case_id,
            "target_case_id": acceptance.target_case_id,
            "outcome": outcome,
            "result": result,
            "evidence_refs": evidence_refs,
            "recorded_by_principal_id": principal_id,
            "recorded_by_participant_id": participant_id,
            "recorded_at_generation": generation,
            "recorded_at_unix_ms": now_unix_ms,
        });
        let digest = digest_json(&material)?;
        let value = Self {
            schema: HANDOFF_RESULT_SCHEMA.to_string(),
            result_id: format!("handoff-result:{}", digest_component(&digest)),
            handoff_id: acceptance.handoff_id.clone(),
            acceptance_id: acceptance.acceptance_id.clone(),
            source_case_id: acceptance.source_case_id.clone(),
            target_case_id: acceptance.target_case_id.clone(),
            outcome,
            result,
            evidence_refs,
            recorded_by_principal_id: principal_id.to_string(),
            recorded_by_participant_id: participant_id.to_string(),
            recorded_at_generation: generation,
            recorded_at_unix_ms: now_unix_ms,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.result.validate()?;
        if self.schema != HANDOFF_RESULT_SCHEMA
            || !self.result_id.starts_with("handoff-result:")
            || !self.handoff_id.starts_with("handoff:")
            || !self.acceptance_id.starts_with("handoff-acceptance:")
            || !self.recorded_by_principal_id.starts_with("principal:")
            || !self.recorded_by_participant_id.starts_with("participant:")
            || self.evidence_refs.len() > MAX_HANDOFF_EVIDENCE_REFS
            || !bounded_identifier(&self.result_id)
            || !bounded_identifier(&self.handoff_id)
            || !bounded_identifier(&self.acceptance_id)
            || !bounded_identifier(&self.source_case_id)
            || !bounded_identifier(&self.target_case_id)
            || !bounded_identifier(&self.recorded_by_principal_id)
            || !bounded_identifier(&self.recorded_by_participant_id)
            || self
                .evidence_refs
                .iter()
                .any(|value| !bounded_identifier(value))
        {
            return Err("handoff_result_contract_invalid".to_string());
        }
        let mut refs = self.evidence_refs.clone();
        refs.sort();
        refs.dedup();
        if refs != self.evidence_refs {
            return Err("handoff_result_evidence_not_canonical".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffReconciliation {
    pub schema: String,
    pub reconciliation_id: String,
    pub handoff_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_acceptance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_result_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_decline_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_terminal_transition_id: Option<String>,
    pub outcome: HandoffOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<HandoffData>,
    pub result_digest: String,
    pub reconciled_by_principal_id: String,
    pub reconciled_at_generation: u64,
    pub reconciled_at_unix_ms: u64,
}

impl HandoffReconciliation {
    pub fn build(
        result: &HandoffResult,
        principal_id: &str,
        generation: u64,
        now_unix_ms: u64,
    ) -> Result<Self, String> {
        let material = serde_json::json!({
            "schema": HANDOFF_RECONCILIATION_SCHEMA,
            "handoff_id": result.handoff_id,
            "target_acceptance_id": result.acceptance_id,
            "target_result_id": result.result_id,
            "target_decline_id": null,
            "target_terminal_transition_id": null,
            "outcome": result.outcome,
            "result": result.result,
            "result_digest": result.result.digest(),
            "reconciled_by_principal_id": principal_id,
            "reconciled_at_generation": generation,
            "reconciled_at_unix_ms": now_unix_ms,
        });
        let digest = digest_json(&material)?;
        let value = Self {
            schema: HANDOFF_RECONCILIATION_SCHEMA.to_string(),
            reconciliation_id: format!("handoff-reconciliation:{}", digest_component(&digest)),
            handoff_id: result.handoff_id.clone(),
            target_acceptance_id: Some(result.acceptance_id.clone()),
            target_result_id: Some(result.result_id.clone()),
            target_decline_id: None,
            target_terminal_transition_id: None,
            outcome: result.outcome.clone(),
            result: Some(result.result.clone()),
            result_digest: result.result.digest(),
            reconciled_by_principal_id: principal_id.to_string(),
            reconciled_at_generation: generation,
            reconciled_at_unix_ms: now_unix_ms,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn build_declined(
        decline: &HandoffDecline,
        principal_id: &str,
        generation: u64,
        now_unix_ms: u64,
    ) -> Result<Self, String> {
        let result = HandoffData {
            kind: HandoffDataKind::Text,
            value: decline.reason.clone(),
        };
        Self::build_material(
            &decline.handoff_id,
            None,
            None,
            Some(decline.decline_id.clone()),
            None,
            HandoffOutcome::Declined,
            Some(result),
            principal_id,
            generation,
            now_unix_ms,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build_target_terminal(
        handoff_id: &str,
        acceptance_id: Option<String>,
        target_transition_id: &str,
        outcome: HandoffOutcome,
        principal_id: &str,
        generation: u64,
        now_unix_ms: u64,
    ) -> Result<Self, String> {
        if !matches!(outcome, HandoffOutcome::Cancelled | HandoffOutcome::Failed) {
            return Err("handoff_terminal_outcome_invalid".to_string());
        }
        Self::build_material(
            handoff_id,
            acceptance_id,
            None,
            None,
            Some(target_transition_id.to_string()),
            outcome,
            None,
            principal_id,
            generation,
            now_unix_ms,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build_material(
        handoff_id: &str,
        target_acceptance_id: Option<String>,
        target_result_id: Option<String>,
        target_decline_id: Option<String>,
        target_terminal_transition_id: Option<String>,
        outcome: HandoffOutcome,
        result: Option<HandoffData>,
        principal_id: &str,
        generation: u64,
        now_unix_ms: u64,
    ) -> Result<Self, String> {
        if let Some(result) = &result {
            result.validate()?;
        }
        let result_digest = result
            .as_ref()
            .map(HandoffData::digest)
            .unwrap_or_else(|| digest_bytes(b"no-result-payload"));
        let material = serde_json::json!({
            "schema": HANDOFF_RECONCILIATION_SCHEMA,
            "handoff_id": handoff_id,
            "target_acceptance_id": target_acceptance_id,
            "target_result_id": target_result_id,
            "target_decline_id": target_decline_id,
            "target_terminal_transition_id": target_terminal_transition_id,
            "outcome": outcome,
            "result": result,
            "result_digest": result_digest,
            "reconciled_by_principal_id": principal_id,
            "reconciled_at_generation": generation,
            "reconciled_at_unix_ms": now_unix_ms,
        });
        let digest = digest_json(&material)?;
        let value = Self {
            schema: HANDOFF_RECONCILIATION_SCHEMA.to_string(),
            reconciliation_id: format!("handoff-reconciliation:{}", digest_component(&digest)),
            handoff_id: handoff_id.to_string(),
            target_acceptance_id,
            target_result_id,
            target_decline_id,
            target_terminal_transition_id,
            outcome,
            result,
            result_digest,
            reconciled_by_principal_id: principal_id.to_string(),
            reconciled_at_generation: generation,
            reconciled_at_unix_ms: now_unix_ms,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != HANDOFF_RECONCILIATION_SCHEMA
            || !self
                .reconciliation_id
                .starts_with("handoff-reconciliation:")
            || !self.handoff_id.starts_with("handoff:")
            || !self.result_digest.starts_with("sha256:")
            || !self.reconciled_by_principal_id.starts_with("principal:")
            || !bounded_identifier(&self.reconciliation_id)
            || !bounded_identifier(&self.handoff_id)
            || !bounded_identifier(&self.reconciled_by_principal_id)
        {
            return Err("handoff_reconciliation_contract_invalid".to_string());
        }
        let dispositions = usize::from(self.target_result_id.is_some())
            + usize::from(self.target_decline_id.is_some())
            + usize::from(self.target_terminal_transition_id.is_some());
        if dispositions != 1
            || self.target_acceptance_id.as_ref().is_some_and(|value| {
                !value.starts_with("handoff-acceptance:") || !bounded_identifier(value)
            })
            || self.target_result_id.as_ref().is_some_and(|value| {
                !value.starts_with("handoff-result:") || !bounded_identifier(value)
            })
            || self.target_decline_id.as_ref().is_some_and(|value| {
                !value.starts_with("handoff-decline:") || !bounded_identifier(value)
            })
            || self
                .target_terminal_transition_id
                .as_ref()
                .is_some_and(|value| {
                    !value.starts_with("transition:") || !bounded_identifier(value)
                })
            || self
                .result
                .as_ref()
                .map(HandoffData::digest)
                .unwrap_or_else(|| digest_bytes(b"no-result-payload"))
                != self.result_digest
        {
            return Err("handoff_reconciliation_disposition_invalid".to_string());
        }
        Ok(())
    }

    pub fn target_disposition_id(&self) -> &str {
        self.target_result_id
            .as_deref()
            .or(self.target_decline_id.as_deref())
            .or(self.target_terminal_transition_id.as_deref())
            .expect("validated Handoff reconciliation has one disposition")
    }
}

fn digest_json(value: &serde_json::Value) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|bytes| digest_bytes(&bytes))
        .map_err(|error| format!("handoff_digest_encode_failed: {error}"))
}

fn bounded_identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_HANDOFF_IDENTIFIER_BYTES
}

fn digest_component(digest: &str) -> &str {
    let value = digest.strip_prefix("sha256:").unwrap_or(digest);
    &value[..value.len().min(32)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offer_identity_is_content_bound_and_same_case_is_rejected() {
        let offer = HandoffOffer::build(
            "tenant:test",
            "case:a",
            "case:b",
            "binding:a",
            "handoff-node",
            HandoffData {
                kind: HandoffDataKind::Text,
                value: "bounded task".to_string(),
            },
            vec!["operator".to_string()],
            "principal:owner",
            3,
            10,
        )
        .unwrap();
        offer.validate().unwrap();
        assert!(HandoffOffer::build(
            "tenant:test",
            "case:a",
            "case:a",
            "binding:a",
            "handoff-node",
            HandoffData {
                kind: HandoffDataKind::Text,
                value: "task".to_string(),
            },
            vec![],
            "principal:owner",
            3,
            10,
        )
        .is_err());
    }
}
