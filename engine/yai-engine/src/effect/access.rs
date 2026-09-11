//! Bounded access contracts for existing Case Resources.
//!
//! These values describe requests and mechanical envelopes, not authority.
//! The admission owner evaluates every operation against current Case policy;
//! protocol adapters consume only admitted requests. No catalog/store lives here.

use super::{digest_bytes, normalize_relative_path, path_within_prefix};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;

pub const LOCAL_ACCESS_BINDING_SCHEMA: &str = "yai.local_resource_access_binding.v1";

pub mod source;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalAccessBinding {
    pub schema: String,
    pub case_id: String,
    pub attachment_id: String,
    pub address: ResourceAddress,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResourceAddress {
    Filesystem {
        root: super::LocalFilesystemBinding,
    },
    Discovery {
        root: super::LocalFilesystemBinding,
    },
    ProcessRunner {
        root: super::LocalFilesystemBinding,
        runners: BTreeMap<String, ProcessRunner>,
    },
    Sqlite {
        root: super::LocalFilesystemBinding,
        path: String,
        queries: BTreeMap<String, String>,
    },
    HttpService {
        endpoint: NetworkResourceAddress,
        paths: BTreeMap<String, String>,
    },
    Mcp {
        endpoint: NetworkResourceAddress,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkResourceAddress {
    pub endpoint: String,
    /// Every DNS answer must belong to this admitted set at dispatch. Changing
    /// deployment addresses requires explicit rebinding, not ambient access.
    pub allowed_ip_addresses: Vec<String>,
    pub credential_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessRunner {
    pub executable: String,
    pub executable_digest: String,
    pub argv: Vec<String>,
    pub working_directory: String,
    pub environment: BTreeMap<String, String>,
    pub timeout_ms: u64,
}

impl LocalAccessBinding {
    pub fn digest(&self) -> String {
        digest_bytes(&serde_json::to_vec(self).expect("typed resource binding serializes"))
    }

    pub fn root(&self) -> Option<&super::LocalFilesystemBinding> {
        match &self.address {
            ResourceAddress::Filesystem { root }
            | ResourceAddress::Discovery { root }
            | ResourceAddress::ProcessRunner { root, .. }
            | ResourceAddress::Sqlite { root, .. } => Some(root),
            _ => None,
        }
    }

    pub fn kind(&self) -> crate::transition::ResourceKind {
        use crate::transition::ResourceKind;
        match self.address {
            ResourceAddress::Filesystem { .. } => ResourceKind::Filesystem,
            ResourceAddress::Discovery { .. } => ResourceKind::Discovery,
            ResourceAddress::ProcessRunner { .. } => ResourceKind::ProcessRunner,
            ResourceAddress::Sqlite { .. } => ResourceKind::Database,
            ResourceAddress::HttpService { .. } => ResourceKind::HttpService,
            ResourceAddress::Mcp { .. } => ResourceKind::Mcp,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != LOCAL_ACCESS_BINDING_SCHEMA
            || !self.case_id.starts_with("case:")
            || !bounded_name(&self.attachment_id)
        {
            return Err("local_resource_access_binding_invalid".into());
        }
        if let Some(root) = self.root() {
            root.validate_secure_carrier()?;
            if root.case_id != self.case_id || root.attachment_id != self.attachment_id {
                return Err("local_resource_access_root_scope_mismatch".into());
            }
        }
        match &self.address {
            ResourceAddress::ProcessRunner { runners, .. } => {
                if runners.is_empty() || runners.len() > 16 {
                    return Err("resource_runner_count_invalid".into());
                }
                for (name, runner) in runners {
                    if !bounded_name(name)
                        || !Path::new(&runner.executable).is_absolute()
                        || !is_digest(&runner.executable_digest)
                        || runner.argv.len() > 32
                        || runner
                            .argv
                            .iter()
                            .any(|arg| arg.len() > 4096 || arg.contains('\0'))
                        || runner.timeout_ms == 0
                        || runner.timeout_ms > 60_000
                        || runner.environment.len() > 16
                        || runner.environment.iter().any(|(key, value)| {
                            key.is_empty()
                                || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                                || value.len() > 4096
                                || value.contains('\0')
                        })
                        || normalize_relative_path(&runner.working_directory)?
                            != runner.working_directory
                    {
                        return Err("resource_runner_contract_invalid".into());
                    }
                }
            }
            ResourceAddress::Sqlite { path, queries, .. } => {
                if normalize_relative_path(path)? != *path
                    || queries.is_empty()
                    || queries.len() > 32
                    || queries.iter().any(|(name, sql)| {
                        !bounded_name(name)
                            || sql.is_empty()
                            || sql.len() > 8192
                            || sql.contains('\0')
                    })
                {
                    return Err("database_query_binding_invalid".into());
                }
            }
            ResourceAddress::HttpService { endpoint, paths } => {
                endpoint.validate()?;
                if paths.is_empty()
                    || paths.len() > 32
                    || paths.iter().any(|(name, path)| {
                        !bounded_name(name)
                            || normalize_relative_path(path).as_deref() != Ok(path.as_str())
                    })
                {
                    return Err("http_resource_paths_invalid".into());
                }
            }
            ResourceAddress::Mcp { endpoint } => endpoint.validate()?,
            _ => {}
        }
        Ok(())
    }

    pub fn validate_attachment(
        &self,
        attachment: &crate::transition::ResourceAttachmentState,
    ) -> Result<(), String> {
        self.validate()?;
        attachment.validate()?;
        if attachment.attachment_id != self.attachment_id
            || attachment.kind != self.kind()
            || attachment
                .access
                .as_ref()
                .is_none_or(|access| access.configuration_digest != self.digest())
        {
            return Err("resource_access_binding_integrity_mismatch".into());
        }
        Ok(())
    }
}

impl NetworkResourceAddress {
    pub fn validate(&self) -> Result<(), String> {
        Self::validate_endpoint(&self.endpoint)?;
        if self.allowed_ip_addresses.is_empty()
            || self.allowed_ip_addresses.len() > 16
            || !unique(&self.allowed_ip_addresses)
            || self
                .allowed_ip_addresses
                .iter()
                .any(|ip| ip.parse::<std::net::IpAddr>().is_err())
            || self.credential_ref.as_ref().is_some_and(|key| {
                key.is_empty() || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            })
        {
            return Err("resource_endpoint_contract_invalid".into());
        }
        Ok(())
    }

    pub(crate) fn validate_endpoint(endpoint: &str) -> Result<(), String> {
        let rest = endpoint
            .strip_prefix("http://")
            .or_else(|| endpoint.strip_prefix("https://"))
            .ok_or_else(|| "resource_endpoint_scheme_invalid".to_string())?;
        if rest.is_empty()
            || rest.starts_with('/')
            || endpoint.len() > 2048
            || endpoint
                .chars()
                .any(|c| c.is_control() || c.is_whitespace())
            || endpoint.contains(['@', '#', '?', '\\', '%'])
            || endpoint.split('/').any(|part| part == "." || part == "..")
        {
            return Err("resource_endpoint_contract_invalid".into());
        }
        Ok(())
    }
}

/// Descriptor-relative, regular-file-only bounded read. The open descriptor is
/// retained through hashing; a candidate admission repeats this read and must
/// compare its digest to the discovered/reviewed one before object publication.
#[cfg(target_os = "linux")]
pub fn read_confined_file(
    root: &super::LocalFilesystemBinding,
    path: &str,
    limit: usize,
) -> Result<Vec<u8>, String> {
    normalize_relative_path(path)?;
    if limit == 0 || limit > MAX_ACCESS_BYTES {
        return Err("resource_read_bound_invalid".into());
    }
    let directory = super::open_verified_filesystem_root(root)?;
    let file = super::open_beneath(
        &directory,
        Path::new(path),
        libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC,
        0,
    )
    .map_err(|e| format!("resource_read_open_refused:{e}"))?;
    let before = file
        .metadata()
        .map_err(|e| format!("resource_read_metadata:{e}"))?;
    if !before.is_file() || before.len() > limit as u64 {
        return Err("resource_read_not_bounded_regular_file".into());
    }
    let mut bytes = Vec::new();
    (&file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("resource_read_failed:{e}"))?;
    let after = file
        .metadata()
        .map_err(|e| format!("resource_read_metadata:{e}"))?;
    use std::os::unix::fs::MetadataExt;
    if bytes.len() > limit
        || before.len() != after.len()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
    {
        return Err("resource_read_source_changed".into());
    }
    Ok(bytes)
}

/// Bounded traversal under one admitted prefix. Kernel-confined reads bind
/// every returned candidate to bytes, not to a suggestive filename. Directory
/// listings are observations, not an atomic filesystem snapshot or admission.
#[cfg(target_os = "linux")]
pub fn inspect_confined_tree(
    binding: &super::LocalFilesystemBinding,
    prefix: &str,
    needle: Option<&str>,
    max_bytes: usize,
    max_items: usize,
) -> Result<Value, String> {
    use std::os::fd::AsRawFd;
    if normalize_relative_path(prefix)? != prefix
        || max_bytes == 0
        || max_bytes > MAX_ACCESS_BYTES
        || max_items == 0
        || max_items > MAX_ACCESS_ITEMS
    {
        return Err("resource_tree_bounds_invalid".into());
    }
    let root = super::open_verified_filesystem_root(binding)?;
    let mut pending = vec![(prefix.to_string(), 0u8)];
    // Exact file discovery uses the same confined carrier, without enumerating
    // or reading siblings. Directory discovery retains its existing bounds.
    let fd = super::open_beneath(
        &root,
        Path::new(prefix),
        libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC,
        0,
    )
    .map_err(|e| format!("resource_tree_open_refused:{e}"))?;
    if fd.metadata().map_err(|e| e.to_string())?.is_file() {
        let bytes = read_confined_file(binding, prefix, max_bytes)?;
        let digest = digest_bytes(&bytes);
        let entries = if let Some(needle) = needle {
            let text = std::str::from_utf8(&bytes).map_err(|_| "resource_search_nontext_file")?;
            let matches: Vec<_> = text
                .lines()
                .enumerate()
                .filter(|(_, line)| line.contains(needle))
                .map(|(line, text)| serde_json::json!({"line":line+1,"text":text}))
                .collect();
            if matches.is_empty() {
                vec![]
            } else {
                vec![serde_json::json!({"path":prefix,"digest":digest,"matches":matches})]
            }
        } else {
            vec![
                serde_json::json!({"path":prefix,"digest":digest,"bytes":bytes.len(),"admitted":false}),
            ]
        };
        if serde_json::to_vec(&entries)
            .map_err(|e| e.to_string())?
            .len()
            > max_bytes
        {
            return Err("resource_tree_result_bound_exceeded".into());
        }
        return Ok(
            serde_json::json!({"posture":"observed_candidates_not_admitted","prefix":prefix,
            "entries":entries,"bytes_read":bytes.len(),"entries_inspected":1}),
        );
    }
    let mut inspected = 0usize;
    let mut bytes_read = 0usize;
    let mut result = Vec::new();
    while let Some((directory, depth)) = pending.pop() {
        if depth > 8 {
            return Err("resource_tree_depth_exceeded".into());
        }
        let fd = super::open_beneath(
            &root,
            Path::new(&directory),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
            0,
        )
        .map_err(|e| format!("resource_tree_open_refused:{e}"))?;
        // The proc path is generated from our own open descriptor, never from
        // request text. It enumerates that pinned directory, not a reopened
        // ambient user path. Entries are reopened beneath the verified root.
        let entries = std::fs::read_dir(format!("/proc/self/fd/{}", fd.as_raw_fd()))
            .map_err(|e| format!("resource_tree_list:{e}"))?;
        for entry in entries {
            inspected += 1;
            if inspected > max_items {
                return Err("resource_tree_item_bound_exceeded".into());
            }
            let entry = entry.map_err(|e| format!("resource_tree_entry:{e}"))?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| "resource_tree_name_not_utf8".to_string())?;
            let path = format!("{directory}/{name}");
            if normalize_relative_path(&path)? != path || path.chars().any(char::is_control) {
                return Err("resource_tree_name_invalid".into());
            }
            let kind = entry
                .file_type()
                .map_err(|e| format!("resource_tree_type:{e}"))?;
            if kind.is_dir() {
                pending.push((path, depth + 1));
                continue;
            }
            if !kind.is_file() {
                return Err("resource_tree_nonregular_entry".into());
            }
            let bytes = read_confined_file(binding, &path, max_bytes.saturating_sub(bytes_read))?;
            bytes_read += bytes.len();
            let digest = digest_bytes(&bytes);
            if let Some(needle) = needle {
                let text = std::str::from_utf8(&bytes)
                    .map_err(|_| "resource_search_nontext_file".to_string())?;
                let matches: Vec<_> = text
                    .lines()
                    .enumerate()
                    .filter(|(_, line)| line.contains(needle))
                    .map(|(line, text)| serde_json::json!({"line":line+1,"text":text}))
                    .collect();
                if !matches.is_empty() {
                    result.push(serde_json::json!({"path":path,"digest":digest,"matches":matches}));
                }
            } else {
                result.push(serde_json::json!({"path":path,"digest":digest,"bytes":bytes.len(),"admitted":false}));
            }
            if serde_json::to_vec(&result)
                .map_err(|e| e.to_string())?
                .len()
                > max_bytes
            {
                return Err("resource_tree_result_bound_exceeded".into());
            }
        }
    }
    result.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
    Ok(
        serde_json::json!({"posture":"observed_candidates_not_admitted", "prefix":prefix,
        "entries":result, "bytes_read":bytes_read, "entries_inspected":inspected}),
    )
}

/// A bounded source image is interrogated read-only, never the ambient SQLite
/// filesystem. This initial adapter admits a standalone rollback-mode database,
/// not WAL/journal-dependent state. Its result identifies the exact image read;
/// it does not claim a live transactional snapshot of an arbitrary SQL service.
#[cfg(target_os = "linux")]
pub fn query_sqlite_snapshot(
    binding: &LocalAccessBinding,
    name: &str,
    max_rows: usize,
    max_bytes: usize,
) -> Result<Value, String> {
    use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
    use rusqlite::{Connection, MAIN_DB};
    binding.validate()?;
    if max_rows == 0
        || max_rows > MAX_ACCESS_ITEMS
        || max_bytes == 0
        || max_bytes > MAX_ACCESS_BYTES
    {
        return Err("database_result_bound_invalid".into());
    }
    let ResourceAddress::Sqlite {
        root,
        path,
        queries,
    } = &binding.address
    else {
        return Err("database_resource_kind_mismatch".into());
    };
    let sql = queries
        .get(name)
        .ok_or_else(|| "database_query_not_bound".to_string())?;
    let bytes = read_confined_file(root, path, MAX_ACCESS_BYTES)?;
    if bytes.len() < 100 || &bytes[..16] != b"SQLite format 3\0" || bytes[18] != 1 || bytes[19] != 1
    {
        return Err("database_requires_bounded_standalone_rollback_image".into());
    }
    let directory = super::open_verified_filesystem_root(root)?;
    for suffix in ["-journal", "-wal", "-shm"] {
        match super::open_beneath(
            &directory,
            Path::new(&format!("{path}{suffix}")),
            libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC,
            0,
        ) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err("database_sidecar_state_requires_transactional_adapter".into()),
        }
    }
    let map_error = |e: rusqlite::Error| format!("database_query_refused:{e}");
    let mut db = Connection::open_in_memory().map_err(map_error)?;
    db.deserialize_read_exact(MAIN_DB, bytes.as_slice(), bytes.len(), true)
        .map_err(map_error)?;
    db.set_limit(
        rusqlite::limits::Limit::SQLITE_LIMIT_LENGTH,
        max_bytes as i32,
    )
    .map_err(map_error)?;
    db.set_limit(rusqlite::limits::Limit::SQLITE_LIMIT_COLUMN, 32)
        .map_err(map_error)?;
    db.set_limit(rusqlite::limits::Limit::SQLITE_LIMIT_SQL_LENGTH, 8192)
        .map_err(map_error)?;
    db.set_limit(rusqlite::limits::Limit::SQLITE_LIMIT_EXPR_DEPTH, 32)
        .map_err(map_error)?;
    let start = std::time::Instant::now();
    let mut progress_calls = 0usize;
    db.progress_handler(
        100,
        Some(move || {
            progress_calls += 1;
            progress_calls > 10_000 || start.elapsed() > std::time::Duration::from_secs(2)
        }),
    )
    .map_err(map_error)?;
    db.authorizer(Some(|context: AuthContext<'_>| match context.action {
        AuthAction::Select | AuthAction::Read { .. } => Authorization::Allow,
        _ => Authorization::Deny,
    }))
    .map_err(map_error)?;
    let mut statement = db.prepare(sql).map_err(map_error)?;
    if !statement.readonly() || statement.parameter_count() != 0 {
        return Err("database_query_not_readonly_bound_statement".into());
    }
    let columns = statement
        .column_names()
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let count = columns.len();
    let mut cursor = statement.query([]).map_err(map_error)?;
    let mut rows = Vec::new();
    while let Some(row) = cursor.next().map_err(map_error)? {
        if rows.len() == max_rows {
            return Err("database_row_bound_exceeded".into());
        }
        let mut values = Vec::new();
        for column in 0..count {
            use rusqlite::types::ValueRef;
            let value = match row.get_ref(column).map_err(map_error)? {
                ValueRef::Null => Value::Null,
                ValueRef::Integer(value) => serde_json::json!(value),
                ValueRef::Real(value) if value.is_finite() => serde_json::json!(value),
                ValueRef::Text(value) => Value::String(
                    std::str::from_utf8(value)
                        .map_err(|_| "database_text_not_utf8".to_string())?
                        .into(),
                ),
                _ => return Err("database_value_shape_not_admitted".into()),
            };
            values.push(value);
        }
        rows.push(values);
        if serde_json::to_vec(&rows).map_err(|e| e.to_string())?.len() > max_bytes {
            return Err("database_byte_bound_exceeded".into());
        }
    }
    // A concurrent replacement or write invalidates this observation, even
    // though the isolated read-only query itself could not affect its source.
    if read_confined_file(root, path, MAX_ACCESS_BYTES)? != bytes {
        return Err("database_source_changed".into());
    }
    let result = serde_json::json!({
        "query": name, "source_digest": digest_bytes(&bytes), "columns": columns,
        "rows": rows, "posture": "bounded_readonly_image", "sqlite_version": rusqlite::version(),
    });
    if serde_json::to_vec(&result)
        .map_err(|e| e.to_string())?
        .len()
        > max_bytes
    {
        return Err("database_byte_bound_exceeded".into());
    }
    Ok(result)
}

pub const RESOURCE_ACCESS_SCHEMA: &str = "yai.resource_access.v1";
pub const RESOURCE_REQUEST_SCHEMA: &str = "yai.resource_request.v1";

/// Short-lived admission snapshot, never a stored permission or authority.
/// The adapter must not retain it across work; publication rechecks generation
/// and current policy in the canonical transaction.
#[derive(Clone, Debug)]
pub struct ResourceReadAdmission {
    pub case_generation: u64,
    pub operation: super::Operation,
    pub decision: super::Decision,
    pub binding: LocalAccessBinding,
    pub access: ResourceAccessContract,
}
pub const MAX_ACCESS_BYTES: usize = 65_536;
pub const MAX_ACCESS_ITEMS: usize = 128;
pub const RESOURCE_OBSERVATION_SCHEMA: &str = "yai.resource_observation.v1";
pub const CASE_CONTENT_ADMISSION_SCHEMA: &str = "yai.case_content_admission.v1";

/// Canonical Case relationship, not a Resource permission or another byte
/// owner. Exact discovered bytes become immutable informational material only
/// after a distinct admission Operation/Decision. No Turn is manufactured.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseContentAdmission {
    pub schema: String,
    pub admission_id: String,
    pub integrity_digest: String,
    pub operation_id: String,
    pub decision_id: String,
    pub discovery_observation_id: String,
    pub source_resource_id: String,
    pub source_configuration_digest: String,
    pub source_path: String,
    pub admitted_by_principal_id: String,
    pub participant_ids: Vec<String>,
    pub object: crate::conversation::ConversationContentObject,
}

impl CaseContentAdmission {
    pub fn new(
        operation: &super::Operation,
        decision: &super::Decision,
        discovery: &ResourceObservation,
        principal: &str,
        participant_ids: Vec<String>,
        object: crate::conversation::ConversationContentObject,
    ) -> Result<Self, String> {
        let request = operation
            .resource_request
            .as_ref()
            .ok_or("content_admission_request_missing")?;
        let ResourceAction::AdmitContent {
            path,
            candidate_digest,
        } = &request.action
        else {
            return Err("content_admission_action_required".into());
        };
        let candidate = discovery.result["entries"]
            .as_array()
            .is_some_and(|entries| {
                entries.iter().any(|entry| {
                    entry["path"] == *path
                        && entry["digest"] == *candidate_digest
                        && entry["bytes"].as_u64() == Some(object.byte_length)
                        && entry["admitted"] == false
                })
            });
        discovery.validate()?;
        operation.validate()?;
        decision.validate_integrity()?;
        if discovery.case_id != operation.case_id
            || discovery.participant_id != operation.participant_id
            || discovery.resource_attachment_id != operation.resource_attachment_id
            || discovery.kind != AccessKind::Discover
            || discovery.configuration_digest != request.configuration_digest
            || !candidate
            || object.case_id != operation.case_id
            || object.content_digest != *candidate_digest
            || decision.operation_id != operation.operation_id
            || decision.operation_digest != operation.operation_digest
            || decision.outcome != super::DecisionOutcome::Allow
            || decision.decision_basis.is_none()
        {
            return Err("content_admission_exact_candidate_or_authority_mismatch".into());
        }
        let mut value = Self {
            schema: CASE_CONTENT_ADMISSION_SCHEMA.into(),
            admission_id: String::new(),
            integrity_digest: String::new(),
            operation_id: operation.operation_id.clone(),
            decision_id: decision.decision_id.clone(),
            discovery_observation_id: discovery.observation_id.clone(),
            source_resource_id: operation.resource_attachment_id.clone(),
            source_configuration_digest: request.configuration_digest.clone(),
            source_path: path.clone(),
            admitted_by_principal_id: principal.into(),
            participant_ids,
            object,
        };
        value.integrity_digest = value.digest();
        value.admission_id = format!("case-content:{}", &value.integrity_digest[7..39]);
        value.validate()?;
        Ok(value)
    }

