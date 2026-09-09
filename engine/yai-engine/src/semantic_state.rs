//! Read-only semantic composition and bounded working-state compilation.
//! No canonical store, provider runtime state or semantic mutation authority.
use self::{DerivedCandidates as DerivedProjectionInput, SemanticScope as ProjectionRequest};
use crate::context::{
    ProjectionBounds, DEFAULT_MAX_CLAIM_CHARS, DEFAULT_MAX_INTERACTION_TURNS,
    DEFAULT_MAX_PROJECTION_ITEMS, DEFAULT_MAX_PROVIDER_CLAIMS,
};
use crate::effect::{DecisionOutcome, EffectOutcome};
use crate::transition::{
    CaseLifecycle, CaseState, EffectLifecycle, ResourceKind, ReviewRequirement, ReviewResolution,
    Transition, TransitionPayload,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SEMANTIC_STATE_SCHEMA: &str = "yai.semantic_state.v1";
pub const WORKING_STATE_SCHEMA: &str = "yai.semantic_working_state.v1";
pub const SEMANTIC_DELTA_SCHEMA: &str = "yai.semantic_delta.v1";
pub const STATE_COMPILER_VERSION: &str = "yai.state_compiler.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticScope {
    pub participant_id: String,
    pub purpose: SemanticPurpose,
    pub consumer: String,
    pub view_kind: String,
    pub max_items: usize,
    pub max_provider_claims: usize,
    pub max_interaction_turns: usize,
}

