//! Content-addressed, rebuildable representations and retrieval indexes.
//!
//! This module owns no semantic truth. It deterministically represents exact
//! `OperationalMemoryEntry` values, builds disposable lexical/vector indexes,
//! and fuses candidates only after the existing Case/Participant/view
//! qualification barrier. Canonical authority remains Transition history.

use crate::context::stable_digest;
use crate::effect::digest_bytes;
use crate::memory::{
    retrieve_operational_memory, OperationalMemoryBuild, OperationalMemoryEntry,
    OperationalMemoryKind, OperationalMemoryPosture, OperationalMemoryValue,
    RetrievalQualification, RetrievalRejections,
};
use crate::transition::CaseState;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[cfg(target_os = "linux")]
use std::ffi::{CStr, CString};
#[cfg(test)]
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, FromRawFd};
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

pub const MEMORY_REPRESENTATION_DOCUMENT_SCHEMA: &str = "yai.memory_representation_document.v1";
pub const MEMORY_REPRESENTATION_CONTRACT: &str = "yai.memory_representation.input.v1";
pub const MEMORY_REPRESENTATION_PROFILE_SCHEMA: &str = "yai.memory_representation_profile.v1";
pub const MEMORY_EMBEDDING_SCHEMA: &str = "yai.memory_embedding.v1";
pub const MEMORY_CORPUS_MANIFEST_SCHEMA: &str = "yai.memory_corpus_manifest.v1";
pub const MEMORY_INDEX_MANIFEST_SCHEMA: &str = "yai.memory_index_manifest.v1";
pub const MEMORY_INDEX_BUNDLE_SCHEMA: &str = "yai.memory_index_bundle.v1";
pub const MEMORY_LEXICAL_INDEX_SCHEMA: &str = "yai.memory_lexical_index.bm25.v1";
pub const MEMORY_VECTOR_INDEX_SCHEMA: &str = "yai.memory_vector_index.exact_cosine.v1";
pub const RETRIEVAL_QUERY_DOCUMENT_SCHEMA: &str = "yai.retrieval_query_document.v1";
pub const HYBRID_RETRIEVAL_SET_SCHEMA: &str = "yai.retrieval_set.v2";
pub const MEMORY_INDEX_BUILD_VERSION: &str = "yai.memory_index.builder.v1";
pub const HYBRID_FUSION_VERSION: &str = "yai.memory_rank_fusion.rrf.v1";
pub const DERIVED_MEMORY_STORE_VERSION: &str = "v2";
pub const DERIVED_MEMORY_PHYSICAL_SCHEMA: &str = "yai.derived_memory_store.v2";

pub const MAX_REPRESENTATION_CHARS: usize = 4096;
pub const MAX_QUERY_CHARS: usize = 2048;
pub const MAX_QUERY_TERMS: usize = 128;
pub const MAX_VECTOR_DIMENSION: usize = 4096;
pub const MAX_CORPUS_DOCUMENTS: usize = 50_000;
/// The cross-product is the actual vector admission boundary. It admits, for
/// example, 50k x 384 and 10k x 1536 while refusing 50k x 4096 before any
/// encoder request or allocation.
pub const MAX_VECTOR_ELEMENTS: usize = 25_000_000;
pub const MAX_VECTOR_BYTES: usize = MAX_VECTOR_ELEMENTS * std::mem::size_of::<f32>();
pub const MAX_DERIVED_METADATA_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_DERIVED_TOTAL_BYTES: usize = 384 * 1024 * 1024;
pub const MAX_CURRENT_POINTER_BYTES: usize = 16 * 1024;
pub const MAX_PHYSICAL_MANIFEST_BYTES: usize = 64 * 1024;
pub const MAX_LAST_RETRIEVAL_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_RETAINED_BUILDS: usize = 2;
pub const MAX_CANDIDATES_PER_PLANE: usize = 256;
pub const MAX_TOTAL_CANDIDATES: usize = 512;
const RRF_K: u64 = 60;
const RRF_SCALE: u64 = 1_000_000;
const EXACT_ANCHOR_BONUS: u64 = 1_000_000;

fn digest_json<T: Serialize>(value: &T, label: &str) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|bytes| digest_bytes(&bytes))
        .map_err(|error| format!("{label}_encode_failed: {error}"))
}

fn short_id(prefix: &str, digest: &str) -> String {
    let value = digest.strip_prefix("sha256:").unwrap_or(digest);
    format!("{prefix}:{}", &value[..value.len().min(32)])
}

fn require_nonempty(label: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{label}_required"))
    } else {
        Ok(())
    }
}

fn bounded(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let suffix = "...";
    let mut output = value
        .chars()
        .take(limit.saturating_sub(suffix.chars().count()))
        .collect::<String>();
    output.push_str(suffix);
    output
}

fn canonicalize_query(value: &str) -> String {
    let mut output = String::new();
    let mut pending_space = false;
    let mut count = 0usize;
    for ch in value.chars() {
        if count >= MAX_QUERY_CHARS {
            break;
        }
        if ch.is_whitespace() || ch.is_control() {
            pending_space = !output.is_empty();
            continue;
        }
        if pending_space && count < MAX_QUERY_CHARS {
            output.push(' ');
            count += 1;
        }
        pending_space = false;
        if count < MAX_QUERY_CHARS {
            output.push(ch);
            count += 1;
        }
    }
    output
}

pub fn validate_memory_index_build_budget(
    corpus: &MemoryRepresentationCorpus,
    profile: &MemoryRepresentationProfile,
) -> Result<(), String> {
    validate_memory_index_build_budget_documents(&corpus.documents, profile)
}

fn validate_memory_index_build_budget_documents(
    documents: &[MemoryRepresentationDocument],
    profile: &MemoryRepresentationProfile,
) -> Result<(), String> {
    profile.validate()?;
    if documents.len() > MAX_CORPUS_DOCUMENTS {
        return Err("memory_index_document_budget_exceeded".to_string());
    }
    let vector_elements = validate_vector_shape_budget(documents.len(), profile.vector_dimension)?;
    let vector_bytes = vector_elements
        .checked_mul(std::mem::size_of::<f32>())
        .ok_or_else(|| "memory_index_vector_byte_budget_overflow".to_string())?;
    let representation_bytes = documents.iter().try_fold(0usize, |total, document| {
        total
            .checked_add(document.canonical_text.len())
            .ok_or_else(|| "memory_index_representation_byte_budget_overflow".to_string())
    })?;
    let estimated_metadata = representation_bytes
        .checked_mul(3)
        .and_then(|value| value.checked_add(documents.len().checked_mul(4096)?))
        .ok_or_else(|| "memory_index_metadata_budget_overflow".to_string())?;
    let estimated_total = estimated_metadata
        .checked_add(vector_bytes)
        .and_then(|value| value.checked_add(MAX_PHYSICAL_MANIFEST_BYTES))
        .ok_or_else(|| "memory_index_total_budget_overflow".to_string())?;
    if estimated_metadata > MAX_DERIVED_METADATA_BYTES || estimated_total > MAX_DERIVED_TOTAL_BYTES
    {
        return Err("memory_index_serialized_budget_exceeded".to_string());
    }
    Ok(())
}

fn validate_vector_shape_budget(
    document_count: usize,
    vector_dimension: usize,
) -> Result<usize, String> {
    if document_count > MAX_CORPUS_DOCUMENTS
        || vector_dimension == 0
        || vector_dimension > MAX_VECTOR_DIMENSION
    {
        return Err("memory_index_vector_shape_invalid".to_string());
    }
    let vector_elements = document_count
        .checked_mul(vector_dimension)
        .ok_or_else(|| "memory_index_vector_element_budget_overflow".to_string())?;
    if vector_elements > MAX_VECTOR_ELEMENTS {
        return Err("memory_index_vector_element_budget_exceeded".to_string());
    }
    Ok(vector_elements)
}