    fn digest(&self) -> String {
        let mut value = self.clone();
        value.admission_id.clear();
        value.integrity_digest.clear();
        digest_bytes(&serde_json::to_vec(&value).expect("typed Case content admission serializes"))
    }

    pub fn validate(&self) -> Result<(), String> {
        self.object.validate()?;
        let mut participants = self.participant_ids.clone();
        participants.sort();
        participants.dedup();
        if self.schema != CASE_CONTENT_ADMISSION_SCHEMA
            || self.object.modality != crate::conversation::ContentModality::File
            || !self.operation_id.starts_with("operation:")
            || !self.decision_id.starts_with("decision:")
            || !self
                .discovery_observation_id
                .starts_with("resource-observation:")
            || !bounded_name(&self.source_resource_id)
            || !is_digest(&self.source_configuration_digest)
            || normalize_relative_path(&self.source_path)? != self.source_path
            || !self.admitted_by_principal_id.starts_with("principal:")
            || participants.is_empty()
            || participants.len() > 32
            || participants != self.participant_ids
            || participants
                .iter()
                .any(|id| !id.starts_with("participant:") || !bounded_name(id))
            || self.integrity_digest != self.digest()
            || self.admission_id != format!("case-content:{}", &self.integrity_digest[7..39])
        {
            return Err("case_content_admission_invalid".into());
        }
        Ok(())
    }
}
pub const PREPARED_RESOURCE_EFFECT_SCHEMA: &str = "yai.prepared_resource_effect.v1";
pub const RESOURCE_EFFECT_RECEIPT_SCHEMA: &str = "yai.resource_effect_receipt.v1";
pub const MCP_EFFECT_BACKEND: &str = "rust.mcp.streamable_http.stateless.2026-07-28.v1";
pub const PROCESS_EFFECT_BACKEND: &str = "yai.process.readonly_single_process.linux_x86_64.v1";

/// Process-local carrier failure using the existing effect taxonomy. It is
/// not a receipt and cannot release a lease without canonical finalization.
#[derive(Clone, Debug)]
pub struct ResourceCarrierFailure {
    pub outcome: super::EffectOutcome,
    pub reason: String,
}

impl ResourceCarrierFailure {
    pub fn before_dispatch(reason: impl Into<String>) -> Self {
        Self {
            outcome: super::EffectOutcome::FailedNoEffect,
            reason: reason.into(),
        }
    }
    pub fn indeterminate(reason: impl Into<String>) -> Self {
        Self {
            outcome: super::EffectOutcome::Indeterminate,
            reason: reason.into(),
        }
    }
}

/// One current Grant is consumed by one PREPARE. Protocol preflight is an
/// observation, not permission. The existing resource-control owner seals the
/// fence in the same transaction as this canonical transition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedResourceEffect {
    pub schema: String,
    pub effect_id: String,
    pub case_id: String,
    pub participant_id: String,
    pub operation_id: String,
    pub decision_id: String,
    pub grant_id: String,
    pub resource_attachment_id: String,
    pub request_digest: String,
    pub idempotency_key: String,
    pub kind: AccessKind,
    pub carrier_backend: String,
    pub expected_pre_observation: ResourceObservation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_fence: Option<crate::resource_control::ResourceFence>,
}

