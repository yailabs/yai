//! Local authenticated Principal projection and Tenant security-domain contracts.
//!
//! This module owns identity and isolation algebra for the current local POSIX
//! product. It does not own Case roles, policy evaluation, credentials, SSO,
//! billing, provider identity, or long-lived login sessions.

use crate::effect::digest_bytes;
use serde::{Deserialize, Serialize};

pub const SECURITY_PRINCIPAL_SCHEMA: &str = "yai.security_principal.v1";
pub const TENANT_SCHEMA: &str = "yai.tenant.v1";
pub const SECURITY_EVENT_SCHEMA: &str = "yai.security_event.v1";
pub const LOCAL_POSIX_AUTHN_METHOD: &str = "local_posix_effective_credential";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PosixAuthenticationBinding {
    pub binding_ref: String,
    pub real_uid: u32,
    pub effective_uid: u32,
    pub real_gid: u32,
    pub effective_gid: u32,
}

impl PosixAuthenticationBinding {
    pub fn validate(&self) -> Result<(), String> {
        let expected = format!("posix:euid:{}", self.effective_uid);
        if self.binding_ref != expected {
            return Err("posix_authentication_binding_mismatch".to_string());
        }
        Ok(())
    }
}

/// Invocation-scoped, kernel-observed identity. Fields are private so callers
/// cannot construct authority from a Principal string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedPrincipal {
    authentication_method: &'static str,
    binding: PosixAuthenticationBinding,
}

impl AuthenticatedPrincipal {
    pub fn authenticate_local() -> Result<Self, String> {
        #[cfg(unix)]
        {
            unsafe extern "C" {
                fn getuid() -> u32;
                fn geteuid() -> u32;
                fn getgid() -> u32;
                fn getegid() -> u32;
            }
            // SAFETY: these POSIX accessors take no pointers and only return
            // credentials of the current process.
            let (real_uid, effective_uid, real_gid, effective_gid) =
                unsafe { (getuid(), geteuid(), getgid(), getegid()) };
            let binding = PosixAuthenticationBinding {
                binding_ref: format!("posix:euid:{effective_uid}"),
                real_uid,
                effective_uid,
                real_gid,
                effective_gid,
            };
            binding.validate()?;
            Ok(Self {
                authentication_method: LOCAL_POSIX_AUTHN_METHOD,
                binding,
            })
        }
        #[cfg(not(unix))]
        {
            Err("local_posix_authentication_unsupported".to_string())
        }
    }

    pub fn authentication_method(&self) -> &str {
        self.authentication_method
    }

    pub fn binding(&self) -> &PosixAuthenticationBinding {
        &self.binding
    }

    pub fn projected_principal_id(&self) -> String {
        principal_identity(self.authentication_method, &self.binding)
    }

    #[cfg(test)]
    pub(crate) fn for_test(effective_uid: u32) -> Self {
        Self::for_test_credentials(effective_uid, effective_uid, effective_uid, effective_uid)
    }

