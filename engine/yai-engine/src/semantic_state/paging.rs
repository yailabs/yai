//! Exact semantic group paging in the existing W compiler. References are
//! locators, never authorization or a persistent working-memory owner.
use super::*;
use super::working_recall::{QualifiedRecall, QualifiedWorkingState, RecalledEvidence};
use crate::memory_hierarchy::recall::{RecallBounds, RecallRequest, RecallTrace};

pub const PAGED_WORKING_SCHEMA: &str = "yai.semantic_working_state.v4";
pub const PAGED_COMPILER: &str = "yai.state_compiler.v4";
pub const PAGE_SCHEMA: &str = "yai.semantic_page.v1";
pub const PAGE_REQUEST_SCHEMA: &str = "yai.semantic_page_request.v1";
pub const REFERENCE_PROFILE: &str = "yai.exact_semantic_group.v1";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct PageSource {
    pub source_id: String,
    pub revision_id: String,
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticReference {
    pub reference_id: String,
    pub profile: String,
    pub group_entry_id: String,
    pub family: String,
    pub at: historical::HistoricalCoordinate,
    /// Exact events/units/source descriptors, NOT a relevance query or cursor.
    pub members: BTreeSet<String>,
    pub sources: BTreeSet<PageSource>,
    pub evidence_digest: String,
    pub mandatory_task_dependency: bool,
}

/// Access segmentation/relevance reasons are not semantic backing. Rehydration
/// independently rebuilds these representations without claiming new discovery.
pub(crate) fn normalize(mut e: RecalledEvidence) -> RecalledEvidence {
    for v in &mut e.events { v.reasons.clear(); }
    for v in &mut e.assertions { v.reasons.clear(); }
    e.segments.clear();
    if let Some(d) = &mut e.documentary {
        for s in &mut d.sources { s.reasons.clear(); }
        for u in &mut d.units { u.reasons.clear(); }
        d.segments.clear();
    }
    e
}

impl SemanticReference {
    fn from_group(entry: &SemanticEntry, required: bool, at: &historical::HistoricalCoordinate)
        -> Result<Option<Self>, String> {
        let SemanticValue::RecalledEvidence { evidence } = &entry.value else { return Ok(None) };
        if !evidence.closure_complete { return Ok(None); }
        let mut members: BTreeSet<_> = evidence.events.iter().map(|e| e.event.transition_id.clone()).collect();
        let mut sources = BTreeSet::new();
        if let Some(d) = &evidence.documentary {
            members.extend(d.units.iter().map(|u| u.unit.id.clone()));
            members.extend(d.sources.iter().map(|s| s.source.id.clone()));
            sources.extend(d.sources.iter().map(|s| PageSource { source_id: s.source.source_id.clone(),
                revision_id: s.source.revision_id.clone(), path: s.source.path.clone() }));
        }
        if members.is_empty() || members.len() > 256 { return Ok(None); }
        let mut r = Self { reference_id: String::new(), profile: REFERENCE_PROFILE.into(),
            group_entry_id: entry.entry_id.clone(),
            family: if sources.is_empty() { "experience" } else if evidence.events.is_empty() { "documentary" } else { "documentary_experience" }.into(),
            at: at.clone(), members, sources,
            evidence_digest: identity(&normalize((**evidence).clone()))?, mandatory_task_dependency: required };
        r.reference_id = format!("semantic-reference:{}", identity(&r)?);
        Ok(Some(r))
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let mut copy = self.clone(); copy.reference_id.clear();
        if self.profile != REFERENCE_PROFILE || self.members.is_empty() || self.members.len() > 256
            || self.sources.len() > 256 || self.members.iter().any(|id| id.len() > 512)
            || self.reference_id != format!("semantic-reference:{}", identity(&copy)?) {
            return Err("semantic_reference_unavailable".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkingPaging {
    pub profile: String,
    pub origin_working_state_id: String,
    pub origin_recall_id: String,
    pub catalog_digest: String,
    pub current_material_ids: BTreeSet<String>,
    pub current_control_digest: String,
    pub nonpageable_groups: usize,
    pub parent_working_state_id: Option<String>,
    pub last_request: Option<PageRequest>,
    pub page_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QualifiedPaging {
    pub metadata: WorkingPaging,
    pub references: Vec<SemanticReference>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageAction { PageIn, PageOut }

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PageRequest {
    pub schema: String,
    pub case_id: String,
    pub participant_id: String,
    pub base_working_state_id: String,
    pub references: Vec<String>,
    pub action: PageAction,
    /// Reuses qualified semantic item/relation/unit/byte bounds. Candidate count
    /// is not used: exact paging does not discover candidates.
    pub bounds: RecallBounds,
}

impl PageRequest {
    pub fn new(base: &SemanticWorkingState, references: Vec<String>) -> Self {
        Self { schema: PAGE_REQUEST_SCHEMA.into(), case_id: base.case_id.clone(),
            participant_id: base.participant_id.clone(), base_working_state_id: base.id().into(),
            references, action: PageAction::PageIn, bounds: RecallBounds::default() }
    }
    pub(crate) fn validate(&self, base: &SemanticWorkingState) -> Result<(), String> {
        let mut r = RecallRequest::integrated(&self.case_id, base.case_generation, &self.participant_id, "exact semantic group");
        r.bounds = self.bounds.clone(); r.validate()?;
        if self.schema != PAGE_REQUEST_SCHEMA || self.case_id != base.case_id
            || self.participant_id != base.participant_id || self.base_working_state_id != base.id()
            || self.references.is_empty() || self.references.len() > 16
            || self.references.iter().any(|r| r.len() > 512)
            || self.references.iter().collect::<BTreeSet<_>>().len() != self.references.len() {
            return Err("semantic_page_request_or_base_mismatch".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticPage {
    pub schema: String,
    pub page_id: String,
    pub request: PageRequest,
    pub current_control_digest: String,
    pub groups: Vec<RecalledEvidence>,
    pub closure: String,
    pub semantic_units: usize,
}

impl SemanticPage {
    pub(crate) fn build(request: PageRequest, control: String, groups: Vec<RecalledEvidence>) -> Result<Self, String> {
        if groups.iter().any(|g| !g.closure_complete) { return Err("semantic_page_backing_unavailable".into()); }
        let mut out = Self { schema: PAGE_SCHEMA.into(), page_id: String::new(), request,
            current_control_digest: control, groups,
            closure: "complete_exact_group_scope_not_global_task_sufficiency".into(), semantic_units: 0 };
        let items: usize = out.groups.iter().map(|g| g.events.len() + g.documentary.as_ref().map_or(0, |d| d.units.len() + d.sources.len())).sum();
        let relations: usize = out.groups.iter().map(|g| g.relations.len() + g.documentary.as_ref().map_or(0, |d| d.relations.len() + d.cross_references.len())).sum();
        let segments: usize = out.groups.iter().map(|g| g.segments.len() + g.documentary.as_ref().map_or(0, |d| d.segments.len())).sum();
        out.page_id = format!("semantic-page:{}", "0".repeat(identity(&())?.len()));
        for _ in 0..8 {
            let encoded = serde_json::to_string(&out).map_err(|e| e.to_string())?;
            let units = encoded.chars().count().div_ceil(4);
            if units == out.semantic_units { break; }
            out.semantic_units = units;
        }
        if items > out.request.bounds.events || relations > out.request.bounds.relations
            || segments > out.request.bounds.segments || out.semantic_units > out.request.bounds.semantic_units
            || serde_json::to_vec(&out).map_err(|e| e.to_string())?.len() > out.request.bounds.bytes {
            return Err("semantic_page_atomic_closure_exceeds_budget".into());
        }
        out.page_id.clear(); out.page_id = format!("semantic-page:{}", identity(&out)?);
        Ok(out)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct PagingMeasurements {
    pub qualified_history_us: u128,
    pub exact_source_resolution_us: u128,
    pub source_derivation_us: u128,
    pub exact_group_resolution_us: u128,
    pub relation_build_us: u128,
    pub page_closure_assembly_us: u128,
    pub working_recompilation_us: u128,
    pub source_documents_resolved: usize,
    pub groups_revalidated: usize,
    pub candidate_discovery_passes: usize,
    pub source_bytes: u64,
    pub page_bytes: usize,
    pub working_bytes: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct PageResult {
    pub page: SemanticPage,
    pub working_state: SemanticWorkingState,
    pub evicted_references: Vec<String>,
    pub measurements: PagingMeasurements,
    #[serde(skip)]
    pub(crate) source: SemanticState,
}
impl PageResult {
    pub fn lower_context(&self) -> Result<crate::context::Projection, String> {
        self.working_state.lower_context(&self.source, self.working_state.request())
    }
}

fn is_current(e: &SemanticEntry) -> bool {
    !matches!(e.value, SemanticValue::RecalledEvidence { .. } | SemanticValue::RecallQualification { .. }
        | SemanticValue::SemanticPageReferences { .. })
}
fn control(entries: &[SemanticEntry]) -> Vec<&SemanticEntry> {
    entries.iter().filter(|e| is_current(e) && matches!(e.posture, AuthorityPosture::ControlState | AuthorityPosture::CommittedOperationalFact | AuthorityPosture::Unresolved)).collect()
}

impl QualifiedWorkingState {
    /// Opt-in derived v4; serialized v3 meaning and ordinary provider paths stay
    /// unchanged. Only already-qualified, source-closed groups become locators.
    pub(crate) fn enable_paging(mut self) -> Result<Self, String> {
        let mut r = self.source.recall.clone().ok_or("paging_requires_recall_working_state")?;
        let mut references = Vec::new();
        let mut nonpageable = 0;
        for (required, e) in &r.groups {
            if let Some(reference) = SemanticReference::from_group(e, *required, &r.metadata.recall_request.at)? {
                references.push(reference);
            } else {
                if *required { return Err("semantic_paging_required_group_not_pageable".into()); }
                nonpageable += 1;
            }
        }
        let metadata = WorkingPaging { profile: REFERENCE_PROFILE.into(),
            origin_working_state_id: self.working_state.id().into(), origin_recall_id: r.metadata.recall_id.clone(),
            catalog_digest: identity(&references)?,
            current_material_ids: self.working_state.entries.iter().filter(|e| is_current(e)).map(|e| e.entry_id.clone()).collect(),
            current_control_digest: identity(&control(&self.working_state.entries))?,
            nonpageable_groups: nonpageable, parent_working_state_id: None, last_request: None, page_id: None };
        r.groups.retain(|(_, e)| references.iter().any(|r| r.group_entry_id == e.entry_id));
        self.source.paging = Some(QualifiedPaging { metadata, references });
        self.source = self.source.clone().with_recall(r)?;
        self.working_state = self.source.compile(self.working_state.request())?;
        self.working_state.validate_paging_envelope()?;
        self.measurements.output_bytes = serde_json::to_vec(&self.working_state).map_err(|e| e.to_string())?.len();
        self.measurements.selected_semantic_units = self.working_state.bounds.selected_semantic_units;
        Ok(self)
    }
}

impl SemanticWorkingState {
    pub fn paging(&self) -> Option<&WorkingPaging> { self.paging.as_ref() }
    pub fn page_references(&self) -> &[SemanticReference] {
        self.entries.iter().find_map(|e| match &e.value {
            SemanticValue::SemanticPageReferences { references, .. } => Some(references.as_slice()), _ => None,
        }).unwrap_or(&[])
    }
    pub fn resident_page_references(&self) -> Vec<String> {
        self.page_references().iter().filter(|r| self.entries.iter().any(|e| e.entry_id == r.group_entry_id))
            .map(|r| r.reference_id.clone()).collect()
    }
    pub(crate) fn validate_paging_envelope(&self) -> Result<(), String> {
        let p = self.paging.as_ref().ok_or("semantic_paging_requires_v4")?;
        let recall = self.recall.as_ref().ok_or("semantic_paging_requires_recall")?;
        recall.request.recall_request()?;
        let mut copy = self.clone(); copy.working_state_id.clear();
        if self.schema != PAGED_WORKING_SCHEMA || self.compiler != PAGED_COMPILER
            || self.request != recall.request.compilation
            || self.case_id != recall.request.case_id
            || self.case_generation != recall.request.expected_generation
            || self.participant_id != self.request.scope.participant_id
            || self.id() != format!("working-state:{}", identity(&copy)?)
            || p.catalog_digest != identity(&self.page_references())?
            || p.profile != REFERENCE_PROFILE || self.page_references().len() > 256 {
            return Err("semantic_page_base_integrity_mismatch".into());
        }
        for r in self.page_references() {
            r.validate()?;
            if r.at != self.recall.as_ref().ok_or("semantic_paging_requires_recall")?.recall_request.at {
                return Err("semantic_page_cut_mismatch".into());
            }
        }
        Ok(())
    }
}

impl SemanticState {
    /// Fixed-generation continuation of a declared working set, NOT a fresh
    /// relevance query. Recheck current control and every retained current item.
    pub(crate) fn validate_paging_current(&self, base: &SemanticWorkingState) -> Result<(), String> {
        let p = base.paging.as_ref().ok_or("semantic_paging_requires_v4")?;
        if self.case_id() != base.case_id || self.generation() != base.case_generation {
            return Err("semantic_page_stale_base_recompile_required".into());
        }
        let current = self.candidates_mode(base.request(), true)?;
        if identity(&control(&current.entries))? != p.current_control_digest
            || identity(&control(&base.entries))? != p.current_control_digest {
            return Err("semantic_page_current_authority_changed".into());
        }
        for e in base.entries.iter().filter(|e| is_current(e)) {
            if !current.entries.contains(e) { return Err("semantic_page_current_material_unavailable".into()); }
        }
        if base.entries.iter().filter(|e| is_current(e)).map(|e| e.entry_id.clone()).collect::<BTreeSet<_>>() != p.current_material_ids {
            return Err("semantic_page_current_material_mismatch".into());
        }
        Ok(())
    }

    pub(crate) fn with_page_groups(mut self, base: &SemanticWorkingState, request: &PageRequest,
        page: &SemanticPage, groups: Vec<(bool, SemanticEntry)>) -> Result<Self, String> {
        let mut metadata = base.paging.clone().ok_or("semantic_paging_requires_v4")?;
        metadata.parent_working_state_id = Some(base.id().into());
        metadata.last_request = Some(request.clone()); metadata.page_id = Some(page.page_id.clone());
        self.paging = Some(QualifiedPaging { metadata, references: base.page_references().to_vec() });
        let input = QualifiedRecall { metadata: base.recall.clone().ok_or("semantic_paging_requires_recall")?,
            groups, excluded_transitions: BTreeSet::new() };
        self.with_recall(input)
    }
}

pub(crate) fn rehydrated_entry(reference: &SemanticReference, trace: RecallTrace, recall_id: &str) -> Result<SemanticEntry, String> {
    let evidence = RecalledEvidence::from(trace);
    if !evidence.closure_complete { return Err("semantic_page_backing_unavailable".into()); }
    if identity(&normalize(evidence.clone()))? != reference.evidence_digest {
        return Err("semantic_page_exact_group_changed".into());
    }
    Ok(SemanticEntry { entry_id: reference.group_entry_id.clone(), posture: AuthorityPosture::DerivedMemory,
        provenance: vec![SemanticProvenance { kind: ProvenanceKind::RecallTrace, source_ref: recall_id.into() }],
        value: SemanticValue::RecalledEvidence { evidence: Box::new(evidence) } })
}
