//! Frontend-independent Resource actions over the existing admission owner.
//! Protocol adapters cannot interpret a request as authority. This is shared
//! application orchestration, not a connector service or another runtime.

pub mod source;

use std::path::{Path, PathBuf};
use yai_core_engine::admission::build_policy_review_request;
use yai_core_engine::effect::{Decision, DecisionOutcome, EffectOutcome, ExecutionGrant, LocalFilesystemBinding, Operation, issue_policy_execution_grant};
use yai_core_engine::store::lmdb::{LmdbRecordStore, PreparedCommitOutcome};
use yai_core_engine::transition::{CaseState, PendingTransition, ResourceAttachmentState, ReviewRequirement, ReviewResolution, ReviewState, TransitionPayload, TransitionScope, TransitionSource, REVIEW_REQUEST_SCHEMA};
use crate::CaseStateMutationResult;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use yai_core_engine::effect::access::{
    inspect_confined_tree, query_sqlite_snapshot, read_confined_file, LocalAccessBinding,
    ResourceAccessContract, ResourceAction, ResourceObservation,
};
use yai_core_engine::security::AuthenticatedPrincipal;

/// Bootstrap/application input, not a new durable configuration authority.
/// Physical roots and configuration digests are observed by YAI at import.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceDefinition {
    schema: String,
    attachment_id: String,
    policy_owner: String,
    participant_ids: Vec<String>,
    operations: Vec<yai_core_engine::effect::access::AccessKind>,
    read_prefixes: Vec<String>,
    names: Vec<String>,
    max_output_bytes: usize,
    max_items: usize,
    address: ResourceAddressInput,
    #[serde(default)]
    write_prefix: Option<String>,
    #[serde(default)]
    max_write_bytes: Option<usize>,
    #[serde(default)]
    review_requirement: ReviewRequirement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ResourceAddressInput {
    Filesystem {
        root: PathBuf,
    },
    Discovery {
        root: PathBuf,
    },
    ProcessRunner {
        root: PathBuf,
        runners: std::collections::BTreeMap<String, yai_core_engine::effect::access::ProcessRunner>,
    },
    Sqlite {
        root: PathBuf,
        path: String,
        queries: std::collections::BTreeMap<String, String>,
    },
    HttpService {
        endpoint: yai_core_engine::effect::access::NetworkResourceAddress,
        paths: std::collections::BTreeMap<String, String>,
    },
    Mcp {
        endpoint: yai_core_engine::effect::access::NetworkResourceAddress,
    },
}

pub fn import_definition(
    store: &LmdbRecordStore,
    authenticated: &AuthenticatedPrincipal,
    case_id: &str,
    definition: ResourceDefinition,
) -> Result<CaseState, String> {
    import_definition_result(store, authenticated, case_id, definition).map(|result| result.state)
}

/// Typed clients share CLI import resolution; callers never fabricate native
/// root identities or configuration digests. Exact attachment retry is owned by attach.
pub fn import_definition_result(
    store: &LmdbRecordStore,
    authenticated: &AuthenticatedPrincipal,
    case_id: &str,
    definition: ResourceDefinition,
) -> Result<CaseStateMutationResult, String> {
    use yai_core_engine::effect::access::{
        ResourceAddress, LOCAL_ACCESS_BINDING_SCHEMA, RESOURCE_ACCESS_SCHEMA,
    };
    if definition.schema != "yai.resource_definition.v1"
        || definition.write_prefix.is_some() != definition.max_write_bytes.is_some()
    {
        return Err("resource_definition_contract_invalid".into());
    }
    // Resolve authority before inspecting any administrator-supplied root.
    let state = store.get_case_state_authorized(authenticated, case_id)?;
    store
        .resolve_security_context(authenticated, state.tenant_id.as_deref().ok_or("resource_attachment_requires_tenant")?)?
        .require_owner()?;
    let root = |path: &Path| LocalFilesystemBinding::new(case_id, &definition.attachment_id, path);
    let address = match definition.address {
        ResourceAddressInput::Filesystem { root: path } => {
            ResourceAddress::Filesystem { root: root(&path)? }
        }
        ResourceAddressInput::Discovery { root: path } => {
            ResourceAddress::Discovery { root: root(&path)? }
        }
        ResourceAddressInput::ProcessRunner {
            root: path,
            runners,
        } => ResourceAddress::ProcessRunner {
            root: root(&path)?,
            runners,
        },
        ResourceAddressInput::Sqlite {
            root: directory,
            path,
            queries,
        } => ResourceAddress::Sqlite {
            root: root(&directory)?,
            path,
            queries,
        },
        ResourceAddressInput::HttpService { endpoint, paths } => {
            ResourceAddress::HttpService { endpoint, paths }
        }
        ResourceAddressInput::Mcp { endpoint } => ResourceAddress::Mcp { endpoint },
    };
    let binding = LocalAccessBinding {
        schema: LOCAL_ACCESS_BINDING_SCHEMA.into(),
        case_id: case_id.into(),
        attachment_id: definition.attachment_id,
        address,
    };
    let access = ResourceAccessContract {
        schema: RESOURCE_ACCESS_SCHEMA.into(),
        configuration_digest: binding.digest(),
        participant_ids: definition.participant_ids,
        operations: definition.operations,
        read_prefixes: definition.read_prefixes,
        names: definition.names,
        max_output_bytes: definition.max_output_bytes,
        max_items: definition.max_items,
    };
    attach(
        store,
        authenticated,
        &binding,
        access,
        &definition.policy_owner,
        definition
            .write_prefix
            .as_deref()
            .zip(definition.max_write_bytes),
        definition.review_requirement,
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "posture", rename_all = "snake_case")]
pub enum ResourceActionOutcome {
    ContentAdmitted {
        admission: Box<yai_core_engine::effect::access::CaseContentAdmission>,
        reused: bool,
    },
    Observed {
        observation: Box<ResourceObservation>,
        reused: bool,
    },
    Effect {
        observation: Box<ResourceObservation>,
        receipt: Box<yai_core_engine::effect::access::ResourceEffectReceipt>,
        reused: bool,
    },
    Indeterminate {
        operation_id: String,
        effect_id: String,
        reason: String,
    },
    Denied {
        operation_id: String,
        decision_id: String,
        reason: String,
    },
    AwaitingReview {
        operation_id: String,
        decision_id: String,
        review_id: String,
    },
    Unsupported {
        operation_id: String,
        reason: String,
    },
}

/// Reconnect-safe operational metadata. Payload/content retrieval remains a
/// separate currently qualified Resource read; observation never dispatches.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceExecutionObservation {
    pub operation_ref: String,
    pub case_ref: String,
    pub participant_ref: String,
    pub generation: u64,
    pub posture: ResourceExecutionPosture,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ResourceExecutionPosture {
    Admitted,
    WaitingForReview { review_ref: String },
    Refused { decision_ref: String },
    Completed { result_ref: String },
    /// A terminal carrier receipt is not an assertion that the requested
    /// consequence succeeded. Preserve the exact domain outcome on reconnect.
    EffectRecorded {
        result_ref: String,
        receipt_ref: String,
        effect_ref: String,
        outcome: EffectOutcome,
        external_execution_started: bool,
    },
    PreparedOrIndeterminate { effect_ref: String },
}

pub fn observe(
    store: &LmdbRecordStore,
    authenticated: &AuthenticatedPrincipal,
    case_id: &str,
    participant_id: &str,
    operation_id: &str,
) -> Result<ResourceExecutionObservation, String> {
    let state = store.get_case_state_authorized(authenticated, case_id)?;
    if !state.principal_participant_links.iter().any(|link|
        link.principal_id == authenticated.projected_principal_id()
            && link.participant_id == participant_id) {
        return Err("resource_execution_not_visible".into());
    }
    let history = store.list_case_transitions(case_id)?;
    let operation = history.iter().find_map(|t| match &t.payload {
        TransitionPayload::OperationRecorded { operation }
            if operation.operation_id == operation_id && operation.participant_id == participant_id => Some(operation),
        _ => None,
    }).ok_or("resource_execution_not_visible")?;
    // Requestability is not disclosure: a policy-denied request must remain
    // observable by its originator while its Resource envelope is visible.
    let access = state.resources.iter()
        .find(|resource| resource.attachment_id == operation.resource_attachment_id)
        .and_then(|resource| resource.access.as_ref())
        .ok_or("resource_execution_not_visible")?;
    access.admits_request(participant_id, operation.resource_request.as_ref()
        .ok_or("resource_execution_not_visible")?)
        .map_err(|_| "resource_execution_not_visible")?;
    let mut posture = ResourceExecutionPosture::Admitted;
    for transition in &history {
        match &transition.payload {
            TransitionPayload::DecisionRecorded { decision }
                if decision.operation_id == operation_id && decision.outcome == DecisionOutcome::Deny => {
                posture = ResourceExecutionPosture::Refused { decision_ref: decision.decision_id.clone() };
            }
            TransitionPayload::CaseContentAdmitted { admission }
                if admission.operation_id == operation_id => {
                store.validate_resource_result_reuse_authorized(authenticated, case_id,
                    operation_id, &admission.decision_id)?;
                posture = ResourceExecutionPosture::Completed { result_ref: admission.admission_id.clone() };
            }
            TransitionPayload::ResourceObservationRecorded { observation }
                if observation.operation_id == operation_id => {
                store.validate_resource_result_reuse_authorized(authenticated, case_id,
                    operation_id, &observation.decision_id)?;
                posture = ResourceExecutionPosture::Completed { result_ref: observation.observation_id.clone() };
            }
            TransitionPayload::ResourceEffectFinalized { observation, receipt, .. }
                if observation.operation_id == operation_id => {
                store.validate_resource_result_reuse_authorized(authenticated, case_id,
                    operation_id, &observation.decision_id)?;
                posture = ResourceExecutionPosture::EffectRecorded {
                    result_ref: observation.observation_id.clone(),
                    receipt_ref: receipt.receipt_id.clone(),
                    effect_ref: receipt.effect_id.clone(),
                    outcome: receipt.outcome.clone(),
                    external_execution_started: receipt.external_execution_started,
                };
            }
            _ => {}
        }
    }
    if !matches!(posture, ResourceExecutionPosture::Completed { .. }
        | ResourceExecutionPosture::EffectRecorded { .. } | ResourceExecutionPosture::Refused { .. }) {
        if let Some(effect) = state.effects.iter().find(|e| e.operation_id == operation_id) {
            posture = ResourceExecutionPosture::PreparedOrIndeterminate { effect_ref: effect.effect_id.clone() };
        } else if let Some(review) = state.reviews.iter().find(|r| r.operation_id == operation_id) {
            if matches!(review.status, ReviewResolution::Pending | ReviewResolution::PendingOperator | ReviewResolution::Deferred) {
                // Historical pending review is not current permission to
                // continue. Policy lifecycle can change without a Case
                // generation change, so the generation fence alone is not
                // sufficient. Observe through the existing policy owner;
                // never mutate/invalidate a review merely to render it.
                let policy = store.case_policy_status(case_id)?;
                if policy.readiness != yai_core_engine::case_policy::NormativeReadiness::Ready
                    || policy.validity != yai_core_engine::case_policy::PolicyValidityPosture::Valid
                    || !policy.effective_policy.as_ref().is_some_and(|effective|
                        effective.effective_policy_id == review.effective_policy_id
                            && effective.semantic_digest == review.effective_policy_digest)
                {
                    return Err("resource_execution_review_basis_stale".into());
                }
                posture = ResourceExecutionPosture::WaitingForReview { review_ref: review.review_id.clone() };
            }
        }
    }
    if store.get_case_state_authorized(authenticated, case_id)?.generation != state.generation {
        return Err("resource_execution_observation_stale".into());
    }
    Ok(ResourceExecutionObservation { operation_ref: operation_id.into(), case_ref: case_id.into(),
        participant_ref: participant_id.into(), generation: state.generation, posture })
}

