//! Source-qualified documentary edges in the existing derived graph owner.
use crate::memory_hierarchy::knowledge::{
    identity, KnowledgeConflict, KnowledgeEntity, KnowledgeSource, KnowledgeStatus, KnowledgeUnit,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeRelationKind {
    Contains,
    Defines,
    SourceSupportsClaim,
    References,
    DescribesResource,
    Contradicts,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeRelation {
    pub id: String,
    pub from: String,
    pub to: String,
    pub kind: KnowledgeRelationKind,
    pub posture: String,
    pub backing_units: Vec<String>,
}
pub(crate) fn derive(
    sources: &[KnowledgeSource],
    units: &[KnowledgeUnit],
    entities: &[KnowledgeEntity],
    conflicts: &[KnowledgeConflict],
) -> Result<Vec<KnowledgeRelation>, String> {
    let mut edges = Vec::new();
    let known: BTreeSet<_> = sources
        .iter()
        .filter(|s| s.status == KnowledgeStatus::Qualified)
        .map(|s| s.id.as_str())
        .chain(units.iter().map(|u| u.id.as_str()))
        .chain(entities.iter().map(|e| e.id.as_str()))
        .chain(
            sources
                .iter()
                .filter(|s| s.status == KnowledgeStatus::Qualified)
                .map(|s| s.resource_id.as_str()),
        )
        .collect();
    let mut edge = |from: &str,
                    to: &str,
                    kind: KnowledgeRelationKind,
                    posture: &str,
                    backing_units: Vec<String>| {
        let id = identity("knowledge-relation", &(from, to, &kind, &backing_units));
        edges.push(KnowledgeRelation {
            id,
            from: from.into(),
            to: to.into(),
            kind,
            posture: posture.into(),
            backing_units,
        });
    };
    for u in units {
        edge(
            u.parent.as_deref().unwrap_or(&u.source),
            &u.id,
            KnowledgeRelationKind::Contains,
            "exact_document_structure",
            vec![u.id.clone()],
        );
        if let Some(e) = &u.entity {
            edge(
                &u.id,
                e,
                KnowledgeRelationKind::Defines,
                "explicit_source_identifier_not_entity_resolution_by_similarity",
                vec![u.id.clone()],
            );
        }
        if u.predicate.is_some() {
            edge(
                &u.source,
                &u.id,
                KnowledgeRelationKind::SourceSupportsClaim,
                "source_states_not_verifies",
                vec![u.id.clone()],
            );
        }
        for r in &u.references {
            if known.contains(r.as_str()) {
                edge(
                    &u.id,
                    r,
                    KnowledgeRelationKind::References,
                    "explicit_document_reference_not_causality",
                    vec![u.id.clone()],
                );
            }
        }
    }
    for s in sources
        .iter()
        .filter(|s| s.status == KnowledgeStatus::Qualified)
    {
        if let Some(unit) = units.iter().find(|u| u.source == s.id) {
            edge(
                &s.id,
                &s.resource_id,
                KnowledgeRelationKind::DescribesResource,
                "exact_admitted_source_resource_relation",
                vec![unit.id.clone()],
            );
        }
    }
    // Linear star represents a conflict set, not quadratic pairwise truth claims.
    for c in conflicts {
        for member in c.members.iter().skip(1) {
            let first = units
                .iter()
                .find(|u| u.id == c.members[0])
                .ok_or("knowledge_conflict_backing_missing")?;
            let other = units
                .iter()
                .find(|u| u.id == *member)
                .ok_or("knowledge_conflict_backing_missing")?;
            if first.value == other.value {
                continue;
            }
            edge(
                &c.members[0],
                member,
                KnowledgeRelationKind::Contradicts,
                "same_explicit_subject_property_disagreement_no_winner",
                vec![c.members[0].clone(), member.clone()],
            );
        }
    }
    edges.sort_by(|a, b| a.id.cmp(&b.id));
    edges.dedup_by(|a, b| a.id == b.id);
    if edges.len() > 65536 {
        return Err("knowledge_relation_budget_exceeded".into());
    }
    Ok(edges)
}
