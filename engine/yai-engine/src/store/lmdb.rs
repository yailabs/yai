//! YAI - LMDB record store
//!
//! Purpose:
//!   Provide durable indexed record lookup for normalized record envelopes.
//!
//! Ownership:
//!   LMDB environment opening, record writes, summary counts and record index
//!   reads.
//!
//! Boundary:
//!   Does not own journal replay/audit truth, hot-state freshness, graph truth
//!   or analytical facts.
//!
//! Status:
//!   active

use crate::compatibility::{
    decode_legacy_record, inspect_legacy_jsonl, LegacyDecodeOutcome, LegacyRecord,
};
use crate::context::SemanticContextArtifact;
use crate::effect::{LocalFilesystemBinding, OperationOrigin, LOCAL_FILESYSTEM_BINDING_SCHEMA};
use crate::journal::Journal;
use crate::record::Record;
use crate::transition::{
    replay_case, CaseState, PendingTransition, Transition, TransitionPayload, CASE_STATE_SCHEMA,
    CASE_STATE_SCHEMA_V1, CASE_STATE_SCHEMA_V2, TRANSITION_SCHEMA, TRANSITION_SCHEMA_V1,
    TRANSITION_SCHEMA_V2,
};
use lmdb::{
    Cursor, Database, DatabaseFlags, Environment, EnvironmentFlags, Error, RoTransaction,
    RwTransaction, Transaction, WriteFlags,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAP_SIZE: usize = 16 * 1024 * 1024;
pub const RECORD_SCHEMA: &str = "yai.record.v1";
pub const GRAPH_RELATION_SCHEMA: &str = "yai.graph_relation.v1";
pub const GRAPH_RELATION_STORE_NAME: &str = "lmdb_graph_relations_v0";
pub const CANONICAL_AUTHORITY_BACKEND: &str = "lmdb_transaction_authority_v1";
pub const LEGACY_COMPATIBILITY_SCHEMA: &str = "yai.legacy.compatibility.v1";
pub const SEMANTIC_CONTEXT_ARTIFACT_SCHEMA: &str = "yai.semantic_context_artifact.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordStoreStatusKind {
    Missing,
    NotInitialized,
    Ready,
    Unavailable,
}