    #[cfg(test)]
    pub(crate) fn for_test_credentials(
        real_uid: u32,
        effective_uid: u32,
        real_gid: u32,
        effective_gid: u32,
    ) -> Self {
        Self {
            authentication_method: LOCAL_POSIX_AUTHN_METHOD,
            binding: PosixAuthenticationBinding {
                binding_ref: format!("posix:euid:{effective_uid}"),
                real_uid,
                effective_uid,
                real_gid,
                effective_gid,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityPrincipal {
    pub schema: String,
    pub principal_id: String,
    pub authentication_method: String,
    pub authentication_binding: PosixAuthenticationBinding,
    pub created_at_unix_ms: u64,
    pub integrity_digest: String,
}

impl SecurityPrincipal {
    pub fn from_authenticated(
        authenticated: &AuthenticatedPrincipal,
        created_at_unix_ms: u64,
    ) -> Result<Self, String> {
        let principal_id = authenticated.projected_principal_id();
        let mut result = Self {
            schema: SECURITY_PRINCIPAL_SCHEMA.to_string(),
            principal_id,
            authentication_method: authenticated.authentication_method().to_string(),
            authentication_binding: authenticated.binding().clone(),
            created_at_unix_ms,
            integrity_digest: String::new(),
        };
        result.integrity_digest = result.expected_digest()?;
        result.validate()?;
        Ok(result)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != SECURITY_PRINCIPAL_SCHEMA
            || self.authentication_method != LOCAL_POSIX_AUTHN_METHOD
        {
            return Err("unsupported_security_principal_contract".to_string());
        }
        self.authentication_binding.validate()?;
        if self.principal_id
            != principal_identity(&self.authentication_method, &self.authentication_binding)
            || self.integrity_digest != self.expected_digest()?
        {
            return Err("security_principal_integrity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn matches_authenticated(&self, authenticated: &AuthenticatedPrincipal) -> bool {
        self.authentication_method == authenticated.authentication_method()
            && self.authentication_binding.binding_ref == authenticated.binding().binding_ref
            && self.authentication_binding.effective_uid == authenticated.binding().effective_uid
            && self.principal_id == authenticated.projected_principal_id()
    }

    fn expected_digest(&self) -> Result<String, String> {
        digest_json(&serde_json::json!({
            "schema": self.schema,
            "principal_id": self.principal_id,
            "authentication_method": self.authentication_method,
            "authentication_binding": self.authentication_binding,
            "created_at_unix_ms": self.created_at_unix_ms,
        }))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tenant {
    pub schema: String,
    pub tenant_id: String,
    pub organization_ref: String,
    pub owner_principal_id: String,
    pub created_at_unix_ms: u64,
    pub integrity_digest: String,
}

impl Tenant {
    pub fn new(
        tenant_id: &str,
        organization_ref: &str,
        owner_principal_id: &str,
        created_at_unix_ms: u64,
    ) -> Result<Self, String> {
        validate_security_id("tenant_id", tenant_id, "tenant:")?;
        validate_security_id("organization_ref", organization_ref, "organization:")?;
        validate_security_id("owner_principal_id", owner_principal_id, "principal:")?;
        let mut result = Self {
            schema: TENANT_SCHEMA.to_string(),
            tenant_id: tenant_id.to_string(),
            organization_ref: organization_ref.to_string(),
            owner_principal_id: owner_principal_id.to_string(),
            created_at_unix_ms,
            integrity_digest: String::new(),
        };
        result.integrity_digest = result.expected_digest()?;
        result.validate()?;
        Ok(result)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != TENANT_SCHEMA {
            return Err("unsupported_tenant_schema".to_string());
        }
        validate_security_id("tenant_id", &self.tenant_id, "tenant:")?;
        validate_security_id("organization_ref", &self.organization_ref, "organization:")?;
        validate_security_id("owner_principal_id", &self.owner_principal_id, "principal:")?;
        if self.integrity_digest != self.expected_digest()? {
            return Err("tenant_integrity_mismatch".to_string());
        }
        Ok(())
    }

    fn expected_digest(&self) -> Result<String, String> {
        digest_json(&serde_json::json!({
            "schema": self.schema,
            "tenant_id": self.tenant_id,
            "organization_ref": self.organization_ref,
            "owner_principal_id": self.owner_principal_id,
            "created_at_unix_ms": self.created_at_unix_ms,
        }))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantMembershipKind {
    Owner,
    Member,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityEventAction {
    LocalPrincipalRegistered,
    TenantCreated,
    TenantMemberAdded,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityEvent {
    pub schema: String,
    pub event_id: String,
    pub sequence: u64,
    pub action: SecurityEventAction,
    pub principal_id: String,
    pub tenant_id: Option<String>,
    pub subject_principal_id: Option<String>,
    pub membership: Option<TenantMembershipKind>,
    pub committed_at_unix_ms: u64,
    pub reason: String,
    pub integrity_digest: String,
}

impl SecurityEvent {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != SECURITY_EVENT_SCHEMA
            || self.sequence == 0
            || self.reason.trim().is_empty()
        {
            return Err("security_event_contract_invalid".to_string());
        }
        validate_security_id("principal_id", &self.principal_id, "principal:")?;
        if let Some(tenant_id) = &self.tenant_id {
            validate_security_id("tenant_id", tenant_id, "tenant:")?;
        }
        if let Some(subject) = &self.subject_principal_id {
            validate_security_id("subject_principal_id", subject, "principal:")?;
        }
        let digest = self.expected_digest()?;
        if self.integrity_digest != digest
            || self.event_id != format!("security-event:{}", digest_prefix(&digest))
        {
            return Err("security_event_integrity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn seal(mut self) -> Result<Self, String> {
        self.integrity_digest = self.expected_digest()?;
        self.event_id = format!("security-event:{}", digest_prefix(&self.integrity_digest));
        self.validate()?;
        Ok(self)
    }

    fn expected_digest(&self) -> Result<String, String> {
        digest_json(&serde_json::json!({
            "schema": self.schema,
            "sequence": self.sequence,
            "action": self.action,
            "principal_id": self.principal_id,
            "tenant_id": self.tenant_id,
            "subject_principal_id": self.subject_principal_id,
            "membership": self.membership,
            "committed_at_unix_ms": self.committed_at_unix_ms,
            "reason": self.reason,
        }))
    }
}

/// Ephemeral result of authenticating one invocation and resolving exactly one
/// Tenant membership. Private fields prevent string construction by callers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityContext {
    principal_id: String,
    tenant_id: String,
    membership: TenantMembershipKind,
    authentication_binding_ref: String,
}

impl SecurityContext {
    pub(crate) fn resolved(
        principal_id: String,
        tenant_id: String,
        membership: TenantMembershipKind,
        authentication_binding_ref: String,
    ) -> Self {
        Self {
            principal_id,
            tenant_id,
            membership,
            authentication_binding_ref,
        }
    }

    pub fn principal_id(&self) -> &str {
        &self.principal_id
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn membership(&self) -> &TenantMembershipKind {
        &self.membership
    }

    pub fn authentication_binding_ref(&self) -> &str {
        &self.authentication_binding_ref
    }

    pub fn require_owner(&self) -> Result<(), String> {
        if self.membership == TenantMembershipKind::Owner {
            Ok(())
        } else {
            Err("tenant_owner_required".to_string())
        }
    }
}

fn principal_identity(method: &str, binding: &PosixAuthenticationBinding) -> String {
    let digest = digest_bytes(
        serde_json::json!({
            "method": method,
            "binding_ref": binding.binding_ref,
            "effective_uid": binding.effective_uid
        })
        .to_string()
        .as_bytes(),
    );
    format!("principal:{}", digest_prefix(&digest))
}

fn digest_prefix(digest: &str) -> &str {
    let value = digest.strip_prefix("sha256:").unwrap_or(digest);
    &value[..value.len().min(32)]
}

fn digest_json(value: &serde_json::Value) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|encoded| digest_bytes(&encoded))
        .map_err(|error| format!("security_integrity_encode_failed: {error}"))
}

pub fn validate_security_id(name: &str, value: &str, prefix: &str) -> Result<(), String> {
    if value.len() <= prefix.len()
        || value.len() > 160
        || !value.starts_with(prefix)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '.' | '_' | '-'))
    {
        return Err(format!("{name}_invalid"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posix_projection_is_stable_and_separate_from_roles() {
        let authenticated = AuthenticatedPrincipal::for_test(1001);
        let first = SecurityPrincipal::from_authenticated(&authenticated, 7).unwrap();
        let second = SecurityPrincipal::from_authenticated(&authenticated, 7).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.authentication_binding.effective_uid, 1001);
        assert!(!serde_json::to_string(&first).unwrap().contains("role"));

        let sudo_posture = AuthenticatedPrincipal::for_test_credentials(1001, 0, 1001, 0);
        let direct_root = AuthenticatedPrincipal::for_test_credentials(0, 0, 0, 0);
        assert_eq!(
            sudo_posture.projected_principal_id(),
            direct_root.projected_principal_id()
        );
        let enrolled_root = SecurityPrincipal::from_authenticated(&sudo_posture, 8).unwrap();
        assert!(enrolled_root.matches_authenticated(&direct_root));
        assert_ne!(
            enrolled_root.authentication_binding.real_uid,
            enrolled_root.authentication_binding.effective_uid
        );
    }
}
