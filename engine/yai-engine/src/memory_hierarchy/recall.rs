//! Query-conditioned experience reconstruction. Sources and authority stay with
//! their current owners; neither candidate rank nor this disposable trace is truth.
use super::{
    derive_contradictions, extract_mechanical_assertions, SemanticContradictionSet,
    SemanticMemoryAssertion, SemanticSupportRef,
};
use crate::graph::experience::{
    self, ExperienceEvent, ExperienceQuery, ExperienceRelation, RelationKind,
};
use crate::memory::{derive_operational_memory, OperationalMemoryBuild};
use crate::memory_index::{
    MemoryLexicalIndex, MemoryRepresentationDocument, MemoryRepresentationProfile,
    MemoryVectorIndex, RetrievalQueryDocument,
};
use crate::semantic_state::historical::{
    digest, HistoricalCoordinate, HistoricalSemanticView, SourceClosure,
};
use crate::transition::{Transition, TransitionPayload as P};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::Instant;

pub const RECALL_REQUEST_SCHEMA: &str = "yai.recall_request.v1";
pub const RECALL_TRACE_SCHEMA: &str = "yai.recall_trace.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallBounds {
    pub candidates: usize,
    pub events: usize,
    pub relations: usize,
    pub segments: usize,
    pub expansion_depth: usize,
    pub semantic_units: usize,
    pub bytes: usize,
}