impl RecordStoreStatusKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::NotInitialized => "not_initialized",
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordStoreStatus {
    pub path: PathBuf,
    pub backend: &'static str,
    pub status: RecordStoreStatusKind,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordStoreSummary {
    pub records_total: usize,
    pub records_by_case: usize,
    pub records_by_kind: usize,
    pub records_by_subject: usize,
    pub records_by_receipt: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CanonicalAuthoritySummary {
    pub transitions_total: usize,
    pub cases_materialized: usize,
    pub legacy_compatibility_payloads: usize,
}

pub struct LmdbRecordStore {
    env: Environment,
    records_by_id: Database,
    records_by_case: Database,
    records_by_kind: Database,
    records_by_subject: Database,
    records_by_receipt: Database,
    graph_relations_by_id: Database,
    graph_relations_by_case: Database,
    graph_relations_by_kind: Database,
    transitions_by_id: Database,
    case_transition_sequence: Database,
    case_state: Database,
    legacy_compatibility_payloads: Database,
    local_resource_bindings: Database,
    semantic_context_artifacts: Database,
    schema_meta: Database,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalCommit {
    pub transition: Transition,
    pub state: CaseState,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LegacyCompatibilityImportReport {
    pub lines_total: usize,
    pub losslessly_promoted: usize,
    pub promoted_with_metadata: usize,
    pub preserved_opaque: usize,
    pub rejected_malformed: usize,
    pub repeated_record_ids: usize,
    pub payloads_written: usize,
    pub payloads_duplicate: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredRecordEnvelope {
    pub raw_json: String,
    pub schema: String,
    pub record_id: String,
    pub record_kind: String,
    pub case_ref: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordListResult {
    pub records_total: usize,
    pub records: Vec<StoredRecordEnvelope>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphRelation {
    pub relation_id: String,
    pub case_ref: String,
    pub from_ref: String,
    pub to_ref: String,
    pub edge_kind: String,
    pub from_kind: String,
    pub to_kind: String,
    pub source_record_id: String,
    pub source_record_kind: String,
    pub confidence: String,
    pub created_at_unix_ms: u128,
    pub provenance: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphRelationListResult {
    pub relations_total: usize,
    pub relations: Vec<GraphRelation>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphMaterializeReport {
    pub relations_seen: usize,
    pub relations_written: usize,
    pub relations_duplicate: usize,
    pub relations_skipped: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeGraphNode {
    pub node_ref: String,
    pub node_kind: String,
    pub case_ref: String,
    pub source_record_ref: String,
    pub generation: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeGraphEdge {
    pub relation_id: String,
    pub edge_kind: String,
    pub from_ref: String,
    pub to_ref: String,
    pub case_ref: String,
    pub source_record_id: String,
    pub generation: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RuntimeGraphLoadResult {
    pub case_ref: String,
    pub nodes_total: usize,
    pub edges_total: usize,
    pub outgoing_index_entries: usize,
    pub incoming_index_entries: usize,
    pub generation: usize,
    pub dirty: bool,
    pub stale: bool,
    pub source: &'static str,
    pub durable_truth: &'static str,
    pub nodes: Vec<RuntimeGraphNode>,
    pub edges: Vec<RuntimeGraphEdge>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JournalImportReport {
    pub records_seen: usize,
    pub records_written: usize,
    pub records_duplicate: usize,
    pub records_skipped: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReplayMetadata {
    pub replay_id: String,
    pub journal_identity: String,
    pub journal_path: String,
    pub record_schema: String,
    pub journal_schema: String,
    pub started_at: String,
    pub completed_at: String,
    pub lines_total: usize,
    pub lines_replayed: usize,
    pub records_written: usize,
    pub records_duplicate: usize,
    pub records_skipped: usize,
    pub invalid_entries: usize,
    pub unsupported_entries: usize,
    pub cursor_line: usize,
    pub status: String,
    pub compatibility: String,
}

impl LmdbRecordStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        fs::create_dir_all(path)
            .map_err(|error| format!("failed to create {}: {error}", path.display()))?;
        let env = Environment::new()
            .set_max_dbs(18)
            .set_map_size(MAP_SIZE)
            .open(path)
            .map_err(|error| format!("failed to open LMDB env {}: {error}", path.display()))?;
        let records_by_id = env
            .create_db(Some("records_by_id"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open records_by_id: {error}"))?;
        let records_by_case = env
            .create_db(Some("records_by_case"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open records_by_case: {error}"))?;
        let records_by_kind = env
            .create_db(Some("records_by_kind"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open records_by_kind: {error}"))?;
        let records_by_subject = env
            .create_db(Some("records_by_subject"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open records_by_subject: {error}"))?;
        let records_by_receipt = env
            .create_db(Some("records_by_receipt"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open records_by_receipt: {error}"))?;
        let graph_relations_by_id = env
            .create_db(Some("graph_relations_by_id"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open graph_relations_by_id: {error}"))?;
        let graph_relations_by_case = env
            .create_db(Some("graph_relations_by_case"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open graph_relations_by_case: {error}"))?;
        let graph_relations_by_kind = env
            .create_db(Some("graph_relations_by_kind"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open graph_relations_by_kind: {error}"))?;
        let transitions_by_id = env
            .create_db(Some("transitions_by_id"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open transitions_by_id: {error}"))?;
        let case_transition_sequence = env
            .create_db(Some("case_transition_sequence"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open case_transition_sequence: {error}"))?;
        let case_state = env
            .create_db(Some("case_state"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open case_state: {error}"))?;
        let legacy_compatibility_payloads = env
            .create_db(
                Some("legacy_compatibility_payloads"),
                DatabaseFlags::empty(),
            )
            .map_err(|error| format!("failed to open legacy_compatibility_payloads: {error}"))?;
        let local_resource_bindings = env
            .create_db(Some("local_resource_bindings"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open local_resource_bindings: {error}"))?;
        let semantic_context_artifacts = env
            .create_db(Some("semantic_context_artifacts"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open semantic_context_artifacts: {error}"))?;
        let schema_meta = env
            .create_db(Some("schema_meta"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open schema_meta: {error}"))?;
        let store = Self {
            env,
            records_by_id,
            records_by_case,
            records_by_kind,
            records_by_subject,
            records_by_receipt,
            graph_relations_by_id,
            graph_relations_by_case,
            graph_relations_by_kind,
            transitions_by_id,
            case_transition_sequence,
            case_state,
            legacy_compatibility_payloads,
            local_resource_bindings,
            semantic_context_artifacts,
            schema_meta,
        };
        store.ensure_schema()?;
        Ok(store)
    }

    pub fn status(path: impl AsRef<Path>) -> RecordStoreStatus {
        let path = path.as_ref().to_path_buf();
        let status = if !path.exists() {
            RecordStoreStatusKind::Missing
        } else if !path.is_dir() {
            RecordStoreStatusKind::Unavailable
        } else if !path.join("data.mdb").exists() {
            RecordStoreStatusKind::NotInitialized
        } else {
            match Self::schema_ready(&path) {
                Ok(true) => RecordStoreStatusKind::Ready,
                Ok(false) => RecordStoreStatusKind::NotInitialized,
                Err(()) => RecordStoreStatusKind::Unavailable,
            }
        };
        RecordStoreStatus {
            path,
            backend: "lmdb",
            status,
        }
    }

    pub fn append_record(&self, record: &Record, source_ref: &str) -> Result<(), String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start LMDB write transaction: {error}"))?;
        self.put_record(&mut txn, record, source_ref)?;
        txn.commit()
            .map_err(|error| format!("failed to commit LMDB record write: {error}"))
    }

    /// Stores a bounded, rebuildable semantic derivation for inspection and
    /// invocation lineage. This database is derived and is never consulted by
    /// canonical replay or CaseState reduction.
    pub fn put_semantic_context_artifact(
        &self,
        artifact: &SemanticContextArtifact,
    ) -> Result<(), String> {
        let key = semantic_context_artifact_key(artifact.id());
        let value = serde_json::to_string(artifact)
            .map_err(|error| format!("semantic_context_artifact_encode_failed: {error}"))?;
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start semantic context write: {error}"))?;
        txn.put(
            self.semantic_context_artifacts,
            &key,
            &value,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to store semantic context artifact: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit semantic context artifact: {error}"))
    }

    pub fn get_semantic_context_artifact(
        &self,
        artifact_id: &str,
    ) -> Result<Option<SemanticContextArtifact>, String> {
        let key = semantic_context_artifact_key(artifact_id);
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start semantic context read: {error}"))?;
        match txn.get(self.semantic_context_artifacts, &key) {
            Ok(value) => serde_json::from_slice(value)
                .map(Some)
                .map_err(|error| format!("semantic_context_artifact_decode_failed: {error}")),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read semantic context artifact: {error}")),
        }
    }

    /// Removes only derived Projection/ContextFrame/render metadata. Canonical
    /// Transition history and CaseState are held in different databases.
    pub fn clear_semantic_context_artifacts(&self) -> Result<(), String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start semantic context clear: {error}"))?;
        txn.clear_db(self.semantic_context_artifacts)
            .map_err(|error| format!("failed to clear semantic context artifacts: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit semantic context clear: {error}"))
    }

    /// Atomically appends one immutable canonical Transition and replaces the
    /// corresponding rebuildable CaseState materialization.
    pub fn commit_transition(&self, pending: PendingTransition) -> Result<CanonicalCommit, String> {
        self.commit_transition_inner(pending, false)
    }

    fn commit_transition_inner(
        &self,
        pending: PendingTransition,
        inject_failure_before_commit: bool,
    ) -> Result<CanonicalCommit, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start canonical write transaction: {error}"))?;
        let transition_key = transition_id_key(&pending.transition_id);
        match txn.get(self.transitions_by_id, &transition_key) {
            Ok(_) => {
                return Err(format!(
                    "duplicate_transition_id: {}",
                    pending.transition_id
                ))
            }
            Err(Error::NotFound) => {}
            Err(error) => {
                return Err(format!(
                    "failed to check transition identity {}: {error}",
                    pending.transition_id
                ));
            }
        }

        let current_state = self.get_case_state_txn(&txn, &pending.case_id)?;
        let actual_generation = current_state
            .as_ref()
            .map(|state| state.generation)
            .unwrap_or(0);
        if pending.expected_generation != actual_generation {
            return Err(format!(
                "stale_case_generation: expected={} actual={actual_generation}",
                pending.expected_generation
            ));
        }
        let sequence = actual_generation + 1;
        let transition = Transition {
            schema: TRANSITION_SCHEMA.to_string(),
            transition_id: pending.transition_id,
            case_id: pending.case_id,
            sequence,
            committed_at_unix_ms: unix_time_ms() as u64,
            source: pending.source,
            scope: pending.scope,
            causal_refs: pending.causal_refs,
            payload: pending.payload,
            provenance: pending.provenance,
            summary: pending.summary,
        };
        transition.validate()?;

        let next_state = if let Some(state) = current_state {
            state.reduce(&transition)?
        } else {
            let crate::transition::TransitionPayload::CaseOpened { lifecycle } =
                &transition.payload
            else {
                return Err("case_history_must_start_with_case_opened".to_string());
            };
            CaseState::new(&transition.case_id, lifecycle.clone()).reduce(&transition)?
        };
        let transition_json = transition.to_json()?;
        let state_json = next_state.to_json()?;
        let sequence_key = case_sequence_key(&transition.case_id, transition.sequence);
        let state_key = case_state_key(&transition.case_id);

        txn.put(
            self.transitions_by_id,
            &transition_key,
            &transition_json,
            WriteFlags::NO_OVERWRITE,
        )
        .map_err(|error| format!("failed to append canonical Transition: {error}"))?;
        txn.put(
            self.case_transition_sequence,
            &sequence_key,
            &transition.transition_id,
            WriteFlags::NO_OVERWRITE,
        )
        .map_err(|error| format!("failed to append Case transition sequence: {error}"))?;
        txn.put(
            self.case_state,
            &state_key,
            &state_json,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to materialize CaseState: {error}"))?;

        if inject_failure_before_commit {
            return Err("injected_failure_before_canonical_commit".to_string());
        }
        txn.commit()
            .map_err(|error| format!("failed to commit canonical transaction: {error}"))?;
        Ok(CanonicalCommit {
            transition,
            state: next_state,
        })
    }

    pub fn get_transition_by_id(&self, transition_id: &str) -> Result<Option<Transition>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start canonical transition read: {error}"))?;
        self.get_transition_by_id_txn(&txn, transition_id)
    }

    pub fn list_case_transitions(&self, case_id: &str) -> Result<Vec<Transition>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start Case transition read: {error}"))?;
        self.list_case_transitions_txn(&txn, case_id)
    }

    pub fn get_case_state(&self, case_id: &str) -> Result<Option<CaseState>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start CaseState read: {error}"))?;
        self.get_case_state_txn(&txn, case_id)
    }

    /// Persists a machine-local resource binding. This database is required
    /// for carrier resolution after restart but is not canonical Case history.
    pub fn put_local_filesystem_binding(
        &self,
        binding: &LocalFilesystemBinding,
    ) -> Result<(), String> {
        binding.validate()?;
        let key = local_binding_key(&binding.case_id, &binding.attachment_id);
        let value = serde_json::to_string(binding)
            .map_err(|error| format!("local_binding_encode_failed: {error}"))?;
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start local binding write: {error}"))?;
        txn.put(
            self.local_resource_bindings,
            &key,
            &value,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to persist local resource binding: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit local resource binding: {error}"))
    }

    pub fn get_local_filesystem_binding(
        &self,
        case_id: &str,
        attachment_id: &str,
    ) -> Result<Option<LocalFilesystemBinding>, String> {
        let key = local_binding_key(case_id, attachment_id);
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start local binding read: {error}"))?;
        match txn.get(self.local_resource_bindings, &key) {
            Ok(value) => {
                let binding: LocalFilesystemBinding = serde_json::from_slice(value)
                    .map_err(|error| format!("local_binding_decode_failed: {error}"))?;
                binding.validate()?;
                Ok(Some(binding))
            }
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read local resource binding: {error}")),
        }
    }

    pub fn replay_case_state(&self, case_id: &str) -> Result<CaseState, String> {
        let transitions = self.list_case_transitions(case_id)?;
        replay_case(case_id, &transitions)
    }

    pub fn verify_case_state(&self, case_id: &str) -> Result<bool, String> {
        let materialized = self
            .get_case_state(case_id)?
            .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
        Ok(materialized == self.replay_case_state(case_id)?)
    }

    pub fn rebuild_case_state(&self, case_id: &str) -> Result<CaseState, String> {
        let transitions = self.list_case_transitions(case_id)?;
        let rebuilt = replay_case(case_id, &transitions)?;
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start CaseState rebuild: {error}"))?;
        let actual_sequence = self.last_case_sequence_txn(&txn, case_id)?;
        if actual_sequence != rebuilt.generation {
            return Err(format!(
                "case_history_changed_during_rebuild: expected={} actual={actual_sequence}",
                rebuilt.generation
            ));
        }
        let state_key = case_state_key(case_id);
        let state_json = rebuilt.to_json()?;
        txn.put(
            self.case_state,
            &state_key,
            &state_json,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to replace rebuilt CaseState: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit rebuilt CaseState: {error}"))?;
        Ok(rebuilt)
    }

    /// Imports legacy bytes as compatibility evidence only. This never appends
    /// a canonical Transition and never materializes CaseState.
    pub fn import_legacy_compatibility(
        &self,
        contents: &str,
        source_ref: &str,
    ) -> Result<LegacyCompatibilityImportReport, String> {
        let corpus = inspect_legacy_jsonl(contents);
        let mut report = LegacyCompatibilityImportReport {
            lines_total: corpus.lines_total,
            losslessly_promoted: corpus.losslessly_promoted,
            promoted_with_metadata: corpus.promoted_with_metadata,
            preserved_opaque: corpus.preserved_opaque,
            rejected_malformed: corpus.rejected_malformed,
            repeated_record_ids: corpus.repeated_record_ids,
            ..Default::default()
        };
        let source_key = relation_id_component(source_ref);
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start legacy compatibility import: {error}"))?;
        for entry in corpus
            .entries
            .iter()
            .filter(|entry| entry.disposition != "rejected_malformed")
        {
            let key = format!("legacy:{source_key}:{:020}", entry.line_number);
            let value = serde_json::json!({
                "schema": LEGACY_COMPATIBILITY_SCHEMA,
                "source_ref": source_ref,
                "line_number": entry.line_number,
                "disposition": entry.disposition,
                "origin_schema": entry.schema,
                "record_id": entry.record_id,
                "record_kind": entry.record_kind,
                "reason": entry.reason,
                "raw_json": entry.raw_json,
            })
            .to_string();
            match txn.put(
                self.legacy_compatibility_payloads,
                &key,
                &value,
                WriteFlags::NO_OVERWRITE,
            ) {
                Ok(()) => report.payloads_written += 1,
                Err(Error::KeyExist) => report.payloads_duplicate += 1,
                Err(error) => {
                    return Err(format!(
                        "failed to preserve legacy compatibility line {}: {error}",
                        entry.line_number
                    ));
                }
            }
        }
        txn.commit()
            .map_err(|error| format!("failed to commit legacy compatibility import: {error}"))?;
        Ok(report)
    }

    pub fn legacy_compatibility_payload_count(&self) -> Result<usize, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to read legacy compatibility store: {error}"))?;
        count_entries(&txn, self.legacy_compatibility_payloads)
    }

    pub fn import_journal(&self, journal: &Journal, journal_ref: &str) -> Result<(), String> {
        self.import_journal_with_report(journal, journal_ref)
            .map(|_| ())
    }

    pub fn import_journal_with_report(
        &self,
        journal: &Journal,
        journal_ref: &str,
    ) -> Result<JournalImportReport, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start LMDB journal import: {error}"))?;
        let mut report = JournalImportReport::default();
        for (index, record) in journal.records().iter().enumerate() {
            report.records_seen += 1;
            let source_ref = format!("{journal_ref}#{}", index + 1);
            if self.record_exists_txn(&txn, &record.id)? {
                report.records_duplicate += 1;
                continue;
            }
            self.put_record(&mut txn, record, &source_ref)?;
            report.records_written += 1;
        }
        txn.commit()
            .map_err(|error| format!("failed to commit LMDB journal import: {error}"))?;
        Ok(report)
    }

    pub fn put_replay_metadata(&self, metadata: &ReplayMetadata) -> Result<(), String> {
        let key = replay_metadata_key(&metadata.journal_identity);
        let value = metadata.to_json();
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start LMDB replay metadata write: {error}"))?;
        txn.put(self.schema_meta, &key, &value, WriteFlags::empty())
            .map_err(|error| {
                format!(
                    "failed to write replay metadata for {}: {error}",
                    metadata.journal_identity
                )
            })?;
        txn.commit()
            .map_err(|error| format!("failed to commit LMDB replay metadata: {error}"))
    }

    pub fn replay_metadata(
        &self,
        journal_identity: &str,
    ) -> Result<Option<ReplayMetadata>, String> {
        let key = replay_metadata_key(journal_identity);
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB replay metadata read: {error}"))?;
        match txn.get(self.schema_meta, &key) {
            Ok(value) => ReplayMetadata::from_bytes(value).map(Some),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!(
                "failed to read replay metadata for {journal_identity}: {error}"
            )),
        }
    }

    pub fn summary(&self) -> Result<RecordStoreSummary, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB read transaction: {error}"))?;
        Ok(RecordStoreSummary {
            records_total: count_entries(&txn, self.records_by_id)?,
            records_by_case: count_entries(&txn, self.records_by_case)?,
            records_by_kind: count_entries(&txn, self.records_by_kind)?,
            records_by_subject: count_entries(&txn, self.records_by_subject)?,
            records_by_receipt: count_entries(&txn, self.records_by_receipt)?,
        })
    }

    pub fn canonical_summary(&self) -> Result<CanonicalAuthoritySummary, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start canonical summary read: {error}"))?;
        Ok(CanonicalAuthoritySummary {
            transitions_total: count_entries(&txn, self.transitions_by_id)?,
            cases_materialized: count_entries(&txn, self.case_state)?,
            legacy_compatibility_payloads: count_entries(&txn, self.legacy_compatibility_payloads)?,
        })
    }

    pub fn get_record_by_id(
        &self,
        record_id: &str,
    ) -> Result<Option<StoredRecordEnvelope>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB record read: {error}"))?;
        self.get_record_by_id_txn(&txn, record_id)
    }

    pub fn list_records_by_case(
        &self,
        case_ref: &str,
        limit: usize,
    ) -> Result<RecordListResult, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB case index read: {error}"))?;
        let prefix = format!("record:case:{case_ref}:");
        self.list_records_by_index(&txn, self.records_by_case, &prefix, limit)
    }

    pub fn list_records_by_kind(
        &self,
        record_kind: &str,
        limit: usize,
    ) -> Result<RecordListResult, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB kind index read: {error}"))?;
        let prefix = format!("record:kind:{record_kind}:");
        self.list_records_by_index(&txn, self.records_by_kind, &prefix, limit)
    }

    pub fn list_records_by_subject(
        &self,
        subject_ref: &str,
        limit: usize,
    ) -> Result<RecordListResult, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB subject index read: {error}"))?;
        let prefix = format!("record:subject:{subject_ref}:");
        self.list_records_by_index(&txn, self.records_by_subject, &prefix, limit)
    }

    pub fn list_records_by_receipt(
        &self,
        receipt_ref: &str,
        limit: usize,
    ) -> Result<RecordListResult, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB receipt index read: {error}"))?;
        let prefix = format!("record:receipt:{receipt_ref}:");
        self.list_records_by_index(&txn, self.records_by_receipt, &prefix, limit)
    }

    pub fn materialize_graph_relations_for_case(
        &self,
        case_ref: &str,
    ) -> Result<GraphMaterializeReport, String> {
        self.materialize_graph_relations_for_case_inner(case_ref, false)
    }

    pub fn rebuild_graph_relations_for_case(
        &self,
        case_ref: &str,
    ) -> Result<GraphMaterializeReport, String> {
        self.clear_graph_relations_for_case(case_ref)?;
        self.materialize_graph_relations_for_case(case_ref)
    }

    fn clear_graph_relations_for_case(&self, case_ref: &str) -> Result<usize, String> {
        let prefix = format!("graph_relation:case:{case_ref}:");
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start graph relation clear: {error}"))?;
        let mut cursor = txn
            .open_ro_cursor(self.graph_relations_by_case)
            .map_err(|error| format!("failed to open graph relation clear cursor: {error}"))?;
        let mut entries = Vec::new();
        for (key, value) in cursor.iter() {
            if !key.starts_with(prefix.as_bytes()) {
                continue;
            }
            let relation_id = std::str::from_utf8(value)
                .map_err(|error| format!("invalid graph relation identity utf8: {error}"))?;
            entries.push((key.to_vec(), relation_id.to_string()));
        }
        drop(cursor);
        for (case_key, relation_id) in &entries {
            let id_key = format!("graph_relation:id:{relation_id}");
            let relation = match txn.get(self.graph_relations_by_id, &id_key) {
                Ok(value) => Some(GraphRelation::from_bytes(value)?),
                Err(Error::NotFound) => None,
                Err(error) => {
                    return Err(format!(
                        "failed to inspect graph relation {relation_id} for clear: {error}"
                    ));
                }
            };
            txn.del(self.graph_relations_by_case, case_key, None)
                .map_err(|error| format!("failed to clear graph case index: {error}"))?;
            match txn.del(self.graph_relations_by_id, &id_key, None) {
                Ok(()) | Err(Error::NotFound) => {}
                Err(error) => return Err(format!("failed to clear graph relation: {error}")),
            }
            if let Some(relation) = relation {
                let kind_key = format!(
                    "graph_relation:kind:{}:{}",
                    relation.edge_kind, relation.relation_id
                );
                match txn.del(self.graph_relations_by_kind, &kind_key, None) {
                    Ok(()) | Err(Error::NotFound) => {}
                    Err(error) => {
                        return Err(format!("failed to clear graph kind index: {error}"));
                    }
                }
            }
        }
        txn.commit()
            .map_err(|error| format!("failed to commit graph relation clear: {error}"))?;
        Ok(entries.len())
    }

    fn materialize_graph_relations_for_case_inner(
        &self,
        case_ref: &str,
        inject_failure_before_commit: bool,
    ) -> Result<GraphMaterializeReport, String> {
        let source_transitions = self.list_case_transitions(case_ref)?;
        let source_records = self.list_records_by_case(case_ref, usize::MAX)?;
        let created_at_unix_ms = unix_time_ms();
        let mut candidates = Vec::new();
        let mut skipped = 0usize;
        for transition in &source_transitions {
            let mut derived = derive_graph_relations_from_transition(transition, &mut skipped);
            candidates.append(&mut derived);
        }
        for record in &source_records.records {
            let mut derived = derive_graph_relations(record, created_at_unix_ms, &mut skipped);
            candidates.append(&mut derived);
        }

        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start LMDB graph relation write: {error}"))?;
        let mut report = GraphMaterializeReport {
            relations_seen: candidates.len(),
            relations_skipped: skipped,
            ..Default::default()
        };
        for relation in candidates {
            if self.graph_relation_exists_txn(&txn, &relation.relation_id)? {
                report.relations_duplicate += 1;
                continue;
            }
            self.put_graph_relation(&mut txn, &relation)?;
            report.relations_written += 1;
        }
        if inject_failure_before_commit {
            return Err("injected_graph_materialization_failure".to_string());
        }
        txn.commit()
            .map_err(|error| format!("failed to commit LMDB graph relations: {error}"))?;
        Ok(report)
    }

    pub fn list_graph_relations_by_case(
        &self,
        case_ref: &str,
        limit: usize,
    ) -> Result<GraphRelationListResult, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB graph relation read: {error}"))?;
        let prefix = format!("graph_relation:case:{case_ref}:");
        self.list_graph_relations_by_index(&txn, self.graph_relations_by_case, &prefix, limit)
    }

    pub fn load_runtime_graph_for_case(
        &self,
        case_ref: &str,
    ) -> Result<RuntimeGraphLoadResult, String> {
        let relations = self.list_graph_relations_by_case(case_ref, usize::MAX)?;
        let generation = 1;
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut outgoing_refs: Vec<String> = Vec::new();
        let mut incoming_refs: Vec<String> = Vec::new();

        for relation in relations.relations {
            push_runtime_node(
                &mut nodes,
                RuntimeGraphNode {
                    node_ref: relation.from_ref.clone(),
                    node_kind: relation.from_kind.clone(),
                    case_ref: relation.case_ref.clone(),
                    source_record_ref: relation.source_record_id.clone(),
                    generation,
                },
            );
            push_runtime_node(
                &mut nodes,
                RuntimeGraphNode {
                    node_ref: relation.to_ref.clone(),
                    node_kind: relation.to_kind.clone(),
                    case_ref: relation.case_ref.clone(),
                    source_record_ref: relation.source_record_id.clone(),
                    generation,
                },
            );
            push_unique_string(&mut outgoing_refs, &relation.from_ref);
            push_unique_string(&mut incoming_refs, &relation.to_ref);
            edges.push(RuntimeGraphEdge {
                relation_id: relation.relation_id,
                edge_kind: relation.edge_kind,
                from_ref: relation.from_ref,
                to_ref: relation.to_ref,
                case_ref: relation.case_ref,
                source_record_id: relation.source_record_id,
                generation,
            });
        }

        Ok(RuntimeGraphLoadResult {
            case_ref: case_ref.to_string(),
            nodes_total: nodes.len(),
            edges_total: edges.len(),
            outgoing_index_entries: outgoing_refs.len(),
            incoming_index_entries: incoming_refs.len(),
            generation,
            dirty: false,
            stale: false,
            source: "graph_relations",
            durable_truth: "canonical_transition_or_legacy_compatibility_input",
            nodes,
            edges,
        })
    }

    fn ensure_schema(&self) -> Result<(), String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start LMDB schema transaction: {error}"))?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:canonical_transition_schema",
            TRANSITION_SCHEMA,
            &[TRANSITION_SCHEMA_V2, TRANSITION_SCHEMA_V1],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:case_state_schema",
            CASE_STATE_SCHEMA,
            &[CASE_STATE_SCHEMA_V2, CASE_STATE_SCHEMA_V1],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:legacy_compatibility_schema",
            LEGACY_COMPATIBILITY_SCHEMA,
            &[],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:local_filesystem_binding_schema",
            LOCAL_FILESYSTEM_BINDING_SCHEMA,
            &[],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:semantic_context_artifact_schema",
            SEMANTIC_CONTEXT_ARTIFACT_SCHEMA,
            &[],
        )?;
        for (key, value) in [
            ("meta:canonical_transition_schema", TRANSITION_SCHEMA),
            ("meta:case_state_schema", CASE_STATE_SCHEMA),
            (
                "meta:legacy_compatibility_schema",
                LEGACY_COMPATIBILITY_SCHEMA,
            ),
            (
                "meta:local_filesystem_binding_schema",
                LOCAL_FILESYSTEM_BINDING_SCHEMA,
            ),
            (
                "meta:semantic_context_artifact_schema",
                SEMANTIC_CONTEXT_ARTIFACT_SCHEMA,
            ),
        ] {
            txn.put(self.schema_meta, &key, &value, WriteFlags::empty())
                .map_err(|error| format!("failed to write persisted schema {key}: {error}"))?;
        }
        txn.put(
            self.schema_meta,
            &"meta:schema",
            &RECORD_SCHEMA,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write LMDB schema meta: {error}"))?;
        txn.put(
            self.schema_meta,
            &"meta:rebuild",
            &"not_started",
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write LMDB rebuild meta: {error}"))?;
        txn.put(
            self.schema_meta,
            &"meta:graph_relation_schema",
            &GRAPH_RELATION_SCHEMA,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write LMDB graph relation schema meta: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit LMDB schema meta: {error}"))
    }

    fn get_transition_by_id_txn<T: Transaction>(
        &self,
        txn: &T,
        transition_id: &str,
    ) -> Result<Option<Transition>, String> {
        let key = transition_id_key(transition_id);
        match txn.get(self.transitions_by_id, &key) {
            Ok(value) => {
                let json = std::str::from_utf8(value)
                    .map_err(|error| format!("invalid canonical Transition utf8: {error}"))?;
                Transition::from_json(json).map(Some)
            }
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!(
                "failed to read canonical Transition {transition_id}: {error}"
            )),
        }
    }

    fn list_case_transitions_txn<T: Transaction>(
        &self,
        txn: &T,
        case_id: &str,
    ) -> Result<Vec<Transition>, String> {
        let prefix = case_sequence_prefix(case_id);
        let mut cursor = txn
            .open_ro_cursor(self.case_transition_sequence)
            .map_err(|error| format!("failed to open Case sequence cursor: {error}"))?;
        let mut identities = Vec::new();
        for (key, value) in cursor.iter() {
            if key.starts_with(prefix.as_bytes()) {
                let transition_id = std::str::from_utf8(value)
                    .map_err(|error| format!("invalid Case sequence identity utf8: {error}"))?;
                identities.push((key.to_vec(), transition_id.to_string()));
            }
        }
        drop(cursor);
        identities.sort_by(|left, right| left.0.cmp(&right.0));
        let mut transitions = Vec::with_capacity(identities.len());
        for (_, transition_id) in identities {
            let transition = self
                .get_transition_by_id_txn(txn, &transition_id)?
                .ok_or_else(|| format!("case_sequence_dangling_transition: {transition_id}"))?;
            transitions.push(transition);
        }
        Ok(transitions)
    }

    fn get_case_state_txn<T: Transaction>(
        &self,
        txn: &T,
        case_id: &str,
    ) -> Result<Option<CaseState>, String> {
        let key = case_state_key(case_id);
        match txn.get(self.case_state, &key) {
            Ok(value) => {
                let json = std::str::from_utf8(value)
                    .map_err(|error| format!("invalid CaseState utf8: {error}"))?;
                CaseState::from_json(json).map(Some)
            }
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read CaseState {case_id}: {error}")),
        }
    }

    fn last_case_sequence_txn<T: Transaction>(
        &self,
        txn: &T,
        case_id: &str,
    ) -> Result<u64, String> {
        let transitions = self.list_case_transitions_txn(txn, case_id)?;
        Ok(transitions
            .last()
            .map(|transition| transition.sequence)
            .unwrap_or(0))
    }

    #[cfg(test)]
    fn discard_case_state_for_test(&self, case_id: &str) -> Result<(), String> {
        let key = case_state_key(case_id);
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start test CaseState deletion: {error}"))?;
        match txn.del(self.case_state, &key, None) {
            Ok(()) | Err(Error::NotFound) => {}
            Err(error) => return Err(format!("failed to delete test CaseState: {error}")),
        }
        txn.commit()
            .map_err(|error| format!("failed to commit test CaseState deletion: {error}"))
    }

    fn put_record(
        &self,
        txn: &mut RwTransaction<'_>,
        record: &Record,
        source_ref: &str,
    ) -> Result<(), String> {
        let id_key = format!("record:id:{}", record.id);
        let case_key = format!("record:case:{}:{}", record.case_ref, record.id);
        let kind_key = format!("record:kind:{}:{}", record.kind.as_str(), record.id);
        let value = record.to_record_plane_json(source_ref);
        txn.put(self.records_by_id, &id_key, &value, WriteFlags::empty())
            .map_err(|error| format!("failed to write records_by_id {}: {error}", record.id))?;
        txn.put(
            self.records_by_case,
            &case_key,
            &record.id,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write records_by_case {}: {error}", record.id))?;
        txn.put(
            self.records_by_kind,
            &kind_key,
            &record.id,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write records_by_kind {}: {error}", record.id))?;
        if !record.subject_ref.is_empty() && record.subject_ref != "subject:none" {
            let subject_key = format!("record:subject:{}:{}", record.subject_ref, record.id);
            txn.put(
                self.records_by_subject,
                &subject_key,
                &record.id,
                WriteFlags::empty(),
            )
            .map_err(|error| {
                format!("failed to write records_by_subject {}: {error}", record.id)
            })?;
        }
        if !record.receipt_id.is_empty() {
            let receipt_key = format!("record:receipt:{}:{}", record.receipt_id, record.id);
            txn.put(
                self.records_by_receipt,
                &receipt_key,
                &record.id,
                WriteFlags::empty(),
            )
            .map_err(|error| {
                format!("failed to write records_by_receipt {}: {error}", record.id)
            })?;
        }
        Ok(())
    }

    fn put_graph_relation(
        &self,
        txn: &mut RwTransaction<'_>,
        relation: &GraphRelation,
    ) -> Result<(), String> {
        let id_key = format!("graph_relation:id:{}", relation.relation_id);
        let case_key = format!(
            "graph_relation:case:{}:{}",
            relation.case_ref, relation.relation_id
        );
        let kind_key = format!(
            "graph_relation:kind:{}:{}",
            relation.edge_kind, relation.relation_id
        );
        let value = relation.to_json();
        txn.put(
            self.graph_relations_by_id,
            &id_key,
            &value,
            WriteFlags::empty(),
        )
        .map_err(|error| {
            format!(
                "failed to write graph_relations_by_id {}: {error}",
                relation.relation_id
            )
        })?;
        txn.put(
            self.schema_meta,
            &"meta:canonical_transition_schema",
            &TRANSITION_SCHEMA,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write canonical transition schema meta: {error}"))?;
        txn.put(
            self.schema_meta,
            &"meta:case_state_schema",
            &CASE_STATE_SCHEMA,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write CaseState schema meta: {error}"))?;
        txn.put(
            self.schema_meta,
            &"meta:canonical_authority_backend",
            &CANONICAL_AUTHORITY_BACKEND,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write canonical authority backend meta: {error}"))?;
        txn.put(
            self.schema_meta,
            &"meta:legacy_compatibility_schema",
            &LEGACY_COMPATIBILITY_SCHEMA,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write legacy compatibility schema meta: {error}"))?;
        txn.put(
            self.graph_relations_by_case,
            &case_key,
            &relation.relation_id,
            WriteFlags::empty(),
        )
        .map_err(|error| {
            format!(
                "failed to write graph_relations_by_case {}: {error}",
                relation.relation_id
            )
        })?;
        txn.put(
            self.graph_relations_by_kind,
            &kind_key,
            &relation.relation_id,
            WriteFlags::empty(),
        )
        .map_err(|error| {
            format!(
                "failed to write graph_relations_by_kind {}: {error}",
                relation.relation_id
            )
        })?;
        Ok(())
    }

    fn record_exists_txn(&self, txn: &RwTransaction<'_>, record_id: &str) -> Result<bool, String> {
        let id_key = format!("record:id:{record_id}");
        match txn.get(self.records_by_id, &id_key) {
            Ok(_) => Ok(true),
            Err(Error::NotFound) => Ok(false),
            Err(error) => Err(format!(
                "failed to check records_by_id {record_id}: {error}"
            )),
        }
    }

    fn graph_relation_exists_txn(
        &self,
        txn: &RwTransaction<'_>,
        relation_id: &str,
    ) -> Result<bool, String> {
        let id_key = format!("graph_relation:id:{relation_id}");
        match txn.get(self.graph_relations_by_id, &id_key) {
            Ok(_) => Ok(true),
            Err(Error::NotFound) => Ok(false),
            Err(error) => Err(format!(
                "failed to check graph_relations_by_id {relation_id}: {error}"
            )),
        }
    }

    fn get_record_by_id_txn(
        &self,
        txn: &RoTransaction<'_>,
        record_id: &str,
    ) -> Result<Option<StoredRecordEnvelope>, String> {
        let id_key = format!("record:id:{record_id}");
        match txn.get(self.records_by_id, &id_key) {
            Ok(value) => StoredRecordEnvelope::from_bytes(value).map(Some),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read records_by_id {record_id}: {error}")),
        }
    }

    fn get_graph_relation_by_id_txn(
        &self,
        txn: &RoTransaction<'_>,
        relation_id: &str,
    ) -> Result<Option<GraphRelation>, String> {
        let id_key = format!("graph_relation:id:{relation_id}");
        match txn.get(self.graph_relations_by_id, &id_key) {
            Ok(value) => GraphRelation::from_bytes(value).map(Some),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!(
                "failed to read graph_relations_by_id {relation_id}: {error}"
            )),
        }
    }

    fn list_records_by_index(
        &self,
        txn: &RoTransaction<'_>,
        db: Database,
        prefix: &str,
        limit: usize,
    ) -> Result<RecordListResult, String> {
        let mut cursor = txn
            .open_ro_cursor(db)
            .map_err(|error| format!("failed to open LMDB index cursor: {error}"))?;
        let mut result = RecordListResult::default();
        for (key, value) in cursor.iter() {
            if !key.starts_with(prefix.as_bytes()) {
                continue;
            }
            result.records_total += 1;
            if result.records.len() >= limit {
                continue;
            }
            let record_id = std::str::from_utf8(value)
                .map_err(|error| format!("invalid LMDB index record id: {error}"))?;
            if let Some(record) = self.get_record_by_id_txn(txn, record_id)? {
                result.records.push(record);
            }
        }
        Ok(result)
    }

    fn list_graph_relations_by_index(
        &self,
        txn: &RoTransaction<'_>,
        db: Database,
        prefix: &str,
        limit: usize,
    ) -> Result<GraphRelationListResult, String> {
        let mut cursor = txn
            .open_ro_cursor(db)
            .map_err(|error| format!("failed to open LMDB graph relation cursor: {error}"))?;
        let mut result = GraphRelationListResult::default();
        for (key, value) in cursor.iter() {
            if !key.starts_with(prefix.as_bytes()) {
                continue;
            }
            result.relations_total += 1;
            if result.relations.len() >= limit {
                continue;
            }
            let relation_id = std::str::from_utf8(value)
                .map_err(|error| format!("invalid LMDB graph relation id: {error}"))?;
            if let Some(relation) = self.get_graph_relation_by_id_txn(txn, relation_id)? {
                result.relations.push(relation);
            }
        }
        Ok(result)
    }

    fn schema_ready(path: &Path) -> Result<bool, ()> {
        let mut builder = Environment::new();
        builder
            .set_max_dbs(16)
            .set_map_size(MAP_SIZE)
            .set_flags(EnvironmentFlags::READ_ONLY);
        let env = builder.open(path).map_err(|_| ())?;
        let Ok(schema_meta) = env.open_db(Some("schema_meta")) else {
            return Ok(false);
        };
        let Ok(txn) = env.begin_ro_txn() else {
            return Err(());
        };
        Ok(
            matches!(txn.get(schema_meta, &"meta:schema"), Ok(value) if value == RECORD_SCHEMA.as_bytes()),
        )
    }
}

impl ReplayMetadata {
    fn to_json(&self) -> String {
        format!(
            "{{\"replay_id\":\"{}\",\"journal_identity\":\"{}\",\"journal_path\":\"{}\",\"record_schema\":\"{}\",\"journal_schema\":\"{}\",\"started_at\":\"{}\",\"completed_at\":\"{}\",\"lines_total\":{},\"lines_replayed\":{},\"records_written\":{},\"records_duplicate\":{},\"records_skipped\":{},\"invalid_entries\":{},\"unsupported_entries\":{},\"cursor_line\":{},\"status\":\"{}\",\"compatibility\":\"{}\"}}",
            escape_json(&self.replay_id),
            escape_json(&self.journal_identity),
            escape_json(&self.journal_path),
            escape_json(&self.record_schema),
            escape_json(&self.journal_schema),
            escape_json(&self.started_at),
            escape_json(&self.completed_at),
            self.lines_total,
            self.lines_replayed,
            self.records_written,
            self.records_duplicate,
            self.records_skipped,
            self.invalid_entries,
            self.unsupported_entries,
            self.cursor_line,
            escape_json(&self.status),
            escape_json(&self.compatibility)
        )
    }

    fn from_bytes(value: &[u8]) -> Result<Self, String> {
        let raw_json = std::str::from_utf8(value)
            .map_err(|error| format!("invalid LMDB replay metadata utf8: {error}"))?;
        Ok(Self {
            replay_id: json_string_field(raw_json, "replay_id").unwrap_or_default(),
            journal_identity: json_string_field(raw_json, "journal_identity").unwrap_or_default(),
            journal_path: json_string_field(raw_json, "journal_path").unwrap_or_default(),
            record_schema: json_string_field(raw_json, "record_schema").unwrap_or_default(),
            journal_schema: json_string_field(raw_json, "journal_schema").unwrap_or_default(),
            started_at: json_string_field(raw_json, "started_at").unwrap_or_default(),
            completed_at: json_string_field(raw_json, "completed_at").unwrap_or_default(),
            lines_total: json_usize_field(raw_json, "lines_total"),
            lines_replayed: json_usize_field(raw_json, "lines_replayed"),
            records_written: json_usize_field(raw_json, "records_written"),
            records_duplicate: json_usize_field(raw_json, "records_duplicate"),
            records_skipped: json_usize_field(raw_json, "records_skipped"),
            invalid_entries: json_usize_field(raw_json, "invalid_entries"),
            unsupported_entries: json_usize_field(raw_json, "unsupported_entries"),
            cursor_line: json_usize_field(raw_json, "cursor_line"),
            status: json_string_field(raw_json, "status").unwrap_or_default(),
            compatibility: json_string_field(raw_json, "compatibility").unwrap_or_default(),
        })
    }
}

impl StoredRecordEnvelope {
    fn from_bytes(value: &[u8]) -> Result<Self, String> {
        let raw_json = std::str::from_utf8(value)
            .map_err(|error| format!("invalid LMDB record envelope utf8: {error}"))?
            .to_string();
        Ok(Self {
            schema: json_string_field(&raw_json, "schema").unwrap_or_default(),
            record_id: json_string_field(&raw_json, "record_id").unwrap_or_default(),
            record_kind: json_string_field(&raw_json, "record_kind").unwrap_or_default(),
            case_ref: json_string_field(&raw_json, "case_ref").unwrap_or_default(),
            raw_json,
        })
    }
}

impl GraphRelation {
    fn to_json(&self) -> String {
        format!(
            "{{\"schema\":\"{}\",\"relation_id\":\"{}\",\"case_ref\":\"{}\",\"from_ref\":\"{}\",\"to_ref\":\"{}\",\"edge_kind\":\"{}\",\"from_kind\":\"{}\",\"to_kind\":\"{}\",\"source_record_id\":\"{}\",\"source_record_kind\":\"{}\",\"confidence\":\"{}\",\"created_at_unix_ms\":{},\"provenance\":\"{}\"}}",
            GRAPH_RELATION_SCHEMA,
            escape_json(&self.relation_id),
            escape_json(&self.case_ref),
            escape_json(&self.from_ref),
            escape_json(&self.to_ref),
            escape_json(&self.edge_kind),
            escape_json(&self.from_kind),
            escape_json(&self.to_kind),
            escape_json(&self.source_record_id),
            escape_json(&self.source_record_kind),
            escape_json(&self.confidence),
            self.created_at_unix_ms,
            escape_json(&self.provenance)
        )
    }

    fn from_bytes(value: &[u8]) -> Result<Self, String> {
        let raw_json = std::str::from_utf8(value)
            .map_err(|error| format!("invalid LMDB graph relation utf8: {error}"))?;
        Ok(Self {
            relation_id: json_string_field(raw_json, "relation_id").unwrap_or_default(),
            case_ref: json_string_field(raw_json, "case_ref").unwrap_or_default(),
            from_ref: json_string_field(raw_json, "from_ref").unwrap_or_default(),
            to_ref: json_string_field(raw_json, "to_ref").unwrap_or_default(),
            edge_kind: json_string_field(raw_json, "edge_kind").unwrap_or_default(),
            from_kind: json_string_field(raw_json, "from_kind").unwrap_or_default(),
            to_kind: json_string_field(raw_json, "to_kind").unwrap_or_default(),
            source_record_id: json_string_field(raw_json, "source_record_id").unwrap_or_default(),
            source_record_kind: json_string_field(raw_json, "source_record_kind")
                .unwrap_or_default(),
            confidence: json_string_field(raw_json, "confidence").unwrap_or_default(),
            created_at_unix_ms: json_u128_field(raw_json, "created_at_unix_ms"),
            provenance: json_string_field(raw_json, "provenance").unwrap_or_default(),
        })
    }
}

fn derive_graph_relations_from_transition(
    transition: &Transition,
    skipped: &mut usize,
) -> Vec<GraphRelation> {
    let mut relations = Vec::new();
    add_transition_relation(
        &mut relations,
        skipped,
        transition,
        "transition_committed_in_case",
        "transition",
        &transition.transition_id,
        "case",
        &transition.case_id,
    );
    match &transition.payload {
        TransitionPayload::CaseOpened { .. } => {}
        TransitionPayload::ParticipantBound { participant_id, .. } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "participant_bound_to_case",
            "participant",
            participant_id,
            "case",
            &transition.case_id,
        ),
        TransitionPayload::ParticipantAdmitted { participant_id, .. } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "participant_admitted_to_case",
            "participant",
            participant_id,
            "case",
            &transition.case_id,
        ),
        TransitionPayload::ProviderAttached {
            participant_id,
            model_id,
            ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "participant_uses_model",
            "participant",
            participant_id,
            "model",
            model_id,
        ),
        TransitionPayload::ProviderInvocationStarted {
            invocation_id,
            model_id,
            ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "provider_invocation_uses_model",
            "provider_invocation",
            invocation_id,
            "model",
            model_id,
        ),
        TransitionPayload::ProviderResultRecorded {
            result_id,
            invocation_id,
            ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "provider_result_closes_invocation",
            "provider_result",
            result_id,
            "provider_invocation",
            invocation_id,
        ),
        TransitionPayload::InteractionTurnRecorded {
            turn_id,
            invocation_id,
            result_id,
            ..
        } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "interaction_turn_uses_invocation",
                "interaction_turn",
                turn_id,
                "provider_invocation",
                invocation_id,
            );
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "interaction_turn_observes_result",
                "interaction_turn",
                turn_id,
                "provider_result",
                result_id,
            );
        }
        TransitionPayload::ModelInterpretationRecorded {
            interpretation_id,
            result_id,
            ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "model_interpretation_from_result",
            "model_interpretation",
            interpretation_id,
            "provider_result",
            result_id,
        ),
        TransitionPayload::ResourceAttached { attachment } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "resource_attached_to_case",
            "resource_attachment",
            &attachment.attachment_id,
            "case",
            &transition.case_id,
        ),
        TransitionPayload::OperationNormalizationFailed {
            provider_result_id, ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "normalization_failure_from_provider_result",
            "transition",
            &transition.transition_id,
            "provider_result",
            provider_result_id,
        ),
        TransitionPayload::OperationRecorded { operation } => {
            match &operation.origin {
                OperationOrigin::ProviderResult {
                    provider_result_id, ..
                } => add_transition_relation(
                    &mut relations,
                    skipped,
                    transition,
                    "operation_from_provider_result",
                    "operation",
                    &operation.operation_id,
                    "provider_result",
                    provider_result_id,
                ),
                OperationOrigin::CompatibilityReview { review_id, .. } => add_transition_relation(
                    &mut relations,
                    skipped,
                    transition,
                    "operation_from_review",
                    "operation",
                    &operation.operation_id,
                    "review_request",
                    review_id,
                ),
            }
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "operation_targets_resource",
                "operation",
                &operation.operation_id,
                "resource_attachment",
                &operation.resource_attachment_id,
            );
        }
        TransitionPayload::DecisionRecorded { decision } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "decision_controls_operation",
            "decision",
            &decision.decision_id,
            "operation",
            &decision.operation_id,
        ),
        TransitionPayload::ExecutionGrantIssued { grant } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "execution_grant_from_decision",
            "execution_grant",
            &grant.grant_id,
            "decision",
            &grant.decision_id,
        ),
        TransitionPayload::EffectPrepared { prepared } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "prepared_effect_consumes_grant",
            "prepared_effect",
            &prepared.effect_id,
            "execution_grant",
            &prepared.grant_id,
        ),
        TransitionPayload::EffectFinalized {
            effect_id, receipt, ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "effect_receipt_closes_prepared_effect",
            "effect_receipt",
            &receipt.receipt_id,
            "prepared_effect",
            effect_id,
        ),
        TransitionPayload::EffectIndeterminate { effect_id, .. } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "indeterminate_transition_tracks_effect",
            "transition",
            &transition.transition_id,
            "prepared_effect",
            effect_id,
        ),
        TransitionPayload::EffectReconciled {
            effect_id, receipt, ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "reconciliation_closes_or_tracks_effect",
            receipt
                .as_ref()
                .map(|_| "effect_receipt")
                .unwrap_or("transition"),
            receipt
                .as_ref()
                .map(|value| value.receipt_id.as_str())
                .unwrap_or(&transition.transition_id),
            "prepared_effect",
            effect_id,
        ),
        TransitionPayload::ReviewRequested { review } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "review_request_for_attempt",
            "review_request",
            &review.review_id,
            "attempt",
            &review.attempt_id,
        ),
        TransitionPayload::ReviewResolved {
            review_id,
            decision_ref,
            receipt_ref,
            ..
        } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "review_decision_resolves_request",
                "decision",
                decision_ref,
                "review_request",
                review_id,
            );
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "review_resolution_produces_receipt",
                "review_request",
                review_id,
                "receipt",
                receipt_ref,
            );
        }
    }
    relations
}

#[allow(clippy::too_many_arguments)]
fn add_transition_relation(
    relations: &mut Vec<GraphRelation>,
    skipped: &mut usize,
    transition: &Transition,
    edge_kind: &str,
    from_kind: &str,
    from_ref: &str,
    to_kind: &str,
    to_ref: &str,
) {
    if from_ref.is_empty() || to_ref.is_empty() {
        *skipped += 1;
        return;
    }
    relations.push(GraphRelation {
        relation_id: format!(
            "edge:{}:{}",
            edge_kind,
            relation_id_component(&transition.transition_id)
        ),
        case_ref: transition.case_id.clone(),
        from_ref: from_ref.to_string(),
        to_ref: to_ref.to_string(),
        edge_kind: edge_kind.to_string(),
        from_kind: from_kind.to_string(),
        to_kind: to_kind.to_string(),
        source_record_id: transition.transition_id.clone(),
        source_record_kind: transition.payload.kind().to_string(),
        confidence: "derived".to_string(),
        created_at_unix_ms: u128::from(transition.committed_at_unix_ms),
        provenance: "canonical_transition".to_string(),
    });
}

fn derive_graph_relations(
    record: &StoredRecordEnvelope,
    created_at_unix_ms: u128,
    skipped: &mut usize,
) -> Vec<GraphRelation> {
    let mut relations = Vec::new();
    let LegacyDecodeOutcome::Promoted(legacy) = decode_legacy_record(&record.raw_json) else {
        *skipped += 1;
        return relations;
    };
    let subject_ref = legacy.subject_ref.clone();
    let attempt_id = legacy.attempt_id.clone();
    let decision_id = legacy.decision_id.clone();
    let receipt_id = legacy.receipt_id.clone();

    add_relation(
        &mut relations,
        skipped,
        relation_from_record(
            record,
            "record_materializes_node",
            "record",
            &record.record_id,
            node_kind_for_record(&record.record_kind),
            &node_ref_for_record(
                record,
                &legacy,
                &subject_ref,
                &attempt_id,
                &decision_id,
                &receipt_id,
            ),
            created_at_unix_ms,
        ),
    );

    if has_subject_ref(&subject_ref) {
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "subject_participates_in_case",
                "subject",
                &subject_ref,
                "case",
                &record.case_ref,
                created_at_unix_ms,
            ),
        );
    }

    if matches!(record.record_kind.as_str(), "attempt" | "carrier_request") {
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "attempt_targets_subject",
                "attempt",
                &attempt_id,
                "subject",
                &subject_ref,
                created_at_unix_ms,
            ),
        );
    }

    if record.record_kind == "decision" {
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "decision_controls_attempt",
                "decision",
                &decision_id,
                "attempt",
                &attempt_id,
                created_at_unix_ms,
            ),
        );
    }

    if matches!(
        record.record_kind.as_str(),
        "receipt" | "effect_receipt" | "filesystem_receipt"
    ) {
        let explicitly_no_effect = legacy.compatibility_value("no_resource_effect") == Some("true")
            || matches!(
                legacy.compatibility_value("status"),
                Some("blocked" | "deferred" | "quarantined" | "not_executed")
            );
        let (target_kind, target_ref) = if explicitly_no_effect {
            (
                "attempt",
                if attempt_id.is_empty() {
                    record.record_id.clone()
                } else {
                    attempt_id.clone()
                },
            )
        } else {
            (
                "effect",
                if attempt_id.is_empty() {
                    format!("effect:{}", record.record_id)
                } else {
                    format!("effect:{attempt_id}")
                },
            )
        };
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                if explicitly_no_effect {
                    "receipt_records_no_effect"
                } else {
                    "receipt_records_effect"
                },
                "receipt",
                &receipt_id,
                target_kind,
                &target_ref,
                created_at_unix_ms,
            ),
        );
    }

    if matches!(
        record.record_kind.as_str(),
        "policy_rule" | "receipt_requirement" | "authority_scope"
    ) {
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "policy_constrains_subject",
                "policy",
                &record.record_id,
                "subject",
                &subject_ref,
                created_at_unix_ms,
            ),
        );
    }

    if matches!(
        record.record_kind.as_str(),
        "projection" | "projection_request" | "projection_result"
    ) {
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "projection_exposes_record",
                "projection",
                &record.record_id,
                "record",
                &record.record_id,
                created_at_unix_ms,
            ),
        );
    }

    if record.record_kind == "model_interpretation" {
        let model_output_ref = format!("model_output:{}", record.record_id);
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "model_output_produces_interpretation",
                "model_output",
                &model_output_ref,
                "model_interpretation",
                &record.record_id,
                created_at_unix_ms,
            ),
        );
    }

    if record.record_kind == "divergence" {
        let (target_kind, target_ref) = if !receipt_id.is_empty() {
            ("receipt", receipt_id.as_str())
        } else if !decision_id.is_empty() {
            ("decision", decision_id.as_str())
        } else {
            ("attempt", attempt_id.as_str())
        };
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "divergence_describes_mismatch",
                "divergence",
                &record.record_id,
                target_kind,
                target_ref,
                created_at_unix_ms,
            ),
        );
    }

    if record.record_kind == "review_request" {
        let review_ref = legacy
            .compatibility_value("review_id")
            .unwrap_or(&record.record_id);
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "review_request_for_attempt",
                "review_request",
                &review_ref,
                "attempt",
                &attempt_id,
                created_at_unix_ms,
            ),
        );
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "review_resolution_produces_receipt",
                "review_request",
                &review_ref,
                "receipt",
                &receipt_id,
                created_at_unix_ms,
            ),
        );
    }

    if record.record_kind == "review_decision" {
        let review_ref = legacy.compatibility_value("review_id").unwrap_or("");
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "review_decision_resolves_request",
                "review_decision",
                &record.record_id,
                "review_request",
                &review_ref,
                created_at_unix_ms,
            ),
        );
    }

    if record.record_kind == "control_pending" {
        let pending_ref = legacy
            .compatibility_value("pending_id")
            .unwrap_or(&record.record_id);
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "control_pending_blocks_attempt",
                "control_pending",
                &pending_ref,
                "attempt",
                &attempt_id,
                created_at_unix_ms,
            ),
        );
    }

    relations
}