/// Product attachment composes local resolution and the immutable canonical
/// envelope in the existing store transaction; it grants no operation policy.
pub fn attach(
    store: &LmdbRecordStore,
    authenticated: &AuthenticatedPrincipal,
    binding: &LocalAccessBinding,
    access: ResourceAccessContract,
    policy_owner: &str,
    write_envelope: Option<(&str, usize)>,
    review_requirement: ReviewRequirement,
) -> Result<CaseStateMutationResult, String> {
    let state = store.get_case_state_authorized(authenticated, &binding.case_id)?;
    let tenant = state
        .tenant_id
        .as_deref()
        .ok_or_else(|| "resource_attachment_requires_tenant".to_string())?;
    store
        .resolve_security_context(authenticated, tenant)?
        .require_owner()?;
    let attachment = ResourceAttachmentState {
        attachment_id: binding.attachment_id.clone(),
        kind: binding.kind(),
        allowed_write_prefix: write_envelope.map_or("", |(prefix, _)| prefix).into(),
        max_write_bytes: write_envelope.map_or(0, |(_, max)| max),
        policy_id: format!("policy:resource-envelope:{}", binding.attachment_id),
        policy_owner_participant_id: policy_owner.into(),
        review_requirement,
        process_signal_actions: vec![],
        access: Some(access),
    };
    binding.validate_attachment(&attachment)?;
    if let Some(old) = state
        .resources
        .iter()
        .find(|old| old.attachment_id == binding.attachment_id)
    {
        if old != &attachment
            || store
                .get_local_access_binding(&state.case_id, &binding.attachment_id)?
                .as_ref()
                != Some(binding)
        {
            return Err("resource_attachment_identity_collision".into());
        }
        return Ok(CaseStateMutationResult::unchanged(state));
    }
    let mut pending = PendingTransition::new(
        format!(
            "transition:resource:{}:{}",
            id_component(&state.case_id),
            binding.attachment_id
        ),
        &state.case_id,
        state.generation,
        TransitionSource {
            component: "yai.resource_access".into(),
            participant_id: None,
            principal_id: Some(authenticated.projected_principal_id()),
            source_ref: Some(binding.digest()),
        },
        TransitionPayload::ResourceAttached {
            attachment: attachment.clone(),
        },
    );
    pending.scope = Some(TransitionScope {
        case_id: state.case_id.clone(),
        participant_refs: vec![policy_owner.into()],
        resource_refs: vec![binding.attachment_id.clone()],
        policy_refs: vec![attachment.policy_id.clone()],
    });
    pending.causal_refs = vec![policy_owner.into()];
    store
        .commit_tenant_access_attachment(authenticated, tenant, pending, binding)
        .map(CaseStateMutationResult::committed)
}

/// Advance one canonical request. The caller (Workbench, conversation host or
/// Workflow) owns bounded iteration. No prose parsing, provider dispatch or
/// automatic external retry occurs here.
pub fn advance(
    home: &Path,
    store: &LmdbRecordStore,
    authenticated: &AuthenticatedPrincipal,
    operation: &Operation,
) -> Result<ResourceActionOutcome, String> {
    let state = store.get_case_state_authorized(authenticated, &operation.case_id)?;
    let attachment = state
        .resources
        .iter()
        .find(|attachment| attachment.attachment_id == operation.resource_attachment_id)
        .ok_or_else(|| "resource_not_attached".to_string())?;
    let access = attachment
        .access
        .as_ref()
        .ok_or_else(|| "resource_access_contract_missing".to_string())?;
    let history = store.list_case_transitions(&state.case_id)?;
    if !history.iter().any(|transition| {
        matches!(&transition.payload,
        TransitionPayload::OperationRecorded { operation: exact } if exact == operation)
    }) {
        return Err("resource_operation_not_canonical".into());
    }
    // Scope remains current even when returning an old canonical observation.
    let principal = authenticated.projected_principal_id();
    let linked = state.principal_participant_links.iter().any(|link| {
        link.principal_id == principal && link.participant_id == operation.participant_id
    });
    if !linked
        && (!matches!(
            operation.origin,
            yai_core_engine::effect::OperationOrigin::ProviderResult { .. }
        ) || store
            .resolve_security_context(authenticated, state.tenant_id.as_deref().unwrap())?
            .require_owner()
            .is_err())
    {
        return Err("resource_action_participant_not_authorized".into());
    }
    if let Some(admission) = state
        .admitted_content
        .iter()
        .find(|admission| admission.operation_id == operation.operation_id)
    {
        access.admits_request(
            &operation.participant_id,
            operation
                .resource_request
                .as_ref()
                .ok_or("resource_request_missing")?,
        )?;
        if !admission
            .participant_ids
            .contains(&operation.participant_id)
        {
            return Err("case_content_participant_not_admitted".into());
        }
        store.validate_resource_result_reuse_authorized(
            authenticated,
            &state.case_id,
            &operation.operation_id,
            &admission.decision_id,
        )?;
        return Ok(ResourceActionOutcome::ContentAdmitted {
            admission: Box::new(admission.clone()),
            reused: true,
        });
    }
    if let Some(observation) = history
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ResourceObservationRecorded { observation }
                if observation.operation_id == operation.operation_id =>
            {
                Some(observation)
            }
            _ => None,
        })
    {
        access.admits_request(
            &operation.participant_id,
            operation
                .resource_request
                .as_ref()
                .ok_or_else(|| "resource_request_missing".to_string())?,
        )?;
        store.validate_resource_result_reuse_authorized(
            authenticated,
            &state.case_id,
            &operation.operation_id,
            &observation.decision_id,
        )?;
        return Ok(ResourceActionOutcome::Observed {
            observation: Box::new(observation.clone()),
            reused: true,
        });
    }
    if let Some((observation, receipt)) =
        history
            .iter()
            .find_map(|transition| match &transition.payload {
                TransitionPayload::ResourceEffectFinalized {
                    observation,
                    receipt,
                    ..
                } if receipt.operation_id == operation.operation_id => Some((observation, receipt)),
                _ => None,
            })
    {
        access.admits_request(
            &operation.participant_id,
            operation
                .resource_request
                .as_ref()
                .ok_or("resource_request_missing")?,
        )?;
        store.validate_resource_result_reuse_authorized(
            authenticated,
            &state.case_id,
            &operation.operation_id,
            &observation.decision_id,
        )?;
        return Ok(ResourceActionOutcome::Effect {
            observation: Box::new(observation.clone()),
            receipt: Box::new(receipt.clone()),
            reused: true,
        });
    }
    if let Some(effect) = state
        .effects
        .iter()
        .find(|effect| effect.operation_id == operation.operation_id)
    {
        // PREPARE without a terminal receipt means possible external execution,
        // not a retry opportunity. A restarted frontend never dispatches it.
        return Ok(ResourceActionOutcome::Indeterminate {
            operation_id: operation.operation_id.clone(),
            effect_id: effect.effect_id.clone(),
            reason: "prepared_effect_requires_exact_result_or_reconciliation_no_redispatch".into(),
        });
    }
    // An exact participant submission that settled in refusal is not a new
    // policy evaluation request on reconnect. A fresh request identity is
    // needed to ask again; CLI and Application share this same behavior.
    if let Some(decision) = history.iter().rev().find_map(|transition| match &transition.payload {
        TransitionPayload::DecisionRecorded { decision }
            if decision.operation_id == operation.operation_id => Some(decision),
        _ => None,
    }) {
        if decision.outcome == DecisionOutcome::Deny {
            return Ok(ResourceActionOutcome::Denied {
                operation_id: operation.operation_id.clone(),
                decision_id: decision.decision_id.clone(),
                reason: decision.reason.clone(),
            });
        }
    }
    let existing_review = state
        .reviews
        .iter()
        .find(|review| review.operation_id == operation.operation_id);
    let decision = if let Some(review) = existing_review {
        match review.status {
            ReviewResolution::Pending
            | ReviewResolution::PendingOperator
            | ReviewResolution::Deferred => {
                // Revalidate policy availability before exposing a resumable review.
                if store
                    .invalidate_review_if_policy_unusable(&state.case_id, &review.review_id)?
                    .is_some()
                {
                    return Err("resource_review_policy_invalidated".into());
                }
                return Ok(ResourceActionOutcome::AwaitingReview {
                    operation_id: operation.operation_id.clone(),
                    decision_id: review.initial_decision_id.clone(),
                    review_id: review.review_id.clone(),
                });
            }
            _ => {
                let action_id = review
                    .latest_action_id
                    .as_deref()
                    .ok_or_else(|| "resource_review_not_resolvable".to_string())?;
                store
                    .derive_and_commit_policy_review_decision(
                        &state.case_id,
                        &operation.operation_id,
                        &review.review_id,
                        action_id,
                    )?
                    .0
            }
        }
    } else {
        store
            .derive_and_commit_policy_decision(&state.case_id, &operation.operation_id)?
            .0
    };
    if decision.outcome == DecisionOutcome::Deny {
        return Ok(ResourceActionOutcome::Denied {
            operation_id: operation.operation_id.clone(),
            decision_id: decision.decision_id,
            reason: decision.reason,
        });
    }
    if decision.outcome == DecisionOutcome::RequireReview {
        let current = store
            .get_case_state(&state.case_id)?
            .ok_or_else(|| "resource_case_missing".to_string())?;
        let review = build_policy_review_request(operation, &decision, current.generation)?;
        commit_review_request(store, &state.case_id, &review)?;
        return Ok(ResourceActionOutcome::AwaitingReview {
            operation_id: operation.operation_id.clone(),
            decision_id: decision.decision_id,
            review_id: review.review_id,
        });
    }
    let request = operation
        .resource_request
        .as_ref()
        .ok_or_else(|| "resource_request_missing".to_string())?;
    if matches!(
        request.action,
        ResourceAction::ProcessRun { .. } | ResourceAction::McpToolCall { .. }
    ) {
        return execute_effect(store, authenticated, operation, &decision);
    }
    if matches!(request.action, ResourceAction::AdmitContent { .. }) {
        let content_store =
            yai_core_engine::conversation::ConversationContentStore::open(home)?;
        return Ok(ResourceActionOutcome::ContentAdmitted {
            admission: Box::new(store.admit_discovered_content_authorized(
                authenticated,
                &content_store,
                &state.case_id,
                &operation.operation_id,
            )?),
            reused: false,
        });
    }
    if request.action.kind().is_external_effect() {
        return Ok(ResourceActionOutcome::Unsupported {
            operation_id: operation.operation_id.clone(),
            reason: "resource_effect_carrier_not_implemented".into(),
        });
    }
    // This read admission is immediately adjacent to adapter dispatch. Result
    // publication checks this exact generation and policy again transactionally.
    let admission = store.admit_resource_read_authorized(
        authenticated,
        &state.case_id,
        &operation.operation_id,
    )?;
    let result = if let ResourceAction::ContentRead { admission_id } = &request.action {
        // Resolve only an exact canonical admission visible through the current
        // resource envelope. Read immutable owned bytes, never reopen the source.
        let current = store.get_case_state_authorized(authenticated, &state.case_id)?;
        let material = current
            .admitted_content
            .iter()
            .find(|a| {
                a.admission_id == *admission_id
                    && a.source_resource_id == operation.resource_attachment_id
                    && a.source_configuration_digest == request.configuration_digest
                    && a.participant_ids.contains(&operation.participant_id)
            })
            .ok_or("content_read_admission_not_visible")?;
        material.validate()?;
        if material.object.byte_length
            > admission.access.max_output_bytes.saturating_sub(2048) as u64
        {
            return Err("content_read_bound_exceeded".into());
        }
        let content = yai_core_engine::conversation::ConversationContentStore::open(home)?;
        let bytes = content.read_bytes(&material.object)?;
        json!({"admission_id":material.admission_id,"object_id":material.object.object_id,
            "content_digest":material.object.content_digest,"source_path":material.source_path,
            "text":String::from_utf8(bytes).map_err(|_|"content_read_requires_text_representation")?,
            "posture":"immutable_admitted_material_not_authority"})
    } else {
        read(
            &admission.binding,
            &request.action,
            admission.access.max_output_bytes,
            admission.access.max_items,
        )?
    };
    Ok(ResourceActionOutcome::Observed {
        observation: Box::new(store.record_resource_observation_authorized(
            authenticated,
            &admission,
            result,
        )?),
        reused: false,
    })
}

