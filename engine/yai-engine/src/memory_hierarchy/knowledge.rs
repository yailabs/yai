//! Source-grounded D: disposable derivation in the existing memory/access owner.
//! Only the source owner's authorized snapshot can supply inputs. Documentary
//! statements are not Case facts, policy rules, or operational observations.
use crate::effect::access::source::{SourceBacking, SourceRole};
use crate::effect::digest_bytes;
use crate::graph::knowledge::{self, KnowledgeRelation};
use crate::memory_index::{LexicalHit, MemoryLexicalIndex};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

mod document;
#[cfg(test)]
mod tests;
pub const KNOWLEDGE_SCHEMA: &str = "yai.source_knowledge.v1";
pub const DERIVATION_PROFILE: &str = "yai.source_knowledge.deterministic.v1";
pub const MAX_KNOWLEDGE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_KNOWLEDGE_UNITS: usize = 8192;
pub const MAX_UNIT_BYTES: usize = 4096;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeRequest {
    pub case_id: String,
    pub source: Option<String>,
    pub revision: Option<String>,
    pub max_units: usize,
}
impl KnowledgeRequest {
    pub fn new(case: impl Into<String>) -> Self {
        Self {
            case_id: case.into(),
            source: None,
            revision: None,
            max_units: 4096,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.case_id.starts_with("case:")
            || self.case_id.len() > 256
            || self.max_units == 0
            || self.max_units > MAX_KNOWLEDGE_UNITS
            || (self.revision.is_some() && self.source.is_none())
            || self
                .source
                .iter()
                .chain(self.revision.iter())
                .any(|s| s.is_empty() || s.len() > 256)
        {
            return Err("knowledge_request_invalid".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgePosture {
    SourceStated,
    DeterministicStructure,
    RecordedObservation,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeStatus {
    Qualified,
    Unsupported,
    NeedsProcessing,
    BackingUnavailable,
}

/// Exact original addressing where possible; PDF coordinates deliberately bind
/// extracted text/profile, never purported original byte offsets or glyph boxes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceLocation {
    WholeSource,
    TextLines {
        first: usize,
        last: usize,
    },
    JsonPointer {
        pointer: String,
        container: Option<Box<SourceLocation>>,
    },
    PdfExtractedLines {
        page: u32,
        first: usize,
        last: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeSource {
    pub id: String,
    pub source_id: String,
    pub logical_name: String,
    pub revision_id: String,
    pub current_revision: bool,
    pub path: String,
    pub digest: String,
    pub bytes: u64,
    pub backing: SourceBacking,
    pub roles: Vec<SourceRole>,
    pub resource_id: String,
    pub media_type: String,
    pub source_kind: String,
    pub extractor: String,
    /// Reusable extraction identity excludes Case binding; source identity does not.
    pub extraction_id: String,
    pub status: KnowledgeStatus,
    pub detail: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeUnit {
    pub id: String,
    pub source: String,
    pub parent: Option<String>,
    pub kind: String,
    pub location: SourceLocation,
    pub text: String,
    pub posture: KnowledgePosture,
    pub entity: Option<String>,
    pub predicate: Option<String>,
    pub value: Option<serde_json::Value>,
    pub references: Vec<String>,
    pub topics: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeEntity {
    pub id: String,
    pub definitions: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeTopic {
    pub name: String,
    pub units: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeConflict {
    pub id: String,
    pub entity: String,
    pub predicate: String,
    pub members: Vec<String>,
    pub posture: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeView {
    pub schema: String,
    pub id: String,
    pub case_id: String,
    pub requesting_principal: String,
    pub profile: String,
    pub sources: Vec<KnowledgeSource>,
    pub units: Vec<KnowledgeUnit>,
    pub entities: Vec<KnowledgeEntity>,
    pub topics: Vec<KnowledgeTopic>,
    pub relations: Vec<KnowledgeRelation>,
    pub contradictions: Vec<KnowledgeConflict>,
    pub source_closure: String,
    pub max_units: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct KnowledgeMeasurements {
    pub source_documents: usize,
    pub original_bytes: u64,
    pub units: usize,
    pub entities: usize,
    pub claims: usize,
    pub relations: usize,
    pub contradictions: usize,
    pub source_resolution_us: u128,
    pub derivation_us: u128,
    pub graph_us: u128,
    pub index_us: u128,
    pub output_bytes: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct KnowledgeResult {
    pub view: KnowledgeView,
    pub measurements: KnowledgeMeasurements,
}

pub(crate) struct QualifiedSource {
    pub source: KnowledgeSource,
    pub bytes: Option<Vec<u8>>,
}
pub(crate) fn identity<T: Serialize>(kind: &str, value: &T) -> String {
    format!(
        "{kind}:{}",
        &digest_bytes(&serde_json::to_vec(value).expect("serializable knowledge"))[7..]
    )
}

pub(crate) fn derive(
    request: &KnowledgeRequest,
    principal: &str,
    mut inputs: Vec<QualifiedSource>,
) -> Result<KnowledgeResult, String> {
    request.validate()?;
    inputs.sort_by(|a, b| a.source.id.cmp(&b.source.id));
    if inputs.len() > 1024
        || inputs.iter().map(|s| s.source.bytes).sum::<u64>() > MAX_KNOWLEDGE_BYTES as u64
    {
        return Err("knowledge_source_budget_exceeded".into());
    }
    let started = Instant::now();
    let mut sources = Vec::new();
    let mut units = Vec::new();
    for mut input in inputs {
        if let Some(bytes) = input.bytes.take() {
            if bytes.len() as u64 != input.source.bytes
                || digest_bytes(&bytes) != input.source.digest
            {
                input.source.status = KnowledgeStatus::BackingUnavailable;
                input.source.detail = "exact_backing_integrity_mismatch".into();
            } else {
                match document::extract(&mut input.source, &bytes) {
                    Ok(mut extracted) => units.append(&mut extracted),
                    Err(reason) => {
                        input.source.status = if reason == "unsupported_source_profile" {
                            KnowledgeStatus::Unsupported
                        } else {
                            KnowledgeStatus::NeedsProcessing
                        };
                        input.source.detail = reason;
                    }
                }
            }
        } else {
            input.source.status = KnowledgeStatus::BackingUnavailable;
            input.source.detail = "exact_backing_unavailable_no_live_substitution".into();
        }
        if units.len() > request.max_units {
            return Err("knowledge_unit_budget_exceeded".into());
        }
        sources.push(input.source);
    }
    let derivation_us = started.elapsed().as_micros();
    let graph_started = Instant::now();
    let mut entities = BTreeMap::<String, Vec<String>>::new();
    let mut topics = BTreeMap::<String, Vec<String>>::new();
    let mut groups = BTreeMap::<(String, String), Vec<&KnowledgeUnit>>::new();
    for unit in &units {
        if let Some(entity) = &unit.entity {
            entities
                .entry(entity.clone())
                .or_default()
                .push(unit.id.clone());
        }
        for topic in &unit.topics {
            topics
                .entry(topic.clone())
                .or_default()
                .push(unit.id.clone());
        }
        if let (Some(entity), Some(predicate), Some(_)) =
            (&unit.entity, &unit.predicate, &unit.value)
        {
            groups
                .entry((entity.clone(), predicate.clone()))
                .or_default()
                .push(unit);
        }
    }
    let mut contradictions = Vec::new();
    for ((entity, predicate), members) in groups {
        let values: BTreeSet<_> = members
            .iter()
            .map(|u| serde_json::to_string(&u.value).unwrap())
            .collect();
        if values.len() < 2 {
            continue;
        }
        // Same exact explicit subject/property, different recorded values. No
        // recency winner and no W20 resource-content supersession rule applied to D.
        let ids: Vec<_> = members.iter().map(|u| u.id.clone()).collect();
        contradictions.push(KnowledgeConflict {
            id: identity("knowledge-conflict", &ids),
            entity,
            predicate,
            members: ids,
            posture: "unresolved_source_value_disagreement_not_current_truth".into(),
        });
    }
    let entities: Vec<_> = entities
        .into_iter()
        .map(|(id, definitions)| KnowledgeEntity { id, definitions })
        .collect();
    let topics = topics
        .into_iter()
        .map(|(name, units)| KnowledgeTopic { name, units })
        .collect();
    let relations = knowledge::derive(&sources, &units, &entities, &contradictions)?;
    let graph_us = graph_started.elapsed().as_micros();
    let source_closure = if sources
        .iter()
        .any(|s| s.status == KnowledgeStatus::BackingUnavailable)
    {
        "incomplete"
    } else {
        "complete_for_retained_sources"
    }
    .into();
    let mut view = KnowledgeView {
        schema: KNOWLEDGE_SCHEMA.into(),
        id: String::new(),
        case_id: request.case_id.clone(),
        requesting_principal: principal.into(),
        profile: DERIVATION_PROFILE.into(),
        sources,
        units,
        entities,
        topics,
        relations,
        contradictions,
        source_closure,
        max_units: request.max_units,
    };
    view.id = identity("knowledge-view", &view);
    let index_started = Instant::now();
    let _ = view.index()?;
    let index_us = index_started.elapsed().as_micros();
    let output_bytes = serde_json::to_vec(&view).map_err(|e| e.to_string())?.len();
    if output_bytes > 16 * 1024 * 1024 {
        return Err("knowledge_output_budget_exceeded".into());
    }
    let measurements = KnowledgeMeasurements {
        source_documents: view.sources.len(),
        original_bytes: view.sources.iter().map(|s| s.bytes).sum(),
        units: view.units.len(),
        entities: view.entities.len(),
        claims: view.units.iter().filter(|u| u.predicate.is_some()).count(),
        relations: view.relations.len(),
        contradictions: view.contradictions.len(),
        source_resolution_us: 0,
        derivation_us,
        graph_us,
        index_us,
        output_bytes,
    };
    Ok(KnowledgeResult { view, measurements })
}
impl KnowledgeView {
    fn index(&self) -> Result<MemoryLexicalIndex, String> {
        MemoryLexicalIndex::build_texts(self.units.iter().map(|u| (u.id.as_str(), u.text.as_str())))
    }
    /// Pure candidate discovery over this qualified immutable snapshot, not
    /// permission to reuse it after a source/scope change. Product queries rebuild.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<LexicalHit>, String> {
        if query.len() > 2048 {
            return Err("knowledge_query_bound".into());
        }
        self.index()?.search(query, limit)
    }
    pub fn resolve(&self, id: &str) -> Result<&KnowledgeUnit, String> {
        self.units
            .iter()
            .find(|u| u.id == id)
            .ok_or_else(|| "knowledge_reference_unavailable".into())
    }
    /// Read-only deterministic wiki/navigation, never a model-written summary.
    pub fn navigation(&self) -> String {
        let mut out = format!("# Case knowledge: {}\n\nView: {}\nDocumentary claims inform; they do not establish Case truth or authority.\n\n## Sources\n", self.case_id,self.id);
        for s in &self.sources {
            out.push_str(&format!(
                "- {} / {} @ {} [{}; {:?}]\n",
                safe(&s.logical_name),
                safe(&s.path),
                s.revision_id,
                if s.current_revision {
                    "current captured revision"
                } else {
                    "historical captured revision"
                },
                s.status
            ));
        }
        out.push_str("\n## Entities\n");
        for e in &self.entities {
            out.push_str(&format!(
                "- {} ({} source-addressed units)\n",
                safe(&e.id),
                e.definitions.len()
            ));
        }
        out.push_str("\n## Topics\n");
        for t in &self.topics {
            out.push_str(&format!("- {}\n", safe(&t.name)));
        }
        out.push_str("\n## Claims\n");
        for u in self.units.iter().filter(|u| u.predicate.is_some()) {
            out.push_str(&format!(
                "- {:?}: {} :: {}\n  source {} at {}\n",
                u.posture,
                safe(&u.text),
                u.id,
                u.source,
                serde_json::to_string(&u.location).unwrap()
            ));
        }
        out.push_str("\n## Contradictions\n");
        for c in &self.contradictions {
            out.push_str(&format!(
                "- {} / {}: {} statements; unresolved, no winner\n",
                safe(&c.entity),
                safe(&c.predicate),
                c.members.len()
            ));
        }
        out.push_str(&format!("\n## Source closure\n{}\nUse exact unit references with knowledge resolve. No Recall/W integration.\n",self.source_closure));
        out
    }
}
fn safe(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_control() {
                vec![' ']
            } else if matches!(c, '<' | '>' | '[' | ']' | '`' | '\\') {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect()
}