fn relation_from_record(
    record: &StoredRecordEnvelope,
    edge_kind: &str,
    from_kind: &str,
    from_ref: &str,
    to_kind: &str,
    to_ref: &str,
    created_at_unix_ms: u128,
) -> Option<GraphRelation> {
    if record.case_ref.is_empty()
        || edge_kind.is_empty()
        || from_kind.is_empty()
        || from_ref.is_empty()
        || to_kind.is_empty()
        || to_ref.is_empty()
        || to_ref == "subject:none"
    {
        return None;
    }
    Some(GraphRelation {
        relation_id: format!(
            "edge:{}:{}",
            edge_kind,
            relation_id_component(&record.record_id)
        ),
        case_ref: record.case_ref.clone(),
        from_ref: from_ref.to_string(),
        to_ref: to_ref.to_string(),
        edge_kind: edge_kind.to_string(),
        from_kind: from_kind.to_string(),
        to_kind: to_kind.to_string(),
        source_record_id: record.record_id.clone(),
        source_record_kind: record.record_kind.clone(),
        confidence: "derived".to_string(),
        created_at_unix_ms,
        provenance: "record".to_string(),
    })
}

fn add_relation(
    relations: &mut Vec<GraphRelation>,
    skipped: &mut usize,
    relation: Option<GraphRelation>,
) {
    if let Some(relation) = relation {
        relations.push(relation);
    } else {
        *skipped += 1;
    }
}

