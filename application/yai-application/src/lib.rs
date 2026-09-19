//! Typed first-party application projections shared by non-owning YAI clients.
//!
//! The engine remains the owner of Case truth, policy, workflow, memory and
//! provider governance. This crate authenticates a local caller and composes
//! bounded, redacted product views from those owners. It owns no persistence.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use yai_core_engine::conversation::turns_from_history;
use yai_core_engine::memory_hierarchy::knowledge::KnowledgeRequest;
use yai_core_engine::security::AuthenticatedPrincipal;
use yai_core_engine::store::lmdb::LmdbRecordStore;
use yai_core_engine::transition::{CaseLifecycle, CaseState, ReviewResolution, Transition};

pub const INTERFACES_REVISION: &str = "bae6cdf7cf17f3e6a58c0323852c7c0efeb26147";
pub const APPLICATION_PROTOCOL: &str = "yai.studio.application.v1";

#[derive(Clone, Debug, Deserialize)]
pub struct OperationRequest {
    pub protocol: String,
    pub operation_ref: String,
    pub correlation_ref: String,
    #[serde(default)]
    pub input: Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct OperationError {
    pub code: String,
    pub message: String,
    pub safe_message: String,
    pub result_state: ResultState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultState {
    Success,
    Partial,
    Unauthorized,
    Stale,
    CorePending,
    NotImplemented,
    TransportUnavailable,
    Error,
}

#[derive(Clone, Debug, Serialize)]
pub struct OperationResult {
    pub operation_ref: String,
    pub result_state: ResultState,
    pub correlation_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<OperationError>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CaseUpdate {
    pub protocol: &'static str,
    pub event_ref: String,
    pub event_type: &'static str,
    pub event_family: &'static str,
    pub case_ref: String,
    pub generation: u64,
    pub sequence: u64,
    pub cursor: String,
    pub affected_views: Vec<&'static str>,
}

#[derive(Clone, Debug)]
pub struct LocalApplication {
    store_path: PathBuf,
}

impl Default for LocalApplication {
    fn default() -> Self {
        Self::from_yai_home(default_yai_home())
    }
}

impl LocalApplication {
    pub fn from_yai_home(home: impl AsRef<Path>) -> Self {
        Self {
            store_path: home.as_ref().join("store").join("lmdb"),
        }
    }

    pub fn call(&self, request: OperationRequest) -> OperationResult {
        if request.protocol != APPLICATION_PROTOCOL {
            return failure(
                &request,
                ResultState::Error,
                "version_mismatch",
                "Studio application protocol version is not supported.",
            );
        }
        match self.call_inner(&request) {
            Ok(data) => success(&request, data),
            Err(error) => map_error(&request, &error),
        }
    }

    pub fn visible_generations(&self) -> Result<BTreeMap<String, u64>, String> {
        let (store, auth) = self.open()?;
        Ok(store
            .list_case_states_authorized(&auth, None, 1024)?
            .into_iter()
            .map(|state| (state.case_id, state.generation))
            .collect())
    }

    pub fn update_for(&self, case_ref: &str, generation: u64, sequence: u64) -> CaseUpdate {
        CaseUpdate {
            protocol: APPLICATION_PROTOCOL,
            event_ref: format!("case-update:{case_ref}:{generation}"),
            event_type: "case_projection_invalidated",
            event_family: "case",
            case_ref: case_ref.to_string(),
            generation,
            sequence,
            cursor: format!("{case_ref}:{generation}:{sequence}"),
            affected_views: vec![
                "overview",
                "environment",
                "knowledge",
                "memory",
                "authority",
                "work",
                "compute",
                "conversation",
            ],
        }
    }

    fn open(&self) -> Result<(LmdbRecordStore, AuthenticatedPrincipal), String> {
        let auth = AuthenticatedPrincipal::authenticate_local()?;
        let store = LmdbRecordStore::open(&self.store_path)?;
        Ok((store, auth))
    }

    fn call_inner(&self, request: &OperationRequest) -> Result<Value, String> {
        if !matches!(
            request.operation_ref.as_str(),
            "system.status"
                | "runtime.readiness"
                | "case.list"
                | "case.recent"
                | "case.open"
                | "case.summary"
                | "events.subscribe"
                | "events.resume"
                | "events.heartbeat"
        ) {
            return Err("operation_not_implemented".to_string());
        }
        let (store, auth) = self.open()?;
        match request.operation_ref.as_str() {
            "system.status" | "runtime.readiness" => Ok(json!({
                "protocol": APPLICATION_PROTOCOL,
                "interfaces_revision": INTERFACES_REVISION,
                "transport": "tauri-local-bridge",
                "local": true,
                "authenticated_principal": auth.projected_principal_id(),
                "runtime_state": "available"
            })),
            "case.list" | "case.recent" => case_list(&store, &auth),
            "case.open" => {
                let case_ref = input_case_ref(request)?;
                case_open(&store, &auth, case_ref)
            }
            "case.summary" => {
                let case_ref = input_case_ref(request)?;
                case_snapshot(
                    &store,
                    &auth,
                    case_ref,
                    request
                        .input
                        .get("expected_generation")
                        .and_then(Value::as_u64),
                )
            }
            "events.subscribe" | "events.resume" => Ok(json!({
                "transport": "tauri-event-bridge",
                "stream_state": "subscribed",
                "resync": "case.summary",
                "resume_posture": "snapshot_required_after_attach_or_gap",
                "heartbeat_interval_ms": 1000,
                "resume_token": request.input.get("resume_token").and_then(Value::as_str)
            })),
            "events.heartbeat" => {
                let case_ref = input_case_ref(request)?;
                let state = store.get_case_state_authorized(&auth, case_ref)?;
                Ok(json!({
                    "transport": "tauri-event-bridge",
                    "stream_state": "live",
                    "case_ref": state.case_id,
                    "generation": state.generation,
                    "cursor": format!("{}:{}:heartbeat", state.case_id, state.generation),
                    "resync": "case.summary"
                }))
            }
            _ => unreachable!("operation references were validated before opening the store"),
        }
    }
}

fn default_yai_home() -> PathBuf {
    std::env::var_os("YAI_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".yai")
        })
}

fn input_case_ref(request: &OperationRequest) -> Result<&str, String> {
    request
        .input
        .get("case_ref")
        .and_then(Value::as_str)
        .filter(|value| value.starts_with("case:"))
        .ok_or_else(|| "case_ref_invalid".to_string())
}

fn case_list(store: &LmdbRecordStore, auth: &AuthenticatedPrincipal) -> Result<Value, String> {
    let mut cases = Vec::new();
    for state in store.list_case_states_authorized(auth, None, 1024)? {
        let history = store.list_case_transitions(&state.case_id)?;
        let updated_at_unix_ms = history
            .last()
            .map(|transition| transition.committed_at_unix_ms);
        cases.push(json!({
            "case_ref": state.case_id,
            "display_name": state.case_id,
            "case_status": lifecycle(&state.lifecycle),
            "generation": state.generation,
            "updated_at_unix_ms": updated_at_unix_ms,
            "participant_count": state.participants.len(),
            "source_count": state.sources.len(),
            "resource_count": state.resources.len(),
            "pending_review_count": pending_reviews(&state),
            "summary_available": true
        }));
    }
    Ok(json!({ "cases": cases, "authority": "yai.application" }))
}

fn case_open(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case_ref: &str,
) -> Result<Value, String> {
    let state = store.get_case_state_authorized(auth, case_ref)?;
    let principal = auth.projected_principal_id();
    let participant = state
        .principal_participant_links
        .iter()
        .find(|link| link.principal_id == principal)
        .map(|link| link.participant_id.clone())
        .ok_or_else(|| "authenticated_principal_participant_link_required".to_string())?;
    let history = store.list_case_transitions(case_ref)?;
    let thread_ref = turns_from_history(case_ref, &history)
        .into_iter()
        .rev()
        .find(|turn| turn.participant_id == participant)
        .map(|turn| turn.thread_id.clone());
    Ok(json!({
        "case_ref": case_ref,
        "case_status": lifecycle(&state.lifecycle),
        "generation": state.generation,
        "authenticated_principal": principal,
        "participant_ref": participant,
        "thread_ref": thread_ref,
        "attachment": "ephemeral",
        "changed_active_case": false,
        "receipt_ref": Value::Null,
        "event_refs": []
    }))
}

fn case_snapshot(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case_ref: &str,
    expected_generation: Option<u64>,
) -> Result<Value, String> {
    let state = store.get_case_state_authorized(auth, case_ref)?;
    if expected_generation.is_some_and(|expected| expected != state.generation) {
        return Err(format!(
            "stale_generation:expected={}:actual={}",
            expected_generation.unwrap_or_default(),
            state.generation
        ));
    }
    let history = store.list_case_transitions(case_ref)?;
    let graph = store.list_graph_relations_by_case(case_ref, 512)?;
    let participant_ref = case_open(store, auth, case_ref)?["participant_ref"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let participants = state
        .participants
        .iter()
        .map(|participant| {
            json!({
                "id": participant.participant_id,
                "roles": participant.roles,
                "is_current": participant.participant_id == participant_ref
            })
        })
        .collect::<Vec<_>>();
    let sources = state.sources.iter().map(|source| json!({
        "id": source.declaration.source_id,
        "label": source.declaration.logical_name,
        "perimeter": source.declaration.perimeter,
        "media_type": source.declaration.media_type,
        "roles": source.declaration.roles,
        "resource_ref": source.declaration.resource_attachment_id,
        "posture": source.progress.as_ref().map(|progress| format!("{:?}", progress.phase).to_lowercase()),
        "revision_ref": source.progress.as_ref().and_then(|progress| progress.revision.as_ref()).map(|revision| revision.revision_id.clone()),
        "items": source.progress.as_ref().and_then(|progress| progress.revision.as_ref()).map(|revision| revision.items.len())
    })).collect::<Vec<_>>();
    let files = state
        .sources
        .iter()
        .flat_map(|source| {
            source.progress.iter().flat_map(move |progress| {
                progress.revision.iter().flat_map(move |revision| {
                    revision.items.iter().map(move |item| json!({
                "id": format!("source-file:{}:{}", source.declaration.source_id, item.digest),
                "source_ref": source.declaration.source_id,
                "source_label": source.declaration.logical_name,
                "revision_ref": revision.revision_id,
                "path": item.path,
                "digest": item.digest,
                "bytes": item.bytes,
                "backing": item.backing
            }))
                })
            })
        })
        .collect::<Vec<_>>();
    let resources = state
        .resources
        .iter()
        .map(|resource| {
            json!({
                "id": resource.attachment_id,
                "kind": format!("{:?}", resource.kind).to_lowercase(),
                "policy_ref": resource.policy_id,
                "review_requirement": format!("{:?}", resource.review_requirement).to_lowercase()
            })
        })
        .collect::<Vec<_>>();
    let timeline = history
        .iter()
        .rev()
        .take(160)
        .rev()
        .map(timeline_entry)
        .collect::<Vec<_>>();
    let relations = graph
        .relations
        .iter()
        .map(|relation| {
            json!({
                "id": relation.relation_id,
                "from": relation.from_ref,
                "to": relation.to_ref,
                "kind": relation.edge_kind,
                "from_kind": relation.from_kind,
                "to_kind": relation.to_kind,
                "source_ref": relation.source_record_id,
                "confidence": relation.confidence
            })
        })
        .collect::<Vec<_>>();
    let authority = authority_projection(&state);
    let workflow = workflow_projection(store, auth, &state)?;
    let compute = compute_projection(store, auth, &state)?;
    let knowledge = knowledge_projection(store, auth, case_ref);
    let conversation = turns_from_history(case_ref, &history)
        .into_iter()
        .map(|turn| {
            json!({
                "id": turn.turn_id,
                "thread_ref": turn.thread_id,
                "participant_ref": turn.participant_id,
                "generation": turn.base_generation + 1,
                "parts": turn.ordered_parts.iter().map(|part| json!({
                    "modality": part.object.modality,
                    "media_type": part.object.media_type,
                    "text": part.object.inline_text
                })).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    let attention = attention_projection(&state, &workflow, &compute);
    Ok(json!({
        "case": {
            "case_ref": state.case_id,
            "display_name": state.case_id,
            "case_status": lifecycle(&state.lifecycle),
            "generation": state.generation,
            "tenant_ref": state.tenant_id,
            "participant_ref": participant_ref,
            "updated_at_unix_ms": history.last().map(|transition| transition.committed_at_unix_ms)
        },
        "overview": { "attention": attention, "participants": participants },
        "environment": { "sources": sources, "files": files, "resources": resources, "artifacts": state.admitted_content },
        "knowledge": knowledge,
        "memory": { "authority": "transition_ledger_and_derived_graph", "timeline": timeline, "relations": relations, "generation": state.generation },
        "authority": authority,
        "work": workflow,
        "compute": compute,
        "conversation": { "read_only": true, "turns": conversation },
        "freshness": { "generation": state.generation, "resync_operation": "case.summary" }
    }))
}

fn knowledge_projection(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case_ref: &str,
) -> Value {
    let mut request = KnowledgeRequest::new(case_ref);
    request.max_units = 512;
    match store.case_knowledge_authorized(auth, request, None) {
        Ok(result) => {
            let view = result.view;
            let status = if view.sources.is_empty() {
                "empty"
            } else {
                "available"
            };
            json!({
                "status": status,
                "message": if status == "empty" { "No qualified knowledge has been derived for this Case." } else { "Qualified source-grounded derivation from YAI." },
                "profile": view.profile,
                "source_closure": view.source_closure,
                "sources": view.sources.into_iter().map(|source| json!({
                    "id": source.id,
                    "source_ref": source.source_id,
                    "label": source.logical_name,
                    "revision_ref": source.revision_id,
                    "path": source.path,
                    "digest": source.digest,
                    "media_type": source.media_type,
                    "extractor": source.extractor,
                    "status": format!("{:?}", source.status).to_lowercase(),
                    "detail": source.detail
                })).collect::<Vec<_>>(),
                "units": view.units.into_iter().map(|unit| json!({
                    "id": unit.id,
                    "source_ref": unit.source,
                    "parent_ref": unit.parent,
                    "kind": unit.kind,
                    "text": unit.text,
                    "posture": format!("{:?}", unit.posture).to_lowercase(),
                    "entity_ref": unit.entity,
                    "predicate": unit.predicate,
                    "value": unit.value,
                    "references": unit.references,
                    "topics": unit.topics
                })).collect::<Vec<_>>(),
                "entities": view.entities,
                "topics": view.topics,
                "contradictions": view.contradictions,
                "relations": view.relations.into_iter().map(|relation| json!({
                    "id": relation.id,
                    "from": relation.from,
                    "to": relation.to,
                    "kind": format!("{:?}", relation.kind).to_lowercase(),
                    "posture": relation.posture,
                    "backing_units": relation.backing_units
                })).collect::<Vec<_>>(),
                "counts": {
                    "sources": result.measurements.source_documents,
                    "units": result.measurements.units,
                    "entities": result.measurements.entities,
                    "claims": result.measurements.claims,
                    "relations": result.measurements.relations,
                    "contradictions": result.measurements.contradictions
                }
            })
        }
        Err(error) => {
            let status = if error.contains("backing") {
                "backing_unavailable"
            } else {
                "unavailable"
            };
            json!({
                "status": status,
                "message": if status == "backing_unavailable" { "Qualified source backing is unavailable." } else { "Knowledge is unavailable to the current principal." },
                "sources": [], "units": [], "entities": [], "topics": [], "contradictions": [], "relations": []
            })
        }
    }
}

fn timeline_entry(transition: &Transition) -> Value {
    json!({
        "id": transition.transition_id,
        "sequence": transition.sequence,
        "committed_at_unix_ms": transition.committed_at_unix_ms,
        "kind": transition.payload.kind(),
        "participant_ref": transition.source.participant_id,
        "component": transition.source.component,
        "causal_refs": transition.causal_refs,
        "summary": transition.summary
    })
}

fn authority_projection(state: &CaseState) -> Value {
    let policies = state
        .policy_bindings
        .iter()
        .map(|binding| serde_json::to_value(binding).unwrap_or(Value::Null))
        .collect::<Vec<_>>();
    let reviews = state
        .reviews
        .iter()
        .map(|review| {
            json!({
                "id": review.review_id,
                "status": review.status,
                "operation_ref": review.operation_id,
                "policy_ref": review.effective_policy_id,
                "decision_ref": review.effective_decision_id,
                "evidence_ref": review.receipt_ref,
                "required_roles": review.required_reviewer_roles
            })
        })
        .collect::<Vec<_>>();
    let grants = state
        .grants
        .iter()
        .map(|grant| serde_json::to_value(grant).unwrap_or(Value::Null))
        .collect::<Vec<_>>();
    json!({
        "policies": policies,
        "reviews": reviews,
        "grants": grants,
        "last_decision": state.last_decision,
        "empty": state.policy_bindings.is_empty() && state.reviews.is_empty() && state.grants.is_empty()
    })
}

fn workflow_projection(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    state: &CaseState,
) -> Result<Value, String> {
    let Some(binding) = &state.workflow_binding else {
        return Ok(
            json!({ "status": "empty", "message": "No workflow is configured for this Case.", "nodes": [], "edges": [] }),
        );
    };
    let resolution = store.workflow_status_authorized(auth, &state.case_id)?;
    let definition =
        store.get_workflow_definition_authorized(auth, &binding.workflow_definition_id)?;
    let edges = definition
        .edges
        .iter()
        .enumerate()
        .map(|(index, edge)| {
            json!({
                "id": format!("workflow-edge:{index}:{}:{}", edge.from, edge.to),
                "from": edge.from,
                "to": edge.to,
                "kind": format!("{:?}", edge.kind).to_lowercase()
            })
        })
        .collect::<Vec<_>>();
    Ok(
        json!({ "status": "available", "definition": definition, "resolution": resolution, "nodes": resolution.nodes, "edges": edges }),
    )
}

fn compute_projection(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    state: &CaseState,
) -> Result<Value, String> {
    let Some(tenant_ref) = state.tenant_id.as_deref() else {
        return Ok(
            json!({ "status": "unavailable", "message": "Provider posture requires a Tenant-scoped Case.", "targets": [] }),
        );
    };
    let targets = store.list_provider_targets_authorized(auth, tenant_ref)?;
    let bound = state
        .provider_binding
        .as_ref()
        .map(|binding| {
            binding
                .ordered_target_ids
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let projected = targets
        .into_iter()
        .filter(|target| bound.contains(&target.target_id))
        .map(|target| {
            let posture = store
                .provider_posture_authorized(auth, &target.target_id)
                .ok()
                .map(|(_, qualification, trust, health)| {
                    json!({
                        "qualification": qualification.map(|value| json!({
                            "id": value.qualification_id,
                            "suite_id": value.suite_id,
                            "run_id": value.run_id,
                            "qualified_at_unix_ms": value.qualified_at_unix_ms,
                            "valid_until_unix_ms": value.valid_until_unix_ms,
                            "capabilities": value.capabilities.into_iter().map(|capability| json!({
                                "capability": format!("{:?}", capability.capability).to_lowercase(),
                                "provenance": format!("{:?}", capability.provenance).to_lowercase(),
                                "evidence_refs": capability.evidence_refs,
                                "verified_minimum": capability.verified_minimum
                            })).collect::<Vec<_>>()
                        })),
                        "trust": trust.map(|value| json!({
                            "event_ref": value.event_id,
                            "posture": format!("{:?}", value.posture).to_lowercase(),
                            "recorded_at_unix_ms": value.recorded_at_unix_ms
                        })),
                        "health": {
                            "posture": format!("{:?}", health.posture).to_lowercase(),
                            "circuit": format!("{:?}", health.circuit).to_lowercase(),
                            "consecutive_failures": health.consecutive_failures,
                            "observed_at_unix_ms": health.observed_at_unix_ms,
                            "failure_class": health.failure_class
                        }
                    })
                });
            json!({
                "id": target.target_id,
                "provider_key": target.provider_key,
                "adapter": target.adapter,
                "model_id": target.model_id,
                "locality": target.locality,
                "endpoint": sanitized_endpoint(&target.endpoint),
                "posture": posture,
                "management": "not_exposed"
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "status": if projected.is_empty() { "empty" } else { "available" },
        "message": if projected.is_empty() { "No provider is connected to this Case." } else { "Provider facts are reported by YAI." },
        "targets": projected
    }))
}

fn attention_projection(state: &CaseState, workflow: &Value, compute: &Value) -> Vec<Value> {
    let mut attention = Vec::new();
    for review in state.reviews.iter().filter(|review| {
        matches!(
            review.status,
            ReviewResolution::Pending | ReviewResolution::Deferred
        )
    }) {
        attention.push(json!({ "kind": "review", "title": "Review required", "detail": review.policy_reason, "ref": review.review_id }));
    }
    if workflow["status"] == "empty" {
        attention.push(json!({ "kind": "workflow", "title": "No workflow configured", "detail": "Work remains available through other Case operations." }));
    }
    if compute["status"] == "empty" {
        attention.push(json!({ "kind": "provider", "title": "No provider connected", "detail": "Inference is unavailable for this Case." }));
    }
    attention
}

fn pending_reviews(state: &CaseState) -> usize {
    state
        .reviews
        .iter()
        .filter(|review| {
            matches!(
                review.status,
                ReviewResolution::Pending | ReviewResolution::Deferred
            )
        })
        .count()
}

fn lifecycle(value: &CaseLifecycle) -> &'static str {
    match value {
        CaseLifecycle::Open => "open",
        CaseLifecycle::Closed => "closed",
    }
}

fn sanitized_endpoint(endpoint: &str) -> String {
    endpoint
        .split_once("://")
        .map(|(scheme, rest)| {
            let authority = rest.split('/').next().unwrap_or(rest);
            let host = authority.rsplit('@').next().unwrap_or(authority);
            format!("{scheme}://{host}")
        })
        .unwrap_or_else(|| "configured".to_string())
}

fn success(request: &OperationRequest, data: Value) -> OperationResult {
    OperationResult {
        operation_ref: request.operation_ref.clone(),
        result_state: ResultState::Success,
        correlation_ref: request.correlation_ref.clone(),
        data: Some(data),
        error: None,
    }
}

fn map_error(request: &OperationRequest, error: &str) -> OperationResult {
    let (state, safe) = if error == "operation_not_implemented" {
        (
            ResultState::NotImplemented,
            "This operation is not implemented by the local YAI host.",
        )
    } else if error.contains("not_visible")
        || error.contains("authentication")
        || error.contains("principal")
        || error.contains("owner")
    {
        (
            ResultState::Unauthorized,
            "This Case is not available to the authenticated local principal.",
        )
    } else if error.contains("stale") {
        (
            ResultState::Stale,
            "The Case changed. Studio must resynchronize before continuing.",
        )
    } else if error.contains("not found")
        || error.contains("No such file")
        || error.contains("lmdb")
    {
        (
            ResultState::TransportUnavailable,
            "The local YAI application store is unavailable.",
        )
    } else {
        (
            ResultState::Error,
            "YAI could not produce this application projection.",
        )
    };
    failure(request, state, error, safe)
}

fn failure(
    request: &OperationRequest,
    state: ResultState,
    code: &str,
    safe: &str,
) -> OperationResult {
    OperationResult {
        operation_ref: request.operation_ref.clone(),
        result_state: state,
        correlation_ref: request.correlation_ref.clone(),
        data: None,
        error: Some(OperationError {
            code: code.to_string(),
            message: code.to_string(),
            safe_message: safe.to_string(),
            result_state: state,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_operation_is_honest() {
        let app = LocalApplication::from_yai_home("/path/does/not/matter");
        let result = app.call(OperationRequest {
            protocol: APPLICATION_PROTOCOL.into(),
            operation_ref: "provider.telemetry.fake".into(),
            correlation_ref: "test:1".into(),
            input: Value::Null,
        });
        assert_eq!(result.result_state, ResultState::NotImplemented);
        assert!(result.data.is_none());
    }

    #[test]
    fn version_mismatch_fails_before_store_access() {
        let app = LocalApplication::from_yai_home("/path/does/not/matter");
        let result = app.call(OperationRequest {
            protocol: "future".into(),
            operation_ref: "case.list".into(),
            correlation_ref: "test:2".into(),
            input: Value::Null,
        });
        assert_eq!(result.error.unwrap().code, "version_mismatch");
    }

    #[test]
    fn endpoint_projection_omits_path_and_userinfo() {
        assert_eq!(
            sanitized_endpoint("https://operator:secret@example.test/v1/chat?key=no"),
            "https://example.test"
        );
        assert_eq!(sanitized_endpoint("opaque-local-target"), "configured");
    }
}
