//! Operator inspection and rebuild commands for derived operational memory.

use super::*;

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
    let state = store
        .get_case_state(case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
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
    let state = store
        .get_case_state(&case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
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