fn push_runtime_node(nodes: &mut Vec<RuntimeGraphNode>, node: RuntimeGraphNode) {
    if nodes
        .iter()
        .any(|existing| existing.node_ref == node.node_ref)
    {
        return;
    }
    nodes.push(node);
}

fn push_unique_string(values: &mut Vec<String>, value: &str) {
    if values.iter().any(|existing| existing == value) {
        return;
    }
    values.push(value.to_string());
}

fn node_ref_for_record(
    record: &StoredRecordEnvelope,
    legacy: &LegacyRecord,
    subject_ref: &str,
    attempt_id: &str,
    decision_id: &str,
    receipt_id: &str,
) -> String {
    match record.record_kind.as_str() {
        "case" => record.case_ref.clone(),
        "subject_binding" | "subject_state" => subject_ref.to_string(),
        "attempt" | "carrier_request" => fallback_ref(attempt_id, &record.record_id),
        "decision" | "decision_basis" | "gate_result" => {
            fallback_ref(decision_id, &record.record_id)
        }
        "review_request" => legacy
            .compatibility_value("review_id")
            .unwrap_or(&record.record_id)
            .to_string(),
        "control_pending" => legacy
            .compatibility_value("pending_id")
            .unwrap_or(&record.record_id)
            .to_string(),
        "review_decision" => record.record_id.clone(),
        "receipt" | "effect_receipt" | "filesystem_receipt" => {
            fallback_ref(receipt_id, &record.record_id)
        }
        "policy_rule" | "receipt_requirement" | "authority_scope" => record.record_id.clone(),
        "projection" | "projection_request" | "projection_result" => record.record_id.clone(),
        "memory_candidate" => record.record_id.clone(),
        "model_interpretation" => record.record_id.clone(),
        "divergence" => record.record_id.clone(),
        _ => record.record_id.clone(),
    }
}

