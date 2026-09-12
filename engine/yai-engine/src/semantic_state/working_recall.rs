//! Recall composition in the existing W compiler, not another retriever or owner.
use super::*;
use crate::memory_hierarchy::recall::{self, RecallBounds, RecallRequest, RecallTrace};

pub const RECALL_WORKING_SCHEMA: &str = "yai.semantic_working_state.v3";
pub const RECALL_COMPILER_VERSION: &str = "yai.state_compiler.v3";

/// One execution request. Textual intent mechanically supplies the Recall query;
/// explicit historical dependencies are separate from current S requirements.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkingStateRequest {
    pub case_id: String,
    pub expected_generation: u64,
    pub compilation: CompilationRequest,
    pub at: Option<historical::HistoricalCoordinate>,
    pub recall_required_refs: Vec<String>,
    pub recall_bounds: RecallBounds,
    pub max_output_bytes: usize,
}

impl WorkingStateRequest {
    pub fn recall_request(&self) -> Result<RecallRequest, String> {
        if self.max_output_bytes == 0 || self.max_output_bytes > 4 * 1024 * 1024 {
            return Err("working_state_output_bound_invalid".into());
        }
        let mut r = RecallRequest::integrated(
            &self.case_id,
            self.expected_generation,
            &self.compilation.scope.participant_id,
            &self.compilation.intent,
        );
        if let Some(at) = &self.at {
            r.at = at.clone();
        }
        r.required_refs = self.recall_required_refs.clone();
        r.bounds = self.recall_bounds.clone();
        r.validate()?;
        Ok(r)
    }
}

/// An indivisible semantic group, qualified by Recall itself. This is neither
/// its candidate list nor a serialized trace/prompt. Nested epistemic classes,
/// source coordinates, historical validity and relation semantics stay typed.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecalledEvidence {
    pub events: Vec<recall::RecallEvent>,
    pub relations: Vec<crate::graph::experience::ExperienceRelation>,
    pub segments: Vec<recall::RecallSegment>,
    pub assertions: Vec<recall::RecallAssertion>,
    pub contradictions: Vec<crate::memory_hierarchy::SemanticContradictionSet>,
    pub documentary: Option<recall::documentary::DocumentaryTrace>,
    pub source_closure: Vec<historical::SourceClosure>,
    pub closure_complete: bool,
}

impl From<RecallTrace> for RecalledEvidence {
    fn from(t: RecallTrace) -> Self {
        Self {
            events: t.events,
            relations: t.relations,
            segments: t.segments,
            assertions: t.assertions,
            contradictions: t.contradictions,
            documentary: t.documentary,
            source_closure: t.source_closure,
            closure_complete: t.closure_complete,
        }
    }
}

