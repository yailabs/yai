//! Shared physical-resource authority and carrier fencing.
//!
//! A resource epoch is independent from Case, Grant and RuntimeInstance
//! generations.  The immutable Case ledger records the fence acquired at
//! PREPARE; this owner records the current cross-Case writer and its history.
//! A fence is evidence, not a bearer capability: carriers must re-resolve it
//! through the canonical resource-control store immediately before mutation.

use crate::effect::digest_bytes;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const RESOURCE_CONTROL_STATE_SCHEMA_V1: &str = "yai.resource_control_state.v1";
pub const RESOURCE_CONTROL_STATE_SCHEMA: &str = "yai.resource_control_state.v2";
pub const RESOURCE_CONTROL_EVENT_SCHEMA_V1: &str = "yai.resource_control_event.v1";
pub const RESOURCE_CONTROL_EVENT_SCHEMA: &str = "yai.resource_control_event.v2";
pub const RESOURCE_FENCE_SCHEMA: &str = "yai.resource_fence.v1";
pub const PROCESS_IDENTITY_SCHEMA: &str = "yai.local_process_identity.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlledResourceKind {
    Filesystem,
    Process,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceIdentity {
    pub resource_id: String,
    pub tenant_id: String,
    pub resource_kind: ControlledResourceKind,
    /// Canonical local identity. Filesystem values are absolute canonical
    /// paths; process values are exact boot/start/PID identities.
    pub canonical_identity: String,
}

impl ResourceIdentity {
    pub fn filesystem(tenant_id: &str, canonical_root: &str) -> Result<Self, String> {
        if !tenant_id.starts_with("tenant:") || !Path::new(canonical_root).is_absolute() {
            return Err("invalid_filesystem_resource_identity".to_string());
        }
        let material = format!("{tenant_id}|filesystem|{canonical_root}");
        Ok(Self {
            resource_id: format!(
                "resource-control:{}",
                &digest_bytes(material.as_bytes())[..32]
            ),
            tenant_id: tenant_id.to_string(),
            resource_kind: ControlledResourceKind::Filesystem,
            canonical_identity: canonical_root.to_string(),
        })
    }

