//! Tenant-scoped provider governance and Case-local provider routing contracts.
//!
//! Provider configuration, qualification evidence and administrative trust form
//! one shared Tenant owner family. Health is operational and non-authoritative.
//! Bindings, selections and attempt outcomes are embedded in Case Transition
//! history by `transition`; this module only owns their typed contracts and the
//! deterministic selector.

use crate::effect::digest_bytes;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub const PROVIDER_TARGET_SCHEMA: &str = "yai.provider_target.v1";
pub const PROVIDER_QUALIFICATION_SCHEMA_V1: &str = "yai.provider_qualification.v1";
pub const PROVIDER_QUALIFICATION_SCHEMA: &str = "yai.provider_qualification.v2";
pub const PROVIDER_TRUST_EVENT_SCHEMA: &str = "yai.provider_trust_event.v1";
pub const PROVIDER_HEALTH_SCHEMA_V1: &str = "yai.provider_health.v1";
pub const PROVIDER_HEALTH_SCHEMA: &str = "yai.provider_health.v2";
pub const CASE_PROVIDER_BINDING_SCHEMA: &str = "yai.case_provider_binding.v1";
pub const PROVIDER_REQUIREMENT_SCHEMA: &str = "yai.provider_requirement.v1";
pub const PROVIDER_SELECTION_SCHEMA: &str = "yai.provider_selection.v1";
pub const PROVIDER_ATTEMPT_OUTCOME_SCHEMA: &str = "yai.provider_attempt_outcome.v1";
pub const PROVIDER_SELECTOR_VERSION: &str = "yai.provider_selector.v1";