fn execute_effect(
    store: &LmdbRecordStore,
    authenticated: &AuthenticatedPrincipal,
    operation: &Operation,
    decision: &Decision,
) -> Result<ResourceActionOutcome, String> {
    use yai_core_engine::effect::access::{
        PreparedResourceEffect, ResourceCarrierFailure, ResourceEffectReceipt,
    };
    let state = store.get_case_state_authorized(authenticated, &operation.case_id)?;
    let request = operation
        .resource_request
        .as_ref()
        .ok_or("resource_request_missing")?;
    let attachment = state
        .resources
        .iter()
        .find(|resource| resource.attachment_id == operation.resource_attachment_id)
        .ok_or("resource_not_attached")?;
    let access = attachment
        .access
        .as_ref()
        .ok_or("resource_access_contract_missing")?;
    access.admits_request(&operation.participant_id, request)?;
    let binding = store
        .get_local_access_binding(&state.case_id, &operation.resource_attachment_id)?
        .ok_or("local_access_binding_missing")?;
    binding.validate_attachment(attachment)?;
    let pre = ResourceObservation::new(
        operation,
        decision,
        json!({"posture":"exact_configured_request", "configuration_digest":binding.digest(), "request_digest":request.digest()}),
        now_unix_ms(),
    )?;
    let grant = issue_policy_execution_grant(operation, decision, state.generation)?;
    let current = commit_grant(store, &grant)?;
    let prepared = PreparedResourceEffect::new(operation, decision, &grant, pre)?;
    let mut pending = PendingTransition::new(
        format!("transition:resource-prepare:{}", prepared.effect_id),
        &state.case_id,
        current.generation,
        TransitionSource {
            component: "yai.resource_access".into(),
            participant_id: Some(operation.participant_id.clone()),
            principal_id: Some(authenticated.projected_principal_id()),
            source_ref: Some(operation.operation_id.clone()),
        },
        TransitionPayload::ResourceEffectPrepared {
            prepared: prepared.clone(),
        },
    );
    pending.scope = Some(operation.scope.clone());
    pending.causal_refs = vec![
        operation.operation_id.clone(),
        decision.decision_id.clone(),
        grant.grant_id.clone(),
        prepared.expected_pre_observation.observation_id.clone(),
    ];
    let commit = match store.commit_fenced_resource_effect_prepared(authenticated, pending)? {
        PreparedCommitOutcome::Prepared(commit) => commit,
        PreparedCommitOutcome::GrantInvalidated(_) => {
            return Err("resource_grant_invalidated_before_prepare".into())
        }
    };
    let TransitionPayload::ResourceEffectPrepared { prepared } = commit.transition.payload else {
        unreachable!()
    };
    let dispatched: Result<Value, ResourceCarrierFailure> = match &request.action {
        ResourceAction::ProcessRun { .. } => {
            #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
            {
                store.execute_prepared_resource_process(authenticated, &prepared)
            }
            #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
            {
                Err(ResourceCarrierFailure::before_dispatch(
                    "process_confinement_platform_not_supported",
                ))
            }
        }
        ResourceAction::McpToolCall {
            name,
            catalog_digest,
            arguments,
        } => crate::resource_transport::call_mcp_tool(
            &binding,
            catalog_digest,
            name,
            arguments,
            || {
                store
                    .validate_resource_effect_dispatch_authorized(authenticated, &prepared)
                    .map(|_| ())
            },
        ),
        _ => unreachable!("only admitted effect kinds"),
    };
    let (result, outcome, started) = match dispatched {
        Ok(value)
            if serde_json::to_vec(&value).map_err(|e| e.to_string())?.len()
                <= access.max_output_bytes =>
        {
            (value, EffectOutcome::Applied, true)
        }
        Ok(_) => {
            return record_indeterminate(
                store,
                operation,
                &prepared,
                "resource_result_bound_exceeded_after_delivery",
            )
        }
        Err(failure) if failure.outcome == EffectOutcome::FailedNoEffect => (
            json!({"posture":"not_executed", "reason":failure.reason}),
            EffectOutcome::FailedNoEffect,
            false,
        ),
        Err(failure) => return record_indeterminate(store, operation, &prepared, &failure.reason),
    };
    let observation = ResourceObservation::new(operation, decision, result, now_unix_ms())?;
    let receipt = ResourceEffectReceipt::new(&prepared, &observation, outcome, started)?;
    let current = store
        .get_case_state(&state.case_id)?
        .ok_or("case_not_visible")?;
    let mut pending = PendingTransition::new(
        format!("transition:resource-terminal:{}", prepared.effect_id),
        &state.case_id,
        current.generation,
        TransitionSource::component("yai.resource_access"),
        TransitionPayload::ResourceEffectFinalized {
            effect_id: prepared.effect_id.clone(),
            observation: observation.clone(),
            receipt: receipt.clone(),
        },
    );
    pending.scope = Some(operation.scope.clone());
    pending.causal_refs = vec![
        prepared.effect_id.clone(),
        operation.operation_id.clone(),
        grant.grant_id.clone(),
        prepared.expected_pre_observation.observation_id.clone(),
    ];
    store.commit_fenced_effect_terminal(pending, prepared.resource_fence.as_ref().unwrap())?;
    Ok(ResourceActionOutcome::Effect {
        observation: Box::new(observation),
        receipt: Box::new(receipt),
        reused: false,
    })
}

fn record_indeterminate(
    store: &LmdbRecordStore,
    operation: &Operation,
    prepared: &yai_core_engine::effect::access::PreparedResourceEffect,
    reason: &str,
) -> Result<ResourceActionOutcome, String> {
    let reason: String = reason.chars().take(240).collect();
    commit_effect_transition(
        store,
        &operation.case_id,
        Some(&operation.participant_id),
        &format!("resource-indeterminate:{}", prepared.effect_id),
        TransitionPayload::ResourceEffectIndeterminate {
            effect_id: prepared.effect_id.clone(),
            reason: reason.clone(),
        },
        Some(operation.scope.clone()),
        vec![prepared.effect_id.clone()],
    )?;
    Ok(ResourceActionOutcome::Indeterminate {
        operation_id: operation.operation_id.clone(),
        effect_id: prepared.effect_id.clone(),
        reason,
    })
}

fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn read(
    binding: &LocalAccessBinding,
    action: &ResourceAction,
    bytes: usize,
    items: usize,
) -> Result<Value, String> {
    let root = || {
        binding
            .root()
            .ok_or_else(|| "resource_local_root_required".to_string())
    };
    let result = match action {
        ResourceAction::FilesystemRead { path } => {
            let data = read_confined_file(root()?, path, bytes)?;
            json!({"path":path,"content_digest":yai_core_engine::effect::digest_bytes(&data),
                "text":String::from_utf8(data).map_err(|_| "resource_read_nontext_use_immutable_import".to_string())?})
        }
        ResourceAction::FilesystemSearch { path, needle } => {
            inspect_confined_tree(root()?, path, Some(needle), bytes, items)?
        }
        ResourceAction::Discover { path } => {
            inspect_confined_tree(root()?, path, None, bytes, items)?
        }
        ResourceAction::DatabaseQuery { name } => {
            query_sqlite_snapshot(binding, name, items, bytes)?
        }
        ResourceAction::HttpFetch { name } => {
            crate::resource_transport::fetch_http(binding, name, bytes)?
        }
        ResourceAction::McpCatalog => {
            serde_json::to_value(crate::resource_transport::inspect_mcp(binding)?)
                .map_err(|e| e.to_string())?
        }
        ResourceAction::McpResourceRead {
            uri,
            catalog_digest,
        } => crate::resource_transport::read_mcp_resource(binding, catalog_digest, uri)?,
        _ => return Err("resource_read_cannot_execute_effect_or_admission".into()),
    };
    if serde_json::to_vec(&result)
        .map_err(|e| e.to_string())?
        .len()
        > bytes
    {
        return Err("resource_observation_output_bound_exceeded".into());
    }
    Ok(result)
}

pub const CONTROLLED_EFFECT_COMPONENT: &str = "yai.controlled_filesystem_effect";