fn scrub_sensitive(value: &str) -> String {
    let suspicious = [
        "authorization",
        "bearer ",
        "api_key",
        "api-key",
        "api key",
        "password",
        "credential",
        "secret",
        "token",
        "sk-",
    ];
    let lower = value.to_ascii_lowercase();
    if suspicious.iter().any(|marker| lower.contains(marker)) {
        "[redacted-sensitive-content]".to_string()
    } else {
        value.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

fn canonical_value_text(value: &OperationalMemoryValue) -> String {
    match value {
        OperationalMemoryValue::ResourceEffect {
            operation_id,
            effect_id,
            resource_attachment_id,
            relative_path,
            outcome,
            content_digest,
            receipt_id,
        } => format!(
            "operation_id={operation_id}\neffect_id={effect_id}\nresource_attachment_id={resource_attachment_id}\nrelative_path={}\noutcome={outcome:?}\ncontent_digest={}\nreceipt_id={receipt_id}",
            scrub_sensitive(relative_path),
            content_digest.as_deref().unwrap_or("none")
        ),
        OperationalMemoryValue::Decision {
            operation_id,
            decision_id,
            outcome,
            resource_attachment_id,
            relative_path,
            reason,
        } => format!(
            "operation_id={operation_id}\ndecision_id={decision_id}\noutcome={outcome:?}\nresource_attachment_id={resource_attachment_id}\nrelative_path={}\nreason={}",
            scrub_sensitive(relative_path),
            scrub_sensitive(reason)
        ),
        OperationalMemoryValue::Review {
            review_id,
            operation_id,
            resource_attachment_id,
            relative_path,
            reviewer_participant_id,
            status,
            action_id,
        } => format!(
            "review_id={review_id}\noperation_id={operation_id}\nresource_attachment_id={resource_attachment_id}\nrelative_path={}\nreviewer_participant_id={reviewer_participant_id}\nstatus={status}\naction_id={}",
            scrub_sensitive(relative_path),
            action_id.as_deref().unwrap_or("none")
        ),
        OperationalMemoryValue::UnresolvedEffect {
            operation_id,
            effect_id,
            resource_attachment_id,
            relative_path,
            state,
            reason,
        } => format!(
            "operation_id={operation_id}\neffect_id={effect_id}\nresource_attachment_id={resource_attachment_id}\nrelative_path={}\nstate={state}\nreason={}",
            scrub_sensitive(relative_path),
            scrub_sensitive(reason)
        ),
        OperationalMemoryValue::NormalizationFailure {
            provider_result_id,
            code,
            detail,
        } => format!(
            "provider_result_id={provider_result_id}\ncode={code}\ndetail={}",
            scrub_sensitive(detail)
        ),
        OperationalMemoryValue::ProviderClaim {
            result_id,
            invocation_id,
            provider_id,
            model_id,
            preview,
        } => format!(
            "result_id={result_id}\ninvocation_id={invocation_id}\nprovider_id={provider_id}\nmodel_id={model_id}\nprovider_claim_preview={}",
            scrub_sensitive(preview)
        ),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryRepresentationDocument {
    pub schema: String,
    pub document_id: String,
    pub case_id: String,
    pub memory_id: String,
    pub source_content_digest: String,
    pub operational_memory_derivation_version: String,
    pub representation_contract_version: String,
    pub semantic_kind: String,
    pub authority_posture: String,
    pub lifecycle: String,
    pub canonical_text: String,
    pub provenance_refs: Vec<String>,
    pub participant_ids: Vec<String>,
    pub consumer: String,
    pub view_kind: String,
}

#[derive(Serialize)]
struct RepresentationDocumentIdentity<'a> {
    schema: &'a str,
    case_id: &'a str,
    memory_id: &'a str,
    source_content_digest: &'a str,
    operational_memory_derivation_version: &'a str,
    representation_contract_version: &'a str,
    semantic_kind: &'a str,
    authority_posture: &'a str,
    lifecycle: &'a str,
    canonical_text: &'a str,
    provenance_refs: &'a [String],
    participant_ids: &'a [String],
    consumer: &'a str,
    view_kind: &'a str,
}

impl MemoryRepresentationDocument {
    pub fn from_memory(entry: &OperationalMemoryEntry) -> Result<Self, String> {
        entry.validate()?;
        let structured_class_matches = matches!(
            (&entry.semantic_kind, &entry.posture, &entry.value),
            (
                OperationalMemoryKind::ResourceEffect,
                OperationalMemoryPosture::FinalizedObservedConsequence,
                OperationalMemoryValue::ResourceEffect { .. }
            ) | (
                OperationalMemoryKind::Decision,
                OperationalMemoryPosture::DecisionControlHistory,
                OperationalMemoryValue::Decision { .. }
            ) | (
                OperationalMemoryKind::Review,
                OperationalMemoryPosture::DecisionControlHistory
                    | OperationalMemoryPosture::Unresolved,
                OperationalMemoryValue::Review { .. }
            ) | (
                OperationalMemoryKind::UnresolvedEffect,
                OperationalMemoryPosture::Unresolved,
                OperationalMemoryValue::UnresolvedEffect { .. }
            ) | (
                OperationalMemoryKind::NormalizationFailure,
                OperationalMemoryPosture::DecisionControlHistory,
                OperationalMemoryValue::NormalizationFailure { .. }
            ) | (
                OperationalMemoryKind::ProviderClaim,
                OperationalMemoryPosture::ProviderOriginatedClaim,
                OperationalMemoryValue::ProviderClaim { .. }
            )
        );
        if !structured_class_matches {
            return Err("memory_representation_structured_class_mismatch".to_string());
        }
        let source_content_digest = digest_json(entry, "memory_representation_source")?;
        let canonical_text = bounded(
            &format!(
                "semantic_kind={}\nauthority_posture={}\nlifecycle={}\n{}\nprovenance_transition_refs={}\nprovenance_observation_refs={}\nprovenance_receipt_refs={}\nprovenance_causal_refs={}",
                entry.semantic_kind.as_str(),
                entry.posture.as_str(),
                entry.lifecycle.as_str(),
                canonical_value_text(&entry.value),
                entry.provenance.transition_ids.join(","),
                entry.provenance.observation_ids.join(","),
                entry.provenance.effect_receipt_ids.join(","),
                entry.provenance.causal_refs.join(",")
            ),
            MAX_REPRESENTATION_CHARS,
        );
        let provenance_refs = entry
            .provenance
            .transition_ids
            .iter()
            .chain(&entry.provenance.observation_ids)
            .chain(&entry.provenance.effect_receipt_ids)
            .chain(&entry.provenance.causal_refs)
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let mut participant_ids = entry.visibility.participant_ids.clone();
        participant_ids.sort();
        participant_ids.dedup();
        let identity = RepresentationDocumentIdentity {
            schema: MEMORY_REPRESENTATION_DOCUMENT_SCHEMA,
            case_id: &entry.case_id,
            memory_id: &entry.memory_id,
            source_content_digest: &source_content_digest,
            operational_memory_derivation_version: &entry.derivation_version,
            representation_contract_version: MEMORY_REPRESENTATION_CONTRACT,
            semantic_kind: entry.semantic_kind.as_str(),
            authority_posture: entry.posture.as_str(),
            lifecycle: entry.lifecycle.as_str(),
            canonical_text: &canonical_text,
            provenance_refs: &provenance_refs,
            participant_ids: &participant_ids,
            consumer: &entry.visibility.consumer,
            view_kind: &entry.visibility.view_kind,
        };
        let digest = digest_json(&identity, "memory_representation_identity")?;
        Ok(Self {
            schema: MEMORY_REPRESENTATION_DOCUMENT_SCHEMA.to_string(),
            document_id: short_id("memory-document", &digest),
            case_id: entry.case_id.clone(),
            memory_id: entry.memory_id.clone(),
            source_content_digest,
            operational_memory_derivation_version: entry.derivation_version.clone(),
            representation_contract_version: MEMORY_REPRESENTATION_CONTRACT.to_string(),
            semantic_kind: entry.semantic_kind.as_str().to_string(),
            authority_posture: entry.posture.as_str().to_string(),
            lifecycle: entry.lifecycle.as_str().to_string(),
            canonical_text,
            provenance_refs,
            participant_ids,
            consumer: entry.visibility.consumer.clone(),
            view_kind: entry.visibility.view_kind.clone(),
        })
    }

    pub fn validate_against(&self, entry: &OperationalMemoryEntry) -> Result<(), String> {
        if Self::from_memory(entry)? != *self {
            return Err("memory_representation_document_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryRepresentationProfile {
    pub schema: String,
    pub profile_id: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub representation_contract_version: String,
    pub encoder_target_id: String,
    pub encoder_model_id: String,
    pub operator_encoder_revision: String,
    pub vector_dimension: usize,
    pub numeric_representation: String,
    pub normalization: String,
    pub distance_metric: String,
    pub query_document_encoding_policy: String,
}

#[derive(Serialize)]
struct RepresentationProfileIdentity<'a> {
    schema: &'a str,
    tenant_id: &'a str,
    representation_contract_version: &'a str,
    encoder_target_id: &'a str,
    encoder_model_id: &'a str,
    operator_encoder_revision: &'a str,
    vector_dimension: usize,
    numeric_representation: &'a str,
    normalization: &'a str,
    distance_metric: &'a str,
    query_document_encoding_policy: &'a str,
}

impl MemoryRepresentationProfile {
    pub fn new(
        tenant_id: &str,
        encoder_target_id: &str,
        encoder_model_id: &str,
        operator_encoder_revision: &str,
        vector_dimension: usize,
    ) -> Result<Self, String> {
        for (label, value) in [
            ("memory_profile_tenant", tenant_id),
            ("memory_profile_encoder_target", encoder_target_id),
            ("memory_profile_encoder_model", encoder_model_id),
            ("memory_profile_encoder_revision", operator_encoder_revision),
        ] {
            require_nonempty(label, value)?;
        }
        if vector_dimension == 0 || vector_dimension > MAX_VECTOR_DIMENSION {
            return Err("memory_profile_vector_dimension_invalid".to_string());
        }
        let identity = RepresentationProfileIdentity {
            schema: MEMORY_REPRESENTATION_PROFILE_SCHEMA,
            tenant_id,
            representation_contract_version: MEMORY_REPRESENTATION_CONTRACT,
            encoder_target_id,
            encoder_model_id,
            operator_encoder_revision,
            vector_dimension,
            numeric_representation: "ieee754-f32-le",
            normalization: "l2-unit",
            distance_metric: "cosine",
            query_document_encoding_policy: "same-profile-query-document.v1",
        };
        let integrity_digest = digest_json(&identity, "memory_profile_identity")?;
        Ok(Self {
            schema: MEMORY_REPRESENTATION_PROFILE_SCHEMA.to_string(),
            profile_id: short_id("memory-profile", &integrity_digest),
            integrity_digest,
            tenant_id: tenant_id.to_string(),
            representation_contract_version: MEMORY_REPRESENTATION_CONTRACT.to_string(),
            encoder_target_id: encoder_target_id.to_string(),
            encoder_model_id: encoder_model_id.to_string(),
            operator_encoder_revision: operator_encoder_revision.to_string(),
            vector_dimension,
            numeric_representation: "ieee754-f32-le".to_string(),
            normalization: "l2-unit".to_string(),
            distance_metric: "cosine".to_string(),
            query_document_encoding_policy: "same-profile-query-document.v1".to_string(),
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        let rebuilt = Self::new(
            &self.tenant_id,
            &self.encoder_target_id,
            &self.encoder_model_id,
            &self.operator_encoder_revision,
            self.vector_dimension,
        )?;
        if rebuilt != *self {
            return Err("memory_representation_profile_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

fn normalized_vector(values: &[f32], dimension: usize) -> Result<Vec<f32>, String> {
    if values.is_empty() || values.len() != dimension || dimension > MAX_VECTOR_DIMENSION {
        return Err("memory_vector_dimension_invalid".to_string());
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err("memory_vector_non_finite".to_string());
    }
    let norm_squared = values
        .iter()
        .map(|value| f64::from(*value) * f64::from(*value))
        .sum::<f64>();
    if !norm_squared.is_finite() || norm_squared <= f64::EPSILON {
        return Err("memory_vector_zero_norm".to_string());
    }
    let norm = norm_squared.sqrt();
    let normalized = values
        .iter()
        .map(|value| (f64::from(*value) / norm) as f32)
        .collect::<Vec<_>>();
    if normalized.iter().any(|value| !value.is_finite()) {
        return Err("memory_vector_normalization_failed".to_string());
    }
    Ok(normalized)
}

fn vector_digest(values: &[f32]) -> String {
    let bytes = values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<_>>();
    digest_bytes(&bytes)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryEmbeddingArtifact {
    pub schema: String,
    pub embedding_id: String,
    pub representation_document_id: String,
    pub representation_document_digest: String,
    pub profile_id: String,
    pub profile_digest: String,
    pub dimension: usize,
    pub normalization: String,
    pub vector_digest: String,
    pub values: Vec<f32>,
    pub encoded_at_derivation: String,
}

impl MemoryEmbeddingArtifact {
    pub fn new(
        document: &MemoryRepresentationDocument,
        profile: &MemoryRepresentationProfile,
        values: &[f32],
    ) -> Result<Self, String> {
        profile.validate()?;
        let values = normalized_vector(values, profile.vector_dimension)?;
        let representation_document_digest = digest_json(document, "memory_embedding_document")?;
        let vector_digest = vector_digest(&values);
        let identity_digest = digest_json(
            &(
                MEMORY_EMBEDDING_SCHEMA,
                &document.document_id,
                &representation_document_digest,
                &profile.profile_id,
                &profile.integrity_digest,
                profile.vector_dimension,
                "l2-unit",
                &vector_digest,
            ),
            "memory_embedding_identity",
        )?;
        Ok(Self {
            schema: MEMORY_EMBEDDING_SCHEMA.to_string(),
            embedding_id: short_id("memory-embedding", &identity_digest),
            representation_document_id: document.document_id.clone(),
            representation_document_digest,
            profile_id: profile.profile_id.clone(),
            profile_digest: profile.integrity_digest.clone(),
            dimension: profile.vector_dimension,
            normalization: "l2-unit".to_string(),
            vector_digest,
            values,
            encoded_at_derivation: MEMORY_INDEX_BUILD_VERSION.to_string(),
        })
    }

    pub fn validate(
        &self,
        document: &MemoryRepresentationDocument,
        profile: &MemoryRepresentationProfile,
    ) -> Result<(), String> {
        profile.validate()?;
        if self.schema != MEMORY_EMBEDDING_SCHEMA
            || self.representation_document_id != document.document_id
            || self.profile_id != profile.profile_id
            || self.profile_digest != profile.integrity_digest
            || self.dimension != profile.vector_dimension
            || self.normalization != "l2-unit"
            || self.encoded_at_derivation != MEMORY_INDEX_BUILD_VERSION
            || self.values.len() != self.dimension
            || self.values.iter().any(|value| !value.is_finite())
        {
            return Err("memory_embedding_integrity_mismatch".to_string());
        }
        let norm = self
            .values
            .iter()
            .map(|value| f64::from(*value) * f64::from(*value))
            .sum::<f64>()
            .sqrt();
        let representation_document_digest = digest_json(document, "memory_embedding_document")?;
        let vector_digest = vector_digest(&self.values);
        let identity_digest = digest_json(
            &(
                MEMORY_EMBEDDING_SCHEMA,
                &document.document_id,
                &representation_document_digest,
                &profile.profile_id,
                &profile.integrity_digest,
                profile.vector_dimension,
                "l2-unit",
                &vector_digest,
            ),
            "memory_embedding_identity",
        )?;
        if !norm.is_finite()
            || (norm - 1.0).abs() > 1e-5
            || self.representation_document_digest != representation_document_digest
            || self.vector_digest != vector_digest
            || self.embedding_id != short_id("memory-embedding", &identity_digest)
        {
            return Err("memory_embedding_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryCorpusItem {
    pub memory_id: String,
    pub document_id: String,
    pub lifecycle: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryCorpusManifest {
    pub schema: String,
    pub manifest_id: String,
    pub corpus_digest: String,
    pub case_id: String,
    pub source_generation: u64,
    pub operational_memory_derivation_version: String,
    pub representation_contract_version: String,
    pub ordered_items: Vec<MemoryCorpusItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryRepresentationCorpus {
    pub manifest: MemoryCorpusManifest,
    pub documents: Vec<MemoryRepresentationDocument>,
}

pub fn derive_representation_corpus(
    build: &OperationalMemoryBuild,
) -> Result<MemoryRepresentationCorpus, String> {
    if build.entries.len() > MAX_CORPUS_DOCUMENTS {
        return Err("memory_corpus_document_bound_exceeded".to_string());
    }
    if build.manifest.schema != crate::memory::OPERATIONAL_MEMORY_MANIFEST_SCHEMA
        || build.manifest.derivation_version != crate::memory::OPERATIONAL_MEMORY_DERIVATION
        || build.manifest.case_id.trim().is_empty()
    {
        return Err("memory_corpus_source_manifest_invalid".to_string());
    }
    let manifest_ids = build.manifest.memory_ids.iter().collect::<BTreeSet<_>>();
    let entry_ids = build
        .entries
        .iter()
        .map(|entry| &entry.memory_id)
        .collect::<BTreeSet<_>>();
    if manifest_ids.len() != build.manifest.memory_ids.len()
        || entry_ids.len() != build.entries.len()
        || manifest_ids != entry_ids
        || build.entries.iter().any(|entry| {
            entry.case_id != build.manifest.case_id
                || entry.derivation_version != build.manifest.derivation_version
                || entry.derived_at_generation != build.manifest.source_generation
        })
    {
        return Err("memory_corpus_source_entries_invalid".to_string());
    }
    let mut documents = build
        .entries
        .iter()
        .map(MemoryRepresentationDocument::from_memory)
        .collect::<Result<Vec<_>, _>>()?;
    documents.sort_by(|left, right| left.memory_id.cmp(&right.memory_id));
    let ordered_items = documents
        .iter()
        .map(|document| MemoryCorpusItem {
            memory_id: document.memory_id.clone(),
            document_id: document.document_id.clone(),
            lifecycle: document.lifecycle.clone(),
        })
        .collect::<Vec<_>>();
    let material = (
        MEMORY_CORPUS_MANIFEST_SCHEMA,
        &build.manifest.case_id,
        build.manifest.source_generation,
        &build.manifest.derivation_version,
        MEMORY_REPRESENTATION_CONTRACT,
        &ordered_items,
    );
    let corpus_digest = digest_json(&material, "memory_corpus_identity")?;
    Ok(MemoryRepresentationCorpus {
        manifest: MemoryCorpusManifest {
            schema: MEMORY_CORPUS_MANIFEST_SCHEMA.to_string(),
            manifest_id: short_id("memory-corpus", &corpus_digest),
            corpus_digest,
            case_id: build.manifest.case_id.clone(),
            source_generation: build.manifest.source_generation,
            operational_memory_derivation_version: build.manifest.derivation_version.clone(),
            representation_contract_version: MEMORY_REPRESENTATION_CONTRACT.to_string(),
            ordered_items,
        },
        documents,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LexicalPosting {
    pub document_id: String,
    pub term_frequency: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryLexicalIndex {
    pub schema: String,
    pub document_count: usize,
    pub total_document_terms: u64,
    pub document_lengths: BTreeMap<String, u32>,
    pub postings: BTreeMap<String, Vec<LexicalPosting>>,
    pub checksum: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LexicalHit {
    pub document_id: String,
    pub score_micros: i64,
    pub matched_terms: Vec<String>,
}

fn tokenize(value: &str) -> Vec<String> {
    value
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_lowercase())
        .take(MAX_REPRESENTATION_CHARS)
        .collect()
}

impl MemoryLexicalIndex {
    pub fn build(documents: &[MemoryRepresentationDocument]) -> Result<Self, String> {
        if documents.len() > MAX_CORPUS_DOCUMENTS {
            return Err("memory_lexical_document_bound_exceeded".to_string());
        }
        let mut document_lengths = BTreeMap::new();
        let mut postings = BTreeMap::<String, Vec<LexicalPosting>>::new();
        let mut total_document_terms = 0u64;
        for document in documents {
            let tokens = tokenize(&document.canonical_text);
            let length = u32::try_from(tokens.len())
                .map_err(|_| "memory_lexical_document_length_overflow".to_string())?;
            total_document_terms = total_document_terms.saturating_add(u64::from(length));
            document_lengths.insert(document.document_id.clone(), length);
            let mut frequencies = BTreeMap::<String, u32>::new();
            for token in tokens {
                *frequencies.entry(token).or_default() += 1;
            }
            for (term, term_frequency) in frequencies {
                postings.entry(term).or_default().push(LexicalPosting {
                    document_id: document.document_id.clone(),
                    term_frequency,
                });
            }
        }
        for values in postings.values_mut() {
            values.sort_by(|left, right| left.document_id.cmp(&right.document_id));
        }
        let checksum = digest_json(
            &(
                MEMORY_LEXICAL_INDEX_SCHEMA,
                documents.len(),
                total_document_terms,
                &document_lengths,
                &postings,
            ),
            "memory_lexical_index",
        )?;
        Ok(Self {
            schema: MEMORY_LEXICAL_INDEX_SCHEMA.to_string(),
            document_count: documents.len(),
            total_document_terms,
            document_lengths,
            postings,
            checksum,
        })
    }

    pub fn validate(&self, documents: &[MemoryRepresentationDocument]) -> Result<(), String> {
        if Self::build(documents)? != *self {
            return Err("memory_lexical_index_integrity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<LexicalHit>, String> {
        let all = self
            .document_lengths
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        self.search_qualified(query, limit, &all)
    }

    pub fn search_qualified(
        &self,
        query: &str,
        limit: usize,
        admitted_document_ids: &BTreeSet<String>,
    ) -> Result<Vec<LexicalHit>, String> {
        if limit == 0 || limit > MAX_CANDIDATES_PER_PLANE {
            return Err("memory_lexical_candidate_bound_invalid".to_string());
        }
        let query_terms = tokenize(&bounded(query, MAX_QUERY_CHARS))
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .take(MAX_QUERY_TERMS)
            .collect::<Vec<_>>();
        if admitted_document_ids
            .iter()
            .any(|document_id| !self.document_lengths.contains_key(document_id))
        {
            return Err("memory_lexical_admitted_document_missing".to_string());
        }
        let admitted_document_count = admitted_document_ids.len();
        let admitted_total_terms = admitted_document_ids.iter().try_fold(0u64, |total, id| {
            total
                .checked_add(u64::from(*self.document_lengths.get(id).ok_or_else(
                    || "memory_lexical_admitted_document_missing".to_string(),
                )?))
                .ok_or_else(|| "memory_lexical_term_count_overflow".to_string())
        })?;
        let n = admitted_document_count as f64;
        let avg_len = if admitted_document_count == 0 {
            1.0
        } else {
            admitted_total_terms as f64 / admitted_document_count as f64
        };
        let mut scores = BTreeMap::<String, (f64, BTreeSet<String>)>::new();
        for term in query_terms {
            let Some(postings) = self.postings.get(&term) else {
                continue;
            };
            let admitted_postings = postings
                .iter()
                .filter(|posting| admitted_document_ids.contains(&posting.document_id))
                .collect::<Vec<_>>();
            let df = admitted_postings.len() as f64;
            if df == 0.0 {
                continue;
            }
            let idf = (1.0 + (n - df + 0.5) / (df + 0.5)).ln();
            for posting in admitted_postings {
                let dl = f64::from(
                    *self
                        .document_lengths
                        .get(&posting.document_id)
                        .ok_or_else(|| "memory_lexical_posting_document_missing".to_string())?,
                );
                let tf = f64::from(posting.term_frequency);
                let k1 = 1.2;
                let b = 0.75;
                let score = idf * (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * dl / avg_len));
                let entry = scores.entry(posting.document_id.clone()).or_default();
                entry.0 += score;
                entry.1.insert(term.clone());
            }
        }
        let mut hits = scores
            .into_iter()
            .map(|(document_id, (score, terms))| LexicalHit {
                document_id,
                score_micros: (score * 1_000_000.0).round() as i64,
                matched_terms: terms.into_iter().collect(),
            })
            .collect::<Vec<_>>();
        hits.sort_by(|left, right| {
            right
                .score_micros
                .cmp(&left.score_micros)
                .then_with(|| left.document_id.cmp(&right.document_id))
        });
        hits.truncate(limit);
        Ok(hits)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryVectorIndex {
    pub schema: String,
    pub profile_id: String,
    pub dimension: usize,
    pub embeddings: Vec<MemoryEmbeddingArtifact>,
    pub checksum: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExactVectorHit {
    pub document_id: String,
    pub similarity_micros: i64,
}

impl MemoryVectorIndex {
    pub fn build(
        documents: &[MemoryRepresentationDocument],
        profile: &MemoryRepresentationProfile,
        vectors: &BTreeMap<String, Vec<f32>>,
    ) -> Result<Self, String> {
        profile.validate()?;
        let vector_elements = documents
            .len()
            .checked_mul(profile.vector_dimension)
            .ok_or_else(|| "memory_index_vector_element_budget_overflow".to_string())?;
        if vector_elements > MAX_VECTOR_ELEMENTS {
            return Err("memory_index_vector_element_budget_exceeded".to_string());
        }
        if documents.len() != vectors.len() {
            return Err("memory_vector_corpus_cardinality_mismatch".to_string());
        }
        let mut embeddings = Vec::with_capacity(documents.len());
        for document in documents {
            let vector = vectors
                .get(&document.document_id)
                .ok_or_else(|| "memory_vector_document_missing".to_string())?;
            embeddings.push(MemoryEmbeddingArtifact::new(document, profile, vector)?);
        }
        embeddings.sort_by(|left, right| {
            left.representation_document_id
                .cmp(&right.representation_document_id)
        });
        let checksum = digest_json(
            &(
                MEMORY_VECTOR_INDEX_SCHEMA,
                &profile.profile_id,
                profile.vector_dimension,
                &embeddings,
            ),
            "memory_vector_index",
        )?;
        Ok(Self {
            schema: MEMORY_VECTOR_INDEX_SCHEMA.to_string(),
            profile_id: profile.profile_id.clone(),
            dimension: profile.vector_dimension,
            embeddings,
            checksum,
        })
    }

    pub fn validate(
        &self,
        documents: &[MemoryRepresentationDocument],
        profile: &MemoryRepresentationProfile,
    ) -> Result<(), String> {
        profile.validate()?;
        if self.schema != MEMORY_VECTOR_INDEX_SCHEMA
            || self.profile_id != profile.profile_id
            || self.dimension != profile.vector_dimension
            || self.embeddings.len() != documents.len()
        {
            return Err("memory_vector_index_integrity_mismatch".to_string());
        }
        let documents = documents
            .iter()
            .map(|document| (document.document_id.as_str(), document))
            .collect::<BTreeMap<_, _>>();
        let mut previous = None;
        for embedding in &self.embeddings {
            if previous
                .is_some_and(|value: &str| value >= embedding.representation_document_id.as_str())
            {
                return Err("memory_vector_index_order_invalid".to_string());
            }
            let document = documents
                .get(embedding.representation_document_id.as_str())
                .ok_or_else(|| "memory_vector_document_missing".to_string())?;
            embedding.validate(document, profile)?;
            previous = Some(embedding.representation_document_id.as_str());
        }
        let checksum = digest_json(
            &(
                MEMORY_VECTOR_INDEX_SCHEMA,
                &profile.profile_id,
                profile.vector_dimension,
                &self.embeddings,
            ),
            "memory_vector_index",
        )?;
        if self.checksum != checksum {
            return Err("memory_vector_index_integrity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn exact_search(
        &self,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<ExactVectorHit>, String> {
        let all = self
            .embeddings
            .iter()
            .map(|embedding| embedding.representation_document_id.clone())
            .collect::<BTreeSet<_>>();
        self.exact_search_qualified(query_vector, limit, &all)
    }

    pub fn exact_search_qualified(
        &self,
        query_vector: &[f32],
        limit: usize,
        admitted_document_ids: &BTreeSet<String>,
    ) -> Result<Vec<ExactVectorHit>, String> {
        if limit == 0 || limit > MAX_CANDIDATES_PER_PLANE {
            return Err("memory_vector_candidate_bound_invalid".to_string());
        }
        let query = normalized_vector(query_vector, self.dimension)?;
        let mut hits = self
            .embeddings
            .iter()
            .filter(|embedding| {
                admitted_document_ids.contains(&embedding.representation_document_id)
            })
            .map(|embedding| {
                if embedding.dimension != self.dimension
                    || embedding.values.len() != self.dimension
                    || embedding.values.iter().any(|value| !value.is_finite())
                {
                    return Err("memory_vector_index_item_invalid".to_string());
                }
                let similarity = query
                    .iter()
                    .zip(&embedding.values)
                    .map(|(left, right)| f64::from(*left) * f64::from(*right))
                    .sum::<f64>()
                    .clamp(-1.0, 1.0);
                Ok(ExactVectorHit {
                    document_id: embedding.representation_document_id.clone(),
                    similarity_micros: (similarity * 1_000_000.0).round() as i64,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        hits.sort_by(|left, right| {
            right
                .similarity_micros
                .cmp(&left.similarity_micros)
                .then_with(|| left.document_id.cmp(&right.document_id))
        });
        hits.truncate(limit);
        Ok(hits)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnPosture {
    DeferredExactScanWithinBound,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryIndexManifest {
    pub schema: String,
    pub index_id: String,
    pub content_checksum: String,
    pub case_id: String,
    pub source_corpus_manifest_id: String,
    pub source_corpus_digest: String,
    pub representation_profile_id: String,
    pub representation_profile_digest: String,
    pub index_types: Vec<String>,
    pub item_count: usize,
    pub dimension: usize,
    pub exact_build_version: String,
    pub lexical_checksum: String,
    pub vector_checksum: String,
    pub ann_posture: AnnPosture,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryIndexBundle {
    pub schema: String,
    pub corpus: MemoryCorpusManifest,
    pub profile: MemoryRepresentationProfile,
    pub documents: Vec<MemoryRepresentationDocument>,
    pub lexical: MemoryLexicalIndex,
    pub vector: MemoryVectorIndex,
    pub manifest: MemoryIndexManifest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryIndexBuildFailpoint {
    AfterCorpusManifest,
    DuringLexicalBuild,
    DuringEmbeddingGeneration,
    DuringVectorSerialization,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryIndexPublicationFailpoint {
    AfterComponentWriteBeforeFileSync,
    AfterFileSyncBeforeBuildDirectorySync,
    AfterBuildRenameBeforeBuildsParentSync,
    AfterBuildsSyncBeforePointerWrite,
    DuringPointerTempWrite,
    AfterPointerRenameBeforeProfileDirectorySync,
    AfterFinalSyncBeforeAcknowledgement,
    AfterCompleteTempBeforePublish,
}

impl MemoryIndexBundle {
    pub fn build(
        corpus: MemoryRepresentationCorpus,
        profile: MemoryRepresentationProfile,
        vectors: &BTreeMap<String, Vec<f32>>,
    ) -> Result<Self, String> {
        Self::build_with_failpoint(corpus, profile, vectors, None)
    }

    pub fn build_with_failpoint(
        corpus: MemoryRepresentationCorpus,
        profile: MemoryRepresentationProfile,
        vectors: &BTreeMap<String, Vec<f32>>,
        failpoint: Option<MemoryIndexBuildFailpoint>,
    ) -> Result<Self, String> {
        validate_memory_index_build_budget(&corpus, &profile)?;
        if failpoint == Some(MemoryIndexBuildFailpoint::AfterCorpusManifest) {
            return Err("memory_index_failpoint_after_corpus_manifest".to_string());
        }
        if corpus.manifest.case_id.is_empty()
            || corpus.documents.len() != corpus.manifest.ordered_items.len()
        {
            return Err("memory_corpus_manifest_invalid".to_string());
        }
        if failpoint == Some(MemoryIndexBuildFailpoint::DuringLexicalBuild) {
            return Err("memory_index_failpoint_during_lexical_build".to_string());
        }
        let lexical = MemoryLexicalIndex::build(&corpus.documents)?;
        if failpoint == Some(MemoryIndexBuildFailpoint::DuringEmbeddingGeneration) {
            return Err("memory_index_failpoint_during_embedding_generation".to_string());
        }
        let vector = MemoryVectorIndex::build(&corpus.documents, &profile, vectors)?;
        if failpoint == Some(MemoryIndexBuildFailpoint::DuringVectorSerialization) {
            return Err("memory_index_failpoint_during_vector_serialization".to_string());
        }
        let content_checksum = digest_json(
            &(
                MEMORY_INDEX_BUNDLE_SCHEMA,
                &corpus.manifest,
                &profile,
                &corpus.documents,
                &lexical,
                &vector,
                MEMORY_INDEX_BUILD_VERSION,
                &AnnPosture::DeferredExactScanWithinBound,
            ),
            "memory_index_content",
        )?;
        let manifest = MemoryIndexManifest {
            schema: MEMORY_INDEX_MANIFEST_SCHEMA.to_string(),
            index_id: short_id("memory-index", &content_checksum),
            content_checksum,
            case_id: corpus.manifest.case_id.clone(),
            source_corpus_manifest_id: corpus.manifest.manifest_id.clone(),
            source_corpus_digest: corpus.manifest.corpus_digest.clone(),
            representation_profile_id: profile.profile_id.clone(),
            representation_profile_digest: profile.integrity_digest.clone(),
            index_types: vec![
                "lexical_bm25".to_string(),
                "vector_exact_cosine".to_string(),
            ],
            item_count: corpus.documents.len(),
            dimension: profile.vector_dimension,
            exact_build_version: MEMORY_INDEX_BUILD_VERSION.to_string(),
            lexical_checksum: lexical.checksum.clone(),
            vector_checksum: vector.checksum.clone(),
            ann_posture: AnnPosture::DeferredExactScanWithinBound,
        };
        let bundle = Self {
            schema: MEMORY_INDEX_BUNDLE_SCHEMA.to_string(),
            corpus: corpus.manifest,
            profile,
            documents: corpus.documents,
            lexical,
            vector,
            manifest,
        };
        bundle.validate_deep()?;
        Ok(bundle)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.validate_deep()
    }

    /// Bounded structural validation for a bundle whose physical components
    /// were already length/checksum sealed. Deep reconstruction belongs to
    /// publication and explicit verification, not the query hot path.
    pub fn validate_loaded(&self) -> Result<(), String> {
        if self.schema != MEMORY_INDEX_BUNDLE_SCHEMA
            || self.corpus.schema != MEMORY_CORPUS_MANIFEST_SCHEMA
            || self.manifest.schema != MEMORY_INDEX_MANIFEST_SCHEMA
            || self.documents.len() != self.corpus.ordered_items.len()
            || self.documents.len() != self.manifest.item_count
            || self.documents.len() > MAX_CORPUS_DOCUMENTS
        {
            return Err("memory_index_manifest_invalid".to_string());
        }
        self.profile.validate()?;
        let document_by_memory = self
            .documents
            .iter()
            .map(|document| (document.memory_id.as_str(), document))
            .collect::<BTreeMap<_, _>>();
        if document_by_memory.len() != self.documents.len()
            || self
                .documents
                .iter()
                .any(|document| document.case_id != self.corpus.case_id)
            || self.corpus.representation_contract_version != MEMORY_REPRESENTATION_CONTRACT
            || self.manifest.index_types
                != [
                    "lexical_bm25".to_string(),
                    "vector_exact_cosine".to_string(),
                ]
            || self.manifest.ann_posture != AnnPosture::DeferredExactScanWithinBound
        {
            return Err("memory_index_manifest_invalid".to_string());
        }
        for item in &self.corpus.ordered_items {
            let document = document_by_memory
                .get(item.memory_id.as_str())
                .ok_or_else(|| "memory_corpus_document_missing".to_string())?;
            if document.document_id != item.document_id || document.lifecycle != item.lifecycle {
                return Err("memory_corpus_item_integrity_mismatch".to_string());
            }
        }
        let corpus_material = (
            MEMORY_CORPUS_MANIFEST_SCHEMA,
            &self.corpus.case_id,
            self.corpus.source_generation,
            &self.corpus.operational_memory_derivation_version,
            MEMORY_REPRESENTATION_CONTRACT,
            &self.corpus.ordered_items,
        );
        let corpus_digest = digest_json(&corpus_material, "memory_corpus_identity")?;
        if self.corpus.corpus_digest != corpus_digest
            || self.corpus.manifest_id != short_id("memory-corpus", &corpus_digest)
        {
            return Err("memory_corpus_manifest_integrity_mismatch".to_string());
        }
        if self.lexical.schema != MEMORY_LEXICAL_INDEX_SCHEMA
            || self.lexical.document_count != self.documents.len()
            || self.lexical.document_lengths.len() != self.documents.len()
            || self.lexical.checksum != self.manifest.lexical_checksum
            || self.vector.schema != MEMORY_VECTOR_INDEX_SCHEMA
            || self.vector.profile_id != self.profile.profile_id
            || self.vector.dimension != self.profile.vector_dimension
            || self.vector.embeddings.len() != self.documents.len()
            || self.vector.checksum != self.manifest.vector_checksum
        {
            return Err("memory_index_loaded_component_mismatch".to_string());
        }
        let document_ids = self
            .documents
            .iter()
            .map(|document| document.document_id.as_str())
            .collect::<BTreeSet<_>>();
        if document_ids.len() != self.documents.len()
            || self
                .lexical
                .document_lengths
                .keys()
                .any(|id| !document_ids.contains(id.as_str()))
        {
            return Err("memory_index_loaded_document_identity_mismatch".to_string());
        }
        let mut previous = None;
        for embedding in &self.vector.embeddings {
            if previous
                .is_some_and(|value: &str| value >= embedding.representation_document_id.as_str())
                || !document_ids.contains(embedding.representation_document_id.as_str())
                || embedding.dimension != self.profile.vector_dimension
                || embedding.values.len() != self.profile.vector_dimension
                || embedding.values.iter().any(|value| !value.is_finite())
            {
                return Err("memory_index_loaded_vector_mismatch".to_string());
            }
            previous = Some(embedding.representation_document_id.as_str());
        }
        if self.manifest.content_checksum.is_empty()
            || self.manifest.index_id != short_id("memory-index", &self.manifest.content_checksum)
            || self.manifest.case_id != self.corpus.case_id
            || self.manifest.source_corpus_manifest_id != self.corpus.manifest_id
            || self.manifest.source_corpus_digest != self.corpus.corpus_digest
            || self.manifest.representation_profile_id != self.profile.profile_id
            || self.manifest.representation_profile_digest != self.profile.integrity_digest
            || self.manifest.dimension != self.profile.vector_dimension
            || self.manifest.exact_build_version != MEMORY_INDEX_BUILD_VERSION
        {
            return Err("memory_index_content_integrity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn validate_deep(&self) -> Result<(), String> {
        self.validate_loaded()?;
        self.lexical.validate(&self.documents)?;
        self.vector.validate(&self.documents, &self.profile)?;
        let content_checksum = digest_json(
            &(
                MEMORY_INDEX_BUNDLE_SCHEMA,
                &self.corpus,
                &self.profile,
                &self.documents,
                &self.lexical,
                &self.vector,
                MEMORY_INDEX_BUILD_VERSION,
                &AnnPosture::DeferredExactScanWithinBound,
            ),
            "memory_index_content",
        )?;
        if self.manifest.content_checksum != content_checksum
            || self.manifest.index_id != short_id("memory-index", &content_checksum)
            || self.manifest.case_id != self.corpus.case_id
            || self.manifest.source_corpus_manifest_id != self.corpus.manifest_id
            || self.manifest.source_corpus_digest != self.corpus.corpus_digest
            || self.manifest.representation_profile_id != self.profile.profile_id
            || self.manifest.representation_profile_digest != self.profile.integrity_digest
            || self.manifest.dimension != self.profile.vector_dimension
            || self.manifest.lexical_checksum != self.lexical.checksum
            || self.manifest.vector_checksum != self.vector.checksum
            || self.manifest.exact_build_version != MEMORY_INDEX_BUILD_VERSION
        {
            return Err("memory_index_content_integrity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn is_current(&self, case_id: &str, generation: u64) -> bool {
        self.corpus.case_id == case_id && self.corpus.source_generation == generation
    }
}

pub fn validate_memory_index_source(
    index: &MemoryIndexBundle,
    entries: &[OperationalMemoryEntry],
) -> Result<(), String> {
    index.validate_loaded()?;
    let scoped_entries = entries
        .iter()
        .filter(|entry| entry.case_id == index.corpus.case_id)
        .collect::<Vec<_>>();
    if scoped_entries.len() != index.documents.len() {
        return Err("memory_index_source_cardinality_divergent".to_string());
    }
    let current = scoped_entries
        .iter()
        .map(|entry| (entry.memory_id.as_str(), *entry))
        .collect::<BTreeMap<_, _>>();
    if current.len() != scoped_entries.len() {
        return Err("memory_index_source_memory_identity_duplicate".to_string());
    }
    for document in &index.documents {
        let entry = current
            .get(document.memory_id.as_str())
            .ok_or_else(|| "memory_index_source_memory_missing".to_string())?;
        document
            .validate_against(entry)
            .map_err(|_| "memory_index_source_divergent".to_string())?;
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RetrievalQueryDocument {
    pub schema: String,
    pub query_id: String,
    pub text_digest: String,
    pub representation_contract_version: String,
    pub canonical_text: String,
}

impl RetrievalQueryDocument {
    pub fn new(query: &str) -> Result<Self, String> {
        require_nonempty("memory_retrieval_query", query)?;
        let canonical_text = canonicalize_query(query);
        require_nonempty("memory_retrieval_query", &canonical_text)?;
        let text_digest = digest_bytes(canonical_text.as_bytes());
        let digest = digest_json(
            &(
                RETRIEVAL_QUERY_DOCUMENT_SCHEMA,
                MEMORY_REPRESENTATION_CONTRACT,
                &text_digest,
                &canonical_text,
            ),
            "memory_query_identity",
        )?;
        Ok(Self {
            schema: RETRIEVAL_QUERY_DOCUMENT_SCHEMA.to_string(),
            query_id: short_id("memory-query", &digest),
            text_digest,
            representation_contract_version: MEMORY_REPRESENTATION_CONTRACT.to_string(),
            canonical_text,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RetrievalPlaneStatus {
    pub plane: String,
    pub available: bool,
    pub candidate_count: usize,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CandidatePlaneRank {
    pub plane: String,
    pub rank: usize,
    pub plane_score_micros: i64,
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HybridRetrievedMemory {
    pub memory: OperationalMemoryEntry,
    pub fusion_score_micros: u64,
    pub plane_ranks: Vec<CandidatePlaneRank>,
    pub ranking_reasons: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PublicRetrievalRejections {
    pub stale_derivation: usize,
    pub lifecycle: usize,
    pub semantic_kind: usize,
    pub resource_qualification: usize,
    pub causal_qualification: usize,
    pub restricted_rejection_counts_redacted: bool,
}

impl From<&RetrievalRejections> for PublicRetrievalRejections {
    fn from(value: &RetrievalRejections) -> Self {
        Self {
            stale_derivation: value.future_or_stale_derivation,
            lifecycle: value.lifecycle,
            semantic_kind: value.semantic_kind,
            resource_qualification: value.resource_qualification,
            causal_qualification: value.causal_qualification,
            // This is an invariant, not an indication that restricted items
            // exist. Wrong-Case and visibility rejection counts are never
            // exposed because even their cardinality can disclose memory.
            restricted_rejection_counts_redacted: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HybridRetrievalSet {
    pub schema: String,
    pub retrieval_id: String,
    pub fusion_version: String,
    pub case_id: String,
    pub case_generation: u64,
    pub participant_id: String,
    pub query: RetrievalQueryDocument,
    pub corpus_manifest_id: Option<String>,
    pub representation_profile_id: Option<String>,
    pub index_manifest_id: Option<String>,
    pub planes: Vec<RetrievalPlaneStatus>,
    pub qualification_rejections: PublicRetrievalRejections,
    pub qualified_count: usize,
    pub selected_count: usize,
    pub omitted_count: usize,
    pub selected_memory_ids: Vec<String>,
    pub selected: Vec<HybridRetrievedMemory>,
}

#[derive(Default)]
struct FusionCandidate {
    plane_ranks: Vec<CandidatePlaneRank>,
    fusion_score_micros: u64,
    exact_anchor: bool,
}

fn add_rank(
    candidates: &mut BTreeMap<String, FusionCandidate>,
    memory_id: &str,
    rank: CandidatePlaneRank,
    exact_anchor: bool,
) -> Result<(), String> {
    if rank.rank == 0 || rank.rank > MAX_CANDIDATES_PER_PLANE {
        return Err("memory_hybrid_plane_rank_invalid".to_string());
    }
    let candidate = candidates.entry(memory_id.to_string()).or_default();
    if candidate
        .plane_ranks
        .iter()
        .any(|existing| existing.plane == rank.plane)
    {
        return Err("memory_hybrid_duplicate_plane_candidate".to_string());
    }
    let rank_u64 =
        u64::try_from(rank.rank).map_err(|_| "memory_hybrid_plane_rank_overflow".to_string())?;
    let contribution = RRF_SCALE
        .checked_div(
            RRF_K
                .checked_add(rank_u64)
                .ok_or_else(|| "memory_hybrid_rank_denominator_overflow".to_string())?,
        )
        .ok_or_else(|| "memory_hybrid_rank_denominator_invalid".to_string())?;
    candidate.fusion_score_micros = candidate
        .fusion_score_micros
        .checked_add(contribution)
        .ok_or_else(|| "memory_hybrid_fusion_score_overflow".to_string())?;
    if exact_anchor {
        candidate.fusion_score_micros = candidate
            .fusion_score_micros
            .checked_add(EXACT_ANCHOR_BONUS)
            .ok_or_else(|| "memory_hybrid_anchor_score_overflow".to_string())?;
        candidate.exact_anchor = true;
    }
    candidate.plane_ranks.push(rank);
    Ok(())
}

pub fn hybrid_retrieve(
    state: &CaseState,
    entries: &[OperationalMemoryEntry],
    qualification: RetrievalQualification,
    query: RetrievalQueryDocument,
    index: Option<&MemoryIndexBundle>,
    query_vector: Result<Option<Vec<f32>>, String>,
) -> Result<HybridRetrievalSet, String> {
    if qualification.max_results == 0 {
        return Err("retrieval_max_results_must_be_positive".to_string());
    }
    let candidate_cap = qualification
        .max_results
        .saturating_mul(4)
        .max(qualification.max_results)
        .min(MAX_CANDIDATES_PER_PLANE)
        .min(MAX_TOTAL_CANDIDATES / 3);
    let mut all_qualification = qualification.clone();
    all_qualification.max_results = entries.len().max(1);
    let qualified = retrieve_operational_memory(state, entries, all_qualification)?;
    let eligible = qualified
        .selected
        .iter()
        .map(|item| (item.memory.memory_id.clone(), item))
        .collect::<BTreeMap<_, _>>();
    let direct_anchor =
        !qualification.resource_refs.is_empty() || !qualification.causal_refs.is_empty();
    let mut candidates = BTreeMap::<String, FusionCandidate>::new();
    let mut exact_count = 0usize;
    for (position, item) in qualified.selected.iter().take(candidate_cap).enumerate() {
        exact_count += 1;
        add_rank(
            &mut candidates,
            &item.memory.memory_id,
            CandidatePlaneRank {
                plane: "exact_operational".to_string(),
                rank: position + 1,
                plane_score_micros: item.score.saturating_mul(1_000_000),
                evidence: item.ranking_reasons.clone(),
            },
            direct_anchor,
        )?;
    }

    let mut planes = vec![RetrievalPlaneStatus {
        plane: "exact_operational".to_string(),
        available: true,
        candidate_count: exact_count,
        reason: if direct_anchor {
            "qualified_direct_causal_or_resource_anchor".to_string()
        } else {
            "qualified_operational_reference_order".to_string()
        },
    }];
    let mut corpus_manifest_id = None;
    let mut representation_profile_id = None;
    let mut index_manifest_id = None;

    if let Some(index) = index {
        index.validate_loaded()?;
        if !index.is_current(&qualification.case_id, qualification.case_generation) {
            return Err("memory_index_stale_for_case_generation".to_string());
        }
        validate_memory_index_source(index, entries)?;
        corpus_manifest_id = Some(index.corpus.manifest_id.clone());
        representation_profile_id = Some(index.profile.profile_id.clone());
        index_manifest_id = Some(index.manifest.index_id.clone());
        let document_by_memory = index
            .documents
            .iter()
            .map(|document| (document.memory_id.as_str(), document))
            .collect::<BTreeMap<_, _>>();
        let mut document_memory = BTreeMap::<String, String>::new();
        let mut admitted_document_ids = BTreeSet::new();
        for (memory_id, item) in &eligible {
            let document = document_by_memory
                .get(memory_id.as_str())
                .ok_or_else(|| "memory_index_qualified_source_document_missing".to_string())?;
            document
                .validate_against(&item.memory)
                .map_err(|_| "memory_index_source_divergent".to_string())?;
            if !admitted_document_ids.insert(document.document_id.clone()) {
                return Err("memory_index_qualified_document_duplicate".to_string());
            }
            document_memory.insert(document.document_id.clone(), (*memory_id).clone());
        }
        let lexical = index.lexical.search_qualified(
            &query.canonical_text,
            candidate_cap,
            &admitted_document_ids,
        )?;
        let mut lexical_count = 0usize;
        for hit in lexical {
            let Some(memory_id) = document_memory.get(hit.document_id.as_str()) else {
                return Err("memory_lexical_candidate_document_missing".to_string());
            };
            lexical_count += 1;
            add_rank(
                &mut candidates,
                memory_id,
                CandidatePlaneRank {
                    plane: "lexical_bm25".to_string(),
                    rank: lexical_count,
                    plane_score_micros: hit.score_micros,
                    evidence: hit
                        .matched_terms
                        .into_iter()
                        .map(|term| format!("term:{term}"))
                        .collect(),
                },
                false,
            )?;
        }
        planes.push(RetrievalPlaneStatus {
            plane: "lexical_bm25".to_string(),
            available: true,
            candidate_count: lexical_count,
            reason: "sealed_bm25_index".to_string(),
        });

        match query_vector {
            Ok(Some(vector)) => {
                let vector_hits = index.vector.exact_search_qualified(
                    &vector,
                    candidate_cap,
                    &admitted_document_ids,
                )?;
                let mut vector_count = 0usize;
                for hit in vector_hits {
                    let Some(memory_id) = document_memory.get(hit.document_id.as_str()) else {
                        return Err("memory_vector_candidate_document_missing".to_string());
                    };
                    vector_count += 1;
                    add_rank(
                        &mut candidates,
                        memory_id,
                        CandidatePlaneRank {
                            plane: "vector_exact_cosine".to_string(),
                            rank: vector_count,
                            plane_score_micros: hit.similarity_micros,
                            evidence: vec![format!(
                                "cosine_similarity_micros:{}",
                                hit.similarity_micros
                            )],
                        },
                        false,
                    )?;
                }
                planes.push(RetrievalPlaneStatus {
                    plane: "vector_exact_cosine".to_string(),
                    available: true,
                    candidate_count: vector_count,
                    reason: "exact_scan_reference".to_string(),
                });
            }
            Ok(None) => planes.push(RetrievalPlaneStatus {
                plane: "vector_exact_cosine".to_string(),
                available: false,
                candidate_count: 0,
                reason: "query_vector_unavailable".to_string(),
            }),
            Err(error) => planes.push(RetrievalPlaneStatus {
                plane: "vector_exact_cosine".to_string(),
                available: false,
                candidate_count: 0,
                reason: format!("query_encoder_unavailable:{error}"),
            }),
        }
        planes.push(RetrievalPlaneStatus {
            plane: "ann".to_string(),
            available: false,
            candidate_count: 0,
            reason: "deferred_exact_scan_within_50000_document_bound".to_string(),
        });
    } else {
        planes.extend([
            RetrievalPlaneStatus {
                plane: "lexical_bm25".to_string(),
                available: false,
                candidate_count: 0,
                reason: "index_unavailable".to_string(),
            },
            RetrievalPlaneStatus {
                plane: "vector_exact_cosine".to_string(),
                available: false,
                candidate_count: 0,
                reason: "index_unavailable".to_string(),
            },
            RetrievalPlaneStatus {
                plane: "ann".to_string(),
                available: false,
                candidate_count: 0,
                reason: "not_admitted".to_string(),
            },
        ]);
    }

    if candidates.len() > MAX_TOTAL_CANDIDATES {
        return Err("memory_hybrid_total_candidate_bound_exceeded".to_string());
    }
    let memory_by_id = entries
        .iter()
        .map(|entry| (entry.memory_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut selected = candidates
        .into_iter()
        .map(|(memory_id, mut candidate)| {
            candidate
                .plane_ranks
                .sort_by(|left, right| left.plane.cmp(&right.plane));
            let memory = memory_by_id
                .get(memory_id.as_str())
                .ok_or_else(|| "memory_hybrid_candidate_missing".to_string())?;
            let mut ranking_reasons = candidate
                .plane_ranks
                .iter()
                .map(|rank| format!("{}:rank={}", rank.plane, rank.rank))
                .collect::<Vec<_>>();
            if candidate.exact_anchor {
                ranking_reasons.push("direct_anchor_privileged".to_string());
            }
            ranking_reasons.push(format!(
                "rrf_k_{RRF_K}:fusion_micros={}",
                candidate.fusion_score_micros
            ));
            Ok((
                candidate.exact_anchor,
                HybridRetrievedMemory {
                    memory: (*memory).clone(),
                    fusion_score_micros: candidate.fusion_score_micros,
                    plane_ranks: candidate.plane_ranks,
                    ranking_reasons,
                },
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    selected.sort_by(|(left_anchor, left), (right_anchor, right)| {
        right_anchor
            .cmp(left_anchor)
            .then_with(|| right.fusion_score_micros.cmp(&left.fusion_score_micros))
            .then_with(|| {
                right
                    .memory
                    .provenance
                    .generation_end
                    .cmp(&left.memory.provenance.generation_end)
            })
            .then_with(|| left.memory.memory_id.cmp(&right.memory.memory_id))
    });
    let qualified_count = qualified.qualified_count;
    let selected = selected
        .into_iter()
        .take(qualification.max_results)
        .map(|(_, item)| item)
        .collect::<Vec<_>>();
    let selected_memory_ids = selected
        .iter()
        .map(|item| item.memory.memory_id.clone())
        .collect::<Vec<_>>();
    let omitted_count = qualified_count.saturating_sub(selected.len());
    let qualification_rejections = PublicRetrievalRejections::from(&qualified.rejections);
    let identity_digest = digest_json(
        &(
            HYBRID_RETRIEVAL_SET_SCHEMA,
            HYBRID_FUSION_VERSION,
            &qualification,
            &query,
            &corpus_manifest_id,
            &representation_profile_id,
            &index_manifest_id,
            &planes,
            &qualification_rejections,
            qualified_count,
            omitted_count,
            &selected,
        ),
        "hybrid_retrieval_identity",
    )?;
    Ok(HybridRetrievalSet {
        schema: HYBRID_RETRIEVAL_SET_SCHEMA.to_string(),
        retrieval_id: short_id("retrieval", &identity_digest),
        fusion_version: HYBRID_FUSION_VERSION.to_string(),
        case_id: qualification.case_id.clone(),
        case_generation: qualification.case_generation,
        participant_id: qualification.participant_id.clone(),
        query,
        corpus_manifest_id,
        representation_profile_id,
        index_manifest_id,
        planes,
        qualification_rejections,
        qualified_count,
        selected_count: selected.len(),
        omitted_count,
        selected_memory_ids,
        selected,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct CurrentIndexPointer {
    schema: String,
    storage_format: String,
    case_id: String,
    profile_id: String,
    index_id: String,
    content_checksum: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct PhysicalComponentSeal {
    name: String,
    size_bytes: usize,
    checksum: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct PhysicalIndexSeal {
    schema: String,
    index_id: String,
    content_checksum: String,
    item_count: usize,
    dimension: usize,
    vector_elements: usize,
    components: Vec<PhysicalComponentSeal>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct StoredEmbeddingMetadata {
    schema: String,
    embedding_id: String,
    representation_document_id: String,
    representation_document_digest: String,
    profile_id: String,
    profile_digest: String,
    dimension: usize,
    normalization: String,
    vector_digest: String,
    encoded_at_derivation: String,
}

#[derive(Serialize)]
struct StoredEmbeddingMetadataRef<'a> {
    schema: &'a str,
    embedding_id: &'a str,
    representation_document_id: &'a str,
    representation_document_digest: &'a str,
    profile_id: &'a str,
    profile_digest: &'a str,
    dimension: usize,
    normalization: &'a str,
    vector_digest: &'a str,
    encoded_at_derivation: &'a str,
}

impl<'a> From<&'a MemoryEmbeddingArtifact> for StoredEmbeddingMetadataRef<'a> {
    fn from(value: &'a MemoryEmbeddingArtifact) -> Self {
        Self {
            schema: &value.schema,
            embedding_id: &value.embedding_id,
            representation_document_id: &value.representation_document_id,
            representation_document_digest: &value.representation_document_digest,
            profile_id: &value.profile_id,
            profile_digest: &value.profile_digest,
            dimension: value.dimension,
            normalization: &value.normalization,
            vector_digest: &value.vector_digest,
            encoded_at_derivation: &value.encoded_at_derivation,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct StoredVectorIndex {
    schema: String,
    profile_id: String,
    dimension: usize,
    embeddings: Vec<StoredEmbeddingMetadata>,
    checksum: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct StoredIndexMetadata {
    storage_schema: String,
    logical_schema: String,
    corpus: MemoryCorpusManifest,
    profile: MemoryRepresentationProfile,
    documents: Vec<MemoryRepresentationDocument>,
    lexical: MemoryLexicalIndex,
    vector: StoredVectorIndex,
    manifest: MemoryIndexManifest,
}

impl StoredIndexMetadata {
    fn into_bundle(self, vector_bytes: &[u8]) -> Result<MemoryIndexBundle, String> {
        if self.storage_schema != DERIVED_MEMORY_PHYSICAL_SCHEMA
            || self.logical_schema != MEMORY_INDEX_BUNDLE_SCHEMA
        {
            return Err("memory_index_physical_metadata_schema_mismatch".to_string());
        }
        let expected_elements = self
            .vector
            .embeddings
            .len()
            .checked_mul(self.vector.dimension)
            .ok_or_else(|| "memory_index_vector_element_budget_overflow".to_string())?;
        if expected_elements > MAX_VECTOR_ELEMENTS {
            return Err("memory_index_vector_element_budget_exceeded".to_string());
        }
        let expected_bytes = expected_elements
            .checked_mul(std::mem::size_of::<f32>())
            .ok_or_else(|| "memory_index_vector_byte_budget_overflow".to_string())?;
        if vector_bytes.len() != expected_bytes {
            return Err("memory_index_vector_binary_length_mismatch".to_string());
        }
        let mut offset = 0usize;
        let mut embeddings = Vec::with_capacity(self.vector.embeddings.len());
        for metadata in self.vector.embeddings {
            if metadata.dimension != self.vector.dimension {
                return Err("memory_index_vector_binary_dimension_mismatch".to_string());
            }
            let mut values = Vec::with_capacity(self.vector.dimension);
            for _ in 0..self.vector.dimension {
                let end = offset
                    .checked_add(std::mem::size_of::<f32>())
                    .ok_or_else(|| "memory_index_vector_binary_offset_overflow".to_string())?;
                let bytes: [u8; 4] = vector_bytes
                    .get(offset..end)
                    .ok_or_else(|| "memory_index_vector_binary_truncated".to_string())?
                    .try_into()
                    .map_err(|_| "memory_index_vector_binary_truncated".to_string())?;
                let value = f32::from_le_bytes(bytes);
                if !value.is_finite() {
                    return Err("memory_index_vector_binary_non_finite".to_string());
                }
                values.push(value);
                offset = end;
            }
            embeddings.push(MemoryEmbeddingArtifact {
                schema: metadata.schema,
                embedding_id: metadata.embedding_id,
                representation_document_id: metadata.representation_document_id,
                representation_document_digest: metadata.representation_document_digest,
                profile_id: metadata.profile_id,
                profile_digest: metadata.profile_digest,
                dimension: metadata.dimension,
                normalization: metadata.normalization,
                vector_digest: metadata.vector_digest,
                values,
                encoded_at_derivation: metadata.encoded_at_derivation,
            });
        }
        if offset != vector_bytes.len() {
            return Err("memory_index_vector_binary_extra_bytes".to_string());
        }
        let bundle = MemoryIndexBundle {
            schema: self.logical_schema,
            corpus: self.corpus,
            profile: self.profile,
            documents: self.documents,
            lexical: self.lexical,
            vector: MemoryVectorIndex {
                schema: self.vector.schema,
                profile_id: self.vector.profile_id,
                dimension: self.vector.dimension,
                embeddings,
                checksum: self.vector.checksum,
            },
            manifest: self.manifest,
        };
        bundle.validate_loaded()?;
        Ok(bundle)
    }
}

#[derive(Serialize)]
struct StoredVectorIndexRef<'a> {
    schema: &'a str,
    profile_id: &'a str,
    dimension: usize,
    embeddings: Vec<StoredEmbeddingMetadataRef<'a>>,
    checksum: &'a str,
}

#[derive(Serialize)]
struct StoredIndexMetadataRef<'a> {
    storage_schema: &'static str,
    logical_schema: &'a str,
    corpus: &'a MemoryCorpusManifest,
    profile: &'a MemoryRepresentationProfile,
    documents: &'a [MemoryRepresentationDocument],
    lexical: &'a MemoryLexicalIndex,
    vector: StoredVectorIndexRef<'a>,
    manifest: &'a MemoryIndexManifest,
}

impl<'a> StoredIndexMetadataRef<'a> {
    fn from_bundle(bundle: &'a MemoryIndexBundle) -> Self {
        Self {
            storage_schema: DERIVED_MEMORY_PHYSICAL_SCHEMA,
            logical_schema: &bundle.schema,
            corpus: &bundle.corpus,
            profile: &bundle.profile,
            documents: &bundle.documents,
            lexical: &bundle.lexical,
            vector: StoredVectorIndexRef {
                schema: &bundle.vector.schema,
                profile_id: &bundle.vector.profile_id,
                dimension: bundle.vector.dimension,
                embeddings: bundle
                    .vector
                    .embeddings
                    .iter()
                    .map(StoredEmbeddingMetadataRef::from)
                    .collect(),
                checksum: &bundle.vector.checksum,
            },
            manifest: &bundle.manifest,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryIndexStatus {
    pub case_id: String,
    pub profile_id: String,
    pub index_id: String,
    pub corpus_manifest_id: String,
    pub source_generation: u64,
    pub item_count: usize,
    pub dimension: usize,
    pub posture: String,
    pub ann_posture: AnnPosture,
    pub physical_format: String,
    pub storage_bytes: usize,
    pub integrity_posture: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity_error: Option<String>,
}

fn storage_component(value: &str) -> String {
    stable_digest(value)
}

#[cfg(test)]
fn case_directory(root: &Path, tenant_id: &str, case_id: &str) -> PathBuf {
    root.join(DERIVED_MEMORY_STORE_VERSION)
        .join(storage_component(tenant_id))
        .join(storage_component(case_id))
}

#[cfg(test)]
fn profile_directory(root: &Path, tenant_id: &str, case_id: &str, profile_id: &str) -> PathBuf {
    case_directory(root, tenant_id, case_id)
        .join("profiles")
        .join(storage_component(profile_id))
}

#[cfg(test)]
fn bundle_path(profile_directory: &Path, index_id: &str) -> PathBuf {
    profile_directory
        .join("builds")
        .join(storage_component(index_id))
        .join("metadata.json")
}

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[cfg(target_os = "linux")]
#[repr(C)]
struct DerivedOpenHow {
    flags: u64,
    mode: u64,
    resolve: u64,
}

#[cfg(target_os = "linux")]
const DERIVED_RESOLVE_NO_MAGICLINKS: u64 = 0x02;
#[cfg(target_os = "linux")]
const DERIVED_RESOLVE_NO_SYMLINKS: u64 = 0x04;
#[cfg(target_os = "linux")]
const DERIVED_RESOLVE_BENEATH: u64 = 0x08;

#[cfg(target_os = "linux")]
fn derived_cstring(value: &[u8], label: &str) -> Result<CString, String> {
    CString::new(value).map_err(|_| format!("{label}_contains_nul"))
}

#[cfg(target_os = "linux")]
fn openat2_file(
    directory_fd: i32,
    path: &Path,
    flags: i32,
    mode: u32,
    beneath: bool,
) -> std::io::Result<File> {
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::from_raw_os_error(libc::EINVAL))?;
    let mut resolve = DERIVED_RESOLVE_NO_MAGICLINKS | DERIVED_RESOLVE_NO_SYMLINKS;
    if beneath {
        resolve |= DERIVED_RESOLVE_BENEATH;
    }
    let how = DerivedOpenHow {
        flags: flags as u64,
        mode: u64::from(mode),
        resolve,
    };
    let fd = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            directory_fd,
            path.as_ptr(),
            &how as *const DerivedOpenHow,
            std::mem::size_of::<DerivedOpenHow>(),
        ) as i32
    };
    if fd < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}

#[cfg(target_os = "linux")]
fn validate_owned_directory(directory: &File) -> Result<(), String> {
    let metadata = directory
        .metadata()
        .map_err(|error| format!("memory_index_directory_fstat_failed: {error}"))?;
    if !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o022 != 0
    {
        return Err("memory_index_directory_not_secure".to_string());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn validate_owned_regular(file: &File, label: &str) -> Result<u64, String> {
    let metadata = file
        .metadata()
        .map_err(|error| format!("{label}_fstat_failed: {error}"))?;
    if !metadata.is_file()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o022 != 0
        || metadata.nlink() != 1
    {
        return Err(format!("{label}_not_private_regular_file"));
    }
    Ok(metadata.len())
}

#[cfg(target_os = "linux")]
fn open_derived_root(root: &Path, create: bool) -> Result<Option<File>, String> {
    if !root.is_absolute() {
        return Err("memory_index_root_must_be_absolute".to_string());
    }
    let parent = root
        .parent()
        .ok_or_else(|| "memory_index_root_parent_missing".to_string())?;
    let name = root
        .file_name()
        .ok_or_else(|| "memory_index_root_name_missing".to_string())?;
    let parent = match openat2_file(
        libc::AT_FDCWD,
        parent,
        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        0,
        false,
    ) {
        Ok(value) => value,
        Err(error) if error.raw_os_error() == Some(libc::ENOENT) && !create => return Ok(None),
        Err(error) => {
            return Err(format!(
                "memory_index_root_parent_resolution_rejected: {error}"
            ))
        }
    };
    let name_c = derived_cstring(name.as_bytes(), "memory_index_root_name")?;
    if create {
        let result = unsafe { libc::mkdirat(parent.as_raw_fd(), name_c.as_ptr(), 0o700) };
        if result != 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::EEXIST) {
                return Err(format!("memory_index_root_create_failed: {error}"));
            }
        }
    }
    let directory = match openat2_file(
        parent.as_raw_fd(),
        Path::new(name),
        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        0,
        true,
    ) {
        Ok(value) => value,
        Err(error) if error.raw_os_error() == Some(libc::ENOENT) && !create => return Ok(None),
        Err(error) => return Err(format!("memory_index_root_open_failed: {error}")),
    };
    validate_owned_directory(&directory)?;
    Ok(Some(directory))
}

#[cfg(target_os = "linux")]
fn open_child_directory(parent: &File, name: &str, create: bool) -> Result<Option<File>, String> {
    if name.is_empty() || name == "." || name == ".." || name.contains('/') {
        return Err("memory_index_directory_component_invalid".to_string());
    }
    let name_c = derived_cstring(name.as_bytes(), "memory_index_directory_component")?;
    if create {
        let result = unsafe { libc::mkdirat(parent.as_raw_fd(), name_c.as_ptr(), 0o700) };
        if result != 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::EEXIST) {
                return Err(format!("memory_index_directory_create_failed: {error}"));
            }
        }
    }
    let directory = match openat2_file(
        parent.as_raw_fd(),
        Path::new(name),
        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        0,
        true,
    ) {
        Ok(value) => value,
        Err(error) if error.raw_os_error() == Some(libc::ENOENT) && !create => return Ok(None),
        Err(error)
            if matches!(
                error.raw_os_error(),
                Some(libc::ELOOP) | Some(libc::EXDEV) | Some(libc::ENOTDIR)
            ) =>
        {
            return Err("memory_index_directory_not_secure".to_string())
        }
        Err(error) => return Err(format!("memory_index_directory_open_failed: {error}")),
    };
    validate_owned_directory(&directory)?;
    Ok(Some(directory))
}

#[cfg(target_os = "linux")]
fn open_profile_directory(
    root: &Path,
    tenant_id: &str,
    case_id: &str,
    profile_id: &str,
    create: bool,
) -> Result<Option<File>, String> {
    let Some(mut directory) = open_derived_root(root, create)? else {
        return Ok(None);
    };
    for component in [
        DERIVED_MEMORY_STORE_VERSION.to_string(),
        storage_component(tenant_id),
        storage_component(case_id),
        "profiles".to_string(),
        storage_component(profile_id),
    ] {
        let Some(next) = open_child_directory(&directory, &component, create)? else {
            return Ok(None);
        };
        directory = next;
    }
    Ok(Some(directory))
}

#[cfg(target_os = "linux")]
fn open_profiles_directory(
    root: &Path,
    tenant_id: &str,
    case_id: &str,
) -> Result<Option<File>, String> {
    let Some(mut directory) = open_derived_root(root, false)? else {
        return Ok(None);
    };
    for component in [
        DERIVED_MEMORY_STORE_VERSION.to_string(),
        storage_component(tenant_id),
        storage_component(case_id),
        "profiles".to_string(),
    ] {
        let Some(next) = open_child_directory(&directory, &component, false)? else {
            return Ok(None);
        };
        directory = next;
    }
    Ok(Some(directory))
}

#[cfg(target_os = "linux")]
fn open_regular_at(directory: &File, name: &str, flags: i32, mode: u32) -> Result<File, String> {
    let file = openat2_file(
        directory.as_raw_fd(),
        Path::new(name),
        flags | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        mode,
        true,
    )
    .map_err(|error| format!("memory_index_file_open_failed:{name}:{error}"))?;
    validate_owned_regular(&file, "memory_index_file")?;
    Ok(file)
}

#[cfg(target_os = "linux")]
fn exists_at(directory: &File, name: &str) -> Result<bool, String> {
    let name = derived_cstring(name.as_bytes(), "memory_index_entry_name")?;
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    let result = unsafe {
        libc::fstatat(
            directory.as_raw_fd(),
            name.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result == 0 {
        Ok(true)
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            Ok(false)
        } else {
            Err(format!("memory_index_entry_stat_failed: {error}"))
        }
    }
}

#[cfg(target_os = "linux")]
fn entry_mode_at(directory: &File, name: &str) -> Result<Option<libc::mode_t>, String> {
    let name = derived_cstring(name.as_bytes(), "memory_index_entry_name")?;
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    let result = unsafe {
        libc::fstatat(
            directory.as_raw_fd(),
            name.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result == 0 {
        Ok(Some(unsafe { stat.assume_init() }.st_mode))
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            Ok(None)
        } else {
            Err(format!("memory_index_entry_stat_failed: {error}"))
        }
    }
}

#[cfg(target_os = "linux")]
fn list_directory_names(directory: &File) -> Result<Vec<String>, String> {
    let duplicate = unsafe { libc::dup(directory.as_raw_fd()) };
    if duplicate < 0 {
        return Err(format!(
            "memory_index_directory_dup_failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    let stream = unsafe { libc::fdopendir(duplicate) };
    if stream.is_null() {
        unsafe { libc::close(duplicate) };
        return Err(format!(
            "memory_index_directory_stream_failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    let mut names = Vec::new();
    loop {
        let entry = unsafe { libc::readdir(stream) };
        if entry.is_null() {
            break;
        }
        let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }
            .to_str()
            .map_err(|_| "memory_index_directory_entry_non_utf8".to_string())?;
        if name != "." && name != ".." {
            names.push(name.to_string());
        }
    }
    unsafe { libc::closedir(stream) };
    names.sort();
    Ok(names)
}

#[cfg(target_os = "linux")]
fn read_bounded_at(
    directory: &File,
    name: &str,
    maximum: usize,
    label: &str,
) -> Result<Vec<u8>, String> {
    let file = open_regular_at(directory, name, libc::O_RDONLY, 0)
        .map_err(|error| format!("{label}_open_failed:{error}"))?;
    let size = validate_owned_regular(&file, label)?;
    let size = usize::try_from(size).map_err(|_| format!("{label}_size_overflow"))?;
    if size > maximum {
        return Err(format!("{label}_size_bound_exceeded"));
    }
    let mut bytes = Vec::with_capacity(size);
    file.take((maximum as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{label}_read_failed:{error}"))?;
    if bytes.len() != size || bytes.len() > maximum {
        return Err(format!("{label}_size_changed_or_exceeded"));
    }
    Ok(bytes)
}

#[cfg(target_os = "linux")]
fn write_new_at(directory: &File, name: &str, bytes: &[u8], maximum: usize) -> Result<(), String> {
    if bytes.len() > maximum {
        return Err("memory_index_component_size_bound_exceeded".to_string());
    }
    let mut file = open_regular_at(
        directory,
        name,
        libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
        0o600,
    )?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("memory_index_component_write_sync_failed: {error}"))
}

#[cfg(target_os = "linux")]
fn write_new_without_sync_at(
    directory: &File,
    name: &str,
    bytes: &[u8],
    maximum: usize,
) -> Result<(), String> {
    if bytes.len() > maximum {
        return Err("memory_index_component_size_bound_exceeded".to_string());
    }
    let mut file = open_regular_at(
        directory,
        name,
        libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
        0o600,
    )?;
    file.write_all(bytes)
        .map_err(|error| format!("memory_index_component_write_failed: {error}"))
}

#[cfg(target_os = "linux")]
fn sync_directory(directory: &File, label: &str) -> Result<(), String> {
    let result = unsafe { libc::fsync(directory.as_raw_fd()) };
    if result == 0 {
        Ok(())
    } else {
        Err(format!(
            "{label}_sync_failed: {}",
            std::io::Error::last_os_error()
        ))
    }
}

#[cfg(target_os = "linux")]
fn rename_at(directory: &File, source: &str, target: &str) -> Result<(), String> {
    let source = derived_cstring(source.as_bytes(), "memory_index_rename_source")?;
    let target = derived_cstring(target.as_bytes(), "memory_index_rename_target")?;
    let result = unsafe {
        libc::renameat(
            directory.as_raw_fd(),
            source.as_ptr(),
            directory.as_raw_fd(),
            target.as_ptr(),
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(format!(
            "memory_index_rename_failed: {}",
            std::io::Error::last_os_error()
        ))
    }
}

#[cfg(target_os = "linux")]
fn unlink_at(directory: &File, name: &str, flags: i32) -> Result<bool, String> {
    let name = derived_cstring(name.as_bytes(), "memory_index_unlink_name")?;
    let result = unsafe { libc::unlinkat(directory.as_raw_fd(), name.as_ptr(), flags) };
    if result == 0 {
        Ok(true)
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            Ok(false)
        } else {
            Err(format!("memory_index_unlink_failed: {error}"))
        }
    }
}

#[cfg(target_os = "linux")]
fn remove_tree_at(parent: &File, name: &str) -> Result<bool, String> {
    if !exists_at(parent, name)? {
        return Ok(false);
    }
    match open_child_directory(parent, name, false) {
        Ok(Some(directory)) => {
            let identity = directory
                .metadata()
                .map_err(|error| format!("memory_index_remove_fstat_failed:{error}"))?;
            for child in list_directory_names(&directory)? {
                let mode = entry_mode_at(&directory, &child)?
                    .ok_or_else(|| "memory_index_remove_child_missing".to_string())?;
                if mode & libc::S_IFMT == libc::S_IFDIR {
                    remove_tree_at(&directory, &child)?;
                } else {
                    unlink_at(&directory, &child, 0)?;
                }
            }
            let name_c = derived_cstring(name.as_bytes(), "memory_index_remove_name")?;
            let mut current = std::mem::MaybeUninit::<libc::stat>::uninit();
            let result = unsafe {
                libc::fstatat(
                    parent.as_raw_fd(),
                    name_c.as_ptr(),
                    current.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            if result != 0 {
                return Err("memory_index_remove_identity_missing".to_string());
            }
            let current = unsafe { current.assume_init() };
            if current.st_dev != identity.dev() || current.st_ino != identity.ino() {
                return Err("memory_index_remove_identity_changed".to_string());
            }
            unlink_at(parent, name, libc::AT_REMOVEDIR)
        }
        Ok(None) => unlink_at(parent, name, 0),
        Err(error) if error.contains("Not a directory") => unlink_at(parent, name, 0),
        Err(error) => Err(error),
    }
}

#[cfg(target_os = "linux")]
fn atomic_write_at(
    directory: &File,
    name: &str,
    bytes: &[u8],
    maximum: usize,
) -> Result<(), String> {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = format!(".{name}.tmp.{}.{}", std::process::id(), sequence);
    write_new_at(directory, &temporary, bytes, maximum)?;
    if let Err(error) = rename_at(directory, &temporary, name) {
        let _ = unlink_at(directory, &temporary, 0);
        return Err(error);
    }
    sync_directory(directory, "memory_index_atomic_directory")
}

#[cfg(target_os = "linux")]
fn json_bytes<T: Serialize>(value: &T, maximum: usize, label: &str) -> Result<Vec<u8>, String> {
    let bytes =
        serde_json::to_vec(value).map_err(|error| format!("{label}_encode_failed:{error}"))?;
    if bytes.len() > maximum {
        return Err(format!("{label}_size_bound_exceeded"));
    }
    Ok(bytes)
}

#[cfg(target_os = "linux")]
fn parse_json<T: for<'de> Deserialize<'de>>(bytes: &[u8], label: &str) -> Result<T, String> {
    serde_json::from_slice(bytes).map_err(|error| format!("{label}_decode_failed:{error}"))
}

#[cfg(target_os = "linux")]
type PhysicalComponentBytes = (Vec<u8>, Vec<u8>, Vec<u8>);

#[cfg(target_os = "linux")]
fn physical_components(bundle: &MemoryIndexBundle) -> Result<PhysicalComponentBytes, String> {
    bundle.validate_deep()?;
    validate_memory_index_build_budget_documents(&bundle.documents, &bundle.profile)?;
    let metadata = json_bytes(
        &StoredIndexMetadataRef::from_bundle(bundle),
        MAX_DERIVED_METADATA_BYTES,
        "memory_index_metadata",
    )?;
    let vector_elements = bundle
        .vector
        .embeddings
        .len()
        .checked_mul(bundle.vector.dimension)
        .ok_or_else(|| "memory_index_vector_element_budget_overflow".to_string())?;
    if vector_elements > MAX_VECTOR_ELEMENTS {
        return Err("memory_index_vector_element_budget_exceeded".to_string());
    }
    let vector_capacity = vector_elements
        .checked_mul(std::mem::size_of::<f32>())
        .ok_or_else(|| "memory_index_vector_byte_budget_overflow".to_string())?;
    let mut vectors = Vec::with_capacity(vector_capacity);
    for embedding in &bundle.vector.embeddings {
        for value in &embedding.values {
            if !value.is_finite() {
                return Err("memory_index_vector_binary_non_finite".to_string());
            }
            vectors.extend_from_slice(&value.to_le_bytes());
        }
    }
    if vectors.len() != vector_capacity || vectors.len() > MAX_VECTOR_BYTES {
        return Err("memory_index_vector_binary_length_mismatch".to_string());
    }
    let total = metadata
        .len()
        .checked_add(vectors.len())
        .and_then(|value| value.checked_add(MAX_PHYSICAL_MANIFEST_BYTES))
        .ok_or_else(|| "memory_index_total_budget_overflow".to_string())?;
    if total > MAX_DERIVED_TOTAL_BYTES {
        return Err("memory_index_serialized_budget_exceeded".to_string());
    }
    let seal = PhysicalIndexSeal {
        schema: DERIVED_MEMORY_PHYSICAL_SCHEMA.to_string(),
        index_id: bundle.manifest.index_id.clone(),
        content_checksum: bundle.manifest.content_checksum.clone(),
        item_count: bundle.documents.len(),
        dimension: bundle.vector.dimension,
        vector_elements,
        components: vec![
            PhysicalComponentSeal {
                name: "metadata.json".to_string(),
                size_bytes: metadata.len(),
                checksum: digest_bytes(&metadata),
            },
            PhysicalComponentSeal {
                name: "vectors.f32le".to_string(),
                size_bytes: vectors.len(),
                checksum: digest_bytes(&vectors),
            },
        ],
    };
    let seal = json_bytes(
        &seal,
        MAX_PHYSICAL_MANIFEST_BYTES,
        "memory_index_physical_manifest",
    )?;
    Ok((metadata, vectors, seal))
}

#[cfg(target_os = "linux")]
fn read_bundle_from_build(build: &File) -> Result<(MemoryIndexBundle, usize), String> {
    let seal_bytes = read_bounded_at(
        build,
        "seal.json",
        MAX_PHYSICAL_MANIFEST_BYTES,
        "memory_index_physical_manifest",
    )?;
    let seal: PhysicalIndexSeal = parse_json(&seal_bytes, "memory_index_physical_manifest")?;
    if seal.schema != DERIVED_MEMORY_PHYSICAL_SCHEMA
        || seal.components.len() != 2
        || seal.components[0].name != "metadata.json"
        || seal.components[1].name != "vectors.f32le"
        || seal.item_count > MAX_CORPUS_DOCUMENTS
        || seal.dimension == 0
        || seal.dimension > MAX_VECTOR_DIMENSION
        || seal.vector_elements > MAX_VECTOR_ELEMENTS
    {
        return Err("memory_index_physical_manifest_invalid".to_string());
    }
    let expected_elements = seal
        .item_count
        .checked_mul(seal.dimension)
        .ok_or_else(|| "memory_index_vector_element_budget_overflow".to_string())?;
    let expected_vector_bytes = expected_elements
        .checked_mul(std::mem::size_of::<f32>())
        .ok_or_else(|| "memory_index_vector_byte_budget_overflow".to_string())?;
    if seal.vector_elements != expected_elements
        || seal.components[0].size_bytes > MAX_DERIVED_METADATA_BYTES
        || seal.components[1].size_bytes != expected_vector_bytes
        || expected_vector_bytes > MAX_VECTOR_BYTES
    {
        return Err("memory_index_physical_size_contract_mismatch".to_string());
    }
    let total = seal_bytes
        .len()
        .checked_add(seal.components[0].size_bytes)
        .and_then(|value| value.checked_add(seal.components[1].size_bytes))
        .ok_or_else(|| "memory_index_total_budget_overflow".to_string())?;
    if total > MAX_DERIVED_TOTAL_BYTES {
        return Err("memory_index_physical_total_size_exceeded".to_string());
    }
    let metadata = read_bounded_at(
        build,
        "metadata.json",
        seal.components[0].size_bytes,
        "memory_index_metadata",
    )?;
    let vectors = read_bounded_at(
        build,
        "vectors.f32le",
        seal.components[1].size_bytes,
        "memory_index_vectors",
    )?;
    if metadata.len() != seal.components[0].size_bytes
        || vectors.len() != seal.components[1].size_bytes
        || digest_bytes(&metadata) != seal.components[0].checksum
        || digest_bytes(&vectors) != seal.components[1].checksum
    {
        return Err("memory_index_physical_component_integrity_mismatch".to_string());
    }
    let stored: StoredIndexMetadata = parse_json(&metadata, "memory_index_metadata")?;
    let bundle = stored.into_bundle(&vectors)?;
    if bundle.manifest.index_id != seal.index_id
        || bundle.manifest.content_checksum != seal.content_checksum
        || bundle.documents.len() != seal.item_count
        || bundle.profile.vector_dimension != seal.dimension
    {
        return Err("memory_index_physical_logical_identity_mismatch".to_string());
    }
    Ok((bundle, total))
}

pub struct MemoryIndexBuildLock {
    #[cfg(target_os = "linux")]
    file: File,
    #[cfg(target_os = "linux")]
    profile: File,
    tenant_id: String,
    case_id: String,
    profile_id: String,
}

#[cfg(target_os = "linux")]
fn lock_file(file: &File, mode: i32) -> Result<(), String> {
    let result = unsafe { libc::flock(file.as_raw_fd(), mode) };
    if result == 0 {
        Ok(())
    } else {
        Err(format!(
            "memory_index_lock_failed: {}",
            std::io::Error::last_os_error()
        ))
    }
}

#[cfg(target_os = "linux")]
fn unlock_file(file: &File) {
    unsafe {
        libc::flock(file.as_raw_fd(), libc::LOCK_UN);
    }
}

impl Drop for MemoryIndexBuildLock {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        unlock_file(&self.file);
    }
}

#[cfg(target_os = "linux")]
struct MemoryIndexReadLock {
    file: File,
    profile: File,
}

#[cfg(target_os = "linux")]
impl Drop for MemoryIndexReadLock {
    fn drop(&mut self) {
        unlock_file(&self.file);
    }
}

#[cfg(target_os = "linux")]
fn acquire_read_lock(profile: File) -> Result<MemoryIndexReadLock, String> {
    let file = open_regular_at(&profile, "build.lock", libc::O_RDONLY, 0)
        .map_err(|_| "memory_index_lock_missing_or_invalid".to_string())?;
    lock_file(&file, libc::LOCK_SH)?;
    Ok(MemoryIndexReadLock { file, profile })
}

#[cfg(target_os = "linux")]
fn cleanup_abandoned_builds(profile: &File) -> Result<(), String> {
    let Some(builds) = open_child_directory(profile, "builds", false)? else {
        return Ok(());
    };
    for name in list_directory_names(&builds)? {
        if name.starts_with(".build.tmp.") {
            remove_tree_at(&builds, &name)?;
        }
    }
    sync_directory(&builds, "memory_index_abandoned_build_cleanup")?;
    for name in list_directory_names(profile)? {
        if name.starts_with(".current.json.tmp.") || name.starts_with(".last-retrieval.json.tmp.") {
            unlink_at(profile, &name, 0)?;
        }
    }
    sync_directory(profile, "memory_index_abandoned_pointer_cleanup")
}

pub fn acquire_memory_index_build_lock(
    root: &Path,
    tenant_id: &str,
    case_id: &str,
    profile_id: &str,
) -> Result<MemoryIndexBuildLock, String> {
    for (label, value) in [
        ("memory_index_tenant", tenant_id),
        ("memory_index_case", case_id),
        ("memory_index_profile", profile_id),
    ] {
        require_nonempty(label, value)?;
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (root, tenant_id, case_id, profile_id);
        return Err("memory_index_mutation_platform_unsupported".to_string());
    }
    #[cfg(target_os = "linux")]
    let profile = open_profile_directory(root, tenant_id, case_id, profile_id, true)?
        .ok_or_else(|| "memory_index_profile_create_failed".to_string())?;
    #[cfg(target_os = "linux")]
    let file = open_regular_at(&profile, "build.lock", libc::O_RDWR | libc::O_CREAT, 0o600)?;
    #[cfg(target_os = "linux")]
    lock_file(&file, libc::LOCK_EX)?;
    #[cfg(target_os = "linux")]
    cleanup_abandoned_builds(&profile)?;
    Ok(MemoryIndexBuildLock {
        #[cfg(target_os = "linux")]
        file,
        #[cfg(target_os = "linux")]
        profile,
        tenant_id: tenant_id.to_string(),
        case_id: case_id.to_string(),
        profile_id: profile_id.to_string(),
    })
}

pub fn publish_memory_index_locked(
    lock: &MemoryIndexBuildLock,
    bundle: &MemoryIndexBundle,
    fail_after_temp: bool,
) -> Result<(), String> {
    publish_memory_index_locked_with_failpoint(
        lock,
        bundle,
        fail_after_temp.then_some(MemoryIndexPublicationFailpoint::AfterCompleteTempBeforePublish),
    )
}

pub fn publish_memory_index_locked_with_failpoint(
    lock: &MemoryIndexBuildLock,
    bundle: &MemoryIndexBundle,
    failpoint: Option<MemoryIndexPublicationFailpoint>,
) -> Result<(), String> {
    bundle.validate_deep()?;
    if bundle.profile.tenant_id != lock.tenant_id
        || bundle.corpus.case_id != lock.case_id
        || bundle.profile.profile_id != lock.profile_id
    {
        return Err("memory_index_publication_scope_mismatch".to_string());
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (lock, bundle, failpoint);
        return Err("memory_index_mutation_platform_unsupported".to_string());
    }
    #[cfg(target_os = "linux")]
    let builds = open_child_directory(&lock.profile, "builds", true)?
        .ok_or_else(|| "memory_index_builds_create_failed".to_string())?;
    #[cfg(target_os = "linux")]
    let target_name = storage_component(&bundle.manifest.index_id);
    #[cfg(target_os = "linux")]
    if let Some(target) = open_child_directory(&builds, &target_name, false)? {
        let (existing, _) = read_bundle_from_build(&target)?;
        existing.validate_loaded()?;
        if existing != *bundle {
            return Err("memory_index_content_identity_collision".to_string());
        }
    } else {
        #[cfg(target_os = "linux")]
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        #[cfg(target_os = "linux")]
        let temporary_name = format!(".build.tmp.{}.{}", std::process::id(), sequence);
        #[cfg(target_os = "linux")]
        let temporary = open_child_directory(&builds, &temporary_name, true)?
            .ok_or_else(|| "memory_index_temp_build_create_failed".to_string())?;
        #[cfg(target_os = "linux")]
        let (metadata, vectors, seal) = physical_components(bundle)?;
        #[cfg(target_os = "linux")]
        if failpoint == Some(MemoryIndexPublicationFailpoint::AfterComponentWriteBeforeFileSync) {
            write_new_without_sync_at(
                &temporary,
                "metadata.json",
                &metadata,
                MAX_DERIVED_METADATA_BYTES,
            )?;
            return Err(
                "memory_index_failpoint_after_component_write_before_file_sync".to_string(),
            );
        }
        if let Err(error) = (|| {
            write_new_at(
                &temporary,
                "metadata.json",
                &metadata,
                MAX_DERIVED_METADATA_BYTES,
            )?;
            write_new_at(&temporary, "vectors.f32le", &vectors, MAX_VECTOR_BYTES)?;
            write_new_at(&temporary, "seal.json", &seal, MAX_PHYSICAL_MANIFEST_BYTES)?;
            if failpoint
                == Some(MemoryIndexPublicationFailpoint::AfterFileSyncBeforeBuildDirectorySync)
            {
                return Err(
                    "memory_index_failpoint_after_file_sync_before_build_directory_sync"
                        .to_string(),
                );
            }
            sync_directory(&temporary, "memory_index_temp_build_directory")
        })() {
            if failpoint
                != Some(MemoryIndexPublicationFailpoint::AfterFileSyncBeforeBuildDirectorySync)
            {
                let _ = remove_tree_at(&builds, &temporary_name);
            }
            return Err(error);
        }
        if failpoint == Some(MemoryIndexPublicationFailpoint::AfterCompleteTempBeforePublish) {
            #[cfg(target_os = "linux")]
            let _ = remove_tree_at(&builds, &temporary_name);
            return Err("memory_index_failpoint_after_complete_temp".to_string());
        }
        #[cfg(target_os = "linux")]
        if let Err(error) = rename_at(&builds, &temporary_name, &target_name) {
            let _ = remove_tree_at(&builds, &temporary_name);
            return Err(format!("memory_index_build_publish_failed:{error}"));
        }
        if failpoint
            == Some(MemoryIndexPublicationFailpoint::AfterBuildRenameBeforeBuildsParentSync)
        {
            return Err(
                "memory_index_failpoint_after_build_rename_before_builds_parent_sync".to_string(),
            );
        }
        #[cfg(target_os = "linux")]
        sync_directory(&builds, "memory_index_builds_parent")?;
    }
    if failpoint == Some(MemoryIndexPublicationFailpoint::AfterBuildsSyncBeforePointerWrite) {
        return Err("memory_index_failpoint_after_builds_sync_before_pointer_write".to_string());
    }
    let pointer = CurrentIndexPointer {
        schema: "yai.memory_index_current.v2".to_string(),
        storage_format: DERIVED_MEMORY_PHYSICAL_SCHEMA.to_string(),
        case_id: bundle.corpus.case_id.clone(),
        profile_id: bundle.profile.profile_id.clone(),
        index_id: bundle.manifest.index_id.clone(),
        content_checksum: bundle.manifest.content_checksum.clone(),
    };
    #[cfg(target_os = "linux")]
    {
        let bytes = json_bytes(
            &pointer,
            MAX_CURRENT_POINTER_BYTES,
            "memory_index_current_pointer",
        )?;
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = format!(".current.json.tmp.{}.{}", std::process::id(), sequence);
        write_new_at(&lock.profile, &temporary, &bytes, MAX_CURRENT_POINTER_BYTES)?;
        if failpoint == Some(MemoryIndexPublicationFailpoint::DuringPointerTempWrite) {
            return Err("memory_index_failpoint_during_pointer_temp_write".to_string());
        }
        rename_at(&lock.profile, &temporary, "current.json")?;
        if failpoint
            == Some(MemoryIndexPublicationFailpoint::AfterPointerRenameBeforeProfileDirectorySync)
        {
            return Err(
                "memory_index_failpoint_after_pointer_rename_before_profile_directory_sync"
                    .to_string(),
            );
        }
        sync_directory(&lock.profile, "memory_index_pointer_profile_directory")?;
        gc_builds_locked(lock, &target_name)?;
        sync_directory(&lock.profile, "memory_index_profile_directory")?;
        if failpoint == Some(MemoryIndexPublicationFailpoint::AfterFinalSyncBeforeAcknowledgement) {
            return Err(
                "memory_index_failpoint_after_final_sync_before_acknowledgement".to_string(),
            );
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn gc_builds_locked(lock: &MemoryIndexBuildLock, current: &str) -> Result<(), String> {
    let Some(builds) = open_child_directory(&lock.profile, "builds", false)? else {
        return Ok(());
    };
    let mut completed = list_directory_names(&builds)?
        .into_iter()
        .filter(|name| !name.starts_with('.'))
        .collect::<Vec<_>>();
    completed.sort();
    let mut retained = BTreeSet::from([current.to_string()]);
    for name in completed
        .iter()
        .rev()
        .filter(|name| name.as_str() != current)
        .take(MAX_RETAINED_BUILDS.saturating_sub(1))
    {
        retained.insert(name.clone());
    }
    for name in completed {
        if !retained.contains(&name) {
            remove_tree_at(&builds, &name)?;
        }
    }
    sync_directory(&builds, "memory_index_gc_builds")
}

#[cfg(target_os = "linux")]
fn load_from_locked_profile(
    profile: &File,
    tenant_id: &str,
    case_id: &str,
    profile_id: &str,
) -> Result<Option<(MemoryIndexBundle, usize)>, String> {
    if !exists_at(profile, "current.json")? {
        return Ok(None);
    }
    let pointer_bytes = read_bounded_at(
        profile,
        "current.json",
        MAX_CURRENT_POINTER_BYTES,
        "memory_index_current_pointer",
    )?;
    let pointer: CurrentIndexPointer = parse_json(&pointer_bytes, "memory_index_current_pointer")?;
    if pointer.schema != "yai.memory_index_current.v2"
        || pointer.storage_format != DERIVED_MEMORY_PHYSICAL_SCHEMA
        || pointer.case_id != case_id
        || pointer.profile_id != profile_id
    {
        return Err("memory_index_current_pointer_integrity_mismatch".to_string());
    }
    let builds = open_child_directory(profile, "builds", false)?
        .ok_or_else(|| "memory_index_builds_missing".to_string())?;
    let build = open_child_directory(&builds, &storage_component(&pointer.index_id), false)?
        .ok_or_else(|| "memory_index_current_build_missing".to_string())?;
    let (bundle, bytes) = read_bundle_from_build(&build)?;
    validate_pointer_bundle_scope(&pointer, &bundle, tenant_id, case_id)?;
    Ok(Some((bundle, bytes.saturating_add(pointer_bytes.len()))))
}

pub fn load_current_memory_index_locked(
    lock: &MemoryIndexBuildLock,
) -> Result<Option<MemoryIndexBundle>, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = lock;
        return Err("memory_index_read_platform_unsupported".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        load_from_locked_profile(
            &lock.profile,
            &lock.tenant_id,
            &lock.case_id,
            &lock.profile_id,
        )
        .map(|value| value.map(|(bundle, _)| bundle))
    }
}

pub fn load_current_memory_index(
    root: &Path,
    tenant_id: &str,
    case_id: &str,
    profile_id: &str,
) -> Result<Option<MemoryIndexBundle>, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (root, tenant_id, case_id, profile_id);
        return Err("memory_index_read_platform_unsupported".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        let Some(profile) = open_profile_directory(root, tenant_id, case_id, profile_id, false)?
        else {
            return Ok(None);
        };
        if !exists_at(&profile, "current.json")? {
            return Ok(None);
        }
        let read = acquire_read_lock(profile)?;
        load_from_locked_profile(&read.profile, tenant_id, case_id, profile_id)
            .map(|value| value.map(|(bundle, _)| bundle))
    }
}

fn validate_pointer_bundle_scope(
    pointer: &CurrentIndexPointer,
    bundle: &MemoryIndexBundle,
    tenant_id: &str,
    case_id: &str,
) -> Result<(), String> {
    if pointer.schema != "yai.memory_index_current.v2"
        || pointer.storage_format != DERIVED_MEMORY_PHYSICAL_SCHEMA
        || pointer.case_id != case_id
        || pointer.profile_id != bundle.profile.profile_id
        || pointer.index_id != bundle.manifest.index_id
        || pointer.content_checksum != bundle.manifest.content_checksum
        || bundle.corpus.case_id != case_id
        || bundle.profile.tenant_id != tenant_id
    {
        return Err("memory_index_pointer_bundle_mismatch".to_string());
    }
    Ok(())
}

pub fn list_memory_index_statuses(
    root: &Path,
    tenant_id: &str,
    case_id: &str,
    current_generation: u64,
) -> Result<Vec<MemoryIndexStatus>, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (root, tenant_id, case_id, current_generation);
        return Err("memory_index_read_platform_unsupported".to_string());
    }
    #[cfg(target_os = "linux")]
    let Some(profiles) = open_profiles_directory(root, tenant_id, case_id)?
    else {
        return Ok(Vec::new());
    };
    let mut statuses = Vec::new();
    #[cfg(target_os = "linux")]
    for profile_name in list_directory_names(&profiles)? {
        let profile = open_child_directory(&profiles, &profile_name, false)?
            .ok_or_else(|| "memory_index_profile_directory_not_secure".to_string())?;
        if !exists_at(&profile, "current.json")? {
            continue;
        }
        let read = match acquire_read_lock(profile) {
            Ok(read) => read,
            Err(error) => {
                statuses.push(corrupt_status(case_id, &profile_name, error));
                continue;
            }
        };
        let pointer = match read_bounded_at(
            &read.profile,
            "current.json",
            MAX_CURRENT_POINTER_BYTES,
            "memory_index_current_pointer",
        )
        .and_then(|bytes| parse_json::<CurrentIndexPointer>(&bytes, "memory_index_current_pointer"))
        {
            Ok(pointer) => pointer,
            Err(error) => {
                statuses.push(corrupt_status(case_id, &profile_name, error));
                continue;
            }
        };
        let loaded =
            load_from_locked_profile(&read.profile, tenant_id, case_id, &pointer.profile_id)
                .and_then(|value| value.ok_or_else(|| "memory_index_current_missing".to_string()))
                .and_then(|(bundle, bytes)| {
                    if profile_name != storage_component(&bundle.profile.profile_id) {
                        return Err("memory_index_profile_directory_mismatch".to_string());
                    }
                    Ok((bundle, bytes))
                });
        match loaded {
            Ok((bundle, bytes)) => statuses.push(MemoryIndexStatus {
                case_id: case_id.to_string(),
                profile_id: bundle.profile.profile_id.clone(),
                index_id: bundle.manifest.index_id.clone(),
                corpus_manifest_id: bundle.corpus.manifest_id.clone(),
                source_generation: bundle.corpus.source_generation,
                item_count: bundle.documents.len(),
                dimension: bundle.profile.vector_dimension,
                posture: if bundle.is_current(case_id, current_generation) {
                    "current".to_string()
                } else {
                    "stale".to_string()
                },
                ann_posture: bundle.manifest.ann_posture.clone(),
                physical_format: DERIVED_MEMORY_PHYSICAL_SCHEMA.to_string(),
                storage_bytes: bytes,
                integrity_posture: "sealed_load_valid".to_string(),
                integrity_error: None,
            }),
            Err(error) => statuses.push(corrupt_status(case_id, &pointer.profile_id, error)),
        }
    }
    statuses.sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
    Ok(statuses)
}

fn corrupt_status(case_id: &str, profile_id: &str, error: String) -> MemoryIndexStatus {
    MemoryIndexStatus {
        case_id: case_id.to_string(),
        profile_id: profile_id.to_string(),
        index_id: "unavailable".to_string(),
        corpus_manifest_id: "unavailable".to_string(),
        source_generation: 0,
        item_count: 0,
        dimension: 0,
        posture: "corrupt".to_string(),
        ann_posture: AnnPosture::DeferredExactScanWithinBound,
        physical_format: DERIVED_MEMORY_PHYSICAL_SCHEMA.to_string(),
        storage_bytes: 0,
        integrity_posture: "corrupt".to_string(),
        integrity_error: Some(error),
    }
}

pub fn find_current_memory_index(
    root: &Path,
    tenant_id: &str,
    case_id: &str,
    profile_id: Option<&str>,
) -> Result<Option<MemoryIndexBundle>, String> {
    if let Some(profile_id) = profile_id {
        return load_current_memory_index(root, tenant_id, case_id, profile_id);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (root, tenant_id, case_id);
        return Err("memory_index_read_platform_unsupported".to_string());
    }
    #[cfg(target_os = "linux")]
    let Some(profiles) = open_profiles_directory(root, tenant_id, case_id)?
    else {
        return Ok(None);
    };
    let mut found = Vec::new();
    #[cfg(target_os = "linux")]
    for profile_name in list_directory_names(&profiles)? {
        let profile = open_child_directory(&profiles, &profile_name, false)?
            .ok_or_else(|| "memory_index_profile_directory_not_secure".to_string())?;
        if !exists_at(&profile, "current.json")? {
            continue;
        }
        let read = acquire_read_lock(profile)?;
        let pointer_bytes = read_bounded_at(
            &read.profile,
            "current.json",
            MAX_CURRENT_POINTER_BYTES,
            "memory_index_current_pointer",
        )?;
        let pointer: CurrentIndexPointer =
            parse_json(&pointer_bytes, "memory_index_current_pointer")?;
        let (bundle, _) =
            load_from_locked_profile(&read.profile, tenant_id, case_id, &pointer.profile_id)?
                .ok_or_else(|| "memory_index_current_missing".to_string())?;
        if profile_name != storage_component(&bundle.profile.profile_id) {
            return Err("memory_index_profile_directory_mismatch".to_string());
        }
        found.push(bundle);
    }
    found.sort_by(|left, right| left.profile.profile_id.cmp(&right.profile.profile_id));
    match found.len() {
        0 => Ok(None),
        1 => Ok(found.pop()),
        _ => Err("memory_index_profile_required_multiple_current_profiles".to_string()),
    }
}

pub fn drop_memory_index_locked(lock: &MemoryIndexBuildLock) -> Result<bool, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = lock;
        return Err("memory_index_mutation_platform_unsupported".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        let existed = exists_at(&lock.profile, "current.json")?
            || exists_at(&lock.profile, "builds")?
            || exists_at(&lock.profile, "last-retrieval.json")?;
        unlink_at(&lock.profile, "current.json", 0)?;
        unlink_at(&lock.profile, "last-retrieval.json", 0)?;
        remove_tree_at(&lock.profile, "builds")?;
        sync_directory(&lock.profile, "memory_index_drop_profile")?;
        Ok(existed)
    }
}

pub fn store_last_hybrid_retrieval(
    root: &Path,
    tenant_id: &str,
    retrieval: &HybridRetrievalSet,
) -> Result<(), String> {
    let profile_id = retrieval
        .representation_profile_id
        .as_deref()
        .ok_or_else(|| "memory_retrieval_profile_missing".to_string())?;
    let lock = acquire_memory_index_build_lock(root, tenant_id, &retrieval.case_id, profile_id)?;
    #[cfg(not(target_os = "linux"))]
    {
        let _ = lock;
        return Err("memory_index_mutation_platform_unsupported".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        let bytes = json_bytes(retrieval, MAX_LAST_RETRIEVAL_BYTES, "memory_last_retrieval")?;
        atomic_write_at(
            &lock.profile,
            "last-retrieval.json",
            &bytes,
            MAX_LAST_RETRIEVAL_BYTES,
        )
    }
}

pub fn load_last_hybrid_retrieval(
    root: &Path,
    tenant_id: &str,
    case_id: &str,
    profile_id: &str,
) -> Result<Option<HybridRetrievalSet>, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (root, tenant_id, case_id, profile_id);
        return Err("memory_index_read_platform_unsupported".to_string());
    }
    #[cfg(target_os = "linux")]
    let Some(profile) = open_profile_directory(root, tenant_id, case_id, profile_id, false)?
    else {
        return Ok(None);
    };
    #[cfg(target_os = "linux")]
    if !exists_at(&profile, "last-retrieval.json")? {
        return Ok(None);
    }
    #[cfg(target_os = "linux")]
    let read = acquire_read_lock(profile)?;
    #[cfg(target_os = "linux")]
    let bytes = read_bounded_at(
        &read.profile,
        "last-retrieval.json",
        MAX_LAST_RETRIEVAL_BYTES,
        "memory_last_retrieval",
    )?;
    #[cfg(target_os = "linux")]
    {
        let retrieval: HybridRetrievalSet = parse_json(&bytes, "memory_last_retrieval")?;
        if retrieval.schema != HYBRID_RETRIEVAL_SET_SCHEMA
            || retrieval.case_id != case_id
            || retrieval.representation_profile_id.as_deref() != Some(profile_id)
        {
            return Err("memory_last_retrieval_integrity_mismatch".to_string());
        }
        Ok(Some(retrieval))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ProjectionPurpose;
    use crate::effect::{DecisionOutcome, EffectOutcome};
    use crate::memory::{
        OperationalMemoryKind, OperationalMemoryLifecycle, OperationalMemoryManifest,
        OperationalMemoryPosture, OperationalMemoryProvenance, OperationalMemoryVisibility,
        OPERATIONAL_MEMORY_DERIVATION, OPERATIONAL_MEMORY_MANIFEST_SCHEMA,
        OPERATIONAL_MEMORY_SCHEMA,
    };
    use crate::transition::{AdmittedView, CaseLifecycle, ParticipantState};
    use std::process::Command;
    use std::time::Instant;

    static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn test_directory(label: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "yai-memory-index-{label}-{}-{}-{nonce}",
            std::process::id(),
            TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn entry(
        case_id: &str,
        memory_id: &str,
        participant: &str,
        text: &str,
        posture: OperationalMemoryPosture,
        generation: u64,
    ) -> OperationalMemoryEntry {
        OperationalMemoryEntry {
            schema: OPERATIONAL_MEMORY_SCHEMA.to_string(),
            memory_id: memory_id.to_string(),
            case_id: case_id.to_string(),
            derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
            semantic_kind: OperationalMemoryKind::ProviderClaim,
            posture,
            value: OperationalMemoryValue::ProviderClaim {
                result_id: format!("result:{memory_id}"),
                invocation_id: format!("invocation:{memory_id}"),
                provider_id: "provider:test".to_string(),
                model_id: "model:test".to_string(),
                preview: text.to_string(),
            },
            description: text.to_string(),
            provenance: OperationalMemoryProvenance {
                transition_ids: vec![format!("transition:{memory_id}")],
                observation_ids: Vec::new(),
                effect_receipt_ids: Vec::new(),
                causal_refs: Vec::new(),
                generation_start: generation,
                generation_end: generation,
            },
            visibility: OperationalMemoryVisibility {
                participant_ids: vec![participant.to_string()],
                consumer: "model".to_string(),
                view_kind: "model_context".to_string(),
            },
            derived_at_generation: 3,
            lifecycle: OperationalMemoryLifecycle::Active,
            superseded_by: None,
        }
    }

    fn fixture() -> (
        CaseState,
        Vec<OperationalMemoryEntry>,
        MemoryRepresentationCorpus,
        MemoryRepresentationProfile,
        BTreeMap<String, Vec<f32>>,
    ) {
        let case_id = "case:memory-index";
        let entries = vec![
            entry(
                case_id,
                "memory:a",
                "participant:a",
                "amber filesystem project codename",
                OperationalMemoryPosture::ProviderOriginatedClaim,
                1,
            ),
            entry(
                case_id,
                "memory:b",
                "participant:a",
                "violet deployment result",
                OperationalMemoryPosture::ProviderOriginatedClaim,
                2,
            ),
            entry(
                case_id,
                "memory:hidden",
                "participant:b",
                "amber hidden participant secret",
                OperationalMemoryPosture::ProviderOriginatedClaim,
                3,
            ),
        ];
        let build = OperationalMemoryBuild {
            manifest: OperationalMemoryManifest {
                schema: OPERATIONAL_MEMORY_MANIFEST_SCHEMA.to_string(),
                case_id: case_id.to_string(),
                derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
                source_generation: 3,
                memory_ids: entries
                    .iter()
                    .map(|entry| entry.memory_id.clone())
                    .collect(),
            },
            entries: entries.clone(),
        };
        let corpus = derive_representation_corpus(&build).unwrap();
        let profile = MemoryRepresentationProfile::new(
            "tenant:test",
            "test-only:fixture-encoder",
            "fixture:model",
            "fixture-revision-1",
            3,
        )
        .unwrap();
        let vectors = corpus
            .documents
            .iter()
            .map(|document| {
                let value = if document.memory_id == "memory:a" {
                    vec![1.0, 0.0, 0.0]
                } else if document.memory_id == "memory:b" {
                    vec![0.0, 1.0, 0.0]
                } else {
                    vec![0.9, 0.1, 0.0]
                };
                (document.document_id.clone(), value)
            })
            .collect();
        let mut state = CaseState::new(case_id, CaseLifecycle::Open);
        state.generation = 3;
        state.tenant_id = Some("tenant:test".to_string());
        state.participants = vec![ParticipantState {
            participant_id: "participant:a".to_string(),
            roles: vec!["model_provider".to_string()],
            admitted_views: vec![AdmittedView {
                consumer: "model".to_string(),
                view_kind: "model_context".to_string(),
            }],
        }];
        (state, entries, corpus, profile, vectors)
    }

    fn qualification(state: &CaseState) -> RetrievalQualification {
        RetrievalQualification {
            case_id: state.case_id.clone(),
            participant_id: "participant:a".to_string(),
            consumer: "model".to_string(),
            view_kind: "model_context".to_string(),
            purpose: ProjectionPurpose::Conversation,
            case_generation: state.generation,
            resource_refs: Vec::new(),
            semantic_kinds: Vec::new(),
            causal_refs: Vec::new(),
            max_results: 2,
            include_superseded: false,
        }
    }

    #[test]
    fn representation_and_profile_identity_are_deterministic_and_content_addressed() {
        let (_, entries, corpus, profile, _) = fixture();
        let again = derive_representation_corpus(&OperationalMemoryBuild {
            manifest: OperationalMemoryManifest {
                schema: OPERATIONAL_MEMORY_MANIFEST_SCHEMA.to_string(),
                case_id: "case:memory-index".to_string(),
                derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
                source_generation: 3,
                memory_ids: entries
                    .iter()
                    .map(|entry| entry.memory_id.clone())
                    .collect(),
            },
            entries,
        })
        .unwrap();
        assert_eq!(corpus, again);
        profile.validate().unwrap();
        assert!(profile.profile_id.starts_with("memory-profile:"));
        assert!(corpus
            .documents
            .iter()
            .all(|document| !document.canonical_text.contains("description=")));
    }

    #[test]
    fn exact_vector_scan_rejects_invalid_vectors_and_has_stable_ties() {
        let (_, _, corpus, profile, mut vectors) = fixture();
        let index = MemoryVectorIndex::build(&corpus.documents, &profile, &vectors).unwrap();
        assert_eq!(index.exact_search(&[1.0, 0.0, 0.0], 3).unwrap().len(), 3);
        assert_eq!(
            index.exact_search(&[1.0, 0.0], 3).unwrap_err(),
            "memory_vector_dimension_invalid"
        );
        assert_eq!(
            index.exact_search(&[f32::NAN, 0.0, 0.0], 3).unwrap_err(),
            "memory_vector_non_finite"
        );
        assert_eq!(
            index
                .exact_search(&[f32::INFINITY, 0.0, 0.0], 3)
                .unwrap_err(),
            "memory_vector_non_finite"
        );
        let first_two = corpus
            .documents
            .iter()
            .take(2)
            .map(|document| document.document_id.clone())
            .collect::<Vec<_>>();
        for document_id in &first_two {
            vectors.insert(document_id.clone(), vec![1.0, 0.0, 0.0]);
        }
        let tied = MemoryVectorIndex::build(&corpus.documents, &profile, &vectors)
            .unwrap()
            .exact_search(&[1.0, 0.0, 0.0], 3)
            .unwrap();
        let mut expected = first_two;
        expected.sort();
        assert_eq!(
            tied.iter()
                .take(2)
                .map(|hit| hit.document_id.clone())
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn bm25_and_hybrid_fusion_are_query_sensitive_and_visibility_safe() {
        let (state, entries, corpus, profile, vectors) = fixture();
        let bundle = MemoryIndexBundle::build(corpus, profile, &vectors).unwrap();
        let lexical = bundle.lexical.search("amber filesystem", 8).unwrap();
        assert!(!lexical.is_empty());
        let result = hybrid_retrieve(
            &state,
            &entries,
            qualification(&state),
            RetrievalQueryDocument::new("amber filesystem").unwrap(),
            Some(&bundle),
            Ok(Some(vec![1.0, 0.0, 0.0])),
        )
        .unwrap();
        assert_eq!(result.schema, HYBRID_RETRIEVAL_SET_SCHEMA);
        assert!(result
            .selected
            .iter()
            .any(|item| item.memory.memory_id == "memory:a"));
        assert!(!result
            .selected
            .iter()
            .any(|item| item.memory.memory_id == "memory:hidden"));
        assert!(!serde_json::to_string(&result)
            .unwrap()
            .contains("memory:hidden"));
        assert!(!serde_json::to_string(&result)
            .unwrap()
            .contains("hidden participant secret"));
        assert!(
            result
                .qualification_rejections
                .restricted_rejection_counts_redacted
        );
        assert!(result.selected.iter().all(|item| {
            item.ranking_reasons
                .iter()
                .any(|reason| reason.starts_with("rrf_k_60"))
        }));
        let alternate = hybrid_retrieve(
            &state,
            &entries,
            qualification(&state),
            RetrievalQueryDocument::new("amber filesystem").unwrap(),
            Some(&bundle),
            Ok(Some(vec![0.0, 1.0, 0.0])),
        )
        .unwrap();
        assert_ne!(
            result.retrieval_id, alternate.retrieval_id,
            "retrieval identity binds per-plane ranks and scores"
        );
    }

    #[test]
    fn hybrid_fixture_exposes_distinct_lexical_vector_and_exact_causal_ranks() {
        let (state, _, _, profile, _) = fixture();
        let mut lexical_a = entry(
            &state.case_id,
            "memory:lexical-a",
            "participant:a",
            "quartz lexical needle needle",
            OperationalMemoryPosture::ProviderOriginatedClaim,
            1,
        );
        let mut vector_b = entry(
            &state.case_id,
            "memory:vector-b",
            "participant:a",
            "unrelated vector candidate",
            OperationalMemoryPosture::ProviderOriginatedClaim,
            2,
        );
        let mut causal_c = entry(
            &state.case_id,
            "memory:causal-c",
            "participant:a",
            "exact governed decision",
            OperationalMemoryPosture::DecisionControlHistory,
            3,
        );
        causal_c.semantic_kind = OperationalMemoryKind::Decision;
        causal_c.value = OperationalMemoryValue::Decision {
            operation_id: "operation:causal".to_string(),
            decision_id: "decision:causal".to_string(),
            outcome: DecisionOutcome::Allow,
            resource_attachment_id: "resource:workspace".to_string(),
            relative_path: "allowed/quartz.txt".to_string(),
            reason: "historical allow is context only".to_string(),
        };
        for memory in [&mut lexical_a, &mut vector_b, &mut causal_c] {
            memory.provenance.causal_refs = vec!["causal:required".to_string()];
        }
        let entries = vec![lexical_a, vector_b, causal_c];
        let corpus = derive_representation_corpus(&OperationalMemoryBuild {
            manifest: OperationalMemoryManifest {
                schema: OPERATIONAL_MEMORY_MANIFEST_SCHEMA.to_string(),
                case_id: state.case_id.clone(),
                derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
                source_generation: state.generation,
                memory_ids: entries
                    .iter()
                    .map(|entry| entry.memory_id.clone())
                    .collect(),
            },
            entries: entries.clone(),
        })
        .unwrap();
        let vectors = corpus
            .documents
            .iter()
            .map(|document| {
                let vector = match document.memory_id.as_str() {
                    "memory:lexical-a" => vec![0.0, 1.0, 0.0],
                    "memory:vector-b" => vec![1.0, 0.0, 0.0],
                    _ => vec![0.0, 0.0, 1.0],
                };
                (document.document_id.clone(), vector)
            })
            .collect();
        let bundle = MemoryIndexBundle::build(corpus, profile, &vectors).unwrap();
        let mut request = qualification(&state);
        request.max_results = 3;
        request.causal_refs = vec!["causal:required".to_string()];
        let result = hybrid_retrieve(
            &state,
            &entries,
            request,
            RetrievalQueryDocument::new("quartz lexical needle").unwrap(),
            Some(&bundle),
            Ok(Some(vec![1.0, 0.0, 0.0])),
        )
        .unwrap();
        let rank = |memory_id: &str, plane: &str| {
            result
                .selected
                .iter()
                .find(|item| item.memory.memory_id == memory_id)
                .unwrap()
                .plane_ranks
                .iter()
                .find(|rank| rank.plane == plane)
                .map(|rank| rank.rank)
        };
        assert_eq!(rank("memory:lexical-a", "lexical_bm25"), Some(1));
        assert_eq!(rank("memory:vector-b", "vector_exact_cosine"), Some(1));
        assert_eq!(rank("memory:causal-c", "exact_operational"), Some(1));
        assert_eq!(result.selected[0].memory.memory_id, "memory:causal-c");
        assert_eq!(
            result
                .selected
                .iter()
                .find(|item| item.memory.memory_id == "memory:causal-c")
                .unwrap()
                .memory
                .posture,
            OperationalMemoryPosture::DecisionControlHistory
        );
        assert!(state.last_operation.is_none());
        assert!(state.last_decision.is_none());
        assert!(state.grants.is_empty());
        assert!(state.effects.is_empty());
    }

    #[test]
    fn representation_scrubs_sensitive_tokens_and_bounds_canonical_input() {
        let (_, mut entries, _, _, _) = fixture();
        entries[0].value = OperationalMemoryValue::ProviderClaim {
            result_id: "result:sensitive".to_string(),
            invocation_id: "invocation:sensitive".to_string(),
            provider_id: "provider:test".to_string(),
            model_id: "model:test".to_string(),
            preview: format!(
                "Authorization: Bearer sk-not-a-real-secret {}",
                "bounded ".repeat(1000)
            ),
        };
        let document = MemoryRepresentationDocument::from_memory(&entries[0]).unwrap();
        assert!(!document.canonical_text.contains("sk-not-a-real-secret"));
        assert!(!document.canonical_text.contains("Authorization:"));
        assert!(!document.canonical_text.contains("Bearer"));
        assert!(document
            .canonical_text
            .contains("[redacted-sensitive-content]"));
        assert!(document.canonical_text.chars().count() <= MAX_REPRESENTATION_CHARS);
    }

    #[cfg(unix)]
    #[test]
    fn h19_s01_derived_parent_symlink_is_rejected_without_external_write() {
        use std::os::unix::fs::symlink;

        let base = test_directory("symlink-parent");
        let root = base.join("derived-memory");
        let outside = base.join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        let marker = outside.join("marker");
        fs::write(&marker, b"untouched").unwrap();
        symlink(&outside, root.join(DERIVED_MEMORY_STORE_VERSION)).unwrap();
        assert_eq!(
            list_memory_index_statuses(&root, "tenant:test", "case:memory-index", 3).unwrap_err(),
            "memory_index_directory_not_secure"
        );
        let error = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            "memory-profile:test",
        )
        .err()
        .unwrap();
        assert_eq!(error, "memory_index_directory_not_secure");
        assert_eq!(fs::read(&marker).unwrap(), b"untouched");
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 1);
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn provider_claim_similarity_never_changes_authority_posture() {
        let (state, mut entries, _, profile, _) = fixture();
        entries[0].value = OperationalMemoryValue::ProviderClaim {
            result_id: "result:claim".to_string(),
            invocation_id: "invocation:claim".to_string(),
            provider_id: "provider:test".to_string(),
            model_id: "model:test".to_string(),
            preview: "file amber.txt was written successfully".to_string(),
        };
        entries[0].description = "file amber.txt was written successfully".to_string();
        entries[1].semantic_kind = OperationalMemoryKind::ResourceEffect;
        entries[1].posture = OperationalMemoryPosture::FinalizedObservedConsequence;
        entries[1].value = OperationalMemoryValue::ResourceEffect {
            operation_id: "operation:observed".to_string(),
            effect_id: "effect:observed".to_string(),
            resource_attachment_id: "resource:workspace".to_string(),
            relative_path: "allowed/amber.txt".to_string(),
            outcome: EffectOutcome::Applied,
            content_digest: Some("sha256:observed".to_string()),
            receipt_id: "receipt:observed".to_string(),
        };
        entries[1].description = "file amber.txt was written successfully".to_string();
        entries[1].provenance.effect_receipt_ids = vec!["receipt:observed".to_string()];
        let build = OperationalMemoryBuild {
            manifest: OperationalMemoryManifest {
                schema: OPERATIONAL_MEMORY_MANIFEST_SCHEMA.to_string(),
                case_id: state.case_id.clone(),
                derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
                source_generation: 3,
                memory_ids: entries
                    .iter()
                    .map(|entry| entry.memory_id.clone())
                    .collect(),
            },
            entries: entries.clone(),
        };
        let corpus = derive_representation_corpus(&build).unwrap();
        let vectors = corpus
            .documents
            .iter()
            .map(|document| (document.document_id.clone(), vec![1.0, 0.0, 0.0]))
            .collect();
        let bundle = MemoryIndexBundle::build(corpus, profile, &vectors).unwrap();
        let result = hybrid_retrieve(
            &state,
            &entries,
            qualification(&state),
            RetrievalQueryDocument::new("amber file successful").unwrap(),
            Some(&bundle),
            Ok(Some(vec![1.0, 0.0, 0.0])),
        )
        .unwrap();
        assert_eq!(
            result
                .selected
                .iter()
                .find(|item| item.memory.memory_id == "memory:a")
                .unwrap()
                .memory
                .posture,
            OperationalMemoryPosture::ProviderOriginatedClaim
        );
    }

    #[test]
    fn profile_replacement_never_compares_incompatible_vectors() {
        let (_, _, corpus, profile_a, vectors_a) = fixture();
        let profile_b = MemoryRepresentationProfile::new(
            "tenant:test",
            "test-only:fixture-encoder",
            "fixture:model:b",
            "fixture-revision-2",
            2,
        )
        .unwrap();
        let vectors_b = corpus
            .documents
            .iter()
            .map(|document| (document.document_id.clone(), vec![0.0, 1.0]))
            .collect();
        let index_a = MemoryIndexBundle::build(corpus.clone(), profile_a, &vectors_a).unwrap();
        let index_b = MemoryIndexBundle::build(corpus, profile_b, &vectors_b).unwrap();
        assert_ne!(index_a.profile.profile_id, index_b.profile.profile_id);
        assert_eq!(
            index_a.vector.exact_search(&[1.0, 0.0], 1).unwrap_err(),
            "memory_vector_dimension_invalid"
        );
        assert_eq!(
            index_b.vector.exact_search(&[0.0, 1.0], 1).unwrap().len(),
            1
        );
    }

    #[test]
    fn stale_and_cross_case_candidates_fail_closed() {
        let (mut state, mut entries, corpus, profile, vectors) = fixture();
        let mut foreign = entries[0].clone();
        foreign.case_id = "case:foreign".to_string();
        foreign.memory_id = "memory:foreign".to_string();
        entries.push(foreign);
        let bundle = MemoryIndexBundle::build(corpus, profile, &vectors).unwrap();
        let result = hybrid_retrieve(
            &state,
            &entries,
            qualification(&state),
            RetrievalQueryDocument::new("amber").unwrap(),
            Some(&bundle),
            Ok(Some(vec![1.0, 0.0, 0.0])),
        )
        .unwrap();
        assert!(!result
            .selected
            .iter()
            .any(|item| item.memory.case_id == "case:foreign"));
        assert!(
            result
                .qualification_rejections
                .restricted_rejection_counts_redacted
        );

        state.generation += 1;
        let mut stale_qualification = qualification(&state);
        stale_qualification.case_generation = state.generation;
        assert_eq!(
            hybrid_retrieve(
                &state,
                &entries,
                stale_qualification,
                RetrievalQueryDocument::new("amber").unwrap(),
                Some(&bundle),
                Ok(Some(vec![1.0, 0.0, 0.0])),
            )
            .unwrap_err(),
            "memory_index_stale_for_case_generation"
        );
    }

    #[test]
    fn build_crash_matrix_never_publishes_partial_index() {
        let (_, _, corpus, profile, vectors) = fixture();
        let old = MemoryIndexBundle::build(corpus.clone(), profile.clone(), &vectors).unwrap();
        let root = test_directory("crash");
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &old, false).unwrap();
        drop(lock);

        for failpoint in [
            MemoryIndexBuildFailpoint::AfterCorpusManifest,
            MemoryIndexBuildFailpoint::DuringLexicalBuild,
            MemoryIndexBuildFailpoint::DuringEmbeddingGeneration,
            MemoryIndexBuildFailpoint::DuringVectorSerialization,
        ] {
            assert!(MemoryIndexBundle::build_with_failpoint(
                corpus.clone(),
                profile.clone(),
                &vectors,
                Some(failpoint),
            )
            .is_err());
            let current = load_current_memory_index(
                &root,
                "tenant:test",
                "case:memory-index",
                &profile.profile_id,
            )
            .unwrap()
            .unwrap();
            assert_eq!(current.manifest.index_id, old.manifest.index_id);
        }

        let changed_vectors = corpus
            .documents
            .iter()
            .map(|document| (document.document_id.clone(), vec![0.0, 0.0, 1.0]))
            .collect::<BTreeMap<_, _>>();
        let changed = MemoryIndexBundle::build(corpus, profile.clone(), &changed_vectors).unwrap();
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        assert_eq!(
            publish_memory_index_locked(&lock, &changed, true).unwrap_err(),
            "memory_index_failpoint_after_complete_temp"
        );
        drop(lock);
        assert_eq!(
            load_current_memory_index(
                &root,
                "tenant:test",
                "case:memory-index",
                &profile.profile_id,
            )
            .unwrap()
            .unwrap()
            .manifest
            .index_id,
            old.manifest.index_id
        );

        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &changed, false).unwrap();
        drop(lock);
        assert_eq!(
            load_current_memory_index(
                &root,
                "tenant:test",
                "case:memory-index",
                &profile.profile_id,
            )
            .unwrap()
            .unwrap()
            .manifest
            .index_id,
            changed.manifest.index_id,
            "lost acknowledgement after publication cannot hide the sealed current index"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn publication_and_automatic_profile_selection_enforce_tenant_case_scope() {
        let (_, _, corpus, profile, vectors) = fixture();
        let bundle = MemoryIndexBundle::build(corpus, profile.clone(), &vectors).unwrap();
        let root = test_directory("scope-binding");
        let foreign_lock = acquire_memory_index_build_lock(
            &root,
            "tenant:foreign",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        assert_eq!(
            publish_memory_index_locked(&foreign_lock, &bundle, false).unwrap_err(),
            "memory_index_publication_scope_mismatch"
        );
        drop(foreign_lock);

        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &bundle, false).unwrap();
        drop(lock);
        let directory = profile_directory(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        );
        let pointer_path = directory.join("current.json");
        let mut pointer: CurrentIndexPointer =
            serde_json::from_slice(&fs::read(&pointer_path).unwrap()).unwrap();
        pointer.case_id = "case:foreign".to_string();
        fs::write(&pointer_path, serde_json::to_vec(&pointer).unwrap()).unwrap();
        assert_eq!(
            find_current_memory_index(&root, "tenant:test", "case:memory-index", None).unwrap_err(),
            "memory_index_current_pointer_integrity_mismatch"
        );
        let statuses =
            list_memory_index_statuses(&root, "tenant:test", "case:memory-index", 3).unwrap();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].posture, "corrupt");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corruption_is_detected_and_drop_preserves_source_material() {
        let (_, entries, corpus, profile, vectors) = fixture();
        let bundle = MemoryIndexBundle::build(corpus, profile.clone(), &vectors).unwrap();
        let root = test_directory("corruption");
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &bundle, false).unwrap();
        drop(lock);
        let directory = profile_directory(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        );
        let path = bundle_path(&directory, &bundle.manifest.index_id);
        let mut bytes = fs::read(&path).unwrap();
        let corrupt_at = bytes.len() / 2;
        bytes[corrupt_at] ^= 0x01;
        fs::write(&path, bytes).unwrap();
        assert!(load_current_memory_index(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .is_err());
        let statuses =
            list_memory_index_statuses(&root, "tenant:test", "case:memory-index", 3).unwrap();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].posture, "corrupt");
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        assert!(drop_memory_index_locked(&lock).unwrap());
        drop(lock);
        assert_eq!(
            entries.len(),
            3,
            "source OperationalMemory input is untouched"
        );
        assert!(load_current_memory_index(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap()
        .is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore]
    fn concurrent_rebuild_child() {
        let Some(root) = std::env::var_os("YAI_MEMORY_INDEX_CHILD_ROOT") else {
            return;
        };
        let (_, _, corpus, profile, vectors) = fixture();
        let bundle = MemoryIndexBundle::build(corpus, profile.clone(), &vectors).unwrap();
        let lock = acquire_memory_index_build_lock(
            Path::new(&root),
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &bundle, false).unwrap();
    }

    #[test]
    fn h19_s09_32_process_concurrent_rebuilds_publish_one_equivalent_manifest() {
        let root = test_directory("concurrent");
        let executable = std::env::current_exe().unwrap();
        let mut children = Vec::new();
        for _ in 0..32 {
            children.push(
                Command::new(&executable)
                    .args([
                        "--ignored",
                        "--exact",
                        "memory_index::tests::concurrent_rebuild_child",
                    ])
                    .env("YAI_MEMORY_INDEX_CHILD_ROOT", &root)
                    .spawn()
                    .unwrap(),
            );
        }
        for mut child in children {
            assert!(child.wait().unwrap().success());
        }
        let (_, _, _, profile, _) = fixture();
        let current = load_current_memory_index(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap()
        .unwrap();
        current.validate().unwrap();
        let build_directories = fs::read_dir(
            profile_directory(
                &root,
                "tenant:test",
                "case:memory-index",
                &profile.profile_id,
            )
            .join("builds"),
        )
        .unwrap()
        .count();
        assert_eq!(build_directories, 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn profile_namespaces_are_independent_and_deletable() {
        let (_, _, corpus, profile_a, vectors_a) = fixture();
        let profile_b = MemoryRepresentationProfile::new(
            "tenant:test",
            "test-only:fixture-encoder",
            "fixture:model:b",
            "fixture-revision-2",
            2,
        )
        .unwrap();
        let vectors_b = corpus
            .documents
            .iter()
            .map(|document| (document.document_id.clone(), vec![1.0, 1.0]))
            .collect::<BTreeMap<_, _>>();
        let a = MemoryIndexBundle::build(corpus.clone(), profile_a.clone(), &vectors_a).unwrap();
        let b = MemoryIndexBundle::build(corpus, profile_b.clone(), &vectors_b).unwrap();
        let root = test_directory("profiles");
        for bundle in [&a, &b] {
            let lock = acquire_memory_index_build_lock(
                &root,
                "tenant:test",
                "case:memory-index",
                &bundle.profile.profile_id,
            )
            .unwrap();
            publish_memory_index_locked(&lock, bundle, false).unwrap();
        }
        assert_eq!(a.vector.exact_search(&[1.0, 0.0, 0.0], 1).unwrap().len(), 1);
        assert_eq!(b.vector.exact_search(&[1.0, 1.0], 1).unwrap().len(), 1);
        let b_before = load_current_memory_index(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile_b.profile_id,
        )
        .unwrap()
        .unwrap()
        .manifest
        .index_id;
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile_a.profile_id,
        )
        .unwrap();
        drop_memory_index_locked(&lock).unwrap();
        drop(lock);
        assert!(load_current_memory_index(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile_a.profile_id,
        )
        .unwrap()
        .is_none());
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile_a.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &a, false).unwrap();
        drop(lock);
        assert!(load_current_memory_index(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile_a.profile_id,
        )
        .unwrap()
        .is_some());
        assert!(load_current_memory_index(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile_b.profile_id,
        )
        .unwrap()
        .is_some());
        assert_eq!(
            load_current_memory_index(
                &root,
                "tenant:test",
                "case:memory-index",
                &profile_b.profile_id,
            )
            .unwrap()
            .unwrap()
            .manifest
            .index_id,
            b_before
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn h19_s02_lock_symlink_substitution_fails_closed() {
        use std::os::unix::fs::symlink;

        let root = test_directory("h19-lock-symlink");
        let (_, _, _, profile, _) = fixture();
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        drop(lock);
        let directory = profile_directory(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        );
        let outside = root.parent().unwrap().join(format!(
            "yai-memory-index-outside-lock-{}",
            TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&outside, b"outside-lock").unwrap();
        fs::remove_file(directory.join("build.lock")).unwrap();
        symlink(&outside, directory.join("build.lock")).unwrap();
        assert!(acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .is_err());
        assert_eq!(fs::read(&outside).unwrap(), b"outside-lock");
        fs::remove_dir_all(root).unwrap();
        fs::remove_file(outside).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn h19_s01_descriptor_anchored_profile_swap_cannot_redirect_publication() {
        use std::os::unix::fs::symlink;

        let (_, _, corpus, profile, vectors) = fixture();
        let bundle = MemoryIndexBundle::build(corpus, profile.clone(), &vectors).unwrap();
        let root = test_directory("h19-profile-swap");
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        let profile_path = profile_directory(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        );
        let anchored_path = profile_path.with_extension("anchored");
        let outside = root.join("outside");
        fs::create_dir(&outside).unwrap();
        let marker = outside.join("marker");
        fs::write(&marker, b"untouched").unwrap();
        fs::rename(&profile_path, &anchored_path).unwrap();
        symlink(&outside, &profile_path).unwrap();
        publish_memory_index_locked(&lock, &bundle, false).unwrap();
        assert_eq!(fs::read(&marker).unwrap(), b"untouched");
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 1);
        assert!(anchored_path.join("current.json").is_file());
        drop(lock);
        assert!(load_current_memory_index(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id
        )
        .is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h19_s03_oversized_pointer_is_refused_before_json_decode() {
        let (_, _, corpus, profile, vectors) = fixture();
        let bundle = MemoryIndexBundle::build(corpus, profile.clone(), &vectors).unwrap();
        let root = test_directory("h19-oversized-pointer");
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &bundle, false).unwrap();
        drop(lock);
        let pointer = profile_directory(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .join("current.json");
        fs::write(&pointer, vec![b'x'; MAX_CURRENT_POINTER_BYTES + 1]).unwrap();
        let error = load_current_memory_index(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap_err();
        assert!(error.contains("size_bound_exceeded"));
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &bundle, false).unwrap();
        drop(lock);
        let metadata = bundle_path(
            &profile_directory(
                &root,
                "tenant:test",
                "case:memory-index",
                &profile.profile_id,
            ),
            &bundle.manifest.index_id,
        );
        let file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(metadata)
            .unwrap();
        file.set_len((MAX_DERIVED_METADATA_BYTES as u64) + 1)
            .unwrap();
        let error = load_current_memory_index(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap_err();
        assert!(error.contains("size_bound_exceeded"));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn h19_hardlinked_pointer_and_insecure_root_are_refused() {
        use std::os::unix::fs::PermissionsExt;

        let (_, _, corpus, profile, vectors) = fixture();
        let bundle = MemoryIndexBundle::build(corpus, profile.clone(), &vectors).unwrap();
        let root = test_directory("h19-hardlink");
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &bundle, false).unwrap();
        drop(lock);

        let profile_path = profile_directory(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        );
        fs::hard_link(
            profile_path.join("current.json"),
            root.join("pointer-hardlink"),
        )
        .unwrap();
        assert!(load_current_memory_index(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap_err()
        .contains("not_private_regular_file"));
        fs::remove_dir_all(&root).unwrap();

        let insecure = test_directory("h19-insecure-root");
        fs::create_dir_all(&insecure).unwrap();
        fs::set_permissions(&insecure, fs::Permissions::from_mode(0o777)).unwrap();
        assert_eq!(
            acquire_memory_index_build_lock(
                &insecure,
                "tenant:test",
                "case:memory-index",
                &profile.profile_id,
            )
            .err()
            .unwrap(),
            "memory_index_directory_not_secure"
        );
        fs::set_permissions(&insecure, fs::Permissions::from_mode(0o700)).unwrap();
        fs::remove_dir_all(insecure).unwrap();
    }

    #[test]
    fn h19_s04_pointer_rollback_is_loaded_but_reported_stale() {
        let (state, entries, _, profile, _) = fixture();
        let build_at = |generation| {
            let mut generation_entries = entries.clone();
            for entry in &mut generation_entries {
                entry.derived_at_generation = generation;
                entry.provenance.generation_start = generation;
                entry.provenance.generation_end = generation;
            }
            let corpus = derive_representation_corpus(&OperationalMemoryBuild {
                manifest: OperationalMemoryManifest {
                    schema: OPERATIONAL_MEMORY_MANIFEST_SCHEMA.to_string(),
                    case_id: state.case_id.clone(),
                    derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
                    source_generation: generation,
                    memory_ids: generation_entries
                        .iter()
                        .map(|entry| entry.memory_id.clone())
                        .collect(),
                },
                entries: generation_entries,
            })
            .unwrap();
            let vectors = corpus
                .documents
                .iter()
                .map(|document| (document.document_id.clone(), vec![1.0, 0.0, 0.0]))
                .collect();
            MemoryIndexBundle::build(corpus, profile.clone(), &vectors).unwrap()
        };
        let old = build_at(2);
        let current = build_at(3);
        let root = test_directory("h19-pointer-rollback");
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &old, false).unwrap();
        drop(lock);
        let pointer_path = profile_directory(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .join("current.json");
        let old_pointer = fs::read(&pointer_path).unwrap();
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &current, false).unwrap();
        drop(lock);
        fs::write(&pointer_path, old_pointer).unwrap();
        let statuses =
            list_memory_index_statuses(&root, "tenant:test", "case:memory-index", 3).unwrap();
        assert_eq!(statuses[0].posture, "stale");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h19_s05_self_consistent_source_divergent_document_is_rejected() {
        let (state, entries, _, profile, _) = fixture();
        let mut forged_entries = entries.clone();
        forged_entries[0].value = OperationalMemoryValue::ProviderClaim {
            result_id: "result:forged".to_string(),
            invocation_id: "invocation:forged".to_string(),
            provider_id: "provider:test".to_string(),
            model_id: "model:test".to_string(),
            preview: "forged representation payload".to_string(),
        };
        forged_entries[0].description = "forged representation payload".to_string();
        let forged_corpus = derive_representation_corpus(&OperationalMemoryBuild {
            manifest: OperationalMemoryManifest {
                schema: OPERATIONAL_MEMORY_MANIFEST_SCHEMA.to_string(),
                case_id: state.case_id.clone(),
                derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
                source_generation: state.generation,
                memory_ids: forged_entries
                    .iter()
                    .map(|entry| entry.memory_id.clone())
                    .collect(),
            },
            entries: forged_entries,
        })
        .unwrap();
        let vectors = forged_corpus
            .documents
            .iter()
            .map(|document| (document.document_id.clone(), vec![1.0, 0.0, 0.0]))
            .collect();
        let forged = MemoryIndexBundle::build(forged_corpus, profile, &vectors).unwrap();
        assert_eq!(
            hybrid_retrieve(
                &state,
                &entries,
                qualification(&state),
                RetrievalQueryDocument::new("forged").unwrap(),
                Some(&forged),
                Ok(Some(vec![1.0, 0.0, 0.0])),
            )
            .unwrap_err(),
            "memory_index_source_divergent"
        );
    }

    #[test]
    fn h19_s06_selected_payload_is_canonical_operational_memory() {
        let (state, entries, corpus, profile, vectors) = fixture();
        let bundle = MemoryIndexBundle::build(corpus, profile, &vectors).unwrap();
        let result = hybrid_retrieve(
            &state,
            &entries,
            qualification(&state),
            RetrievalQueryDocument::new("amber filesystem").unwrap(),
            Some(&bundle),
            Ok(Some(vec![1.0, 0.0, 0.0])),
        )
        .unwrap();
        let selected = result
            .selected
            .iter()
            .find(|item| item.memory.memory_id == "memory:a")
            .unwrap();
        assert_eq!(selected.memory, entries[0]);
        assert_ne!(
            selected.memory.description, bundle.documents[0].canonical_text,
            "representation text is never the projected payload"
        );
    }

    #[test]
    fn h19_self_consistent_vector_steering_can_change_rank_but_not_content() {
        let (state, entries, corpus, profile, _) = fixture();
        let vectors = corpus
            .documents
            .iter()
            .map(|document| {
                let vector = if document.memory_id == "memory:b" {
                    vec![1.0, 0.0, 0.0]
                } else {
                    vec![0.0, 1.0, 0.0]
                };
                (document.document_id.clone(), vector)
            })
            .collect();
        let steered = MemoryIndexBundle::build(corpus, profile, &vectors).unwrap();
        let result = hybrid_retrieve(
            &state,
            &entries,
            qualification(&state),
            RetrievalQueryDocument::new("unmatched-vector-only-query").unwrap(),
            Some(&steered),
            Ok(Some(vec![1.0, 0.0, 0.0])),
        )
        .unwrap();
        let selected = result
            .selected
            .iter()
            .find(|item| item.memory.memory_id == "memory:b")
            .unwrap();
        assert_eq!(selected.memory, entries[1]);
        assert_eq!(
            selected.memory.posture,
            OperationalMemoryPosture::ProviderOriginatedClaim
        );
    }

    #[test]
    fn h19_rrf_and_numeric_edges_are_checked_and_stably_tied() {
        let mut candidates = BTreeMap::new();
        add_rank(
            &mut candidates,
            "memory:a",
            CandidatePlaneRank {
                plane: "lexical_bm25".to_string(),
                rank: MAX_CANDIDATES_PER_PLANE,
                plane_score_micros: i64::MAX,
                evidence: Vec::new(),
            },
            false,
        )
        .unwrap();
        assert_eq!(
            add_rank(
                &mut candidates,
                "memory:a",
                CandidatePlaneRank {
                    plane: "lexical_bm25".to_string(),
                    rank: 1,
                    plane_score_micros: 0,
                    evidence: Vec::new(),
                },
                false,
            )
            .unwrap_err(),
            "memory_hybrid_duplicate_plane_candidate"
        );
        assert_eq!(
            add_rank(
                &mut candidates,
                "memory:b",
                CandidatePlaneRank {
                    plane: "vector_exact_cosine".to_string(),
                    rank: 0,
                    plane_score_micros: 0,
                    evidence: Vec::new(),
                },
                false,
            )
            .unwrap_err(),
            "memory_hybrid_plane_rank_invalid"
        );
        let (_, _, corpus, profile, mut vectors) = fixture();
        for vector in vectors.values_mut() {
            *vector = vec![f32::MIN_POSITIVE, 1.0, -f32::MIN_POSITIVE];
        }
        let index = MemoryVectorIndex::build(&corpus.documents, &profile, &vectors).unwrap();
        let hits = index
            .exact_search(&[f32::MIN_POSITIVE, 1.0, -f32::MIN_POSITIVE], 3)
            .unwrap();
        assert!(hits.iter().all(|hit| hit.similarity_micros == 1_000_000));
        assert!(hits
            .windows(2)
            .all(|pair| pair[0].document_id < pair[1].document_id));
    }

    #[test]
    fn h19_query_tokenization_is_bounded_and_control_safe() {
        let query = RetrievalQueryDocument::new(&format!(
            "  A\0e\u{301}\n{} tail",
            "x".repeat(MAX_QUERY_CHARS * 4)
        ))
        .unwrap();
        assert!(query.canonical_text.chars().count() <= MAX_QUERY_CHARS);
        assert!(!query.canonical_text.contains('\0'));
        assert!(!query.canonical_text.contains('\n'));
        assert_eq!(
            tokenize(&query.canonical_text)
                .into_iter()
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
        assert!(RetrievalQueryDocument::new("\0\n\t").is_err());
    }

    #[test]
    fn h19_s07_drop_query_race_returns_snapshot_or_unavailable() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        let (_, _, corpus, profile, vectors) = fixture();
        let bundle = MemoryIndexBundle::build(corpus, profile.clone(), &vectors).unwrap();
        let root = test_directory("h19-drop-query");
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &bundle, false).unwrap();
        drop(lock);
        let barrier = Arc::new(Barrier::new(17));
        let mut readers = Vec::new();
        for _ in 0..16 {
            let barrier = barrier.clone();
            let root = root.clone();
            let profile_id = profile.profile_id.clone();
            readers.push(thread::spawn(move || {
                barrier.wait();
                load_current_memory_index(&root, "tenant:test", "case:memory-index", &profile_id)
            }));
        }
        barrier.wait();
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        drop_memory_index_locked(&lock).unwrap();
        drop(lock);
        for reader in readers {
            match reader.join().unwrap() {
                Ok(Some(snapshot)) => {
                    assert_eq!(snapshot.manifest.index_id, bundle.manifest.index_id)
                }
                Ok(None) => {}
                Err(error) => panic!("partial reader state: {error}"),
            }
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h19_s08_rebuild_query_race_never_mixes_components() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        let (_, _, corpus, profile, vectors) = fixture();
        let old = MemoryIndexBundle::build(corpus.clone(), profile.clone(), &vectors).unwrap();
        let changed_vectors = corpus
            .documents
            .iter()
            .map(|document| (document.document_id.clone(), vec![0.0, 0.0, 1.0]))
            .collect();
        let new = MemoryIndexBundle::build(corpus, profile.clone(), &changed_vectors).unwrap();
        let root = test_directory("h19-rebuild-query");
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &old, false).unwrap();
        drop(lock);
        let barrier = Arc::new(Barrier::new(65));
        let mut readers = Vec::new();
        for _ in 0..64 {
            let barrier = barrier.clone();
            let root = root.clone();
            let profile_id = profile.profile_id.clone();
            readers.push(thread::spawn(move || {
                barrier.wait();
                load_current_memory_index(&root, "tenant:test", "case:memory-index", &profile_id)
                    .unwrap()
                    .unwrap()
                    .manifest
                    .index_id
            }));
        }
        barrier.wait();
        let lock = acquire_memory_index_build_lock(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .unwrap();
        publish_memory_index_locked(&lock, &new, false).unwrap();
        drop(lock);
        for reader in readers {
            let id = reader.join().unwrap();
            assert!(id == old.manifest.index_id || id == new.manifest.index_id);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h19_s15_hidden_documents_do_not_change_visible_bm25_or_vector_rank() {
        let (state, mut entries, _, profile, _) = fixture();
        entries.truncate(1);
        for index in 0..10_000 {
            entries.push(entry(
                &state.case_id,
                &format!("memory:hidden-{index:05}"),
                "participant:b",
                "quasar rare hidden term",
                OperationalMemoryPosture::ProviderOriginatedClaim,
                3,
            ));
        }
        let build = |values: Vec<OperationalMemoryEntry>| {
            let corpus = derive_representation_corpus(&OperationalMemoryBuild {
                manifest: OperationalMemoryManifest {
                    schema: OPERATIONAL_MEMORY_MANIFEST_SCHEMA.to_string(),
                    case_id: state.case_id.clone(),
                    derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
                    source_generation: state.generation,
                    memory_ids: values.iter().map(|entry| entry.memory_id.clone()).collect(),
                },
                entries: values,
            })
            .unwrap();
            let vectors = corpus
                .documents
                .iter()
                .map(|document| (document.document_id.clone(), vec![1.0, 0.0, 0.0]))
                .collect();
            MemoryIndexBundle::build(corpus, profile.clone(), &vectors).unwrap()
        };
        let full = build(entries.clone());
        let visible_entries = vec![entries[0].clone()];
        let visible = build(visible_entries.clone());
        let run = |source: &[OperationalMemoryEntry], index: &MemoryIndexBundle| {
            hybrid_retrieve(
                &state,
                source,
                qualification(&state),
                RetrievalQueryDocument::new("amber quasar").unwrap(),
                Some(index),
                Ok(Some(vec![1.0, 0.0, 0.0])),
            )
            .unwrap()
        };
        let full_result = run(&entries, &full);
        let visible_result = run(&visible_entries, &visible);
        let rank = |result: &HybridRetrievalSet, plane: &str| {
            result.selected[0]
                .plane_ranks
                .iter()
                .find(|rank| rank.plane == plane)
                .map(|rank| (rank.rank, rank.plane_score_micros))
        };
        assert_eq!(
            rank(&full_result, "lexical_bm25"),
            rank(&visible_result, "lexical_bm25")
        );
        assert_eq!(
            rank(&full_result, "vector_exact_cosine"),
            rank(&visible_result, "vector_exact_cosine")
        );
        assert_eq!(full_result.qualified_count, visible_result.qualified_count);
    }

    #[test]
    fn h19_s16_superseded_documents_do_not_change_active_ranking() {
        let (state, mut entries, _, profile, _) = fixture();
        entries.truncate(1);
        for index in 0..128 {
            let mut superseded = entry(
                &state.case_id,
                &format!("memory:superseded-{index:03}"),
                "participant:a",
                "amber filesystem",
                OperationalMemoryPosture::ProviderOriginatedClaim,
                3,
            );
            superseded.lifecycle = OperationalMemoryLifecycle::Superseded;
            superseded.superseded_by = Some("memory:a".to_string());
            entries.push(superseded);
        }
        let corpus = derive_representation_corpus(&OperationalMemoryBuild {
            manifest: OperationalMemoryManifest {
                schema: OPERATIONAL_MEMORY_MANIFEST_SCHEMA.to_string(),
                case_id: state.case_id.clone(),
                derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
                source_generation: state.generation,
                memory_ids: entries
                    .iter()
                    .map(|entry| entry.memory_id.clone())
                    .collect(),
            },
            entries: entries.clone(),
        })
        .unwrap();
        let vectors = corpus
            .documents
            .iter()
            .map(|document| (document.document_id.clone(), vec![1.0, 0.0, 0.0]))
            .collect();
        let bundle = MemoryIndexBundle::build(corpus, profile, &vectors).unwrap();
        let result = hybrid_retrieve(
            &state,
            &entries,
            qualification(&state),
            RetrievalQueryDocument::new("amber filesystem").unwrap(),
            Some(&bundle),
            Ok(Some(vec![1.0, 0.0, 0.0])),
        )
        .unwrap();
        assert_eq!(result.qualified_count, 1);
        assert_eq!(result.selected[0].memory.memory_id, "memory:a");
        assert!(result.selected[0]
            .plane_ranks
            .iter()
            .all(|rank| rank.rank == 1));
    }

    #[test]
    fn h19_s18_query_encoder_unavailable_degrades_to_qualified_planes() {
        let (state, entries, corpus, profile, vectors) = fixture();
        let bundle = MemoryIndexBundle::build(corpus, profile, &vectors).unwrap();
        let result = hybrid_retrieve(
            &state,
            &entries,
            qualification(&state),
            RetrievalQueryDocument::new("amber filesystem").unwrap(),
            Some(&bundle),
            Err("fixture_encoder_unavailable".to_string()),
        )
        .unwrap();
        assert!(result.planes.iter().any(|plane| {
            plane.plane == "vector_exact_cosine"
                && !plane.available
                && plane.reason.contains("fixture_encoder_unavailable")
        }));
        assert!(result
            .planes
            .iter()
            .any(|plane| plane.plane == "lexical_bm25" && plane.available));
    }

    #[test]
    fn h19_s17_resource_and_causal_qualification_precede_fuzzy_rank() {
        hybrid_fixture_exposes_distinct_lexical_vector_and_exact_causal_ranks();
    }

    #[test]
    fn h19_s19_old_profile_remains_independent() {
        profile_namespaces_are_independent_and_deletable();
    }

    #[test]
    fn h19_s20_vector_cross_product_is_an_enforced_admission_bound() {
        assert_eq!(
            validate_vector_shape_budget(50_000, 384).unwrap(),
            19_200_000
        );
        assert!(validate_vector_shape_budget(10_000, 1536).is_ok());
        assert_eq!(
            validate_vector_shape_budget(50_000, 768).unwrap_err(),
            "memory_index_vector_element_budget_exceeded"
        );
        assert_eq!(
            validate_vector_shape_budget(50_001, 8).unwrap_err(),
            "memory_index_vector_shape_invalid"
        );
    }

    #[test]
    fn h19_s21_derived_gc_retains_a_bounded_number_of_builds() {
        let (_, _, corpus, profile, _) = fixture();
        let root = test_directory("h19-gc");
        for generation in 0..6 {
            let vectors = corpus
                .documents
                .iter()
                .enumerate()
                .map(|(index, document)| {
                    let seed = (generation + 1 + index) as f32;
                    (document.document_id.clone(), vec![seed, 1.0, 0.5])
                })
                .collect();
            let bundle =
                MemoryIndexBundle::build(corpus.clone(), profile.clone(), &vectors).unwrap();
            let lock = acquire_memory_index_build_lock(
                &root,
                "tenant:test",
                "case:memory-index",
                &profile.profile_id,
            )
            .unwrap();
            publish_memory_index_locked(&lock, &bundle, false).unwrap();
        }
        let builds = profile_directory(
            &root,
            "tenant:test",
            "case:memory-index",
            &profile.profile_id,
        )
        .join("builds");
        assert!(fs::read_dir(builds).unwrap().count() <= MAX_RETAINED_BUILDS);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h19_s22_crash_durability_boundaries_never_expose_partial_current() {
        let (_, _, corpus, profile, vectors) = fixture();
        let old = MemoryIndexBundle::build(corpus.clone(), profile.clone(), &vectors).unwrap();
        let changed_vectors = corpus
            .documents
            .iter()
            .map(|document| (document.document_id.clone(), vec![0.0, 0.0, 1.0]))
            .collect();
        let new = MemoryIndexBundle::build(corpus, profile.clone(), &changed_vectors).unwrap();
        for failpoint in [
            MemoryIndexPublicationFailpoint::AfterComponentWriteBeforeFileSync,
            MemoryIndexPublicationFailpoint::AfterFileSyncBeforeBuildDirectorySync,
            MemoryIndexPublicationFailpoint::AfterBuildRenameBeforeBuildsParentSync,
            MemoryIndexPublicationFailpoint::AfterBuildsSyncBeforePointerWrite,
            MemoryIndexPublicationFailpoint::DuringPointerTempWrite,
            MemoryIndexPublicationFailpoint::AfterPointerRenameBeforeProfileDirectorySync,
            MemoryIndexPublicationFailpoint::AfterFinalSyncBeforeAcknowledgement,
        ] {
            let root = test_directory("h19-crash-durability");
            let lock = acquire_memory_index_build_lock(
                &root,
                "tenant:test",
                "case:memory-index",
                &profile.profile_id,
            )
            .unwrap();
            publish_memory_index_locked(&lock, &old, false).unwrap();
            assert!(
                publish_memory_index_locked_with_failpoint(&lock, &new, Some(failpoint)).is_err()
            );
            drop(lock);
            let current = load_current_memory_index(
                &root,
                "tenant:test",
                "case:memory-index",
                &profile.profile_id,
            )
            .unwrap()
            .unwrap();
            assert!(
                current.manifest.index_id == old.manifest.index_id
                    || current.manifest.index_id == new.manifest.index_id
            );
            current.validate_loaded().unwrap();
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn h19_s23_provider_claim_ranking_never_changes_posture() {
        provider_claim_similarity_never_changes_authority_posture();
    }

    #[test]
    fn h19_s24_historical_allow_remains_context_not_authority() {
        hybrid_fixture_exposes_distinct_lexical_vector_and_exact_causal_ranks();
    }

    #[test]
    #[ignore]
    fn memory_index_scale_characterization() {
        for size in [1_000usize, 10_000, 50_000] {
            let entries = (0..size)
                .map(|index| {
                    let mut value = entry(
                        "case:scale",
                        &format!("memory:{index:05}"),
                        "participant:a",
                        &format!(
                            "resource effect project codename item {index} bucket {}",
                            index % 97
                        ),
                        OperationalMemoryPosture::ProviderOriginatedClaim,
                        index as u64 + 1,
                    );
                    value.derived_at_generation = size as u64;
                    value
                })
                .collect::<Vec<_>>();
            let build = OperationalMemoryBuild {
                manifest: OperationalMemoryManifest {
                    schema: OPERATIONAL_MEMORY_MANIFEST_SCHEMA.to_string(),
                    case_id: "case:scale".to_string(),
                    derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
                    source_generation: size as u64,
                    memory_ids: entries
                        .iter()
                        .map(|entry| entry.memory_id.clone())
                        .collect(),
                },
                entries,
            };
            let start = Instant::now();
            let corpus = derive_representation_corpus(&build).unwrap();
            let representation_ms = start.elapsed().as_millis();
            let profile = MemoryRepresentationProfile::new(
                "tenant:scale",
                "test-only:fixture-encoder",
                "fixture:model",
                "scale-revision-1",
                8,
            )
            .unwrap();
            let vectors = corpus
                .documents
                .iter()
                .enumerate()
                .map(|(index, document)| {
                    let mut vector = vec![0.0; 8];
                    vector[index % 8] = 1.0;
                    (document.document_id.clone(), vector)
                })
                .collect::<BTreeMap<_, _>>();
            let lexical_start = Instant::now();
            let lexical = MemoryLexicalIndex::build(&corpus.documents).unwrap();
            let lexical_ms = lexical_start.elapsed().as_millis();
            let vector_start = Instant::now();
            let vector = MemoryVectorIndex::build(&corpus.documents, &profile, &vectors).unwrap();
            let vector_ms = vector_start.elapsed().as_millis();
            let query_start = Instant::now();
            let exact_hits = vector
                .exact_search(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 32)
                .unwrap();
            let exact_query_micros = query_start.elapsed().as_micros();
            let lexical_query_start = Instant::now();
            let lexical_hits = lexical.search("project codename bucket 42", 32).unwrap();
            let lexical_query_micros = lexical_query_start.elapsed().as_micros();
            let bundle = MemoryIndexBundle::build(corpus, profile, &vectors).unwrap();
            let mut state = CaseState::new("case:scale", CaseLifecycle::Open);
            state.generation = size as u64;
            state.tenant_id = Some("tenant:scale".to_string());
            state.participants = vec![ParticipantState {
                participant_id: "participant:a".to_string(),
                roles: vec!["model_provider".to_string()],
                admitted_views: vec![AdmittedView {
                    consumer: "model".to_string(),
                    view_kind: "model_context".to_string(),
                }],
            }];
            let mut retrieval_qualification = qualification(&state);
            retrieval_qualification.max_results = 32;
            let mut all_qualification = retrieval_qualification.clone();
            all_qualification.max_results = build.entries.len();
            let qualification_start = Instant::now();
            let qualified =
                retrieve_operational_memory(&state, &build.entries, all_qualification).unwrap();
            let qualification_micros = qualification_start.elapsed().as_micros();
            let load_validation_start = Instant::now();
            bundle.validate_loaded().unwrap();
            let load_validation_micros = load_validation_start.elapsed().as_micros();
            let source_validation_start = Instant::now();
            validate_memory_index_source(&bundle, &build.entries).unwrap();
            let source_validation_micros = source_validation_start.elapsed().as_micros();
            let admitted = bundle
                .documents
                .iter()
                .map(|document| document.document_id.clone())
                .collect::<BTreeSet<_>>();
            let qualified_lexical_start = Instant::now();
            bundle
                .lexical
                .search_qualified("project codename bucket 42", 32, &admitted)
                .unwrap();
            let qualified_lexical_micros = qualified_lexical_start.elapsed().as_micros();
            let qualified_vector_start = Instant::now();
            bundle
                .vector
                .exact_search_qualified(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 32, &admitted)
                .unwrap();
            let qualified_vector_micros = qualified_vector_start.elapsed().as_micros();
            let deep_validation_start = Instant::now();
            bundle.validate_deep().unwrap();
            let deep_validation_micros = deep_validation_start.elapsed().as_micros();
            let hybrid_query_start = Instant::now();
            let hybrid = hybrid_retrieve(
                &state,
                &build.entries,
                retrieval_qualification.clone(),
                RetrievalQueryDocument::new("project codename bucket 42").unwrap(),
                Some(&bundle),
                Ok(Some(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])),
            )
            .unwrap();
            let hybrid_query_micros = hybrid_query_start.elapsed().as_micros();
            let warm_query_start = Instant::now();
            let warm = hybrid_retrieve(
                &state,
                &build.entries,
                retrieval_qualification,
                RetrievalQueryDocument::new("project codename bucket 42").unwrap(),
                Some(&bundle),
                Ok(Some(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])),
            )
            .unwrap();
            let warm_hybrid_micros = warm_query_start.elapsed().as_micros();
            let root = test_directory("scale-physical");
            let physical_build_start = Instant::now();
            let lock = acquire_memory_index_build_lock(
                &root,
                "tenant:scale",
                "case:scale",
                &bundle.profile.profile_id,
            )
            .unwrap();
            publish_memory_index_locked(&lock, &bundle, false).unwrap();
            drop(lock);
            let physical_build_ms = physical_build_start.elapsed().as_millis();
            let load_start = Instant::now();
            let loaded = load_current_memory_index(
                &root,
                "tenant:scale",
                "case:scale",
                &bundle.profile.profile_id,
            )
            .unwrap()
            .unwrap();
            let physical_load_ms = load_start.elapsed().as_millis();
            let storage_bytes =
                list_memory_index_statuses(&root, "tenant:scale", "case:scale", size as u64)
                    .unwrap()[0]
                    .storage_bytes;
            assert_eq!(loaded.manifest.index_id, bundle.manifest.index_id);
            fs::remove_dir_all(root).unwrap();
            println!(
                "memory_scale entries={size} dimension=8 representation_ms={representation_ms} lexical_build_ms={lexical_ms} fixture_embedding_build_ms={vector_ms} physical_build_ms={physical_build_ms} storage_bytes={storage_bytes} physical_load_ms={physical_load_ms} deep_validation_us={deep_validation_micros} load_validation_us={load_validation_micros} source_qualification_us={qualification_micros} source_revalidation_us={source_validation_micros} bm25_us={qualified_lexical_micros} exact_cosine_us={qualified_vector_micros} hybrid_cold_us={hybrid_query_micros} hybrid_warm_us={warm_hybrid_micros} exact_reference_us={exact_query_micros} lexical_reference_us={lexical_query_micros} exact_hits={} lexical_hits={} qualified={} hybrid_hits={} warm_hits={} peak_memory=not_observed ann=deferred",
                exact_hits.len(),
                lexical_hits.len(),
                qualified.qualified_count,
                hybrid.selected.len(),
                warm.selected.len(),
            );
        }
    }

    #[test]
    #[ignore]
    fn h19_realistic_vector_dimension_characterization() {
        for (size, dimensions) in [
            (1_000usize, vec![384usize, 768, 1024, 1536]),
            (10_000usize, vec![384usize, 768, 1024, 1536]),
            (50_000usize, vec![384usize, 768]),
        ] {
            let entries = (0..size)
                .map(|index| {
                    let mut value = entry(
                        "case:vector-scale",
                        &format!("memory:{index:05}"),
                        "participant:a",
                        "controlled vector scale document",
                        OperationalMemoryPosture::ProviderOriginatedClaim,
                        size as u64,
                    );
                    value.derived_at_generation = size as u64;
                    value
                })
                .collect::<Vec<_>>();
            let corpus = derive_representation_corpus(&OperationalMemoryBuild {
                manifest: OperationalMemoryManifest {
                    schema: OPERATIONAL_MEMORY_MANIFEST_SCHEMA.to_string(),
                    case_id: "case:vector-scale".to_string(),
                    derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
                    source_generation: size as u64,
                    memory_ids: entries
                        .iter()
                        .map(|entry| entry.memory_id.clone())
                        .collect(),
                },
                entries,
            })
            .unwrap();
            for dimension in dimensions {
                let admission = validate_vector_shape_budget(size, dimension);
                if let Err(error) = admission {
                    println!(
                        "memory_vector_scale entries={size} dimension={dimension} posture=refused reason={error} max_elements={MAX_VECTOR_ELEMENTS}"
                    );
                    continue;
                }
                let profile = MemoryRepresentationProfile::new(
                    "tenant:scale",
                    "test-only:fixture-encoder",
                    "fixture:model",
                    &format!("scale-{dimension}"),
                    dimension,
                )
                .unwrap();
                let encode_start = Instant::now();
                let vectors = corpus
                    .documents
                    .iter()
                    .enumerate()
                    .map(|(index, document)| {
                        let mut vector = vec![0.0; dimension];
                        vector[index % dimension] = 1.0;
                        (document.document_id.clone(), vector)
                    })
                    .collect::<BTreeMap<_, _>>();
                let fixture_encode_ms = encode_start.elapsed().as_millis();
                let build_start = Instant::now();
                let index =
                    MemoryVectorIndex::build(&corpus.documents, &profile, &vectors).unwrap();
                let vector_build_ms = build_start.elapsed().as_millis();
                drop(vectors);
                let mut query = vec![0.0; dimension];
                query[0] = 1.0;
                let query_start = Instant::now();
                let hits = index.exact_search(&query, 32).unwrap();
                let exact_query_us = query_start.elapsed().as_micros();
                let raw_vector_bytes = size
                    .checked_mul(dimension)
                    .and_then(|value| value.checked_mul(std::mem::size_of::<f32>()))
                    .unwrap();
                let peak_rss = fs::read_to_string("/proc/self/status")
                    .ok()
                    .and_then(|status| {
                        status
                            .lines()
                            .find(|line| line.starts_with("VmHWM:"))
                            .map(str::to_string)
                    })
                    .unwrap_or_else(|| "not_observed".to_string());
                println!(
                    "memory_vector_scale entries={size} dimension={dimension} posture=admitted fixture_encode_ms={fixture_encode_ms} vector_build_ms={vector_build_ms} exact_query_us={exact_query_us} raw_vector_bytes={raw_vector_bytes} hits={} peak_rss={}",
                    hits.len(),
                    peak_rss.replace(' ', "_")
                );
            }
        }
    }
}
