//! Recall composition in the existing W compiler, not another retriever or owner.
use super::*;
use crate::memory_hierarchy::recall::{self, RecallBounds, RecallRequest, RecallTrace};

pub const RECALL_WORKING_SCHEMA: &str = "yai.semantic_working_state.v3";
pub const RECALL_COMPILER_VERSION: &str = "yai.state_compiler.v3";

pub const REFRESH_REQUEST_SCHEMA: &str = "yai.working_refresh_request.v1";
pub const REFRESH_RESULT_SCHEMA: &str = "yai.working_refresh_result.v1";
pub const AMBIENT_REFRESH_REQUEST_SCHEMA: &str = "yai.ambient_refresh_request.v1";
pub const AMBIENT_REFRESH_RESULT_SCHEMA: &str = "yai.ambient_refresh_result.v1";

/// Explicit envelope adjustment, not a change of task, exact anchors or scope.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkingRefreshBudget {
    pub max_items: usize,
    pub max_semantic_units: usize,
    pub max_output_bytes: usize,
}

/// No prompt/intent field. A different task must enter through compilation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkingRefreshRequest {
    pub schema: String,
    pub case_id: String,
    pub participant_id: String,
    pub base_working_state_id: String,
    pub budget: Option<WorkingRefreshBudget>,
}

impl WorkingRefreshRequest {
    pub fn new(base: &SemanticWorkingState) -> Self {
        Self { schema: REFRESH_REQUEST_SCHEMA.into(), case_id: base.case_id.clone(),
            participant_id: base.participant_id.clone(), base_working_state_id: base.id().into(), budget: None }
    }

