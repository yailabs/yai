//! Operator inspection and rebuild commands for derived operational memory.

use super::*;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::memory_hierarchy::{
    assertion_is_visible, build_memory_hierarchy, derive_consolidation_input, episode_is_visible,
    MemoryConsolidationInput, SemanticMemoryHierarchy, CONSOLIDATION_CANDIDATE_SCHEMA,
    CONSOLIDATION_NORMALIZER_VERSION, CONSOLIDATION_SEMANTIC_UNIT_BUDGET,
};
use yai_core_engine::memory_index::{
    acquire_memory_index_build_lock, derive_hierarchy_representation_corpus,
    drop_memory_index_locked, find_current_memory_index, hybrid_retrieve_hierarchy,
    list_memory_index_statuses, load_current_memory_index, load_current_memory_index_locked,
    load_last_hierarchy_retrieval, load_last_hybrid_retrieval, publish_memory_index_locked,
    store_last_hierarchy_retrieval, validate_hierarchy_memory_index_compatibility,
    validate_hierarchy_memory_index_source, validate_memory_index_build_budget,
    validate_memory_index_source, HybridRetrievalSetV3, MemoryIndexBundle,
    MemoryRepresentationDocument, MemoryRepresentationProfile, RetrievalQueryDocument,
};
use yai_core_engine::provider_governance::{
    CapabilityProvenance, ProviderCapability, ProviderCircuitPosture, ProviderLocality,
    ProviderTarget, ProviderTrustPosture,
};

fn derive_current_memory_hierarchy(
    store: &LmdbRecordStore,
    case_id: &str,
) -> Result<
    (
        SemanticMemoryHierarchy,
        yai_core_engine::memory::OperationalMemoryBuild,
    ),
    String,
> {
    let state = security::authorize_case_read_if_scoped(store, case_id)?;
    let transitions = store.list_case_transitions(case_id)?;
    let memory = derive_operational_memory(case_id, &transitions)?;
    let hierarchy = build_memory_hierarchy(&state, &transitions, &memory)?;
    Ok((hierarchy, memory))
}

pub(super) fn memory_summary(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    let summary = MemorySummary::from_journal(&journal);
    println!("authority: legacy_compatibility_only");
    println!("records: {}", summary.records);
    println!("memory_candidates: {}", summary.memory_candidates);
    println!("operational: {}", summary.operational);
    println!("decision: {}", summary.decision);
    println!("subject: {}", summary.subject);
    println!("error: {}", summary.error);
    println!("recovery: {}", summary.recovery);
    Ok(())
}

fn parse_memory_purpose(value: &str) -> Result<ProjectionPurpose, String> {
    match value {
        "conversation" | "continue_task" => Ok(ProjectionPurpose::Conversation),
        "filesystem_write_proposal" | "propose_operation" => {
            Ok(ProjectionPurpose::FilesystemWriteProposal)
        }
        "effect_consequence" | "inspect_resource" => Ok(ProjectionPurpose::EffectConsequence),
        "inspection" => Ok(ProjectionPurpose::Inspection),
        _ => Err(format!("unsupported memory retrieval purpose: {value}")),
    }
}

fn derive_current_operational_memory(
    store: &LmdbRecordStore,
    case_id: &str,
) -> Result<(yai_core_engine::memory::OperationalMemoryBuild, usize), String> {
    let state = security::authorize_case_read_if_scoped(store, case_id)?;
    let transitions = store.list_case_transitions(case_id)?;
    let ledger_count = transitions.len();
    let build = derive_operational_memory(case_id, &transitions)?;
    if build.manifest.source_generation != state.generation {
        return Err("operational_memory_case_generation_mismatch".to_string());
    }
    Ok((build, ledger_count))
}

pub(super) fn memory_rebuild(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let dry_run = args.iter().any(|value| value == "--dry-run");
    let store = LmdbRecordStore::open(record_store_path())?;
    let (build, ledger_count) = derive_current_operational_memory(&store, &case_id)?;
    if !dry_run {
        store.replace_case_operational_memory(&build)?;
    }
    println!(
        "memory_rebuild: {}",
        if dry_run { "dry_run" } else { "committed" }
    );
    println!("case_id: {case_id}");
    println!("source_generation: {}", build.manifest.source_generation);
    println!("source_transitions: {ledger_count}");
    println!("derived_entries: {}", build.entries.len());
    println!("derivation_version: {}", build.manifest.derivation_version);
    println!("canonical_ledger_mutated: no");
    Ok(())
}

pub(super) fn memory_clear(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    security::authorize_case_read_if_scoped(&store, &case_id)?;
    let transition_count = store.list_case_transitions(&case_id)?.len();
    store.clear_case_operational_memory(&case_id)?;
    println!("memory_clear: completed");
    println!("case_id: {case_id}");
    println!("derived_entries_remaining: 0");
    println!("canonical_transitions_remaining: {transition_count}");
    Ok(())
}

fn print_operational_memory(entry: &OperationalMemoryEntry) -> Result<(), String> {
    println!("memory_id: {}", entry.memory_id);
    println!("schema: {}", entry.schema);
    println!("case_id: {}", entry.case_id);
    println!("kind: {}", entry.semantic_kind.as_str());
    println!("posture: {}", entry.posture.as_str());
    println!("lifecycle: {}", entry.lifecycle.as_str());
    println!(
        "superseded_by: {}",
        entry.superseded_by.as_deref().unwrap_or("none")
    );
    println!("derived_generation: {}", entry.derived_at_generation);
    println!("description: {}", entry.description);
    println!("value: {:?}", entry.value);
    println!(
        "visible_participants: {}",
        entry.visibility.participant_ids.join(",")
    );
    Ok(())
}

pub(super) fn memory_list(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let include_superseded = args.iter().any(|value| value == "--include-superseded");
    let limit = parse_limit(args)?;
    let store = LmdbRecordStore::open(record_store_path())?;
    security::authorize_case_read_if_scoped(&store, &case_id)?;
    let manifest = store.operational_memory_manifest(&case_id)?;
    let mut entries = store.list_operational_memory(&case_id)?;
    if !include_superseded {
        entries.retain(|entry| entry.lifecycle == OperationalMemoryLifecycle::Active);
    }
    entries.sort_by(|left, right| {
        right
            .provenance
            .generation_end
            .cmp(&left.provenance.generation_end)
            .then_with(|| left.memory_id.cmp(&right.memory_id))
    });
    println!("case_id: {case_id}");
    println!(
        "source_generation: {}",
        manifest
            .as_ref()
            .map(|value| value.source_generation)
            .unwrap_or(0)
    );
    println!("entries_total: {}", entries.len());
    println!("limit: {limit}");
    for entry in entries.into_iter().take(limit) {
        println!(
            "entry: {} kind:{} posture:{} lifecycle:{} generation:{} description:{}",
            entry.memory_id,
            entry.semantic_kind.as_str(),
            entry.posture.as_str(),
            entry.lifecycle.as_str(),
            entry.provenance.generation_end,
            entry.description
        );
    }
    Ok(())
}

pub(super) fn memory_show(args: &[String]) -> Result<(), String> {
    let memory_id = args
        .first()
        .ok_or_else(|| "memory show requires <memory_id>".to_string())?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let entry = store
        .get_operational_memory(memory_id)?
        .ok_or_else(|| format!("operational memory not found: {memory_id}"))?;
    security::authorize_case_read_if_scoped(&store, &entry.case_id)?;
    print_operational_memory(&entry)
}

pub(super) fn memory_provenance(args: &[String]) -> Result<(), String> {
    let memory_id = args
        .first()
        .ok_or_else(|| "memory provenance requires <memory_id>".to_string())?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let entry = store
        .get_operational_memory(memory_id)?
        .ok_or_else(|| format!("operational memory not found: {memory_id}"))?;
    security::authorize_case_read_if_scoped(&store, &entry.case_id)?;
    let transitions = store.list_case_transitions(&entry.case_id)?;
    yai_core_engine::memory::validate_memory_provenance(&entry, &transitions)?;
    println!("memory_id: {}", entry.memory_id);
    println!("provenance_valid: yes");
    println!("generation_start: {}", entry.provenance.generation_start);
    println!("generation_end: {}", entry.provenance.generation_end);
    println!(
        "transition_ids: {}",
        entry.provenance.transition_ids.join(",")
    );
    println!(
        "observation_ids: {}",
        entry.provenance.observation_ids.join(",")
    );
    println!(
        "effect_receipt_ids: {}",
        entry.provenance.effect_receipt_ids.join(",")
    );
    println!("causal_refs: {}", entry.provenance.causal_refs.join(","));
    Ok(())
}