impl PreparedResourceEffect {
    pub fn new(
        operation: &super::Operation,
        decision: &super::Decision,
        grant: &super::ExecutionGrant,
        observation: ResourceObservation,
    ) -> Result<Self, String> {
        super::validate_grant(operation, decision, grant, grant.expected_case_generation)?;
        if grant.schema != super::RESOURCE_EXECUTION_GRANT_SCHEMA {
            return Err("resource_prepare_requires_grant_v4".into());
        }
        observation.validate()?;
        let request = operation
            .resource_request
            .as_ref()
            .ok_or("resource_prepare_request_missing")?;
        if observation.case_id != operation.case_id
            || observation.participant_id != operation.participant_id
            || observation.operation_id != operation.operation_id
            || observation.decision_id != decision.decision_id
            || observation.resource_attachment_id != operation.resource_attachment_id
            || observation.configuration_digest != request.configuration_digest
            || observation.request_digest != request.digest()
            || observation.kind != request.action.kind()
        {
            return Err("resource_prepare_pre_observation_mismatch".into());
        }
        let carrier_backend = match request.action.kind() {
            AccessKind::ProcessRun => PROCESS_EFFECT_BACKEND,
            AccessKind::McpToolCall => MCP_EFFECT_BACKEND,
            _ => return Err("resource_effect_kind_not_admitted".into()),
        };
        Ok(Self {
            schema: PREPARED_RESOURCE_EFFECT_SCHEMA.into(),
            effect_id: format!("effect:{}", &grant.integrity_digest[7..39]),
            case_id: operation.case_id.clone(),
            participant_id: operation.participant_id.clone(),
            operation_id: operation.operation_id.clone(),
            decision_id: decision.decision_id.clone(),
            grant_id: grant.grant_id.clone(),
            resource_attachment_id: operation.resource_attachment_id.clone(),
            request_digest: request.digest(),
            idempotency_key: grant.idempotency_key.clone(),
            kind: request.action.kind(),
            carrier_backend: carrier_backend.into(),
            expected_pre_observation: observation,
            resource_fence: None,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        self.expected_pre_observation.validate()?;
        let fence = self
            .resource_fence
            .as_ref()
            .ok_or("resource_prepare_fence_required")?;
        fence.validate_integrity()?;
        let expected_backend = match self.kind {
            AccessKind::ProcessRun => PROCESS_EFFECT_BACKEND,
            AccessKind::McpToolCall => MCP_EFFECT_BACKEND,
            _ => return Err("resource_effect_kind_not_admitted".into()),
        };
        let pre = &self.expected_pre_observation;
        if self.schema != PREPARED_RESOURCE_EFFECT_SCHEMA
            || self.carrier_backend != expected_backend
            || !self.effect_id.starts_with("effect:")
            || !self.grant_id.starts_with("grant:")
            || !self.idempotency_key.starts_with("effect-key:")
            || !is_digest(&self.request_digest)
            || pre.case_id != self.case_id
            || pre.participant_id != self.participant_id
            || pre.operation_id != self.operation_id
            || pre.decision_id != self.decision_id
            || pre.resource_attachment_id != self.resource_attachment_id
            || pre.kind != self.kind
            || pre.request_digest != self.request_digest
            || fence.case_id != self.case_id
            || fence.operation_id != self.operation_id
            || fence.grant_id != self.grant_id
            || fence.effect_id != self.effect_id
        {
            return Err("resource_prepare_contract_invalid".into());
        }
        Ok(())
    }
}

/// Terminal response receipt, not proof that assertions returned by a tool are
/// true. Applied means the admitted operation executed and its bounded outcome
/// was observed; a process exit 7 can therefore be an Applied process execution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceEffectReceipt {
    pub schema: String,
    pub receipt_id: String,
    pub integrity_digest: String,
    pub effect_id: String,
    pub operation_id: String,
    pub decision_id: String,
    pub grant_id: String,
    pub resource_attachment_id: String,
    pub pre_observation_id: String,
    pub post_observation_id: String,
    pub post_observation_digest: String,
    pub outcome: super::EffectOutcome,
    pub external_execution_started: bool,
    pub carrier_backend: String,
}

impl ResourceEffectReceipt {
    pub fn new(
        prepared: &PreparedResourceEffect,
        post: &ResourceObservation,
        outcome: super::EffectOutcome,
        external_execution_started: bool,
    ) -> Result<Self, String> {
        prepared.validate()?;
        let pre = &prepared.expected_pre_observation;
        if post.case_id != prepared.case_id
            || post.participant_id != prepared.participant_id
            || post.operation_id != prepared.operation_id
            || post.decision_id != prepared.decision_id
            || post.resource_attachment_id != prepared.resource_attachment_id
            || post.kind != prepared.kind
            || post.configuration_digest != pre.configuration_digest
            || post.request_digest != prepared.request_digest
            || post.observed_at_unix_ms < pre.observed_at_unix_ms
        {
            return Err("resource_effect_post_observation_mismatch".into());
        }
        let mut value = Self {
            schema: RESOURCE_EFFECT_RECEIPT_SCHEMA.into(),
            receipt_id: String::new(),
            integrity_digest: String::new(),
            effect_id: prepared.effect_id.clone(),
            operation_id: prepared.operation_id.clone(),
            decision_id: prepared.decision_id.clone(),
            grant_id: prepared.grant_id.clone(),
            resource_attachment_id: prepared.resource_attachment_id.clone(),
            pre_observation_id: pre.observation_id.clone(),
            post_observation_id: post.observation_id.clone(),
            post_observation_digest: post.integrity_digest.clone(),
            outcome,
            external_execution_started,
            carrier_backend: prepared.carrier_backend.clone(),
        };
        value.integrity_digest = value.digest();
        value.receipt_id = format!("effect-receipt:{}", &value.integrity_digest[7..39]);
        value.validate(post)?;
        Ok(value)
    }

