//! Documentary family within Recall's single bounded resolver, not another
//! search, history or knowledge owner. All inputs are request-scoped qualified D.
use super::*;
use crate::effect::access::source::SourceBacking;
use crate::graph::knowledge::KnowledgeRelation;
use crate::memory_hierarchy::knowledge::{
    KnowledgeConflict, KnowledgeSource, KnowledgeUnit, KnowledgeView,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecalledSource {
    pub source: KnowledgeSource,
    pub admitted_at_generation: u64,
    pub admission_transition: String,
    pub applicable_at_cut: bool,
    pub reasons: Vec<SelectionReason>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecalledUnit {
    pub unit: KnowledgeUnit,
    pub reasons: Vec<SelectionReason>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CrossReference {
    pub unit: String,
    pub exact_reference: String,
    pub event: String,
    pub posture: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DocumentSegment {
    pub source: String,
    pub units: Vec<String>,
    pub grouping: String,
}
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DocumentaryTrace {
    pub sources: Vec<RecalledSource>,
    pub units: Vec<RecalledUnit>,
    pub relations: Vec<KnowledgeRelation>,
    pub cross_references: Vec<CrossReference>,
    pub contradictions: Vec<KnowledgeConflict>,
    pub segments: Vec<DocumentSegment>,
}

pub(super) fn discover(
    d: &KnowledgeView,
    request: &RecallRequest,
    cut: &[Transition],
) -> Result<Vec<crate::memory_index::LexicalHit>, String> {
    let mut applicable = BTreeMap::new();
    for t in cut {
        if let P::CaseSourceProgressed { progress } = &t.payload {
            applicable.insert(
                &progress.source_id,
                progress.revision.as_ref().map(|r| &r.revision_id),
            );
        }
    }
    let sources: BTreeSet<_> = d
        .sources
        .iter()
        .filter(|s| {
            applicable
                .get(&s.source_id)
                .is_some_and(|r| *r == Some(&s.revision_id))
                || request.required_refs.iter().any(|id| {
                    [s.id.as_str(), s.source_id.as_str(), s.revision_id.as_str()]
                        .contains(&id.as_str())
                        || d.units.iter().any(|u| u.id == *id && u.source == s.id)
                })
        })
        .map(|s| &s.id)
        .collect();
    crate::memory_index::MemoryLexicalIndex::build_texts(
        d.units
            .iter()
            .filter(|u| sources.contains(&u.source))
            .map(|u| (u.id.as_str(), u.text.as_str())),
    )?
    .search(&request.query, request.bounds.candidates)
}

pub(super) fn aliases(d: &KnowledgeView, aliases: &mut BTreeMap<String, BTreeSet<String>>) {
    for s in &d.sources {
        for id in [&s.id, &s.source_id, &s.revision_id] {
            aliases.entry(id.clone()).or_default().insert(s.id.clone());
        }
    }
    for u in &d.units {
        aliases
            .entry(u.id.clone())
            .or_default()
            .insert(u.id.clone());
        if let Some(entity) = &u.entity {
            aliases
                .entry(entity.clone())
                .or_default()
                .insert(u.id.clone());
        }
    }
}

/// Source/parent/conflict closure is mandatory, not a relevance cutoff. Explicit
/// document references may seed already disclosed H objects, never topology
/// inferred from matching words. No resource-wide experiential fanout.
pub(super) fn close(
    selected: &mut Selection,
    d: &KnowledgeView,
    aliases: &BTreeMap<String, BTreeSet<String>>,
    bounds: &RecallBounds,
) -> Result<(), String> {
    for depth in 0..=bounds.expansion_depth {
        let before = selected.len();
        loop {
            let n = selected.len();
            for u in &d.units {
                if selected.contains_key(&u.id) {
                    selected
                        .entry(u.source.clone())
                        .or_default()
                        .insert(SelectionReason::SourceClosure);
                    if let Some(p) = &u.parent {
                        selected
                            .entry(p.clone())
                            .or_default()
                            .insert(SelectionReason::SourceClosure);
                    }
                }
            }
            for c in &d.contradictions {
                if c.members.iter().any(|id| selected.contains_key(id)) {
                    for id in &c.members {
                        selected
                            .entry(id.clone())
                            .or_default()
                            .insert(SelectionReason::ContradictionContext);
                    }
                }
            }
            if selected.len() > bounds.events {
                return Err("recall_required_context_exceeds_event_budget".into());
            }
            if n == selected.len() {
                break;
            }
        }
        if depth == bounds.expansion_depth {
            break;
        }
        let seeds: Vec<_> = d
            .units
            .iter()
            .filter(|u| selected.contains_key(&u.id))
            .collect();
        for u in seeds {
            for r in &u.references {
                // Exact resources are shared subjects, not proof that every
                // operation over that resource explains this documentary claim.
                if r.starts_with("resource:") {
                    continue;
                }
                if let Some(nodes) = aliases.get(r) {
                    for id in nodes {
                        selected
                            .entry(id.clone())
                            .or_default()
                            .insert(SelectionReason::QualifiedRelationExpansion);
                    }
                }
            }
        }
        if selected.len() > bounds.events {
            return Err("recall_required_context_exceeds_event_budget".into());
        }
        if before == selected.len() {
            break;
        }
    }
    Ok(())
}

pub(super) fn assemble(
    d: &KnowledgeView,
    selected: &Selection,
    aliases: &BTreeMap<String, BTreeSet<String>>,
    history: &[Transition],
    cut: u64,
) -> DocumentaryTrace {
    let sources: Vec<_> = d
        .sources
        .iter()
        .filter(|s| selected.contains_key(&s.id))
        .map(|s| {
            let admissions: Vec<_> = history
                .iter()
                .filter_map(|t| match &t.payload {
                    P::CaseSourceProgressed { progress }
                        if progress.source_id == s.source_id && t.sequence <= cut =>
                    {
                        Some((t, progress))
                    }
                    _ => None,
                })
                .collect();
            let admitted = admissions.iter().find(|(_, p)| {
                p.phase == crate::effect::access::source::SourcePhase::Acquired
                    && p.revision
                        .as_ref()
                        .is_some_and(|r| r.revision_id == s.revision_id)
            });
            RecalledSource {
                source: s.clone(),
                admitted_at_generation: admitted.map_or(0, |(t, _)| t.sequence),
                admission_transition: admitted
                    .map_or_else(String::new, |(t, _)| t.transition_id.clone()),
                applicable_at_cut: admissions.last().is_some_and(|(_, p)| {
                    p.phase == crate::effect::access::source::SourcePhase::Acquired
                        && p.revision
                            .as_ref()
                            .is_some_and(|r| r.revision_id == s.revision_id)
                }),
                reasons: selected[&s.id].iter().cloned().collect(),
            }
        })
        .collect();
    let units: Vec<_> = d
        .units
        .iter()
        .filter(|u| selected.contains_key(&u.id))
        .map(|u| RecalledUnit {
            unit: u.clone(),
            reasons: selected[&u.id].iter().cloned().collect(),
        })
        .collect();
    let visible: BTreeSet<_> = selected
        .keys()
        .cloned()
        .chain(units.iter().filter_map(|u| u.unit.entity.clone()))
        .chain(sources.iter().map(|s| s.source.resource_id.clone()))
        .collect();
    let relations = d
        .relations
        .iter()
        .filter(|r| {
            visible.contains(&r.from)
                && visible.contains(&r.to)
                && r.backing_units.iter().all(|id| selected.contains_key(id))
        })
        .cloned()
        .collect();
    let mut cross_references = Vec::new();
    for u in &units {
        for r in &u.unit.references {
            for id in aliases
                .get(r)
                .into_iter()
                .flatten()
                .filter(|id| id.starts_with("transition:") && selected.contains_key(*id))
            {
                cross_references.push(CrossReference {
                    unit: u.unit.id.clone(),
                    exact_reference: r.clone(),
                    event: id.clone(),
                    posture: "source_stated_exact_reference_not_causality_or_normative_support"
                        .into(),
                });
            }
        }
    }
    let contradictions = d
        .contradictions
        .iter()
        .filter(|c| c.members.iter().all(|id| selected.contains_key(id)))
        .cloned()
        .collect();
    let segments = sources
        .iter()
        .map(|s| DocumentSegment {
            source: s.source.id.clone(),
            units: units
                .iter()
                .filter(|u| u.unit.source == s.source.id)
                .map(|u| u.unit.id.clone())
                .collect(),
            grouping: "source_structure_order_not_case_chronology".into(),
        })
        .collect();
    DocumentaryTrace {
        sources,
        units,
        relations,
        cross_references,
        contradictions,
        segments,
    }
}

/// Source-owned acquisition evidence cannot bypass current source disclosure by
/// entering via H/W20 instead of D. Exact backing IDs determine the cut; no text
/// matching, proximity or blanket policy-history removal. Independently admitted
/// operational evidence not owned by source acquisition keeps its H contract.
pub(crate) fn restrict_acquisition_history(
    h: &mut HistoricalSemanticView,
    history: &[Transition],
    allowed: &BTreeSet<String>,
) {
    let source_ids: BTreeSet<_> = history
        .iter()
        .filter_map(|t| match &t.payload {
            P::CaseSourceDeclared { declaration }
                if declaration
                    .roles
                    .contains(&crate::effect::access::source::SourceRole::Knowledge) =>
            {
                Some(declaration.source_id.clone())
            }
            _ => None,
        })
        .collect();
    let denied: BTreeSet<_> = history
        .iter()
        .filter_map(|t| match &t.payload {
            P::CaseSourceProgressed { progress } if source_ids.contains(&progress.source_id) => {
                progress.revision.as_ref()
            }
            _ => None,
        })
        .flat_map(|r| r.items.iter())
        .map(|i| backing_id(&i.backing))
        .filter(|id| !allowed.contains(id))
        .collect();
    let mut operations = BTreeSet::new();
    let mut observations = BTreeSet::new();
    for t in history {
        match &t.payload {
            P::CaseContentAdmitted { admission } if denied.contains(&admission.admission_id) => {
                observations.insert(admission.discovery_observation_id.clone());
                operations.insert(admission.operation_id.clone());
            }
            P::ResourceObservationRecorded { observation }
                if denied.contains(&observation.observation_id) =>
            {
                observations.insert(observation.observation_id.clone());
            }
            _ => (),
        }
    }
    for t in history {
        if let P::ResourceObservationRecorded { observation } = &t.payload {
            if observations.contains(&observation.observation_id) {
                operations.insert(observation.operation_id.clone());
            }
        }
    }
    h.known_by_then.retain(|e| match &e.payload {
        P::CaseContentAdmitted { admission } => !denied.contains(&admission.admission_id),
        P::OperationRecorded { operation } => !operations.contains(&operation.operation_id),
        P::DecisionRecorded { decision } => !operations.contains(&decision.operation_id),
        P::ResourceObservationRecorded { observation } => {
            !operations.contains(&observation.operation_id)
        }
        _ => true,
    });
}
pub(crate) fn backing_id(b: &SourceBacking) -> String {
    match b {
        SourceBacking::Content { admission_id } => admission_id.clone(),
        SourceBacking::Observation { observation_id } => observation_id.clone(),
        SourceBacking::Policy { source_id, .. } => source_id.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_hierarchy::knowledge::{
        self as k, KnowledgePosture, KnowledgeStatus, SourceLocation,
    };

    fn view(reference: bool) -> KnowledgeView {
        let mut d = k::derive(
            &k::KnowledgeRequest::new("case:resolver"),
            "principal:reader",
            vec![],
        )
        .unwrap()
        .view;
        d.sources.push(KnowledgeSource {
            id: "knowledge-source:manual".into(),
            source_id: "case-source:manual".into(),
            logical_name: "manual".into(),
            revision_id: "source-revision:1".into(),
            current_revision: true,
            path: "manual.json".into(),
            digest: crate::effect::digest_bytes(b"contract-fixture"),
            bytes: 16,
            backing: SourceBacking::Observation {
                observation_id: "observation:backing".into(),
            },
            roles: vec![crate::effect::access::source::SourceRole::Knowledge],
            resource_id: "resource:manual".into(),
            media_type: "application/json".into(),
            source_kind: "document".into(),
            extractor: k::DERIVATION_PROFILE.into(),
            extraction_id: "extraction:qualified-input-fixture".into(),
            status: KnowledgeStatus::Qualified,
            detail: "resolver-only fixture; acquisition independently product-qualified".into(),
        });
        for (id, value, posture) in [
            ("document", 90, KnowledgePosture::SourceStated),
            ("observation", 30, KnowledgePosture::RecordedObservation),
        ] {
            d.units.push(KnowledgeUnit {
                id: format!("knowledge-unit:{id}"),
                source: d.sources[0].id.clone(),
                parent: None,
                kind: "value".into(),
                location: SourceLocation::JsonPointer {
                    pointer: format!("/{id}"),
                    container: None,
                },
                text: format!("retention_days {value}"),
                posture,
                entity: Some("urn:billing".into()),
                predicate: Some("retention_days".into()),
                value: Some(value.into()),
                references: if reference {
                    vec!["decision:exact".into(), "decision:hidden".into()]
                } else {
                    vec![]
                },
                topics: vec![],
            });
        }
        d.contradictions.push(KnowledgeConflict {
            id: "conflict:unresolved".into(),
            entity: "urn:billing".into(),
            predicate: "retention_days".into(),
            members: d.units.iter().map(|u| u.id.clone()).collect(),
            posture: "unresolved_no_winner".into(),
        });
        d
    }

    #[test]
    fn documentary_reference_counterfactual_conflict_closure_and_budget() {
        for reference in [true, false] {
            let d = view(reference);
            let mut a = BTreeMap::from([(
                "decision:exact".into(),
                BTreeSet::from(["transition:decision".into()]),
            )]);
            aliases(&d, &mut a);
            let mut s = Selection::from([(
                "knowledge-unit:document".into(),
                BTreeSet::from([SelectionReason::ExactRequiredRef]),
            )]);
            close(&mut s, &d, &a, &RecallBounds::default()).unwrap();
            assert_eq!(s.contains_key("transition:decision"), reference);
            assert!(!s.contains_key("decision:hidden"));
            assert!(
                s.contains_key("knowledge-source:manual")
                    && s.contains_key("knowledge-unit:observation")
            );
            let t = assemble(&d, &s, &a, &[], 1);
            assert_eq!(t.cross_references.len(), usize::from(reference) * 2);
            assert_eq!(t.contradictions.len(), 1);
            assert_eq!(t.units[0].unit.posture, KnowledgePosture::SourceStated);
            assert_eq!(
                t.units[1].unit.posture,
                KnowledgePosture::RecordedObservation
            );
            let mut tiny = RecallBounds::default();
            tiny.events = 1;
            assert!(close(&mut s, &d, &a, &tiny).is_err());
            assert_eq!(s, {
                let mut repeated = Selection::from([(
                    "knowledge-unit:document".into(),
                    BTreeSet::from([SelectionReason::ExactRequiredRef]),
                )]);
                close(&mut repeated, &d, &a, &RecallBounds::default()).unwrap();
                repeated
            });
        }
        println!("documentary_resolver counterfactual_reference=present_only conflict=unresolved source_stated!=recorded_observation hidden_intermediate=absent mandatory_closure_overflow=refused");
    }
}
