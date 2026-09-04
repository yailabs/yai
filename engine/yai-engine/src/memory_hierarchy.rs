//! Case-native episodic and evidence-bound semantic memory.
//!
//! Every value in this module is a disposable projection. Canonical authority
//! remains the ordered Case `Transition` history; recorded provider output is
//! candidate material and can only become a bounded inference after support
//! validation. Nothing here can authorize an operation.

use crate::context::stable_digest;
use crate::memory::{OperationalMemoryBuild, OperationalMemoryEntry, OperationalMemoryValue};
use crate::transition::{CaseLifecycle, CaseState, Transition, TransitionPayload};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const MEMORY_EPISODE_SCHEMA: &str = "yai.memory_episode.v1";
pub const EPISODE_DERIVATION_VERSION: &str = "yai.memory_episode.derivation.v1";
pub const EPISODE_BOUNDARY_POLICY: &str = "yai.memory_episode.boundary.structural_refs.v1";
pub const SEMANTIC_ASSERTION_SCHEMA: &str = "yai.semantic_memory_assertion.v1";
pub const CONTRADICTION_SET_SCHEMA: &str = "yai.semantic_contradiction_set.v1";
pub const CONSOLIDATION_INPUT_SCHEMA: &str = "yai.memory_consolidation_input.v1";
pub const CONSOLIDATION_CANDIDATE_SCHEMA: &str = "yai.memory_consolidation_candidate.v1";
pub const CONSOLIDATION_NORMALIZER_VERSION: &str = "yai.memory_consolidation.normalizer.v1";
pub const MEMORY_HIERARCHY_SCHEMA: &str = "yai.memory_hierarchy_manifest.v1";
pub const RETENTION_POLICY_SCHEMA: &str = "yai.memory_retention_policy.v1";
pub const MAX_CONSOLIDATION_ASSERTIONS: usize = 64;
pub const MAX_SUPPORT_REFS: usize = 16;
pub const MAX_SUPPORT_DEPTH: usize = 16;
pub const MAX_PREDICATE_BYTES: usize = 128;
pub const MAX_SUBJECT_BYTES: usize = 256;
pub const MAX_VALUE_BYTES: usize = 2048;
pub const MAX_NARRATIVE_BYTES: usize = 8192;
pub const MAX_CONSOLIDATION_RESULT_BYTES: usize = 256 * 1024;
pub const MAX_CONTRADICTION_MEMBERS: usize = 1024;
pub const DEFAULT_RECENT_EPISODES: usize = 128;
pub const MAX_CONSOLIDATION_EPISODES: usize = 16;
pub const MAX_CONSOLIDATION_OPERATIONAL_ITEMS: usize = 32;
pub const MAX_CONSOLIDATION_EXISTING_ASSERTIONS: usize = 16;
pub const CONSOLIDATION_SEMANTIC_UNIT_BUDGET: usize = 16_384;

fn digest_json<T: Serialize>(value: &T, label: &str) -> Result<String, String> {
    serde_json::to_string(value)
        .map(|encoded| format!("sha256:{}", stable_digest(&encoded)))
        .map_err(|error| format!("{label}_encode_failed: {error}"))
}

fn short_id(prefix: &str, digest: &str) -> String {
    let digest = digest.strip_prefix("sha256:").unwrap_or(digest);
    format!("{prefix}:{}", &digest[..digest.len().min(32)])
}

fn bounded_text(value: &str, maximum: usize, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{label}_required"));
    }
    if value.len() > maximum {
        return Err(format!("{label}_too_large"));
    }
    if value.chars().any(|ch| ch == '\0') {
        return Err(format!("{label}_nul_rejected"));
    }
    Ok(())
}