    fn digest(&self) -> String {
        let mut value = self.clone();
        value.receipt_id.clear();
        value.integrity_digest.clear();
        digest_bytes(&serde_json::to_vec(&value).expect("typed resource effect receipt serializes"))
    }

    pub fn validate(&self, post: &ResourceObservation) -> Result<(), String> {
        post.validate()?;
        use super::EffectOutcome;
        let terminal = matches!(
            (&self.outcome, self.external_execution_started),
            (EffectOutcome::Applied, true)
                | (
                    EffectOutcome::FailedNoEffect | EffectOutcome::NoEffect,
                    false
                )
        );
        if self.schema != RESOURCE_EFFECT_RECEIPT_SCHEMA
            || !terminal
            || self.operation_id != post.operation_id
            || self.decision_id != post.decision_id
            || self.resource_attachment_id != post.resource_attachment_id
            || self.post_observation_id != post.observation_id
            || self.post_observation_digest != post.integrity_digest
            || self.pre_observation_id == self.post_observation_id
            || !self.effect_id.starts_with("effect:")
            || !self.grant_id.starts_with("grant:")
            || !matches!(
                self.carrier_backend.as_str(),
                PROCESS_EFFECT_BACKEND | MCP_EFFECT_BACKEND
            )
            || self.integrity_digest != self.digest()
            || self.receipt_id != format!("effect-receipt:{}", &self.integrity_digest[7..39])
        {
            return Err("resource_effect_receipt_contract_invalid".into());
        }
        Ok(())
    }
}

/// External material admitted under an exact ALLOW Decision. This records an
/// observation, not the truth of the external system's assertions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceObservation {
    pub schema: String,
    pub observation_id: String,
    pub integrity_digest: String,
    pub case_id: String,
    pub participant_id: String,
    pub operation_id: String,
    pub decision_id: String,
    pub resource_attachment_id: String,
    pub configuration_digest: String,
    pub request_digest: String,
    pub kind: AccessKind,
    pub result: Value,
    pub observed_at_unix_ms: u64,
}