pub(super) fn memory_retrieve(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let purpose = parse_memory_purpose(&named_arg(args, "--purpose")?)?;
    let limit = parse_limit(args)?;
    let resource_refs = optional_arg(args, "--resource").into_iter().collect();
    let causal_refs = optional_arg(args, "--causal-ref").into_iter().collect();
    let semantic_kinds = optional_arg(args, "--kind")
        .map(|value| {
            value
                .split(',')
                .map(|kind| {
                    yai_core_engine::memory::OperationalMemoryKind::parse(kind)
                        .ok_or_else(|| format!("unsupported memory kind: {kind}"))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = security::authorize_case_read_if_scoped(&store, &case_id)?;
    let manifest = store.operational_memory_manifest(&case_id)?;
    let entries = if manifest
        .as_ref()
        .is_some_and(|value| value.is_current(&case_id, state.generation))
    {
        store.list_operational_memory(&case_id)?
    } else {
        let (build, _) = derive_current_operational_memory(&store, &case_id)?;
        store.replace_case_operational_memory(&build)?;
        build.entries
    };
    let transition_count_before = store.list_case_transitions(&case_id)?.len();
    let result = retrieve_operational_memory(
        &state,
        &entries,
        RetrievalQualification {
            case_id: case_id.clone(),
            participant_id,
            consumer: "model".to_string(),
            view_kind: "model_context".to_string(),
            purpose,
            case_generation: state.generation,
            resource_refs,
            semantic_kinds,
            causal_refs,
            max_results: limit,
            include_superseded: args.iter().any(|value| value == "--include-superseded"),
        },
    )?;
    let transition_count_after = store.list_case_transitions(&case_id)?.len();
    println!("retrieval_id: {}", result.retrieval_id);
    println!("source_memories: {}", result.source_memory_count);
    println!("qualified: {}", result.qualified_count);
    println!("selected: {}", result.selected_count);
    println!("omitted: {}", result.omitted_count);
    println!("rejections: {:?}", result.rejections);
    println!(
        "canonical_ledger_mutated: {}",
        yes_no(transition_count_before != transition_count_after)
    );
    for selected in result.selected {
        println!(
            "selected_memory: {} score:{} reasons:{} description:{}",
            selected.memory.memory_id,
            selected.score,
            selected.ranking_reasons.join(","),
            selected.memory.description
        );
    }
    Ok(())
}

fn memory_index_root() -> PathBuf {
    yai_home().join("store").join("derived-memory")
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn parse_dimension(args: &[String]) -> Result<usize, String> {
    named_arg(args, "--dimension")?
        .parse::<usize>()
        .map_err(|_| "memory_profile_vector_dimension_invalid".to_string())
}

fn credential_for_memory_encoder(target: &ProviderTarget) -> Result<Option<String>, String> {
    if target.credential_ref == "none" {
        return Ok(None);
    }
    let name = target
        .credential_ref
        .strip_prefix("env:")
        .ok_or_else(|| "memory_encoder_credential_reference_unsupported".to_string())?;
    super::provider::env_var(name)
        .map(Some)
        .ok_or_else(|| "memory_encoder_credential_unavailable".to_string())
}

#[derive(Clone)]
struct QualifiedMemoryEncoder {
    target: ProviderTarget,
    credential: Option<String>,
    qualification_id: String,
    credential_revision: u64,
}

impl QualifiedMemoryEncoder {
    fn same_governed_identity(&self, other: &Self) -> bool {
        self.target == other.target
            && self.qualification_id == other.qualification_id
            && self.credential_revision == other.credential_revision
            && self.credential == other.credential
    }
}

fn require_case_generation(expected: u64, current: u64, publication: bool) -> Result<(), String> {
    if expected == current {
        Ok(())
    } else if publication {
        Err("memory_index_case_generation_changed_during_publication".to_string())
    } else {
        Err("memory_index_case_generation_changed_during_build".to_string())
    }
}

fn require_memory_encoder_snapshot(
    admitted: &QualifiedMemoryEncoder,
    current: Result<QualifiedMemoryEncoder, String>,
) -> Result<(), String> {
    let current = current?;
    if admitted.same_governed_identity(&current) {
        Ok(())
    } else {
        Err("memory_encoder_governance_changed_during_build".to_string())
    }
}

fn qualified_memory_encoder(
    store: &LmdbRecordStore,
    tenant_id: &str,
    profile: &MemoryRepresentationProfile,
) -> Result<QualifiedMemoryEncoder, String> {
    profile.validate()?;
    if profile.tenant_id != tenant_id {
        return Err("memory_encoder_profile_tenant_mismatch".to_string());
    }
    let authenticated = security::authenticate_local()?;
    let (target, qualification, trust, health) =
        store.provider_posture_authorized(&authenticated, &profile.encoder_target_id)?;
    if target.tenant_id != tenant_id
        || target.model_id != profile.encoder_model_id
        || target.locality != ProviderLocality::Loopback
    {
        return Err("memory_encoder_target_profile_or_locality_mismatch".to_string());
    }
    let qualification =
        qualification.ok_or_else(|| "memory_encoder_qualification_missing".to_string())?;
    if !qualification.is_current(current_time_ms()) {
        return Err("memory_encoder_qualification_stale".to_string());
    }
    let capability = qualification
        .capabilities
        .iter()
        .find(|evidence| evidence.capability == ProviderCapability::TextEmbedding)
        .ok_or_else(|| "memory_encoder_text_embedding_capability_missing".to_string())?;
    if capability.provenance != CapabilityProvenance::Qualified
        || capability.verified_minimum != Some(profile.vector_dimension as u64)
    {
        return Err("memory_encoder_dimension_not_qualified".to_string());
    }
    if trust.is_none_or(|event| event.posture != ProviderTrustPosture::Approved) {
        return Err("memory_encoder_trust_not_approved".to_string());
    }
    if health.circuit_at(current_time_ms()) != ProviderCircuitPosture::Closed {
        return Err("memory_encoder_circuit_not_closed".to_string());
    }
    let credential = credential_for_memory_encoder(&target)?;
    Ok(QualifiedMemoryEncoder {
        target,
        credential,
        qualification_id: qualification.qualification_id,
        credential_revision: qualification.credential_revision,
    })
}

fn parse_embedding_response(
    body: &[u8],
    model_id: &str,
    expected_count: usize,
    expected_dimension: usize,
) -> Result<Vec<Vec<f32>>, String> {
    let value = super::provider_governance_cli::strict_json(body)?;
    if value.get("model").and_then(Value::as_str) != Some(model_id) {
        return Err("memory_encoder_response_model_mismatch".to_string());
    }
    let data = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| "memory_encoder_response_data_missing".to_string())?;
    if data.len() != expected_count {
        return Err("memory_encoder_response_count_mismatch".to_string());
    }
    let mut indexed = BTreeMap::new();
    for item in data {
        let index = item
            .get("index")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .ok_or_else(|| "memory_encoder_response_index_missing".to_string())?;
        if index >= expected_count || indexed.contains_key(&index) {
            return Err("memory_encoder_response_index_invalid".to_string());
        }
        let values = item
            .get("embedding")
            .and_then(Value::as_array)
            .ok_or_else(|| "memory_encoder_response_vector_missing".to_string())?;
        if values.len() != expected_dimension {
            return Err("memory_encoder_response_dimension_mismatch".to_string());
        }
        let vector = values
            .iter()
            .map(|value| {
                value
                    .as_f64()
                    .filter(|value| value.is_finite())
                    .map(|value| value as f32)
                    .filter(|value| value.is_finite())
                    .ok_or_else(|| "memory_encoder_response_non_finite".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let norm_squared = vector
            .iter()
            .map(|value| f64::from(*value) * f64::from(*value))
            .sum::<f64>();
        if !norm_squared.is_finite() || norm_squared <= f64::EPSILON {
            return Err("memory_encoder_response_zero_vector".to_string());
        }
        indexed.insert(index, vector);
    }
    (0..expected_count)
        .map(|index| {
            indexed
                .remove(&index)
                .ok_or_else(|| "memory_encoder_response_index_missing".to_string())
        })
        .collect()
}

fn encode_memory_texts(
    store: &LmdbRecordStore,
    tenant_id: &str,
    profile: &MemoryRepresentationProfile,
    texts: &[String],
) -> Result<Vec<Vec<f32>>, String> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let admitted = qualified_memory_encoder(store, tenant_id, profile)?;
    let endpoint = super::provider_transport::parse_provider_endpoint(&admitted.target.endpoint)?;
    let mut vectors = Vec::with_capacity(texts.len());
    for batch in texts.chunks(32) {
        require_memory_encoder_snapshot(
            &admitted,
            qualified_memory_encoder(store, tenant_id, profile),
        )?;
        let body = serde_json::to_vec(&serde_json::json!({
            "model": admitted.target.model_id,
            "input": batch,
            "encoding_format": "float"
        }))
        .map_err(|error| format!("memory_encoder_request_encode_failed: {error}"))?;
        let response = super::provider_transport::provider_http(
            &endpoint,
            Some(&admitted.target.locality),
            "POST",
            &endpoint.api_path("embeddings"),
            &body,
            admitted.credential.as_deref(),
        )?;
        if !(200..300).contains(&response.status) {
            return Err(format!(
                "memory_encoder_http_status:{}:bytes={}",
                response.status, response.request_bytes_written
            ));
        }
        let parsed = parse_embedding_response(
            &response.body,
            &profile.encoder_model_id,
            batch.len(),
            profile.vector_dimension,
        )?;
        require_memory_encoder_snapshot(
            &admitted,
            qualified_memory_encoder(store, tenant_id, profile),
        )?;
        vectors.extend(parsed);
    }
    Ok(vectors)
}

fn emit_json<T: Serialize>(value: &T) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string(value)
            .map_err(|error| format!("memory_cli_json_encode_failed: {error}"))?
    );
    Ok(())
}

fn profile_from_build_args(
    args: &[String],
    tenant_id: &str,
    store: &LmdbRecordStore,
) -> Result<MemoryRepresentationProfile, String> {
    let target_id = named_arg(args, "--encoder-target")?;
    let revision = named_arg(args, "--encoder-revision")?;
    let dimension = parse_dimension(args)?;
    let authenticated = security::authenticate_local()?;
    let (target, _, _, _) = store.provider_posture_authorized(&authenticated, &target_id)?;
    MemoryRepresentationProfile::new_v2(
        tenant_id,
        &target.target_id,
        &target.model_id,
        &revision,
        dimension,
    )
}

fn emit_index_build_result(
    args: &[String],
    action: &str,
    bundle: &MemoryIndexBundle,
    reused: bool,
) -> Result<(), String> {
    if args.iter().any(|value| value == "--json") {
        return emit_json(&serde_json::json!({
            "schema": "yai.memory_index_build_result.v1",
            "action": action,
            "case_id": bundle.corpus.case_id,
            "case_generation": bundle.corpus.source_generation,
            "corpus_manifest_id": bundle.corpus.manifest_id,
            "representation_profile_id": bundle.profile.profile_id,
            "index_manifest_id": bundle.manifest.index_id,
            "documents": bundle.documents.len(),
            "dimension": bundle.profile.vector_dimension,
            "lexical": "bm25",
            "vector": "exact_cosine",
            "ann": "deferred_exact_scan_within_bound",
            "publication": if reused { "existing_equivalent" } else { "atomic" },
            "canonical_transition_mutated": false
        }));
    }
    println!(
        "memory_index_{action}: {}",
        if reused {
            "existing_equivalent"
        } else {
            "published"
        }
    );
    println!("case_id: {}", bundle.corpus.case_id);
    println!("case_generation: {}", bundle.corpus.source_generation);
    println!("corpus_manifest_id: {}", bundle.corpus.manifest_id);
    println!("representation_profile_id: {}", bundle.profile.profile_id);
    println!("index_manifest_id: {}", bundle.manifest.index_id);
    println!("documents: {}", bundle.documents.len());
    println!("dimension: {}", bundle.profile.vector_dimension);
    println!("lexical: bm25");
    println!("vector: exact_cosine");
    println!("ann: deferred_exact_scan_within_bound");
    println!("canonical_transition_mutated: no");
    Ok(())
}

fn memory_index_build_or_rebuild(args: &[String], rebuild: bool) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = security::authorize_case_read_if_scoped(&store, &case_id)?;
    let tenant_id = state
        .tenant_id
        .as_deref()
        .ok_or_else(|| "memory_index_requires_tenant_scoped_case".to_string())?;
    let (hierarchy, memory) = derive_current_memory_hierarchy(&store, &case_id)?;
    store.replace_case_operational_memory(&memory)?;
    let corpus = derive_hierarchy_representation_corpus(&memory, &hierarchy)?;

    let requested_profile = optional_arg(args, "--profile");
    let configured_profile = optional_arg(args, "--encoder-target").is_some();
    let profile = if configured_profile {
        profile_from_build_args(args, tenant_id, &store)?
    } else if rebuild {
        let existing = find_current_memory_index(
            &memory_index_root(),
            tenant_id,
            &case_id,
            requested_profile.as_deref(),
        )?
        .ok_or_else(|| "memory_index_rebuild_source_profile_missing".to_string())?;
        existing.profile
    } else {
        return Err("memory_index_build_encoder_profile_required".to_string());
    };
    if requested_profile
        .as_deref()
        .is_some_and(|value| value != profile.profile_id)
    {
        return Err("memory_index_requested_profile_mismatch".to_string());
    }
    validate_memory_index_build_budget(&corpus, &profile)?;

    let lock = acquire_memory_index_build_lock(
        &memory_index_root(),
        tenant_id,
        &case_id,
        &profile.profile_id,
    )?;
    let after = load_current_memory_index_locked(&lock);
    if after.is_err() {
        drop_memory_index_locked(&lock)?;
    } else if let Some(existing) = after? {
        if existing.corpus.corpus_digest == corpus.manifest.corpus_digest {
            return emit_index_build_result(
                args,
                if rebuild { "rebuild" } else { "build" },
                &existing,
                true,
            );
        }
    }

    let texts = corpus
        .documents
        .iter()
        .map(|document| document.canonical_text.clone())
        .collect::<Vec<_>>();
    let encoded = encode_memory_texts(&store, tenant_id, &profile, &texts)?;
    let current_generation = store
        .get_case_state(&case_id)?
        .ok_or_else(|| "memory_index_case_disappeared_during_build".to_string())?
        .generation;
    require_case_generation(corpus.manifest.source_generation, current_generation, false)?;
    let vectors = corpus
        .documents
        .iter()
        .zip(encoded)
        .map(|(document, vector)| (document.document_id.clone(), vector))
        .collect::<BTreeMap<_, _>>();
    let bundle = MemoryIndexBundle::build(corpus, profile, &vectors)?;
    let current_generation = store
        .get_case_state(&case_id)?
        .ok_or_else(|| "memory_index_case_disappeared_during_build".to_string())?
        .generation;
    require_case_generation(bundle.corpus.source_generation, current_generation, false)?;
    publish_memory_index_locked(&lock, &bundle, false)?;
    let current_generation = store
        .get_case_state(&case_id)?
        .ok_or_else(|| "memory_index_case_disappeared_during_build".to_string())?
        .generation;
    if require_case_generation(bundle.corpus.source_generation, current_generation, true).is_err() {
        drop_memory_index_locked(&lock)?;
        return Err("memory_index_case_generation_changed_during_publication".to_string());
    }
    emit_index_build_result(
        args,
        if rebuild { "rebuild" } else { "build" },
        &bundle,
        false,
    )
}

fn memory_index_status(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = security::authorize_case_read_if_scoped(&store, &case_id)?;
    let tenant_id = state
        .tenant_id
        .as_deref()
        .ok_or_else(|| "memory_index_requires_tenant_scoped_case".to_string())?;
    let statuses =
        list_memory_index_statuses(&memory_index_root(), tenant_id, &case_id, state.generation)?;
    if args.iter().any(|value| value == "--json") {
        return emit_json(&serde_json::json!({
            "schema": "yai.memory_index_status.v1",
            "case_id": case_id,
            "case_generation": state.generation,
            "derived_store": memory_index_root(),
            "indexes": statuses,
            "canonical_authority": "transition_history",
            "lmdb_databases_added": 0
        }));
    }
    println!("case_id: {case_id}");
    println!("case_generation: {}", state.generation);
    println!("indexes: {}", statuses.len());
    for status in statuses {
        println!(
            "index: {} profile:{} posture:{} generation:{} items:{} dimension:{} format:{} bytes:{} integrity:{} ann:{:?} integrity_error:{}",
            status.index_id,
            status.profile_id,
            status.posture,
            status.source_generation,
            status.item_count,
            status.dimension,
            status.physical_format,
            status.storage_bytes,
            status.integrity_posture,
            status.ann_posture,
            status.integrity_error.as_deref().unwrap_or("none")
        );
    }
    println!("canonical_authority: transition_history");
    println!("lmdb_databases_added: 0");
    Ok(())
}

fn memory_index_verify(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let profile_id = named_arg(args, "--profile")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = security::authorize_case_read_if_scoped(&store, &case_id)?;
    let tenant_id = state
        .tenant_id
        .as_deref()
        .ok_or_else(|| "memory_index_requires_tenant_scoped_case".to_string())?;
    let (hierarchy, memory) = derive_current_memory_hierarchy(&store, &case_id)?;
    let (posture, index_id, corpus_id, detail) =
        match load_current_memory_index(&memory_index_root(), tenant_id, &case_id, &profile_id) {
            Ok(None) => (
                "missing",
                "none".to_string(),
                "none".to_string(),
                "no current derived index".to_string(),
            ),
            Err(error) => (
                "corrupt",
                "unavailable".to_string(),
                "unavailable".to_string(),
                error,
            ),
            Ok(Some(index)) if !index.is_current(&case_id, state.generation) => (
                "stale",
                index.manifest.index_id,
                index.corpus.manifest_id,
                "source generation differs from current Case".to_string(),
            ),
            Ok(Some(index)) => {
                let index_id = index.manifest.index_id.clone();
                let corpus_id = index.corpus.manifest_id.clone();
                let source_validation = if index.corpus.representation_contract_version
                    == yai_core_engine::memory_index::MEMORY_REPRESENTATION_CONTRACT_V2
                {
                    validate_hierarchy_memory_index_source(&index, &memory, &hierarchy)
                } else {
                    validate_memory_index_source(&index, &memory.entries)
                };
                match index.validate_deep().and(source_validation) {
                    Ok(()) => (
                        "current",
                        index_id,
                        corpus_id,
                        "deep validation and canonical source revalidation passed".to_string(),
                    ),
                    Err(error) if error.contains("source_") => {
                        ("source_divergent", index_id, corpus_id, error)
                    }
                    Err(error) => ("corrupt", index_id, corpus_id, error),
                }
            }
        };
    let result = serde_json::json!({
        "schema": "yai.memory_index_verify.v1",
        "case_id": case_id,
        "case_generation": state.generation,
        "profile_id": profile_id,
        "index_manifest_id": index_id,
        "corpus_manifest_id": corpus_id,
        "posture": posture,
        "physical_format": yai_core_engine::memory_index::DERIVED_MEMORY_PHYSICAL_SCHEMA,
        "validation": "deep_plus_current_memory_hierarchy_source",
        "detail": detail,
        "canonical_authority": "transition_history"
    });
    if args.iter().any(|value| value == "--json") {
        emit_json(&result)
    } else {
        println!("case_id: {case_id}");
        println!("case_generation: {}", state.generation);
        println!("profile_id: {profile_id}");
        println!("index_manifest_id: {index_id}");
        println!("corpus_manifest_id: {corpus_id}");
        println!("posture: {posture}");
        println!(
            "physical_format: {}",
            yai_core_engine::memory_index::DERIVED_MEMORY_PHYSICAL_SCHEMA
        );
        println!("validation: deep_plus_current_memory_hierarchy_source");
        println!("detail: {detail}");
        println!("canonical_authority: transition_history");
        Ok(())
    }
}

fn memory_case_show(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = security::authorize_case_read_if_scoped(&store, &case_id)?;
    let tenant_id = state
        .tenant_id
        .as_deref()
        .ok_or_else(|| "memory_index_requires_tenant_scoped_case".to_string())?;
    let (memory, transition_count) = derive_current_operational_memory(&store, &case_id)?;
    let active = memory
        .entries
        .iter()
        .filter(|entry| entry.lifecycle == OperationalMemoryLifecycle::Active)
        .count();
    let statuses =
        list_memory_index_statuses(&memory_index_root(), tenant_id, &case_id, state.generation)?;
    let last_retrievals = statuses
        .iter()
        .filter(|status| status.posture != "corrupt")
        .filter_map(|status| {
            load_last_hybrid_retrieval(
                &memory_index_root(),
                tenant_id,
                &case_id,
                &status.profile_id,
            )
            .ok()
            .flatten()
            .map(|retrieval| retrieval.retrieval_id)
        })
        .collect::<Vec<_>>();
    if args.iter().any(|value| value == "--json") {
        return emit_json(&serde_json::json!({
            "schema": "yai.case_memory_view.v1",
            "case_id": case_id,
            "canonical_source_generation": state.generation,
            "canonical_transition_count": transition_count,
            "operational_memory": {
                "schema": memory.manifest.schema,
                "derivation": memory.manifest.derivation_version,
                "entries": memory.entries.len(),
                "active": active,
                "superseded": memory.entries.len().saturating_sub(active)
            },
            "indexes": statuses,
            "last_retrieval_ids": last_retrievals
        }));
    }
    println!("case_id: {case_id}");
    println!("canonical_source_generation: {}", state.generation);
    println!("canonical_transitions: {transition_count}");
    println!("operational_memory_entries: {}", memory.entries.len());
    println!("operational_memory_active: {active}");
    println!(
        "operational_memory_superseded: {}",
        memory.entries.len().saturating_sub(active)
    );
    println!("indexes: {}", statuses.len());
    for status in statuses {
        println!(
            "index: {} profile:{} posture:{} lexical:bm25 vector:exact_cosine ann:{:?} integrity_error:{}",
            status.index_id,
            status.profile_id,
            status.posture,
            status.ann_posture,
            status.integrity_error.as_deref().unwrap_or("none")
        );
    }
    println!(
        "last_retrieval_ids: {}",
        if last_retrievals.is_empty() {
            "none".to_string()
        } else {
            last_retrievals.join(",")
        }
    );
    Ok(())
}

fn hybrid_qualification(
    state: &yai_core_engine::transition::CaseState,
    args: &[String],
    purpose: ProjectionPurpose,
) -> Result<RetrievalQualification, String> {
    Ok(RetrievalQualification {
        case_id: state.case_id.clone(),
        participant_id: named_arg(args, "--participant")?,
        consumer: "model".to_string(),
        view_kind: "model_context".to_string(),
        purpose,
        case_generation: state.generation,
        resource_refs: optional_arg(args, "--resource").into_iter().collect(),
        semantic_kinds: Vec::new(),
        causal_refs: optional_arg(args, "--causal-ref").into_iter().collect(),
        max_results: parse_limit(args)?,
        include_superseded: args.iter().any(|value| value == "--include-superseded"),
    })
}

fn execute_hybrid_retrieval(
    store: &LmdbRecordStore,
    state: &yai_core_engine::transition::CaseState,
    entries: &[OperationalMemoryEntry],
    qualification: RetrievalQualification,
    query: &str,
    profile_id: Option<&str>,
) -> Result<HybridRetrievalSetV3, String> {
    let tenant_id = state
        .tenant_id
        .as_deref()
        .ok_or_else(|| "memory_index_requires_tenant_scoped_case".to_string())?;
    let (index, mut derived_failure) = match find_current_memory_index(
        &memory_index_root(),
        tenant_id,
        &state.case_id,
        profile_id,
    ) {
        Ok(index) => (index, None),
        Err(error) => (None, Some(error)),
    };
    let transitions = store.list_case_transitions(&state.case_id)?;
    let operational = derive_operational_memory(&state.case_id, &transitions)?;
    if operational.entries != entries {
        return Err("runtime_operational_memory_source_divergent".to_string());
    }
    let hierarchy = build_memory_hierarchy(state, &transitions, &operational)?;
    let query_document = RetrievalQueryDocument::new_v2(query)?;
    let stale_index = index
        .as_ref()
        .is_some_and(|index| !index.is_current(&state.case_id, state.generation));
    let (usable_index, query_vector) = match index {
        Some(index) if index.is_current(&state.case_id, state.generation) => {
            match validate_hierarchy_memory_index_compatibility(&index, &operational, &hierarchy) {
                Ok(()) => {
                    let vector = encode_memory_texts(
                        store,
                        tenant_id,
                        &index.profile,
                        std::slice::from_ref(&query_document.canonical_text),
                    )
                    .map(|mut vectors| vectors.pop());
                    (Some(index), vector)
                }
                Err(error) => {
                    derived_failure = Some(error);
                    (None, Ok(None))
                }
            }
        }
        Some(_) => {
            derived_failure = Some("memory_index_stale_for_case_generation".to_string());
            (None, Ok(None))
        }
        None => (None, Ok(None)),
    };
    let indexed_retrieval = hybrid_retrieve_hierarchy(
        state,
        &operational,
        &hierarchy,
        qualification.clone(),
        query_document.clone(),
        usable_index.as_ref(),
        query_vector,
    );
    let mut retrieval = match indexed_retrieval {
        Ok(retrieval) => retrieval,
        Err(error) if usable_index.is_some() => {
            derived_failure = Some(error);
            hybrid_retrieve_hierarchy(
                state,
                &operational,
                &hierarchy,
                qualification,
                query_document,
                None,
                Ok(None),
            )?
        }
        Err(error) => return Err(error),
    };
    if stale_index || derived_failure.is_some() {
        let reason = derived_failure
            .as_deref()
            .unwrap_or("memory_index_stale_for_case_generation");
        for plane in &mut retrieval.planes {
            if plane.plane == "lexical_bm25" || plane.plane == "vector_exact_cosine" {
                plane.reason = format!("derived_index_unavailable:{reason}");
            }
        }
    }
    if retrieval.representation_profile_id.is_some() {
        store_last_hierarchy_retrieval(&memory_index_root(), tenant_id, &retrieval)?;
    }
    Ok(retrieval)
}

fn refresh_runtime_index_if_stale(
    store: &LmdbRecordStore,
    state: &yai_core_engine::transition::CaseState,
    entries: &[OperationalMemoryEntry],
    profile_id: &str,
) -> Result<(), String> {
    let tenant_id = state
        .tenant_id
        .as_deref()
        .ok_or_else(|| "memory_index_requires_tenant_scoped_case".to_string())?;
    let existing = find_current_memory_index(
        &memory_index_root(),
        tenant_id,
        &state.case_id,
        Some(profile_id),
    )?
    .ok_or_else(|| "memory_index_not_found".to_string())?;
    if existing.is_current(&state.case_id, state.generation) {
        return Ok(());
    }
    if existing.profile.profile_id != profile_id {
        return Err("memory_index_requested_profile_mismatch".to_string());
    }
    let manifest = store
        .operational_memory_manifest(&state.case_id)?
        .filter(|manifest| manifest.is_current(&state.case_id, state.generation))
        .ok_or_else(|| "operational_memory_manifest_not_current".to_string())?;
    let operational = yai_core_engine::memory::OperationalMemoryBuild {
        manifest,
        entries: entries.to_vec(),
    };
    let transitions = store.list_case_transitions(&state.case_id)?;
    let hierarchy = build_memory_hierarchy(state, &transitions, &operational)?;
    let corpus = derive_hierarchy_representation_corpus(&operational, &hierarchy)?;
    let lock = acquire_memory_index_build_lock(
        &memory_index_root(),
        tenant_id,
        &state.case_id,
        profile_id,
    )?;
    if load_current_memory_index_locked(&lock)?
        .is_some_and(|bundle| bundle.is_current(&state.case_id, state.generation))
    {
        return Ok(());
    }
    let texts = corpus
        .documents
        .iter()
        .map(|document| document.canonical_text.clone())
        .collect::<Vec<_>>();
    let encoded = encode_memory_texts(store, tenant_id, &existing.profile, &texts)?;
    let current_generation = store
        .get_case_state(&state.case_id)?
        .ok_or_else(|| "memory_index_case_disappeared_during_build".to_string())?
        .generation;
    require_case_generation(corpus.manifest.source_generation, current_generation, false)?;
    let vectors = corpus
        .documents
        .iter()
        .zip(encoded)
        .map(|(document, vector)| (document.document_id.clone(), vector))
        .collect::<BTreeMap<_, _>>();
    let bundle = MemoryIndexBundle::build(corpus, existing.profile, &vectors)?;
    publish_memory_index_locked(&lock, &bundle, false)?;
    let current_generation = store
        .get_case_state(&state.case_id)?
        .ok_or_else(|| "memory_index_case_disappeared_during_build".to_string())?
        .generation;
    if require_case_generation(bundle.corpus.source_generation, current_generation, true).is_err() {
        drop_memory_index_locked(&lock)?;
        return Err("memory_index_case_generation_changed_during_publication".to_string());
    }
    Ok(())
}

fn memory_search(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let query = named_arg(args, "--query")?;
    let purpose = optional_arg(args, "--purpose")
        .map(|value| parse_memory_purpose(&value))
        .transpose()?
        .unwrap_or(ProjectionPurpose::Conversation);
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = security::authorize_case_read_if_scoped(&store, &case_id)?;
    let (memory, _) = derive_current_operational_memory(&store, &case_id)?;
    store.replace_case_operational_memory(&memory)?;
    let qualification = hybrid_qualification(&state, args, purpose)?;
    let retrieval = execute_hybrid_retrieval(
        &store,
        &state,
        &memory.entries,
        qualification,
        &query,
        optional_arg(args, "--profile").as_deref(),
    )?;
    if args.iter().any(|value| value == "--json") {
        return emit_json(&retrieval);
    }
    println!("retrieval_id: {}", retrieval.retrieval_id);
    println!("case_id: {}", retrieval.case_id);
    println!("case_generation: {}", retrieval.case_generation);
    println!("query_id: {}", retrieval.query.query_id);
    println!("qualified: {}", retrieval.qualified_count);
    println!("selected: {}", retrieval.selected_count);
    for plane in &retrieval.planes {
        println!(
            "plane: {} available:{} candidates:{} reason:{}",
            plane.plane, plane.available, plane.candidate_count, plane.reason
        );
    }
    println!("rank\tfamily\tepistemic_class\tlifecycle\tplanes\tdescription\tgeneration");
    for (rank, selected) in retrieval.selected.iter().enumerate() {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            rank + 1,
            selected.source.family(),
            selected.source.epistemic_class(),
            selected.source.lifecycle(),
            selected
                .plane_ranks
                .iter()
                .map(|plane| plane.plane.as_str())
                .collect::<Vec<_>>()
                .join(","),
            selected.source.description().replace('\t', " "),
            selected.source.generation_end()
        );
    }
    Ok(())
}

fn memory_index_drop(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let profile_id = named_arg(args, "--profile")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = security::authorize_case_read_if_scoped(&store, &case_id)?;
    let tenant_id = state
        .tenant_id
        .as_deref()
        .ok_or_else(|| "memory_index_requires_tenant_scoped_case".to_string())?;
    let before = store.list_case_transitions(&case_id)?.len();
    let lock =
        acquire_memory_index_build_lock(&memory_index_root(), tenant_id, &case_id, &profile_id)?;
    let dropped = drop_memory_index_locked(&lock)?;
    let after = store.list_case_transitions(&case_id)?.len();
    let result = serde_json::json!({
        "schema": "yai.memory_index_drop_result.v1",
        "case_id": case_id,
        "profile_id": profile_id,
        "dropped": dropped,
        "canonical_transitions_before": before,
        "canonical_transitions_after": after,
        "semantic_continuity_preserved": before == after
    });
    if args.iter().any(|value| value == "--json") {
        emit_json(&result)
    } else {
        println!(
            "memory_index_drop: {}",
            if dropped { "completed" } else { "absent" }
        );
        println!("case_id: {case_id}");
        println!("profile_id: {profile_id}");
        println!("canonical_transitions_remaining: {after}");
        println!("semantic_continuity_preserved: {}", yes_no(before == after));
        Ok(())
    }
}

fn memory_retrieval_show(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = security::authorize_case_read_if_scoped(&store, &case_id)?;
    let tenant_id = state
        .tenant_id
        .as_deref()
        .ok_or_else(|| "memory_index_requires_tenant_scoped_case".to_string())?;
    let index = find_current_memory_index(
        &memory_index_root(),
        tenant_id,
        &case_id,
        optional_arg(args, "--profile").as_deref(),
    )?
    .ok_or_else(|| "memory_index_not_found".to_string())?;
    let retrieval = load_last_hierarchy_retrieval(
        &memory_index_root(),
        tenant_id,
        &case_id,
        &index.profile.profile_id,
    )?
    .ok_or_else(|| "memory_retrieval_not_found".to_string())?;
    if args.iter().any(|value| value == "--json") {
        emit_json(&retrieval)
    } else {
        println!("retrieval_id: {}", retrieval.retrieval_id);
        println!("case_id: {}", retrieval.case_id);
        println!("case_generation: {}", retrieval.case_generation);
        println!("query_id: {}", retrieval.query.query_id);
        println!(
            "index_manifest_id: {}",
            retrieval.index_manifest_id.as_deref().unwrap_or("none")
        );
        println!(
            "selected_memory_ids: {}",
            retrieval.selected_memory_ids.join(",")
        );
        Ok(())
    }
}

pub(super) fn runtime_hybrid_retrieve(
    store: &LmdbRecordStore,
    state: &yai_core_engine::transition::CaseState,
    entries: &[OperationalMemoryEntry],
    qualification: RetrievalQualification,
    query: &str,
) -> Result<HybridRetrievalSetV3, String> {
    let configured_profile = super::provider::env_var("YAI_MEMORY_PROFILE_ID");
    if let Some(profile_id) = configured_profile.as_deref() {
        if let Err(error) = refresh_runtime_index_if_stale(store, state, entries, profile_id) {
            eprintln!(
                "warning: runtime memory index refresh unavailable; derived planes will degrade: {error}"
            );
        }
    }
    execute_hybrid_retrieval(
        store,
        state,
        entries,
        qualification,
        query,
        configured_profile.as_deref(),
    )
}

fn memory_episodes_show(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let (hierarchy, _) = derive_current_memory_hierarchy(&store, &case_id)?;
    let episodes = hierarchy
        .episodes
        .into_iter()
        .filter(|episode| episode_is_visible(episode, &participant_id))
        .collect::<Vec<_>>();
    if args.iter().any(|value| value == "--json") {
        return emit_json(&serde_json::json!({
            "schema": "yai.memory_episode_list.v1",
            "case_id": case_id,
            "source_generation": hierarchy.manifest.source_generation,
            "hierarchy_id": hierarchy.manifest.hierarchy_id,
            "episodes": episodes
        }));
    }
    println!("case_id: {case_id}");
    println!(
        "source_generation: {}",
        hierarchy.manifest.source_generation
    );
    println!("hierarchy_id: {}", hierarchy.manifest.hierarchy_id);
    println!("episode_id\tgeneration\tkind\tposture\tresources\ttransitions");
    for episode in episodes {
        println!(
            "{}\t{}..{}\t{:?}\t{:?}\t{}\t{}",
            episode.episode_id,
            episode.start_generation,
            episode.end_generation,
            episode.episode_kind,
            episode.completion_posture,
            episode.resource_refs.join(","),
            episode.transition_ids.len()
        );
    }
    Ok(())
}

fn memory_episode_show(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let episode_id = named_arg(args, "--episode")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let (hierarchy, _) = derive_current_memory_hierarchy(&store, &case_id)?;
    let episode = hierarchy
        .episodes
        .iter()
        .find(|episode| {
            episode.episode_id == episode_id && episode_is_visible(episode, &participant_id)
        })
        .ok_or_else(|| "memory_episode_not_found_or_not_visible".to_string())?;
    if args.iter().any(|value| value == "--json") {
        emit_json(episode)
    } else {
        println!("episode_id: {}", episode.episode_id);
        println!("case_id: {}", episode.case_id);
        println!(
            "generation: {}..{}",
            episode.start_generation, episode.end_generation
        );
        println!("kind: {:?}", episode.episode_kind);
        println!("completion_posture: {:?}", episode.completion_posture);
        println!("participants: {}", episode.participant_refs.join(","));
        println!("resources: {}", episode.resource_refs.join(","));
        println!("operations: {}", episode.operation_refs.join(","));
        println!("effects: {}", episode.effect_refs.join(","));
        println!("unresolved: {}", episode.unresolved_refs.join(","));
        println!("transition_ids: {}", episode.transition_ids.join(","));
        println!("provenance_digest: {}", episode.provenance_digest);
        Ok(())
    }
}

fn memory_semantic_show(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let include_historical = args.iter().any(|value| value == "--include-historical");
    let store = LmdbRecordStore::open(record_store_path())?;
    let (hierarchy, _) = derive_current_memory_hierarchy(&store, &case_id)?;
    let assertions = hierarchy
        .assertions
        .iter()
        .filter(|assertion| assertion_is_visible(assertion, &participant_id))
        .filter(|assertion| {
            include_historical
                || hierarchy
                    .manifest
                    .active_semantic_ids
                    .contains(&assertion.assertion_id)
        })
        .collect::<Vec<_>>();
    if args.iter().any(|value| value == "--json") {
        return emit_json(&serde_json::json!({
            "schema": "yai.semantic_memory_view.v1",
            "case_id": case_id,
            "source_generation": hierarchy.manifest.source_generation,
            "hierarchy_id": hierarchy.manifest.hierarchy_id,
            "unresolved_consolidation_result_ids": hierarchy.unresolved_consolidation_result_ids,
            "background_assertion_count": hierarchy.manifest.background_semantic_ids.len(),
            "assertions": assertions
        }));
    }
    println!("case_id: {case_id}");
    println!("hierarchy_id: {}", hierarchy.manifest.hierarchy_id);
    println!("assertion_id\tsubject\tpredicate\tvalue\tepistemic_class\tlifecycle\tsupports\tcontradiction");
    for assertion in assertions {
        println!(
            "{}\t{:?}\t{}\t{:?}\t{:?}\t{:?}\t{}\t{}",
            assertion.assertion_id,
            assertion.subject,
            assertion.predicate,
            assertion.value,
            assertion.epistemic_class,
            assertion.lifecycle,
            assertion.support_refs.len(),
            assertion.contradiction_set_ref.as_deref().unwrap_or("none")
        );
    }
    println!(
        "unresolved_consolidation_results: {}",
        hierarchy.unresolved_consolidation_result_ids.join(",")
    );
    println!(
        "background_semantic_assertions: {}",
        hierarchy.manifest.background_semantic_ids.len()
    );
    Ok(())
}

fn memory_contradictions_show(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let (hierarchy, _) = derive_current_memory_hierarchy(&store, &case_id)?;
    let visible_ids = hierarchy
        .assertions
        .iter()
        .filter(|assertion| assertion_is_visible(assertion, &participant_id))
        .map(|assertion| assertion.assertion_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let contradictions = hierarchy
        .contradictions
        .iter()
        .filter(|set| {
            set.competing_assertion_ids
                .iter()
                .all(|id| visible_ids.contains(id.as_str()))
        })
        .collect::<Vec<_>>();
    if args.iter().any(|value| value == "--json") {
        return emit_json(&serde_json::json!({
            "schema": "yai.semantic_contradiction_list.v1",
            "case_id": case_id,
            "hierarchy_id": hierarchy.manifest.hierarchy_id,
            "contradictions": contradictions
        }));
    }
    println!("case_id: {case_id}");
    println!("contradiction_id\tsubject\tpredicate\tresolution\tassertions");
    for set in contradictions {
        println!(
            "{}\t{:?}\t{}\t{}\t{}",
            set.contradiction_id,
            set.subject,
            set.predicate,
            set.resolution_posture,
            set.competing_assertion_ids.join(",")
        );
    }
    Ok(())
}

fn memory_hierarchy_show(args: &[String], action: &str) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let before = store.list_case_transitions(&case_id)?.len();
    let (hierarchy, _) = derive_current_memory_hierarchy(&store, &case_id)?;
    let visible_episodes = hierarchy
        .episodes
        .iter()
        .filter(|episode| episode_is_visible(episode, &participant_id))
        .count();
    let visible_assertions = hierarchy
        .assertions
        .iter()
        .filter(|assertion| assertion_is_visible(assertion, &participant_id))
        .count();
    let after = store.list_case_transitions(&case_id)?.len();
    let result = serde_json::json!({
        "schema": "yai.memory_hierarchy_status.v1",
        "action": action,
        "case_id": case_id,
        "source_generation": hierarchy.manifest.source_generation,
        "hierarchy_id": hierarchy.manifest.hierarchy_id,
        "hierarchy_digest": hierarchy.manifest.hierarchy_digest,
        "operational_manifest_id": hierarchy.manifest.operational_manifest_id,
        "visible_episode_count": visible_episodes,
        "visible_semantic_assertion_count": visible_assertions,
        "background_semantic_assertion_count": hierarchy.manifest.background_semantic_ids.len(),
        "contradiction_count": hierarchy.contradictions.len(),
        "unresolved_consolidation_result_count": hierarchy.unresolved_consolidation_result_ids.len(),
        "persistent_cache": "absent_by_design_rebuilt_from_transition_history",
        "canonical_transition_mutated": before != after
    });
    if args.iter().any(|value| value == "--json") {
        emit_json(&result)
    } else {
        println!("memory_hierarchy_{action}: complete");
        println!("case_id: {case_id}");
        println!(
            "source_generation: {}",
            hierarchy.manifest.source_generation
        );
        println!("hierarchy_id: {}", hierarchy.manifest.hierarchy_id);
        println!("hierarchy_digest: {}", hierarchy.manifest.hierarchy_digest);
        println!("visible_episodes: {visible_episodes}");
        println!("visible_semantic_assertions: {visible_assertions}");
        println!(
            "background_semantic_assertions: {}",
            hierarchy.manifest.background_semantic_ids.len()
        );
        println!("contradictions: {}", hierarchy.contradictions.len());
        println!("persistent_cache: absent_by_design_rebuilt_from_transition_history");
        println!("canonical_transition_mutated: {}", yes_no(before != after));
        Ok(())
    }
}

fn memory_hierarchy_drop(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let before = store.list_case_transitions(&case_id)?.len();
    let (hierarchy, _) = derive_current_memory_hierarchy(&store, &case_id)?;
    let visible_source_exists = hierarchy
        .episodes
        .iter()
        .any(|episode| episode_is_visible(episode, &participant_id));
    let after = store.list_case_transitions(&case_id)?.len();
    let result = serde_json::json!({
        "schema": "yai.memory_hierarchy_drop_result.v1",
        "case_id": case_id,
        "dropped": false,
        "reason": "persistent_hierarchy_cache_absent_by_design",
        "visible_source_exists": visible_source_exists,
        "canonical_transitions_before": before,
        "canonical_transitions_after": after,
        "rebuild_requires_provider_invocation": false
    });
    if args.iter().any(|value| value == "--json") {
        emit_json(&result)
    } else {
        println!("memory_hierarchy_drop: absent_by_design");
        println!("case_id: {case_id}");
        println!("canonical_transitions_remaining: {after}");
        println!("rebuild_requires_provider_invocation: no");
        Ok(())
    }
}

fn consolidation_task(
    input: &MemoryConsolidationInput,
    hierarchy: &SemanticMemoryHierarchy,
    operational: &[OperationalMemoryEntry],
) -> Result<String, String> {
    let episodes = hierarchy
        .episodes
        .iter()
        .filter(|episode| input.episode_ids.contains(&episode.episode_id))
        .collect::<Vec<_>>();
    let representations = operational
        .iter()
        .filter(|entry| input.operational_memory_ids.contains(&entry.memory_id))
        .map(MemoryRepresentationDocument::from_operational_v2)
        .collect::<Result<Vec<_>, _>>()?;
    let semantic = hierarchy
        .assertions
        .iter()
        .filter(|assertion| {
            input
                .existing_semantic_assertion_ids
                .contains(&assertion.assertion_id)
        })
        .collect::<Vec<_>>();
    let packet = serde_json::json!({
        "consolidation_input": input,
        "episodes": episodes,
        "operational_representation_documents": representations,
        "existing_semantic_assertions": semantic
    });
    let encoded = serde_json::to_string_pretty(&packet)
        .map_err(|error| format!("memory_consolidation_input_encode_failed: {error}"))?;
    Ok(format!(
        "Perform bounded Case-memory consolidation. Return exactly one JSON object with schema {schema}. Do not return markdown, tools, operations, policy, grants, authority decisions, epistemic_class, confidence, grounded, observed, or authoritative fields. Every assertion must use only support_refs present in consolidation_input.allowed_support_refs. Open predicates must be lowercase dot-separated names. Preserve disagreement; do not select a winner.\n\nExact typed input:\n{encoded}",
        schema = CONSOLIDATION_CANDIDATE_SCHEMA
    ))
}

fn memory_consolidate(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let initial_store = LmdbRecordStore::open(record_store_path())?;
    let initial_state = security::authorize_case_read_if_scoped(&initial_store, &case_id)?;
    let logical_turn_id = format!(
        "memory-consolidation-{}-{}",
        initial_state.generation.saturating_add(1),
        participant_id.replace(':', "-")
    );
    let requirement =
        yai_core_engine::provider_governance::ProviderRequirement::memory_consolidation()?;
    let route = super::case_runtime::governed_provider_route_for_requirement(
        &case_id,
        &participant_id,
        &requirement,
        &logical_turn_id,
    )?;

    // Provider selection is itself canonical. Derive the exact consolidation
    // snapshot only after that transition so lineage can reproduce it.
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = security::authorize_case_read_if_scoped(&store, &case_id)?;
    let (hierarchy, memory) = derive_current_memory_hierarchy(&store, &case_id)?;
    let input = derive_consolidation_input(
        &case_id,
        state.generation,
        &participant_id,
        &hierarchy.episodes,
        &memory.entries,
        &hierarchy.assertions,
    )?;
    let task = consolidation_task(&input, &hierarchy, &memory.entries)?;
    let output_contract = InvocationOutputContract::MemoryConsolidation {
        schema: CONSOLIDATION_CANDIDATE_SCHEMA.to_string(),
        consolidation_input_id: input.input_id.clone(),
        maximum_assertions: input.maximum_assertion_count,
        maximum_support_refs: input.maximum_support_refs,
        normalizer_version: CONSOLIDATION_NORMALIZER_VERSION.to_string(),
    };
    let consolidation_options = super::provider::RuntimeInvocationOptions {
        max_resident_items: 8,
        max_semantic_units: CONSOLIDATION_SEMANTIC_UNIT_BUDGET,
        max_estimated_input_units: CONSOLIDATION_SEMANTIC_UNIT_BUDGET * 2,
        retrieval_limit: 8,
        previous_item_ids: Vec::new(),
        workflow_execution_id: None,
    };
    let invocation = super::provider::invoke_runtime_provider(
        &route.args,
        ProjectionPurpose::MemoryConsolidation,
        &task,
        output_contract,
        &consolidation_options,
    );
    let result = match invocation {
        Ok(result) => {
            if let Some(selection) = &route.selection {
                super::case_runtime::record_governed_provider_outcome(
                    &case_id,
                    selection,
                    None,
                    Some(result.request_bytes_written),
                )?;
            }
            result
        }
        Err(error) => {
            if let Some(selection) = &route.selection {
                super::case_runtime::record_governed_provider_outcome(
                    &case_id,
                    selection,
                    Some(&error),
                    None,
                )?;
            }
            return Err(error);
        }
    };
    let store = LmdbRecordStore::open(record_store_path())?;
    let (rebuilt, _) = derive_current_memory_hierarchy(&store, &case_id)?;
    let produced = rebuilt
        .assertions
        .iter()
        .filter(|assertion| assertion.origin.contains(&result.result_id))
        .map(|assertion| assertion.assertion_id.clone())
        .collect::<Vec<_>>();
    if produced.is_empty()
        && rebuilt
            .unresolved_consolidation_result_ids
            .contains(&result.result_id)
    {
        return Err(format!(
            "memory_consolidation_result_recorded_but_normalization_failed:{}",
            result.result_id
        ));
    }
    let output = serde_json::json!({
        "schema": "yai.memory_consolidation_result.v1",
        "case_id": case_id,
        "consolidation_input_id": input.input_id,
        "provider_selection_id": route.selection.as_ref().map(|selection| &selection.selection_id),
        "provider_invocation_id": result.invocation_id,
        "provider_result_id": result.result_id,
        "provider_id": result.provider_id,
        "model_id": result.model_id,
        "projection_id": result.projection_id,
        "context_frame_id": result.context_frame_id,
        "semantic_assertion_ids": produced,
        "hierarchy_id": rebuilt.manifest.hierarchy_id,
        "normalizer_version": CONSOLIDATION_NORMALIZER_VERSION,
        "provider_result_recorded_before_normalization": true,
        "rebuild_requires_reinference": false
    });
    if args.iter().any(|value| value == "--json") {
        emit_json(&output)
    } else {
        println!("memory_consolidation: normalized");
        println!("case_id: {case_id}");
        println!("consolidation_input_id: {}", input.input_id);
        println!("provider_invocation_id: {}", result.invocation_id);
        println!("provider_result_id: {}", result.result_id);
        println!("projection_id: {}", result.projection_id);
        println!("context_frame_id: {}", result.context_frame_id);
        println!("semantic_assertion_ids: {}", produced.join(","));
        println!("hierarchy_id: {}", rebuilt.manifest.hierarchy_id);
        println!("rebuild_requires_reinference: no");
        Ok(())
    }
}

pub(super) fn memory_case_command(operation_id: &str, args: &[String]) -> Result<(), String> {
    match operation_id {
        "yai.case.memory.show" => memory_case_show(args),
        "yai.case.memory.search" => memory_search(args),
        "yai.case.memory.index.status" => memory_index_status(args),
        "yai.case.memory.index.verify" => memory_index_verify(args),
        "yai.case.memory.index.build" => memory_index_build_or_rebuild(args, false),
        "yai.case.memory.index.rebuild" => memory_index_build_or_rebuild(args, true),
        "yai.case.memory.index.drop" => memory_index_drop(args),
        "yai.case.memory.retrieval.show" => memory_retrieval_show(args),
        "yai.case.memory.episodes.show" => memory_episodes_show(args),
        "yai.case.memory.episode.show" => memory_episode_show(args),
        "yai.case.memory.semantic.show" => memory_semantic_show(args),
        "yai.case.memory.contradictions" => memory_contradictions_show(args),
        "yai.case.memory.hierarchy.show" => memory_hierarchy_show(args, "show"),
        "yai.case.memory.hierarchy.rebuild" => memory_hierarchy_show(args, "rebuild"),
        "yai.case.memory.hierarchy.drop" => memory_hierarchy_drop(args),
        "yai.case.memory.consolidate" => memory_consolidate(args),
        _ => Err(format!("unsupported Case memory operation: {operation_id}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yai_core_engine::provider_governance::{ProviderAdapterKind, ProviderTargetInput};

    fn encoder_snapshot() -> QualifiedMemoryEncoder {
        QualifiedMemoryEncoder {
            target: ProviderTarget::from_input(ProviderTargetInput {
                tenant_id: "tenant:test".to_string(),
                provider_key: "local-encoder".to_string(),
                adapter: ProviderAdapterKind::OpenAiCompatible,
                endpoint: "http://127.0.0.1:12345/v1".to_string(),
                model_id: "encoder:model".to_string(),
                credential_ref: "env:YAI_TEST_ENCODER_KEY".to_string(),
                locality: ProviderLocality::Loopback,
                extension_adapter_id: None,
                created_by_principal_id: "principal:test".to_string(),
                created_at_unix_ms: 1,
            })
            .unwrap(),
            credential: Some("test-secret-a".to_string()),
            qualification_id: "provider-qualification:test".to_string(),
            credential_revision: 1,
        }
    }

    #[test]
    fn embedding_response_requires_exact_count_dimension_model_and_finite_values() {
        let valid = br#"{"model":"encoder:model","data":[{"index":1,"embedding":[0.0,1.0]},{"index":0,"embedding":[1.0,0.0]}]}"#;
        assert_eq!(
            parse_embedding_response(valid, "encoder:model", 2, 2).unwrap(),
            vec![vec![1.0, 0.0], vec![0.0, 1.0]]
        );
        assert_eq!(
            parse_embedding_response(valid, "encoder:other", 2, 2).unwrap_err(),
            "memory_encoder_response_model_mismatch"
        );
        let missing_model = br#"{"data":[{"index":0,"embedding":[1.0,0.0]}]}"#;
        assert_eq!(
            parse_embedding_response(missing_model, "encoder:model", 1, 2).unwrap_err(),
            "memory_encoder_response_model_mismatch"
        );
        assert_eq!(
            parse_embedding_response(valid, "encoder:model", 2, 3).unwrap_err(),
            "memory_encoder_response_dimension_mismatch"
        );
        let duplicate = br#"{"model":"encoder:model","data":[{"index":0,"embedding":[1.0,0.0]},{"index":0,"embedding":[0.0,1.0]}]}"#;
        assert_eq!(
            parse_embedding_response(duplicate, "encoder:model", 2, 2).unwrap_err(),
            "memory_encoder_response_index_invalid"
        );
        let overflow =
            br#"{"model":"encoder:model","data":[{"index":0,"embedding":[1e1000,0.0]}]}"#;
        assert!(parse_embedding_response(overflow, "encoder:model", 1, 2).is_err());
    }

    #[test]
    fn h19_s10_case_generation_change_aborts_build_or_publication() {
        assert!(require_case_generation(9, 9, false).is_ok());
        assert_eq!(
            require_case_generation(9, 10, false).unwrap_err(),
            "memory_index_case_generation_changed_during_build"
        );
        assert_eq!(
            require_case_generation(9, 10, true).unwrap_err(),
            "memory_index_case_generation_changed_during_publication"
        );
    }

    #[test]
    fn h19_s11_trust_revocation_stops_encoder_recheck() {
        let admitted = encoder_snapshot();
        assert_eq!(
            require_memory_encoder_snapshot(
                &admitted,
                Err("memory_encoder_trust_not_approved".to_string())
            )
            .unwrap_err(),
            "memory_encoder_trust_not_approved"
        );
    }

    #[test]
    fn h19_s12_qualification_invalidation_stops_encoder_recheck() {
        let admitted = encoder_snapshot();
        assert_eq!(
            require_memory_encoder_snapshot(
                &admitted,
                Err("memory_encoder_qualification_stale".to_string())
            )
            .unwrap_err(),
            "memory_encoder_qualification_stale"
        );
    }

    #[test]
    fn h19_s13_credential_rotation_cannot_mix_one_build() {
        let admitted = encoder_snapshot();
        let mut rotated = admitted.clone();
        rotated.credential_revision += 1;
        rotated.credential = Some("test-secret-b".to_string());
        assert_eq!(
            require_memory_encoder_snapshot(&admitted, Ok(rotated)).unwrap_err(),
            "memory_encoder_governance_changed_during_build"
        );
    }

    #[test]
    fn h19_s14_embedding_response_adversarial_matrix_fails_closed() {
        let cases: &[(&[u8], &str)] = &[
            (
                br#"{"model":"wrong","data":[{"index":0,"embedding":[1.0,0.0]}]}"#,
                "memory_encoder_response_model_mismatch",
            ),
            (
                br#"{"model":"encoder:model","data":[]}"#,
                "memory_encoder_response_count_mismatch",
            ),
            (
                br#"{"model":"encoder:model","data":[{"embedding":[1.0,0.0]}]}"#,
                "memory_encoder_response_index_missing",
            ),
            (
                br#"{"model":"encoder:model","data":[{"index":0,"embedding":[]}]}"#,
                "memory_encoder_response_dimension_mismatch",
            ),
            (
                br#"{"model":"encoder:model","data":[{"index":0,"embedding":[0.0,0.0]}]}"#,
                "memory_encoder_response_zero_vector",
            ),
            (
                br#"{"model":"encoder:model","data":[{"index":1,"embedding":[1.0,0.0]}]}"#,
                "memory_encoder_response_index_invalid",
            ),
            (
                br#"{"model":"encoder:model","data":[{"index":0,"embedding":[1.0,0.0]},{"index":1,"embedding":[0.0,1.0]}]}"#,
                "memory_encoder_response_count_mismatch",
            ),
            (
                br#"{"model":"encoder:model","data":"invalid"}"#,
                "memory_encoder_response_data_missing",
            ),
        ];
        for (body, expected) in cases {
            assert_eq!(
                parse_embedding_response(body, "encoder:model", 1, 2).unwrap_err(),
                *expected
            );
        }
        assert!(parse_embedding_response(b"{", "encoder:model", 1, 2).is_err());
    }
}