fn contains_sensitive_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "authorization:",
        "bearer ",
        "api_key",
        "api-key",
        "api key",
        "password",
        "credential",
        "secret",
        "sk-",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn scrub_provider_claim(value: &str) -> String {
    if contains_sensitive_marker(value) {
        "[redacted-sensitive-content]".to_string()
    } else {
        value.to_string()
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeKind {
    ModelInteraction,
    OperationChain,
    WorkflowProgression,
    HandoffProgression,
    HumanInput,
    CaseLifecycle,
    StructuralEvent,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeCompletionPosture {
    Completed,
    BlockedReview,
    BlockedProvider,
    EffectIndeterminate,
    Denied,
    Cancelled,
    Failed,
    HandoffWaiting,
    Open,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EpisodeStructuralSummary {
    pub transition_kinds: Vec<String>,
    pub attempted_work: Vec<String>,
    pub observed_outcomes: Vec<String>,
    pub unresolved_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryEpisode {
    pub schema: String,
    pub episode_id: String,
    pub case_id: String,
    pub derivation_version: String,
    pub boundary_policy_version: String,
    pub start_generation: u64,
    pub end_generation: u64,
    pub transition_ids: Vec<String>,
    pub participant_refs: Vec<String>,
    pub resource_refs: Vec<String>,
    pub workflow_refs: Vec<String>,
    pub operation_refs: Vec<String>,
    pub decision_refs: Vec<String>,
    pub review_refs: Vec<String>,
    pub effect_refs: Vec<String>,
    pub handoff_refs: Vec<String>,
    pub provider_refs: Vec<String>,
    pub episode_kind: EpisodeKind,
    pub completion_posture: EpisodeCompletionPosture,
    pub unresolved_refs: Vec<String>,
    pub structural_summary: EpisodeStructuralSummary,
    pub provenance_digest: String,
}

impl MemoryEpisode {
    pub fn validate(&self, history: &[Transition]) -> Result<(), String> {
        if self.schema != MEMORY_EPISODE_SCHEMA
            || self.derivation_version != EPISODE_DERIVATION_VERSION
            || self.boundary_policy_version != EPISODE_BOUNDARY_POLICY
            || self.transition_ids.is_empty()
            || self.start_generation == 0
            || self.start_generation > self.end_generation
        {
            return Err("memory_episode_contract_invalid".to_string());
        }
        let source = history
            .iter()
            .filter(|transition| self.transition_ids.contains(&transition.transition_id))
            .cloned()
            .collect::<Vec<_>>();
        if source.len() != self.transition_ids.len()
            || source
                .iter()
                .any(|transition| transition.case_id != self.case_id)
        {
            return Err("memory_episode_source_missing_or_cross_case".to_string());
        }
        let rebuilt = derive_episodes(&self.case_id, history)?
            .into_iter()
            .find(|episode| episode.episode_id == self.episode_id)
            .ok_or_else(|| "memory_episode_source_divergent".to_string())?;
        if rebuilt != *self {
            return Err("memory_episode_source_divergent".to_string());
        }
        Ok(())
    }
}

fn experience_bearing(payload: &TransitionPayload) -> bool {
    !matches!(
        payload,
        TransitionPayload::CaseOpened { .. }
            | TransitionPayload::TenantCaseOpened { .. }
            | TransitionPayload::ParticipantBound { .. }
            | TransitionPayload::ParticipantAdmitted { .. }
            | TransitionPayload::ParticipantPrincipalLinked { .. }
            | TransitionPayload::ProviderAttached { .. }
            | TransitionPayload::CaseProviderBindingRecorded { .. }
            | TransitionPayload::ResourceAttached { .. }
            | TransitionPayload::CasePolicyBound { .. }
            | TransitionPayload::CasePolicyReplaced { .. }
            | TransitionPayload::CasePolicyUnbound { .. }
            | TransitionPayload::CaseWorkflowBound { .. }
    )
}

fn strong_reference_key(key: &str) -> bool {
    matches!(
        key,
        "invocation_id"
            | "result_id"
            | "interpretation_id"
            | "turn_id"
            | "logical_turn_id"
            | "selection_id"
            | "operation_id"
            | "decision_id"
            | "grant_id"
            | "effect_id"
            | "receipt_id"
            | "review_id"
            | "action_id"
            | "execution_id"
            | "satisfaction_id"
            | "resolution_id"
            | "input_id"
            | "proposal_id"
            | "patch_id"
            | "amendment_id"
            | "handoff_id"
            | "acceptance_id"
            | "decline_id"
            | "reconciliation_id"
    )
}

fn collect_named_strings(value: &Value, keys: &mut BTreeMap<String, BTreeSet<String>>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if let Some(text) = value.as_str() {
                    keys.entry(key.clone())
                        .or_default()
                        .insert(text.to_string());
                }
                collect_named_strings(value, keys);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_named_strings(value, keys);
            }
        }
        _ => {}
    }
}

fn transition_fields(
    transition: &Transition,
) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let value = serde_json::to_value(&transition.payload)
        .map_err(|error| format!("memory_episode_transition_encode_failed: {error}"))?;
    let mut fields = BTreeMap::new();
    collect_named_strings(&value, &mut fields);
    Ok(fields)
}

fn episode_kind(kinds: &BTreeSet<String>) -> EpisodeKind {
    if kinds.iter().any(|kind| kind.starts_with("handoff_")) {
        EpisodeKind::HandoffProgression
    } else if kinds.contains("workflow_human_input_recorded") {
        EpisodeKind::HumanInput
    } else if kinds.iter().any(|kind| kind.starts_with("workflow_")) {
        EpisodeKind::WorkflowProgression
    } else if kinds.iter().any(|kind| {
        kind.contains("operation")
            || kind.contains("decision")
            || kind.contains("effect")
            || kind.contains("review")
            || kind.contains("grant")
    }) {
        EpisodeKind::OperationChain
    } else if kinds.iter().any(|kind| {
        kind.starts_with("provider_")
            || kind == "interaction_turn_recorded"
            || kind == "model_interpretation_recorded"
    }) {
        EpisodeKind::ModelInteraction
    } else if kinds.contains("case_closed") || kinds.contains("case_cancellation_requested") {
        EpisodeKind::CaseLifecycle
    } else {
        EpisodeKind::StructuralEvent
    }
}

fn episode_posture(
    transitions: &[&Transition],
    kinds: &BTreeSet<String>,
    fields: &BTreeMap<String, BTreeSet<String>>,
) -> EpisodeCompletionPosture {
    if kinds.contains("case_cancellation_requested") {
        return EpisodeCompletionPosture::Cancelled;
    }
    if transitions.iter().any(|transition| {
        matches!(
            &transition.payload,
            TransitionPayload::DecisionRecorded { decision }
                if decision.outcome == crate::effect::DecisionOutcome::Deny
        )
    }) {
        return EpisodeCompletionPosture::Denied;
    }
    if kinds.contains("effect_indeterminate") || kinds.contains("process_effect_indeterminate") {
        return EpisodeCompletionPosture::EffectIndeterminate;
    }
    if transitions.iter().any(|transition| {
        matches!(
            &transition.payload,
            TransitionPayload::EffectFinalized { receipt, .. }
                if matches!(receipt.outcome, crate::effect::EffectOutcome::FailedNoEffect | crate::effect::EffectOutcome::Conflict)
        )
    }) {
        return EpisodeCompletionPosture::Failed;
    }
    if kinds.contains("review_requested")
        && !kinds.contains("review_action_recorded")
        && !kinds.contains("review_resolved")
    {
        return EpisodeCompletionPosture::BlockedReview;
    }
    if kinds.contains("provider_invocation_started") && !kinds.contains("provider_result_recorded")
    {
        return EpisodeCompletionPosture::BlockedProvider;
    }
    if (kinds.contains("handoff_offered") || kinds.contains("handoff_accepted"))
        && !kinds.contains("handoff_declined")
        && !kinds.contains("handoff_result_recorded")
        && !kinds.contains("handoff_reconciled")
    {
        return EpisodeCompletionPosture::HandoffWaiting;
    }
    let terminal = kinds.contains("provider_result_recorded")
        || kinds.contains("effect_finalized")
        || kinds.contains("process_effect_finalized")
        || kinds.contains("workflow_node_satisfied")
        || kinds.contains("handoff_result_recorded")
        || kinds.contains("handoff_declined")
        || kinds.contains("handoff_reconciled")
        || kinds.contains("case_closed")
        || kinds.contains("decision_recorded") && fields.get("outcome").is_some();
    if terminal {
        EpisodeCompletionPosture::Completed
    } else {
        EpisodeCompletionPosture::Open
    }
}

fn field_values(fields: &BTreeMap<String, BTreeSet<String>>, names: &[&str]) -> Vec<String> {
    names
        .iter()
        .filter_map(|name| fields.get(*name))
        .flatten()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Deterministically partitions experience-bearing transitions by connected
/// structural identity. Shared participant/Case/resource names are explicitly
/// excluded from connectivity so unrelated work cannot collapse into one
/// giant episode.
pub fn derive_episodes(
    case_id: &str,
    history: &[Transition],
) -> Result<Vec<MemoryEpisode>, String> {
    if case_id.trim().is_empty()
        || history
            .iter()
            .any(|transition| transition.case_id != case_id)
        || history
            .windows(2)
            .any(|pair| pair[0].sequence >= pair[1].sequence)
    {
        return Err("memory_episode_history_invalid".to_string());
    }
    let candidates = history
        .iter()
        .filter(|transition| experience_bearing(&transition.payload))
        .collect::<Vec<_>>();
    let fields = candidates
        .iter()
        .map(|transition| transition_fields(transition))
        .collect::<Result<Vec<_>, _>>()?;
    let mut parent = (0..candidates.len()).collect::<Vec<_>>();
    fn root(parent: &mut [usize], mut index: usize) -> usize {
        while parent[index] != index {
            parent[index] = parent[parent[index]];
            index = parent[index];
        }
        index
    }
    let mut seen = BTreeMap::<String, usize>::new();
    for (index, item) in fields.iter().enumerate() {
        for (key, values) in item {
            if !strong_reference_key(key) {
                continue;
            }
            for value in values {
                let token = format!("{key}\0{value}");
                if let Some(previous) = seen.get(&token).copied() {
                    let left = root(&mut parent, previous);
                    let right = root(&mut parent, index);
                    if left != right {
                        parent[right] = left;
                    }
                } else {
                    seen.insert(token, index);
                }
            }
        }
    }
    let mut groups = BTreeMap::<usize, Vec<usize>>::new();
    for index in 0..candidates.len() {
        let key = root(&mut parent, index);
        groups.entry(key).or_default().push(index);
    }
    let mut episodes = Vec::with_capacity(groups.len());
    for indexes in groups.values() {
        let source = indexes
            .iter()
            .map(|index| candidates[*index])
            .collect::<Vec<_>>();
        let mut combined = BTreeMap::<String, BTreeSet<String>>::new();
        let mut kinds = BTreeSet::new();
        for index in indexes {
            kinds.insert(candidates[*index].payload.kind().to_string());
            for (key, values) in &fields[*index] {
                combined
                    .entry(key.clone())
                    .or_default()
                    .extend(values.iter().cloned());
            }
        }
        let transition_ids = source
            .iter()
            .map(|item| item.transition_id.clone())
            .collect::<Vec<_>>();
        let start_generation = source.first().map(|item| item.sequence).unwrap_or(0);
        let end_generation = source.last().map(|item| item.sequence).unwrap_or(0);
        let mut participant_sets = source
            .iter()
            .zip(indexes.iter())
            .map(|(transition, index)| {
                let mut values = transition
                    .scope
                    .as_ref()
                    .map(|scope| {
                        scope
                            .participant_refs
                            .iter()
                            .cloned()
                            .collect::<BTreeSet<_>>()
                    })
                    .unwrap_or_default();
                if let Some(participant) = transition.source.participant_id.as_ref() {
                    values.insert(participant.clone());
                }
                values.extend(field_values(
                    &fields[*index],
                    &[
                        "participant_id",
                        "reviewer_participant_id",
                        "accepted_by_participant_id",
                        "recorded_by_participant_id",
                    ],
                ));
                values
            })
            .filter(|values| !values.is_empty());
        let participant_refs = participant_sets
            .next()
            .map(|first| {
                participant_sets.fold(first, |current, next| {
                    current.intersection(&next).cloned().collect()
                })
            })
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        let mut resource_refs = source
            .iter()
            .filter_map(|transition| transition.scope.as_ref())
            .flat_map(|scope| scope.resource_refs.iter().cloned())
            .collect::<BTreeSet<_>>();
        resource_refs.extend(field_values(&combined, &["resource_attachment_id"]));
        let posture = episode_posture(&source, &kinds, &combined);
        let unresolved_refs = if matches!(
            posture,
            EpisodeCompletionPosture::Open
                | EpisodeCompletionPosture::BlockedReview
                | EpisodeCompletionPosture::BlockedProvider
                | EpisodeCompletionPosture::EffectIndeterminate
                | EpisodeCompletionPosture::HandoffWaiting
        ) {
            field_values(
                &combined,
                &["review_id", "effect_id", "invocation_id", "handoff_id"],
            )
        } else {
            Vec::new()
        };
        let attempted_work = field_values(
            &combined,
            &["operation_id", "execution_id", "proposal_id", "handoff_id"],
        );
        let observed_outcomes = field_values(
            &combined,
            &[
                "receipt_id",
                "post_observation_id",
                "result_id",
                "resolution_id",
            ],
        );
        let structural_summary = EpisodeStructuralSummary {
            transition_kinds: kinds.iter().cloned().collect(),
            attempted_work,
            observed_outcomes,
            unresolved_refs: unresolved_refs.clone(),
        };
        let provenance_digest = digest_json(
            &(
                EPISODE_DERIVATION_VERSION,
                EPISODE_BOUNDARY_POLICY,
                case_id,
                start_generation,
                end_generation,
                &transition_ids,
                &structural_summary,
            ),
            "memory_episode_provenance",
        )?;
        episodes.push(MemoryEpisode {
            schema: MEMORY_EPISODE_SCHEMA.to_string(),
            episode_id: short_id("memory-episode", &provenance_digest),
            case_id: case_id.to_string(),
            derivation_version: EPISODE_DERIVATION_VERSION.to_string(),
            boundary_policy_version: EPISODE_BOUNDARY_POLICY.to_string(),
            start_generation,
            end_generation,
            transition_ids,
            participant_refs,
            resource_refs: resource_refs.into_iter().collect(),
            workflow_refs: field_values(
                &combined,
                &["binding_id", "workflow_definition_id", "node_id"],
            ),
            operation_refs: field_values(&combined, &["operation_id"]),
            decision_refs: field_values(&combined, &["decision_id"]),
            review_refs: field_values(&combined, &["review_id", "action_id"]),
            effect_refs: field_values(&combined, &["effect_id", "receipt_id"]),
            handoff_refs: field_values(&combined, &["handoff_id", "acceptance_id", "decline_id"]),
            provider_refs: field_values(&combined, &["selection_id", "invocation_id", "result_id"]),
            episode_kind: episode_kind(&kinds),
            completion_posture: posture,
            unresolved_refs,
            structural_summary,
            provenance_digest,
        });
    }
    episodes.sort_by(|left, right| {
        left.start_generation
            .cmp(&right.start_generation)
            .then_with(|| left.episode_id.cmp(&right.episode_id))
    });
    Ok(episodes)
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum SemanticSubject {
    Case(String),
    Participant(String),
    ResourceAttachment(String),
    Workflow(String),
    WorkflowNode(String),
    Operation(String),
    Review(String),
    Effect(String),
    Handoff(String),
    ProviderTarget(String),
    NamedEntity(String),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum SemanticValue {
    Boolean(bool),
    Integer(i64),
    String(String),
    Reference(String),
    Digest(String),
    Symbol(String),
    StringList(Vec<String>),
    ReferenceList(Vec<String>),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicClass {
    MechanicallyGrounded,
    EvidenceBoundInference,
    ProviderOriginatedClaim,
    ControlHistory,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticLifecycle {
    Active,
    Contradicted,
    Superseded,
    Historical,
    Invalid,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "family", content = "id", rename_all = "snake_case")]
pub enum SemanticSupportRef {
    Transition(String),
    Operational(String),
    Episode(String),
    Assertion(String),
    ProviderResult(String),
}

impl SemanticSupportRef {
    pub fn id(&self) -> &str {
        match self {
            Self::Transition(id)
            | Self::Operational(id)
            | Self::Episode(id)
            | Self::Assertion(id)
            | Self::ProviderResult(id) => id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticMemoryAssertion {
    pub schema: String,
    pub assertion_id: String,
    pub case_id: String,
    pub semantic_schema_version: String,
    pub subject: SemanticSubject,
    pub predicate: String,
    pub value: SemanticValue,
    pub epistemic_class: EpistemicClass,
    pub support_refs: Vec<SemanticSupportRef>,
    pub source_generation_start: u64,
    pub source_generation_end: u64,
    pub origin: String,
    pub lifecycle: SemanticLifecycle,
    pub contradiction_set_ref: Option<String>,
    pub supersession_refs: Vec<String>,
    pub participant_refs: Vec<String>,
    pub assertion_digest: String,
}

impl SemanticMemoryAssertion {
    #[allow(clippy::too_many_arguments)]
    fn build(
        case_id: &str,
        subject: SemanticSubject,
        predicate: String,
        value: SemanticValue,
        epistemic_class: EpistemicClass,
        mut support_refs: Vec<SemanticSupportRef>,
        source_generation_start: u64,
        source_generation_end: u64,
        origin: String,
        mut participant_refs: Vec<String>,
    ) -> Result<Self, String> {
        validate_subject(&subject)?;
        validate_predicate(&predicate)?;
        validate_value(&value)?;
        if support_refs.is_empty() || support_refs.len() > MAX_SUPPORT_REFS {
            return Err("semantic_assertion_support_bounds_invalid".to_string());
        }
        support_refs.sort();
        support_refs.dedup();
        participant_refs.sort();
        participant_refs.dedup();
        let identity_digest = digest_json(
            &(
                SEMANTIC_ASSERTION_SCHEMA,
                case_id,
                &subject,
                &predicate,
                &value,
                &epistemic_class,
                &support_refs,
                source_generation_start,
                source_generation_end,
                &participant_refs,
            ),
            "semantic_assertion_identity",
        )?;
        let assertion_id = short_id("semantic-assertion", &identity_digest);
        let lifecycle = SemanticLifecycle::Active;
        let assertion_digest = digest_json(
            &(
                &assertion_id,
                &lifecycle,
                Option::<String>::None,
                Vec::<String>::new(),
            ),
            "semantic_assertion",
        )?;
        Ok(Self {
            schema: SEMANTIC_ASSERTION_SCHEMA.to_string(),
            assertion_id,
            case_id: case_id.to_string(),
            semantic_schema_version: SEMANTIC_ASSERTION_SCHEMA.to_string(),
            subject,
            predicate,
            value,
            epistemic_class,
            support_refs,
            source_generation_start,
            source_generation_end,
            origin,
            lifecycle,
            contradiction_set_ref: None,
            supersession_refs: Vec::new(),
            participant_refs,
            assertion_digest,
        })
    }

    fn seal_posture(&mut self) -> Result<(), String> {
        self.assertion_digest = digest_json(
            &(
                &self.assertion_id,
                &self.lifecycle,
                &self.contradiction_set_ref,
                &self.supersession_refs,
            ),
            "semantic_assertion",
        )?;
        Ok(())
    }
}

fn validate_subject(subject: &SemanticSubject) -> Result<(), String> {
    let encoded = serde_json::to_vec(subject)
        .map_err(|error| format!("semantic_subject_encode_failed: {error}"))?;
    if encoded.len() > MAX_SUBJECT_BYTES {
        return Err("semantic_subject_too_large".to_string());
    }
    let id = match subject {
        SemanticSubject::Case(id)
        | SemanticSubject::Participant(id)
        | SemanticSubject::ResourceAttachment(id)
        | SemanticSubject::Workflow(id)
        | SemanticSubject::WorkflowNode(id)
        | SemanticSubject::Operation(id)
        | SemanticSubject::Review(id)
        | SemanticSubject::Effect(id)
        | SemanticSubject::Handoff(id)
        | SemanticSubject::ProviderTarget(id)
        | SemanticSubject::NamedEntity(id) => id,
    };
    bounded_text(id, MAX_SUBJECT_BYTES, "semantic_subject")?;
    if contains_sensitive_marker(id) {
        Err("semantic_subject_sensitive_marker_rejected".to_string())
    } else {
        Ok(())
    }
}

fn validate_predicate(predicate: &str) -> Result<(), String> {
    bounded_text(predicate, MAX_PREDICATE_BYTES, "semantic_predicate")?;
    if predicate.split('.').any(|part| {
        part.is_empty()
            || !part
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    }) {
        return Err("semantic_predicate_namespace_invalid".to_string());
    }
    Ok(())
}

fn validate_value(value: &SemanticValue) -> Result<(), String> {
    let encoded = serde_json::to_vec(value)
        .map_err(|error| format!("semantic_value_encode_failed: {error}"))?;
    if encoded.len() > MAX_VALUE_BYTES {
        return Err("semantic_value_too_large".to_string());
    }
    match value {
        SemanticValue::String(value)
        | SemanticValue::Reference(value)
        | SemanticValue::Digest(value)
        | SemanticValue::Symbol(value) => {
            bounded_text(value, MAX_VALUE_BYTES, "semantic_value")?;
            if contains_sensitive_marker(value) {
                Err("semantic_value_sensitive_marker_rejected".to_string())
            } else {
                Ok(())
            }
        }
        SemanticValue::StringList(values) | SemanticValue::ReferenceList(values) => {
            if values.len() > MAX_SUPPORT_REFS
                || values.iter().any(|value| value.trim().is_empty())
                || values.iter().any(|value| contains_sensitive_marker(value))
            {
                Err("semantic_value_list_bounds_invalid".to_string())
            } else {
                Ok(())
            }
        }
        SemanticValue::Boolean(_) | SemanticValue::Integer(_) => Ok(()),
    }
}

fn participants_for_memory(entry: &OperationalMemoryEntry) -> Vec<String> {
    entry.visibility.participant_ids.clone()
}

pub fn extract_mechanical_assertions(
    case_id: &str,
    memory: &OperationalMemoryBuild,
) -> Result<Vec<SemanticMemoryAssertion>, String> {
    let mut assertions = Vec::new();
    for entry in &memory.entries {
        if entry.case_id != case_id {
            return Err("semantic_mechanical_source_cross_case".to_string());
        }
        let support = vec![SemanticSupportRef::Operational(entry.memory_id.clone())];
        let (subject, predicate, value, class) = match &entry.value {
            OperationalMemoryValue::ResourceEffect {
                resource_attachment_id,
                content_digest,
                outcome,
                ..
            } => {
                assertions.push(SemanticMemoryAssertion::build(
                    case_id,
                    SemanticSubject::ResourceAttachment(resource_attachment_id.clone()),
                    "effect.outcome".to_string(),
                    SemanticValue::Symbol(format!("{outcome:?}").to_ascii_lowercase()),
                    EpistemicClass::MechanicallyGrounded,
                    support.clone(),
                    entry.provenance.generation_start,
                    entry.provenance.generation_end,
                    "deterministic_operational_extractor.v1".to_string(),
                    participants_for_memory(entry),
                )?);
                let Some(digest) = content_digest else {
                    continue;
                };
                (
                    SemanticSubject::ResourceAttachment(resource_attachment_id.clone()),
                    "resource.content_digest".to_string(),
                    SemanticValue::Digest(digest.clone()),
                    EpistemicClass::MechanicallyGrounded,
                )
            }
            OperationalMemoryValue::Decision {
                operation_id,
                outcome,
                ..
            } => (
                SemanticSubject::Operation(operation_id.clone()),
                "decision.outcome".to_string(),
                SemanticValue::Symbol(format!("{outcome:?}").to_ascii_lowercase()),
                EpistemicClass::ControlHistory,
            ),
            OperationalMemoryValue::Review {
                review_id, status, ..
            } => (
                SemanticSubject::Review(review_id.clone()),
                "review.resolution".to_string(),
                SemanticValue::Symbol(status.to_ascii_lowercase()),
                EpistemicClass::ControlHistory,
            ),
            OperationalMemoryValue::UnresolvedEffect {
                effect_id, state, ..
            } => (
                SemanticSubject::Effect(effect_id.clone()),
                "effect.outcome".to_string(),
                SemanticValue::Symbol(state.to_ascii_lowercase()),
                EpistemicClass::MechanicallyGrounded,
            ),
            OperationalMemoryValue::NormalizationFailure {
                provider_result_id,
                code,
                ..
            } => (
                SemanticSubject::ProviderTarget(provider_result_id.clone()),
                "provider.normalization_failure".to_string(),
                SemanticValue::Symbol(code.to_ascii_lowercase()),
                EpistemicClass::ControlHistory,
            ),
            OperationalMemoryValue::ProviderClaim {
                provider_id,
                preview,
                ..
            } => (
                SemanticSubject::ProviderTarget(provider_id.clone()),
                "provider.claim".to_string(),
                SemanticValue::String(scrub_provider_claim(preview)),
                EpistemicClass::ProviderOriginatedClaim,
            ),
        };
        assertions.push(SemanticMemoryAssertion::build(
            case_id,
            subject,
            predicate,
            value,
            class,
            support,
            entry.provenance.generation_start,
            entry.provenance.generation_end,
            "deterministic_operational_extractor.v1".to_string(),
            participants_for_memory(entry),
        )?);
    }
    assertions.sort_by(|left, right| left.assertion_id.cmp(&right.assertion_id));
    assertions.dedup_by(|left, right| left.assertion_id == right.assertion_id);
    Ok(assertions)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryConsolidationInput {
    pub schema: String,
    pub input_id: String,
    pub case_id: String,
    pub case_generation: u64,
    pub participant_id: String,
    pub purpose: String,
    pub episode_ids: Vec<String>,
    pub operational_memory_ids: Vec<String>,
    pub existing_semantic_assertion_ids: Vec<String>,
    pub allowed_support_refs: Vec<SemanticSupportRef>,
    pub source_digest: String,
    pub maximum_assertion_count: usize,
    pub maximum_support_refs: usize,
    pub output_contract: String,
    pub semantic_unit_budget: usize,
}

pub fn derive_consolidation_input(
    case_id: &str,
    case_generation: u64,
    participant_id: &str,
    episodes: &[MemoryEpisode],
    operational: &[OperationalMemoryEntry],
    semantic: &[SemanticMemoryAssertion],
) -> Result<MemoryConsolidationInput, String> {
    bounded_text(case_id, 256, "consolidation_case")?;
    bounded_text(participant_id, 256, "consolidation_participant")?;
    if episodes.iter().any(|episode| episode.case_id != case_id)
        || operational.iter().any(|entry| entry.case_id != case_id)
        || semantic
            .iter()
            .any(|assertion| assertion.case_id != case_id)
    {
        return Err("memory_consolidation_cross_case_source_rejected".to_string());
    }
    let mut episode_ids = episodes
        .iter()
        .filter(|episode| {
            episode
                .participant_refs
                .contains(&participant_id.to_string())
        })
        .rev()
        .take(MAX_CONSOLIDATION_EPISODES)
        .map(|episode| episode.episode_id.clone())
        .collect::<Vec<_>>();
    episode_ids.reverse();
    let mut operational_memory_ids = operational
        .iter()
        .filter(|entry| {
            entry
                .visibility
                .participant_ids
                .contains(&participant_id.to_string())
        })
        .rev()
        .take(MAX_CONSOLIDATION_OPERATIONAL_ITEMS)
        .map(|entry| entry.memory_id.clone())
        .collect::<Vec<_>>();
    operational_memory_ids.reverse();
    let mut visible_semantic = semantic
        .iter()
        .filter(|entry| entry.participant_refs.contains(&participant_id.to_string()))
        .collect::<Vec<_>>();
    visible_semantic.sort_by(|left, right| {
        right
            .source_generation_end
            .cmp(&left.source_generation_end)
            .then_with(|| left.assertion_id.cmp(&right.assertion_id))
    });
    let mut existing_semantic_assertion_ids = visible_semantic
        .into_iter()
        .take(MAX_CONSOLIDATION_EXISTING_ASSERTIONS)
        .map(|entry| entry.assertion_id.clone())
        .collect::<Vec<_>>();
    existing_semantic_assertion_ids.sort();
    let mut allowed_support_refs = episode_ids
        .iter()
        .cloned()
        .map(SemanticSupportRef::Episode)
        .chain(
            operational_memory_ids
                .iter()
                .cloned()
                .map(SemanticSupportRef::Operational),
        )
        .chain(
            existing_semantic_assertion_ids
                .iter()
                .cloned()
                .map(SemanticSupportRef::Assertion),
        )
        .collect::<Vec<_>>();
    allowed_support_refs.sort();
    let source_digest = digest_json(
        &(
            case_id,
            case_generation,
            participant_id,
            &episode_ids,
            &operational_memory_ids,
            &existing_semantic_assertion_ids,
            CONSOLIDATION_NORMALIZER_VERSION,
        ),
        "memory_consolidation_source",
    )?;
    let identity = digest_json(
        &(
            CONSOLIDATION_INPUT_SCHEMA,
            case_id,
            case_generation,
            participant_id,
            &source_digest,
            MAX_CONSOLIDATION_ASSERTIONS,
            MAX_SUPPORT_REFS,
        ),
        "memory_consolidation_input",
    )?;
    Ok(MemoryConsolidationInput {
        schema: CONSOLIDATION_INPUT_SCHEMA.to_string(),
        input_id: short_id("memory-consolidation-input", &identity),
        case_id: case_id.to_string(),
        case_generation,
        participant_id: participant_id.to_string(),
        purpose: "memory_consolidation".to_string(),
        episode_ids,
        operational_memory_ids,
        existing_semantic_assertion_ids,
        allowed_support_refs,
        source_digest,
        maximum_assertion_count: MAX_CONSOLIDATION_ASSERTIONS,
        maximum_support_refs: MAX_SUPPORT_REFS,
        output_contract: CONSOLIDATION_CANDIDATE_SCHEMA.to_string(),
        semantic_unit_budget: CONSOLIDATION_SEMANTIC_UNIT_BUDGET,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsolidationCandidateAssertion {
    pub subject: SemanticSubject,
    pub predicate: String,
    pub value: SemanticValue,
    pub support_refs: Vec<SemanticSupportRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryConsolidationCandidate {
    pub schema: String,
    pub case_id: String,
    pub consolidation_input_id: String,
    #[serde(default)]
    pub episode_narratives: Vec<EpisodeNarrativeCandidate>,
    pub assertions: Vec<ConsolidationCandidateAssertion>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeNarrativeCandidate {
    pub episode_id: String,
    pub narrative: String,
    pub support_refs: Vec<SemanticSupportRef>,
}

pub fn normalize_consolidation_candidate(
    input: &MemoryConsolidationInput,
    provider_result_id: &str,
    output: &str,
    episodes: &[MemoryEpisode],
    operational: &[OperationalMemoryEntry],
    existing: &[SemanticMemoryAssertion],
) -> Result<Vec<SemanticMemoryAssertion>, String> {
    if output.len() > MAX_CONSOLIDATION_RESULT_BYTES {
        return Err("memory_consolidation_result_too_large".to_string());
    }
    let candidate: MemoryConsolidationCandidate = serde_json::from_str(output)
        .map_err(|error| format!("memory_consolidation_candidate_invalid: {error}"))?;
    if candidate.schema != CONSOLIDATION_CANDIDATE_SCHEMA
        || candidate.case_id != input.case_id
        || candidate.consolidation_input_id != input.input_id
        || candidate.assertions.len() > input.maximum_assertion_count
        || candidate.episode_narratives.len() > input.episode_ids.len()
    {
        return Err("memory_consolidation_candidate_contract_mismatch".to_string());
    }
    let allowed = input
        .allowed_support_refs
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for narrative in &candidate.episode_narratives {
        if !input.episode_ids.contains(&narrative.episode_id)
            || narrative.narrative.len() > MAX_NARRATIVE_BYTES
            || narrative.narrative.contains('\0')
            || narrative.support_refs.is_empty()
            || narrative.support_refs.len() > input.maximum_support_refs
            || narrative
                .support_refs
                .iter()
                .any(|support| !allowed.contains(support))
        {
            return Err("memory_consolidation_narrative_contract_mismatch".to_string());
        }
    }
    let episode_by_id = episodes
        .iter()
        .map(|episode| (episode.episode_id.as_str(), episode))
        .collect::<BTreeMap<_, _>>();
    let operational_by_id = operational
        .iter()
        .map(|entry| (entry.memory_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let existing_by_id = existing
        .iter()
        .map(|entry| (entry.assertion_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut normalized = Vec::new();
    for value in candidate.assertions {
        if value.support_refs.is_empty()
            || value.support_refs.len() > input.maximum_support_refs
            || value
                .support_refs
                .iter()
                .any(|support| !allowed.contains(support))
        {
            return Err("memory_consolidation_support_not_admitted".to_string());
        }
        let mut participant_intersection: Option<BTreeSet<String>> = None;
        let mut generation_start = input.case_generation;
        let mut generation_end = 0u64;
        for support in &value.support_refs {
            let (participants, start, end) = match support {
                SemanticSupportRef::Episode(id) => {
                    let source = episode_by_id.get(id.as_str()).ok_or_else(|| {
                        "memory_consolidation_episode_support_missing".to_string()
                    })?;
                    if source.case_id != input.case_id
                        || source.end_generation > input.case_generation
                        || !source.participant_refs.contains(&input.participant_id)
                    {
                        return Err(
                            "memory_consolidation_episode_support_not_qualified".to_string()
                        );
                    }
                    (
                        &source.participant_refs,
                        source.start_generation,
                        source.end_generation,
                    )
                }
                SemanticSupportRef::Operational(id) => {
                    let source = operational_by_id.get(id.as_str()).ok_or_else(|| {
                        "memory_consolidation_operational_support_missing".to_string()
                    })?;
                    if source.case_id != input.case_id
                        || source.provenance.generation_end > input.case_generation
                        || !source
                            .visibility
                            .participant_ids
                            .contains(&input.participant_id)
                    {
                        return Err(
                            "memory_consolidation_operational_support_not_qualified".to_string()
                        );
                    }
                    (
                        &source.visibility.participant_ids,
                        source.provenance.generation_start,
                        source.provenance.generation_end,
                    )
                }
                SemanticSupportRef::Assertion(id) => {
                    let source = existing_by_id.get(id.as_str()).ok_or_else(|| {
                        "memory_consolidation_assertion_support_missing".to_string()
                    })?;
                    if source.case_id != input.case_id
                        || source.source_generation_end > input.case_generation
                        || !source.participant_refs.contains(&input.participant_id)
                    {
                        return Err(
                            "memory_consolidation_assertion_support_not_qualified".to_string()
                        );
                    }
                    (
                        &source.participant_refs,
                        source.source_generation_start,
                        source.source_generation_end,
                    )
                }
                SemanticSupportRef::Transition(_) | SemanticSupportRef::ProviderResult(_) => {
                    return Err("memory_consolidation_support_family_not_admitted".to_string())
                }
            };
            let participants = participants.iter().cloned().collect::<BTreeSet<_>>();
            participant_intersection = Some(match participant_intersection {
                Some(current) => current.intersection(&participants).cloned().collect(),
                None => participants,
            });
            generation_start = generation_start.min(start);
            generation_end = generation_end.max(end);
        }
        let participant_refs = participant_intersection
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        if !participant_refs.contains(&input.participant_id) {
            return Err("memory_consolidation_visibility_widening_rejected".to_string());
        }
        normalized.push(SemanticMemoryAssertion::build(
            &input.case_id,
            value.subject,
            value.predicate,
            value.value,
            EpistemicClass::EvidenceBoundInference,
            value.support_refs,
            generation_start,
            generation_end,
            format!("provider_result:{provider_result_id}:normalizer:{CONSOLIDATION_NORMALIZER_VERSION}"),
            participant_refs,
        )?);
    }
    normalized.sort_by(|left, right| left.assertion_id.cmp(&right.assertion_id));
    normalized.dedup_by(|left, right| left.assertion_id == right.assertion_id);
    validate_support_graph(existing, &normalized)?;
    Ok(normalized)
}

pub fn validate_support_graph(
    existing: &[SemanticMemoryAssertion],
    added: &[SemanticMemoryAssertion],
) -> Result<(), String> {
    let all = existing
        .iter()
        .chain(added)
        .map(|assertion| (assertion.assertion_id.as_str(), assertion))
        .collect::<BTreeMap<_, _>>();
    for assertion in added {
        let mut queue = VecDeque::from([(
            assertion.assertion_id.clone(),
            0usize,
            BTreeSet::<String>::new(),
        )]);
        let mut canonical_leaf = false;
        while let Some((id, depth, mut path)) = queue.pop_front() {
            if depth > MAX_SUPPORT_DEPTH {
                return Err("semantic_support_depth_exceeded".to_string());
            }
            if !path.insert(id.clone()) {
                return Err("semantic_support_cycle_rejected".to_string());
            }
            let source = all
                .get(id.as_str())
                .ok_or_else(|| "semantic_support_assertion_missing".to_string())?;
            for support in &source.support_refs {
                match support {
                    SemanticSupportRef::Assertion(parent) => {
                        if parent == &id {
                            return Err("semantic_support_self_cycle_rejected".to_string());
                        }
                        queue.push_back((parent.clone(), depth + 1, path.clone()));
                    }
                    _ => canonical_leaf = true,
                }
            }
        }
        if !canonical_leaf {
            return Err("semantic_support_canonical_leaf_required".to_string());
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticContradictionSet {
    pub schema: String,
    pub contradiction_id: String,
    pub case_id: String,
    pub subject: SemanticSubject,
    pub predicate: String,
    pub competing_assertion_ids: Vec<String>,
    pub contradiction_kind: String,
    pub resolution_posture: String,
    pub mechanical_basis: String,
    pub derivation_version: String,
}

fn is_single_current_state(predicate: &str) -> bool {
    matches!(predicate, "resource.content_digest" | "case.lifecycle")
}

pub fn derive_contradictions(
    assertions: &mut [SemanticMemoryAssertion],
) -> Result<Vec<SemanticContradictionSet>, String> {
    let mut groups = BTreeMap::<(SemanticSubject, String), Vec<usize>>::new();
    for (index, assertion) in assertions.iter().enumerate() {
        groups
            .entry((assertion.subject.clone(), assertion.predicate.clone()))
            .or_default()
            .push(index);
    }
    let mut sets = Vec::new();
    for ((subject, predicate), indexes) in groups {
        let distinct = indexes
            .iter()
            .map(|index| assertions[*index].value.clone())
            .collect::<BTreeSet<_>>();
        if distinct.len() < 2 {
            continue;
        }
        if indexes.len() > MAX_CONTRADICTION_MEMBERS {
            return Err("semantic_contradiction_member_bound_exceeded".to_string());
        }
        let mut competing = indexes
            .iter()
            .map(|index| assertions[*index].assertion_id.clone())
            .collect::<Vec<_>>();
        competing.sort();
        let digest = digest_json(
            &(
                CONTRADICTION_SET_SCHEMA,
                &assertions[indexes[0]].case_id,
                &subject,
                &predicate,
                &competing,
            ),
            "semantic_contradiction",
        )?;
        let contradiction_id = short_id("semantic-contradiction", &digest);
        let grounded = indexes
            .iter()
            .filter(|index| {
                assertions[**index].epistemic_class == EpistemicClass::MechanicallyGrounded
            })
            .copied()
            .collect::<Vec<_>>();
        let newest_grounded_generation = grounded
            .iter()
            .map(|index| assertions[*index].source_generation_end)
            .max();
        let uniquely_newest_grounded = newest_grounded_generation.and_then(|generation| {
            let newest = grounded
                .iter()
                .filter(|index| assertions[**index].source_generation_end == generation)
                .copied()
                .collect::<Vec<_>>();
            (newest.len() == 1).then_some(newest[0])
        });
        let resolution_posture = if let (true, Some(newest)) = (
            is_single_current_state(&predicate),
            uniquely_newest_grounded,
        ) {
            for index in &indexes {
                if *index != newest {
                    assertions[*index].lifecycle = if assertions[*index].epistemic_class
                        == EpistemicClass::MechanicallyGrounded
                    {
                        assertions[*index].supersession_refs =
                            vec![assertions[newest].assertion_id.clone()];
                        SemanticLifecycle::Superseded
                    } else {
                        SemanticLifecycle::Contradicted
                    };
                }
            }
            "mechanically_superseded".to_string()
        } else if is_single_current_state(&predicate) && grounded.len() > 1 {
            for index in &indexes {
                if assertions[*index].epistemic_class != EpistemicClass::MechanicallyGrounded {
                    assertions[*index].lifecycle = SemanticLifecycle::Contradicted;
                }
            }
            "grounded_state_tie_unresolved".to_string()
        } else if !grounded.is_empty() {
            for index in &indexes {
                if assertions[*index].epistemic_class != EpistemicClass::MechanicallyGrounded {
                    assertions[*index].lifecycle = SemanticLifecycle::Contradicted;
                }
            }
            "grounded_assertion_preserved".to_string()
        } else {
            for index in &indexes {
                assertions[*index].lifecycle = SemanticLifecycle::Contradicted;
            }
            "unresolved".to_string()
        };
        for index in &indexes {
            assertions[*index].contradiction_set_ref = Some(contradiction_id.clone());
            assertions[*index].seal_posture()?;
        }
        sets.push(SemanticContradictionSet {
            schema: CONTRADICTION_SET_SCHEMA.to_string(),
            contradiction_id,
            case_id: assertions[indexes[0]].case_id.clone(),
            subject,
            predicate,
            competing_assertion_ids: competing,
            contradiction_kind: "structural_value_conflict".to_string(),
            resolution_posture,
            mechanical_basis: "same_typed_subject_and_predicate_distinct_typed_value.v1"
                .to_string(),
            derivation_version: "yai.semantic_contradiction.derivation.v1".to_string(),
        });
    }
    sets.sort_by(|left, right| left.contradiction_id.cmp(&right.contradiction_id));
    Ok(sets)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryRetentionPolicy {
    pub schema: String,
    pub recent_episode_count: usize,
    pub maximum_default_semantic_items: usize,
    pub preserve_unresolved: bool,
    pub preserve_contradiction_support: bool,
    pub preserve_active_support_ancestors: bool,
    pub historical_retrieval_opt_in: bool,
}

impl Default for MemoryRetentionPolicy {
    fn default() -> Self {
        Self {
            schema: RETENTION_POLICY_SCHEMA.to_string(),
            recent_episode_count: DEFAULT_RECENT_EPISODES,
            maximum_default_semantic_items: 4096,
            preserve_unresolved: true,
            preserve_contradiction_support: true,
            preserve_active_support_ancestors: true,
            historical_retrieval_opt_in: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryHierarchyManifest {
    pub schema: String,
    pub hierarchy_id: String,
    pub case_id: String,
    pub source_generation: u64,
    pub operational_manifest_id: String,
    pub episode_ids: Vec<String>,
    pub active_episode_ids: Vec<String>,
    pub open_episode_ids: Vec<String>,
    pub semantic_assertion_ids: Vec<String>,
    pub contradiction_set_ids: Vec<String>,
    pub active_semantic_ids: Vec<String>,
    pub background_semantic_ids: Vec<String>,
    pub historical_semantic_ids: Vec<String>,
    pub retention_posture: MemoryRetentionPolicy,
    pub hierarchy_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticMemoryHierarchy {
    pub manifest: MemoryHierarchyManifest,
    pub episodes: Vec<MemoryEpisode>,
    pub assertions: Vec<SemanticMemoryAssertion>,
    pub contradictions: Vec<SemanticContradictionSet>,
    pub unresolved_consolidation_result_ids: Vec<String>,
}

fn episode_is_unresolved(episode: &MemoryEpisode) -> bool {
    matches!(
        episode.completion_posture,
        EpisodeCompletionPosture::BlockedReview
            | EpisodeCompletionPosture::BlockedProvider
            | EpisodeCompletionPosture::EffectIndeterminate
            | EpisodeCompletionPosture::HandoffWaiting
            | EpisodeCompletionPosture::Open
    )
}

fn retained_episode_ids(
    episodes: &[MemoryEpisode],
    assertions: &[SemanticMemoryAssertion],
    retained_assertions: &BTreeSet<String>,
) -> Vec<String> {
    let episode_ids = episodes
        .iter()
        .map(|episode| episode.episode_id.as_str())
        .collect::<BTreeSet<_>>();
    episodes
        .iter()
        .rev()
        .take(DEFAULT_RECENT_EPISODES)
        .map(|episode| episode.episode_id.clone())
        .chain(
            episodes
                .iter()
                .filter(|episode| episode_is_unresolved(episode))
                .map(|episode| episode.episode_id.clone()),
        )
        .chain(
            assertions
                .iter()
                .filter(|assertion| retained_assertions.contains(&assertion.assertion_id))
                .flat_map(|assertion| assertion.support_refs.iter())
                .filter_map(|support| match support {
                    SemanticSupportRef::Episode(id) if episode_ids.contains(id.as_str()) => {
                        Some(id.clone())
                    }
                    _ => None,
                }),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn build_memory_hierarchy(
    state: &CaseState,
    history: &[Transition],
    memory: &OperationalMemoryBuild,
) -> Result<SemanticMemoryHierarchy, String> {
    if state.case_id != memory.manifest.case_id
        || state.generation != memory.manifest.source_generation
        || history.last().map(|item| item.sequence).unwrap_or(0) != state.generation
    {
        return Err("memory_hierarchy_source_generation_mismatch".to_string());
    }
    let episodes = derive_episodes(&state.case_id, history)?;
    let mut assertions = extract_mechanical_assertions(&state.case_id, memory)?;
    let mut unresolved = Vec::new();
    // Rebuild normalized provider consolidations in canonical result order.
    for transition in history {
        let TransitionPayload::ProviderResultRecorded {
            result_id,
            invocation_id,
            semantic_lineage: Some(lineage),
            output,
            ..
        } = &transition.payload
        else {
            continue;
        };
        let Some(invocation) = history.iter().find(|candidate| {
            matches!(
                &candidate.payload,
                TransitionPayload::ProviderInvocationStarted { invocation_id: id, .. } if id == invocation_id
            )
        }) else {
            continue;
        };
        let TransitionPayload::ProviderInvocationStarted { participant_id, .. } =
            &invocation.payload
        else {
            unreachable!()
        };
        let prefix = history
            .iter()
            .filter(|item| item.sequence <= lineage.case_generation)
            .cloned()
            .collect::<Vec<_>>();
        let prefix_state = crate::transition::replay_case(&state.case_id, &prefix)?;
        let prefix_memory = crate::memory::derive_operational_memory(&state.case_id, &prefix)?;
        let prefix_episodes = derive_episodes(&state.case_id, &prefix)?;
        let visible_existing = assertions
            .iter()
            .filter(|assertion| assertion.source_generation_end <= lineage.case_generation)
            .cloned()
            .collect::<Vec<_>>();
        let input = derive_consolidation_input(
            &state.case_id,
            prefix_state.generation,
            participant_id,
            &prefix_episodes,
            &prefix_memory.entries,
            &visible_existing,
        )?;
        let expected_contract = crate::context::InvocationOutputContract::MemoryConsolidation {
            schema: CONSOLIDATION_CANDIDATE_SCHEMA.to_string(),
            consolidation_input_id: input.input_id.clone(),
            maximum_assertions: input.maximum_assertion_count,
            maximum_support_refs: input.maximum_support_refs,
            normalizer_version: CONSOLIDATION_NORMALIZER_VERSION.to_string(),
        };
        if lineage.output_contract_id != expected_contract.contract_id() {
            continue;
        }
        match normalize_consolidation_candidate(
            &input,
            result_id,
            output,
            &prefix_episodes,
            &prefix_memory.entries,
            &visible_existing,
        ) {
            Ok(values) => assertions.extend(values),
            Err(_) => unresolved.push(result_id.clone()),
        }
    }
    assertions.sort_by(|left, right| left.assertion_id.cmp(&right.assertion_id));
    assertions.dedup_by(|left, right| left.assertion_id == right.assertion_id);
    let contradictions = derive_contradictions(&mut assertions)?;
    let episode_ids = episodes
        .iter()
        .map(|value| value.episode_id.clone())
        .collect::<Vec<_>>();
    let open_episode_ids = episodes
        .iter()
        .filter(|value| episode_is_unresolved(value))
        .map(|value| value.episode_id.clone())
        .collect::<Vec<_>>();
    let semantic_assertion_ids = assertions
        .iter()
        .map(|value| value.assertion_id.clone())
        .collect::<Vec<_>>();
    let retention_posture = MemoryRetentionPolicy::default();
    let mut retention_candidates = assertions
        .iter()
        .filter(|value| {
            matches!(
                value.lifecycle,
                SemanticLifecycle::Active | SemanticLifecycle::Contradicted
            )
        })
        .collect::<Vec<_>>();
    retention_candidates.sort_by(|left, right| {
        let priority = |value: &SemanticMemoryAssertion| {
            if value.lifecycle == SemanticLifecycle::Contradicted
                || value.contradiction_set_ref.is_some()
            {
                0u8
            } else if value.epistemic_class == EpistemicClass::MechanicallyGrounded {
                1u8
            } else {
                2u8
            }
        };
        priority(left)
            .cmp(&priority(right))
            .then_with(|| right.source_generation_end.cmp(&left.source_generation_end))
            .then_with(|| left.assertion_id.cmp(&right.assertion_id))
    });
    let assertion_by_id = assertions
        .iter()
        .map(|value| (value.assertion_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let mut retained = BTreeSet::<String>::new();
    for candidate in retention_candidates {
        if retained.len() == retention_posture.maximum_default_semantic_items {
            break;
        }
        retained.insert(candidate.assertion_id.clone());
        let mut ancestors = candidate
            .support_refs
            .iter()
            .filter_map(|support| match support {
                SemanticSupportRef::Assertion(id) => Some(id.clone()),
                _ => None,
            })
            .collect::<VecDeque<_>>();
        while let Some(ancestor_id) = ancestors.pop_front() {
            if retained.len() == retention_posture.maximum_default_semantic_items {
                break;
            }
            if !retained.insert(ancestor_id.clone()) {
                continue;
            }
            if let Some(ancestor) = assertion_by_id.get(ancestor_id.as_str()) {
                ancestors.extend(ancestor.support_refs.iter().filter_map(
                    |support| match support {
                        SemanticSupportRef::Assertion(id) => Some(id.clone()),
                        _ => None,
                    },
                ));
            }
        }
    }
    let active_semantic_ids = retained.iter().cloned().collect::<Vec<_>>();
    let active_episode_ids = retained_episode_ids(&episodes, &assertions, &retained);
    let background_semantic_ids = assertions
        .iter()
        .filter(|value| {
            matches!(
                value.lifecycle,
                SemanticLifecycle::Active | SemanticLifecycle::Contradicted
            ) && !retained.contains(&value.assertion_id)
        })
        .map(|value| value.assertion_id.clone())
        .collect::<Vec<_>>();
    let historical_semantic_ids = assertions
        .iter()
        .filter(|value| {
            matches!(
                value.lifecycle,
                SemanticLifecycle::Historical | SemanticLifecycle::Superseded
            )
        })
        .map(|value| value.assertion_id.clone())
        .collect::<Vec<_>>();
    let contradiction_set_ids = contradictions
        .iter()
        .map(|value| value.contradiction_id.clone())
        .collect::<Vec<_>>();
    let operational_manifest_id = short_id(
        "operational-memory-manifest",
        &digest_json(&memory.manifest, "operational_memory_manifest")?,
    );
    let hierarchy_digest = digest_json(
        &(
            MEMORY_HIERARCHY_SCHEMA,
            &state.case_id,
            state.generation,
            &operational_manifest_id,
            &episode_ids,
            &active_episode_ids,
            &open_episode_ids,
            &semantic_assertion_ids,
            &contradiction_set_ids,
            &active_semantic_ids,
            &background_semantic_ids,
            &historical_semantic_ids,
            &retention_posture,
        ),
        "memory_hierarchy",
    )?;
    let manifest = MemoryHierarchyManifest {
        schema: MEMORY_HIERARCHY_SCHEMA.to_string(),
        hierarchy_id: short_id("memory-hierarchy", &hierarchy_digest),
        case_id: state.case_id.clone(),
        source_generation: state.generation,
        operational_manifest_id,
        episode_ids,
        active_episode_ids,
        open_episode_ids,
        semantic_assertion_ids,
        contradiction_set_ids,
        active_semantic_ids,
        background_semantic_ids,
        historical_semantic_ids,
        retention_posture,
        hierarchy_digest,
    };
    Ok(SemanticMemoryHierarchy {
        manifest,
        episodes,
        assertions,
        contradictions,
        unresolved_consolidation_result_ids: unresolved,
    })
}

pub fn assertion_is_visible(assertion: &SemanticMemoryAssertion, participant_id: &str) -> bool {
    assertion
        .participant_refs
        .iter()
        .any(|value| value == participant_id)
}

pub fn episode_is_visible(episode: &MemoryEpisode, participant_id: &str) -> bool {
    episode
        .participant_refs
        .iter()
        .any(|value| value == participant_id)
}

pub fn case_lifecycle_assertion(
    state: &CaseState,
    support_transition_id: &str,
    participant_refs: Vec<String>,
) -> Result<SemanticMemoryAssertion, String> {
    let symbol = match state.lifecycle {
        CaseLifecycle::Open => "open",
        CaseLifecycle::Closed => "closed",
    };
    SemanticMemoryAssertion::build(
        &state.case_id,
        SemanticSubject::Case(state.case_id.clone()),
        "case.lifecycle".to_string(),
        SemanticValue::Symbol(symbol.to_string()),
        EpistemicClass::MechanicallyGrounded,
        vec![SemanticSupportRef::Transition(
            support_transition_id.to_string(),
        )],
        state.generation,
        state.generation,
        "deterministic_case_state_extractor.v1".to_string(),
        participant_refs,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transition::TransitionSource;

    fn committed(case_id: &str, sequence: u64, payload: TransitionPayload) -> Transition {
        Transition {
            schema: crate::transition::TRANSITION_SCHEMA.to_string(),
            transition_id: format!("transition:{sequence}"),
            case_id: case_id.to_string(),
            sequence,
            committed_at_unix_ms: 0,
            source: TransitionSource {
                component: "test".to_string(),
                participant_id: Some("participant:test".to_string()),
                principal_id: None,
                source_ref: None,
            },
            scope: None,
            causal_refs: Vec::new(),
            payload,
            provenance: Vec::new(),
            summary: None,
        }
    }

    fn open_provider_episode(case_id: &str, participant: &str) -> MemoryEpisode {
        derive_episodes(
            case_id,
            &[committed(
                case_id,
                1,
                TransitionPayload::ProviderInvocationStarted {
                    invocation_id: "provider-invocation:test".to_string(),
                    participant_id: participant.to_string(),
                    provider_id: "provider-target:test".to_string(),
                    provider_kind: "openai_compatible".to_string(),
                    model_id: "model:test".to_string(),
                    semantic_lineage: None,
                    governance: None,
                },
            )],
        )
        .unwrap()
        .remove(0)
    }

    fn assertion(
        predicate: &str,
        value: i64,
        class: EpistemicClass,
        generation: u64,
    ) -> SemanticMemoryAssertion {
        SemanticMemoryAssertion::build(
            "case:test",
            SemanticSubject::NamedEntity("project:test".to_string()),
            predicate.to_string(),
            SemanticValue::Integer(value),
            class,
            vec![SemanticSupportRef::Transition(format!(
                "transition:{generation}"
            ))],
            generation,
            generation,
            "test".to_string(),
            vec!["participant:test".to_string()],
        )
        .unwrap()
    }

    #[test]
    fn w20_e01_e02_e03_episode_identity_is_structural_and_provider_independent() {
        let case_id = "case:test";
        let history = vec![committed(
            case_id,
            1,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: "provider-invocation:test".to_string(),
                participant_id: "participant:test".to_string(),
                provider_id: "provider-target:a".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:a".to_string(),
                semantic_lineage: None,
                governance: None,
            },
        )];
        assert_eq!(
            derive_episodes(case_id, &history).unwrap(),
            derive_episodes(case_id, &history).unwrap()
        );
        let episodes = derive_episodes(case_id, &history).unwrap();
        assert_eq!(episodes.len(), 1);
        assert_eq!(
            episodes[0].completion_posture,
            EpisodeCompletionPosture::BlockedProvider
        );
        assert_eq!(episodes[0].start_generation, 1);
        assert_eq!(episodes[0].end_generation, 1);
    }

    #[test]
    fn w20_s04_provider_cannot_choose_epistemic_class() {
        let input =
            derive_consolidation_input("case:test", 1, "participant:test", &[], &[], &[]).unwrap();
        let forged = format!(
            "{{\"schema\":\"{}\",\"case_id\":\"case:test\",\"consolidation_input_id\":\"{}\",\"episode_narratives\":[],\"assertions\":[{{\"subject\":{{\"kind\":\"case\",\"id\":\"case:test\"}},\"predicate\":\"open.fact\",\"value\":{{\"type\":\"boolean\",\"value\":true}},\"support_refs\":[],\"epistemic_class\":\"mechanically_grounded\"}}]}}",
            CONSOLIDATION_CANDIDATE_SCHEMA, input.input_id
        );
        assert!(
            normalize_consolidation_candidate(&input, "result:test", &forged, &[], &[], &[])
                .unwrap_err()
                .contains("candidate_invalid")
        );
    }

    #[test]
    fn w20_k01_inference_conflict_is_unresolved_without_class_inflation() {
        let support = vec![SemanticSupportRef::Operational("memory:a".to_string())];
        let mut values = vec![
            SemanticMemoryAssertion::build(
                "case:test",
                SemanticSubject::NamedEntity("project".to_string()),
                "project.value".to_string(),
                SemanticValue::Integer(1),
                EpistemicClass::EvidenceBoundInference,
                support.clone(),
                1,
                1,
                "test".to_string(),
                vec!["participant:test".to_string()],
            )
            .unwrap(),
            SemanticMemoryAssertion::build(
                "case:test",
                SemanticSubject::NamedEntity("project".to_string()),
                "project.value".to_string(),
                SemanticValue::Integer(2),
                EpistemicClass::EvidenceBoundInference,
                support,
                2,
                2,
                "test".to_string(),
                vec!["participant:test".to_string()],
            )
            .unwrap(),
        ];
        let contradictions = derive_contradictions(&mut values).unwrap();
        assert_eq!(contradictions[0].resolution_posture, "unresolved");
        assert!(values
            .iter()
            .all(|value| value.lifecycle == SemanticLifecycle::Contradicted));
        assert!(values
            .iter()
            .all(|value| value.epistemic_class == EpistemicClass::EvidenceBoundInference));
    }

    #[test]
    fn w20_cn03_cn09_support_is_exact_and_duplicate_candidates_deduplicate() {
        let episode = open_provider_episode("case:test", "participant:test");
        let input = derive_consolidation_input(
            "case:test",
            1,
            "participant:test",
            std::slice::from_ref(&episode),
            &[],
            &[],
        )
        .unwrap();
        let support = SemanticSupportRef::Episode(episode.episode_id.clone());
        let candidate = serde_json::json!({
            "schema": CONSOLIDATION_CANDIDATE_SCHEMA,
            "case_id": "case:test",
            "consolidation_input_id": input.input_id,
            "episode_narratives": [],
            "assertions": [
                {
                    "subject": {"kind": "named_entity", "id": "project:test"},
                    "predicate": "project.fact",
                    "value": {"type": "integer", "value": 4188},
                    "support_refs": [support]
                },
                {
                    "subject": {"kind": "named_entity", "id": "project:test"},
                    "predicate": "project.fact",
                    "value": {"type": "integer", "value": 4188},
                    "support_refs": [{"family": "episode", "id": episode.episode_id}]
                }
            ]
        });
        let normalized = normalize_consolidation_candidate(
            &input,
            "provider-result:test",
            &candidate.to_string(),
            &[episode],
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(normalized.len(), 1);
        assert_eq!(
            normalized[0].epistemic_class,
            EpistemicClass::EvidenceBoundInference
        );

        let forged = candidate
            .to_string()
            .replace("memory-episode:", "memory-episode:invented-");
        assert!(normalize_consolidation_candidate(
            &input,
            "provider-result:test",
            &forged,
            &[],
            &[],
            &[],
        )
        .unwrap_err()
        .contains("support_not_admitted"));
    }

    #[test]
    fn w20_s06_s07_cross_case_and_hidden_support_fail_closed() {
        let foreign = open_provider_episode("case:foreign", "participant:test");
        assert!(derive_consolidation_input(
            "case:test",
            1,
            "participant:test",
            &[foreign],
            &[],
            &[],
        )
        .unwrap_err()
        .contains("cross_case"));

        let mut hidden = open_provider_episode("case:test", "participant:hidden");
        hidden.participant_refs = vec!["participant:hidden".to_string()];
        let input = derive_consolidation_input(
            "case:test",
            1,
            "participant:test",
            std::slice::from_ref(&hidden),
            &[],
            &[],
        )
        .unwrap();
        assert!(input.allowed_support_refs.is_empty());
    }

    #[test]
    fn w20_s08_support_cycle_rejected_but_shared_ancestor_is_valid() {
        let leaf = assertion("project.leaf", 1, EpistemicClass::EvidenceBoundInference, 1);
        let mut left = assertion("project.left", 2, EpistemicClass::EvidenceBoundInference, 2);
        left.support_refs = vec![SemanticSupportRef::Assertion(leaf.assertion_id.clone())];
        let mut right = assertion(
            "project.right",
            3,
            EpistemicClass::EvidenceBoundInference,
            3,
        );
        right.support_refs = vec![SemanticSupportRef::Assertion(leaf.assertion_id.clone())];
        let mut root = assertion("project.root", 4, EpistemicClass::EvidenceBoundInference, 4);
        root.support_refs = vec![
            SemanticSupportRef::Assertion(left.assertion_id.clone()),
            SemanticSupportRef::Assertion(right.assertion_id.clone()),
        ];
        validate_support_graph(&[leaf.clone(), left.clone(), right], &[root]).unwrap();

        let mut cyclic_leaf = leaf;
        cyclic_leaf.support_refs = vec![SemanticSupportRef::Assertion(left.assertion_id.clone())];
        assert!(
            validate_support_graph(&[cyclic_leaf, left.clone()], &[left])
                .unwrap_err()
                .contains("cycle")
        );
    }

    #[test]
    fn w20_k02_k03_grounded_assertion_is_not_displaced_by_inference_or_claim() {
        let grounded = assertion(
            "project.open_fact",
            4188,
            EpistemicClass::MechanicallyGrounded,
            1,
        );
        let inference = assertion(
            "project.open_fact",
            9999,
            EpistemicClass::EvidenceBoundInference,
            2,
        );
        let claim = assertion(
            "project.open_fact",
            7777,
            EpistemicClass::ProviderOriginatedClaim,
            3,
        );
        let mut values = vec![grounded, inference, claim];
        let sets = derive_contradictions(&mut values).unwrap();
        assert_eq!(sets[0].resolution_posture, "grounded_assertion_preserved");
        assert_eq!(values[0].lifecycle, SemanticLifecycle::Active);
        assert_eq!(values[1].lifecycle, SemanticLifecycle::Contradicted);
        assert_eq!(values[2].lifecycle, SemanticLifecycle::Contradicted);
    }

    #[test]
    fn w20_k04_k05_only_unique_newer_grounded_state_supersedes() {
        let old = assertion(
            "resource.content_digest",
            1,
            EpistemicClass::MechanicallyGrounded,
            1,
        );
        let new = assertion(
            "resource.content_digest",
            2,
            EpistemicClass::MechanicallyGrounded,
            2,
        );
        let mut values = vec![old, new];
        assert_eq!(
            derive_contradictions(&mut values).unwrap()[0].resolution_posture,
            "mechanically_superseded"
        );
        assert_eq!(values[0].lifecycle, SemanticLifecycle::Superseded);
        assert_eq!(values[1].lifecycle, SemanticLifecycle::Active);

        let mut open = vec![
            assertion(
                "project.preference",
                1,
                EpistemicClass::MechanicallyGrounded,
                1,
            ),
            assertion(
                "project.preference",
                2,
                EpistemicClass::MechanicallyGrounded,
                2,
            ),
        ];
        assert_eq!(
            derive_contradictions(&mut open).unwrap()[0].resolution_posture,
            "grounded_assertion_preserved"
        );
        assert!(open
            .iter()
            .all(|assertion| assertion.lifecycle == SemanticLifecycle::Active));
    }

    #[test]
    fn w20_s05_identity_deduplicates_repeated_provider_origin() {
        let first = SemanticMemoryAssertion::build(
            "case:test",
            SemanticSubject::NamedEntity("project:test".to_string()),
            "project.fact".to_string(),
            SemanticValue::Integer(4188),
            EpistemicClass::EvidenceBoundInference,
            vec![SemanticSupportRef::Transition("transition:1".to_string())],
            1,
            1,
            "provider_result:first".to_string(),
            vec!["participant:test".to_string()],
        )
        .unwrap();
        let second = SemanticMemoryAssertion::build(
            "case:test",
            SemanticSubject::NamedEntity("project:test".to_string()),
            "project.fact".to_string(),
            SemanticValue::Integer(4188),
            EpistemicClass::EvidenceBoundInference,
            vec![SemanticSupportRef::Transition("transition:1".to_string())],
            1,
            1,
            "provider_result:second".to_string(),
            vec!["participant:test".to_string()],
        )
        .unwrap();
        assert_eq!(first.assertion_id, second.assertion_id);
    }

    #[test]
    fn w20_support_depth_and_contradiction_storm_are_bounded() {
        let mut chain = vec![assertion(
            "scale.leaf",
            0,
            EpistemicClass::EvidenceBoundInference,
            1,
        )];
        for depth in 1..=(MAX_SUPPORT_DEPTH + 1) {
            let parent = chain.last().unwrap().assertion_id.clone();
            let mut node = assertion(
                &format!("scale.depth_{depth}"),
                depth as i64,
                EpistemicClass::EvidenceBoundInference,
                (depth + 1) as u64,
            );
            node.support_refs = vec![SemanticSupportRef::Assertion(parent)];
            chain.push(node);
        }
        assert_eq!(
            validate_support_graph(&[], &chain).unwrap_err(),
            "semantic_support_depth_exceeded"
        );

        let mut storm = (0..=MAX_CONTRADICTION_MEMBERS)
            .map(|index| {
                assertion(
                    "scale.storm",
                    index as i64,
                    EpistemicClass::EvidenceBoundInference,
                    index as u64 + 1,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            derive_contradictions(&mut storm).unwrap_err(),
            "semantic_contradiction_member_bound_exceeded"
        );
    }

    #[test]
    fn w20_r04_active_support_keeps_old_episode_without_reopening_terminal_history() {
        let mut episodes = (0..131)
            .map(|index| {
                let mut episode = open_provider_episode("case:test", "participant:test");
                episode.episode_id = format!("memory-episode:{index:03}");
                episode.start_generation = index as u64 + 1;
                episode.end_generation = index as u64 + 1;
                episode.completion_posture = EpisodeCompletionPosture::Completed;
                episode
            })
            .collect::<Vec<_>>();
        episodes[1].completion_posture = EpisodeCompletionPosture::Denied;
        episodes[2].completion_posture = EpisodeCompletionPosture::BlockedReview;
        let mut supported = assertion(
            "project.supported_fact",
            4188,
            EpistemicClass::EvidenceBoundInference,
            1,
        );
        supported.support_refs = vec![SemanticSupportRef::Episode(episodes[0].episode_id.clone())];
        let retained_assertions = BTreeSet::from([supported.assertion_id.clone()]);
        let retained = retained_episode_ids(&episodes, &[supported], &retained_assertions);

        assert!(retained.contains(&episodes[0].episode_id));
        assert!(!retained.contains(&episodes[1].episode_id));
        assert!(retained.contains(&episodes[2].episode_id));
        assert!(!episode_is_unresolved(&episodes[1]));
        assert!(episode_is_unresolved(&episodes[2]));
    }

    #[test]
    #[ignore = "explicit W20 scale characterization"]
    fn w20_episode_and_semantic_scale_characterization() {
        use std::time::Instant;

        println!("family\titems\tmaterialize_ms\tgroup_ms\toutput_items");
        for count in [100usize, 1_000, 10_000] {
            let started = Instant::now();
            let history = (0..count)
                .map(|index| {
                    committed(
                        "case:scale",
                        index as u64 + 1,
                        TransitionPayload::ProviderInvocationStarted {
                            invocation_id: format!("provider-invocation:{index}"),
                            participant_id: "participant:test".to_string(),
                            provider_id: "provider-target:test".to_string(),
                            provider_kind: "openai_compatible".to_string(),
                            model_id: "model:test".to_string(),
                            semantic_lineage: None,
                            governance: None,
                        },
                    )
                })
                .collect::<Vec<_>>();
            let materialize_ms = started.elapsed().as_millis();
            let grouped = Instant::now();
            let episodes = derive_episodes("case:scale", &history).unwrap();
            println!(
                "episode\t{count}\t{materialize_ms}\t{}\t{}",
                grouped.elapsed().as_millis(),
                episodes.len()
            );
            assert_eq!(episodes.len(), count);
        }

        for count in [1_000usize, 10_000, 50_000] {
            let started = Instant::now();
            let mut assertions = (0..count)
                .map(|index| {
                    SemanticMemoryAssertion::build(
                        "case:scale",
                        SemanticSubject::NamedEntity(format!("entity:{index}")),
                        format!("scale.fact_{index}"),
                        SemanticValue::Integer(index as i64),
                        EpistemicClass::EvidenceBoundInference,
                        vec![SemanticSupportRef::Transition(format!(
                            "transition:{index}"
                        ))],
                        index as u64 + 1,
                        index as u64 + 1,
                        "scale".to_string(),
                        vec!["participant:test".to_string()],
                    )
                    .unwrap()
                })
                .collect::<Vec<_>>();
            let materialize_ms = started.elapsed().as_millis();
            let grouped = Instant::now();
            let contradictions = derive_contradictions(&mut assertions).unwrap();
            println!(
                "semantic\t{count}\t{materialize_ms}\t{}\t{}",
                grouped.elapsed().as_millis(),
                contradictions.len()
            );
            assert!(contradictions.is_empty());
        }
    }
}