    pub fn process(tenant_id: &str, process: &LocalProcessIdentity) -> Result<Self, String> {
        process.validate()?;
        if !tenant_id.starts_with("tenant:") {
            return Err("invalid_process_resource_tenant".to_string());
        }
        let identity = process.canonical_identity();
        let material = format!("{tenant_id}|process|{identity}");
        Ok(Self {
            resource_id: format!(
                "resource-control:{}",
                &digest_bytes(material.as_bytes())[..32]
            ),
            tenant_id: tenant_id.to_string(),
            resource_kind: ControlledResourceKind::Process,
            canonical_identity: identity,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.resource_id.is_empty()
            || !self.tenant_id.starts_with("tenant:")
            || self.canonical_identity.is_empty()
        {
            return Err("invalid_resource_identity".to_string());
        }
        let expected = match self.resource_kind {
            ControlledResourceKind::Filesystem => {
                if !Path::new(&self.canonical_identity).is_absolute() {
                    return Err("filesystem_resource_identity_not_absolute".to_string());
                }
                Self::filesystem(&self.tenant_id, &self.canonical_identity)?
            }
            ControlledResourceKind::Process => {
                let material = format!("{}|process|{}", self.tenant_id, self.canonical_identity);
                Self {
                    resource_id: format!(
                        "resource-control:{}",
                        &digest_bytes(material.as_bytes())[..32]
                    ),
                    tenant_id: self.tenant_id.clone(),
                    resource_kind: ControlledResourceKind::Process,
                    canonical_identity: self.canonical_identity.clone(),
                }
            }
        };
        if expected.resource_id != self.resource_id {
            return Err("resource_identity_digest_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalProcessIdentity {
    pub schema: String,
    pub pid: u32,
    pub boot_id: String,
    pub start_ticks: u64,
}

impl LocalProcessIdentity {
    pub fn capture(pid: u32) -> Result<Self, String> {
        if pid <= 1 {
            return Err("process_identity_pid_forbidden".to_string());
        }
        #[cfg(target_os = "linux")]
        {
            let stat = fs::read_to_string(format!("/proc/{pid}/stat"))
                .map_err(|error| format!("process_identity_stat_unavailable: {error}"))?;
            let close = stat
                .rfind(')')
                .ok_or_else(|| "process_identity_stat_malformed".to_string())?;
            // Fields after comm begin at field 3; starttime is field 22 and
            // therefore index 19 in this suffix.
            let fields = stat[close + 1..].split_whitespace().collect::<Vec<_>>();
            let start_ticks = fields
                .get(19)
                .ok_or_else(|| "process_identity_starttime_missing".to_string())?
                .parse::<u64>()
                .map_err(|error| format!("process_identity_starttime_invalid: {error}"))?;
            let boot_id = fs::read_to_string("/proc/sys/kernel/random/boot_id")
                .map_err(|error| format!("process_identity_boot_id_unavailable: {error}"))?
                .trim()
                .to_string();
            let value = Self {
                schema: PROCESS_IDENTITY_SCHEMA.to_string(),
                pid,
                boot_id,
                start_ticks,
            };
            value.validate()?;
            Ok(value)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = pid;
            Err("process_identity_unsupported_platform".to_string())
        }
    }

    pub fn canonical_identity(&self) -> String {
        format!(
            "linux-process-v1:{}:{}:{}",
            self.boot_id, self.pid, self.start_ticks
        )
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != PROCESS_IDENTITY_SCHEMA
            || self.pid <= 1
            || self.boot_id.trim().is_empty()
            || self.start_ticks == 0
        {
            return Err("invalid_local_process_identity".to_string());
        }
        Ok(())
    }

    pub fn is_live(&self) -> bool {
        Self::capture(self.pid).is_ok_and(|current| current == *self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceFence {
    pub schema: String,
    pub fence_id: String,
    pub integrity_digest: String,
    pub resource_id: String,
    pub tenant_id: String,
    pub resource_kind: ControlledResourceKind,
    pub resource_epoch: u64,
    pub case_id: String,
    pub operation_id: String,
    pub grant_id: String,
    pub effect_id: String,
    pub owner_pid: u32,
    pub owner_process_identity: String,
    pub issued_at_unix_ms: u64,
}

impl ResourceFence {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn issue(
        identity: &ResourceIdentity,
        resource_epoch: u64,
        case_id: &str,
        operation_id: &str,
        grant_id: &str,
        effect_id: &str,
        owner_pid: u32,
        owner_process_identity: &str,
        issued_at_unix_ms: u64,
    ) -> Result<Self, String> {
        identity.validate()?;
        let mut fence = Self {
            schema: RESOURCE_FENCE_SCHEMA.to_string(),
            fence_id: String::new(),
            integrity_digest: String::new(),
            resource_id: identity.resource_id.clone(),
            tenant_id: identity.tenant_id.clone(),
            resource_kind: identity.resource_kind.clone(),
            resource_epoch,
            case_id: case_id.to_string(),
            operation_id: operation_id.to_string(),
            grant_id: grant_id.to_string(),
            effect_id: effect_id.to_string(),
            owner_pid,
            owner_process_identity: owner_process_identity.to_string(),
            issued_at_unix_ms,
        };
        fence.integrity_digest = fence.digest();
        fence.fence_id = format!("resource-fence:{}", &fence.integrity_digest[..32]);
        fence.validate_integrity()?;
        Ok(fence)
    }

    fn digest(&self) -> String {
        let value = serde_json::json!({
            "schema": self.schema,
            "resource_id": self.resource_id,
            "tenant_id": self.tenant_id,
            "resource_kind": self.resource_kind,
            "resource_epoch": self.resource_epoch,
            "case_id": self.case_id,
            "operation_id": self.operation_id,
            "grant_id": self.grant_id,
            "effect_id": self.effect_id,
            "owner_pid": self.owner_pid,
            "owner_process_identity": self.owner_process_identity,
            "issued_at_unix_ms": self.issued_at_unix_ms,
        });
        digest_bytes(value.to_string().as_bytes())
    }

    pub fn validate_integrity(&self) -> Result<(), String> {
        if self.schema != RESOURCE_FENCE_SCHEMA
            || self.resource_id.is_empty()
            || !self.tenant_id.starts_with("tenant:")
            || self.resource_epoch == 0
            || self.case_id.is_empty()
            || self.operation_id.is_empty()
            || self.grant_id.is_empty()
            || self.effect_id.is_empty()
            || self.owner_pid == 0
            || self.owner_process_identity.is_empty()
            || self.issued_at_unix_ms == 0
        {
            return Err("invalid_resource_fence".to_string());
        }
        let digest = self.digest();
        if digest != self.integrity_digest
            || self.fence_id != format!("resource-fence:{}", &digest[..32])
        {
            return Err("resource_fence_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActiveResourceLease {
    pub fence: ResourceFence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceControlState {
    pub schema: String,
    pub identity: ResourceIdentity,
    pub resource_epoch: u64,
    pub event_sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_lease: Option<ActiveResourceLease>,
}

impl ResourceControlState {
    pub fn validate(&self) -> Result<(), String> {
        self.identity.validate()?;
        if (self.schema != RESOURCE_CONTROL_STATE_SCHEMA
            && self.schema != RESOURCE_CONTROL_STATE_SCHEMA_V1)
            || self.resource_epoch == 0
            || self.event_sequence == 0
        {
            return Err("invalid_resource_control_state".to_string());
        }
        if self.schema == RESOURCE_CONTROL_STATE_SCHEMA
            && (self.last_event_id.as_deref().is_none_or(str::is_empty)
                || self.last_event_digest.as_deref().is_none_or(str::is_empty))
        {
            return Err("resource_control_state_event_head_missing".to_string());
        }
        if let Some(active) = &self.active_lease {
            active.fence.validate_integrity()?;
            if active.fence.resource_id != self.identity.resource_id
                || active.fence.tenant_id != self.identity.tenant_id
                || active.fence.resource_kind != self.identity.resource_kind
                || active.fence.resource_epoch != self.resource_epoch
            {
                return Err("resource_control_active_fence_mismatch".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceControlAction {
    Acquired,
    Reclaimed,
    Released,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceControlEvent {
    pub schema: String,
    pub event_id: String,
    pub resource_id: String,
    pub resource_epoch: u64,
    pub sequence: u64,
    pub action: ResourceControlAction,
    pub fence_id: String,
    pub case_id: String,
    pub operation_id: String,
    pub grant_id: String,
    pub effect_id: String,
    pub owner_process_identity: String,
    pub committed_at_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_identity: Option<ResourceIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fence: Option<ResourceFence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_event_digest: Option<String>,
    pub integrity_digest: String,
}

impl ResourceControlEvent {
    pub(crate) fn build(
        action: ResourceControlAction,
        identity: &ResourceIdentity,
        fence: &ResourceFence,
        sequence: u64,
        committed_at_unix_ms: u64,
        previous: Option<&ResourceControlEvent>,
    ) -> Result<Self, String> {
        identity.validate()?;
        fence.validate_integrity()?;
        if identity.resource_id != fence.resource_id
            || identity.tenant_id != fence.tenant_id
            || identity.resource_kind != fence.resource_kind
        {
            return Err("resource_event_identity_fence_mismatch".to_string());
        }
        let mut event = Self {
            schema: RESOURCE_CONTROL_EVENT_SCHEMA.to_string(),
            event_id: String::new(),
            resource_id: fence.resource_id.clone(),
            resource_epoch: fence.resource_epoch,
            sequence,
            action,
            fence_id: fence.fence_id.clone(),
            case_id: fence.case_id.clone(),
            operation_id: fence.operation_id.clone(),
            grant_id: fence.grant_id.clone(),
            effect_id: fence.effect_id.clone(),
            owner_process_identity: fence.owner_process_identity.clone(),
            committed_at_unix_ms,
            resource_identity: Some(identity.clone()),
            fence: Some(fence.clone()),
            previous_event_id: previous.map(|event| event.event_id.clone()),
            previous_event_digest: previous.map(|event| event.integrity_digest.clone()),
            integrity_digest: String::new(),
        };
        event.integrity_digest = event.digest();
        event.event_id = format!("resource-event:{}", &event.integrity_digest[..32]);
        event.validate_integrity()?;
        Ok(event)
    }

    fn digest(&self) -> String {
        let value = if self.schema == RESOURCE_CONTROL_EVENT_SCHEMA_V1 {
            serde_json::json!({
                "schema": self.schema,
                "resource_id": self.resource_id,
                "resource_epoch": self.resource_epoch,
                "sequence": self.sequence,
                "action": self.action,
                "fence_id": self.fence_id,
                "case_id": self.case_id,
                "operation_id": self.operation_id,
                "grant_id": self.grant_id,
                "effect_id": self.effect_id,
                "owner_process_identity": self.owner_process_identity,
                "committed_at_unix_ms": self.committed_at_unix_ms,
            })
        } else {
            serde_json::json!({
                "schema": self.schema,
                "resource_id": self.resource_id,
                "resource_epoch": self.resource_epoch,
                "sequence": self.sequence,
                "action": self.action,
                "fence_id": self.fence_id,
                "case_id": self.case_id,
                "operation_id": self.operation_id,
                "grant_id": self.grant_id,
                "effect_id": self.effect_id,
                "owner_process_identity": self.owner_process_identity,
                "committed_at_unix_ms": self.committed_at_unix_ms,
                "resource_identity": self.resource_identity,
                "fence": self.fence,
                "previous_event_id": self.previous_event_id,
                "previous_event_digest": self.previous_event_digest,
            })
        };
        digest_bytes(value.to_string().as_bytes())
    }

    pub fn validate_integrity(&self) -> Result<(), String> {
        if (self.schema != RESOURCE_CONTROL_EVENT_SCHEMA
            && self.schema != RESOURCE_CONTROL_EVENT_SCHEMA_V1)
            || self.resource_id.is_empty()
            || self.resource_epoch == 0
            || self.sequence == 0
            || self.fence_id.is_empty()
            || self.effect_id.is_empty()
            || self.owner_process_identity.is_empty()
            || self.committed_at_unix_ms == 0
        {
            return Err("invalid_resource_control_event".to_string());
        }
        if self.schema == RESOURCE_CONTROL_EVENT_SCHEMA {
            let identity = self
                .resource_identity
                .as_ref()
                .ok_or_else(|| "resource_event_identity_missing".to_string())?;
            let fence = self
                .fence
                .as_ref()
                .ok_or_else(|| "resource_event_fence_missing".to_string())?;
            identity.validate()?;
            fence.validate_integrity()?;
            if identity.resource_id != self.resource_id
                || identity.resource_id != fence.resource_id
                || self.resource_epoch != fence.resource_epoch
                || self.fence_id != fence.fence_id
                || self.case_id != fence.case_id
                || self.operation_id != fence.operation_id
                || self.grant_id != fence.grant_id
                || self.effect_id != fence.effect_id
                || self.owner_process_identity != fence.owner_process_identity
                || (self.sequence == 1
                    && (self.previous_event_id.is_some() || self.previous_event_digest.is_some()))
                || (self.sequence > 1
                    && (self.previous_event_id.as_deref().is_none_or(str::is_empty)
                        || self
                            .previous_event_digest
                            .as_deref()
                            .is_none_or(str::is_empty)))
            {
                return Err("resource_control_event_semantic_mismatch".to_string());
            }
        }
        let digest = self.digest();
        if digest != self.integrity_digest
            || self.event_id != format!("resource-event:{}", &digest[..32])
        {
            return Err("resource_control_event_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

/// Rebuilds one exact current resource-control view solely from authoritative
/// v2 events.  Events may arrive in arbitrary LMDB key order; sequence and the
/// predecessor digest chain define their only accepted order.
pub fn rebuild_resource_control_state(
    events: &[ResourceControlEvent],
) -> Result<ResourceControlState, String> {
    if events.is_empty() {
        return Err("resource_history_empty".to_string());
    }
    let mut ordered = events.to_vec();
    ordered.sort_by_key(|event| event.sequence);
    let mut state: Option<ResourceControlState> = None;
    let mut previous: Option<ResourceControlEvent> = None;
    let mut last_committed_at = 0u64;

    for event in ordered {
        event.validate_integrity()?;
        if event.schema != RESOURCE_CONTROL_EVENT_SCHEMA {
            return Err("resource_history_v1_not_rebuildable".to_string());
        }
        let identity = event
            .resource_identity
            .as_ref()
            .ok_or_else(|| "resource_event_identity_missing".to_string())?;
        let fence = event
            .fence
            .as_ref()
            .ok_or_else(|| "resource_event_fence_missing".to_string())?;
        let expected_sequence = previous.as_ref().map_or(1, |prior| prior.sequence + 1);
        if event.sequence != expected_sequence {
            return Err("resource_history_sequence_gap_or_duplicate".to_string());
        }
        match previous.as_ref() {
            None if event.previous_event_id.is_some() || event.previous_event_digest.is_some() => {
                return Err("resource_history_unexpected_predecessor".to_string())
            }
            Some(prior)
                if event.previous_event_id.as_deref() != Some(prior.event_id.as_str())
                    || event.previous_event_digest.as_deref()
                        != Some(prior.integrity_digest.as_str()) =>
            {
                return Err("resource_history_predecessor_mismatch".to_string())
            }
            _ => {}
        }
        if event.committed_at_unix_ms < last_committed_at {
            return Err("resource_history_authority_time_regression".to_string());
        }

        match (&mut state, &event.action) {
            (None, ResourceControlAction::Acquired) if event.resource_epoch == 1 => {
                state = Some(ResourceControlState {
                    schema: RESOURCE_CONTROL_STATE_SCHEMA.to_string(),
                    identity: identity.clone(),
                    resource_epoch: 1,
                    event_sequence: event.sequence,
                    last_event_id: Some(event.event_id.clone()),
                    last_event_digest: Some(event.integrity_digest.clone()),
                    active_lease: Some(ActiveResourceLease {
                        fence: fence.clone(),
                    }),
                });
            }
            (None, _) => {
                return Err("resource_history_must_begin_with_acquired_epoch_one".to_string())
            }
            (Some(current), ResourceControlAction::Acquired) => {
                if current.identity != *identity
                    || current.active_lease.is_some()
                    || event.resource_epoch != current.resource_epoch.saturating_add(1)
                {
                    return Err("resource_history_invalid_acquisition".to_string());
                }
                current.resource_epoch = event.resource_epoch;
                current.active_lease = Some(ActiveResourceLease {
                    fence: fence.clone(),
                });
            }
            (Some(current), ResourceControlAction::Reclaimed) => {
                let active = current
                    .active_lease
                    .as_ref()
                    .ok_or_else(|| "resource_history_reclaim_without_active_lease".to_string())?;
                if current.identity != *identity
                    || event.resource_epoch != current.resource_epoch.saturating_add(1)
                    || active.fence.case_id != fence.case_id
                    || active.fence.operation_id != fence.operation_id
                    || active.fence.grant_id != fence.grant_id
                    || active.fence.effect_id != fence.effect_id
                    || active.fence.fence_id == fence.fence_id
                {
                    return Err("resource_history_invalid_reclaim".to_string());
                }
                current.resource_epoch = event.resource_epoch;
                current.active_lease = Some(ActiveResourceLease {
                    fence: fence.clone(),
                });
            }
            (Some(current), ResourceControlAction::Released) => {
                let active = current
                    .active_lease
                    .as_ref()
                    .ok_or_else(|| "resource_history_release_without_active_lease".to_string())?;
                if current.identity != *identity
                    || event.resource_epoch != current.resource_epoch
                    || active.fence != *fence
                {
                    return Err("resource_history_invalid_release".to_string());
                }
                current.active_lease = None;
            }
        }
        let current = state.as_mut().expect("state initialized by first event");
        current.event_sequence = event.sequence;
        current.last_event_id = Some(event.event_id.clone());
        current.last_event_digest = Some(event.integrity_digest.clone());
        current.validate()?;
        last_committed_at = event.committed_at_unix_ms;
        previous = Some(event);
    }

    state.ok_or_else(|| "resource_history_empty".to_string())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilesystemRelation {
    SameOrOverlapping,
    Disjoint,
    Unknown,
}

pub fn filesystem_relation(left: &str, right: &str) -> FilesystemRelation {
    let left = Path::new(left);
    let right = Path::new(right);
    if !left.is_absolute() || !right.is_absolute() {
        return FilesystemRelation::Unknown;
    }
    if left.starts_with(right) || right.starts_with(left) {
        FilesystemRelation::SameOrOverlapping
    } else {
        FilesystemRelation::Disjoint
    }
}

pub trait ResourceFenceAuthority {
    fn validate_carrier_fence(&self, fence: &ResourceFence) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filesystem_identity_and_relation_are_deterministic() {
        let left = ResourceIdentity::filesystem("tenant:a", "/tmp/a").unwrap();
        let again = ResourceIdentity::filesystem("tenant:a", "/tmp/a").unwrap();
        let other_tenant = ResourceIdentity::filesystem("tenant:b", "/tmp/a").unwrap();
        assert_eq!(left, again);
        assert_ne!(left.resource_id, other_tenant.resource_id);
        assert_eq!(
            filesystem_relation("/tmp/a", "/tmp/a/b"),
            FilesystemRelation::SameOrOverlapping
        );
        assert_eq!(
            filesystem_relation("relative", "/tmp/a"),
            FilesystemRelation::Unknown
        );
    }

    #[test]
    fn process_identity_rejects_pid_reuse_shape() {
        let first = LocalProcessIdentity {
            schema: PROCESS_IDENTITY_SCHEMA.to_string(),
            pid: 123,
            boot_id: "boot-a".to_string(),
            start_ticks: 10,
        };
        let reused = LocalProcessIdentity {
            start_ticks: 11,
            ..first.clone()
        };
        assert_ne!(first.canonical_identity(), reused.canonical_identity());
    }

    fn test_fence(
        identity: &ResourceIdentity,
        epoch: u64,
        suffix: &str,
        effect_id: &str,
    ) -> ResourceFence {
        ResourceFence::issue(
            identity,
            epoch,
            &format!("case:{suffix}"),
            &format!("operation:{suffix}"),
            &format!("grant:{suffix}"),
            effect_id,
            42,
            &format!("process:{suffix}"),
            1_000 + epoch,
        )
        .unwrap()
    }

    #[test]
    fn h14_resource_event_fsm_rebuilds_exact_state() {
        let identity = ResourceIdentity::filesystem("tenant:h14", "/tmp/h14-resource").unwrap();
        let first = test_fence(&identity, 1, "first", "effect:first");
        let acquired = ResourceControlEvent::build(
            ResourceControlAction::Acquired,
            &identity,
            &first,
            1,
            2_001,
            None,
        )
        .unwrap();
        let released = ResourceControlEvent::build(
            ResourceControlAction::Released,
            &identity,
            &first,
            2,
            2_002,
            Some(&acquired),
        )
        .unwrap();
        let second = test_fence(&identity, 2, "second", "effect:second");
        let acquired_again = ResourceControlEvent::build(
            ResourceControlAction::Acquired,
            &identity,
            &second,
            3,
            2_003,
            Some(&released),
        )
        .unwrap();
        let rebuilt = rebuild_resource_control_state(&[
            acquired_again.clone(),
            acquired.clone(),
            released.clone(),
        ])
        .unwrap();
        assert_eq!(rebuilt.identity, identity);
        assert_eq!(rebuilt.resource_epoch, 2);
        assert_eq!(rebuilt.event_sequence, 3);
        assert_eq!(
            rebuilt.active_lease.as_ref().map(|lease| &lease.fence),
            Some(&second)
        );
        assert_eq!(
            rebuilt.last_event_id.as_deref(),
            Some(acquired_again.event_id.as_str())
        );
        assert_eq!(
            rebuilt.last_event_digest.as_deref(),
            Some(acquired_again.integrity_digest.as_str())
        );
    }

    #[test]
    fn h14_resource_event_fsm_rejects_impossible_content_valid_histories() {
        let identity = ResourceIdentity::filesystem("tenant:h14", "/tmp/h14-invalid").unwrap();
        let first = test_fence(&identity, 1, "first", "effect:first");
        let acquired = ResourceControlEvent::build(
            ResourceControlAction::Acquired,
            &identity,
            &first,
            1,
            3_001,
            None,
        )
        .unwrap();
        let release_first = ResourceControlEvent::build(
            ResourceControlAction::Released,
            &identity,
            &first,
            1,
            3_001,
            None,
        )
        .unwrap();
        assert_eq!(
            rebuild_resource_control_state(&[release_first]).unwrap_err(),
            "resource_history_must_begin_with_acquired_epoch_one"
        );

        let second = test_fence(&identity, 2, "second", "effect:second");
        let double_acquire = ResourceControlEvent::build(
            ResourceControlAction::Acquired,
            &identity,
            &second,
            2,
            3_002,
            Some(&acquired),
        )
        .unwrap();
        assert_eq!(
            rebuild_resource_control_state(&[acquired.clone(), double_acquire]).unwrap_err(),
            "resource_history_invalid_acquisition"
        );
        assert_eq!(
            rebuild_resource_control_state(&[acquired.clone(), acquired.clone()]).unwrap_err(),
            "resource_history_sequence_gap_or_duplicate"
        );

        let released = ResourceControlEvent::build(
            ResourceControlAction::Released,
            &identity,
            &first,
            2,
            3_002,
            Some(&acquired),
        )
        .unwrap();
        let wrong_effect = test_fence(&identity, 2, "first", "effect:wrong");
        let wrong_reclaim = ResourceControlEvent::build(
            ResourceControlAction::Reclaimed,
            &identity,
            &wrong_effect,
            2,
            3_002,
            Some(&acquired),
        )
        .unwrap();
        assert_eq!(
            rebuild_resource_control_state(&[acquired.clone(), wrong_reclaim]).unwrap_err(),
            "resource_history_invalid_reclaim"
        );

        let regressed = test_fence(&identity, 1, "regressed", "effect:regressed");
        let regressed_acquire = ResourceControlEvent::build(
            ResourceControlAction::Acquired,
            &identity,
            &regressed,
            3,
            3_003,
            Some(&released),
        )
        .unwrap();
        assert_eq!(
            rebuild_resource_control_state(&[
                acquired.clone(),
                released.clone(),
                regressed_acquire,
            ])
            .unwrap_err(),
            "resource_history_invalid_acquisition"
        );

        let other_identity =
            ResourceIdentity::filesystem("tenant:other", "/tmp/h14-invalid").unwrap();
        let other_fence = test_fence(&other_identity, 2, "other", "effect:other");
        let identity_switch = ResourceControlEvent::build(
            ResourceControlAction::Acquired,
            &other_identity,
            &other_fence,
            3,
            3_003,
            Some(&released),
        )
        .unwrap();
        assert_eq!(
            rebuild_resource_control_state(&[acquired.clone(), released.clone(), identity_switch])
                .unwrap_err(),
            "resource_history_invalid_acquisition"
        );

        let gap = ResourceControlEvent::build(
            ResourceControlAction::Acquired,
            &identity,
            &second,
            3,
            3_003,
            Some(&released),
        )
        .unwrap();
        assert_eq!(
            rebuild_resource_control_state(&[acquired, gap]).unwrap_err(),
            "resource_history_sequence_gap_or_duplicate"
        );
    }
}