pub fn id_component(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, ':' | '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

pub fn effect_source(participant_id: Option<&str>, source_ref: &str) -> TransitionSource {
    TransitionSource {
        component: CONTROLLED_EFFECT_COMPONENT.to_string(),
        participant_id: participant_id.map(ToString::to_string),
        principal_id: None,
        source_ref: Some(source_ref.to_string()),
    }
}

pub fn commit_effect_transition(
    store: &LmdbRecordStore,
    case_id: &str,
    participant_id: Option<&str>,
    label: &str,
    payload: TransitionPayload,
    scope: Option<TransitionScope>,
    causal_refs: Vec<String>,
) -> Result<CaseState, String> {
    let pending = build_effect_pending(
        store,
        case_id,
        participant_id,
        label,
        payload,
        scope,
        causal_refs,
    )?;
    store.commit_transition(pending).map(|commit| commit.state)
}

pub fn build_effect_pending(
    store: &LmdbRecordStore,
    case_id: &str,
    participant_id: Option<&str>,
    label: &str,
    payload: TransitionPayload,
    scope: Option<TransitionScope>,
    causal_refs: Vec<String>,
) -> Result<PendingTransition, String> {
    let generation = store
        .get_case_state(case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?
        .generation;
    let transition_id = format!(
        "transition:controlled-effect:{}:{:020}:{}",
        id_component(case_id),
        generation + 1,
        id_component(label)
    );
    let mut pending = PendingTransition::new(
        transition_id,
        case_id,
        generation,
        effect_source(participant_id, label),
        payload,
    );
    pending.scope = scope;
    pending.causal_refs = causal_refs;
    Ok(pending)
}

pub fn commit_review_request(
    store: &LmdbRecordStore,
    case_id: &str,
    review: &ReviewState,
) -> Result<CaseState, String> {
    let mut causal_refs = vec![
        review.operation_id.clone(),
        review.initial_decision_id.clone(),
    ];
    if review.schema == REVIEW_REQUEST_SCHEMA {
        causal_refs.push(review.decision_basis_id.clone());
        causal_refs.push(review.effective_policy_id.clone());
        causal_refs.extend(review.policy_binding_refs.iter().cloned());
        causal_refs.extend(review.policy_artifact_refs.iter().cloned());
    }
    commit_effect_transition(
        store,
        case_id,
        Some(&review.requested_by_participant),
        &format!("review-request:{}", review.review_id),
        TransitionPayload::ReviewRequested {
            review: review.clone(),
        },
        None,
        causal_refs,
    )
}

pub fn commit_grant(store: &LmdbRecordStore, grant: &ExecutionGrant) -> Result<CaseState, String> {
    let mut causal_refs = vec![grant.operation_id.clone(), grant.decision_id.clone()];
    if grant.has_current_policy_basis() {
        if let Some(basis_id) = &grant.decision_basis_id {
            causal_refs.push(basis_id.clone());
        }
        if let Some(effective_policy_id) = &grant.effective_policy_id {
            causal_refs.push(effective_policy_id.clone());
        }
        causal_refs.extend(grant.policy_binding_refs.iter().cloned());
        causal_refs.extend(grant.policy_artifact_refs.iter().cloned());
        if let Some(action_id) = &grant.review_action_ref {
            causal_refs.push(action_id.clone());
        }
    }
    commit_effect_transition(
        store,
        &grant.case_id,
        Some(&grant.participant_id),
        &format!("grant:{}", grant.grant_id),
        TransitionPayload::ExecutionGrantIssued {
            grant: grant.clone(),
        },
        None,
        causal_refs,
    )
}


use yai_core_engine::effect::{
    PreparedEffect, PreparedProcessEffect, CarrierResult, ProcessCarrierResult,
    FilesystemObservation, ResourceState, CarrierFailpoint, OperationKind,
    build_effect_receipt, build_process_effect_receipt, prepare_fenced_effect,
    prepare_process_effect, execute_fenced_filesystem_write, execute_fenced_process_signal,
    process_signal_retry_posture, validate_finalized_effect_chain,
};
use yai_core_engine::transition::{CaseLifecycle, EffectLifecycle, ResourceKind, Transition};

/// Optional diagnostics and qualification failpoints. Hooks cannot supply a
/// Decision, Grant, receipt or authority; all admission remains in the owners.
pub trait ControlledEffectHooks {
    fn report(&mut self, _message: String) {}
    fn failpoint(&self) -> Option<&str> { None }
    fn interrupt(&mut self, name: &str, _code: i32) -> Result<(), String> {
        Err(format!("controlled_effect_interrupted:{name}"))
    }
    fn after_commit(&mut self, store: &LmdbRecordStore, case: &str) {
        let _ = store.materialize_graph_relations_for_case(case);
        let _ = store.list_case_transitions(case).and_then(|history| {
            yai_core_engine::memory::derive_operational_memory(case, &history)
                .and_then(|build| store.replace_case_operational_memory(&build).map(|_| ()))
        });
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ControlledEffectObservation {
    pub schema: String,
    pub case_ref: String,
    pub participant_ref: String,
    pub operation_ref: String,
    pub observed_generation: u64,
    /// None means no waiting/refused/effect posture is recorded yet.
    /// A historical status never grants permission to advance it.
    pub progress: Option<ControlledEffectTurnResult>,
}

pub fn observe_controlled_effect(
    store: &LmdbRecordStore, authenticated: &AuthenticatedPrincipal,
    case: &str, participant: &str, operation: &str,
) -> Result<ControlledEffectObservation, String> {
    let exact = store.controlled_operation_authorized(authenticated, case, participant, operation)?;
    if matches!(exact.kind, OperationKind::ResourceAccess(_)) {
        return Err("controlled_effect_kind_required".into());
    }
    let state = store.get_case_state_authorized(authenticated, case)?;
    let history = store.list_case_transitions(case)?;
    let progress = if let Some(effect) = state.effects.iter().find(|e| e.operation_id == operation) {
        Some(ControlledEffectTurnResult {
            status: if effect.status == EffectLifecycle::Finalized {
                ControlledEffectTurnStatus::Finalized
            } else { ControlledEffectTurnStatus::Indeterminate },
            operation_id: Some(operation.into()), decision_id: Some(effect.decision_id.clone()),
            review_id: None, effect_id: Some(effect.effect_id.clone()),
            receipt_id: effect.receipt_id.clone(), outcome: effect.outcome.clone(),
        })
    } else {
        history.iter().rev().find_map(|t| match &t.payload {
            TransitionPayload::DecisionRecorded { decision } if decision.operation_id == operation => Some(decision),
            _ => None,
        }).filter(|decision| decision.outcome != DecisionOutcome::Allow)
            .map(|decision| ControlledEffectTurnResult {
                    status: if decision.outcome == DecisionOutcome::Deny {
                        ControlledEffectTurnStatus::Denied
                    } else { ControlledEffectTurnStatus::AwaitingReview },
                    operation_id: Some(operation.into()), decision_id: Some(decision.decision_id.clone()),
                    review_id: state.reviews.iter().find(|r| r.operation_id == operation).map(|r| r.review_id.clone()),
                    effect_id: None, receipt_id: None, outcome: None,
                })
    };
    store.controlled_operation_authorized(authenticated, case, participant, operation)?;
    if store.get_case_state_authorized(authenticated, case)?.generation != state.generation {
        return Err("controlled_effect_observation_stale".into());
    }
    Ok(ControlledEffectObservation {
        schema: "yai.controlled_effect_observation.v1".into(), case_ref: case.into(),
        participant_ref: participant.into(), operation_ref: operation.into(),
        observed_generation: state.generation, progress,
    })
}

pub fn commit_prepare(
    store: &LmdbRecordStore,
    prepared: &PreparedEffect,
) -> Result<(PreparedEffect, CaseState), String> {
    let generation = store
        .get_case_state(&prepared.case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {}", prepared.case_id))?
        .generation;
    let label = format!("prepare:{}", prepared.effect_id);
    let mut pending = PendingTransition::new(
        format!(
            "transition:controlled-effect:{}:{:020}:{}",
            id_component(&prepared.case_id),
            generation + 1,
            id_component(&label)
        ),
        &prepared.case_id,
        generation,
        effect_source(Some(&prepared.participant_id), &label),
        TransitionPayload::EffectPrepared {
            prepared: prepared.clone(),
        },
    );
    pending.causal_refs = vec![
        prepared.operation_id.clone(),
        prepared.decision_id.clone(),
        prepared.grant_id.clone(),
        prepared.expected_pre_observation.observation_id.clone(),
    ];
    match store.commit_fenced_effect_prepared(pending, std::process::id())? {
        PreparedCommitOutcome::Prepared(commit) => {
            let committed = match &commit.transition.payload {
                TransitionPayload::EffectPrepared { prepared } => prepared.clone(),
                _ => unreachable!("prepared commit payload"),
            };
            Ok((committed, commit.state))
        }
        PreparedCommitOutcome::GrantInvalidated(commit) => Err(format!(
            "execution_grant_invalidated_before_prepare: generation={}",
            commit.state.generation
        )),
    }
}

pub fn commit_process_prepare(
    store: &LmdbRecordStore,
    prepared: &PreparedProcessEffect,
) -> Result<(PreparedProcessEffect, CaseState), String> {
    let generation = store
        .get_case_state(&prepared.case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {}", prepared.case_id))?
        .generation;
    let label = format!("process-prepare:{}", prepared.effect_id);
    let mut pending = PendingTransition::new(
        format!(
            "transition:controlled-effect:{}:{:020}:{}",
            id_component(&prepared.case_id),
            generation + 1,
            id_component(&label)
        ),
        &prepared.case_id,
        generation,
        effect_source(Some(&prepared.participant_id), &label),
        TransitionPayload::ProcessEffectPrepared {
            prepared: prepared.clone(),
        },
    );
    pending.causal_refs = vec![
        prepared.operation_id.clone(),
        prepared.decision_id.clone(),
        prepared.grant_id.clone(),
        prepared.expected_pre_observation.observation_id.clone(),
    ];
    match store.commit_fenced_process_effect_prepared(pending, std::process::id())? {
        PreparedCommitOutcome::Prepared(commit) => {
            let committed = match &commit.transition.payload {
                TransitionPayload::ProcessEffectPrepared { prepared } => prepared.clone(),
                _ => unreachable!("prepared process commit payload"),
            };
            Ok((committed, commit.state))
        }
        PreparedCommitOutcome::GrantInvalidated(commit) => Err(format!(
            "execution_grant_invalidated_before_prepare: generation={}",
            commit.state.generation
        )),
    }
}

pub fn commit_indeterminate(
    store: &LmdbRecordStore,
    prepared: &PreparedEffect,
    reason: String,
    observation: Option<FilesystemObservation>,
) -> Result<CaseState, String> {
    commit_effect_transition(
        store,
        &prepared.case_id,
        Some(&prepared.participant_id),
        &format!("indeterminate:{}", prepared.effect_id),
        TransitionPayload::EffectIndeterminate {
            effect_id: prepared.effect_id.clone(),
            reason,
            observation,
        },
        None,
        vec![prepared.effect_id.clone()],
    )
}

pub fn commit_process_indeterminate(
    store: &LmdbRecordStore,
    prepared: &PreparedProcessEffect,
    reason: String,
    observation: Option<yai_core_engine::effect::ProcessObservation>,
) -> Result<CaseState, String> {
    commit_effect_transition(
        store,
        &prepared.case_id,
        Some(&prepared.participant_id),
        &format!("process-indeterminate:{}", prepared.effect_id),
        TransitionPayload::ProcessEffectIndeterminate {
            effect_id: prepared.effect_id.clone(),
            reason,
            observation,
        },
        None,
        vec![prepared.effect_id.clone()],
    )
}

pub fn commit_finalize(
    store: &LmdbRecordStore,
    prepared: &PreparedEffect,
    result: &CarrierResult,
) -> Result<CaseState, String> {
    let receipt = build_effect_receipt(prepared, result);
    let pending = build_effect_pending(
        store,
        &prepared.case_id,
        Some(&prepared.participant_id),
        &format!("finalize:{}", prepared.effect_id),
        TransitionPayload::EffectFinalized {
            effect_id: prepared.effect_id.clone(),
            post_observation: result.post_observation.clone(),
            receipt: receipt.clone(),
        },
        None,
        vec![prepared.effect_id.clone(), receipt.receipt_id],
    )?;
    if let Some(fence) = &prepared.resource_fence {
        store
            .commit_fenced_effect_terminal(pending, fence)
            .map(|commit| commit.state)
    } else {
        store.commit_transition(pending).map(|commit| commit.state)
    }
}

pub fn commit_process_finalize(
    store: &LmdbRecordStore,
    prepared: &PreparedProcessEffect,
    result: &ProcessCarrierResult,
) -> Result<CaseState, String> {
    let receipt = build_process_effect_receipt(prepared, result);
    let pending = build_effect_pending(
        store,
        &prepared.case_id,
        Some(&prepared.participant_id),
        &format!("process-finalize:{}", prepared.effect_id),
        TransitionPayload::ProcessEffectFinalized {
            effect_id: prepared.effect_id.clone(),
            observation: result.post_observation.clone(),
            receipt: receipt.clone(),
        },
        None,
        vec![prepared.effect_id.clone(), receipt.receipt_id],
    )?;
    let fence = prepared
        .resource_fence
        .as_ref()
        .ok_or_else(|| "prepared_process_effect_resource_fence_missing".to_string())?;
    store
        .commit_fenced_effect_terminal(pending, fence)
        .map(|commit| commit.state)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlledEffectTurnStatus {
    NormalizationRejected,
    Denied,
    AwaitingReview,
    Finalized,
    Indeterminate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ControlledEffectTurnResult {
    pub status: ControlledEffectTurnStatus,
    pub operation_id: Option<String>,
    pub decision_id: Option<String>,
    pub review_id: Option<String>,
    pub effect_id: Option<String>,
    pub receipt_id: Option<String>,
    pub outcome: Option<EffectOutcome>,
}

fn resource_for_case(
    state: &CaseState,
    attachment_id: &str,
) -> Result<ResourceAttachmentState, String> {
    state
        .resources
        .iter()
        .find(|resource| resource.attachment_id == attachment_id)
        .cloned()
        .ok_or_else(|| {
            format!(
                "resource attachment {attachment_id} is not bound to {}",
                state.case_id
            )
        })
}

pub fn advance_controlled_operation(
    authenticated: &AuthenticatedPrincipal,
    hooks: &mut dyn ControlledEffectHooks,
    store: &LmdbRecordStore,
    operation: &Operation,
) -> Result<ControlledEffectTurnResult, String> {
    macro_rules! report { ($($arg:tt)*) => { hooks.report(format!($($arg)*)) }; }
    let exact = store.controlled_operation_authorized(authenticated, &operation.case_id,
        &operation.participant_id, &operation.operation_id)?;
    if exact != *operation {
        return Err("controlled_operation_not_canonical".into());
    }
    let operation = operation.clone();
    let case_id = operation.case_id.clone();
    let case_id = case_id.as_str();
    let attachment_id = operation.resource_attachment_id.clone();
    let attachment_id = attachment_id.as_str();
    let state = store
        .get_case_state_authorized(authenticated, case_id)?;
    let resource = resource_for_case(&state, attachment_id)?;
    let existing = store.list_case_transitions(case_id)?;
    if !existing.iter().any(|t|matches!(&t.payload,TransitionPayload::OperationRecorded {operation:exact} if exact == &operation)) {
        return Err("controlled_operation_not_canonical".into());
    }
    if let Some(effect) = state
        .effects
        .iter()
        .find(|effect| effect.operation_id == operation.operation_id)
    {
        // PREPARE without a terminal record is possibly delivered. The existing
        // reconciliation surface must establish its outcome; a new host call
        // must not blindly re-enter a carrier after restart.
        return Ok(ControlledEffectTurnResult {
            status: if effect.status == EffectLifecycle::Finalized {
                ControlledEffectTurnStatus::Finalized
            } else {
                ControlledEffectTurnStatus::Indeterminate
            },
            operation_id: Some(operation.operation_id),
            decision_id: Some(effect.decision_id.clone()),
            review_id: None,
            effect_id: Some(effect.effect_id.clone()),
            receipt_id: effect.receipt_id.clone(),
            outcome: effect.outcome.clone(),
        });
    }
    if state.lifecycle == CaseLifecycle::Closed || state.cancellation.is_some() {
        return Err("case_closed_or_cancelled_new_effect_forbidden".into());
    }
    if matches!(operation.kind, OperationKind::ResourceAccess(_)) {
        return Err("resource_access_requires_typed_capability_execution".into());
    }
    report!("operation_normalization: accepted");
    report!("operation_id: {}", operation.operation_id);
    report!(
        "operation_kind: {}",
        match operation.kind {
            OperationKind::ResourceAccess(kind) => kind.operation_name(),
            OperationKind::FilesystemWrite => "filesystem.write",
            OperationKind::ProcessSignal => "process.signal",
        }
    );

    let existing_decisions = existing
        .iter()
        .filter_map(|transition| match &transition.payload {
            TransitionPayload::DecisionRecorded { decision }
                if decision.operation_id == operation.operation_id =>
            {
                Some(decision.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let existing_effective = existing_decisions
        .iter()
        .rev()
        .find(|decision| decision.outcome != DecisionOutcome::RequireReview)
        .cloned();
    let (mut decision, mut state_after_decision) = if let Some(decision) = existing_effective {
        let state = store
            .get_case_state(case_id)?
            .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
        (decision, state)
    } else {
        let (initial, state_after_initial) = if let Some(decision) = existing_decisions
            .iter()
            .find(|decision| decision.outcome == DecisionOutcome::RequireReview)
            .cloned()
        {
            let state = store
                .get_case_state(case_id)?
                .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
            (decision, state)
        } else {
            let (decision, commit) =
                store.derive_and_commit_policy_decision(case_id, &operation.operation_id)?;
            (decision, commit.state)
        };
        if initial.outcome != DecisionOutcome::RequireReview {
            (initial, state_after_initial)
        } else {
            if matches!(
                hooks.failpoint(),
                Some("review_after_require_decision" | "review_r1")
            ) {
                hooks.interrupt("review_after_require_decision", 94)?;
            }
            let review = if let Some(review) = state_after_initial
                .reviews
                .iter()
                .find(|review| review.operation_id == operation.operation_id)
                .cloned()
            {
                review
            } else {
                let review = build_policy_review_request(
                    &operation,
                    &initial,
                    state_after_initial.generation,
                )?;
                commit_review_request(store, case_id, &review)?;
                if matches!(
                    hooks.failpoint(),
                    Some("review_after_request" | "review_r2")
                ) {
                    hooks.interrupt("review_after_request", 95)?;
                }
                review
            };
            if matches!(
                review.status,
                ReviewResolution::Pending
                    | ReviewResolution::PendingOperator
                    | ReviewResolution::Deferred
            ) {
                hooks.after_commit(store, case_id);
                report!("decision: require_review");
                report!("review_id: {}", review.review_id);
                report!("execution_grant: none");
                report!("external_effect: none");
                return Ok(ControlledEffectTurnResult {
                    status: ControlledEffectTurnStatus::AwaitingReview,
                    operation_id: Some(operation.operation_id),
                    decision_id: Some(initial.decision_id),
                    review_id: Some(review.review_id),
                    effect_id: None,
                    receipt_id: None,
                    outcome: None,
                });
            }
            if review.status == ReviewResolution::Quarantined {
                return Err("legacy_quarantined_review_is_not_executable".to_string());
            }
            let action_id = review
                .latest_action_id
                .as_ref()
                .ok_or_else(|| "resolved review is missing its human action".to_string())?;
            let action = existing
                .iter()
                .find_map(|transition| match &transition.payload {
                    TransitionPayload::ReviewActionRecorded { action }
                        if action.action_id == *action_id =>
                    {
                        Some(action.clone())
                    }
                    _ => None,
                })
                .ok_or_else(|| "review action transition is missing".to_string())?;
            let (effective, commit) = store.derive_and_commit_policy_review_decision(
                case_id,
                &operation.operation_id,
                &review.review_id,
                &action.action_id,
            )?;
            let state = commit.state;
            if effective.outcome == DecisionOutcome::Allow
                && matches!(
                    hooks.failpoint(),
                    Some("review_after_allow_decision" | "review_r4")
                )
            {
                hooks.interrupt("review_after_allow_decision", 96)?;
            }
            if effective.outcome == DecisionOutcome::Deny
                && matches!(
                    hooks.failpoint(),
                    Some("review_after_deny_decision" | "review_r6")
                )
            {
                hooks.interrupt("review_after_deny_decision", 97)?;
            }
            (effective, state)
        }
    };
    let existing_grant = existing
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ExecutionGrantIssued { grant }
                if grant.operation_id == operation.operation_id =>
            {
                state_after_decision
                    .grants
                    .iter()
                    .find(|current| {
                        current.grant_id == grant.grant_id
                            && current.status == yai_core_engine::transition::GrantLifecycle::Issued
                    })
                    .map(|_| grant.clone())
            }
            _ => None,
        });
    if existing_grant.is_none()
        && decision.outcome == DecisionOutcome::Allow
        && state_after_decision.generation != decision.decided_at_case_generation + 1
    {
        // H10 freshness is deliberately transition-adjacent. Any canonical
        // transition after ALLOW requires a new semantic derivation; the
        // runtime never classifies intervening state as harmless and never
        // asks the provider to synthesize another Operation.
        let (refreshed, commit) = if let Some(action_id) = decision
            .decision_basis
            .as_ref()
            .and_then(|basis| basis.review_action_ref.as_deref())
        {
            let review = state_after_decision
                .reviews
                .iter()
                .find(|review| {
                    review.operation_id == operation.operation_id
                        && review.latest_action_id.as_deref() == Some(action_id)
                })
                .ok_or_else(|| "canonical_review_resolution_not_current".to_string())?;
            store.derive_and_commit_policy_review_decision(
                case_id,
                &operation.operation_id,
                &review.review_id,
                action_id,
            )?
        } else {
            store.derive_and_commit_policy_decision(case_id, &operation.operation_id)?
        };
        state_after_decision = commit.state;
        decision = refreshed;
    }
    report!("decision_id: {}", decision.decision_id);
    report!("decision_reason: {}", decision.reason);
    if let Some(basis) = &decision.decision_basis {
        report!("decision_basis_id: {}", basis.basis_id);
        report!("effective_policy_id: {}", basis.effective_policy_id);
        report!(
            "matched_policy_rules: {}",
            basis.matched_rule_refs.join(",")
        );
        report!(
            "authority_requirements: {}",
            serde_json::to_string(&basis.authority)
                .map_err(|error| format!("authority_render_failed: {error}"))?
        );
        report!(
            "evidence_obligations: {}",
            serde_json::to_string(&basis.obligations)
                .map_err(|error| format!("obligation_render_failed: {error}"))?
        );
    }
    report!(
        "decision: {}",
        match decision.outcome {
            DecisionOutcome::Allow => "allow",
            DecisionOutcome::Deny => "deny",
            DecisionOutcome::RequireReview => "require_review",
        }
    );
    if decision.outcome == DecisionOutcome::Deny {
        hooks.after_commit(store, case_id);
        report!("execution_grant: none");
        report!("external_effect: none");
        return Ok(ControlledEffectTurnResult {
            status: ControlledEffectTurnStatus::Denied,
            operation_id: Some(operation.operation_id.clone()),
            decision_id: Some(decision.decision_id),
            review_id: state_after_decision
                .reviews
                .iter()
                .find(|review| review.operation_id == operation.operation_id)
                .map(|review| review.review_id.clone()),
            effect_id: None,
            receipt_id: None,
            outcome: None,
        });
    }

    let grant = if let Some(grant) = existing_grant {
        grant
    } else {
        if state_after_decision
            .last_decision
            .as_ref()
            .is_none_or(|current| current.decision_id != decision.decision_id)
        {
            return Err("execution_grant_requires_latest_case_decision".to_string());
        }
        let grant =
            issue_policy_execution_grant(&operation, &decision, state_after_decision.generation)?;
        commit_grant(store, &grant)?;
        grant
    };
    report!("execution_grant_id: {}", grant.grant_id);
    report!(
        "execution_grant_decision_basis_id: {}",
        grant.decision_basis_id.as_deref().unwrap_or("none")
    );
    if matches!(
        hooks.failpoint(),
        Some("after_grant_before_prepare" | "review_r5")
    ) {
        hooks.interrupt("after_grant_before_prepare", 84)?;
    }

    if operation.kind == OperationKind::ProcessSignal {
        return advance_process_signal_after_grant(
            authenticated, hooks, store, &existing, &operation, &decision, &grant, &resource,
        );
    }

    let binding = store
        .get_local_filesystem_binding(case_id, attachment_id)?
        .ok_or_else(|| format!("local binding missing for {case_id}/{attachment_id}"))?;

    let existing_prepared = existing
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::EffectPrepared { prepared }
                if prepared.operation_id == operation.operation_id =>
            {
                Some(prepared.clone())
            }
            _ => None,
        });
    let (prepared, state_after_prepare) = if let Some(prepared) = existing_prepared {
        let current = store
            .get_case_state(case_id)?
            .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
        if let Some(effect) = current
            .effects
            .iter()
            .find(|effect| effect.effect_id == prepared.effect_id)
        {
            if effect.status == EffectLifecycle::Finalized {
                return Ok(ControlledEffectTurnResult {
                    status: ControlledEffectTurnStatus::Finalized,
                    operation_id: Some(operation.operation_id),
                    decision_id: Some(decision.decision_id),
                    review_id: None,
                    effect_id: Some(prepared.effect_id),
                    receipt_id: effect.receipt_id.clone(),
                    outcome: effect.outcome.clone(),
                });
            }
            if effect.status == EffectLifecycle::Indeterminate {
                return Ok(ControlledEffectTurnResult {
                    status: ControlledEffectTurnStatus::Indeterminate,
                    operation_id: Some(operation.operation_id),
                    decision_id: Some(decision.decision_id),
                    review_id: None,
                    effect_id: Some(prepared.effect_id),
                    receipt_id: None,
                    outcome: effect.outcome.clone(),
                });
            }
        }
        (prepared, current)
    } else {
        let pre_observation = store.observe_filesystem_authorized(
            authenticated,
            case_id,
            &operation.operation_id,
            &grant.grant_id,
            &format!("observation:{}:pre", grant.grant_id),
        )?;
        if pre_observation.state == ResourceState::Unavailable {
            return Err(format!(
                "pre_effect_observation_unavailable: {}",
                pre_observation.error.as_deref().unwrap_or("unknown")
            ));
        }
        let prepared = prepare_fenced_effect(&operation, &decision, &grant, pre_observation)?;
        commit_prepare(store, &prepared)?
    };
    let fence = prepared
        .resource_fence
        .as_ref()
        .ok_or_else(|| "prepared_effect_resource_fence_missing".to_string())?;
    report!("effect_id: {}", prepared.effect_id);
    report!("resource_id: {}", fence.resource_id);
    report!("resource_epoch: {}", fence.resource_epoch);
    report!("resource_fence_id: {}", fence.fence_id);
    report!("effect_state: prepared_durable_before_mutation");
    if hooks.failpoint() == Some("after_prepare_before_effect") {
        hooks.interrupt("after_prepare_before_effect", 85)?;
    }

    let carrier_failpoint = match hooks.failpoint() {
        Some("carrier_failure") => CarrierFailpoint::FailBeforeMutation,
        Some("after_effect_before_finalize") => CarrierFailpoint::CrashAfterVisibleEffect,
        _ => CarrierFailpoint::None,
    };
    let result = execute_fenced_filesystem_write(
        store,
        fence,
        &operation,
        &decision,
        &grant,
        &prepared,
        &state_after_prepare,
        &binding,
        &resource,
        carrier_failpoint,
    )?;
    if result.crash_injected_after_effect {
        hooks.interrupt("after_effect_before_finalize", 86)?;
    }
    if matches!(
        result.outcome,
        EffectOutcome::Conflict | EffectOutcome::Indeterminate
    ) {
        let state = commit_indeterminate(
            store,
            &prepared,
            result.detail.clone(),
            Some(result.post_observation),
        )?;
        hooks.after_commit(store, case_id);
        report!("effect_state: indeterminate");
        report!("case_generation: {}", state.generation);
        return Ok(ControlledEffectTurnResult {
            status: ControlledEffectTurnStatus::Indeterminate,
            operation_id: Some(operation.operation_id),
            decision_id: Some(decision.decision_id),
            review_id: None,
            effect_id: Some(prepared.effect_id),
            receipt_id: None,
            outcome: Some(result.outcome),
        });
    }
    let receipt = build_effect_receipt(&prepared, &result);
    if hooks.failpoint() == Some("after_receipt_before_finalize") {
        report!("prepared_receipt_id: {}", receipt.receipt_id);
        hooks.interrupt("after_receipt_before_finalize", 87)?;
    }
    commit_finalize(store, &prepared, &result)?;
    if hooks.failpoint() == Some("after_terminal_resource_release_commit") {
        hooks.interrupt("after_terminal_resource_release_commit", 89)?;
    }
    report!("effect_receipt_id: {}", receipt.receipt_id);
    report!("effect_outcome: {:?}", result.outcome);
    report!("effect_state: finalized");
    hooks.after_commit(store, case_id);
    let transitions = store.list_case_transitions(case_id)?;
    validate_finalized_effect_chain(&transitions, &prepared.effect_id)?;
    report!("effect_chain_closure: valid");
    Ok(ControlledEffectTurnResult {
        status: ControlledEffectTurnStatus::Finalized,
        operation_id: Some(operation.operation_id),
        decision_id: Some(decision.decision_id),
        review_id: None,
        effect_id: Some(prepared.effect_id),
        receipt_id: Some(receipt.receipt_id),
        outcome: Some(result.outcome),
    })
}

fn advance_process_signal_after_grant(
    authenticated: &AuthenticatedPrincipal,
    hooks: &mut dyn ControlledEffectHooks,
    store: &LmdbRecordStore,
    existing: &[Transition],
    operation: &Operation,
    decision: &Decision,
    grant: &ExecutionGrant,
    resource: &ResourceAttachmentState,
) -> Result<ControlledEffectTurnResult, String> {
    macro_rules! report { ($($arg:tt)*) => { hooks.report(format!($($arg)*)) }; }
    if resource.kind != ResourceKind::Process {
        return Err("process_operation_resource_kind_mismatch".to_string());
    }
    let binding = store
        .get_local_process_binding(&operation.case_id, &operation.resource_attachment_id)?
        .ok_or_else(|| "local_process_binding_missing".to_string())?;
    let existing_prepared = existing
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ProcessEffectPrepared { prepared }
                if prepared.operation_id == operation.operation_id =>
            {
                Some(prepared.clone())
            }
            _ => None,
        });
    let (prepared, state_after_prepare) = if let Some(prepared) = existing_prepared {
        let current = store
            .get_case_state(&operation.case_id)?
            .ok_or_else(|| format!("canonical CaseState missing for {}", operation.case_id))?;
        if let Some(effect) = current
            .effects
            .iter()
            .find(|effect| effect.effect_id == prepared.effect_id)
        {
            if effect.status == EffectLifecycle::Finalized {
                return Ok(ControlledEffectTurnResult {
                    status: ControlledEffectTurnStatus::Finalized,
                    operation_id: Some(operation.operation_id.clone()),
                    decision_id: Some(decision.decision_id.clone()),
                    review_id: None,
                    effect_id: Some(prepared.effect_id),
                    receipt_id: effect.receipt_id.clone(),
                    outcome: effect.outcome.clone(),
                });
            }
            if effect.status == EffectLifecycle::Indeterminate {
                return Ok(ControlledEffectTurnResult {
                    status: ControlledEffectTurnStatus::Indeterminate,
                    operation_id: Some(operation.operation_id.clone()),
                    decision_id: Some(decision.decision_id.clone()),
                    review_id: None,
                    effect_id: Some(prepared.effect_id),
                    receipt_id: None,
                    outcome: effect.outcome.clone(),
                });
            }
            if effect.status == EffectLifecycle::Prepared {
                let observation = store.observe_process_authorized(
                    authenticated,
                    &operation.case_id,
                    &operation.operation_id,
                    &grant.grant_id,
                    &format!("observation:{}:uncertain-recovery", prepared.effect_id),
                )?;
                let posture = process_signal_retry_posture(&prepared.action);
                let state = commit_process_indeterminate(
                    store,
                    &prepared,
                    format!(
                        "process_signal_acknowledgement_missing: retry_posture={posture:?}; observation_only_recovery"
                    ),
                    Some(observation),
                )?;
                report!("process_retry_posture: {posture:?}");
                report!("process_signal_repeated: false");
                report!("effect_state: indeterminate");
                return Ok(ControlledEffectTurnResult {
                    status: ControlledEffectTurnStatus::Indeterminate,
                    operation_id: Some(operation.operation_id.clone()),
                    decision_id: Some(decision.decision_id.clone()),
                    review_id: None,
                    effect_id: Some(prepared.effect_id),
                    receipt_id: None,
                    outcome: state
                        .effects
                        .iter()
                        .find(|candidate| candidate.operation_id == operation.operation_id)
                        .and_then(|candidate| candidate.outcome.clone()),
                });
            }
        }
        (prepared, current)
    } else {
        let pre = store.observe_process_authorized(
            authenticated,
            &operation.case_id,
            &operation.operation_id,
            &grant.grant_id,
            &format!("observation:{}:pre", grant.grant_id),
        )?;
        if matches!(
            pre.state,
            yai_core_engine::effect::ProcessObservedState::Unavailable
                | yai_core_engine::effect::ProcessObservedState::Exited
        ) {
            return Err(format!("process_pre_observation_not_live: {:?}", pre.state));
        }
        let prepared = prepare_process_effect(operation, decision, grant, pre)?;
        commit_process_prepare(store, &prepared)?
    };
    let fence = prepared
        .resource_fence
        .as_ref()
        .ok_or_else(|| "prepared_process_effect_resource_fence_missing".to_string())?;
    report!("effect_id: {}", prepared.effect_id);
    report!("resource_id: {}", fence.resource_id);
    report!("resource_epoch: {}", fence.resource_epoch);
    report!("resource_fence_id: {}", fence.fence_id);
    report!("effect_state: prepared_durable_before_signal");
    if hooks.failpoint() == Some("after_prepare_before_effect") {
        hooks.interrupt("after_prepare_before_effect", 85)?;
    }
    let result = execute_fenced_process_signal(
        store,
        fence,
        operation,
        decision,
        grant,
        &prepared,
        &state_after_prepare,
        &binding,
    )?;
    if hooks.failpoint() == Some("after_process_signal_before_finalize") {
        report!("kernel_signal: {}", result.kernel_signal);
        report!("kernel_syscall_accepted: {}", result.syscall_accepted);
        hooks.interrupt("after_process_signal_before_finalize", 88)?;
    }
    if matches!(
        result.outcome,
        EffectOutcome::Conflict | EffectOutcome::Indeterminate
    ) {
        let state = commit_process_indeterminate(
            store,
            &prepared,
            result.detail,
            Some(result.post_observation),
        )?;
        hooks.after_commit(store, &operation.case_id);
        report!("effect_state: indeterminate");
        report!("case_generation: {}", state.generation);
        return Ok(ControlledEffectTurnResult {
            status: ControlledEffectTurnStatus::Indeterminate,
            operation_id: Some(operation.operation_id.clone()),
            decision_id: Some(decision.decision_id.clone()),
            review_id: None,
            effect_id: Some(prepared.effect_id),
            receipt_id: None,
            outcome: Some(result.outcome),
        });
    }
    let receipt = build_process_effect_receipt(&prepared, &result);
    commit_process_finalize(store, &prepared, &result)?;
    if hooks.failpoint() == Some("after_terminal_resource_release_commit") {
        hooks.interrupt("after_terminal_resource_release_commit", 89)?;
    }
    report!("kernel_signal: {}", result.kernel_signal);
    report!("kernel_syscall_accepted: {}", result.syscall_accepted);
    report!(
        "observed_process_state: {:?}",
        result.post_observation.state
    );
    report!("effect_receipt_id: {}", receipt.receipt_id);
    report!("effect_outcome: {:?}", result.outcome);
    report!("effect_state: finalized");
    hooks.after_commit(store, &operation.case_id);
    Ok(ControlledEffectTurnResult {
        status: ControlledEffectTurnStatus::Finalized,
        operation_id: Some(operation.operation_id.clone()),
        decision_id: Some(decision.decision_id.clone()),
        review_id: None,
        effect_id: Some(prepared.effect_id),
        receipt_id: Some(receipt.receipt_id),
        outcome: Some(result.outcome),
    })
}

use yai_core_engine::resource_control::{ResourceFence, ResourceFenceAuthority};
use yai_core_engine::effect::{classify_reconciliation, execute_filesystem_write, ReconciliationConclusion};
pub struct StoredEffectChain {
    pub operation: Operation,
    pub decision: Decision,
    pub grant: ExecutionGrant,
    pub prepared: PreparedEffect,
}

pub fn load_effect_chain(
    transitions: &[Transition],
    effect_id: &str,
) -> Result<StoredEffectChain, String> {
    let prepared = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::EffectPrepared { prepared } if prepared.effect_id == effect_id => {
                Some(prepared.clone())
            }
            _ => None,
        })
        .ok_or_else(|| format!("effect PREPARE not found: {effect_id}"))?;
    let operation = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::OperationRecorded { operation }
                if operation.operation_id == prepared.operation_id =>
            {
                Some(operation.clone())
            }
            _ => None,
        })
        .ok_or_else(|| "prepared effect operation missing".to_string())?;
    let decision = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::DecisionRecorded { decision }
                if decision.decision_id == prepared.decision_id =>
            {
                Some(decision.clone())
            }
            _ => None,
        })
        .ok_or_else(|| "prepared effect decision missing".to_string())?;
    let grant = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ExecutionGrantIssued { grant }
                if grant.grant_id == prepared.grant_id =>
            {
                Some(grant.clone())
            }
            _ => None,
        })
        .ok_or_else(|| "prepared effect grant missing".to_string())?;
    Ok(StoredEffectChain {
        operation,
        decision,
        grant,
        prepared,
    })
}

