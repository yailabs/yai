//! Tenant-owned, explicitly pinned machine identity. A machine is not a
//! provider target, a Case Resource, or evidence of a running model.

use base64::Engine as _;
use serde::{Deserialize, Serialize};

use crate::effect::digest_bytes;

pub const MACHINE_ASSET_SCHEMA: &str = "yai.machine_asset.v1";
pub const MACHINE_REVOCATION_SCHEMA: &str = "yai.machine_revocation.v1";
pub const MAX_MACHINES_PER_TENANT: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MachineAssetInput {
    pub tenant_id: String,
    pub address: String,
    pub port: u16,
    pub management_user: String,
    /// Exact independently approved OpenSSH Ed25519 public key, without a comment.
    pub host_public_key: String,
    /// Operator evidence reference; its presence is not network authentication.
    pub approval_ref: String,
    pub approved_by_principal_id: String,
    pub approved_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MachineAsset {
    pub schema: String,
    pub asset_id: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub device_identity: String,
    pub address: String,
    pub port: u16,
    pub management_user: String,
    pub host_public_key: String,
    pub approval_ref: String,
    pub approved_by_principal_id: String,
    pub approved_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MachineRevocation {
    pub schema: String,
    pub asset_id: String,
    pub tenant_id: String,
    pub device_identity: String,
    pub revoked_by_principal_id: String,
    pub revoked_at_unix_ms: u64,
    pub reason: String,
    pub integrity_digest: String,
}

impl MachineRevocation {
    pub fn new(asset: &MachineAsset, principal_id: &str, at: u64, reason: &str) -> Result<Self, String> {
        if !identifier(principal_id, 128) || reason.is_empty() || reason.len() > 256 || reason.chars().any(char::is_control) {
            return Err("machine_revocation_input_invalid".into());
        }
        let mut result = Self {
            schema: MACHINE_REVOCATION_SCHEMA.into(),
            asset_id: asset.asset_id.clone(),
            tenant_id: asset.tenant_id.clone(),
            device_identity: asset.device_identity.clone(),
            revoked_by_principal_id: principal_id.into(),
            revoked_at_unix_ms: at,
            reason: reason.into(),
            integrity_digest: String::new(),
        };
        result.integrity_digest = result.digest()?;
        Ok(result)
    }

    fn digest(&self) -> Result<String, String> {
        let mut value = self.clone();
        value.integrity_digest.clear();
        serde_json::to_vec(&value)
            .map(|bytes| digest_bytes(&bytes))
            .map_err(|error| format!("machine_revocation_encode_failed:{error}"))
    }

    pub fn validate(&self, asset: &MachineAsset) -> Result<(), String> {
        if &Self::new(asset, &self.revoked_by_principal_id, self.revoked_at_unix_ms, &self.reason)? != self {
            return Err("machine_revocation_integrity_invalid".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MachineAssetView {
    pub registration: MachineAsset,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation: Option<MachineRevocation>,
}

fn identifier(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-/".contains(&byte))
}

fn ssh_string<'a>(bytes: &'a [u8], cursor: &mut usize) -> Result<&'a [u8], String> {
    let size = bytes
        .get(*cursor..*cursor + 4)
        .ok_or("machine_host_key_wire_invalid")?;
    let size = u32::from_be_bytes(size.try_into().map_err(|_| "machine_host_key_wire_invalid")?) as usize;
    *cursor += 4;
    let value = bytes
        .get(*cursor..*cursor + size)
        .ok_or("machine_host_key_wire_invalid")?;
    *cursor += size;
    Ok(value)
}

/// The YVEX remote bootstrap identifies a device by SHA-256 of the exact SSH
/// public-key blob, not by hostname, endpoint or self-reported JSON identity.
pub fn device_identity(host_public_key: &str) -> Result<String, String> {
    let mut parts = host_public_key.split_ascii_whitespace();
    if parts.next() != Some("ssh-ed25519") {
        return Err("machine_host_key_algorithm_invalid".into());
    }
    let encoded = parts.next().ok_or("machine_host_key_missing")?;
    if parts.next().is_some() || encoded.len() > 256 {
        return Err("machine_host_key_format_invalid".into());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| "machine_host_key_base64_invalid")?;
    let mut cursor = 0;
    if ssh_string(&bytes, &mut cursor)? != b"ssh-ed25519"
        || ssh_string(&bytes, &mut cursor)?.len() != 32
        || cursor != bytes.len()
    {
        return Err("machine_host_key_wire_invalid".into());
    }
    Ok(format!("ssh-ed25519:{}", digest_bytes(&bytes)))
}

impl MachineAsset {
    pub fn same_registration(&self, other: &Self) -> bool {
        self.tenant_id == other.tenant_id
            && self.device_identity == other.device_identity
            && self.address == other.address
            && self.port == other.port
            && self.management_user == other.management_user
            && self.host_public_key == other.host_public_key
            && self.approval_ref == other.approval_ref
            && self.approved_by_principal_id == other.approved_by_principal_id
    }

    pub fn from_input(input: MachineAssetInput) -> Result<Self, String> {
        if !identifier(&input.tenant_id, 128)
            || !identifier(&input.approval_ref, 256)
            || !identifier(&input.approved_by_principal_id, 128)
        {
            return Err("machine_registration_identity_invalid".into());
        }
        if input.address.is_empty()
            || input.address.len() > 253
            || input.address.starts_with('-')
            || !input
                .address
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b".:-".contains(&byte))
            || input.port == 0
            || input.management_user.is_empty()
            || input.management_user.len() > 64
            || !input.management_user.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || b"._-".contains(&byte)
            })
            || input.management_user.starts_with('-')
        {
            return Err("machine_endpoint_invalid".into());
        }
        let device_identity = device_identity(&input.host_public_key)?;
        let encoded = input.host_public_key.split_ascii_whitespace().nth(1)
            .ok_or("machine_host_key_missing")?;
        let host_public_key = format!("ssh-ed25519 {encoded}");
        let asset_id = format!("machine:{}:{}", input.tenant_id, device_identity);
        let mut asset = Self {
            schema: MACHINE_ASSET_SCHEMA.into(),
            asset_id,
            integrity_digest: String::new(),
            tenant_id: input.tenant_id,
            device_identity,
            address: input.address,
            port: input.port,
            management_user: input.management_user,
            host_public_key,
            approval_ref: input.approval_ref,
            approved_by_principal_id: input.approved_by_principal_id,
            approved_at_unix_ms: input.approved_at_unix_ms,
        };
        asset.integrity_digest = asset.digest()?;
        Ok(asset)
    }

    fn digest(&self) -> Result<String, String> {
        let mut value = self.clone();
        value.integrity_digest.clear();
        serde_json::to_vec(&value)
            .map(|bytes| digest_bytes(&bytes))
            .map_err(|error| format!("machine_asset_encode_failed:{error}"))
    }

    pub fn validate(&self) -> Result<(), String> {
        let rebuilt = Self::from_input(MachineAssetInput {
            tenant_id: self.tenant_id.clone(),
            address: self.address.clone(),
            port: self.port,
            management_user: self.management_user.clone(),
            host_public_key: self.host_public_key.clone(),
            approval_ref: self.approval_ref.clone(),
            approved_by_principal_id: self.approved_by_principal_id.clone(),
            approved_at_unix_ms: self.approved_at_unix_ms,
        })?;
        if &rebuilt != self {
            return Err("machine_asset_integrity_invalid".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 8032 Ed25519 public key encoded as an OpenSSH wire blob.
    const KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJ0zGAI5z9NlKS5al2atSGTQS7HfiuVQoQGaGe4HWlp4";

    #[test]
    fn exact_host_key_identity_and_integrity() {
        let identity = device_identity(KEY).unwrap();
        assert_eq!(identity,
            "ssh-ed25519:sha256:c8f904c18de5f91852f4b195ac194738bc7b861864967fb88b5a0091848f1a54");
        let asset = MachineAsset::from_input(MachineAssetInput {
            tenant_id: "tenant:test".into(),
            address: "dgx.example.internal".into(),
            port: 2222,
            management_user: "yvex".into(),
            host_public_key: KEY.into(),
            approval_ref: "approval:out-of-band-1".into(),
            approved_by_principal_id: "principal:test".into(),
            approved_at_unix_ms: 1,
        }).unwrap();
        asset.validate().unwrap();
        let mut tampered = asset.clone();
        tampered.address = "wrong.example.internal".into();
        assert_eq!(tampered.validate().unwrap_err(), "machine_asset_integrity_invalid");
    }

    #[test]
    fn untrusted_or_malformed_key_and_endpoint_refused() {
        assert!(device_identity("ssh-rsa AAAA").is_err());
        assert!(device_identity("ssh-ed25519 AAAA comment").is_err());
        assert!(device_identity("ssh-ed25519 AAAA").is_err());
        let input = MachineAssetInput {
            tenant_id: "tenant:test".into(),
            address: "-oProxyCommand=bad".into(),
            port: 2222,
            management_user: "yvex".into(),
            host_public_key: KEY.into(),
            approval_ref: "approval:out-of-band-1".into(),
            approved_by_principal_id: "principal:test".into(),
            approved_at_unix_ms: 1,
        };
        assert_eq!(MachineAsset::from_input(input).unwrap_err(), "machine_endpoint_invalid");
    }
}