    pub(crate) fn recover_request(&self, base: &SemanticWorkingState) -> Result<WorkingStateRequest, String> {
        base.validate_refresh_envelope()?;
        if self.schema != REFRESH_REQUEST_SCHEMA || self.case_id != base.case_id
            || self.participant_id != base.participant_id || self.base_working_state_id != base.id() {
            return Err("working_refresh_request_or_base_mismatch".into());
        }
        let mut request = base.recall.as_ref().ok_or("working_recall_required")?.request.clone();
        if let Some(budget) = &self.budget {
            if budget.max_items == 0 || budget.max_semantic_units == 0 {
                return Err("working_refresh_budget_invalid".into());
            }
            request.compilation.scope.max_items = budget.max_items;
            request.compilation.max_semantic_units = budget.max_semantic_units;
            request.max_output_bytes = budget.max_output_bytes;
        }
        request.recall_request()?;
        Ok(request)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RefreshPosture { Unchanged, RecompiledEquivalent, Refreshed, Incomplete }

/// Comparison of supplied predecessor and CURRENT qualified result. Removed
/// identities/counts are deliberately absent: lost disclosure must not become
/// an explanation channel. This is not a dependency database or freshness lease.
#[derive(Clone, Debug, Serialize)]
pub struct RefreshAssessment {
    pub identity_equal: bool,
    pub selected_material_equal: bool,
    pub current_control_equal: bool,
    pub recall_identity_equal: bool,
    pub requires_replacement: bool,
    pub historical_cut_pinned: bool,
    pub qualification: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct WorkingRefreshResult {
    pub schema: String,
    pub request: WorkingRefreshRequest,
    pub posture: RefreshPosture,
    pub assessment: RefreshAssessment,
    pub working_state: SemanticWorkingState,
    pub compilation_mode: DeltaCompilationMode,
    pub measurements: WorkingStateMeasurements,
    pub paging_recompilation_us: u128,
    #[serde(skip)]
    pub(crate) source: SemanticState,
}

impl WorkingRefreshResult {
    pub fn lower_context(&self) -> Result<crate::context::Projection, String> {
        self.working_state.lower_context(&self.source, self.working_state.request())
    }

    pub(crate) fn qualified(base: &SemanticWorkingState, request: WorkingRefreshRequest,
        current: QualifiedWorkingState, paging_recompilation_us: u128) -> Result<Self, String> {
        let w = &current.working_state;
        let identity_equal = base.id() == w.id();
        let selected_material_equal = refresh_material(base, false)? == refresh_material(w, false)?;
        let assessment = RefreshAssessment { identity_equal, selected_material_equal,
            current_control_equal: refresh_material(base, true)? == refresh_material(w, true)?,
            recall_identity_equal: base.recall.as_ref().map(|r| &r.recall_id) == w.recall.as_ref().map(|r| &r.recall_id),
            requires_replacement: !identity_equal,
            historical_cut_pinned: w.recall.as_ref().is_some_and(|r| r.request.at.is_some()),
            qualification: "current_snapshot_full_requalification_before_use; no_authority_from_prior_artifact; relevance_rebuilt_not_delta_applied".into() };
        let posture = if w.recall.as_ref().is_some_and(|r| !r.recall_closure_complete) {
            RefreshPosture::Incomplete
        } else if identity_equal { RefreshPosture::Unchanged }
        else if selected_material_equal { RefreshPosture::RecompiledEquivalent }
        else { RefreshPosture::Refreshed };
        Ok(Self { schema: REFRESH_RESULT_SCHEMA.into(), request, posture, assessment,
            working_state: current.working_state, compilation_mode: current.compilation_mode,
            measurements: current.measurements, paging_recompilation_us, source: current.source })
    }
}

/// Presentation-independent identity of the still-live semantic consumer. The
/// reference is lineage only; it grants no access and is rechecked by the
/// Conversation/Workflow application owner before this engine operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveSemanticConsumerKind {
    Conversation,
    Workflow,
}

/// A bounded notification from an existing canonical/source/control owner.
/// These signals explain why freshness is being assessed; they never prove
/// freshness or authorize the resulting W. Full current requalification below
/// remains the decision oracle.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmbientSemanticChangeKind {
    CanonicalTransition,
    SourceQualification,
    AuthorityOrDisclosure,
    BackingAvailability,
    ConsumerRecovery,
    Other,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmbientSemanticChange {
    pub kind: AmbientSemanticChangeKind,
    /// Exact owner-produced identity when it is already visible to the caller.
    /// It is never resolved as authority and is not returned on refusal.
    pub reference: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmbientRefreshRequest {
    pub schema: String,
    pub consumer: ActiveSemanticConsumerKind,
    pub consumer_ref: String,
    pub case_id: String,
    pub participant_id: String,
    pub base_working_state_id: String,
    pub task_id: String,
    pub changes: Vec<AmbientSemanticChange>,
}

impl AmbientRefreshRequest {
    pub fn new(
        base: &SemanticWorkingState,
        consumer: ActiveSemanticConsumerKind,
        consumer_ref: impl Into<String>,
        changes: Vec<AmbientSemanticChange>,
    ) -> Result<Self, String> {
        let request = Self {
            schema: AMBIENT_REFRESH_REQUEST_SCHEMA.into(),
            consumer,
            consumer_ref: consumer_ref.into(),
            case_id: base.case_id.clone(),
            participant_id: base.participant_id.clone(),
            base_working_state_id: base.id().into(),
            task_id: base.semantic_task_id()?,
            changes,
        };
        request.validate(base)?;
        Ok(request)
    }

    pub(crate) fn validate(&self, base: &SemanticWorkingState) -> Result<(), String> {
        base.validate_refresh_envelope()?;
        let unique: BTreeSet<_> = self.changes.iter().collect();
        if self.schema != AMBIENT_REFRESH_REQUEST_SCHEMA
            || self.case_id != base.case_id
            || self.participant_id != base.participant_id
            || self.base_working_state_id != base.id()
            || self.task_id != base.semantic_task_id()?
            || self.consumer_ref.is_empty()
            || self.consumer_ref.len() > 512
            || self.changes.is_empty()
            || self.changes.len() > 64
            || unique.len() != self.changes.len()
            || self
                .changes
                .iter()
                .any(|change| change.reference.is_empty() || change.reference.len() > 512)
        {
            return Err("ambient_refresh_request_or_base_mismatch".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AmbientFreshness {
    Fresh,
    RefreshRequired,
    Invalidated,
}

#[derive(Clone, Debug, Serialize)]
pub struct AmbientRefreshMeasurements {
    pub coalesced_changes: usize,
    pub current_requalification_us: u128,
}

/// Derived active-consumer posture. `refresh` contains the one current
/// WorkingRefresh result when requalification succeeded. An invalidation never
/// exports hidden object identities or the internal refusal detail.
#[derive(Debug, Serialize)]
pub struct AmbientRefreshResult {
    pub schema: String,
    pub request: AmbientRefreshRequest,
    pub freshness: AmbientFreshness,
    pub reason: String,
    pub task_preserved: bool,
    pub current_working_state_id: Option<String>,
    pub current_recall_id: Option<String>,
    pub refresh: Option<WorkingRefreshResult>,
    pub measurements: AmbientRefreshMeasurements,
}

impl AmbientRefreshResult {
    pub(crate) fn qualified(
        base: &SemanticWorkingState,
        request: AmbientRefreshRequest,
        refresh: WorkingRefreshResult,
        current_requalification_us: u128,
    ) -> Result<Self, String> {
        let coalesced_changes = request.changes.len();
        let task_preserved = base.semantic_task_id()? == refresh.working_state.semantic_task_id()?;
        if !task_preserved {
            return Err("ambient_refresh_task_changed".into());
        }
        let freshness = if refresh.assessment.identity_equal {
            AmbientFreshness::Fresh
        } else {
            AmbientFreshness::RefreshRequired
        };
        let reason = match freshness {
            AmbientFreshness::Fresh => "current_qualified_basis_equal".into(),
            AmbientFreshness::RefreshRequired => {
                "current_qualified_basis_changed_replacement_ready".into()
            }
            AmbientFreshness::Invalidated => unreachable!(),
        };
        Ok(Self {
            schema: AMBIENT_REFRESH_RESULT_SCHEMA.into(),
            request,
            freshness,
            reason,
            task_preserved,
            current_working_state_id: Some(refresh.working_state.id().into()),
            current_recall_id: refresh
                .working_state
                .recall()
                .map(|recall| recall.recall_id.clone()),
            measurements: AmbientRefreshMeasurements {
                coalesced_changes,
                current_requalification_us,
            },
            refresh: Some(refresh),
        })
    }

    pub(crate) fn invalidated(
        request: AmbientRefreshRequest,
        current_requalification_us: u128,
    ) -> Self {
        let coalesced_changes = request.changes.len();
        Self {
            schema: AMBIENT_REFRESH_RESULT_SCHEMA.into(),
            request,
            freshness: AmbientFreshness::Invalidated,
            reason: "current_authority_source_or_required_backing_refused".into(),
            task_preserved: true,
            current_working_state_id: None,
            current_recall_id: None,
            refresh: None,
            measurements: AmbientRefreshMeasurements {
                coalesced_changes,
                current_requalification_us,
            },
        }
    }
}

fn refresh_material(w: &SemanticWorkingState, control_only: bool) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for e in &w.entries {
        match &e.value {
            SemanticValue::RecallQualification { .. } | SemanticValue::SemanticPageReferences { .. } => {},
            SemanticValue::RecalledEvidence { evidence } if !control_only => {
                out.push(identity(&paging::normalize((**evidence).clone()))?);
            },
            SemanticValue::RecalledEvidence { .. } => {},
            _ => out.push(identity(e)?),
        }
    }
    out.sort();
    Ok(out)
}

impl SemanticWorkingState {
    /// Stable task/objective identity across generations and qualification
    /// bases. Budgets, provider selection and current authority are deliberately
    /// excluded: changing them requires requalification, not a new objective.
    pub fn semantic_task_id(&self) -> Result<String, String> {
        self.validate_refresh_envelope()?;
        let request = &self.recall.as_ref().ok_or("working_recall_required")?.request;
        Ok(format!(
            "semantic-task:{}",
            identity(&(
                &request.case_id,
                &request.compilation.scope.participant_id,
                &request.compilation.scope.purpose,
                &request.compilation.scope.consumer,
                &request.compilation.scope.view_kind,
                &request.compilation.intent,
                &request.recall_query,
                &request.at,
                &request.recall_required_refs,
                &request.compilation.resource_refs,
                &request.compilation.required_refs,
                &request.compilation.output_contract_id,
            ))?
        ))
    }

    pub(crate) fn validate_refresh_envelope(&self) -> Result<(), String> {
        if self.paging.is_some() { return self.validate_paging_envelope(); }
        let recall = self.recall.as_ref().ok_or("working_recall_required")?;
        recall.request.recall_request()?;
        let mut copy = self.clone(); copy.working_state_id.clear();
        if self.schema != RECALL_WORKING_SCHEMA || self.compiler != RECALL_COMPILER_VERSION
            || self.request != recall.request.compilation || self.case_id != recall.request.case_id
            || self.case_generation != recall.request.expected_generation
            || self.participant_id != self.request.scope.participant_id
            || self.id() != format!("working-state:{}", identity(&copy)?) {
            return Err("working_refresh_base_integrity_mismatch".into());
        }
        Ok(())
    }
}

/// One execution request. Textual intent mechanically supplies the Recall query;
/// explicit historical dependencies are separate from current S requirements.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkingStateRequest {
    pub case_id: String,
    pub expected_generation: u64,
    pub compilation: CompilationRequest,
    /// Mechanical query from the exact current input, when distinct from the
    /// execution instruction. Bound into W provenance/refresh, not a second
    /// copy of immediate input in compatibility lowering. Old requests retain
    /// their intent-as-query meaning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recall_query: Option<String>,
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
            self.recall_query.as_deref().unwrap_or(&self.compilation.intent),
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
                || e.resource_reference.as_deref() == Some(reference)
                || e.content_reference.as_ref().is_some_and(|c|
                    c.admission_id == reference || c.object_id == reference || c.source_resource_id == reference)
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
        if let Some(paging) = &self.paging {
            self.source_id = format!("semantic-paging-basis:{}", identity(&(&self.source_id, &paging.metadata))?);
        }
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
            let resident = self.resident_page_references();
            for entry in &mut self.entries {
                if let SemanticValue::SemanticPageReferences { resident_references, .. } = &mut entry.value {
                    *resident_references = resident.clone();
                }
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