fn reconciliation_result(
    chain: &StoredEffectChain,
    observation: FilesystemObservation,
    conclusion: &ReconciliationConclusion,
) -> CarrierResult {
    CarrierResult {
        outcome: match conclusion {
            ReconciliationConclusion::EffectObserved => EffectOutcome::AlreadyApplied,
            ReconciliationConclusion::NoEffectObserved => EffectOutcome::NoEffect,
            ReconciliationConclusion::Conflict => EffectOutcome::Conflict,
            ReconciliationConclusion::StillIndeterminate => EffectOutcome::Indeterminate,
        },
        post_observation: observation,
        carrier_attempted: false,
        mutation_performed: false,
        crash_injected_after_effect: false,
        detail: format!("reconciled prepared effect {}", chain.prepared.effect_id),
    }
}

fn effect_fence_for_current_process(
    store: &LmdbRecordStore,
    prepared: &PreparedEffect,
) -> Result<Option<ResourceFence>, String> {
    let Some(original) = prepared.resource_fence.as_ref() else {
        return Ok(None);
    };
    let state = store
        .get_resource_control_state(&original.resource_id)?
        .ok_or_else(|| "resource_control_state_missing_for_effect".to_string())?;
    let current = state
        .active_lease
        .as_ref()
        .ok_or_else(|| "unresolved_effect_resource_lease_missing".to_string())?
        .fence
        .clone();
    if current.effect_id != prepared.effect_id
        || current.case_id != prepared.case_id
        || current.grant_id != prepared.grant_id
    {
        return Err("unresolved_effect_resource_owned_by_other_authority".to_string());
    }
    if store.validate_carrier_fence(&current).is_ok() {
        Ok(Some(current))
    } else {
        store
            .reclaim_resource_for_effect(&current, std::process::id())
            .map(Some)
    }
}