pub const MAX_PROVIDER_TARGETS_PER_TENANT: usize = 128;
pub const MAX_PROVIDER_TARGETS_PER_CASE: usize = 32;
pub const MAX_PROVIDER_ENDPOINT_BYTES: usize = 2048;
pub const MAX_PROVIDER_MODEL_ID_BYTES: usize = 256;
pub const MAX_PROVIDER_KEY_BYTES: usize = 128;
pub const MAX_PROVIDER_CREDENTIAL_REF_BYTES: usize = 256;
pub const MAX_PROVIDER_CAPABILITIES: usize = 16;
pub const MAX_PROVIDER_EXCLUSIONS: usize = 32;
pub const MAX_PROVIDER_EVIDENCE_REFS: usize = 32;
pub const MAX_PROVIDER_ATTEMPTS_PER_TURN: u32 = 3;
pub const PROVIDER_HEALTH_FRESHNESS_MS: u64 = 60_000;
pub const PROVIDER_CIRCUIT_FAILURE_THRESHOLD: u32 = 3;
pub const PROVIDER_CIRCUIT_COOLDOWN_MS: u64 = 30_000;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAdapterKind {
    OpenAiCompatible,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderLocality {
    Loopback,
    PrivateNetwork,
    Remote,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderTargetInput {
    pub tenant_id: String,
    pub provider_key: String,
    pub adapter: ProviderAdapterKind,
    pub endpoint: String,
    pub model_id: String,
    pub credential_ref: String,
    pub locality: ProviderLocality,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension_adapter_id: Option<String>,
    pub created_by_principal_id: String,
    pub created_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderTarget {
    pub schema: String,
    pub target_id: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub provider_key: String,
    pub adapter: ProviderAdapterKind,
    pub endpoint: String,
    pub model_id: String,
    pub credential_ref: String,
    pub locality: ProviderLocality,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension_adapter_id: Option<String>,
    pub created_by_principal_id: String,
    pub created_at_unix_ms: u64,
}

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

fn endpoint_authority(endpoint: &str) -> Result<(&str, &str, &str), String> {
    let (scheme, remainder) = endpoint
        .split_once("://")
        .ok_or_else(|| "provider_endpoint_scheme_missing".to_string())?;
    if scheme != "http" && scheme != "https" {
        return Err("provider_endpoint_scheme_not_admitted".to_string());
    }
    if remainder.contains('#') || remainder.contains('?') || remainder.contains('@') {
        return Err("provider_endpoint_credentials_query_or_fragment_forbidden".to_string());
    }
    let (authority, path) = remainder.split_once('/').unwrap_or((remainder, ""));
    if authority.is_empty()
        || authority.len() > 512
        || path.len() > 1536
        || path.chars().any(char::is_whitespace)
        || path.contains('\\')
    {
        return Err("provider_endpoint_authority_or_path_invalid".to_string());
    }
    Ok((scheme, authority, path))
}

fn normalized_endpoint_host(authority: &str) -> Result<&str, String> {
    let (host, port) = if let Some(bracketed) = authority.strip_prefix('[') {
        let (host, suffix) = bracketed
            .split_once(']')
            .ok_or_else(|| "provider_endpoint_ipv6_bracket_invalid".to_string())?;
        let port = if suffix.is_empty() {
            None
        } else {
            Some(
                suffix
                    .strip_prefix(':')
                    .ok_or_else(|| "provider_endpoint_authority_invalid".to_string())?,
            )
        };
        (host, port)
    } else if authority.matches(':').count() == 1 {
        let (host, port) = authority
            .rsplit_once(':')
            .ok_or_else(|| "provider_endpoint_authority_invalid".to_string())?;
        (host, Some(port))
    } else if authority.contains(':') {
        return Err("provider_endpoint_ipv6_must_be_bracketed".to_string());
    } else {
        (authority, None)
    };
    if host.is_empty()
        || host.len() > 253
        || (!host.contains(':')
            && !host
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '-'))
        || port.is_some_and(|value| value.parse::<u16>().is_err())
    {
        return Err("provider_endpoint_host_or_port_invalid".to_string());
    }
    Ok(host)
}

pub fn normalize_provider_endpoint(
    endpoint: &str,
    locality: &ProviderLocality,
) -> Result<String, String> {
    if endpoint.is_empty() || endpoint.len() > MAX_PROVIDER_ENDPOINT_BYTES {
        return Err("provider_endpoint_size_invalid".to_string());
    }
    let trimmed = endpoint.trim_end_matches('/');
    let (scheme, authority, path) = endpoint_authority(trimmed)?;
    let authority = authority.to_ascii_lowercase();
    let host = normalized_endpoint_host(&authority)?;
    if let Ok(address) = host.parse::<IpAddr>() {
        if !provider_address_admitted(locality, address) {
            return Err("provider_endpoint_literal_locality_mismatch".to_string());
        }
    }
    if *locality == ProviderLocality::Remote && scheme != "https" {
        return Err("provider_remote_plain_http_not_admitted".to_string());
    }
    let suffix = if path.is_empty() {
        String::new()
    } else {
        format!("/{path}")
    };
    Ok(format!("{scheme}://{authority}{suffix}"))
}

fn ipv4_private(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    octets[0] == 10
        || (octets[0] == 172 && (16..=31).contains(&octets[1]))
        || (octets[0] == 192 && octets[1] == 168)
}

fn ipv6_unique_local(address: Ipv6Addr) -> bool {
    address.segments()[0] & 0xfe00 == 0xfc00
}

fn ipv6_link_local(address: Ipv6Addr) -> bool {
    address.segments()[0] & 0xffc0 == 0xfe80
}

/// Dispatch-time locality predicate. Callers must apply it to every resolved
/// address and refuse a mixed answer set.
pub fn provider_address_admitted(locality: &ProviderLocality, address: IpAddr) -> bool {
    if let IpAddr::V6(address_v6) = address {
        if let Some(mapped) = address_v6.to_ipv4_mapped() {
            return provider_address_admitted(locality, IpAddr::V4(mapped));
        }
    }
    match (locality, address) {
        (ProviderLocality::Loopback, address) => address.is_loopback(),
        (ProviderLocality::PrivateNetwork, IpAddr::V4(address)) => ipv4_private(address),
        (ProviderLocality::PrivateNetwork, IpAddr::V6(address)) => ipv6_unique_local(address),
        (ProviderLocality::Remote, IpAddr::V4(address)) => {
            !address.is_loopback()
                && !address.is_unspecified()
                && !address.is_multicast()
                && !address.is_link_local()
                && !ipv4_private(address)
        }
        (ProviderLocality::Remote, IpAddr::V6(address)) => {
            !address.is_loopback()
                && !address.is_unspecified()
                && !address.is_multicast()
                && !ipv6_link_local(address)
                && !ipv6_unique_local(address)
        }
    }
}

impl ProviderTarget {
    pub fn from_input(mut input: ProviderTargetInput) -> Result<Self, String> {
        require_identifier("provider_tenant_id", &input.tenant_id, 256)?;
        require_identifier("provider_key", &input.provider_key, MAX_PROVIDER_KEY_BYTES)?;
        require_identifier(
            "provider_model_id",
            &input.model_id,
            MAX_PROVIDER_MODEL_ID_BYTES,
        )?;
        require_identifier(
            "provider_credential_ref",
            &input.credential_ref,
            MAX_PROVIDER_CREDENTIAL_REF_BYTES,
        )?;
        require_identifier(
            "provider_creator_principal",
            &input.created_by_principal_id,
            256,
        )?;
        if !input.tenant_id.starts_with("tenant:") {
            return Err("provider_target_tenant_invalid".to_string());
        }
        if input.credential_ref != "none" && !input.credential_ref.starts_with("env:") {
            return Err("provider_credential_ref_kind_not_admitted".to_string());
        }
        if let Some(extension) = &input.extension_adapter_id {
            require_identifier("provider_extension_adapter_id", extension, 128)?;
        }
        input.endpoint = normalize_provider_endpoint(&input.endpoint, &input.locality)?;
        let identity_digest = digest_of(
            &(
                &input.tenant_id,
                &input.provider_key,
                &input.adapter,
                &input.endpoint,
                &input.model_id,
                &input.credential_ref,
                &input.locality,
                &input.extension_adapter_id,
            ),
            "provider_target_identity",
        )?;
        let integrity_digest = digest_of(&input, "provider_target_integrity")?;
        let target = Self {
            schema: PROVIDER_TARGET_SCHEMA.to_string(),
            target_id: short_identity("provider-target", &identity_digest),
            integrity_digest,
            tenant_id: input.tenant_id,
            provider_key: input.provider_key,
            adapter: input.adapter,
            endpoint: input.endpoint,
            model_id: input.model_id,
            credential_ref: input.credential_ref,
            locality: input.locality,
            extension_adapter_id: input.extension_adapter_id,
            created_by_principal_id: input.created_by_principal_id,
            created_at_unix_ms: input.created_at_unix_ms,
        };
        Ok(target)
    }

    pub fn same_configuration(&self, other: &Self) -> bool {
        self.tenant_id == other.tenant_id
            && self.provider_key == other.provider_key
            && self.adapter == other.adapter
            && self.endpoint == other.endpoint
            && self.model_id == other.model_id
            && self.credential_ref == other.credential_ref
            && self.locality == other.locality
            && self.extension_adapter_id == other.extension_adapter_id
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != PROVIDER_TARGET_SCHEMA {
            return Err("unsupported_provider_target_schema".to_string());
        }
        let rebuilt = Self::from_input(ProviderTargetInput {
            tenant_id: self.tenant_id.clone(),
            provider_key: self.provider_key.clone(),
            adapter: self.adapter.clone(),
            endpoint: self.endpoint.clone(),
            model_id: self.model_id.clone(),
            credential_ref: self.credential_ref.clone(),
            locality: self.locality.clone(),
            extension_adapter_id: self.extension_adapter_id.clone(),
            created_by_principal_id: self.created_by_principal_id.clone(),
            created_at_unix_ms: self.created_at_unix_ms,
        })?;
        if rebuilt.target_id != self.target_id || rebuilt.integrity_digest != self.integrity_digest
        {
            return Err("provider_target_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCapability {
    ChatText,
    StructuredJsonObject,
    ModelExactAddressing,
    UsageAccounting,
    HealthProbe,
    /// Historical v1 name. It means only that extension-shaped telemetry was
    /// observed; it never authenticated a first-party implementation.
    FirstPartyTelemetry,
    ExtensionCompatibleTelemetry,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityProvenance {
    Configured,
    Observed,
    Qualified,
    /// Historical v1 name. It is not a cryptographic attestation.
    ExtensionAttested,
    ExtensionObserved,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderCapabilityEvidence {
    pub capability: ProviderCapability,
    pub provenance: CapabilityProvenance,
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_minimum: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderProbeEvidence {
    pub run_id: String,
    pub target_id: String,
    pub started_at_unix_ms: u64,
    pub completed_at_unix_ms: u64,
    pub transport_connected: bool,
    pub exact_model_addressed: bool,
    pub chat_text_envelope_valid: bool,
    pub structured_json_object_valid: bool,
    pub usage_accounting_observed: bool,
    pub health_endpoint_observed: bool,
    pub extension_telemetry_observed: bool,
    #[serde(default)]
    pub failure_codes: Vec<String>,
}

impl ProviderProbeEvidence {
    pub fn validate(&self) -> Result<(), String> {
        require_identifier("provider_probe_run_id", &self.run_id, 256)?;
        require_identifier("provider_probe_target_id", &self.target_id, 256)?;
        if self.completed_at_unix_ms < self.started_at_unix_ms || self.failure_codes.len() > 16 {
            return Err("provider_probe_evidence_invalid".to_string());
        }
        for code in &self.failure_codes {
            require_identifier("provider_probe_failure_code", code, 128)?;
        }
        if self.structured_json_object_valid && !self.chat_text_envelope_valid {
            return Err("provider_probe_json_without_chat_evidence".to_string());
        }
        Ok(())
    }
}

fn derived_capabilities(
    evidence: &ProviderProbeEvidence,
    schema: &str,
) -> Vec<ProviderCapabilityEvidence> {
    let mut capabilities = Vec::new();
    let mut add = |capability, proof: &str| {
        capabilities.push(ProviderCapabilityEvidence {
            capability,
            provenance: CapabilityProvenance::Qualified,
            evidence_refs: vec![format!("probe:{}:{proof}", evidence.run_id)],
            verified_minimum: None,
        });
    };
    if evidence.chat_text_envelope_valid {
        add(ProviderCapability::ChatText, "chat_text");
    }
    if evidence.structured_json_object_valid {
        add(
            ProviderCapability::StructuredJsonObject,
            "structured_json_object",
        );
    }
    if evidence.exact_model_addressed {
        add(
            ProviderCapability::ModelExactAddressing,
            "model_exact_addressing",
        );
    }
    if evidence.usage_accounting_observed {
        add(ProviderCapability::UsageAccounting, "usage_accounting");
    }
    if evidence.health_endpoint_observed {
        add(ProviderCapability::HealthProbe, "health_probe");
    }
    if evidence.extension_telemetry_observed {
        let (capability, provenance) = if schema == PROVIDER_QUALIFICATION_SCHEMA_V1 {
            (
                ProviderCapability::FirstPartyTelemetry,
                CapabilityProvenance::ExtensionAttested,
            )
        } else {
            (
                ProviderCapability::ExtensionCompatibleTelemetry,
                CapabilityProvenance::ExtensionObserved,
            )
        };
        capabilities.push(ProviderCapabilityEvidence {
            capability,
            provenance,
            evidence_refs: vec![format!("probe:{}:extension_telemetry", evidence.run_id)],
            verified_minimum: None,
        });
    }
    capabilities.sort_by(|left, right| left.capability.cmp(&right.capability));
    capabilities
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderQualification {
    pub schema: String,
    pub qualification_id: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub target_id: String,
    pub target_digest: String,
    /// Non-secret credential generation used by this qualification. Historical
    /// v1 records decode as generation zero.
    #[serde(default)]
    pub credential_revision: u64,
    pub suite_id: String,
    pub run_id: String,
    pub qualified_at_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until_unix_ms: Option<u64>,
    pub evidence: ProviderProbeEvidence,
    pub capabilities: Vec<ProviderCapabilityEvidence>,
    pub operator_principal_id: String,
}

#[derive(Serialize)]
struct QualificationIdentity<'a> {
    tenant_id: &'a str,
    target_id: &'a str,
    target_digest: &'a str,
    suite_id: &'a str,
    run_id: &'a str,
    qualified_at_unix_ms: u64,
    valid_until_unix_ms: Option<u64>,
    evidence: &'a ProviderProbeEvidence,
    capabilities: &'a [ProviderCapabilityEvidence],
    operator_principal_id: &'a str,
}

#[derive(Serialize)]
struct QualificationIdentityV2<'a> {
    schema: &'a str,
    credential_revision: u64,
    #[serde(flatten)]
    identity: QualificationIdentity<'a>,
}

impl ProviderQualification {
    pub fn from_evidence(
        target: &ProviderTarget,
        evidence: ProviderProbeEvidence,
        suite_id: &str,
        operator_principal_id: &str,
        valid_until_unix_ms: Option<u64>,
    ) -> Result<Self, String> {
        Self::from_evidence_at_credential_revision(
            target,
            evidence,
            suite_id,
            operator_principal_id,
            valid_until_unix_ms,
            0,
        )
    }

    pub fn from_evidence_at_credential_revision(
        target: &ProviderTarget,
        evidence: ProviderProbeEvidence,
        suite_id: &str,
        operator_principal_id: &str,
        valid_until_unix_ms: Option<u64>,
        credential_revision: u64,
    ) -> Result<Self, String> {
        target.validate()?;
        evidence.validate()?;
        if evidence.target_id != target.target_id {
            return Err("provider_qualification_evidence_target_mismatch".to_string());
        }
        require_identifier("provider_qualification_suite_id", suite_id, 128)?;
        require_identifier(
            "provider_qualification_operator",
            operator_principal_id,
            256,
        )?;
        if valid_until_unix_ms.is_some_and(|until| until <= evidence.completed_at_unix_ms) {
            return Err("provider_qualification_validity_invalid".to_string());
        }
        let capabilities = derived_capabilities(&evidence, PROVIDER_QUALIFICATION_SCHEMA);
        let identity = QualificationIdentity {
            tenant_id: &target.tenant_id,
            target_id: &target.target_id,
            target_digest: &target.integrity_digest,
            suite_id,
            run_id: &evidence.run_id,
            qualified_at_unix_ms: evidence.completed_at_unix_ms,
            valid_until_unix_ms,
            evidence: &evidence,
            capabilities: &capabilities,
            operator_principal_id,
        };
        let digest = digest_of(
            &QualificationIdentityV2 {
                schema: PROVIDER_QUALIFICATION_SCHEMA,
                credential_revision,
                identity,
            },
            "provider_qualification_identity",
        )?;
        let qualification = Self {
            schema: PROVIDER_QUALIFICATION_SCHEMA.to_string(),
            qualification_id: short_identity("provider-qualification", &digest),
            integrity_digest: digest,
            tenant_id: target.tenant_id.clone(),
            target_id: target.target_id.clone(),
            target_digest: target.integrity_digest.clone(),
            credential_revision,
            suite_id: suite_id.to_string(),
            run_id: evidence.run_id.clone(),
            qualified_at_unix_ms: evidence.completed_at_unix_ms,
            valid_until_unix_ms,
            evidence,
            capabilities,
            operator_principal_id: operator_principal_id.to_string(),
        };
        qualification.validate(target)?;
        Ok(qualification)
    }

    pub fn validate(&self, target: &ProviderTarget) -> Result<(), String> {
        if self.schema != PROVIDER_QUALIFICATION_SCHEMA
            && self.schema != PROVIDER_QUALIFICATION_SCHEMA_V1
        {
            return Err("unsupported_provider_qualification_schema".to_string());
        }
        target.validate()?;
        self.evidence.validate()?;
        if self.tenant_id != target.tenant_id
            || self.target_id != target.target_id
            || self.target_digest != target.integrity_digest
            || self.evidence.target_id != target.target_id
            || self.capabilities != derived_capabilities(&self.evidence, &self.schema)
            || self.capabilities.len() > MAX_PROVIDER_CAPABILITIES
            || (self.schema == PROVIDER_QUALIFICATION_SCHEMA_V1 && self.credential_revision != 0)
        {
            return Err("provider_qualification_integrity_mismatch".to_string());
        }
        let identity = QualificationIdentity {
            tenant_id: &self.tenant_id,
            target_id: &self.target_id,
            target_digest: &self.target_digest,
            suite_id: &self.suite_id,
            run_id: &self.run_id,
            qualified_at_unix_ms: self.qualified_at_unix_ms,
            valid_until_unix_ms: self.valid_until_unix_ms,
            evidence: &self.evidence,
            capabilities: &self.capabilities,
            operator_principal_id: &self.operator_principal_id,
        };
        let digest = if self.schema == PROVIDER_QUALIFICATION_SCHEMA_V1 {
            digest_of(&identity, "provider_qualification_identity")?
        } else {
            digest_of(
                &QualificationIdentityV2 {
                    schema: &self.schema,
                    credential_revision: self.credential_revision,
                    identity,
                },
                "provider_qualification_identity",
            )?
        };
        if self.integrity_digest != digest
            || self.qualification_id != short_identity("provider-qualification", &digest)
        {
            return Err("provider_qualification_digest_mismatch".to_string());
        }
        Ok(())
    }

    pub fn is_current(&self, now_unix_ms: u64) -> bool {
        self.valid_until_unix_ms
            .is_none_or(|until| now_unix_ms < until)
    }

    pub fn capability_at_least(
        &self,
        required: &ProviderCapability,
        minimum: &CapabilityProvenance,
    ) -> bool {
        self.capabilities.iter().any(|capability| {
            if &capability.capability != required {
                return false;
            }
            match minimum {
                CapabilityProvenance::Configured => matches!(
                    capability.provenance,
                    CapabilityProvenance::Configured
                        | CapabilityProvenance::Observed
                        | CapabilityProvenance::Qualified
                ),
                CapabilityProvenance::Observed => matches!(
                    capability.provenance,
                    CapabilityProvenance::Observed | CapabilityProvenance::Qualified
                ),
                CapabilityProvenance::Qualified => {
                    capability.provenance == CapabilityProvenance::Qualified
                }
                CapabilityProvenance::ExtensionAttested => {
                    *required == ProviderCapability::FirstPartyTelemetry
                        && capability.provenance == CapabilityProvenance::ExtensionAttested
                }
                CapabilityProvenance::ExtensionObserved => {
                    *required == ProviderCapability::ExtensionCompatibleTelemetry
                        && capability.provenance == CapabilityProvenance::ExtensionObserved
                }
            }
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderCredentialRevision {
    pub schema: String,
    pub revision_id: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub target_id: String,
    pub target_digest: String,
    pub sequence: u64,
    pub revision_label: String,
    pub principal_id: String,
    pub recorded_at_unix_ms: u64,
}

impl ProviderCredentialRevision {
    pub fn new(
        target: &ProviderTarget,
        sequence: u64,
        revision_label: &str,
        principal_id: &str,
        recorded_at_unix_ms: u64,
    ) -> Result<Self, String> {
        if sequence == 0 {
            return Err("provider_credential_revision_sequence_invalid".to_string());
        }
        require_identifier("provider_credential_revision_label", revision_label, 128)?;
        require_identifier("provider_credential_revision_principal", principal_id, 256)?;
        let material = (
            "yai.provider_credential_revision.v1",
            &target.tenant_id,
            &target.target_id,
            &target.integrity_digest,
            sequence,
            revision_label,
            principal_id,
            recorded_at_unix_ms,
        );
        let digest = digest_of(&material, "provider_credential_revision_identity")?;
        Ok(Self {
            schema: "yai.provider_credential_revision.v1".to_string(),
            revision_id: short_identity("provider-credential-revision", &digest),
            integrity_digest: digest,
            tenant_id: target.tenant_id.clone(),
            target_id: target.target_id.clone(),
            target_digest: target.integrity_digest.clone(),
            sequence,
            revision_label: revision_label.to_string(),
            principal_id: principal_id.to_string(),
            recorded_at_unix_ms,
        })
    }

    pub fn validate(&self, target: &ProviderTarget) -> Result<(), String> {
        let rebuilt = Self::new(
            target,
            self.sequence,
            &self.revision_label,
            &self.principal_id,
            self.recorded_at_unix_ms,
        )?;
        if rebuilt != *self {
            return Err("provider_credential_revision_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderTrustPosture {
    Unreviewed,
    Approved,
    Denied,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderTrustEvent {
    pub schema: String,
    pub event_id: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub target_id: String,
    pub target_digest: String,
    pub sequence: u64,
    pub posture: ProviderTrustPosture,
    pub principal_id: String,
    pub recorded_at_unix_ms: u64,
}

impl ProviderTrustEvent {
    pub fn new(
        target: &ProviderTarget,
        sequence: u64,
        posture: ProviderTrustPosture,
        principal_id: &str,
        recorded_at_unix_ms: u64,
    ) -> Result<Self, String> {
        if sequence == 0 || posture == ProviderTrustPosture::Unreviewed {
            return Err("provider_trust_event_transition_invalid".to_string());
        }
        let material = (
            &target.tenant_id,
            &target.target_id,
            &target.integrity_digest,
            sequence,
            &posture,
            principal_id,
            recorded_at_unix_ms,
        );
        let digest = digest_of(&material, "provider_trust_event_identity")?;
        Ok(Self {
            schema: PROVIDER_TRUST_EVENT_SCHEMA.to_string(),
            event_id: short_identity("provider-trust-event", &digest),
            integrity_digest: digest,
            tenant_id: target.tenant_id.clone(),
            target_id: target.target_id.clone(),
            target_digest: target.integrity_digest.clone(),
            sequence,
            posture,
            principal_id: principal_id.to_string(),
            recorded_at_unix_ms,
        })
    }

    pub fn validate(&self, target: &ProviderTarget) -> Result<(), String> {
        if self.schema != PROVIDER_TRUST_EVENT_SCHEMA
            || self.tenant_id != target.tenant_id
            || self.target_id != target.target_id
            || self.target_digest != target.integrity_digest
        {
            return Err("provider_trust_event_target_mismatch".to_string());
        }
        let rebuilt = Self::new(
            target,
            self.sequence,
            self.posture.clone(),
            &self.principal_id,
            self.recorded_at_unix_ms,
        )?;
        if rebuilt != *self {
            return Err("provider_trust_event_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderHealthPosture {
    Unknown,
    Healthy,
    Degraded,
    Unavailable,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCircuitPosture {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderHealthState {
    pub schema: String,
    #[serde(default)]
    pub integrity_digest: String,
    pub target_id: String,
    pub target_digest: String,
    pub posture: ProviderHealthPosture,
    pub circuit: ProviderCircuitPosture,
    pub consecutive_failures: u32,
    pub observed_at_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circuit_opened_at_unix_ms: Option<u64>,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<String>,
    #[serde(default)]
    pub effective_time_floor_unix_ms: u64,
    #[serde(default)]
    pub probe_epoch: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probe_owner: Option<ProviderProbeOwner>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderProbeOwner {
    pub boot_id: String,
    pub pid: u32,
    pub process_start_ticks: u64,
    pub token: String,
    pub started_at_unix_ms: u64,
}

#[derive(Serialize)]
struct ProviderHealthIdentity<'a> {
    schema: &'a str,
    target_id: &'a str,
    target_digest: &'a str,
    posture: &'a ProviderHealthPosture,
    circuit: &'a ProviderCircuitPosture,
    consecutive_failures: u32,
    observed_at_unix_ms: u64,
    circuit_opened_at_unix_ms: Option<u64>,
    source: &'a str,
    failure_class: &'a Option<String>,
    effective_time_floor_unix_ms: u64,
    probe_epoch: u64,
    probe_owner: &'a Option<ProviderProbeOwner>,
}

impl ProviderProbeOwner {
    pub fn capture(token: &str, started_at_unix_ms: u64) -> Result<Self, String> {
        require_identifier("provider_probe_owner_token", token, 256)?;
        let process = crate::resource_control::LocalProcessIdentity::capture(std::process::id())?;
        Ok(Self {
            boot_id: process.boot_id,
            pid: process.pid,
            process_start_ticks: process.start_ticks,
            token: token.to_string(),
            started_at_unix_ms,
        })
    }

    pub fn is_live(&self) -> bool {
        crate::resource_control::LocalProcessIdentity {
            schema: crate::resource_control::PROCESS_IDENTITY_SCHEMA.to_string(),
            pid: self.pid,
            boot_id: self.boot_id.clone(),
            start_ticks: self.process_start_ticks,
        }
        .is_live()
    }
}

impl ProviderHealthState {
    pub fn unknown(target: &ProviderTarget) -> Self {
        let mut state = Self {
            schema: PROVIDER_HEALTH_SCHEMA.to_string(),
            integrity_digest: String::new(),
            target_id: target.target_id.clone(),
            target_digest: target.integrity_digest.clone(),
            posture: ProviderHealthPosture::Unknown,
            circuit: ProviderCircuitPosture::Closed,
            consecutive_failures: 0,
            observed_at_unix_ms: 0,
            circuit_opened_at_unix_ms: None,
            source: "no_observation".to_string(),
            failure_class: None,
            effective_time_floor_unix_ms: 0,
            probe_epoch: 0,
            probe_owner: None,
        };
        state
            .reseal()
            .expect("provider health identity serialization is infallible");
        state
    }

    pub fn effective_posture(&self, now_unix_ms: u64) -> ProviderHealthPosture {
        let now_unix_ms = now_unix_ms.max(self.effective_time_floor_unix_ms);
        if self.observed_at_unix_ms == 0
            || now_unix_ms.saturating_sub(self.observed_at_unix_ms) > PROVIDER_HEALTH_FRESHNESS_MS
        {
            ProviderHealthPosture::Unknown
        } else {
            self.posture.clone()
        }
    }

    pub fn circuit_at(&self, now_unix_ms: u64) -> ProviderCircuitPosture {
        let now_unix_ms = now_unix_ms.max(self.effective_time_floor_unix_ms);
        if self.circuit == ProviderCircuitPosture::Open
            && self.circuit_opened_at_unix_ms.is_some_and(|opened| {
                now_unix_ms.saturating_sub(opened) >= PROVIDER_CIRCUIT_COOLDOWN_MS
            })
        {
            ProviderCircuitPosture::HalfOpen
        } else {
            self.circuit.clone()
        }
    }

    pub fn validate(&self, target: &ProviderTarget) -> Result<(), String> {
        if (self.schema != PROVIDER_HEALTH_SCHEMA && self.schema != PROVIDER_HEALTH_SCHEMA_V1)
            || self.target_id != target.target_id
            || self.target_digest != target.integrity_digest
            || self.source.len() > 128
            || self
                .failure_class
                .as_ref()
                .is_some_and(|value| value.len() > 128)
            || (self.schema == PROVIDER_HEALTH_SCHEMA_V1
                && (!self.integrity_digest.is_empty()
                    || self.effective_time_floor_unix_ms != 0
                    || self.probe_epoch != 0
                    || self.probe_owner.is_some()))
        {
            return Err("provider_health_integrity_invalid".to_string());
        }
        if let Some(owner) = &self.probe_owner {
            require_identifier("provider_probe_owner_boot_id", &owner.boot_id, 128)?;
            require_identifier("provider_probe_owner_token", &owner.token, 256)?;
            if owner.pid <= 1 || owner.process_start_ticks == 0 {
                return Err("provider_probe_owner_identity_invalid".to_string());
            }
        }
        if self.schema == PROVIDER_HEALTH_SCHEMA
            && self.integrity_digest != self.computed_digest()?
        {
            return Err("provider_health_integrity_invalid".to_string());
        }
        Ok(())
    }

    pub fn reseal(&mut self) -> Result<(), String> {
        if self.schema != PROVIDER_HEALTH_SCHEMA {
            return Err("provider_health_schema_invalid".to_string());
        }
        self.integrity_digest = self.computed_digest()?;
        Ok(())
    }

    fn computed_digest(&self) -> Result<String, String> {
        digest_of(
            &ProviderHealthIdentity {
                schema: &self.schema,
                target_id: &self.target_id,
                target_digest: &self.target_digest,
                posture: &self.posture,
                circuit: &self.circuit,
                consecutive_failures: self.consecutive_failures,
                observed_at_unix_ms: self.observed_at_unix_ms,
                circuit_opened_at_unix_ms: self.circuit_opened_at_unix_ms,
                source: &self.source,
                failure_class: &self.failure_class,
                effective_time_floor_unix_ms: self.effective_time_floor_unix_ms,
                probe_epoch: self.probe_epoch,
                probe_owner: &self.probe_owner,
            },
            "provider_health_identity",
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderFailoverPolicy {
    None,
    SafeOnly,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CaseProviderBinding {
    pub schema: String,
    pub binding_id: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub case_id: String,
    pub participant_id: String,
    pub ordered_target_ids: Vec<String>,
    pub failover_policy: ProviderFailoverPolicy,
    pub max_attempts_per_turn: u32,
    pub bound_by_principal_id: String,
    pub bound_at_generation: u64,
}

impl CaseProviderBinding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        case_id: &str,
        participant_id: &str,
        ordered_target_ids: Vec<String>,
        failover_policy: ProviderFailoverPolicy,
        max_attempts_per_turn: u32,
        bound_by_principal_id: &str,
        bound_at_generation: u64,
    ) -> Result<Self, String> {
        if ordered_target_ids.is_empty()
            || ordered_target_ids.len() > MAX_PROVIDER_TARGETS_PER_CASE
            || max_attempts_per_turn == 0
            || max_attempts_per_turn > MAX_PROVIDER_ATTEMPTS_PER_TURN
        {
            return Err("case_provider_binding_bounds_invalid".to_string());
        }
        let unique = ordered_target_ids.iter().collect::<BTreeSet<_>>();
        if unique.len() != ordered_target_ids.len() {
            return Err("case_provider_binding_duplicate_target".to_string());
        }
        let material = (
            tenant_id,
            case_id,
            participant_id,
            &ordered_target_ids,
            &failover_policy,
            max_attempts_per_turn,
            bound_by_principal_id,
            bound_at_generation,
        );
        let digest = digest_of(&material, "case_provider_binding_identity")?;
        Ok(Self {
            schema: CASE_PROVIDER_BINDING_SCHEMA.to_string(),
            binding_id: short_identity("case-provider-binding", &digest),
            integrity_digest: digest,
            tenant_id: tenant_id.to_string(),
            case_id: case_id.to_string(),
            participant_id: participant_id.to_string(),
            ordered_target_ids,
            failover_policy,
            max_attempts_per_turn,
            bound_by_principal_id: bound_by_principal_id.to_string(),
            bound_at_generation,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != CASE_PROVIDER_BINDING_SCHEMA {
            return Err("unsupported_case_provider_binding_schema".to_string());
        }
        let rebuilt = Self::new(
            &self.tenant_id,
            &self.case_id,
            &self.participant_id,
            self.ordered_target_ids.clone(),
            self.failover_policy.clone(),
            self.max_attempts_per_turn,
            &self.bound_by_principal_id,
            self.bound_at_generation,
        )?;
        if rebuilt != *self {
            return Err("case_provider_binding_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderCapabilityRequirement {
    pub capability: ProviderCapability,
    pub minimum_provenance: CapabilityProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderRequirement {
    pub schema: String,
    pub requirement_id: String,
    pub integrity_digest: String,
    pub purpose: String,
    pub capabilities: Vec<ProviderCapabilityRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_context_units: Option<u64>,
}

impl ProviderRequirement {
    pub fn text(purpose: &str) -> Result<Self, String> {
        Self::new(
            purpose,
            vec![ProviderCapabilityRequirement {
                capability: ProviderCapability::ChatText,
                minimum_provenance: CapabilityProvenance::Qualified,
            }],
            None,
        )
    }

    pub fn plan_patch() -> Result<Self, String> {
        Self::new(
            "workflow_plan_patch",
            vec![
                ProviderCapabilityRequirement {
                    capability: ProviderCapability::ChatText,
                    minimum_provenance: CapabilityProvenance::Qualified,
                },
                ProviderCapabilityRequirement {
                    capability: ProviderCapability::StructuredJsonObject,
                    minimum_provenance: CapabilityProvenance::Qualified,
                },
            ],
            None,
        )
    }

    pub fn new(
        purpose: &str,
        mut capabilities: Vec<ProviderCapabilityRequirement>,
        minimum_context_units: Option<u64>,
    ) -> Result<Self, String> {
        require_identifier("provider_requirement_purpose", purpose, 128)?;
        if capabilities.is_empty() || capabilities.len() > MAX_PROVIDER_CAPABILITIES {
            return Err("provider_requirement_capability_bounds_invalid".to_string());
        }
        capabilities.sort_by(|left, right| left.capability.cmp(&right.capability));
        capabilities.dedup_by(|left, right| left.capability == right.capability);
        let material = (purpose, &capabilities, minimum_context_units);
        let digest = digest_of(&material, "provider_requirement_identity")?;
        Ok(Self {
            schema: PROVIDER_REQUIREMENT_SCHEMA.to_string(),
            requirement_id: short_identity("provider-requirement", &digest),
            integrity_digest: digest,
            purpose: purpose.to_string(),
            capabilities,
            minimum_context_units,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != PROVIDER_REQUIREMENT_SCHEMA {
            return Err("unsupported_provider_requirement_schema".to_string());
        }
        let rebuilt = Self::new(
            &self.purpose,
            self.capabilities.clone(),
            self.minimum_context_units,
        )?;
        if rebuilt != *self {
            return Err("provider_requirement_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderExclusionCode {
    TargetMissing,
    TargetIntegrityInvalid,
    TenantMismatch,
    CredentialUnavailable,
    TrustNotApproved,
    QualificationMissing,
    QualificationInvalid,
    RequiredCapabilityMissing,
    ContextCapacityInsufficient,
    CircuitOpen,
    ProviderUnavailable,
    AlreadyAttempted,
    FailoverNotSafe,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderExclusion {
    pub target_id: String,
    pub code: ProviderExclusionCode,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ProviderCandidateSnapshot<'a> {
    pub target: Option<&'a ProviderTarget>,
    pub qualification: Option<&'a ProviderQualification>,
    pub trust: Option<&'a ProviderTrustEvent>,
    pub health: Option<&'a ProviderHealthState>,
    pub credential_available: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderSelection {
    pub schema: String,
    pub selection_id: String,
    pub integrity_digest: String,
    pub selector_version: String,
    pub tenant_id: String,
    pub case_id: String,
    pub case_generation: u64,
    pub participant_id: String,
    pub logical_turn_id: String,
    pub attempt_number: u32,
    pub requirement_id: String,
    pub binding_id: String,
    pub candidate_target_ids: Vec<String>,
    pub selected_target_id: String,
    pub selected_model_id: String,
    pub qualification_id: String,
    pub eligibility_reason: String,
    pub exclusions: Vec<ProviderExclusion>,
    pub selected_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSelectionPreview {
    pub selected_target_id: Option<String>,
    pub selected_model_id: Option<String>,
    pub qualification_id: Option<String>,
    pub exclusions: Vec<ProviderExclusion>,
}

fn exclusion(
    target_id: &str,
    code: ProviderExclusionCode,
    evidence: Vec<String>,
) -> ProviderExclusion {
    ProviderExclusion {
        target_id: target_id.to_string(),
        code,
        evidence_refs: evidence,
    }
}

pub fn select_provider(
    binding: &CaseProviderBinding,
    requirement: &ProviderRequirement,
    snapshots: &BTreeMap<String, ProviderCandidateSnapshot<'_>>,
    attempted_targets: &BTreeSet<String>,
    prior_attempt_retry_safe: bool,
    now_unix_ms: u64,
) -> ProviderSelectionPreview {
    let mut eligible = Vec::<(u8, usize, String, String, String)>::new();
    let mut exclusions = Vec::new();
    for (priority, target_id) in binding.ordered_target_ids.iter().enumerate() {
        let Some(snapshot) = snapshots.get(target_id) else {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::TargetMissing,
                vec![],
            ));
            continue;
        };
        let Some(target) = snapshot.target else {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::TargetMissing,
                vec![],
            ));
            continue;
        };
        if target.validate().is_err() {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::TargetIntegrityInvalid,
                vec![],
            ));
            continue;
        }
        if target.tenant_id != binding.tenant_id {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::TenantMismatch,
                vec![],
            ));
            continue;
        }
        if !snapshot.credential_available {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::CredentialUnavailable,
                vec![],
            ));
            continue;
        }
        let approved = snapshot.trust.is_some_and(|trust| {
            trust.tenant_id == binding.tenant_id
                && trust.target_id == *target_id
                && trust.target_digest == target.integrity_digest
                && trust.posture == ProviderTrustPosture::Approved
        });
        if !approved {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::TrustNotApproved,
                vec![],
            ));
            continue;
        }
        let Some(qualification) = snapshot.qualification else {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::QualificationMissing,
                vec![],
            ));
            continue;
        };
        if qualification.validate(target).is_err() || !qualification.is_current(now_unix_ms) {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::QualificationInvalid,
                vec![qualification.qualification_id.clone()],
            ));
            continue;
        }
        let capability_missing = requirement.capabilities.iter().any(|required| {
            !qualification.capability_at_least(&required.capability, &required.minimum_provenance)
        });
        if capability_missing {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::RequiredCapabilityMissing,
                vec![qualification.qualification_id.clone()],
            ));
            continue;
        }
        // W18 has no honest context-capacity evidence family. A caller that
        // explicitly requires a minimum therefore fails closed instead of
        // inferring capacity from a model name or silently ignoring it.
        if requirement.minimum_context_units.is_some() {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::ContextCapacityInsufficient,
                vec![qualification.qualification_id.clone()],
            ));
            continue;
        }
        if !attempted_targets.is_empty()
            && (binding.failover_policy != ProviderFailoverPolicy::SafeOnly
                || !prior_attempt_retry_safe)
        {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::FailoverNotSafe,
                vec![],
            ));
            continue;
        }
        if attempted_targets.contains(target_id) {
            let code = ProviderExclusionCode::AlreadyAttempted;
            exclusions.push(exclusion(target_id, code, vec![]));
            continue;
        }
        let health = snapshot
            .health
            .map(|health| health.effective_posture(now_unix_ms))
            .unwrap_or(ProviderHealthPosture::Unknown);
        let circuit = snapshot
            .health
            .map(|health| health.circuit_at(now_unix_ms))
            .unwrap_or(ProviderCircuitPosture::Closed);
        // Half-open is a probe-only posture. Ordinary Case work must not turn
        // a cooldown into an unbounded multi-process recovery stampede.
        if circuit != ProviderCircuitPosture::Closed {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::CircuitOpen,
                vec![],
            ));
            continue;
        }
        if health == ProviderHealthPosture::Unavailable {
            exclusions.push(exclusion(
                target_id,
                ProviderExclusionCode::ProviderUnavailable,
                vec![],
            ));
            continue;
        }
        let health_rank = match health {
            ProviderHealthPosture::Healthy => 0,
            ProviderHealthPosture::Unknown => 1,
            ProviderHealthPosture::Degraded => 2,
            ProviderHealthPosture::Unavailable => 3,
        };
        eligible.push((
            health_rank,
            priority,
            target.target_id.clone(),
            target.model_id.clone(),
            qualification.qualification_id.clone(),
        ));
    }
    eligible.sort();
    exclusions.sort_by(|left, right| left.target_id.cmp(&right.target_id));
    let selected = eligible.first();
    ProviderSelectionPreview {
        selected_target_id: selected.map(|value| value.2.clone()),
        selected_model_id: selected.map(|value| value.3.clone()),
        qualification_id: selected.map(|value| value.4.clone()),
        exclusions,
    }
}

impl ProviderSelection {
    #[allow(clippy::too_many_arguments)]
    pub fn from_preview(
        binding: &CaseProviderBinding,
        requirement: &ProviderRequirement,
        case_generation: u64,
        logical_turn_id: &str,
        attempt_number: u32,
        preview: ProviderSelectionPreview,
        selected_at_unix_ms: u64,
    ) -> Result<Self, String> {
        let selected_target_id = preview
            .selected_target_id
            .ok_or_else(|| "provider_selection_no_eligible_target".to_string())?;
        let selected_model_id = preview.selected_model_id.unwrap_or_default();
        let qualification_id = preview.qualification_id.unwrap_or_default();
        if attempt_number == 0 || attempt_number > binding.max_attempts_per_turn {
            return Err("provider_selection_attempt_out_of_bounds".to_string());
        }
        let material = (
            PROVIDER_SELECTOR_VERSION,
            &binding.tenant_id,
            &binding.case_id,
            case_generation,
            &binding.participant_id,
            logical_turn_id,
            attempt_number,
            &requirement.requirement_id,
            &binding.binding_id,
            &binding.ordered_target_ids,
            &selected_target_id,
            &selected_model_id,
            &qualification_id,
            &preview.exclusions,
            selected_at_unix_ms,
        );
        let digest = digest_of(&material, "provider_selection_identity")?;
        Ok(Self {
            schema: PROVIDER_SELECTION_SCHEMA.to_string(),
            selection_id: short_identity("provider-selection", &digest),
            integrity_digest: digest,
            selector_version: PROVIDER_SELECTOR_VERSION.to_string(),
            tenant_id: binding.tenant_id.clone(),
            case_id: binding.case_id.clone(),
            case_generation,
            participant_id: binding.participant_id.clone(),
            logical_turn_id: logical_turn_id.to_string(),
            attempt_number,
            requirement_id: requirement.requirement_id.clone(),
            binding_id: binding.binding_id.clone(),
            candidate_target_ids: binding.ordered_target_ids.clone(),
            selected_target_id,
            selected_model_id,
            qualification_id,
            eligibility_reason: "first_eligible_by_health_then_binding_order_then_target_id"
                .to_string(),
            exclusions: preview.exclusions,
            selected_at_unix_ms,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != PROVIDER_SELECTION_SCHEMA
            || !matches!(self.selector_version.as_str(), "yai.provider_selector.v1")
            || self.candidate_target_ids.is_empty()
            || self.candidate_target_ids.len() > MAX_PROVIDER_TARGETS_PER_CASE
            || self.exclusions.len() > MAX_PROVIDER_EXCLUSIONS
            || !self.candidate_target_ids.contains(&self.selected_target_id)
            || self.attempt_number == 0
            || self.attempt_number > MAX_PROVIDER_ATTEMPTS_PER_TURN
        {
            return Err("provider_selection_contract_invalid".to_string());
        }
        let material = (
            self.selector_version.as_str(),
            &self.tenant_id,
            &self.case_id,
            self.case_generation,
            &self.participant_id,
            &self.logical_turn_id,
            self.attempt_number,
            &self.requirement_id,
            &self.binding_id,
            &self.candidate_target_ids,
            &self.selected_target_id,
            &self.selected_model_id,
            &self.qualification_id,
            &self.exclusions,
            self.selected_at_unix_ms,
        );
        let digest = digest_of(&material, "provider_selection_identity")?;
        if self.integrity_digest != digest
            || self.selection_id != short_identity("provider-selection", &digest)
        {
            return Err("provider_selection_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderDeliveryClass {
    NotDispatched,
    DefinitivelyRejected,
    DeliveryIndeterminate,
    ResponseInvalid,
    ResultReceived,
    Cancelled,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderTransportStage {
    UrlParse,
    Connect,
    RequestSerialized,
    RequestWriting,
    ResponseHeaders,
    ResponseBody,
    JsonParse,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderAttemptOutcome {
    pub schema: String,
    pub outcome_id: String,
    pub integrity_digest: String,
    pub selection_id: String,
    pub target_id: String,
    pub logical_turn_id: String,
    pub attempt_number: u32,
    pub delivery: ProviderDeliveryClass,
    pub stage: ProviderTransportStage,
    pub request_bytes_written: u64,
    pub response_status: Option<u16>,
    pub no_execution_proven: bool,
    pub failure_class: Option<String>,
    pub recorded_at_unix_ms: u64,
}

impl ProviderAttemptOutcome {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        selection: &ProviderSelection,
        delivery: ProviderDeliveryClass,
        stage: ProviderTransportStage,
        request_bytes_written: u64,
        response_status: Option<u16>,
        no_execution_proven: bool,
        failure_class: Option<String>,
        recorded_at_unix_ms: u64,
    ) -> Result<Self, String> {
        let delivery_contract_valid = match delivery {
            ProviderDeliveryClass::NotDispatched | ProviderDeliveryClass::Cancelled => {
                request_bytes_written == 0 && response_status.is_none() && !no_execution_proven
            }
            ProviderDeliveryClass::DefinitivelyRejected => {
                request_bytes_written > 0 && response_status.is_some() && no_execution_proven
            }
            ProviderDeliveryClass::DeliveryIndeterminate => {
                request_bytes_written > 0 && !no_execution_proven
            }
            ProviderDeliveryClass::ResponseInvalid => {
                request_bytes_written > 0
                    && matches!(
                        stage,
                        ProviderTransportStage::ResponseHeaders
                            | ProviderTransportStage::ResponseBody
                            | ProviderTransportStage::JsonParse
                    )
                    && !no_execution_proven
            }
            ProviderDeliveryClass::ResultReceived => {
                request_bytes_written > 0
                    && response_status.is_some_and(|status| (200..300).contains(&status))
                    && stage == ProviderTransportStage::Completed
                    && !no_execution_proven
                    && failure_class.is_none()
            }
        };
        if selection.schema != PROVIDER_SELECTION_SCHEMA
            || !delivery_contract_valid
            || failure_class
                .as_ref()
                .is_some_and(|value| value.len() > 128)
        {
            return Err("provider_attempt_outcome_contract_invalid".to_string());
        }
        let material = (
            &selection.selection_id,
            &selection.selected_target_id,
            &selection.logical_turn_id,
            selection.attempt_number,
            &delivery,
            &stage,
            request_bytes_written,
            response_status,
            no_execution_proven,
            &failure_class,
            recorded_at_unix_ms,
        );
        let digest = digest_of(&material, "provider_attempt_outcome_identity")?;
        Ok(Self {
            schema: PROVIDER_ATTEMPT_OUTCOME_SCHEMA.to_string(),
            outcome_id: short_identity("provider-attempt-outcome", &digest),
            integrity_digest: digest,
            selection_id: selection.selection_id.clone(),
            target_id: selection.selected_target_id.clone(),
            logical_turn_id: selection.logical_turn_id.clone(),
            attempt_number: selection.attempt_number,
            delivery,
            stage,
            request_bytes_written,
            response_status,
            no_execution_proven,
            failure_class,
            recorded_at_unix_ms,
        })
    }

    pub fn validate(&self, selection: &ProviderSelection) -> Result<(), String> {
        let rebuilt = Self::new(
            selection,
            self.delivery.clone(),
            self.stage.clone(),
            self.request_bytes_written,
            self.response_status,
            self.no_execution_proven,
            self.failure_class.clone(),
            self.recorded_at_unix_ms,
        )?;
        if &rebuilt != self {
            return Err("provider_attempt_outcome_integrity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn retry_safe(&self) -> bool {
        self.delivery == ProviderDeliveryClass::NotDispatched
            || (self.delivery == ProviderDeliveryClass::DefinitivelyRejected
                && self.no_execution_proven)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(locality: ProviderLocality, endpoint: &str) -> ProviderTarget {
        ProviderTarget::from_input(ProviderTargetInput {
            tenant_id: "tenant:test".to_string(),
            provider_key: "fixture".to_string(),
            adapter: ProviderAdapterKind::OpenAiCompatible,
            endpoint: endpoint.to_string(),
            model_id: "model:test".to_string(),
            credential_ref: "none".to_string(),
            locality,
            extension_adapter_id: None,
            created_by_principal_id: "principal:test".to_string(),
            created_at_unix_ms: 1,
        })
        .unwrap()
    }

    fn qualification(target: &ProviderTarget, json: bool) -> ProviderQualification {
        ProviderQualification::from_evidence(
            target,
            ProviderProbeEvidence {
                run_id: "qualification-run:test".to_string(),
                target_id: target.target_id.clone(),
                started_at_unix_ms: 10,
                completed_at_unix_ms: 11,
                transport_connected: true,
                exact_model_addressed: true,
                chat_text_envelope_valid: true,
                structured_json_object_valid: json,
                usage_accounting_observed: true,
                health_endpoint_observed: false,
                extension_telemetry_observed: false,
                failure_codes: vec![],
            },
            "yai.openai_compatible.synthetic.v1",
            "principal:test",
            Some(1000),
        )
        .unwrap()
    }

    #[test]
    fn target_identity_is_tenant_and_configuration_bound() {
        let a = target(ProviderLocality::Loopback, "http://LOCALHOST:8080/v1/");
        let b = target(ProviderLocality::Loopback, "http://localhost:8080/v1");
        assert_eq!(a.target_id, b.target_id);
        let later = ProviderTarget::from_input(ProviderTargetInput {
            tenant_id: a.tenant_id.clone(),
            provider_key: a.provider_key.clone(),
            adapter: a.adapter.clone(),
            endpoint: a.endpoint.clone(),
            model_id: a.model_id.clone(),
            credential_ref: a.credential_ref.clone(),
            locality: a.locality.clone(),
            extension_adapter_id: a.extension_adapter_id.clone(),
            created_by_principal_id: a.created_by_principal_id.clone(),
            created_at_unix_ms: a.created_at_unix_ms + 1,
        })
        .unwrap();
        assert_eq!(a.target_id, later.target_id);
        assert_ne!(a.integrity_digest, later.integrity_digest);
        assert!(ProviderTarget::from_input(ProviderTargetInput {
            tenant_id: "tenant:test".to_string(),
            provider_key: "fixture".to_string(),
            adapter: ProviderAdapterKind::OpenAiCompatible,
            endpoint: "http://user:secret@example.test".to_string(),
            model_id: "model:test".to_string(),
            credential_ref: "none".to_string(),
            locality: ProviderLocality::Remote,
            extension_adapter_id: None,
            created_by_principal_id: "principal:test".to_string(),
            created_at_unix_ms: 1,
        })
        .is_err());
        assert!(
            normalize_provider_endpoint("http://example.test", &ProviderLocality::Remote).is_err()
        );
    }

    #[test]
    fn capability_is_derived_and_json_is_not_forgeable() {
        let target = target(ProviderLocality::Loopback, "http://127.0.0.1:8080");
        let qualification = qualification(&target, false);
        assert!(qualification.capability_at_least(
            &ProviderCapability::ChatText,
            &CapabilityProvenance::Qualified
        ));
        assert!(!qualification.capability_at_least(
            &ProviderCapability::StructuredJsonObject,
            &CapabilityProvenance::Qualified
        ));
        let mut forged = qualification.clone();
        forged.capabilities.push(ProviderCapabilityEvidence {
            capability: ProviderCapability::StructuredJsonObject,
            provenance: CapabilityProvenance::Qualified,
            evidence_refs: vec!["caller:boolean".to_string()],
            verified_minimum: None,
        });
        assert_eq!(
            forged.validate(&target).unwrap_err(),
            "provider_qualification_integrity_mismatch"
        );
    }

    #[test]
    fn historical_qualification_v1_remains_valid_without_extension_promotion() {
        let target = target(ProviderLocality::Loopback, "http://127.0.0.1:8080");
        let evidence = ProviderProbeEvidence {
            run_id: "qualification-run:historical-v1".to_string(),
            target_id: target.target_id.clone(),
            started_at_unix_ms: 10,
            completed_at_unix_ms: 11,
            transport_connected: true,
            exact_model_addressed: true,
            chat_text_envelope_valid: true,
            structured_json_object_valid: false,
            usage_accounting_observed: false,
            health_endpoint_observed: false,
            extension_telemetry_observed: true,
            failure_codes: vec![],
        };
        let capabilities = derived_capabilities(&evidence, PROVIDER_QUALIFICATION_SCHEMA_V1);
        let identity = QualificationIdentity {
            tenant_id: &target.tenant_id,
            target_id: &target.target_id,
            target_digest: &target.integrity_digest,
            suite_id: "yai.openai_compatible.synthetic.v1",
            run_id: &evidence.run_id,
            qualified_at_unix_ms: evidence.completed_at_unix_ms,
            valid_until_unix_ms: None,
            evidence: &evidence,
            capabilities: &capabilities,
            operator_principal_id: "principal:test",
        };
        let digest = digest_of(&identity, "provider_qualification_identity").unwrap();
        let qualification = ProviderQualification {
            schema: PROVIDER_QUALIFICATION_SCHEMA_V1.to_string(),
            qualification_id: short_identity("provider-qualification", &digest),
            integrity_digest: digest,
            tenant_id: target.tenant_id.clone(),
            target_id: target.target_id.clone(),
            target_digest: target.integrity_digest.clone(),
            credential_revision: 0,
            suite_id: "yai.openai_compatible.synthetic.v1".to_string(),
            run_id: evidence.run_id.clone(),
            qualified_at_unix_ms: evidence.completed_at_unix_ms,
            valid_until_unix_ms: None,
            evidence,
            capabilities,
            operator_principal_id: "principal:test".to_string(),
        };
        qualification.validate(&target).unwrap();
        assert!(!qualification.capability_at_least(
            &ProviderCapability::ChatText,
            &CapabilityProvenance::ExtensionAttested
        ));
    }

    #[test]
    fn selector_filters_then_uses_health_and_explicit_order() {
        let first = target(ProviderLocality::Loopback, "http://127.0.0.1:8001");
        let second = target(ProviderLocality::Loopback, "http://127.0.0.1:8002");
        let q1 = qualification(&first, true);
        let q2 = qualification(&second, true);
        let trust1 = ProviderTrustEvent::new(
            &first,
            1,
            ProviderTrustPosture::Approved,
            "principal:test",
            20,
        )
        .unwrap();
        let trust2 = ProviderTrustEvent::new(
            &second,
            1,
            ProviderTrustPosture::Approved,
            "principal:test",
            20,
        )
        .unwrap();
        let binding = CaseProviderBinding::new(
            "tenant:test",
            "case:test",
            "participant:model",
            vec![first.target_id.clone(), second.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            3,
            "principal:test",
            4,
        )
        .unwrap();
        let unknown = ProviderHealthState::unknown(&first);
        let mut healthy = ProviderHealthState::unknown(&second);
        healthy.posture = ProviderHealthPosture::Healthy;
        healthy.observed_at_unix_ms = 25;
        healthy.source = "probe".to_string();
        healthy.reseal().unwrap();
        let snapshots = BTreeMap::from([
            (
                first.target_id.clone(),
                ProviderCandidateSnapshot {
                    target: Some(&first),
                    qualification: Some(&q1),
                    trust: Some(&trust1),
                    health: Some(&unknown),
                    credential_available: true,
                },
            ),
            (
                second.target_id.clone(),
                ProviderCandidateSnapshot {
                    target: Some(&second),
                    qualification: Some(&q2),
                    trust: Some(&trust2),
                    health: Some(&healthy),
                    credential_available: true,
                },
            ),
        ]);
        let preview = select_provider(
            &binding,
            &ProviderRequirement::plan_patch().unwrap(),
            &snapshots,
            &BTreeSet::new(),
            true,
            30,
        );
        assert_eq!(
            preview.selected_target_id.as_deref(),
            Some(second.target_id.as_str())
        );
    }

    #[test]
    fn indeterminate_delivery_is_never_retry_safe() {
        let outcome = ProviderAttemptOutcome {
            schema: PROVIDER_ATTEMPT_OUTCOME_SCHEMA.to_string(),
            outcome_id: "outcome:test".to_string(),
            integrity_digest: "sha256:test".to_string(),
            selection_id: "selection:test".to_string(),
            target_id: "target:test".to_string(),
            logical_turn_id: "turn:test".to_string(),
            attempt_number: 1,
            delivery: ProviderDeliveryClass::DeliveryIndeterminate,
            stage: ProviderTransportStage::ResponseBody,
            request_bytes_written: 200,
            response_status: None,
            no_execution_proven: false,
            failure_class: Some("connection_reset".to_string()),
            recorded_at_unix_ms: 1,
        };
        assert!(!outcome.retry_safe());
    }
}