fn node_kind_for_record(record_kind: &str) -> &'static str {
    match record_kind {
        "case" => "case",
        "subject_binding" | "subject_state" => "subject",
        "attempt" => "attempt",
        "decision" | "decision_basis" | "gate_result" => "decision",
        "review_request" => "review_request",
        "review_decision" => "review_decision",
        "control_pending" => "control_pending",
        "carrier_request" => "dispatch",
        "receipt" | "effect_receipt" | "filesystem_receipt" => "receipt",
        "policy_rule" | "receipt_requirement" | "authority_scope" => "policy",
        "projection" | "projection_request" | "projection_result" => "projection",
        "memory_candidate" => "memory_candidate",
        "model_interpretation" => "model_interpretation",
        "divergence" => "divergence",
        _ => "record",
    }
}

fn fallback_ref(preferred: &str, fallback: &str) -> String {
    if preferred.is_empty() {
        fallback.to_string()
    } else {
        preferred.to_string()
    }
}

fn has_subject_ref(subject_ref: &str) -> bool {
    !subject_ref.is_empty() && subject_ref != "subject:none"
}

fn relation_id_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn count_entries(txn: &RoTransaction<'_>, db: Database) -> Result<usize, String> {
    let mut cursor = txn
        .open_ro_cursor(db)
        .map_err(|error| format!("failed to open LMDB cursor: {error}"))?;
    Ok(cursor.iter().count())
}