pub fn reconcile_controlled_effect(
    store: &LmdbRecordStore,
    authenticated: &AuthenticatedPrincipal,
    case_id: &str,
    requested_effect: Option<&str>,
    retry: bool,
    hooks: &mut dyn ControlledEffectHooks,
) -> Result<(), String> {
    macro_rules! report { ($($arg:tt)*) => { hooks.report(format!($($arg)*)) }; }
    let state = store
        .get_case_state(case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
    store.get_case_state_authorized(authenticated, case_id)?;
    let effect_state = if let Some(effect_id) = requested_effect {
        state
            .effects
            .iter()
            .find(|effect| effect.effect_id == effect_id)
    } else {
        state.effects.iter().find(|effect| {
            matches!(
                effect.status,
                EffectLifecycle::Prepared | EffectLifecycle::Indeterminate
            )
        })
    }
    .ok_or_else(|| "no matching prepared or finalized effect".to_string())?;
    let transitions = store.list_case_transitions(case_id)?;
    let participant = transitions.iter().find_map(|t| match &t.payload {
        TransitionPayload::OperationRecorded { operation } if operation.operation_id == effect_state.operation_id =>
            Some(operation.participant_id.as_str()),
        _ => None,
    }).ok_or("execution_not_visible")?;
    store.controlled_operation_authorized(authenticated, case_id,
        participant, &effect_state.operation_id)?;
    if effect_state.status == EffectLifecycle::Finalized {
        report!("reconciliation: already_finalized");
        report!("effect_id: {}", effect_state.effect_id);
        report!(
            "receipt_id: {}",
            effect_state.receipt_id.as_deref().unwrap_or("none")
        );
        return Ok(());
    }
    if effect_state.kind == OperationKind::ProcessSignal {
        let prepared = transitions
            .iter()
            .find_map(|transition| match &transition.payload {
                TransitionPayload::ProcessEffectPrepared { prepared }
                    if prepared.effect_id == effect_state.effect_id =>
                {
                    Some(prepared.clone())
                }
                _ => None,
            })
            .ok_or_else(|| "prepared process effect missing".to_string())?;
        let observation = store.observe_process_authorized(
            authenticated,
            case_id,
            &prepared.operation_id,
            &prepared.grant_id,
            &format!("observation:{}:reconcile", prepared.effect_id),
        )?;
        let posture = process_signal_retry_posture(&prepared.action);
        if effect_state.status == EffectLifecycle::Prepared {
            commit_process_indeterminate(
                store,
                &prepared,
                format!(
                    "process_signal_uncertain_after_prepare: retry_posture={posture:?}; syscall_not_repeated"
                ),
                Some(observation.clone()),
            )?;
        }
        report!("reconciliation: StillIndeterminate");
        report!("process_recovery_mode: observation_only");
        report!("process_retry_posture: {posture:?}");
        report!("process_signal_repeated: false");
        report!("observed_process_state: {:?}", observation.state);
        report!(
            "process_observation_error: {}",
            observation.error.as_deref().unwrap_or("none")
        );
        report!("effect_id: {}", prepared.effect_id);
        return Ok(());
    }
    let chain = load_effect_chain(&transitions, &effect_state.effect_id)?;
    let resource = resource_for_case(&state, &chain.prepared.resource_attachment_id)?;
    let binding = store
        .get_local_filesystem_binding(case_id, &resource.attachment_id)?
        .ok_or_else(|| "local filesystem binding unavailable for reconciliation".to_string())?;
    let observation = store.observe_filesystem_authorized(
        authenticated,
        case_id,
        &chain.operation.operation_id,
        &chain.grant.grant_id,
        &format!("observation:{}:reconcile", chain.prepared.effect_id),
    )?;
    let mut conclusion = classify_reconciliation(&chain.prepared, &observation);

    if conclusion == ReconciliationConclusion::NoEffectObserved
        && retry
        && effect_state.status == EffectLifecycle::Prepared
    {
        let fence = effect_fence_for_current_process(store, &chain.prepared)?;
        let result = if let Some(fence) = &fence {
            execute_fenced_filesystem_write(
                store,
                fence,
                &chain.operation,
                &chain.decision,
                &chain.grant,
                &chain.prepared,
                &state,
                &binding,
                &resource,
                CarrierFailpoint::None,
            )?
        } else {
            execute_filesystem_write(
                &chain.operation,
                &chain.decision,
                &chain.grant,
                &chain.prepared,
                &state,
                &binding,
                &resource,
                CarrierFailpoint::None,
            )?
        };
        conclusion = if matches!(
            result.outcome,
            EffectOutcome::Applied | EffectOutcome::AlreadyApplied
        ) {
            ReconciliationConclusion::EffectObserved
        } else if matches!(
            result.outcome,
            EffectOutcome::FailedNoEffect | EffectOutcome::NoEffect
        ) {
            ReconciliationConclusion::NoEffectObserved
        } else if result.outcome == EffectOutcome::Conflict {
            ReconciliationConclusion::Conflict
        } else {
            ReconciliationConclusion::StillIndeterminate
        };
        return commit_reconciliation(
            store,
            hooks,
            &chain,
            conclusion,
            result.post_observation.clone(),
            Some(result),
        );
    }
    commit_reconciliation(store, hooks, &chain, conclusion, observation, None)
}

fn commit_reconciliation(
    store: &LmdbRecordStore,
    hooks: &mut dyn ControlledEffectHooks,
    chain: &StoredEffectChain,
    conclusion: ReconciliationConclusion,
    observation: FilesystemObservation,
    carrier_result: Option<CarrierResult>,
) -> Result<(), String> {
    macro_rules! report { ($($arg:tt)*) => { hooks.report(format!($($arg)*)) }; }
    let receipt = if matches!(
        conclusion,
        ReconciliationConclusion::EffectObserved | ReconciliationConclusion::NoEffectObserved
    ) {
        let result = carrier_result
            .unwrap_or_else(|| reconciliation_result(chain, observation.clone(), &conclusion));
        Some(build_effect_receipt(&chain.prepared, &result))
    } else {
        None
    };
    let refs = vec![chain.prepared.effect_id.clone()];
    let pending = build_effect_pending(
        store,
        &chain.prepared.case_id,
        Some(&chain.prepared.participant_id),
        &format!("reconcile:{}", chain.prepared.effect_id),
        TransitionPayload::EffectReconciled {
            effect_id: chain.prepared.effect_id.clone(),
            conclusion: conclusion.clone(),
            observation,
            receipt,
        },
        None,
        refs,
    )?;
    let terminal = matches!(
        conclusion,
        ReconciliationConclusion::EffectObserved | ReconciliationConclusion::NoEffectObserved
    );
    let state = if terminal {
        if let Some(fence) = effect_fence_for_current_process(store, &chain.prepared)? {
            store.commit_fenced_effect_terminal(pending, &fence)?.state
        } else {
            store.commit_transition(pending)?.state
        }
    } else {
        store.commit_transition(pending)?.state
    };
    report!("reconciliation: {:?}", conclusion);
    report!("effect_id: {}", chain.prepared.effect_id);
    report!(
        "effect_state: {:?}",
        state
            .effects
            .iter()
            .find(|effect| effect.effect_id == chain.prepared.effect_id)
            .map(|effect| &effect.status)
    );
    Ok(())
}