impl ResourceObservation {
    pub fn new(
        operation: &super::Operation,
        decision: &super::Decision,
        result: Value,
        observed_at_unix_ms: u64,
    ) -> Result<Self, String> {
        operation.validate()?;
        decision.validate_integrity()?;
        let request = operation
            .resource_request
            .as_ref()
            .ok_or_else(|| "resource_observation_request_missing".to_string())?;
        if decision.operation_id != operation.operation_id
            || decision.operation_digest != operation.operation_digest
            || decision.outcome != super::DecisionOutcome::Allow
            || decision.decision_basis.is_none()
        {
            return Err("resource_observation_requires_exact_allow_decision".into());
        }
        let mut value = Self {
            schema: RESOURCE_OBSERVATION_SCHEMA.into(),
            observation_id: String::new(),
            integrity_digest: String::new(),
            case_id: operation.case_id.clone(),
            participant_id: operation.participant_id.clone(),
            operation_id: operation.operation_id.clone(),
            decision_id: decision.decision_id.clone(),
            resource_attachment_id: operation.resource_attachment_id.clone(),
            configuration_digest: request.configuration_digest.clone(),
            request_digest: request.digest(),
            kind: request.action.kind(),
            result,
            observed_at_unix_ms,
        };
        value.integrity_digest = value.digest();
        value.observation_id = format!("resource-observation:{}", &value.integrity_digest[7..39]);
        value.validate()?;
        Ok(value)
    }