impl RecalledEvidence {
    pub(super) fn matches(&self, reference: &str) -> bool {
        self.events.iter().any(|e| {
            e.event.transition_id == reference || e.event.object_refs.iter().any(|r| r == reference)
        }) || self
            .source_closure
            .iter()
            .any(|s| s.source_ref == reference)
            || self
                .assertions
                .iter()
                .any(|a| a.assertion.assertion_id == reference)
            || self.documentary.as_ref().is_some_and(|d| {
                d.sources.iter().any(|s| {
                    [
                        &s.source.id,
                        &s.source.source_id,
                        &s.source.revision_id,
                        &s.source.resource_id,
                    ]
                    .iter()
                    .any(|r| r.as_str() == reference)
                }) || d
                    .units
                    .iter()
                    .any(|u| u.unit.id == reference || u.unit.entity.as_deref() == Some(reference))
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkingRecall {
    pub request: WorkingStateRequest,
    pub recall_id: String,
    pub recall_request: RecallRequest,
    pub disclosure_digest: String,
    pub qualified_source_digest: String,
    pub recall_closure_complete: bool,
    pub omitted_recall_candidates: usize,
    pub expansion_stopped_at_depth: bool,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QualifiedRecall {
    pub metadata: WorkingRecall,
    pub groups: Vec<(bool, SemanticEntry)>,
    pub excluded_transitions: BTreeSet<String>,
}

impl QualifiedRecall {
    pub(crate) fn new(
        request: WorkingStateRequest,
        result: &recall::RecallResult,
        excluded_transitions: BTreeSet<String>,
    ) -> Result<Self, String> {
        let t = &result.trace;
        if t.schema != recall::RECALL_TRACE_V2
            || t.request.case_id != request.case_id
            || t.request.expected_generation != request.expected_generation
            || t.request.participant_id != request.compilation.scope.participant_id
        {
            return Err("working_recall_scope_mismatch".into());
        }
        let mut groups: BTreeMap<String, (bool, SemanticEntry)> = BTreeMap::new();
        // Preserve resolver order; do not compare family ranking scores in W.
        let mut order = Vec::new();
        let mut semantic_groups = BTreeMap::new();
        for (required, trace) in &result.compilation_groups {
            let evidence = RecalledEvidence::from(trace.clone());
            if *required && !evidence.closure_complete {
                return Err("working_required_recall_backing_unavailable".into());
            }
            // Multiple lexical hits can resolve to the same atomic conflict /
            // source group. Keep its first qualified selection explanation,
            // not one copy per ranking hit. No new discovery or truth decision.
            let mut membership: Vec<String> = evidence
                .events
                .iter()
                .map(|e| e.event.transition_id.clone())
                .chain(
                    evidence
                        .assertions
                        .iter()
                        .map(|a| a.assertion.assertion_id.clone()),
                )
                .chain(evidence.source_closure.iter().map(|s| s.source_ref.clone()))
                .collect();
            if let Some(d) = &evidence.documentary {
                membership.extend(d.units.iter().map(|u| u.unit.id.clone()));
            }
            membership.sort();
            membership.dedup();
            let key = identity(&(
                &membership,
                &evidence.relations,
                &evidence.contradictions,
                &evidence.source_closure,
                evidence
                    .documentary
                    .as_ref()
                    .map(|d| (&d.relations, &d.cross_references, &d.contradictions)),
            ))?;
            let id = semantic_groups
                .entry(key)
                .or_insert(format!("recalled-group:{}", identity(&evidence)?))
                .clone();
            if let Some((mandatory, _)) = groups.get_mut(&id) {
                *mandatory |= required;
                continue;
            }
            order.push(id.clone());
            groups.insert(
                id.clone(),
                (
                    *required,
                    SemanticEntry {
                        entry_id: id,
                        posture: AuthorityPosture::DerivedMemory,
                        provenance: vec![SemanticProvenance {
                            kind: ProvenanceKind::RecallTrace,
                            source_ref: t.trace_id.clone(),
                        }],
                        value: SemanticValue::RecalledEvidence {
                            evidence: Box::new(evidence),
                        },
                    },
                ),
            );
        }
        Ok(Self {
            metadata: WorkingRecall {
                request,
                recall_id: t.trace_id.clone(),
                recall_request: t.request.clone(),
                disclosure_digest: t.disclosure_digest.clone(),
                qualified_source_digest: t.qualified_source_digest.clone(),
                recall_closure_complete: t.closure_complete,
                omitted_recall_candidates: t.omitted_candidates,
                expansion_stopped_at_depth: t.expansion_stopped_at_depth,
                limitations: t.limitations.clone(),
            },
            groups: order
                .into_iter()
                .map(|id| groups.remove(&id).unwrap())
                .collect(),
            excluded_transitions,
        })
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct WorkingStateMeasurements {
    pub recall: recall::RecallMeasurements,
    pub current_composition_us: u128,
    pub group_provenance_us: u128,
    pub working_compilation_us: u128,
    pub output_bytes: usize,
    pub selected_semantic_units: usize,
}

/// Request-scoped capability: source is not deserializable. Lowering consumes
/// this already qualified input without another store read or memory search.
#[derive(Clone, Debug, Serialize)]
pub struct QualifiedWorkingState {
    pub working_state: SemanticWorkingState,
    pub compilation_mode: DeltaCompilationMode,
    pub measurements: WorkingStateMeasurements,
    #[serde(skip)]
    pub(crate) source: SemanticState,
}

impl QualifiedWorkingState {
    pub fn lower_context(&self) -> Result<crate::context::Projection, String> {
        self.working_state
            .lower_context(&self.source, self.working_state.request())
    }
}

impl SemanticState {
    /// Current requirements are not scored by Recall. Only references not
    /// supplied by current owners become mandatory Recall dependencies.
    pub(crate) fn recalled_requirements(
        &self,
        request: &CompilationRequest,
    ) -> Result<Vec<String>, String> {
        let current = self.candidates_mode(request, true)?;
        Ok(request
            .required_refs
            .iter()
            .filter(|r| !current.entries.iter().any(|e| entry_matches(e, r)))
            .cloned()
            .collect())
    }

    pub(crate) fn with_recall(mut self, input: QualifiedRecall) -> Result<Self, String> {
        let request = input.metadata.request.compilation.clone();
        self.recall = Some(input);
        // Scope-qualified semantics, not a private full-history digest. Equal
        // hidden/absent candidate universes cannot be distinguished by W ID.
        self.source_id = format!(
            "semantic-working-basis:{}",
            identity(&(
                RECALL_COMPILER_VERSION,
                self.case_id(),
                self.generation(),
                &request.scope,
                self.candidates(&request)?.entries,
                &self.recall.as_ref().unwrap().metadata,
            ))?
        );
        Ok(self)
    }
}

impl SemanticWorkingState {
    pub fn recall(&self) -> Option<&WorkingRecall> {
        self.recall.as_ref()
    }

    pub(super) fn fit_recall_envelope(&mut self) -> Result<(), String> {
        let max_bytes = self.recall.as_ref().unwrap().request.max_output_bytes;
        loop {
            for entry in &mut self.entries {
                if let SemanticValue::RecallQualification {
                    working_omitted_items,
                    ..
                } = &mut entry.value
                {
                    *working_omitted_items = self.bounds.omitted_items;
                }
            }
            // Fixed-size digest placeholder charges final representation size;
            // measurements and optional access caches are not part of identity.
            self.working_state_id = format!("working-state:{}", "0".repeat(identity(&())?.len()));
            let mut units = self.bounds.selected_semantic_units;
            let mut bytes = 0;
            for _ in 0..8 {
                self.bounds.selected_semantic_units = units;
                let encoded = serde_json::to_string(self).map_err(|e| e.to_string())?;
                bytes = encoded.len();
                let next = encoded.chars().count().div_ceil(4);
                if next == units {
                    break;
                }
                units = next;
            }
            if units <= self.request.max_semantic_units && bytes <= max_bytes {
                self.working_state_id.clear();
                return Ok(());
            }
            let Some(id) = self
                .decisions
                .iter()
                .rev()
                .find(|d| {
                    d.disposition != crate::residency::ResidencyDisposition::Pinned
                        && self.entries.iter().any(|e| e.entry_id == d.item_id)
                })
                .map(|d| d.item_id.clone())
            else {
                return Err("working_output_budget_below_mandatory_state".into());
            };
            self.entries.retain(|e| e.entry_id != id);
            let d = self.decisions.iter_mut().find(|d| d.item_id == id).unwrap();
            d.disposition = crate::residency::ResidencyDisposition::Omitted;
            d.reasons
                .push("atomic_group_omitted_by_final_working_envelope".into());
            self.bounds.selected_items = self.entries.len();
            self.bounds.omitted_items += 1;
            self.bounds.omitted_by_budget += 1;
        }
    }
}