fn ensure_meta_upgradeable<T: Transaction>(
    txn: &T,
    database: Database,
    key: &str,
    expected: &str,
    previous: &[&str],
) -> Result<(), String> {
    match txn.get(database, &key) {
        Ok(actual) if actual == expected.as_bytes() => Ok(()),
        Ok(actual) if previous.iter().any(|value| actual == value.as_bytes()) => Ok(()),
        Ok(actual) => Err(format!(
            "unsupported_persisted_schema: {key} expected={expected} actual={}",
            String::from_utf8_lossy(actual)
        )),
        Err(Error::NotFound) => Ok(()),
        Err(error) => Err(format!("failed to read persisted schema {key}: {error}")),
    }
}

fn transition_id_key(transition_id: &str) -> String {
    format!("transition:id:{transition_id}")
}

fn case_sequence_prefix(case_id: &str) -> String {
    format!("transition:case:{case_id}:")
}

fn case_sequence_key(case_id: &str, sequence: u64) -> String {
    format!("{}{:020}", case_sequence_prefix(case_id), sequence)
}

fn case_state_key(case_id: &str) -> String {
    format!("case_state:{case_id}")
}

fn semantic_context_artifact_key(artifact_id: &str) -> String {
    format!("semantic-context:id:{artifact_id}")
}

fn local_binding_key(case_id: &str, attachment_id: &str) -> String {
    format!(
        "local_binding:{}:{}:{}:{}",
        case_id.len(),
        case_id,
        attachment_id.len(),
        attachment_id
    )
}