impl SemanticScope {
    pub fn model(participant_id: impl Into<String>, purpose: SemanticPurpose) -> Self {
        Self {
            participant_id: participant_id.into(),
            purpose,
            consumer: "model".to_string(),
            view_kind: "model_context".to_string(),
            max_items: DEFAULT_MAX_PROJECTION_ITEMS,
            max_provider_claims: DEFAULT_MAX_PROVIDER_CLAIMS,
            max_interaction_turns: DEFAULT_MAX_INTERACTION_TURNS,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DerivedMemoryInput {
    pub memory_ref: String,
    pub semantic_kind: String,
    pub memory_posture: String,
    pub description: String,
    pub lifecycle: String,
    pub score: i64,
    pub ranking_reasons: Vec<String>,
    pub transition_refs: Vec<String>,
    pub observation_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
    pub derived_memory_refs: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DerivedCandidates {
    pub graph_available: bool,
    pub memory_available: bool,
    pub memory: Vec<DerivedMemoryInput>,
    pub retrieval_id: Option<String>,
    pub retrieval_candidates: usize,
    pub retrieval_omitted: usize,
}

/// A read-only composition of qualified current owners. It is deliberately not
/// deserializable or persistent: only replay-qualified sources construct S.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticState {
    source_id: String,
    state: CaseState,
    history: Vec<Transition>,
}

/// Explicit semantic selection, not a target/provider routing request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompilationRequest {
    pub scope: ProjectionRequest,
    pub intent: String,
    pub output_contract_id: String,
    pub max_semantic_units: usize,
    pub max_derived_items: usize,
    pub resource_refs: Vec<String>,
    /// Exact required entries or provenance refs. Missing OR unauthorized fails
    /// identically; relevance cannot grant access or silently lose a requirement.
    pub required_refs: Vec<String>,
    pub previous_item_ids: Vec<String>,
    /// Existing governed selection may qualify the model view for its exact
    /// Participant. This is proof, not an ambient view override or a router.
    pub view_selection_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkingStateBounds {
    pub selected_items: usize,
    pub selected_semantic_units: usize,
    pub visible_candidates: usize,
    pub omitted_items: usize,
    pub omitted_by_locality: usize,
    pub omitted_by_budget: usize,
    /// Aggregate explanations contain no undisclosed IDs or unbounded history.
    pub locality_reasons: BTreeMap<String, usize>,
}

/// Disposable execution state; values retain their epistemic class and exact
/// source provenance. Diagnostics are not sent as historical model context.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticWorkingState {
    schema: String,
    compiler: String,
    working_state_id: String,
    source_id: String,
    case_id: String,
    case_generation: u64,
    participant_id: String,
    request: CompilationRequest,
    entries: Vec<SemanticEntry>,
    bounds: WorkingStateBounds,
    decisions: Vec<crate::residency::ResidencyDecision>,
}

fn identity<T: Serialize>(value: &T) -> Result<String, String> {
    Ok(crate::effect::digest_bytes(
        &serde_json::to_vec(value).map_err(|e| format!("semantic_state_identity:{e}"))?,
    ))
}

impl SemanticState {
    pub fn compose(state: &CaseState, history: &[Transition]) -> Result<Self, String> {
        if crate::transition::replay_case(&state.case_id, history)? != *state {
            return Err("semantic_state_materialization_replay_mismatch".into());
        }
        Ok(Self {
            source_id: format!(
                "semantic-state:{}",
                identity(&(SEMANTIC_STATE_SCHEMA, state, history))?
            ),
            state: state.clone(),
            history: history.to_vec(),
        })
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }
    pub fn case_id(&self) -> &str {
        &self.state.case_id
    }
    pub fn generation(&self) -> u64 {
        self.state.generation
    }

    fn candidates(&self, request: &CompilationRequest) -> Result<SemanticCandidates, String> {
        if request.intent.trim().is_empty()
            || request.output_contract_id.is_empty()
            || request.scope.max_items == 0
            || request.max_semantic_units == 0
        {
            return Err("semantic_compilation_request_invalid".into());
        }
        let mut scope = request.scope.clone();
        // Candidate discovery is not working-state selection. Exact addressing
        // must not be cut off by a recent-history window before qualification.
        scope.max_items = usize::MAX;
        scope.max_interaction_turns = usize::MAX;
        scope.max_provider_claims = usize::MAX;
        let mut qualified = self.state.clone();
        if let Some(id) = &request.view_selection_id {
            if scope.consumer != "model"
                || scope.view_kind != "model_context"
                || !qualified.provider_selections.iter().any(|s| {
                    &s.selection_id == id
                        && s.participant_id == scope.participant_id
                        && s.case_id == qualified.case_id
                        && qualified.tenant_id.as_deref() == Some(s.tenant_id.as_str())
                        && qualified.provider_binding.as_ref().is_some_and(|binding| {
                            binding.binding_id == s.binding_id
                                && binding.participant_id == scope.participant_id
                                && binding.ordered_target_ids.contains(&s.selected_target_id)
                        })
                })
            {
                return Err("semantic_view_selection_not_qualified".into());
            }
            let participant = qualified
                .participants
                .iter_mut()
                .find(|p| p.participant_id == scope.participant_id)
                .ok_or("semantic_participant_not_bound")?;
            let view = crate::transition::AdmittedView {
                consumer: scope.consumer.clone(),
                view_kind: scope.view_kind.clone(),
            };
            if !participant.admitted_views.contains(&view) {
                participant.admitted_views.push(view);
            }
        }
        let derived = if scope.purpose == SemanticPurpose::MemoryConsolidation {
            DerivedProjectionInput::default()
        } else {
            canonical_memory(&qualified, &self.history, &scope)?
        };
        let mut candidates = collect_candidates(&qualified, &self.history, &scope, &derived)?;
        if scope.purpose != SemanticPurpose::MemoryConsolidation {
            for binding in &self.state.policy_bindings {
                candidates.entries.push(SemanticEntry {
                    entry_id: format!("policy:{}", binding.binding_id),
                    posture: AuthorityPosture::ControlState,
                    value: SemanticValue::PolicyBinding {
                        binding: binding.clone(),
                    },
                    provenance: vec![SemanticProvenance {
                        kind: ProvenanceKind::CaseStateGeneration,
                        source_ref: format!("{}@{}", self.state.case_id, self.state.generation),
                    }],
                });
            }
        }
        // No historic result can preserve removed/rebound resource disclosure.
        // Also cover legacy effect entries whose attachment is now absent.
        candidates.entries.retain(|entry| match &entry.value {
            SemanticValue::ResourceConsequence { effect_id, .. } => self
                .state
                .effects
                .iter()
                .find(|e| &e.effect_id == effect_id)
                .is_some_and(|effect| {
                    self.state.resources.iter().any(|r| {
                        r.attachment_id == effect.resource_attachment_id
                            && resource_visible(r, &scope.participant_id)
                    })
                }),
            _ => true,
        });
        let ids = candidates
            .entries
            .iter()
            .map(|e| &e.entry_id)
            .collect::<BTreeSet<_>>();
        if ids.len() != candidates.entries.len() {
            return Err("semantic_source_duplicate_identity".into());
        }
        Ok(candidates)
    }

    pub fn compile(&self, request: &CompilationRequest) -> Result<SemanticWorkingState, String> {
        let candidates = self.candidates(request)?;
        let visible_candidates = candidates.entries.len();
        let mut mandatory_refs = BTreeSet::new();
        for required in &request.required_refs {
            let matches = candidates
                .entries
                .iter()
                .filter(|e| entry_matches(e, required))
                .collect::<Vec<_>>();
            if matches.is_empty() {
                return Err("semantic_required_source_unavailable".into());
            }
            mandatory_refs.extend(matches.into_iter().map(|e| e.entry_id.clone()));
        }
        // Qualify first; exact task references then locality; the shared semantic
        // budget selector runs once. No provider identity participates here.
        let mut remaining_turns = request.scope.max_interaction_turns;
        let mut remaining_claims = request.scope.max_provider_claims;
        let mut remaining_observations = 12usize;
        let mut remaining_content = 8usize;
        let mut remaining_effects = 4usize;
        let mut selected_candidates = Vec::new();
        let mut locality_reasons = BTreeMap::new();
        for entry in candidates.entries.into_iter().rev() {
            let explicit = mandatory_refs.contains(&entry.entry_id);
            let retain = match &entry.value {
                SemanticValue::ResourceObservation { .. } => {
                    let keep = explicit || remaining_observations > 0;
                    remaining_observations = remaining_observations.saturating_sub(1);
                    keep
                }
                SemanticValue::CaseContent { .. } => {
                    let keep = explicit || remaining_content > 0;
                    remaining_content = remaining_content.saturating_sub(1);
                    keep
                }
                SemanticValue::ResourceConsequence {
                    lifecycle: EffectLifecycle::Finalized,
                    ..
                } => {
                    let keep = explicit || remaining_effects > 0;
                    remaining_effects = remaining_effects.saturating_sub(1);
                    keep
                }
                SemanticValue::ConversationTurn { .. } | SemanticValue::InteractionTurn { .. } => {
                    let retain = explicit || remaining_turns > 0;
                    remaining_turns = remaining_turns.saturating_sub(1);
                    retain
                }
                SemanticValue::ProviderClaim { .. } => {
                    let retain = explicit || remaining_claims > 0;
                    remaining_claims = remaining_claims.saturating_sub(1);
                    retain
                }
                _ => true,
            };
            if retain {
                selected_candidates.push(entry);
            } else {
                let reason = match &entry.value {
                    SemanticValue::ResourceObservation { .. } => "recent_observation_window",
                    SemanticValue::CaseContent { .. } => "recent_content_window",
                    SemanticValue::ResourceConsequence { .. } => "recent_finalized_effect_window",
                    SemanticValue::ProviderClaim { .. } => "recent_claim_window",
                    _ => "recent_interaction_window",
                };
                *locality_reasons.entry(reason.to_string()).or_insert(0) += 1;
            }
        }
        selected_candidates.reverse();
        // Derived sources are reconstructed before ranking. The index is not a
        // source of text or authority. Explicit refs outrank lexical relevance;
        // ties use exact identity, never provider names or wall-clock salience.
        let terms = request
            .intent
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() >= 3)
            .map(str::to_lowercase)
            .collect::<BTreeSet<_>>();
        let focused_transitions = self
            .history
            .iter()
            .filter(|t| {
                t.scope.as_ref().is_some_and(|scope| {
                    scope
                        .resource_refs
                        .iter()
                        .any(|id| request.resource_refs.contains(id))
                })
            })
            .map(|t| t.transition_id.as_str())
            .collect::<BTreeSet<_>>();
        let mut memory_ranks = selected_candidates
            .iter()
            .filter_map(|entry| {
                if let SemanticValue::DerivedMemory { description, .. } = &entry.value {
                    let description = description.to_lowercase();
                    let resource_match = entry.provenance.iter().any(|p| {
                        p.kind == ProvenanceKind::Transition
                            && focused_transitions.contains(p.source_ref.as_str())
                    });
                    Some((
                        mandatory_refs.contains(&entry.entry_id),
                        terms
                            .iter()
                            .filter(|t| description.contains(t.as_str()))
                            .count()
                            + usize::from(resource_match),
                        entry.entry_id.clone(),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        memory_ranks.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
        let allowed_memory = memory_ranks
            .iter()
            .enumerate()
            .filter(|(i, r)| r.0 || (r.1 > 0 && *i < request.max_derived_items))
            .map(|(_, r)| &r.2)
            .collect::<BTreeSet<_>>();
        for (required, rank, id) in &memory_ranks {
            if !allowed_memory.contains(id) && !required {
                let reason = if *rank == 0 {
                    "no_current_intent_match"
                } else {
                    "derived_candidate_budget"
                };
                *locality_reasons.entry(reason.to_string()).or_insert(0) += 1;
            }
        }
        selected_candidates.retain(|e| {
            !matches!(e.value, SemanticValue::DerivedMemory { .. })
                || allowed_memory.contains(&e.entry_id)
        });
        let omitted_by_locality = visible_candidates - selected_candidates.len();
        let selection_request = crate::residency::ResidencyRequest {
            case_id: self.state.case_id.clone(),
            case_generation: self.state.generation,
            participant_id: request.scope.participant_id.clone(),
            purpose: request.scope.purpose.clone(),
            provider_id: String::new(),
            model_id: String::new(),
            max_items: request.scope.max_items,
            max_semantic_units: request.max_semantic_units,
            resource_refs: request.resource_refs.clone(),
            previous_item_ids: request.previous_item_ids.clone(),
        };
        let selection = crate::residency::select_entries_required(
            &selected_candidates,
            &selection_request,
            &mandatory_refs,
        )?;
        let selected = selection.selected_item_ids.iter().collect::<BTreeSet<_>>();
        let omitted_by_budget = selected_candidates.len() - selected.len();
        let entries = selected_candidates
            .into_iter()
            .filter(|e| selected.contains(&e.entry_id))
            .collect::<Vec<_>>();
        let mut output = SemanticWorkingState {
            schema: WORKING_STATE_SCHEMA.into(),
            compiler: STATE_COMPILER_VERSION.into(),
            working_state_id: String::new(),
            source_id: self.source_id.clone(),
            case_id: self.state.case_id.clone(),
            case_generation: self.state.generation,
            participant_id: request.scope.participant_id.clone(),
            request: request.clone(),
            bounds: WorkingStateBounds {
                selected_items: entries.len(),
                selected_semantic_units: selection.selected_semantic_units,
                visible_candidates,
                omitted_items: omitted_by_locality + omitted_by_budget,
                omitted_by_locality,
                omitted_by_budget,
                locality_reasons,
            },
            entries,
            decisions: selection.decisions,
        };
        output.working_state_id = format!("working-state:{}", identity(&output)?);
        Ok(output)
    }
}

fn entry_matches(entry: &SemanticEntry, reference: &str) -> bool {
    entry.entry_id == reference
        || entry.provenance.iter().any(|p| p.source_ref == reference)
        || matches!(&entry.value, SemanticValue::DerivedMemory { memory_ref, .. } if memory_ref == reference)
}

fn resource_visible(
    resource: &crate::transition::ResourceAttachmentState,
    participant: &str,
) -> bool {
    resource
        .access
        .as_ref()
        .is_none_or(|access| access.participant_ids.iter().any(|id| id == participant))
}

impl SemanticWorkingState {
    pub fn id(&self) -> &str {
        &self.working_state_id
    }
    pub fn case_id(&self) -> &str {
        &self.case_id
    }
    pub fn generation(&self) -> u64 {
        self.case_generation
    }
    pub fn entries(&self) -> &[SemanticEntry] {
        &self.entries
    }
    pub fn request(&self) -> &CompilationRequest {
        &self.request
    }
    pub fn bounds(&self) -> &WorkingStateBounds {
        &self.bounds
    }
    pub fn validate_current(
        &self,
        source: &SemanticState,
        request: &CompilationRequest,
    ) -> Result<(), String> {
        if self.schema != WORKING_STATE_SCHEMA
            || self.compiler != STATE_COMPILER_VERSION
            || self.source_id != source.source_id
            || self.request != *request
        {
            return Err("stale_semantic_working_state".into());
        }
        if source.compile(request)? != *self {
            return Err("semantic_working_state_integrity_mismatch".into());
        }
        Ok(())
    }

    pub fn lower_context(
        &self,
        source: &SemanticState,
        request: &CompilationRequest,
    ) -> Result<crate::context::Projection, String> {
        self.validate_current(source, request)?;
        let mut projection = crate::context::lower_candidates(SemanticCandidates {
            case_id: self.case_id.clone(),
            case_generation: self.case_generation,
            participant_id: self.participant_id.clone(),
            purpose: request.scope.purpose.clone(),
            visibility: SemanticVisibility {
                consumer: request.scope.consumer.clone(),
                view_kind: request.scope.view_kind.clone(),
            },
            entries: self.entries.clone(),
            bounds: ProjectionBounds {
                max_items: request.scope.max_items,
                selected_items: self.entries.len(),
                omitted_items: self.bounds.omitted_items,
                history_transitions_considered: source.history.len(),
                graph_available: false,
                memory_available: true,
                retrieval_id: None,
                retrieval_candidates: 0,
                retrieval_selected: self
                    .entries
                    .iter()
                    .filter(|e| matches!(e.value, SemanticValue::DerivedMemory { .. }))
                    .count(),
                retrieval_omitted: 0,
                residency_plan_id: None,
                semantic_unit_budget: Some(request.max_semantic_units),
                selected_semantic_units: Some(self.bounds.selected_semantic_units),
                working_state_id: Some(self.id().to_string()),
            },
        })?;
        crate::context::refresh_projection_identity(&mut projection)?;
        Ok(projection)
    }

    /// Compatibility inspection only, never a second selector. Provider fields
    /// label the realization consumer, not W's identity or selection algorithm.
    pub fn residency_report(
        &self,
        projection: &crate::context::Projection,
        provider_id: String,
        model_id: String,
    ) -> Result<crate::residency::ResidencyPlan, String> {
        let mut report = crate::residency::ResidencyPlan {
            schema: crate::residency::RESIDENCY_PLAN_SCHEMA.into(),
            plan_id: String::new(),
            request: crate::residency::ResidencyRequest {
                case_id: self.case_id.clone(),
                case_generation: self.case_generation,
                participant_id: self.participant_id.clone(),
                purpose: self.request.scope.purpose.clone(),
                provider_id,
                model_id,
                max_items: self.request.scope.max_items,
                max_semantic_units: self.request.max_semantic_units,
                resource_refs: self.request.resource_refs.clone(),
                previous_item_ids: self.request.previous_item_ids.clone(),
            },
            source_projection_id: projection.projection_id.clone(),
            source_item_count: self.bounds.visible_candidates,
            source_semantic_units: self.decisions.iter().map(|d| d.semantic_units).sum(),
            selected_item_ids: self.entries.iter().map(|e| e.entry_id.clone()).collect(),
            selected_semantic_units: self.bounds.selected_semantic_units,
            omitted_item_count: self.bounds.omitted_items,
            decisions: self.decisions.clone(),
        };
        // A consumer-labelled report has its own identity; two model labels
        // must not overwrite the same derived artifact even though W is equal.
        report.plan_id = format!("residency:{}", identity(&report)?);
        report.validate()?;
        Ok(report)
    }
}

/// Derived semantic differences, not commands to mutate a Case. A scope change
/// can remove visibility; no removed payload is exported by this delta.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SemanticChange {
    Added {
        entry_id: String,
        destination_digest: String,
    },
    Replaced {
        entry_id: String,
        source_digest: String,
        destination_digest: String,
    },
    Removed {
        entry_id: String,
        source_digest: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticDelta {
    pub schema: String,
    pub delta_id: String,
    pub source_id: String,
    pub destination_id: String,
    pub source_generation: u64,
    pub destination_generation: u64,
    pub request_id: String,
    pub changes: Vec<SemanticChange>,
}

pub fn derive_delta(
    before: &SemanticState,
    after: &SemanticState,
    request: &CompilationRequest,
) -> Result<SemanticDelta, String> {
    if before.case_id() != after.case_id()
        || before.generation() >= after.generation()
        || !after.history.starts_with(&before.history)
    {
        return Err("semantic_delta_history_not_forward_extension".into());
    }
    let old = before
        .candidates(request)?
        .entries
        .into_iter()
        .map(|e| (e.entry_id.clone(), e))
        .collect::<BTreeMap<_, _>>();
    let new = after
        .candidates(request)?
        .entries
        .into_iter()
        .map(|e| (e.entry_id.clone(), e))
        .collect::<BTreeMap<_, _>>();
    let mut changes = Vec::new();
    for (id, entry) in &old {
        match new.get(id) {
            None => changes.push(SemanticChange::Removed {
                entry_id: id.clone(),
                source_digest: identity(entry)?,
            }),
            Some(next) if next != entry => changes.push(SemanticChange::Replaced {
                entry_id: id.clone(),
                source_digest: identity(entry)?,
                destination_digest: identity(next)?,
            }),
            _ => {}
        }
    }
    for (id, entry) in &new {
        if !old.contains_key(id) {
            changes.push(SemanticChange::Added {
                entry_id: id.clone(),
                destination_digest: identity(entry)?,
            });
        }
    }
    let mut delta = SemanticDelta {
        schema: SEMANTIC_DELTA_SCHEMA.into(),
        delta_id: String::new(),
        source_id: before.source_id.clone(),
        destination_id: after.source_id.clone(),
        source_generation: before.generation(),
        destination_generation: after.generation(),
        request_id: identity(request)?,
        changes,
    };
    delta.delta_id = format!("semantic-delta:{}", identity(&delta)?);
    Ok(delta)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DeltaCompilationMode {
    FullRecompilation,
}

/// Correctness-first incremental entrypoint: requalify exact delta and sources,
/// then explicitly fall back to full compilation. No incremental speed claim;
/// omitted candidates can become necessary after replacements/removals.
pub fn compile_delta(
    previous: &SemanticWorkingState,
    before: &SemanticState,
    after: &SemanticState,
    request: &CompilationRequest,
    delta: &SemanticDelta,
) -> Result<(SemanticWorkingState, DeltaCompilationMode), String> {
    previous.validate_current(before, request)?;
    if derive_delta(before, after, request)? != *delta {
        return Err("stale_or_invalid_semantic_delta".into());
    }
    Ok((
        after.compile(request)?,
        DeltaCompilationMode::FullRecompilation,
    ))
}

fn canonical_memory(
    state: &CaseState,
    history: &[Transition],
    scope: &ProjectionRequest,
) -> Result<DerivedProjectionInput, String> {
    use crate::memory_index::RetrievedMemoryFamily;
    let memory = crate::memory::derive_operational_memory(&state.case_id, history)?;
    let hierarchy = crate::memory_hierarchy::build_memory_hierarchy(state, history, &memory)?;
    let retrieved = crate::memory::retrieve_operational_memory(
        state,
        &memory.entries,
        crate::memory::RetrievalQualification {
            case_id: state.case_id.clone(),
            case_generation: state.generation,
            participant_id: scope.participant_id.clone(),
            consumer: scope.consumer.clone(),
            view_kind: scope.view_kind.clone(),
            purpose: scope.purpose.clone(),
            resource_refs: Vec::new(),
            semantic_kinds: Vec::new(),
            causal_refs: Vec::new(),
            max_results: usize::MAX,
            include_superseded: false,
        },
    )?;
    let mut families = retrieved
        .selected
        .into_iter()
        .filter(|item| {
            item.memory.value.resource_refs().iter().all(|id| {
                state
                    .resources
                    .iter()
                    .any(|r| &r.attachment_id == id && resource_visible(r, &scope.participant_id))
            })
        })
        .map(|item| RetrievedMemoryFamily::Operational(item.memory))
        .collect::<Vec<_>>();
    families.extend(
        hierarchy
            .episodes
            .into_iter()
            .filter(|e| {
                crate::memory_hierarchy::episode_is_visible(e, &scope.participant_id)
                    && e.resource_refs.iter().all(|id| {
                        state.resources.iter().any(|r| {
                            &r.attachment_id == id && resource_visible(r, &scope.participant_id)
                        })
                    })
            })
            .map(RetrievedMemoryFamily::Episodic),
    );
    // A visible derived assertion cannot launder currently invisible support.
    // Compute a finite closure over already qualified canonical leaves; absent,
    // inactive or unresolvable support fails closed rather than trusting a cache.
    let mut support = BTreeSet::new();
    for family in &families {
        support.insert(family.source_id().to_string());
        match family {
            RetrievedMemoryFamily::Operational(m) => {
                support.extend(m.provenance.transition_ids.iter().cloned());
            }
            RetrievedMemoryFamily::Episodic(e) => {
                support.extend(e.transition_ids.iter().cloned());
            }
            _ => {}
        }
    }
    let mut pending = hierarchy
        .assertions
        .into_iter()
        .filter(|a| {
            crate::memory_hierarchy::assertion_is_visible(a, &scope.participant_id)
                && a.lifecycle == crate::memory_hierarchy::SemanticLifecycle::Active
                && match &a.subject {
                    crate::memory_hierarchy::SemanticSubject::ResourceAttachment(id) => {
                        state.resources.iter().any(|r| {
                            &r.attachment_id == id && resource_visible(r, &scope.participant_id)
                        })
                    }
                    _ => true,
                }
        })
        .collect::<Vec<_>>();
    loop {
        let mut next = Vec::new();
        let mut advanced = false;
        for assertion in pending {
            if assertion
                .support_refs
                .iter()
                .all(|s| support.contains(s.id()))
            {
                support.insert(assertion.assertion_id.clone());
                families.push(RetrievedMemoryFamily::Semantic(assertion));
                advanced = true;
            } else {
                next.push(assertion);
            }
        }
        if !advanced {
            break;
        }
        pending = next;
    }
    families.sort_by(|a, b| a.source_id().cmp(b.source_id()));
    let entries = families
        .into_iter()
        .map(|family| {
            let (transition_refs, observation_refs, receipt_refs, derived_memory_refs) =
                match &family {
                    RetrievedMemoryFamily::Operational(m) => (
                        m.provenance.transition_ids.clone(),
                        m.provenance.observation_ids.clone(),
                        m.provenance.effect_receipt_ids.clone(),
                        Vec::new(),
                    ),
                    RetrievedMemoryFamily::Episodic(e) => {
                        (e.transition_ids.clone(), Vec::new(), Vec::new(), Vec::new())
                    }
                    RetrievedMemoryFamily::Semantic(a) => (
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        a.support_refs.iter().map(|s| s.id().to_string()).collect(),
                    ),
                };
            crate::context::DerivedMemoryInput {
                memory_ref: family.source_id().to_string(),
                semantic_kind: match &family {
                    RetrievedMemoryFamily::Operational(m) => m.semantic_kind.as_str(),
                    _ => family.family(),
                }
                .to_string(),
                memory_posture: match &family {
                    RetrievedMemoryFamily::Operational(memory) => memory.posture.as_str(),
                    _ => family.epistemic_class(),
                }
                .to_string(),
                description: family.description(),
                lifecycle: family.lifecycle(),
                score: 0,
                ranking_reasons: vec!["canonical_reconstruction_no_index_dependency".into()],
                transition_refs,
                observation_refs,
                receipt_refs,
                derived_memory_refs,
            }
        })
        .collect::<Vec<_>>();
    Ok(DerivedProjectionInput {
        memory_available: true,
        retrieval_candidates: entries.len(),
        memory: entries,
        ..Default::default()
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPurpose {
    Conversation,
    FilesystemWriteProposal,
    ProcessSignalProposal,
    WorkflowPlanPatchProposal,
    EffectConsequence,
    MemoryConsolidation,
    Inspection,
}

impl SemanticPurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Conversation => "conversation",
            Self::FilesystemWriteProposal => "filesystem_write_proposal",
            Self::ProcessSignalProposal => "process_signal_proposal",
            Self::WorkflowPlanPatchProposal => "workflow_plan_patch_proposal",
            Self::EffectConsequence => "effect_consequence",
            Self::MemoryConsolidation => "memory_consolidation",
            Self::Inspection => "inspection",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticVisibility {
    pub consumer: String,
    pub view_kind: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityPosture {
    CommittedApplicationContent,
    CommittedOperationalFact,
    ObservedResourceState,
    ControlState,
    DerivedMemory,
    ProviderClaim,
    Unresolved,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceKind {
    ContentObject,
    Transition,
    Observation,
    EffectReceipt,
    CaseStateGeneration,
    DerivedMemory,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticProvenance {
    pub kind: ProvenanceKind,
    pub source_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SemanticValue {
    PolicyBinding {
        binding: crate::case_policy::CasePolicyBinding,
    },
    CaseLifecycle {
        lifecycle: CaseLifecycle,
    },
    TenantSecurityDomain {
        tenant_id: String,
    },
    ParticipantBinding {
        participant_id: String,
        roles: Vec<String>,
        admitted_consumer: String,
        admitted_view_kind: String,
    },
    ProviderBinding {
        provider_id: String,
        provider_kind: String,
        model_id: String,
    },
    ResourceAttachment {
        attachment_id: String,
        resource_kind: ResourceKind,
        allowed_write_prefix: String,
        max_write_bytes: usize,
        review_requirement: ReviewRequirement,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        process_signal_actions: Vec<crate::effect::ProcessSignalAction>,
    },
    DecisionOutcome {
        operation_id: String,
        decision_id: String,
        outcome: DecisionOutcome,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        decision_basis_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effective_policy_id: Option<String>,
    },
    ReviewPosture {
        review_id: String,
        operation_id: String,
        reviewer_participant_id: String,
        status: ReviewResolution,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        latest_action_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effective_decision_id: Option<String>,
    },
    ResourceConsequence {
        operation_id: String,
        effect_id: String,
        relative_path: String,
        lifecycle: EffectLifecycle,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        outcome: Option<EffectOutcome>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content_digest: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        receipt_id: Option<String>,
    },
    ResourceObservation {
        observation_id: String,
        operation_id: String,
        resource_id: String,
        result_digest: String,
        preview: String,
        truncated: bool,
    },
    CaseContent {
        admission_id: String,
        object: crate::conversation::ConversationContentObject,
        source_resource_id: String,
        source_path: String,
    },
    ProviderClaim {
        result_id: String,
        invocation_id: String,
        preview: String,
    },
    InteractionTurn {
        turn_id: String,
        thread_id: String,
        operator_input: String,
        result_id: String,
    },
    ConversationTurn {
        turn_id: String,
        thread_id: String,
        ordered_parts: Vec<SemanticContentPart>,
    },
    DerivedMemory {
        memory_ref: String,
        semantic_kind: String,
        memory_posture: String,
        description: String,
        lifecycle: String,
        score: i64,
        ranking_reasons: Vec<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticContentPart {
    pub ordinal: u16,
    pub part_id: String,
    pub object_id: String,
    pub modality: crate::conversation::ContentModality,
    pub media_type: String,
    pub byte_length: u64,
    pub content_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub provenance_posture: String,
    #[serde(default)]
    pub source_part_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticEntry {
    pub entry_id: String,
    pub posture: AuthorityPosture,
    pub value: SemanticValue,
    pub provenance: Vec<SemanticProvenance>,
}

// Compatibility names are re-exported by context, not independent types.
use SemanticContentPart as ProjectedConversationContentPart;
use SemanticEntry as ProjectionEntry;
use SemanticPurpose as ProjectionPurpose;
use SemanticValue as ProjectedValue;
use SemanticVisibility as ProjectionVisibility;

pub(crate) struct SemanticCandidates {
    pub case_id: String,
    pub case_generation: u64,
    pub participant_id: String,
    pub purpose: SemanticPurpose,
    pub visibility: SemanticVisibility,
    pub entries: Vec<SemanticEntry>,
    pub bounds: ProjectionBounds,
}

pub(crate) fn collect_candidates(
    state: &CaseState,
    transitions: &[Transition],
    request: &ProjectionRequest,
    derived: &DerivedProjectionInput,
) -> Result<SemanticCandidates, String> {
    if request.max_items == 0 {
        return Err("projection_max_items_must_be_positive".to_string());
    }
    if state.generation != transitions.last().map(|item| item.sequence).unwrap_or(0) {
        return Err("projection_history_generation_mismatch".to_string());
    }
    let participant = state
        .participants
        .iter()
        .find(|item| item.participant_id == request.participant_id)
        .ok_or_else(|| "projection_participant_not_bound".to_string())?;
    let admitted = participant
        .admitted_views
        .iter()
        .any(|view| view.consumer == request.consumer && view.view_kind == request.view_kind);
    if !admitted {
        return Err("projection_view_not_admitted".to_string());
    }

    let mut mandatory = Vec::new();
    mandatory.push(ProjectionEntry {
        entry_id: "case:lifecycle".to_string(),
        posture: AuthorityPosture::CommittedOperationalFact,
        value: ProjectedValue::CaseLifecycle {
            lifecycle: state.lifecycle.clone(),
        },
        provenance: provenance_for_latest(transitions, |payload| {
            matches!(
                payload,
                TransitionPayload::CaseOpened { .. } | TransitionPayload::TenantCaseOpened { .. }
            )
        }),
    });
    if let Some(tenant_id) = &state.tenant_id {
        mandatory.push(ProjectionEntry {
            entry_id: "case:tenant-security-domain".to_string(),
            posture: AuthorityPosture::CommittedOperationalFact,
            value: ProjectedValue::TenantSecurityDomain {
                tenant_id: tenant_id.clone(),
            },
            provenance: provenance_for_latest(transitions, |payload| {
                matches!(payload, TransitionPayload::TenantCaseOpened { tenant_id: opened, .. } if opened == tenant_id)
            }),
        });
    }
    mandatory.push(ProjectionEntry {
        entry_id: format!("participant:{}", request.participant_id),
        posture: AuthorityPosture::CommittedOperationalFact,
        value: ProjectedValue::ParticipantBinding {
            participant_id: request.participant_id.clone(),
            roles: participant.roles.clone(),
            admitted_consumer: request.consumer.clone(),
            admitted_view_kind: request.view_kind.clone(),
        },
        provenance: provenance_for_latest(transitions, |payload| {
            matches!(
                payload,
                TransitionPayload::ParticipantAdmitted { participant_id, consumer, view_kind }
                    if participant_id == &request.participant_id
                        && consumer == &request.consumer
                        && view_kind == &request.view_kind
            )
        }),
    });
    if let Some(provider) = &state.provider {
        if provider.participant_id == request.participant_id {
            mandatory.push(ProjectionEntry {
                entry_id: "provider:binding".to_string(),
                posture: AuthorityPosture::CommittedOperationalFact,
                value: ProjectedValue::ProviderBinding {
                    provider_id: provider.provider_id.clone(),
                    provider_kind: provider.provider_kind.clone(),
                    model_id: provider.model_id.clone(),
                },
                provenance: provenance_for_latest(transitions, |payload| {
                    matches!(payload, TransitionPayload::ProviderAttached { participant_id, .. } if participant_id == &request.participant_id)
                }),
            });
        }
    }
    for resource in state.resources.iter().filter(|resource| {
        resource
            .access
            .as_ref()
            .is_none_or(|access| access.participant_ids.contains(&request.participant_id))
    }) {
        mandatory.push(ProjectionEntry {
            entry_id: format!("resource:{}", resource.attachment_id),
            posture: AuthorityPosture::CommittedOperationalFact,
            value: ProjectedValue::ResourceAttachment {
                attachment_id: resource.attachment_id.clone(),
                resource_kind: resource.kind.clone(),
                allowed_write_prefix: resource.allowed_write_prefix.clone(),
                max_write_bytes: resource.max_write_bytes,
                review_requirement: resource.review_requirement.clone(),
                process_signal_actions: resource.process_signal_actions.clone(),
            },
            provenance: provenance_for_latest(transitions, |payload| {
                matches!(payload, TransitionPayload::ResourceAttached { attachment } if attachment.attachment_id == resource.attachment_id)
            }),
        });
    }
    if let Some(decision) = &state.last_decision {
        mandatory.push(ProjectionEntry {
            entry_id: format!("decision:{}", decision.decision_id),
            posture: AuthorityPosture::ControlState,
            value: ProjectedValue::DecisionOutcome {
                operation_id: decision.operation_id.clone(),
                decision_id: decision.decision_id.clone(),
                outcome: decision.outcome.clone(),
                decision_basis_id: decision.decision_basis_id.clone(),
                effective_policy_id: decision.effective_policy_id.clone(),
            },
            provenance: provenance_for_latest(transitions, |payload| {
                matches!(payload, TransitionPayload::DecisionRecorded { decision: item } if item.decision_id == decision.decision_id)
            }),
        });
    }
    for review in state.reviews.iter().filter(|review| {
        !review.operation_id.is_empty()
            && (review.requested_by_participant == request.participant_id
                || review.reviewer_participant == request.participant_id
                || state.participants.iter().any(|participant| {
                    participant.participant_id == request.participant_id
                        && !review.required_reviewer_roles.is_empty()
                        && review
                            .required_reviewer_roles
                            .iter()
                            .all(|role| participant.roles.contains(role))
                }))
            && matches!(
                review.status,
                ReviewResolution::Pending
                    | ReviewResolution::PendingOperator
                    | ReviewResolution::Deferred
            )
    }) {
        mandatory.push(ProjectionEntry {
            entry_id: format!("review:{}", review.review_id),
            posture: AuthorityPosture::Unresolved,
            value: ProjectedValue::ReviewPosture {
                review_id: review.review_id.clone(),
                operation_id: review.operation_id.clone(),
                reviewer_participant_id: review.reviewer_participant.clone(),
                status: review.status.clone(),
                latest_action_id: review.latest_action_id.clone(),
                effective_decision_id: review.effective_decision_id.clone(),
            },
            provenance: provenance_for_latest(transitions, |payload| match payload {
                TransitionPayload::ReviewRequested { review: item } => {
                    item.review_id == review.review_id
                }
                TransitionPayload::ReviewActionRecorded { action } => {
                    action.review_id == review.review_id
                }
                _ => false,
            }),
        });
    }
    let mut selected_effects = state
        .effects
        .iter()
        .rev()
        .filter(|effect| effect.status != EffectLifecycle::Finalized)
        .collect::<Vec<_>>();
    selected_effects.extend(
        state
            .effects
            .iter()
            .rev()
            .filter(|effect| effect.status == EffectLifecycle::Finalized)
            .take(if request.max_items == usize::MAX {
                usize::MAX
            } else {
                4
            }),
    );
    selected_effects.sort_by_key(|effect| effect.updated_at_generation);
    selected_effects.retain(|effect| {
        state
            .resources
            .iter()
            .find(|resource| resource.attachment_id == effect.resource_attachment_id)
            .is_none_or(|resource| {
                resource
                    .access
                    .as_ref()
                    .is_none_or(|access| access.participant_ids.contains(&request.participant_id))
            })
    });
    selected_effects.dedup_by(|left, right| left.effect_id == right.effect_id);
    let mut omitted_historical_effects = state.effects.len().saturating_sub(selected_effects.len());
    for effect in selected_effects {
        let posture = match effect.status {
            EffectLifecycle::Finalized => AuthorityPosture::ObservedResourceState,
            EffectLifecycle::Prepared | EffectLifecycle::Indeterminate => {
                AuthorityPosture::Unresolved
            }
        };
        let mut provenance = provenance_for_latest(transitions, |payload| match payload {
            TransitionPayload::EffectPrepared { prepared } => {
                prepared.effect_id == effect.effect_id
            }
            TransitionPayload::ProcessEffectPrepared { prepared } => {
                prepared.effect_id == effect.effect_id
            }
            TransitionPayload::ResourceEffectPrepared { prepared } => {
                prepared.effect_id == effect.effect_id
            }
            TransitionPayload::EffectFinalized { effect_id, .. }
            | TransitionPayload::EffectIndeterminate { effect_id, .. }
            | TransitionPayload::ProcessEffectFinalized { effect_id, .. }
            | TransitionPayload::ProcessEffectIndeterminate { effect_id, .. }
            | TransitionPayload::ResourceEffectFinalized { effect_id, .. }
            | TransitionPayload::ResourceEffectIndeterminate { effect_id, .. }
            | TransitionPayload::EffectReconciled { effect_id, .. } => {
                effect_id == &effect.effect_id
            }
            _ => false,
        });
        if let Some(observation_id) = effect.post_observation_id.as_ref() {
            provenance.push(SemanticProvenance {
                kind: ProvenanceKind::Observation,
                source_ref: observation_id.clone(),
            });
        }
        if let Some(receipt_id) = effect.receipt_id.as_ref() {
            provenance.push(SemanticProvenance {
                kind: ProvenanceKind::EffectReceipt,
                source_ref: receipt_id.clone(),
            });
        }
        mandatory.push(ProjectionEntry {
            entry_id: format!("effect:{}", effect.effect_id),
            posture,
            value: ProjectedValue::ResourceConsequence {
                operation_id: effect.operation_id.clone(),
                effect_id: effect.effect_id.clone(),
                relative_path: effect.relative_path.clone(),
                lifecycle: effect.status.clone(),
                outcome: effect.outcome.clone(),
                content_digest: effect
                    .post_observation_id
                    .as_ref()
                    .map(|_| effect.intended_content_digest.clone()),
                receipt_id: effect.receipt_id.clone(),
            },
            provenance,
        });
    }
    if request.purpose == ProjectionPurpose::MemoryConsolidation {
        // Consolidation source material is carried by its immutable,
        // content-addressed task packet. Keep only the identity/control
        // envelope here so unrelated effects, reviews, resources, turns, and
        // provider claims cannot enter the consolidation context by accident.
        mandatory.retain(|entry| {
            matches!(
                entry.value,
                ProjectedValue::CaseLifecycle { .. }
                    | ProjectedValue::TenantSecurityDomain { .. }
                    | ProjectedValue::ParticipantBinding { .. }
                    | ProjectedValue::ProviderBinding { .. }
            )
        });
        omitted_historical_effects = 0;
    }
    if mandatory.len() > request.max_items {
        return Err(format!(
            "projection_budget_below_mandatory_state: required={} max={}",
            mandatory.len(),
            request.max_items
        ));
    }

    let mut optional = Vec::new();
    for transition in transitions.iter().rev() {
        match &transition.payload {
            TransitionPayload::ResourceObservationRecorded { observation }
            | TransitionPayload::ResourceEffectFinalized { observation, .. }
                if observation.participant_id == request.participant_id
                    && state.resources.iter().any(|resource| {
                        resource.attachment_id == observation.resource_attachment_id
                            && resource.access.as_ref().is_some_and(|access| {
                                access.configuration_digest == observation.configuration_digest
                                    && access.participant_ids.contains(&request.participant_id)
                            })
                    })
                    && optional
                        .iter()
                        .filter(|entry: &&ProjectionEntry| {
                            matches!(entry.value, ProjectedValue::ResourceObservation { .. })
                        })
                        .count()
                        < if request.max_items == usize::MAX {
                            usize::MAX
                        } else {
                            12
                        } =>
            {
                let text = serde_json::to_string(&observation.result)
                    .map_err(|e| format!("projection_resource_result:{e}"))?;
                let mut provenance = transition_provenance(transition);
                provenance.push(SemanticProvenance {
                    kind: ProvenanceKind::Observation,
                    source_ref: observation.observation_id.clone(),
                });
                optional.push(ProjectionEntry {
                    entry_id: observation.observation_id.clone(),
                    posture: AuthorityPosture::ObservedResourceState,
                    value: ProjectedValue::ResourceObservation {
                        observation_id: observation.observation_id.clone(),
                        operation_id: observation.operation_id.clone(),
                        resource_id: observation.resource_attachment_id.clone(),
                        result_digest: crate::effect::digest_bytes(text.as_bytes()),
                        preview: bounded_text(&text, 2048),
                        truncated: text.chars().count() > 2048,
                    },
                    provenance,
                });
            }
            TransitionPayload::CaseContentAdmitted { admission }
                if admission.participant_ids.contains(&request.participant_id)
                    && optional
                        .iter()
                        .filter(|entry: &&ProjectionEntry| {
                            matches!(entry.value, ProjectedValue::CaseContent { .. })
                        })
                        .count()
                        < if request.max_items == usize::MAX {
                            usize::MAX
                        } else {
                            8
                        } =>
            {
                let mut provenance = transition_provenance(transition);
                provenance.push(SemanticProvenance {
                    kind: ProvenanceKind::ContentObject,
                    source_ref: admission.object.object_id.clone(),
                });
                optional.push(ProjectionEntry {
                    entry_id: admission.admission_id.clone(),
                    posture: AuthorityPosture::CommittedApplicationContent,
                    value: ProjectedValue::CaseContent {
                        admission_id: admission.admission_id.clone(),
                        object: admission.object.clone(),
                        source_resource_id: admission.source_resource_id.clone(),
                        source_path: admission.source_path.clone(),
                    },
                    provenance,
                });
            }
            TransitionPayload::ConversationTurnCommitted { turn }
                if (turn.participant_id == request.participant_id
                    || crate::conversation::authorize_turn_execution(
                        state,
                        transitions,
                        turn,
                        &request.participant_id,
                        &turn.submitted_by_principal_id,
                    )
                    .is_ok())
                    && optional
                        .iter()
                        .filter(|entry: &&ProjectionEntry| {
                            matches!(
                                entry.value,
                                ProjectedValue::InteractionTurn { .. }
                                    | ProjectedValue::ConversationTurn { .. }
                            )
                        })
                        .count()
                        < request.max_interaction_turns =>
            {
                let ordered_parts = turn
                    .ordered_parts
                    .iter()
                    .map(|part| {
                        let (provenance_posture, source_part_ids) = match &part.provenance {
                            crate::conversation::ContentPartProvenance::Original { .. } => {
                                ("original".to_string(), Vec::new())
                            }
                            crate::conversation::ContentPartProvenance::Derived { derivation }
                                if derivation.kind
                                    == crate::conversation::ContentDerivationKind::HumanEdit =>
                            {
                                (
                                    "human_edited_derived".to_string(),
                                    derivation.source_part_ids.clone(),
                                )
                            }
                            crate::conversation::ContentPartProvenance::Derived { derivation } => (
                                "machine_or_deterministic_derived".to_string(),
                                derivation.source_part_ids.clone(),
                            ),
                        };
                        ProjectedConversationContentPart {
                            ordinal: part.ordinal,
                            part_id: part.part_id.clone(),
                            object_id: part.object.object_id.clone(),
                            modality: part.object.modality.clone(),
                            media_type: part.object.media_type.clone(),
                            byte_length: part.object.byte_length,
                            content_digest: part.object.content_digest.clone(),
                            text: part.object.inline_text.clone(),
                            provenance_posture,
                            source_part_ids,
                        }
                    })
                    .collect::<Vec<_>>();
                let mut provenance = transition_provenance(transition);
                if turn.participant_id != request.participant_id {
                    provenance.extend(transitions.iter().filter_map(|t| match &t.payload {
                        TransitionPayload::ConversationExecutionIntentRecorded {
                            request: intent,
                        } if intent.source_turn_id == turn.turn_id
                            && intent.participant_id == request.participant_id =>
                        {
                            Some(SemanticProvenance {
                                kind: ProvenanceKind::Transition,
                                source_ref: t.transition_id.clone(),
                            })
                        }
                        _ => None,
                    }));
                }
                provenance.extend(turn.ordered_parts.iter().map(|part| SemanticProvenance {
                    kind: ProvenanceKind::ContentObject,
                    source_ref: part.object.object_id.clone(),
                }));
                optional.push(ProjectionEntry {
                    entry_id: turn.turn_id.clone(),
                    posture: AuthorityPosture::CommittedApplicationContent,
                    value: ProjectedValue::ConversationTurn {
                        turn_id: turn.turn_id.clone(),
                        thread_id: turn.thread_id.clone(),
                        ordered_parts,
                    },
                    provenance,
                });
            }
            TransitionPayload::InteractionTurnRecorded {
                turn_id,
                thread_id,
                participant_id,
                invocation_id: _,
                result_id,
                operator_input,
            } if participant_id == &request.participant_id
                && optional
                    .iter()
                    .filter(|entry: &&ProjectionEntry| {
                        matches!(
                            entry.value,
                            ProjectedValue::InteractionTurn { .. }
                                | ProjectedValue::ConversationTurn { .. }
                        )
                    })
                    .count()
                    < request.max_interaction_turns =>
            {
                optional.push(ProjectionEntry {
                    entry_id: format!("turn:{turn_id}"),
                    posture: AuthorityPosture::ProviderClaim,
                    value: ProjectedValue::InteractionTurn {
                        turn_id: turn_id.clone(),
                        thread_id: thread_id.clone(),
                        operator_input: bounded_text(operator_input, DEFAULT_MAX_CLAIM_CHARS),
                        result_id: result_id.clone(),
                    },
                    provenance: transition_provenance(transition),
                });
            }
            TransitionPayload::ProviderResultRecorded {
                result_id,
                invocation_id,
                output,
                ..
            } if provider_invocation_participant(transitions, invocation_id)
                == Some(request.participant_id.as_str())
                && optional
                    .iter()
                    .filter(|entry: &&ProjectionEntry| {
                        matches!(entry.value, ProjectedValue::ProviderClaim { .. })
                    })
                    .count()
                    < request.max_provider_claims =>
            {
                optional.push(ProjectionEntry {
                    entry_id: format!("provider-claim:{result_id}"),
                    posture: AuthorityPosture::ProviderClaim,
                    value: ProjectedValue::ProviderClaim {
                        result_id: result_id.clone(),
                        invocation_id: invocation_id.clone(),
                        preview: bounded_text(output, DEFAULT_MAX_CLAIM_CHARS),
                    },
                    provenance: transition_provenance(transition),
                });
            }
            _ => {}
        }
    }
    optional.reverse();
    // Retrieval is score-descending. Optional selection keeps the tail, so
    // reverse here to ensure higher-ranked entries survive a Projection budget.
    for memory in derived.memory.iter().rev() {
        optional.push(ProjectionEntry {
            entry_id: format!("memory:{}", memory.memory_ref),
            posture: AuthorityPosture::DerivedMemory,
            value: ProjectedValue::DerivedMemory {
                memory_ref: memory.memory_ref.clone(),
                semantic_kind: memory.semantic_kind.clone(),
                memory_posture: memory.memory_posture.clone(),
                description: bounded_text(&memory.description, DEFAULT_MAX_CLAIM_CHARS),
                lifecycle: memory.lifecycle.clone(),
                score: memory.score,
                ranking_reasons: memory.ranking_reasons.clone(),
            },
            provenance: memory
                .transition_refs
                .iter()
                .map(|source_ref| SemanticProvenance {
                    kind: ProvenanceKind::Transition,
                    source_ref: source_ref.clone(),
                })
                .chain(
                    memory
                        .observation_refs
                        .iter()
                        .map(|source_ref| SemanticProvenance {
                            kind: ProvenanceKind::Observation,
                            source_ref: source_ref.clone(),
                        }),
                )
                .chain(
                    memory
                        .receipt_refs
                        .iter()
                        .map(|source_ref| SemanticProvenance {
                            kind: ProvenanceKind::EffectReceipt,
                            source_ref: source_ref.clone(),
                        }),
                )
                .chain(
                    memory
                        .derived_memory_refs
                        .iter()
                        .map(|source_ref| SemanticProvenance {
                            kind: ProvenanceKind::DerivedMemory,
                            source_ref: source_ref.clone(),
                        }),
                )
                .chain(std::iter::once(SemanticProvenance {
                    kind: ProvenanceKind::DerivedMemory,
                    source_ref: memory.memory_ref.clone(),
                }))
                .collect(),
        });
    }
    if request.purpose == ProjectionPurpose::MemoryConsolidation {
        optional.clear();
    }

    let available = request.max_items - mandatory.len();
    let consolidation_projection = request.purpose == ProjectionPurpose::MemoryConsolidation;
    let omitted_items = optional
        .len()
        .saturating_sub(available)
        .saturating_add(omitted_historical_effects)
        .saturating_add(if consolidation_projection {
            0
        } else {
            derived.retrieval_omitted
        });
    let keep_from = optional.len().saturating_sub(available);
    let mut entries = mandatory;
    entries.extend(optional.into_iter().skip(keep_from));
    let bounds = ProjectionBounds {
        working_state_id: None,
        max_items: request.max_items,
        selected_items: entries.len(),
        omitted_items,
        history_transitions_considered: transitions.len(),
        graph_available: !consolidation_projection && derived.graph_available,
        memory_available: !consolidation_projection && derived.memory_available,
        retrieval_id: if consolidation_projection {
            None
        } else {
            derived.retrieval_id.clone()
        },
        retrieval_candidates: if consolidation_projection {
            0
        } else {
            derived.retrieval_candidates
        },
        retrieval_selected: if consolidation_projection {
            0
        } else {
            derived.memory.len()
        },
        retrieval_omitted: if consolidation_projection {
            0
        } else {
            derived.retrieval_omitted
        },
        residency_plan_id: None,
        semantic_unit_budget: None,
        selected_semantic_units: None,
    };
    let projection = SemanticCandidates {
        case_id: state.case_id.clone(),
        case_generation: state.generation,
        participant_id: request.participant_id.clone(),
        purpose: request.purpose.clone(),
        visibility: ProjectionVisibility {
            consumer: request.consumer.clone(),
            view_kind: request.view_kind.clone(),
        },
        entries,
        bounds,
    };
    Ok(projection)
}

fn provenance_for_latest<F>(transitions: &[Transition], predicate: F) -> Vec<SemanticProvenance>
where
    F: Fn(&TransitionPayload) -> bool,
{
    transitions
        .iter()
        .rev()
        .find(|transition| predicate(&transition.payload))
        .map(transition_provenance)
        .unwrap_or_default()
}

fn transition_provenance(transition: &Transition) -> Vec<SemanticProvenance> {
    vec![
        SemanticProvenance {
            kind: ProvenanceKind::Transition,
            source_ref: transition.transition_id.clone(),
        },
        SemanticProvenance {
            kind: ProvenanceKind::CaseStateGeneration,
            source_ref: format!("{}@{}", transition.case_id, transition.sequence),
        },
    ]
}

fn provider_invocation_participant<'a>(
    transitions: &'a [Transition],
    invocation_id: &str,
) -> Option<&'a str> {
    transitions
        .iter()
        .rev()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: candidate,
                participant_id,
                ..
            } if candidate == invocation_id => Some(participant_id.as_str()),
            _ => None,
        })
}

fn bounded_text(value: &str, max_chars: usize) -> String {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut output = compact.chars().take(max_chars).collect::<String>();
    if compact.chars().count() > max_chars {
        output.push_str("...");
    }
    output
}

#[cfg(test)]
#[path = "semantic_state_tests.rs"]
mod tests;