    fn digest(&self) -> String {
        let mut value = self.clone();
        value.observation_id.clear();
        value.integrity_digest.clear();
        digest_bytes(&serde_json::to_vec(&value).expect("typed observation serializes"))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != RESOURCE_OBSERVATION_SCHEMA
            || !self.case_id.starts_with("case:")
            || !self.participant_id.starts_with("participant:")
            || !self.operation_id.starts_with("operation:")
            || !self.decision_id.starts_with("decision:")
            || self.resource_attachment_id.is_empty()
            || !is_digest(&self.configuration_digest)
            || !is_digest(&self.request_digest)
            || self.observed_at_unix_ms == 0
            || !self.result.is_object()
            || serde_json::to_vec(&self.result)
                .map_err(|e| e.to_string())?
                .len()
                > MAX_ACCESS_BYTES
            || self.integrity_digest != self.digest()
            || self.observation_id
                != format!("resource-observation:{}", &self.integrity_digest[7..39])
        {
            return Err("resource_observation_contract_invalid".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessKind {
    FilesystemRead,
    FilesystemSearch,
    Discover,
    AdmitContent,
    ContentRead,
    ProcessRun,
    DatabaseQuery,
    DatabaseMutation,
    HttpFetch,
    McpCatalog,
    McpResourceRead,
    McpToolCall,
}

impl AccessKind {
    pub fn operation_name(self) -> &'static str {
        match self {
            Self::FilesystemRead => "filesystem.read",
            Self::FilesystemSearch => "filesystem.search",
            Self::Discover => "discovery.enumerate",
            Self::AdmitContent => "content.admit",
            Self::ContentRead => "content.read",
            Self::ProcessRun => "process.run",
            Self::DatabaseQuery => "database.query",
            Self::DatabaseMutation => "database.mutate",
            Self::HttpFetch => "http.fetch",
            Self::McpCatalog => "mcp.catalog",
            Self::McpResourceRead => "mcp.resource.read",
            Self::McpToolCall => "mcp.tool.call",
        }
    }

    pub fn resource_name(self) -> &'static str {
        match self {
            Self::FilesystemRead | Self::FilesystemSearch => "filesystem",
            Self::Discover | Self::AdmitContent | Self::ContentRead => "discovery",
            Self::ProcessRun => "process_runner",
            Self::DatabaseQuery | Self::DatabaseMutation => "database",
            Self::HttpFetch => "http_service",
            Self::McpCatalog | Self::McpResourceRead | Self::McpToolCall => "mcp",
        }
    }

    /// MCP annotations never downgrade a tool to observation-only authority.
    pub fn is_external_effect(self) -> bool {
        matches!(
            self,
            Self::ProcessRun | Self::DatabaseMutation | Self::McpToolCall
        )
    }
}

/// Canonical attachment envelope. Physical addresses/SQL/executable/environment
/// live in the existing local resource binding owner and are integrity-bound.
/// No credentials, arbitrary endpoint, SQL or shell string comes from a model.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceAccessContract {
    pub schema: String,
    pub configuration_digest: String,
    pub participant_ids: Vec<String>,
    pub operations: Vec<AccessKind>,
    pub read_prefixes: Vec<String>,
    pub names: Vec<String>,
    pub max_output_bytes: usize,
    pub max_items: usize,
}

impl ResourceAccessContract {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != RESOURCE_ACCESS_SCHEMA
            || !is_digest(&self.configuration_digest)
            || self.participant_ids.is_empty()
            || self.participant_ids.len() > 64
            || self
                .participant_ids
                .iter()
                .any(|id| !id.starts_with("participant:"))
            || self.operations.is_empty()
            || self.max_output_bytes == 0
            || self.max_output_bytes > MAX_ACCESS_BYTES
            || self.max_items == 0
            || self.max_items > MAX_ACCESS_ITEMS
            || self.names.len() > MAX_ACCESS_ITEMS
            || self.read_prefixes.len() > MAX_ACCESS_ITEMS
            || !unique(&self.operations)
            || !unique(&self.participant_ids)
            || !unique(&self.names)
            || !unique(&self.read_prefixes)
        {
            return Err("resource_access_contract_invalid".into());
        }
        for prefix in &self.read_prefixes {
            if normalize_relative_path(prefix)? != *prefix {
                return Err("resource_access_prefix_not_canonical".into());
            }
        }
        if self.names.iter().any(|name| !bounded_name(name)) {
            return Err("resource_access_name_invalid".into());
        }
        Ok(())
    }