fn json_string_field(content: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = content.find(&marker)? + marker.len();
    let rest = &content[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn json_usize_field(content: &str, key: &str) -> usize {
    let marker = format!("\"{key}\":");
    let Some(start) = content.find(&marker).map(|index| index + marker.len()) else {
        return 0;
    };
    let rest = &content[start..];
    let end = rest
        .find(|ch: char| matches!(ch, ',' | '}'))
        .unwrap_or(rest.len());
    rest[..end].trim().parse::<usize>().unwrap_or(0)
}

fn json_u128_field(content: &str, key: &str) -> u128 {
    let marker = format!("\"{key}\":");
    let Some(start) = content.find(&marker).map(|index| index + marker.len()) else {
        return 0;
    };
    let rest = &content[start..];
    let end = rest
        .find(|ch: char| matches!(ch, ',' | '}'))
        .unwrap_or(rest.len());
    rest[..end].trim().parse::<u128>().unwrap_or(0)
}

fn replay_metadata_key(journal_identity: &str) -> String {
    format!("meta:replay:{journal_identity}")
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn unix_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{RenderedInputMetadata, SemanticContextArtifact, RENDERED_INPUT_SCHEMA};
    use crate::effect::{
        build_effect_receipt, classify_reconciliation, decide_filesystem_write,
        execute_filesystem_write, issue_execution_grant, normalize_filesystem_write_candidate,
        observe_filesystem, prepare_effect, validate_finalized_effect_chain, CarrierFailpoint,
        CarrierResult, EffectOutcome, LocalFilesystemBinding, NormalizationContext,
        ReconciliationConclusion,
    };
    use crate::record::{Record, RecordKind};
    use crate::transition::{
        CaseLifecycle, InterpretationAuthority, PendingTransition, ProviderInvocationLineage,
        ResourceAttachmentState, ResourceKind, ReviewResolution, ReviewState, TransitionPayload,
        TransitionScope, TransitionSource,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_store_path(name: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("yai-{name}-{}-{now}", std::process::id()))
    }

    fn pending(
        id: &str,
        case_id: &str,
        generation: u64,
        payload: TransitionPayload,
    ) -> PendingTransition {
        PendingTransition::new(
            id,
            case_id,
            generation,
            TransitionSource::component("canonical-test"),
            payload,
        )
    }

    fn test_provider_lineage(generation: u64) -> ProviderInvocationLineage {
        ProviderInvocationLineage {
            projection_id: format!("projection:{generation}"),
            context_frame_id: format!("context-frame:{generation}"),
            case_generation: generation,
            rendered_input_id: format!("rendered-input:{generation}"),
            rendered_input_digest: format!("digest:{generation}"),
            output_contract_id: "output-contract:natural-language".to_string(),
            continuation_disposition: "not_provided".to_string(),
        }
    }

    fn commit_typed(
        store: &LmdbRecordStore,
        id: &str,
        case_id: &str,
        generation: u64,
        payload: TransitionPayload,
        scope: Option<TransitionScope>,
        causal_refs: Vec<String>,
    ) -> CanonicalCommit {
        let mut value = pending(id, case_id, generation, payload);
        value.scope = scope;
        value.causal_refs = causal_refs;
        store
            .commit_transition(value)
            .expect("commit typed transition")
    }

    #[test]
    fn canonical_transition_and_case_state_are_atomic_replayable_and_restart_safe() {
        let path = temp_store_path("canonical-authority");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let opened = store
            .commit_transition(pending(
                "transition:open",
                "case:canonical",
                0,
                TransitionPayload::CaseOpened {
                    lifecycle: CaseLifecycle::Open,
                },
            ))
            .expect("commit Case open");
        assert_eq!(opened.transition.sequence, 1);
        assert_eq!(opened.state.generation, 1);

        let bound = store
            .commit_transition(pending(
                "transition:bound",
                "case:canonical",
                1,
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:model".to_string(),
                    role: "model_participant".to_string(),
                },
            ))
            .expect("commit participant binding");
        assert_eq!(bound.transition.sequence, 2);
        assert_eq!(bound.state.generation, 2);
        assert!(store
            .verify_case_state("case:canonical")
            .expect("verify replay"));

        let transitions = store
            .list_case_transitions("case:canonical")
            .expect("list canonical transitions");
        assert_eq!(
            transitions
                .iter()
                .map(|transition| transition.sequence)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        let expected = bound.state;
        store
            .discard_case_state_for_test("case:canonical")
            .expect("discard materialization");
        assert_eq!(
            store.get_case_state("case:canonical").expect("state read"),
            None
        );
        assert_eq!(
            store
                .rebuild_case_state("case:canonical")
                .expect("rebuild CaseState"),
            expected
        );
        drop(store);

        let reopened = LmdbRecordStore::open(&path).expect("reopen after restart");
        assert_eq!(
            reopened
                .get_case_state("case:canonical")
                .expect("restart state"),
            Some(expected)
        );
        assert!(reopened
            .verify_case_state("case:canonical")
            .expect("restart replay"));
        drop(reopened);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn deleting_semantic_context_artifacts_cannot_delete_case_continuity() {
        let path = temp_store_path("context-artifact-derived");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        store
            .commit_transition(pending(
                "transition:open",
                "case:context-artifact",
                0,
                TransitionPayload::CaseOpened {
                    lifecycle: CaseLifecycle::Open,
                },
            ))
            .expect("commit canonical case");
        let before = store
            .get_case_state("case:context-artifact")
            .unwrap()
            .unwrap();
        let artifact = SemanticContextArtifact::RenderedInputMetadata(RenderedInputMetadata {
            schema: RENDERED_INPUT_SCHEMA.to_string(),
            rendered_input_id: "rendered-input:test".to_string(),
            context_frame_id: "context-frame:test".to_string(),
            provider_id: "provider:test".to_string(),
            model_id: "model:test".to_string(),
            content_digest: "digest:test".to_string(),
            content_chars: 42,
        });
        store
            .put_semantic_context_artifact(&artifact)
            .expect("store derived artifact");
        assert_eq!(
            store
                .get_semantic_context_artifact("rendered-input:test")
                .unwrap(),
            Some(artifact)
        );
        store
            .clear_semantic_context_artifacts()
            .expect("clear derived artifacts");
        assert_eq!(
            store
                .get_semantic_context_artifact("rendered-input:test")
                .unwrap(),
            None
        );
        assert_eq!(
            store
                .get_case_state("case:context-artifact")
                .unwrap()
                .unwrap(),
            before
        );
        fs::remove_dir_all(path).expect("remove temp store");
    }

    #[test]
    fn canonical_commit_rejects_duplicate_stale_and_partial_writes() {
        let path = temp_store_path("canonical-failures");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        store
            .commit_transition(pending(
                "transition:open",
                "case:failures",
                0,
                TransitionPayload::CaseOpened {
                    lifecycle: CaseLifecycle::Open,
                },
            ))
            .expect("open Case");

        assert!(store
            .commit_transition(pending(
                "transition:open",
                "case:failures",
                1,
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:duplicate".to_string(),
                    role: "test".to_string(),
                },
            ))
            .unwrap_err()
            .contains("duplicate_transition_id"));
        assert!(store
            .commit_transition(pending(
                "transition:stale",
                "case:failures",
                0,
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:stale".to_string(),
                    role: "test".to_string(),
                },
            ))
            .unwrap_err()
            .contains("stale_case_generation"));

        let injected = pending(
            "transition:rollback",
            "case:failures",
            1,
            TransitionPayload::ParticipantBound {
                participant_id: "participant:rollback".to_string(),
                role: "test".to_string(),
            },
        );
        assert!(store
            .commit_transition_inner(injected, true)
            .unwrap_err()
            .contains("injected_failure"));
        assert_eq!(
            store
                .get_transition_by_id("transition:rollback")
                .expect("transition lookup"),
            None
        );
        assert_eq!(
            store
                .get_case_state("case:failures")
                .expect("state lookup")
                .expect("state present")
                .generation,
            1
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn controlled_effect_is_atomic_replayable_and_reconciles_visible_crash() {
        let path = temp_store_path("controlled-effect");
        let resource_root = temp_store_path("controlled-effect-resource");
        fs::create_dir_all(resource_root.join("allowed")).expect("create resource fixture");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let case_id = "case:controlled";
        let model = "participant:model";
        let operator = "participant:operator";
        commit_typed(
            &store,
            "transition:open",
            case_id,
            0,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:model",
            case_id,
            1,
            TransitionPayload::ParticipantBound {
                participant_id: model.to_string(),
                role: "model_provider".to_string(),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:operator",
            case_id,
            2,
            TransitionPayload::ParticipantBound {
                participant_id: operator.to_string(),
                role: "policy_owner".to_string(),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:provider",
            case_id,
            3,
            TransitionPayload::ProviderAttached {
                participant_id: model.to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                base_url: "http://127.0.0.1:1".to_string(),
                model_id: "model:test".to_string(),
                credential_ref: "env:TEST".to_string(),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:invocation",
            case_id,
            4,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: "invocation:1".to_string(),
                participant_id: model.to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(test_provider_lineage(4)),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:result",
            case_id,
            5,
            TransitionPayload::ProviderResultRecorded {
                result_id: "provider-result:1".to_string(),
                invocation_id: "invocation:1".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(test_provider_lineage(4)),
                output: "candidate".to_string(),
            },
            None,
            vec!["invocation:1".to_string()],
        );
        let resource = ResourceAttachmentState {
            attachment_id: "workspace".to_string(),
            kind: ResourceKind::Filesystem,
            allowed_write_prefix: "allowed".to_string(),
            max_write_bytes: 1024,
            policy_id: "policy:workspace".to_string(),
            policy_owner_participant_id: operator.to_string(),
        };
        commit_typed(
            &store,
            "transition:resource",
            case_id,
            6,
            TransitionPayload::ResourceAttached {
                attachment: resource.clone(),
            },
            None,
            vec![operator.to_string()],
        );
        let binding = LocalFilesystemBinding::new(case_id, "workspace", &resource_root)
            .expect("create local binding");
        store
            .put_local_filesystem_binding(&binding)
            .expect("persist local binding");

        let commit_effect =
            |store: &LmdbRecordStore,
             invocation: &str,
             result_id: &str,
             path: &str,
             content: &str,
             first_generation: u64,
             suffix: &str,
             crash_after_visible: bool|
             -> (crate::effect::PreparedEffect, crate::effect::CarrierResult) {
                let raw = format!(
                "{{\"schema\":\"yai.operation_proposal.filesystem_write.v1\",\"operation\":\"filesystem.write\",\"resource\":\"workspace\",\"path\":\"{path}\",\"content\":\"{content}\"}}"
            );
                let operation = normalize_filesystem_write_candidate(
                    &raw,
                    &NormalizationContext {
                        case_id,
                        participant_id: model,
                        provider_result_id: result_id,
                        provider_invocation_id: invocation,
                        case_generation: first_generation,
                        resource: &resource,
                    },
                )
                .expect("normalize operation");
                let operation_commit = commit_typed(
                    store,
                    &format!("transition:operation:{suffix}"),
                    case_id,
                    first_generation,
                    TransitionPayload::OperationRecorded {
                        operation: operation.clone(),
                    },
                    Some(operation.scope.clone()),
                    vec![result_id.to_string(), invocation.to_string()],
                );
                let decision = decide_filesystem_write(
                    &operation,
                    &resource,
                    operation_commit.state.generation,
                );
                assert_eq!(decision.outcome, crate::effect::DecisionOutcome::Allow);
                let decision_commit = commit_typed(
                    store,
                    &format!("transition:decision:{suffix}"),
                    case_id,
                    operation_commit.state.generation,
                    TransitionPayload::DecisionRecorded {
                        decision: decision.clone(),
                    },
                    None,
                    vec![operation.operation_id.clone()],
                );
                let grant =
                    issue_execution_grant(&operation, &decision, decision_commit.state.generation)
                        .expect("issue execution grant");
                let grant_commit = commit_typed(
                    store,
                    &format!("transition:grant:{suffix}"),
                    case_id,
                    decision_commit.state.generation,
                    TransitionPayload::ExecutionGrantIssued {
                        grant: grant.clone(),
                    },
                    None,
                    vec![operation.operation_id.clone(), decision.decision_id.clone()],
                );
                let pre = observe_filesystem(
                    &binding,
                    &resource,
                    path,
                    format!("observation:pre:{suffix}"),
                );
                let prepared = prepare_effect(&operation, &decision, &grant, pre)
                    .expect("prepare controlled effect");
                let prepared_commit = commit_typed(
                    store,
                    &format!("transition:prepare:{suffix}"),
                    case_id,
                    grant_commit.state.generation,
                    TransitionPayload::EffectPrepared {
                        prepared: prepared.clone(),
                    },
                    None,
                    vec![
                        operation.operation_id.clone(),
                        decision.decision_id.clone(),
                        grant.grant_id.clone(),
                        prepared.expected_pre_observation.observation_id.clone(),
                    ],
                );
                let result = execute_filesystem_write(
                    &operation,
                    &decision,
                    &grant,
                    &prepared,
                    &prepared_commit.state,
                    &binding,
                    &resource,
                    if crash_after_visible {
                        CarrierFailpoint::CrashAfterVisibleEffect
                    } else {
                        CarrierFailpoint::None
                    },
                )
                .expect("execute grant-validated carrier");
                (prepared, result)
            };

        let (prepared, result) = commit_effect(
            &store,
            "invocation:1",
            "provider-result:1",
            "allowed/one.txt",
            "one",
            7,
            "one",
            false,
        );
        assert_eq!(result.outcome, EffectOutcome::Applied);
        let receipt = build_effect_receipt(&prepared, &result);
        let finalized = commit_typed(
            &store,
            "transition:finalize:one",
            case_id,
            11,
            TransitionPayload::EffectFinalized {
                effect_id: prepared.effect_id.clone(),
                post_observation: result.post_observation.clone(),
                receipt: receipt.clone(),
            },
            None,
            vec![prepared.effect_id.clone(), receipt.receipt_id.clone()],
        );
        assert_eq!(
            finalized.state.effects[0].outcome,
            Some(EffectOutcome::Applied)
        );
        validate_finalized_effect_chain(
            &store.list_case_transitions(case_id).expect("list chain"),
            &prepared.effect_id,
        )
        .expect("closed effect chain");
        assert_eq!(
            fs::read_to_string(resource_root.join("allowed/one.txt")).unwrap(),
            "one"
        );

        commit_typed(
            &store,
            "transition:invocation:two",
            case_id,
            12,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: "invocation:2".to_string(),
                participant_id: model.to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(test_provider_lineage(12)),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:result:two",
            case_id,
            13,
            TransitionPayload::ProviderResultRecorded {
                result_id: "provider-result:2".to_string(),
                invocation_id: "invocation:2".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(test_provider_lineage(12)),
                output: "candidate two".to_string(),
            },
            None,
            vec!["invocation:2".to_string()],
        );
        let (crashed, crash_result) = commit_effect(
            &store,
            "invocation:2",
            "provider-result:2",
            "allowed/two.txt",
            "two",
            14,
            "two",
            true,
        );
        assert!(crash_result.crash_injected_after_effect);
        assert_eq!(
            store
                .get_case_state(case_id)
                .expect("state")
                .expect("present")
                .effects[1]
                .status,
            crate::transition::EffectLifecycle::Prepared
        );
        drop(store);

        let reopened = LmdbRecordStore::open(&path).expect("restart after visible effect");
        let state = reopened.get_case_state(case_id).unwrap().unwrap();
        let binding = reopened
            .get_local_filesystem_binding(case_id, "workspace")
            .unwrap()
            .expect("binding survives restart");
        let observed = observe_filesystem(
            &binding,
            &resource,
            "allowed/two.txt",
            "observation:reconcile:two",
        );
        assert_eq!(
            classify_reconciliation(&crashed, &observed),
            ReconciliationConclusion::EffectObserved
        );
        let reconciliation_result = CarrierResult {
            outcome: EffectOutcome::AlreadyApplied,
            post_observation: observed.clone(),
            carrier_attempted: false,
            mutation_performed: false,
            crash_injected_after_effect: false,
            detail: "restart observed intended post-state".to_string(),
        };
        let reconciled_receipt = build_effect_receipt(&crashed, &reconciliation_result);
        commit_typed(
            &reopened,
            "transition:reconcile:two",
            case_id,
            state.generation,
            TransitionPayload::EffectReconciled {
                effect_id: crashed.effect_id.clone(),
                conclusion: ReconciliationConclusion::EffectObserved,
                observation: observed,
                receipt: Some(reconciled_receipt),
            },
            None,
            vec![crashed.effect_id.clone()],
        );
        validate_finalized_effect_chain(
            &reopened.list_case_transitions(case_id).expect("list chain"),
            &crashed.effect_id,
        )
        .expect("reconciled chain closed");
        assert!(reopened
            .verify_case_state(case_id)
            .expect("replay equivalence"));
        let expected = reopened.get_case_state(case_id).unwrap().unwrap();
        reopened.discard_case_state_for_test(case_id).unwrap();
        assert_eq!(reopened.rebuild_case_state(case_id).unwrap(), expected);
        let graph = reopened
            .materialize_graph_relations_for_case(case_id)
            .expect("derived graph from typed transitions");
        assert!(graph.relations_written > 0);
        drop(reopened);
        fs::remove_dir_all(path).expect("remove LMDB test store");
        fs::remove_dir_all(resource_root).expect("remove resource fixture");
    }

    #[test]
    fn legacy_compatibility_import_is_isolated_from_canonical_authority() {
        let path = temp_store_path("legacy-compatibility-import");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let contents = concat!(
            "{\"schema\":\"yai.store.record.v0\",\"record_id\":\"record:one\",\"case_ref\":\"case:legacy\",\"record_kind\":\"case\",\"summary\":\"\"}\n",
            "{\"schema\":\"yai.store.record.v0\",\"record_id\":\"record:two\",\"case_ref\":\"case:legacy\",\"record_kind\":\"future_kind\"}\n",
            "{bad\n"
        );
        let first = store
            .import_legacy_compatibility(contents, "fixture:legacy")
            .expect("import compatibility corpus");
        assert_eq!(first.lines_total, 3);
        assert_eq!(first.losslessly_promoted, 1);
        assert_eq!(first.preserved_opaque, 1);
        assert_eq!(first.rejected_malformed, 1);
        assert_eq!(first.payloads_written, 2);
        assert_eq!(store.legacy_compatibility_payload_count().unwrap(), 2);
        assert!(store
            .list_case_transitions("case:legacy")
            .unwrap()
            .is_empty());
        assert_eq!(store.get_case_state("case:legacy").unwrap(), None);

        let second = store
            .import_legacy_compatibility(contents, "fixture:legacy")
            .expect("repeat compatibility import");
        assert_eq!(second.payloads_written, 0);
        assert_eq!(second.payloads_duplicate, 2);
        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn persisted_future_canonical_schema_is_not_overwritten() {
        let path = temp_store_path("future-canonical-schema");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let mut txn = store
            .env
            .begin_rw_txn()
            .expect("schema mutation transaction");
        txn.put(
            store.schema_meta,
            &"meta:canonical_transition_schema",
            &"yai.transition.v99",
            WriteFlags::empty(),
        )
        .expect("write future schema marker");
        txn.commit().expect("commit future schema marker");
        drop(store);

        let error = match LmdbRecordStore::open(&path) {
            Ok(_) => panic!("future persisted schema must not be overwritten"),
            Err(error) => error,
        };
        assert!(error.contains("unsupported_persisted_schema"));
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn provider_and_review_payloads_reduce_without_summary_semantics() {
        let path = temp_store_path("typed-reducers");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let case_id = "case:typed";
        let payloads = vec![
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
            TransitionPayload::ParticipantBound {
                participant_id: "participant:model".to_string(),
                role: "model_participant".to_string(),
            },
            TransitionPayload::ProviderAttached {
                participant_id: "participant:model".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                base_url: "http://127.0.0.1:1".to_string(),
                model_id: "model:test".to_string(),
                credential_ref: "env:TEST_KEY".to_string(),
            },
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: "invocation:1".to_string(),
                participant_id: "participant:model".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(test_provider_lineage(3)),
            },
        ];
        for (index, payload) in payloads.into_iter().enumerate() {
            store
                .commit_transition(pending(
                    &format!("transition:{}", index + 1),
                    case_id,
                    index as u64,
                    payload,
                ))
                .expect("commit typed setup");
        }
        let mut result = pending(
            "transition:result",
            case_id,
            4,
            TransitionPayload::ProviderResultRecorded {
                result_id: "result:1".to_string(),
                invocation_id: "invocation:1".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(test_provider_lineage(3)),
                output: "typed output".to_string(),
            },
        );
        result.causal_refs = vec!["invocation:1".to_string()];
        result.summary = Some("invocation:wrong output_chars:999".to_string());
        store
            .commit_transition(result)
            .expect("commit typed result");

        let mut interpretation = pending(
            "transition:interpretation",
            case_id,
            5,
            TransitionPayload::ModelInterpretationRecorded {
                interpretation_id: "interpretation:1".to_string(),
                result_id: "result:1".to_string(),
                authority: InterpretationAuthority::NonAuthoritative,
            },
        );
        interpretation.causal_refs = vec!["result:1".to_string()];
        store
            .commit_transition(interpretation)
            .expect("commit interpretation");

        let mut review = pending(
            "transition:review",
            case_id,
            6,
            TransitionPayload::ReviewRequested {
                review: ReviewState {
                    review_id: "review:1".to_string(),
                    attempt_id: "attempt:1".to_string(),
                    requested_by_participant: "participant:model".to_string(),
                    target_participant: "participant:filesystem".to_string(),
                    reviewer_participant: "participant:operator".to_string(),
                    operation_kind: "fs.write".to_string(),
                    carrier_family: "filesystem".to_string(),
                    target_display: "sandbox/output.txt".to_string(),
                    sandbox_path: "/tmp/sandbox".to_string(),
                    target_path: "/tmp/sandbox/output.txt".to_string(),
                    policy_reason: "review_required".to_string(),
                    status: ReviewResolution::PendingOperator,
                    carrier_attempted: false,
                    execution_performed: false,
                    decision_ref: None,
                    receipt_ref: None,
                },
            },
        );
        review.causal_refs = vec!["attempt:1".to_string()];
        store.commit_transition(review).expect("commit review");

        let mut resolved = pending(
            "transition:resolved",
            case_id,
            7,
            TransitionPayload::ReviewResolved {
                review_id: "review:1".to_string(),
                attempt_id: "attempt:1".to_string(),
                resolution: ReviewResolution::Denied,
                reason: "operator denied".to_string(),
                decision_ref: "decision:1".to_string(),
                receipt_ref: "receipt:1".to_string(),
                carrier_attempted: false,
                execution_performed: false,
            },
        );
        resolved.causal_refs = vec!["review:1".to_string(), "attempt:1".to_string()];
        let committed = store
            .commit_transition(resolved)
            .expect("commit resolution");
        assert_eq!(committed.state.generation, 8);
        assert_eq!(
            committed.state.last_provider_result.unwrap().output_chars,
            12
        );
        assert_eq!(committed.state.reviews[0].status, ReviewResolution::Denied);
        assert!(store.verify_case_state(case_id).expect("verify replay"));
        assert!(store
            .materialize_graph_relations_for_case_inner(case_id, true)
            .unwrap_err()
            .contains("injected_graph_materialization_failure"));
        assert!(store
            .verify_case_state(case_id)
            .expect("derived failure cannot affect canonical state"));
        let graph_report = store
            .materialize_graph_relations_for_case(case_id)
            .expect("materialize typed graph");
        assert!(graph_report.relations_written > 0);
        let relations = store
            .list_graph_relations_by_case(case_id, usize::MAX)
            .expect("typed graph relations");
        assert!(relations.relations.iter().any(|relation| {
            relation.edge_kind == "provider_result_closes_invocation"
                && relation.provenance == "canonical_transition"
        }));
        let expected_relation_count = relations.relations_total;
        let rebuild = store
            .rebuild_graph_relations_for_case(case_id)
            .expect("rebuild typed graph from source authority");
        assert_eq!(rebuild.relations_written, expected_relation_count);
        assert_eq!(rebuild.relations_duplicate, 0);
        assert_eq!(
            store
                .list_graph_relations_by_case(case_id, usize::MAX)
                .expect("rebuilt relations")
                .relations_total,
            expected_relation_count
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn freeze_supports_control_carrier_and_divergence_records() {
        let path = temp_store_path("record-freeze");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let records = [
            Record::from_parts(
                "rec:freeze-attempt",
                "case:spine34-freeze",
                RecordKind::Attempt,
                "subject:filesystem-sandbox",
                "op:freeze-write",
                "",
                "",
                "attempt:file.write",
            ),
            Record::from_parts(
                "rec:freeze-decision",
                "case:spine34-freeze",
                RecordKind::Decision,
                "subject:filesystem-sandbox",
                "op:freeze-write",
                "decision:freeze-deny",
                "",
                "decision:deny",
            ),
            Record::from_parts(
                "rec:freeze-carrier-request",
                "case:spine34-freeze",
                RecordKind::CarrierRequest,
                "subject:filesystem-sandbox",
                "op:freeze-write",
                "decision:freeze-deny",
                "",
                "carrier:filesystem requested_outcome:blocked",
            ),
            Record::from_parts(
                "rec:freeze-effect-receipt",
                "case:spine34-freeze",
                RecordKind::EffectReceipt,
                "subject:filesystem-sandbox",
                "op:freeze-write",
                "decision:freeze-deny",
                "receipt:freeze-blocked",
                "receipt:blocked",
            ),
            Record::from_parts(
                "rec:freeze-divergence",
                "case:spine34-freeze",
                RecordKind::Divergence,
                "subject:filesystem-sandbox",
                "op:freeze-write",
                "decision:freeze-deny",
                "receipt:freeze-blocked",
                "divergence:none result:consistent",
            ),
        ];

        for record in &records {
            store
                .append_record(record, "spine34-freeze-test")
                .expect("append freeze record");
        }

        let summary = store.summary().expect("summary");
        assert_eq!(summary.records_total, 5);
        assert_eq!(summary.records_by_case, 5);
        assert_eq!(summary.records_by_kind, 5);
        assert_eq!(summary.records_by_subject, 5);
        assert_eq!(summary.records_by_receipt, 2);

        let divergence = store
            .list_records_by_kind("divergence", 10)
            .expect("list divergence records");
        assert_eq!(divergence.records_total, 1);
        assert_eq!(
            divergence
                .records
                .first()
                .map(|record| record.record_id.as_str()),
            Some("rec:freeze-divergence")
        );

        let receipt_records = store
            .list_records_by_receipt("receipt:freeze-blocked", 10)
            .expect("list receipt records");
        assert_eq!(receipt_records.records_total, 2);
        assert!(receipt_records
            .records
            .iter()
            .any(|record| record.record_kind == "effect_receipt"));
        assert!(receipt_records
            .records
            .iter()
            .any(|record| record.record_kind == "divergence"));

        let carrier_request = store
            .get_record_by_id("rec:freeze-carrier-request")
            .expect("get carrier request")
            .expect("carrier request present");
        assert_eq!(carrier_request.schema, RECORD_SCHEMA);
        assert_eq!(carrier_request.record_kind, "carrier_request");
        assert!(carrier_request
            .raw_json
            .contains("\"schema\":\"yai.record.v1\""));
        assert!(carrier_request
            .raw_json
            .contains("\"source\":{\"plane\":\"journal\""));

        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn legacy_no_effect_receipt_does_not_derive_a_successful_effect_relation() {
        let path = temp_store_path("legacy-no-effect-graph");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let receipt = Record::from_parts(
            "rec:fixture-no-effect",
            "case:fixture-no-effect",
            RecordKind::FilesystemReceipt,
            "subject:fixture",
            "attempt:fixture-write",
            "decision:fixture-descriptor",
            "receipt:fixture-no-effect",
            "fixture_receipt status:not_executed no_resource_effect:true carrier_attempted:false execution_performed:false",
        );
        store
            .append_record(&receipt, "legacy-fixture-test")
            .expect("append compatibility receipt");
        store
            .materialize_graph_relations_for_case("case:fixture-no-effect")
            .expect("materialize compatibility graph");
        let relations = store
            .list_graph_relations_by_case("case:fixture-no-effect", usize::MAX)
            .expect("list graph relations");
        assert!(relations.relations.iter().any(|relation| {
            relation.source_record_id == "rec:fixture-no-effect"
                && relation.edge_kind == "receipt_records_no_effect"
                && relation.to_kind == "attempt"
        }));
        assert!(!relations.relations.iter().any(|relation| {
            relation.source_record_id == "rec:fixture-no-effect"
                && relation.edge_kind == "receipt_records_effect"
        }));
        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn journal_import_report_is_idempotent() {
        let path = temp_store_path("journal-import-idempotent");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let mut journal = Journal::new();
        journal.append(Record::from_parts(
            "rec:journal-import-one",
            "case:journal-import",
            RecordKind::Receipt,
            "subject:journal-import",
            "op:journal-import",
            "decision:journal-import",
            "receipt:journal-import",
            "receipt:journal-import",
        ));
        journal.append(Record::from_parts(
            "rec:journal-import-two",
            "case:journal-import",
            RecordKind::Divergence,
            "subject:journal-import",
            "op:journal-import",
            "decision:journal-import",
            "receipt:journal-import",
            "divergence:none",
        ));

        let first = store
            .import_journal_with_report(&journal, "journal-import-test")
            .expect("first journal import");
        assert_eq!(first.records_seen, 2);
        assert_eq!(first.records_written, 2);
        assert_eq!(first.records_duplicate, 0);

        let second = store
            .import_journal_with_report(&journal, "journal-import-test")
            .expect("second journal import");
        assert_eq!(second.records_seen, 2);
        assert_eq!(second.records_written, 0);
        assert_eq!(second.records_duplicate, 2);

        let summary = store.summary().expect("summary");
        assert_eq!(summary.records_total, 2);
        assert_eq!(summary.records_by_case, 2);
        assert_eq!(summary.records_by_kind, 2);
        assert_eq!(summary.records_by_subject, 2);
        assert_eq!(summary.records_by_receipt, 2);

        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }
}