/// Optional pre-encoded search evidence. Recall never invokes an encoder;
/// this input cannot establish semantic suitability, truth or authority.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecallVectorInput {
    pub profile: MemoryRepresentationProfile,
    pub index: MemoryVectorIndex,
    pub query: RetrievalQueryDocument,
    pub query_vector: Vec<f32>,
}
impl Default for RecallBounds {
    fn default() -> Self {
        Self {
            candidates: 16,
            events: 64,
            relations: 128,
            segments: 16,
            expansion_depth: 4,
            semantic_units: 16_384,
            bytes: 1_048_576,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallRequest {
    pub schema: String,
    pub case_id: String,
    /// The exact current source generation, also when recalling an older cut.
    pub expected_generation: u64,
    pub participant_id: String,
    pub at: HistoricalCoordinate,
    pub query: String,
    pub required_refs: Vec<String>,
    pub bounds: RecallBounds,
}
impl RecallRequest {
    pub fn new(
        case: impl Into<String>,
        generation: u64,
        participant: impl Into<String>,
        query: impl Into<String>,
    ) -> Self {
        Self {
            schema: RECALL_REQUEST_SCHEMA.into(),
            case_id: case.into(),
            expected_generation: generation,
            participant_id: participant.into(),
            at: HistoricalCoordinate::Generation(generation),
            query: query.into(),
            required_refs: vec![],
            bounds: RecallBounds::default(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        let b = &self.bounds;
        if self.schema != RECALL_REQUEST_SCHEMA
            || self.case_id.is_empty()
            || self.participant_id.is_empty()
            || self.query.trim().is_empty()
            || self.query.chars().count() > crate::memory_index::MAX_QUERY_CHARS
            || self
                .query
                .chars()
                .any(|c| c.is_control() && !c.is_whitespace())
            || self
                .query
                .split(|c: char| !c.is_alphanumeric())
                .filter(|s| !s.is_empty())
                .count()
                > crate::memory_index::MAX_QUERY_TERMS
            || self.required_refs.len() > 16
            || self
                .required_refs
                .iter()
                .any(|r| r.is_empty() || r.len() > 512)
            || b.candidates == 0
            || b.candidates > 256
            || b.events == 0
            || b.events > 256
            || b.relations == 0
            || b.relations > 1024
            || b.segments == 0
            || b.segments > 64
            || b.expansion_depth == 0
            || b.expansion_depth > 16
            || b.semantic_units == 0
            || b.semantic_units > 262_144
            || b.bytes == 0
            || b.bytes > 1_048_576
        {
            return Err("recall_request_or_bounds_invalid".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionReason {
    ExactRequiredRef,
    CurrentStateAnchor,
    LexicalCandidate,
    VectorCandidate,
    QualifiedRelationExpansion,
    SourceClosure,
    SupersessionChain,
    ContradictionContext,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallEvent {
    pub event: ExperienceEvent,
    pub label: String,
    pub validity_at_cut: String,
    pub reasons: Vec<SelectionReason>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallSegment {
    pub segment_id: String,
    pub events: Vec<String>,
    /// Structural grouping is not itself a causal relation.
    pub grouping: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallCandidate {
    pub source_ref: String,
    pub plane: String,
    pub score_micros: i64,
    pub matched_terms: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallAssertion {
    pub assertion: SemanticMemoryAssertion,
    pub source_events: Vec<String>,
    pub reasons: Vec<SelectionReason>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallTrace {
    pub schema: String,
    pub trace_id: String,
    pub request_id: String,
    pub request: RecallRequest,
    pub generation: u64,
    pub disclosure_digest: String,
    pub qualified_source_digest: String,
    pub events: Vec<RecallEvent>,
    pub relations: Vec<ExperienceRelation>,
    pub segments: Vec<RecallSegment>,
    pub assertions: Vec<RecallAssertion>,
    pub contradictions: Vec<SemanticContradictionSet>,
    pub candidates: Vec<RecallCandidate>,
    pub vector_evidence_digest: Option<String>,
    pub source_closure: Vec<SourceClosure>,
    pub closure_complete: bool,
    pub omitted_candidates: usize,
    pub expansion_stopped_at_depth: bool,
    pub limitations: Vec<String>,
}
/// Measurements are not semantic identity and expose no hidden source counts.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallMeasurements {
    pub qualified_events: usize,
    pub discovery_us: u128,
    pub relation_build_us: u128,
    pub qualification_us: u128,
    pub source_closure_us: u128,
    pub semantic_units: usize,
    pub output_bytes: usize,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallResult {
    pub trace: RecallTrace,
    pub measurements: RecallMeasurements,
}

fn label(payload: &P) -> String {
    // No raw payload/causal_refs: labels cannot expose a hidden intermediate ID.
    let value = match payload {
        P::CasePolicyBound { binding } | P::CasePolicyReplaced { binding, .. } => format!(
            "policy {} version {}",
            binding.policy_key, binding.artifact_version
        ),
        P::OperationRecorded { operation } => format!(
            "operation {:?} resource {} path {}",
            operation.kind,
            operation.resource_attachment_id,
            operation
                .resource_request
                .as_ref()
                .and_then(|r| r.action.path())
                .unwrap_or(&operation.filesystem_write.relative_path)
        ),
        P::DecisionRecorded { decision } => format!("decision {:?}", decision.outcome),
        P::ResourceObservationRecorded { observation }
        | P::ResourceEffectFinalized { observation, .. } => format!(
            "observation {} resource {}",
            observation.observation_id, observation.resource_attachment_id
        ),
        _ => payload.kind().to_string(),
    };
    value.chars().take(4096).collect()
}

fn event_document(
    case: &str,
    participant: &str,
    e: &ExperienceEvent,
    label: &str,
) -> MemoryRepresentationDocument {
    let source_digest = digest(&(e, label));
    MemoryRepresentationDocument {
        schema: "yai.recall_candidate_document.v1".into(),
        document_id: format!(
            "recall-document:{}",
            digest(&(case, participant, &source_digest))
        ),
        case_id: case.into(),
        memory_id: e.transition_id.clone(),
        source_content_digest: source_digest,
        operational_memory_derivation_version: RECALL_TRACE_SCHEMA.into(),
        representation_contract_version: "yai.recall_candidate_document.v1".into(),
        semantic_kind: e.kind.clone(),
        authority_posture: format!("{:?}", e.posture),
        lifecycle: "recorded".into(),
        canonical_text: label.into(),
        provenance_refs: vec![e.transition_id.clone()],
        participant_ids: vec![participant.into()],
        consumer: "operator".into(),
        view_kind: "case_inspection".into(),
        source_family: "experience".into(),
        source_id: e.transition_id.clone(),
        epistemic_class: format!("{:?}", e.posture),
        support_refs: vec![e.transition_id.clone()],
        contradiction_set_ref: None,
    }
}

fn source_events(
    a: &SemanticMemoryAssertion,
    memory: &OperationalMemoryBuild,
    allowed: &BTreeSet<String>,
) -> Vec<String> {
    a.support_refs
        .iter()
        .filter_map(|s| match s {
            SemanticSupportRef::Operational(id) => {
                memory.entries.iter().find(|m| &m.memory_id == id)
            }
            _ => None,
        })
        .flat_map(|m| m.provenance.transition_ids.iter())
        .filter(|id| allowed.contains(*id))
        .cloned()
        .collect()
}

type Selection = BTreeMap<String, BTreeSet<SelectionReason>>;

/// Conflict context and exact backing are mandatory once a source is selected.
/// They are never rank-limited and never cross the already qualified source set.
fn close_selection(
    selected: &mut Selection,
    assertions: &[SemanticMemoryAssertion],
    sources: &BTreeMap<String, Vec<String>>,
    conflicts: &[SemanticContradictionSet],
    limit: usize,
) -> Result<(), String> {
    loop {
        let before = selected.len();
        for a in assertions {
            if sources[&a.assertion_id]
                .iter()
                .any(|id| selected.contains_key(id))
            {
                for id in &sources[&a.assertion_id] {
                    selected
                        .entry(id.clone())
                        .or_default()
                        .insert(SelectionReason::SourceClosure);
                }
            }
        }
        for set in conflicts {
            if set.competing_assertion_ids.iter().any(|id| {
                sources
                    .get(id)
                    .is_some_and(|s| s.iter().any(|id| selected.contains_key(id)))
            }) {
                for id in &set.competing_assertion_ids {
                    for node in sources.get(id).into_iter().flatten() {
                        selected.entry(node.clone()).or_default().insert(
                            if set.resolution_posture == "mechanically_superseded" {
                                SelectionReason::SupersessionChain
                            } else {
                                SelectionReason::ContradictionContext
                            },
                        );
                    }
                }
            }
        }
        if selected.len() > limit {
            return Err("recall_required_context_exceeds_event_budget".into());
        }
        if before == selected.len() {
            return Ok(());
        }
    }
}

fn expand(
    selected: &mut Selection,
    relations: &[ExperienceRelation],
    depth_limit: usize,
    limit: usize,
) -> Result<bool, String> {
    let seeds: BTreeSet<_> = selected.keys().cloned().collect();
    let mut queue: VecDeque<_> = seeds.iter().cloned().map(|id| (id, 0)).collect();
    let mut visited = seeds.clone();
    let mut stopped = false;
    while let Some((id, depth)) = queue.pop_front() {
        for relation in relations
            .iter()
            .filter(|r| r.kind != RelationKind::RecordedBefore)
        {
            // Follow evidence towards its backing; don't fan out from a common
            // policy/support ancestor into every unrelated operation it governed.
            let other = if relation.to_event == id {
                &relation.from_event
            } else if relation.from_event == id
                && relation.kind != RelationKind::PolicyBasis
                && (relation.kind != RelationKind::EvidenceSupport || seeds.contains(&id))
            {
                &relation.to_event
            } else {
                continue;
            };
            if visited.contains(other) {
                continue;
            }
            if depth >= depth_limit {
                stopped = true;
                continue;
            }
            visited.insert(other.clone());
            selected
                .entry(other.clone())
                .or_default()
                .insert(SelectionReason::QualifiedRelationExpansion);
            if selected.len() > limit {
                return Err("recall_required_context_exceeds_event_budget".into());
            }
            queue.push_back((other.clone(), depth + 1));
        }
    }
    Ok(stopped)
}

/// Each discovery seed has its own depth-bounded context. Previously selected
/// nodes must not become fresh depth-zero seeds when another candidate arrives.
fn resolve_group(
    mut group: Selection,
    assertions: &[SemanticMemoryAssertion],
    sources: &BTreeMap<String, Vec<String>>,
    conflicts: &[SemanticContradictionSet],
    relations: &[ExperienceRelation],
    bounds: &RecallBounds,
) -> Result<(Selection, bool), String> {
    close_selection(&mut group, assertions, sources, conflicts, bounds.events)?;
    let stopped = expand(&mut group, relations, bounds.expansion_depth, bounds.events)?;
    close_selection(&mut group, assertions, sources, conflicts, bounds.events)?;
    Ok((group, stopped))
}

/// Internal constructor: called only after transactional historical qualification.
pub(crate) fn compile(
    h: &HistoricalSemanticView,
    disclosure: &str,
    history: &[Transition],
    mut request: RecallRequest,
    vectors: Option<&RecallVectorInput>,
) -> Result<RecallResult, String> {
    request.validate()?;
    if h.case_id != request.case_id
        || h.current_generation != request.expected_generation
        || h.request.participant_id != request.participant_id
    {
        return Err("recall_source_generation_or_scope_mismatch".into());
    }
    request.at = HistoricalCoordinate::Generation(h.generation);
    request.required_refs.sort();
    request.required_refs.dedup();
    let query = RetrievalQueryDocument::new_v2(&request.query)?;
    request.query = query.canonical_text.clone();
    let mut metrics = RecallMeasurements::default();
    let start = Instant::now();
    let graph = experience::derive(
        h,
        ExperienceQuery {
            max_events: 4096,
            max_relations: 32768,
            max_bytes: 16_777_216,
            ..Default::default()
        },
        disclosure,
    )?;
    metrics.relation_build_us = start.elapsed().as_micros();
    metrics.qualified_events = graph.events.len();
    let allowed: BTreeSet<_> = graph
        .events
        .iter()
        .map(|e| e.transition_id.clone())
        .collect();
    // W20 derives from canonical history, never cached values. Its support must
    // close through disclosed events; own Invocation metadata only backs a visible Result.
    let cut = crate::semantic_state::historical::prefix(history, &request.at)?;
    let mut source_allowed = allowed.clone();
    for e in &h.known_by_then {
        if let P::ProviderResultRecorded { invocation_id, .. } = &e.payload {
            if let Some(t) = cut.iter().find(|t| matches!(&t.payload, P::ProviderInvocationStarted { invocation_id: id, participant_id, .. }
                if id == invocation_id && participant_id == &request.participant_id)) {
                source_allowed.insert(t.transition_id.clone());
            }
        }
    }
    let start = Instant::now();
    let mut memory = derive_operational_memory(&request.case_id, cut)?;
    memory.entries.retain(|m| {
        m.provenance
            .transition_ids
            .iter()
            .all(|id| source_allowed.contains(id))
    });
    let mut assertions = extract_mechanical_assertions(&request.case_id, &memory)?;
    let contradictions = derive_contradictions(&mut assertions)?;
    let assertion_sources: BTreeMap<_, _> = assertions
        .iter()
        .map(|a| (a.assertion_id.clone(), source_events(a, &memory, &allowed)))
        .collect();
    assertions.retain(|a| !assertion_sources[&a.assertion_id].is_empty());
    metrics.qualification_us = start.elapsed().as_micros();
    let labels: BTreeMap<_, _> = h
        .known_by_then
        .iter()
        .map(|e| (e.transition_id.clone(), label(&e.payload)))
        .collect();
    let mut aliases = BTreeMap::<String, BTreeSet<String>>::new();
    for e in &graph.events {
        for id in std::iter::once(&e.transition_id).chain(e.object_refs.iter()) {
            aliases
                .entry(id.clone())
                .or_default()
                .insert(e.transition_id.clone());
        }
    }
    // A resource anchor denotes all disclosed events with that exact typed subject,
    // not every event containing its spelling. Associations do not mint graph edges.
    for e in &h.known_by_then {
        let resource = match &e.payload {
            P::OperationRecorded { operation } => Some(&operation.resource_attachment_id),
            P::ResourceObservationRecorded { observation }
            | P::ResourceEffectFinalized { observation, .. } => {
                Some(&observation.resource_attachment_id)
            }
            P::EffectPrepared { prepared } => Some(&prepared.resource_attachment_id),
            _ => None,
        };
        if let Some(id) = resource {
            aliases
                .entry(id.clone())
                .or_default()
                .insert(e.transition_id.clone());
        }
    }
    for slice in &graph.episode_slices {
        aliases.insert(
            slice.slice_id.clone(),
            slice.transition_ids.iter().cloned().collect(),
        );
    }
    // W20 Episode IDs are accepted only when the complete structural source is
    // currently disclosed. Hidden members must not leak through an Episode hash.
    for episode in super::derive_episodes(&request.case_id, cut)? {
        if episode
            .transition_ids
            .iter()
            .all(|id| source_allowed.contains(id))
        {
            let nodes: BTreeSet<_> = episode
                .transition_ids
                .into_iter()
                .filter(|id| allowed.contains(id))
                .collect();
            if !nodes.is_empty() {
                aliases.insert(episode.episode_id, nodes);
            }
        }
    }
    for a in &assertions {
        aliases.insert(
            a.assertion_id.clone(),
            assertion_sources[&a.assertion_id].iter().cloned().collect(),
        );
    }
    let mut selected = BTreeMap::<String, BTreeSet<SelectionReason>>::new();
    for id in &request.required_refs {
        let nodes = aliases
            .get(id)
            .ok_or("recall_required_anchor_unavailable")?;
        if nodes.is_empty() {
            return Err("recall_required_anchor_unavailable".into());
        }
        for node in nodes {
            selected
                .entry(node.clone())
                .or_default()
                .insert(SelectionReason::ExactRequiredRef);
        }
    }
    let start = Instant::now();
    let mut documents: Vec<_> = graph
        .events
        .iter()
        .map(|e| {
            event_document(
                &request.case_id,
                &request.participant_id,
                e,
                &labels[&e.transition_id],
            )
        })
        .collect();
    for a in &assertions {
        documents.push(MemoryRepresentationDocument::from_semantic_assertion(a)?);
    }
    let index = MemoryLexicalIndex::build(&documents)?;
    let hits = index.search(&request.query, request.bounds.candidates)?;
    let by_document: BTreeMap<_, _> = documents
        .iter()
        .map(|d| (d.document_id.as_str(), d))
        .collect();
    let mut candidates: Vec<_> = hits
        .iter()
        .map(|hit| RecallCandidate {
            source_ref: by_document[hit.document_id.as_str()].source_id.clone(),
            plane: "lexical_bm25".into(),
            score_micros: hit.score_micros,
            matched_terms: hit.matched_terms.clone(),
        })
        .collect();
    let vector_evidence_digest = if let Some(v) = vectors {
        v.profile.validate()?;
        if v.query != query
            || v.index.schema != crate::memory_index::MEMORY_VECTOR_INDEX_SCHEMA
            || v.index.embeddings.len() > crate::memory_index::MAX_CORPUS_DOCUMENTS
            || v.profile.representation_contract_version
                != crate::memory_index::MEMORY_REPRESENTATION_CONTRACT_V2
            || h.state_then
                .principal_association
                .as_ref()
                .is_none_or(|l| l.tenant_id != v.profile.tenant_id)
            || v.index.profile_id != v.profile.profile_id
            || v.index.dimension != v.profile.vector_dimension
        {
            return Err("recall_vector_query_profile_or_scope_mismatch".into());
        }
        // No cached text or undisclosed corpus statistics enter Recall. Validate
        // each candidate embedding against a freshly rebuilt v2 assertion.
        let mut admitted = BTreeSet::new();
        let mut identities = Vec::new();
        for embedding in &v.index.embeddings {
            let Some(d) = by_document.get(embedding.representation_document_id.as_str()) else {
                continue;
            };
            if d.representation_contract_version != v.profile.representation_contract_version {
                continue;
            }
            embedding.validate(d, &v.profile)?;
            if !admitted.insert(d.document_id.clone()) {
                return Err("recall_vector_duplicate_document".into());
            }
            identities.push(embedding.embedding_id.clone());
        }
        let hits = v.index.exact_search_qualified(
            &v.query_vector,
            request.bounds.candidates,
            &admitted,
        )?;
        for hit in hits {
            candidates.push(RecallCandidate {
                source_ref: by_document[hit.document_id.as_str()].source_id.clone(),
                plane: "exact_vector_candidate".into(),
                score_micros: hit.similarity_micros,
                matched_terms: vec![],
            });
        }
        identities.sort();
        Some(digest(&(&v.profile, &v.query, &v.query_vector, identities)))
    } else {
        None
    };
    // Stable round-robin plane merge. Enabling another plane does not double
    // the candidate budget; cosine and BM25 scores are not comparable.
    if vector_evidence_digest.is_some() {
        let lexical: Vec<_> = candidates
            .iter()
            .filter(|c| c.plane == "lexical_bm25")
            .cloned()
            .collect();
        let vector: Vec<_> = candidates
            .iter()
            .filter(|c| c.plane != "lexical_bm25")
            .cloned()
            .collect();
        candidates = (0..request.bounds.candidates)
            .flat_map(|i| [lexical.get(i), vector.get(i)])
            .flatten()
            .cloned()
            .take(request.bounds.candidates)
            .collect();
    }
    metrics.discovery_us = start.elapsed().as_micros();
    let qualification_start = Instant::now();
    // Each candidate plus its necessary context is an atomic optional group.
    // Exact anchors were inserted first and can never be evicted by ranking.
    let (required, mut stopped) = resolve_group(
        selected,
        &assertions,
        &assertion_sources,
        &contradictions,
        &graph.relations,
        &request.bounds,
    )?;
    let mut selected = required;
    let mut omitted_candidates = 0;
    for candidate in &candidates {
        if let Some(nodes) = aliases.get(&candidate.source_ref) {
            let mut proposed = Selection::new();
            for node in nodes {
                proposed.entry(node.clone()).or_default().insert(
                    if candidate.plane == "lexical_bm25" {
                        SelectionReason::LexicalCandidate
                    } else {
                        SelectionReason::VectorCandidate
                    },
                );
            }
            let accepted = resolve_group(
                proposed,
                &assertions,
                &assertion_sources,
                &contradictions,
                &graph.relations,
                &request.bounds,
            );
            match accepted {
                Ok((group, cutoff))
                    if selected.len()
                        + group
                            .keys()
                            .filter(|id| !selected.contains_key(*id))
                            .count()
                        <= request.bounds.events =>
                {
                    for (id, reasons) in group {
                        selected.entry(id).or_default().extend(reasons);
                    }
                    stopped |= cutoff;
                }
                _ => omitted_candidates += 1,
            }
        }
    }
    // Current-at-cut policy is mandatory when recalling its lineage, independently
    // of which historical version received a high lexical score.
    let selected_policy_keys: BTreeSet<_> = h
        .known_by_then
        .iter()
        .filter(|e| selected.contains_key(&e.transition_id))
        .filter_map(|e| match &e.payload {
            P::CasePolicyBound { binding } | P::CasePolicyReplaced { binding, .. } => {
                Some(binding.lineage_id.clone())
            }
            _ => None,
        })
        .collect();
    for binding in &h.state_then.policy_bindings {
        if selected_policy_keys.contains(&binding.lineage_id) {
            if let Some(nodes) = aliases.get(&binding.binding_id) {
                for node in nodes {
                    selected
                        .entry(node.clone())
                        .or_default()
                        .insert(SelectionReason::CurrentStateAnchor);
                }
            }
        }
    }
    close_selection(
        &mut selected,
        &assertions,
        &assertion_sources,
        &contradictions,
        request.bounds.events,
    )?;
    let selected_assertions: Vec<_> = assertions
        .iter()
        .filter(|a| {
            assertion_sources[&a.assertion_id]
                .iter()
                .any(|id| selected.contains_key(id))
        })
        .map(|a| RecallAssertion {
            assertion: a.clone(),
            source_events: assertion_sources[&a.assertion_id].clone(),
            reasons: vec![SelectionReason::SourceClosure],
        })
        .collect();
    // Every assertion's backing events must be present, not only its candidate label.
    for a in &selected_assertions {
        for id in &a.source_events {
            selected
                .entry(id.clone())
                .or_default()
                .insert(SelectionReason::SourceClosure);
        }
    }
    let mut events = Vec::new();
    for event in &graph.events {
        let Some(reasons) = selected.get(&event.transition_id) else {
            continue;
        };
        let mut validity = "recorded_evidence_not_current_authority";
        if let Some(P::CasePolicyBound { binding } | P::CasePolicyReplaced { binding, .. }) = h
            .known_by_then
            .iter()
            .find(|e| e.transition_id == event.transition_id)
            .map(|e| &e.payload)
        {
            validity = if h
                .state_then
                .policy_bindings
                .iter()
                .any(|b| b.binding_id == binding.binding_id)
            {
                "bound_at_cut_not_execution_permission"
            } else {
                "historical_binding_not_current_at_cut"
            };
        }
        events.push(RecallEvent {
            event: event.clone(),
            label: labels[&event.transition_id].clone(),
            validity_at_cut: validity.into(),
            reasons: reasons.iter().cloned().collect(),
        });
    }
    let relations: Vec<_> = graph
        .relations
        .iter()
        .filter(|r| selected.contains_key(&r.from_event) && selected.contains_key(&r.to_event))
        .cloned()
        .collect();
    let mut segments = Vec::new();
    let mut grouped = BTreeSet::new();
    for s in &graph.episode_slices {
        let ids: Vec<_> = s
            .transition_ids
            .iter()
            .filter(|id| selected.contains_key(*id))
            .cloned()
            .collect();
        if !ids.is_empty() {
            grouped.extend(ids.clone());
            segments.push(RecallSegment {
                segment_id: s.slice_id.clone(),
                events: ids,
                grouping: "typed_episode_slice_not_causality".into(),
            });
        }
    }
    for event in &events {
        if !grouped.contains(&event.event.transition_id) {
            segments.push(RecallSegment {
                segment_id: format!(
                    "recall-segment:{}",
                    digest(&(&request.case_id, &event.event.transition_id))
                ),
                events: vec![event.event.transition_id.clone()],
                grouping: "independent_recorded_event".into(),
            });
        }
    }
    let order: BTreeMap<_, _> = events
        .iter()
        .map(|e| (e.event.transition_id.as_str(), e.event.recorded_generation))
        .collect();
    segments.sort_by_key(|s| {
        s.events
            .iter()
            .filter_map(|id| order.get(id.as_str()))
            .min()
            .copied()
            .unwrap_or(0)
    });
    let kept_assertions: BTreeSet<_> = selected_assertions
        .iter()
        .map(|a| a.assertion.assertion_id.clone())
        .collect();
    let contradictions = contradictions
        .into_iter()
        .filter(|c| {
            c.competing_assertion_ids
                .iter()
                .all(|id| kept_assertions.contains(id))
        })
        .collect();
    metrics.qualification_us += qualification_start.elapsed().as_micros();
    let start = Instant::now();
    let mut closure: Vec<_> = events
        .iter()
        .map(|e| SourceClosure {
            source_ref: e.event.transition_id.clone(),
            posture: if e.event.source_closed {
                "exact_qualified_recorded_source"
            } else {
                "required_backing_unavailable"
            }
            .into(),
        })
        .collect();
    let mut backing_refs = BTreeSet::new();
    for item in h
        .known_by_then
        .iter()
        .filter(|e| selected.contains_key(&e.transition_id))
    {
        match &item.payload {
            P::CasePolicyBound { binding } | P::CasePolicyReplaced { binding, .. } => {
                backing_refs.extend([
                    binding.artifact_id.clone(),
                    binding.source_id.clone(),
                    binding.publication_event_id.clone(),
                ]);
            }
            P::DecisionRecorded { decision } => {
                if let Some(b) = &decision.decision_basis {
                    backing_refs.extend(b.policy_artifact_refs.clone());
                }
            }
            P::CaseContentAdmitted { admission } => {
                backing_refs.insert(admission.object.object_id.clone());
            }
            P::ConversationTurnCommitted { turn } => {
                backing_refs.extend(
                    turn.ordered_parts
                        .iter()
                        .map(|p| p.object.object_id.clone()),
                );
            }
            _ => (),
        }
    }
    for id in backing_refs {
        if let Some(s) = h
            .content_backing
            .iter()
            .chain(&h.normative_then.source_closure)
            .find(|s| s.source_ref == id)
        {
            closure.push(s.clone());
        } else {
            closure.push(SourceClosure {
                source_ref: id,
                posture: "exact_backing_not_resolved".into(),
            });
        }
    }
    for a in &selected_assertions {
        for support in &a.assertion.support_refs {
            closure.push(SourceClosure {
                source_ref: support.id().into(),
                posture: "rebuilt_w20_support_from_qualified_recorded_sources".into(),
            });
        }
    }
    closure.sort_by(|a, b| a.source_ref.cmp(&b.source_ref));
    closure.dedup();
    let complete = closure.iter().all(|s| {
        matches!(
            s.posture.as_str(),
            "exact_qualified_recorded_source"
                | "exact_original_available"
                | "exact_immutable_artifact"
                | "exact_binding_publication_anchor"
                | "exact_source_available"
                | "rebuilt_w20_support_from_qualified_recorded_sources"
        )
    });
    metrics.source_closure_us = start.elapsed().as_micros();
    let mut trace = RecallTrace {
        schema: RECALL_TRACE_SCHEMA.into(), trace_id: String::new(), request_id: format!("recall-request:{}", digest(&request)),
        request, generation: h.generation, disclosure_digest: disclosure.into(),
        qualified_source_digest: digest(&(&graph.events, &graph.relations, &assertions)),
        events, relations, segments, assertions: selected_assertions, contradictions, candidates, vector_evidence_digest,
        source_closure: closure, closure_complete: complete, omitted_candidates,
        expansion_stopped_at_depth: stopped,
        limitations: vec!["derived Recall; not authority, W or model input".into(),
            "recording order is not physical causality; occurrence time may be absent".into(),
            "bounded mechanical W20 assertions; general consolidation/Workflow/Handoff Recall not qualified".into(),
            "lexical discovery is not completeness; no implicit encoder/model call".into()],
    };
    trace.trace_id = format!("recall-trace:{}", digest(&trace));
    let encoded = serde_json::to_string(&trace).map_err(|e| e.to_string())?;
    metrics.semantic_units = encoded.chars().count().div_ceil(4).max(1);
    metrics.output_bytes = encoded.len();
    if trace.events.len() > trace.request.bounds.events
        || trace.relations.len() > trace.request.bounds.relations
        || trace.segments.len() > trace.request.bounds.segments
        || metrics.output_bytes > trace.request.bounds.bytes
        || metrics.semantic_units > trace.request.bounds.semantic_units
    {
        return Err("recall_required_context_exceeds_output_budget".into());
    }
    Ok(RecallResult {
        trace,
        measurements: metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_hierarchy::{
        EpistemicClass, SemanticLifecycle, SemanticSubject, SemanticValue,
    };

    #[test]
    fn recall_mechanical_supersession_is_mandatory_and_contradiction_is_not_replacement() {
        let build = |generation, digest: &str, class| {
            SemanticMemoryAssertion::build(
                "case:recall",
                SemanticSubject::ResourceAttachment("resource:exact".into()),
                "resource.content_digest".into(),
                SemanticValue::Digest(digest.into()),
                class,
                vec![SemanticSupportRef::Transition(format!(
                    "transition:{generation}"
                ))],
                generation,
                generation,
                "contract-test".into(),
                vec!["participant:one".into()],
            )
            .unwrap()
        };
        let mut assertions = vec![
            build(1, "sha256:old", EpistemicClass::MechanicallyGrounded),
            build(10, "sha256:new", EpistemicClass::MechanicallyGrounded),
        ];
        let sets = derive_contradictions(&mut assertions).unwrap();
        let sources = assertions
            .iter()
            .map(|a| {
                (
                    a.assertion_id.clone(),
                    vec![format!("transition:{}", a.source_generation_end)],
                )
            })
            .collect();
        let mut selected = Selection::from([(
            "transition:1".into(),
            BTreeSet::from([SelectionReason::LexicalCandidate]),
        )]);
        assert!(close_selection(&mut selected.clone(), &assertions, &sources, &sets, 1).is_err());
        close_selection(&mut selected, &assertions, &sources, &sets, 2).unwrap();
        assert_eq!(selected.len(), 2);
        assert!(selected["transition:10"].contains(&SelectionReason::SupersessionChain));
        assert_eq!(assertions[0].lifecycle, SemanticLifecycle::Superseded);
        assert_eq!(assertions[1].lifecycle, SemanticLifecycle::Active);
        assert_eq!(
            assertions[0].supersession_refs,
            vec![assertions[1].assertion_id.clone()]
        );
        let mut earlier = vec![build(1, "sha256:old", EpistemicClass::MechanicallyGrounded)];
        assert!(derive_contradictions(&mut earlier).unwrap().is_empty());
        assert_eq!(earlier[0].lifecycle, SemanticLifecycle::Active);
        println!("recall_mechanical_supersession old=historical new=active tiny_budget=refused asof_old=active score_does_not_choose_validity=true");

        // A repeated lexical/vector candidate cannot re-root previously expanded
        // nodes and walk an arbitrarily longer path through successive searches.
        use crate::graph::experience::{RelationPosture, RelationSource};
        let relations: Vec<_> = (0..6)
            .map(|n| ExperienceRelation {
                relation_id: format!("relation:{n}"),
                from_event: format!("e{n}"),
                to_event: format!("e{}", n + 1),
                kind: RelationKind::OperationDecision,
                posture: RelationPosture::StructuralReference,
                known_at_generation: n + 2,
                sources: vec![RelationSource {
                    transition_id: format!("e{}", n + 1),
                    field: "operation_id".into(),
                }],
            })
            .collect();
        let bounds = RecallBounds {
            expansion_depth: 2,
            ..Default::default()
        };
        let mut union = Selection::new();
        for _ in 0..16 {
            let seed = Selection::from([(
                "e0".into(),
                BTreeSet::from([SelectionReason::LexicalCandidate]),
            )]);
            let (group, limited) =
                resolve_group(seed, &[], &BTreeMap::new(), &[], &relations, &bounds).unwrap();
            assert!(limited);
            for (id, reasons) in group {
                union.entry(id).or_default().extend(reasons);
            }
        }
        assert_eq!(
            union.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["e0", "e1", "e2"]
        );
        println!("recall_expansion repeated_candidates=16 hops=2 selected_nodes=3 no_cumulative_depth_reset=true");
    }
}