    /// Mechanical and disclosure checks only. Success is NOT an ALLOW Decision.
    pub fn admits_request(
        &self,
        participant: &str,
        request: &ResourceRequest,
    ) -> Result<(), String> {
        self.validate()?;
        request.validate()?;
        if !self.participant_ids.iter().any(|id| id == participant) {
            return Err("resource_participant_not_disclosed".into());
        }
        if !self.operations.contains(&request.action.kind()) {
            return Err("resource_operation_not_attached".into());
        }
        if request.configuration_digest != self.configuration_digest {
            return Err("resource_configuration_stale".into());
        }
        if let Some(path) = request.action.path() {
            if !self
                .read_prefixes
                .iter()
                .any(|prefix| path_within_prefix(prefix, path))
            {
                return Err("resource_path_outside_attachment".into());
            }
        }
        if let Some(name) = request.action.name() {
            if !self.names.iter().any(|admitted| admitted == name) {
                return Err("resource_name_not_attached".into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceRequest {
    pub schema: String,
    pub configuration_digest: String,
    pub action: ResourceAction,
}

/// Query/runner names resolve to operator-bound definitions. They are not SQL,
/// executable paths, commands or URLs. MCP arguments remain candidate data.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResourceAction {
    FilesystemRead {
        path: String,
    },
    FilesystemSearch {
        path: String,
        needle: String,
    },
    Discover {
        path: String,
    },
    AdmitContent {
        path: String,
        candidate_digest: String,
    },
    ContentRead {
        admission_id: String,
    },
    ProcessRun {
        name: String,
    },
    DatabaseQuery {
        name: String,
    },
    DatabaseMutation {
        name: String,
    },
    HttpFetch {
        name: String,
    },
    McpCatalog,
    McpResourceRead {
        uri: String,
        catalog_digest: String,
    },
    McpToolCall {
        name: String,
        catalog_digest: String,
        arguments: Value,
    },
}

impl ResourceAction {
    pub fn kind(&self) -> AccessKind {
        match self {
            Self::FilesystemRead { .. } => AccessKind::FilesystemRead,
            Self::FilesystemSearch { .. } => AccessKind::FilesystemSearch,
            Self::Discover { .. } => AccessKind::Discover,
            Self::AdmitContent { .. } => AccessKind::AdmitContent,
            Self::ContentRead { .. } => AccessKind::ContentRead,
            Self::ProcessRun { .. } => AccessKind::ProcessRun,
            Self::DatabaseQuery { .. } => AccessKind::DatabaseQuery,
            Self::DatabaseMutation { .. } => AccessKind::DatabaseMutation,
            Self::HttpFetch { .. } => AccessKind::HttpFetch,
            Self::McpCatalog => AccessKind::McpCatalog,
            Self::McpResourceRead { .. } => AccessKind::McpResourceRead,
            Self::McpToolCall { .. } => AccessKind::McpToolCall,
        }
    }

    pub fn path(&self) -> Option<&str> {
        match self {
            Self::FilesystemRead { path }
            | Self::FilesystemSearch { path, .. }
            | Self::Discover { path }
            | Self::AdmitContent { path, .. } => Some(path),
            _ => None,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::ProcessRun { name }
            | Self::DatabaseQuery { name }
            | Self::DatabaseMutation { name }
            | Self::HttpFetch { name }
            | Self::McpToolCall { name, .. } => Some(name),
            Self::McpResourceRead { uri, .. } => Some(uri),
            _ => None,
        }
    }
}

impl ResourceRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != RESOURCE_REQUEST_SCHEMA || !is_digest(&self.configuration_digest) {
            return Err("resource_request_contract_invalid".into());
        }
        if let Some(path) = self.action.path() {
            if normalize_relative_path(path)? != path {
                return Err("resource_request_path_invalid".into());
            }
        }
        if self.action.name().is_some_and(|name| !bounded_name(name)) {
            return Err("resource_request_name_invalid".into());
        }
        match &self.action {
            ResourceAction::ContentRead { admission_id }
                if !admission_id.starts_with("case-content:") || !bounded_name(admission_id) =>
            {
                return Err("content_read_exact_admission_required".into());
            }
            ResourceAction::FilesystemSearch { needle, .. }
                if needle.is_empty() || needle.len() > 256 =>
            {
                return Err("resource_search_not_bounded".into());
            }
            ResourceAction::AdmitContent {
                candidate_digest, ..
            } if !is_digest(candidate_digest) => {
                return Err("discovery_candidate_digest_required".into());
            }
            ResourceAction::McpResourceRead { catalog_digest, .. }
            | ResourceAction::McpToolCall { catalog_digest, .. }
                if !is_digest(catalog_digest) =>
            {
                return Err("mcp_catalog_identity_required".into());
            }
            _ => {}
        }
        if let ResourceAction::McpToolCall { arguments, .. } = &self.action {
            if !arguments.is_object()
                || serde_json::to_vec(arguments)
                    .map_err(|e| e.to_string())?
                    .len()
                    > 8192
            {
                return Err("mcp_arguments_not_bounded_object".into());
            }
        }
        Ok(())
    }

    pub fn digest(&self) -> String {
        digest_bytes(&serde_json::to_vec(self).expect("typed resource request serializes"))
    }
}

fn unique<T: Ord>(values: &[T]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

fn bounded_name(value: &str) -> bool {
    !value.is_empty() && value.len() <= 1024 && !value.chars().any(char::is_control)
}

pub(crate) fn is_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);

    struct Scratch(std::path::PathBuf);
    impl Scratch {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "yai-resource-access-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn root(&self) -> super::super::LocalFilesystemBinding {
            super::super::LocalFilesystemBinding::new("case:access", "resource:workspace", &self.0)
                .unwrap()
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn envelope() -> ResourceAccessContract {
        ResourceAccessContract {
            schema: RESOURCE_ACCESS_SCHEMA.into(),
            configuration_digest: digest_bytes(b"configuration"),
            participant_ids: vec!["participant:model".into()],
            operations: vec![AccessKind::FilesystemRead],
            read_prefixes: vec!["source".into()],
            names: Vec::new(),
            max_output_bytes: 4096,
            max_items: 16,
        }
    }
    fn read_request(path: &str) -> ResourceRequest {
        ResourceRequest {
            schema: RESOURCE_REQUEST_SCHEMA.into(),
            configuration_digest: digest_bytes(b"configuration"),
            action: ResourceAction::FilesystemRead { path: path.into() },
        }
    }

    #[test]
    fn access_envelope_is_scoped_bounded_and_not_authority() {
        let value = envelope();
        assert!(value
            .admits_request("participant:model", &read_request("source/code.rs"))
            .is_ok());
        assert!(value
            .admits_request("participant:other", &read_request("source/code.rs"))
            .is_err());
        assert!(value
            .admits_request("participant:model", &read_request("source-secret/key"))
            .is_err());
        assert!(value
            .admits_request("participant:model", &read_request("source/../secrets"))
            .is_err());
        let mut request = read_request("source/code.rs");
        request.configuration_digest = digest_bytes(b"different");
        assert!(value
            .admits_request("participant:model", &request)
            .unwrap_err()
            .contains("stale"));
        let mut invalid = value.clone();
        invalid.max_output_bytes = MAX_ACCESS_BYTES + 1;
        assert!(invalid.validate().is_err());
        invalid = value;
        invalid.participant_ids.push("participant:model".into());
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn resource_request_shapes_fail_closed_without_name_inference() {
        for action in [
            AccessKind::ProcessRun,
            AccessKind::DatabaseMutation,
            AccessKind::McpToolCall,
        ] {
            assert!(action.is_external_effect());
        }
        assert!(!AccessKind::FilesystemRead.is_external_effect());
        let request = ResourceRequest {
            schema: RESOURCE_REQUEST_SCHEMA.into(),
            configuration_digest: digest_bytes(b"configuration"),
            action: ResourceAction::McpToolCall {
                name: "safe_read_only_whisper".into(),
                catalog_digest: digest_bytes(b"catalog"),
                arguments: serde_json::json!({}),
            },
        };
        request.validate().unwrap();
        assert!(
            request.action.kind().is_external_effect(),
            "names and MCP annotations cannot downgrade a tool effect"
        );
        let mut unknown = serde_json::to_value(&request).unwrap();
        unknown["action"]["action"] = serde_json::json!("arbitrary_shell");
        assert!(serde_json::from_value::<ResourceRequest>(unknown).is_err());
        let mut extra = serde_json::to_value(&request).unwrap();
        extra["authorized"] = serde_json::json!(true);
        assert!(serde_json::from_value::<ResourceRequest>(extra).is_err());
    }

    #[test]
    fn confined_read_rejects_symlink_fifo_bound_and_replaced_root() {
        use std::os::unix::fs::symlink;
        let temp = Scratch::new();
        std::fs::create_dir(temp.0.join("source")).unwrap();
        std::fs::write(temp.0.join("source/code"), b"bounded evidence").unwrap();
        let root = temp.root();
        assert_eq!(
            read_confined_file(&root, "source/code", 128).unwrap(),
            b"bounded evidence"
        );
        assert!(read_confined_file(&root, "source/code", 2).is_err());
        symlink("code", temp.0.join("source/link")).unwrap();
        assert!(read_confined_file(&root, "source/link", 128).is_err());
        assert!(read_confined_file(&root, "source", 128).is_err());
        let fifo = std::ffi::CString::new(temp.0.join("source/fifo").to_str().unwrap()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
        assert!(read_confined_file(&root, "source/fifo", 128).is_err());
        let mut stale = root;
        stale.root_inode += 1;
        assert!(read_confined_file(&stale, "source/code", 128).is_err());
    }

    #[test]
    fn sqlite_adapter_uses_real_engine_and_refuses_mutation_and_ambient_access() {
        let temp = Scratch::new();
        let db_path = temp.0.join("release.sqlite");
        let db = rusqlite::Connection::open(&db_path).unwrap();
        db.execute_batch("CREATE TABLE release(channel TEXT, maximum_ms INTEGER); INSERT INTO release VALUES ('stable', 800);").unwrap();
        drop(db);
        let initial = std::fs::read(&db_path).unwrap();
        let binding = LocalAccessBinding {
            schema: LOCAL_ACCESS_BINDING_SCHEMA.into(),
            case_id: "case:access".into(),
            attachment_id: "resource:workspace".into(),
            address: ResourceAddress::Sqlite {
                root: temp.root(),
                path: "release.sqlite".into(),
                queries: BTreeMap::from([
                    (
                        "release".into(),
                        "SELECT channel, maximum_ms FROM release".into(),
                    ),
                    ("mutation".into(), "DELETE FROM release".into()),
                    (
                        "ambient".into(),
                        "ATTACH DATABASE '/tmp/not-an-admitted-database' AS external".into(),
                    ),
                ]),
            },
        };
        let result = query_sqlite_snapshot(&binding, "release", 8, 4096).unwrap();
        assert_eq!(
            result["columns"],
            serde_json::json!(["channel", "maximum_ms"])
        );
        assert_eq!(result["rows"], serde_json::json!([["stable", 800]]));
        assert_eq!(result["source_digest"], digest_bytes(&initial));
        assert!(query_sqlite_snapshot(&binding, "mutation", 8, 4096).is_err());
        assert!(query_sqlite_snapshot(&binding, "ambient", 8, 4096).is_err());
        assert!(query_sqlite_snapshot(&binding, "not-bound", 8, 4096).is_err());
        assert_eq!(std::fs::read(&db_path).unwrap(), initial);
        std::fs::write(temp.0.join("release.sqlite-wal"), b"external state").unwrap();
        assert!(query_sqlite_snapshot(&binding, "release", 8, 4096)
            .unwrap_err()
            .contains("sidecar"));
    }

    #[test]
    fn discovery_and_search_are_bounded_candidates_not_admissions() {
        let temp = Scratch::new();
        std::fs::create_dir(temp.0.join("notes")).unwrap();
        std::fs::write(temp.0.join("notes/z.txt"), b"unrelated\n").unwrap();
        std::fs::write(temp.0.join("notes/a.txt"), b"release constraint\n").unwrap();
        let root = temp.root();
        let found = inspect_confined_tree(&root, "notes", None, 4096, 16).unwrap();
        assert_eq!(found["entries"][0]["path"], "notes/a.txt");
        assert_eq!(found["entries"][0]["admitted"], false);
        assert_eq!(
            found["entries"][0]["digest"],
            digest_bytes(b"release constraint\n")
        );
        let matches = inspect_confined_tree(&root, "notes", Some("release"), 4096, 16).unwrap();
        assert_eq!(matches["entries"].as_array().unwrap().len(), 1);
        assert_eq!(matches["entries"][0]["matches"][0]["line"], 1);
        assert!(inspect_confined_tree(&root, "notes", None, 4096, 1).is_err());
        assert!(inspect_confined_tree(&root, "../notes", None, 4096, 16).is_err());
        std::fs::write(temp.0.join("notes/a.txt"), b"changed constraint\n").unwrap();
        assert_ne!(
            digest_bytes(&read_confined_file(&root, "notes/a.txt", 4096).unwrap()),
            found["entries"][0]["digest"]
        );
        std::os::unix::fs::symlink("/tmp", temp.0.join("notes/link")).unwrap();
        assert!(inspect_confined_tree(&root, "notes", None, 4096, 16).is_err());
    }

    #[test]
    fn network_resource_contract_refuses_ambient_addressing_and_header_injection() {
        let valid = NetworkResourceAddress {
            endpoint: "http://127.0.0.1:8000/release".into(),
            allowed_ip_addresses: vec!["127.0.0.1".into()],
            credential_ref: None,
        };
        valid.validate().unwrap();
        for endpoint in [
            "http://user:secret@host/path",
            "file:///etc/passwd",
            "http://host/../escape",
            "http://host/%2e%2e/escape",
            "http://host/path\r\nInjected: yes",
        ] {
            let mut value = valid.clone();
            value.endpoint = endpoint.into();
            assert!(value.validate().is_err());
        }
    }
}
