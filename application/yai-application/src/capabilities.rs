//! Static product-surface metadata for current YAI capabilities.
//!
//! This catalog classifies product meaning; it does not own domain semantics,
//! authorization, persistence, or CLI syntax. Application operation descriptors
//! are the stable client identities used by the local dispatcher. A capability
//! may remain explicitly deferred at the application boundary, but it may not
//! remain unclassified.

use serde::Serialize;
use std::collections::BTreeSet;

pub const CAPABILITY_CATALOG_SCHEMA: &str = "yai.application_capability_catalog.v1";
pub const APPLICATION_OPERATION_SCHEMA: &str = "yai.application_operation.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDisposition {
    ProductRead,
    ProductAction,
    OperatorDiagnostic,
    InternalMechanic,
    TargetOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StateImpact {
    Read,
    DerivedComputation,
    Proposal,
    CanonicalMutation,
    /// Changes execution control, not canonical Case truth or an external effect.
    OperationalMutation,
    ExternalEffect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationPosture {
    Ready,
    Deferred,
    NotApplicable,
}

/// A missing semantic contract, rather than historical presentation debt.
/// These blockers are deliberately narrow: code placement or an absent
/// wrapper is never a valid blocker.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationBlockerClass {
    DisconnectSafeExecutionLifecycle,
    RuntimeSupervisionLifecycle,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ApplicationCapabilityBlocker {
    pub capability_id: &'static str,
    pub class: ApplicationBlockerClass,
    pub missing_contract: &'static str,
    pub existing_partial_operation_ids: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CliPosture {
    Exposed,
    NotUseful,
    DeferredWithReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StudioPosture {
    ApplicationReady,
    UiAlreadyConsumed,
    UiTarget,
    NotUiRelevant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityClass {
    LocalAuthenticated,
    CaseDisclosure,
    CurrentCaseAuthority,
    ReviewGrantAndEffectFence,
    None,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ApplicationOperationDescriptor {
    pub schema: &'static str,
    pub operation_id: &'static str,
    pub meaning: &'static str,
    pub input_contract: &'static str,
    pub output_contract: &'static str,
    pub impact: StateImpact,
    pub authority: AuthorityClass,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ProductCapabilityDescriptor {
    pub capability_id: &'static str,
    pub meaning: &'static str,
    pub engine_owner: &'static str,
    pub roadmap_refs: &'static [&'static str],
    pub disposition: CapabilityDisposition,
    pub impact: StateImpact,
    pub authority: AuthorityClass,
    pub application_posture: ApplicationPosture,
    pub application_operation_ids: &'static [&'static str],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_deferred_reason: Option<&'static str>,
    pub cli_posture: CliPosture,
    pub cli_operation_ids: &'static [&'static str],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_reason: Option<&'static str>,
    pub studio_posture: StudioPosture,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_capability_id: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ApplicationCapabilityCatalog {
    pub schema: &'static str,
    pub application_protocol: &'static str,
    pub capabilities: &'static [ProductCapabilityDescriptor],
    pub operations: &'static [ApplicationOperationDescriptor],
    pub blockers: &'static [ApplicationCapabilityBlocker],
}

const fn operation(
    operation_id: &'static str,
    meaning: &'static str,
    input_contract: &'static str,
    output_contract: &'static str,
    impact: StateImpact,
    authority: AuthorityClass,
) -> ApplicationOperationDescriptor {
    ApplicationOperationDescriptor {
        schema: APPLICATION_OPERATION_SCHEMA,
        operation_id,
        meaning,
        input_contract,
        output_contract,
        impact,
        authority,
    }
}

pub static APPLICATION_OPERATIONS: &[ApplicationOperationDescriptor] = &[
    operation("application.capabilities", "Discover stable product capabilities and application operations", "yai.application_capabilities_input.v1", CAPABILITY_CATALOG_SCHEMA, StateImpact::Read, AuthorityClass::LocalAuthenticated),
    operation("case.cancel", "Cancel further Case advancement and terminalize still-usable review/grant authority", "yai.case_terminal_input.v1", "yai.case_cancellation_outcome.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("case.capabilities", "Inspect current Participant-scoped Resource requestability without granting execution authority", "yai.case_capabilities_input.v1", "yai.case_capability_view.v1", StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority),
    operation("case.close", "Close a terminal Case non-destructively", "yai.case_terminal_input.v1", "yai.case_closure_outcome.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("case.create", "Create one Tenant-scoped governed Case", "yai.case_create_input.v1", "yai.canonical_commit.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("case.list", "List Cases visible to the authenticated local Principal", "yai.case_list_input.v1", "yai.case_list_projection.v1", StateImpact::Read, AuthorityClass::LocalAuthenticated),
    operation("case.open", "Attach an authenticated local client to an existing visible Case", "yai.case_open_input.v1", "yai.case_attachment.v1", StateImpact::Read, AuthorityClass::CaseDisclosure),
    operation("case.recent", "List recently updated visible Cases", "yai.case_list_input.v1", "yai.case_list_projection.v1", StateImpact::Read, AuthorityClass::LocalAuthenticated),
    operation("case.run", "Durably submit exact bounded work to the existing runtime scheduler with an explicit idempotency identity", "yai.case_run_input.v1", "yai.execution_submission_result.v1", StateImpact::ExternalEffect, AuthorityClass::CurrentCaseAuthority),
    operation("case.stop", "Request a cooperative stop of one exact admitted runner; does not cancel or close its Case", "yai.case_stop_input.v1", "yai.runtime_execution_observation.v1", StateImpact::OperationalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("case.summary", "Project the current authorized Case workspace", "yai.case_summary_input.v1", "yai.case_workspace_projection.v1", StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure),
    operation("cognitive.binding.set", "Bind one exact ordered qualified target set to a Case cognitive role", "yai.cognitive_bind_input.v1", "yai.case_cognitive_binding.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("cognitive.plan", "Plan one current qualified cognitive execution without invoking a provider", "yai.cognitive_plan_input.v1", "yai.cognitive_execution_plan.v1", StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority),
    operation("decision.frontier.prepare", "Derive the current disclosed finite cognitive candidate frontier from an exact W/task", "yai.decision_frontier_prepare_input.v1", "yai.cognitive_decision_frontier_qualification.v1", StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority),
    operation("decision.request.prepare", "Prepare a non-authoritative CognitiveDecisionRequest from a current exact frontier", "yai.decision_request_prepare_input.v1", "yai.cognitive_decision_request.v1", StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority),
    operation("events.heartbeat", "Report the current generation cursor for one visible Case", "yai.events_heartbeat_input.v1", "yai.case_event_heartbeat.v1", StateImpact::Read, AuthorityClass::CaseDisclosure),
    operation("events.resume", "Describe local event resynchronization from a client cursor", "yai.events_resume_input.v1", "yai.events_subscription.v1", StateImpact::Read, AuthorityClass::LocalAuthenticated),
    operation("events.subscribe", "Describe local event subscription and snapshot posture", "yai.events_subscribe_input.v1", "yai.events_subscription.v1", StateImpact::Read, AuthorityClass::LocalAuthenticated),
    operation("execution.get", "Observe an exact durable domain submission after client loss without dispatching work", "yai.execution_get_input.v1", "yai.execution_observation.v1", StateImpact::Read, AuthorityClass::CurrentCaseAuthority),
    operation("handoff.accept", "Accept one visible same-Tenant Handoff as an eligible target Participant", "yai.handoff_accept_input.v1", "yai.canonical_commit.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("handoff.decline", "Decline one visible same-Tenant Handoff without transferring authority", "yai.handoff_decline_input.v1", "yai.canonical_commit.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("handoff.offer", "Offer one bounded data item and role requirement set to a same-Tenant Case", "yai.handoff_offer_input.v1", "yai.canonical_commit.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("handoff.reconcile", "Reconcile one terminal target Handoff result into source-local truth", "yai.handoff_reconcile_input.v1", "yai.canonical_commit.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("handoff.result.record", "Record one bounded terminal result in the target Case", "yai.handoff_result_input.v1", "yai.canonical_commit.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("identity.bootstrap", "Enroll the kernel-authenticated local Principal and initialize one Tenant", "yai.identity_bootstrap_input.v1", "yai.security_bootstrap_outcome.v1", StateImpact::CanonicalMutation, AuthorityClass::LocalAuthenticated),
    operation("identity.current", "Inspect the kernel-authenticated Principal and its Tenant relations", "yai.identity_current_input.v1", "yai.identity_projection.v1", StateImpact::Read, AuthorityClass::LocalAuthenticated),
    operation("knowledge.inspect", "Reconstruct the currently disclosed source-grounded Knowledge view including exact graph relations", "yai.knowledge_inspect_input.v1", "yai.source_knowledge.v1", StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure),
    operation("knowledge.navigation", "Derive documentary navigation from the current qualified Knowledge view", "yai.knowledge_inspect_input.v1", "yai.knowledge_navigation_result.v1", StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure),
    operation("knowledge.resolve", "Resolve one exact unit only inside current disclosed Knowledge with source closure", "yai.knowledge_resolve_input.v1", "yai.knowledge_resolve_result.v1", StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure),
    operation("knowledge.search", "Search current qualified Knowledge with bounded lexical discovery, not confidence scoring", "yai.knowledge_search_input.v1", "yai.knowledge_search_result.v1", StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure),
    operation("material.read", "Read exact retained source material after current Case/source qualification", "yai.material_read_input.v1", "yai.material_read_projection.v1", StateImpact::Read, AuthorityClass::CurrentCaseAuthority),
    operation("participant.principal.link", "Link one enrolled Principal to one Case Participant under Tenant-owner authority", "yai.participant_principal_link_input.v1", "yai.case_state_mutation_result.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("participant.role.add", "Bind one exact role to a Case Participant", "yai.participant_role_add_input.v1", "yai.case_state_mutation_result.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("participant.view.admit", "Admit one supported disclosure view for a bound Case Participant", "yai.participant_view_admit_input.v1", "yai.case_state_mutation_result.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("policy.case.bind", "Bind one exact currently published PolicyArtifact to a current Case", "yai.case_policy_bind_input.v1", "yai.case_policy_mutation_outcome.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("policy.case.replace", "Replace one current Case Policy binding with a published artifact from the same lineage", "yai.case_policy_replace_input.v1", "yai.case_policy_mutation_outcome.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("policy.case.unbind", "Remove one exact current Case Policy binding", "yai.case_policy_unbind_input.v1", "yai.case_policy_mutation_outcome.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("policy.ingest", "Compile and ingest immutable Tenant-scoped Policy source bytes as a candidate artifact", "yai.policy_ingest_input.v1", "yai.policy_ingest_outcome.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("policy.publish", "Publish one qualified PolicyArtifact", "yai.policy_artifact_lifecycle_input.v1", "yai.policy_lifecycle_outcome.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("policy.retire", "Retire one PolicyArtifact from catalog use", "yai.policy_artifact_lifecycle_input.v1", "yai.policy_lifecycle_outcome.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("policy.revoke", "Withdraw one published, superseded or retired PolicyArtifact from authority", "yai.policy_artifact_lifecycle_input.v1", "yai.policy_lifecycle_outcome.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("policy.validate", "Validate one qualified candidate PolicyArtifact", "yai.policy_artifact_lifecycle_input.v1", "yai.policy_lifecycle_outcome.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("provider.case.bind", "Bind one ordered governed provider target envelope to a Case Participant", "yai.provider_case_bind_input.v1", "yai.case_provider_binding.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("provider.qualify", "Persist exact mechanical provider qualification evidence", "yai.provider_qualify_input.v1", "yai.provider_qualification.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("provider.register", "Register one immutable generic provider target without secrets", "yai.provider_register_input.v1", "yai.provider_target.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("provider.trust.set", "Set current owner-authenticated provider trust posture", "yai.provider_trust_input.v1", "yai.provider_trust_event.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("resource.attach", "Attach one exact typed local Resource binding and immutable access envelope", "yai.resource_attach_input.v1", "yai.case_state_mutation_result.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("resource.attach_process", "Capture and attach one exact live process identity with explicit signal and review bounds", "yai.resource_attach_process_input.v1", "yai.case_state_mutation_result.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("resource.request", "Admit an exact participant Resource request and advance through existing governed carriers; observe the same submission after reconnect", "yai.resource_request_input.v1", "yai.resource_submission_result.v1", StateImpact::ExternalEffect, AuthorityClass::ReviewGrantAndEffectFence),
    operation("review.approve", "Record an eligible authenticated approval and derive its current effective Decision", "yai.review_resolve_input.v1", "yai.review_resolve_result.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("review.defer", "Record an eligible authenticated deferral without creating an execution Grant", "yai.review_resolve_input.v1", "yai.review_resolve_result.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("review.deny", "Record an eligible authenticated denial and derive its current effective Decision", "yai.review_resolve_input.v1", "yai.review_resolve_result.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("runtime.readiness", "Inspect local application-host readiness", "yai.runtime_readiness_input.v1", "yai.application_status.v1", StateImpact::Read, AuthorityClass::LocalAuthenticated),
    operation("semantic.ambient_refresh.assess", "Requalify one active semantic consumer after admitted changes without model execution", "yai.ambient_refresh_assess_input.v1", "yai.ambient_refresh_result.v1", StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority),
    operation("semantic.recall", "Reconstruct bounded task-qualified D/H/S evidence through Recall v2", "yai.recall_execute_input.v1", "yai.recall_result.v2", StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure),
    operation("semantic.working_state.compile", "Compile current Recall-aware W3 or exact-group W4 state", "yai.working_state_compile_input.v1", "yai.qualified_working_state.v1", StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority),
    operation("semantic.working_state.page", "Requalify and expand or evict exact W4 semantic groups", "yai.working_state_page_input.v1", "yai.semantic_page_result.v1", StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority),
    operation("semantic.working_state.refresh", "Recompile the same stored task against current authority and Recall", "yai.working_state_refresh_input.v1", "yai.working_refresh_result.v1", StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority),
    operation("source.acquire", "Submit one exact durable source acquisition attempt; duplicate submissions observe without redispatch", "yai.source_acquire_input.v1", "yai.source_acquisition_submission_result.v1", StateImpact::ExternalEffect, AuthorityClass::CurrentCaseAuthority),
    operation("source.declare", "Declare one Case-local source relation over an attached typed Resource", "yai.source_declare_input.v1", "yai.case_state_mutation_result.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("source.publish", "Publish a qualified bootstrap-policy source through the existing Policy lifecycle", "yai.source_publish_input.v1", "yai.source_policy_publication_result.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("source.resume", "Resume one exact settled acquisition interruption once; never blindly redispatch a still-acquiring attempt", "yai.source_resume_input.v1", "yai.source_acquisition_submission_result.v1", StateImpact::ExternalEffect, AuthorityClass::CurrentCaseAuthority),
    operation("source.revoke", "Revoke one Case-local source relation without deleting retained physical backing", "yai.source_revoke_input.v1", "yai.case_state_mutation_result.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("system.status", "Inspect local Application protocol and runtime posture", "yai.system_status_input.v1", "yai.application_status.v1", StateImpact::Read, AuthorityClass::LocalAuthenticated),
    operation("tenant.get", "Inspect one Tenant visible to the authenticated Principal", "yai.tenant_get_input.v1", "yai.tenant_projection.v1", StateImpact::Read, AuthorityClass::LocalAuthenticated),
    operation("tenant.list", "List Tenant relations visible to the authenticated Principal", "yai.tenant_list_input.v1", "yai.principal_tenant_relation_collection.v1", StateImpact::Read, AuthorityClass::LocalAuthenticated),
    operation("tenant.member.add", "Add one enrolled Principal to a Tenant as Owner-authorized membership", "yai.tenant_member_add_input.v1", "yai.security_event.v1", StateImpact::CanonicalMutation, AuthorityClass::LocalAuthenticated),
    operation("workflow.bind", "Bind one exact Workflow definition and explicit slots to a Case", "yai.workflow_bind_input.v1", "yai.canonical_commit.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("workflow.define", "Admit one immutable typed Workflow definition", "yai.workflow_define_input.v1", "yai.workflow_definition.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("workflow.input.record", "Record bounded HumanInput for a ready Workflow node", "yai.workflow_input_record_input.v1", "yai.canonical_commit.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("workflow.patch.adopt", "Adopt one valid Case-local Workflow PlanPatch", "yai.workflow_patch_adopt_input.v1", "yai.canonical_commit.v1", StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority),
    operation("workflow.patch.propose", "Propose one bounded human-authored Workflow PlanPatch", "yai.workflow_patch_propose_input.v1", "yai.canonical_commit.v1", StateImpact::Proposal, AuthorityClass::CurrentCaseAuthority),
];

const fn capability(
    capability_id: &'static str,
    meaning: &'static str,
    engine_owner: &'static str,
    roadmap_refs: &'static [&'static str],
    disposition: CapabilityDisposition,
    impact: StateImpact,
    authority: AuthorityClass,
    application_posture: ApplicationPosture,
    application_operation_ids: &'static [&'static str],
    application_deferred_reason: Option<&'static str>,
    cli_posture: CliPosture,
    cli_operation_ids: &'static [&'static str],
    cli_reason: Option<&'static str>,
    studio_posture: StudioPosture,
    parent_capability_id: Option<&'static str>,
    rationale: Option<&'static str>,
) -> ProductCapabilityDescriptor {
    ProductCapabilityDescriptor {
        capability_id,
        meaning,
        engine_owner,
        roadmap_refs,
        disposition,
        impact,
        authority,
        application_posture,
        application_operation_ids,
        application_deferred_reason,
        cli_posture,
        cli_operation_ids,
        cli_reason,
        studio_posture,
        parent_capability_id,
        rationale,
    }
}

const CASE_RUNTIME_BLOCKED: Option<&str> = Some("Create/open/cancel/close and durable run submission/observation are typed Application operations. Normal Host startup supervises the existing RuntimeInstance; active-work recovery and explicit Application resume/stop are still under qualification.");
const EXECUTION_LIFECYCLE_BLOCKED: Option<&str> = Some("Only the explicitly listed partial operations are available. Provider/resource dispatch still needs qualified durable submission and reconnect-safe result observation. This blocker record does not substitute for lifecycle tests or prove that existing domain attempts cannot be reused.");

pub static APPLICATION_BLOCKERS: &[ApplicationCapabilityBlocker] = &[
    ApplicationCapabilityBlocker {
        capability_id: "case.lifecycle",
        class: ApplicationBlockerClass::RuntimeSupervisionLifecycle,
        missing_contract: "Resident Host ownership of RuntimeInstance work submission, checkpoint observation, stop and resume without transferring canonical Case ownership.",
        existing_partial_operation_ids: &["case.cancel", "case.close", "case.create", "case.open", "case.run", "case.stop", "execution.get"],
    },
    ApplicationCapabilityBlocker {
        capability_id: "cognitive.bindings_and_realization",
        class: ApplicationBlockerClass::DisconnectSafeExecutionLifecycle,
        missing_contract: "Stable execution identity and result observation for provider realization/compose across client disconnect and Host restart.",
        existing_partial_operation_ids: &["cognitive.binding.set", "cognitive.plan"],
    },
    ApplicationCapabilityBlocker {
        capability_id: "conversation.execution",
        class: ApplicationBlockerClass::DisconnectSafeExecutionLifecycle,
        missing_contract: "Typed SEND submission and observation contract that distinguishes committed Turn, provider dispatch, indeterminate delivery and reconnect-safe result retrieval.",
        existing_partial_operation_ids: &[],
    },
    ApplicationCapabilityBlocker {
        capability_id: "effect.controlled_execution",
        class: ApplicationBlockerClass::DisconnectSafeExecutionLifecycle,
        missing_contract: "Typed effect submission/result observation contract that preserves PREPARE, no-redispatch and reconciliation across Application transport loss.",
        existing_partial_operation_ids: &["execution.get", "resource.request"],
    },
    ApplicationCapabilityBlocker {
        capability_id: "source.lifecycle",
        class: ApplicationBlockerClass::DisconnectSafeExecutionLifecycle,
        missing_contract: "Exact acquire/resume identity, lost-response retry, real review and in-flight crash observation are qualified. Unresolved progress cannot yet distinguish a live carrier from carrier loss or reconcile its terminal outcome; automatic redispatch remains forbidden.",
        existing_partial_operation_ids: &["execution.get", "source.acquire", "source.declare", "source.publish", "source.resume", "source.revoke"],
    },
];

pub static CAPABILITIES: &[ProductCapabilityDescriptor] = &[
    capability("authority.inspect", "Inspect current policy bindings, Reviews, Grants and last governed Decision", "CaseState + EffectivePolicy/admission", &["A02", "A03"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure, ApplicationPosture::Ready, &["case.summary"], None, CliPosture::Exposed, &["yai.case.policy.show", "yai.review.pending", "yai.review.show"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("authority.policy.lifecycle", "Ingest, validate, publish, bind, replace, retire and revoke Policy through normal governance", "governance + case_policy", &["A01", "A02"], CapabilityDisposition::ProductAction, StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["policy.case.bind", "policy.case.replace", "policy.case.unbind", "policy.ingest", "policy.publish", "policy.retire", "policy.revoke", "policy.validate"], None, CliPosture::Exposed, &["yai.policy.ingest", "yai.policy.validate", "yai.policy.publish", "yai.case.policy.bind", "yai.case.policy.replace", "yai.case.policy.unbind"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("authority.review_and_grant", "Resolve authenticated Reviews and finite Grants without bypassing admission", "admission + Transition review lifecycle", &["A03", "A05"], CapabilityDisposition::ProductAction, StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["review.approve", "review.defer", "review.deny"], None, CliPosture::Exposed, &["yai.review.approve", "yai.review.deny", "yai.review.defer"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("case.catalog", "List and attach visible Cases", "LMDB authorized Case readers", &["K01", "X03"], CapabilityDisposition::ProductRead, StateImpact::Read, AuthorityClass::LocalAuthenticated, ApplicationPosture::Ready, &["case.list", "case.recent", "case.open"], None, CliPosture::Exposed, &["yai.case.list", "yai.case.open"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("case.history_and_experience", "Inspect canonical history, historical cuts and exact experience relations", "Transition ledger + experience resolver", &["K01", "M03", "M06"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure, ApplicationPosture::Ready, &["case.summary"], None, CliPosture::Exposed, &["yai.case.history", "yai.case.as_of", "yai.case.experience"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("case.lifecycle", "Create, open, run, stop, cancel and close governed Cases", "Case transition/controller owners", &["K03", "X01"], CapabilityDisposition::ProductAction, StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Deferred, &["case.cancel", "case.close", "case.create", "case.open", "case.run", "case.stop", "execution.get"], CASE_RUNTIME_BLOCKED, CliPosture::Exposed, &["yai.case.create", "yai.case.run", "yai.case.resume", "yai.case.stop", "yai.case.cancel", "yai.case.close"], None, StudioPosture::UiTarget, None, Some("Studio consumes create/open/cancel/close and exact bounded run/stop/observation; general resume and complete recovery remain partial.")),
    capability("case.participants", "Bind Participants, roles, Principal links and disclosed Participant views", "Case Participant/security transitions", &["A04"], CapabilityDisposition::ProductAction, StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["case.summary", "participant.principal.link", "participant.role.add", "participant.view.admit"], None, CliPosture::Exposed, &["yai.case.participant.role.add", "yai.case.participant.link_principal", "yai.case.participant.view.admit", "yai.case.participant.list"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("case.resource_requestability", "Inspect current Case/Participant Resource requests that may be proposed; never execution permission", "CaseCapabilityView/admission", &["A03", "O07"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["case.capabilities"], None, CliPosture::Exposed, &["yai.case.capabilities"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("case.workspace", "Project current Case overview, environment, authority, work, compute and conversation", "yai-application projection over engine owners", &["X03", "X04"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure, ApplicationPosture::Ready, &["case.open", "case.summary", "events.heartbeat", "events.subscribe", "events.resume"], None, CliPosture::Exposed, &["yai.case.show"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("cognitive.bindings_and_realization", "Bind cognitive roles, plan qualified providers and realize/compose explicit cognition", "cognitive bindings + provider realization", &["E01", "E03", "E04"], CapabilityDisposition::ProductAction, StateImpact::ExternalEffect, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Deferred, &["cognitive.binding.set", "cognitive.plan"], EXECUTION_LIFECYCLE_BLOCKED, CliPosture::Exposed, &["yai.case.cognitive.bind", "yai.case.cognitive.plan", "yai.case.cognitive.realize", "yai.case.cognitive.compose"], None, StudioPosture::UiTarget, None, None),
    capability("cognitive.decision_frontier", "Construct and inspect the current finite typed Decision Frontier without scoring", "cognitive frontier + LMDB current qualification", &["E07"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["decision.frontier.prepare"], None, CliPosture::Exposed, &["yai.case.cognitive.frontier"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("cognitive.decision_readout", "Score an exact DecisionRequest with a qualified production producer", "future YVEX Decision Readout/Core", &["E07"], CapabilityDisposition::TargetOnly, StateImpact::DerivedComputation, AuthorityClass::None, ApplicationPosture::NotApplicable, &[], None, CliPosture::DeferredWithReason, &[], Some("No production scorer or public producer contract exists."), StudioPosture::NotUiRelevant, None, Some("The deterministic scorer remains test-only.")),
    capability("cognitive.decision_request", "Prepare and inspect a W-bound non-authoritative CognitiveDecisionRequest", "Cognitive Decision Plane", &["E07"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["decision.request.prepare"], None, CliPosture::Exposed, &["yai.case.cognitive.decision_request"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("cognitive.hot_path_preparation", "Compose W requalification, Frontier and DecisionRequest inside one operation-scoped read basis", "LMDB cognitive preparation", &["E07"], CapabilityDisposition::InternalMechanic, StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::NotApplicable, &[], None, CliPosture::NotUseful, &[], Some("The transient witness and measurements are not product authority."), StudioPosture::NotUiRelevant, Some("cognitive.decision_frontier"), Some("Serves Decision Frontier/Request preparation without becoming a transferable capability token.")),
    capability("conversation.execution", "Draft/send Turns and execute ordinary Recall-aware Conversation work", "ConversationController + invocation lifecycle", &["C06", "E03"], CapabilityDisposition::ProductAction, StateImpact::ExternalEffect, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Deferred, &[], EXECUTION_LIFECYCLE_BLOCKED, CliPosture::Exposed, &["yai.case.conversation.draft.send", "yai.case.workbench"], None, StudioPosture::UiTarget, None, None),
    capability("conversation.inspect", "Inspect authorized retained Conversation turns", "Turn/Conversation projection", &["K02", "X03"], CapabilityDisposition::ProductRead, StateImpact::Read, AuthorityClass::CaseDisclosure, ApplicationPosture::Ready, &["case.summary"], None, CliPosture::Exposed, &["yai.case.conversation.turn.list", "yai.case.conversation.turn.show"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("effect.controlled_execution", "Submit governed Resource/effect requests through Decision, review, PREPARE and final fences", "effect admission + carrier", &["A03", "A05", "O07"], CapabilityDisposition::ProductAction, StateImpact::ExternalEffect, AuthorityClass::ReviewGrantAndEffectFence, ApplicationPosture::Deferred, &["execution.get", "resource.request"], EXECUTION_LIFECYCLE_BLOCKED, CliPosture::Exposed, &["yai.case.resource.request", "yai.effect.filesystem_write", "yai.effect.process_signal", "yai.effect.reconcile"], None, StudioPosture::UiTarget, None, Some("Studio authors seven bound Resource request variants and observes exact receipts; other effect families and interrupted PREPARE reconciliation remain partial.")),
    capability("handoff.lifecycle", "Offer, accept, decline, result and reconcile explicit same-Tenant Handoffs", "Handoff transitions", &["W02"], CapabilityDisposition::ProductAction, StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["handoff.accept", "handoff.decline", "handoff.offer", "handoff.reconcile", "handoff.result.record"], None, CliPosture::Exposed, &["yai.case.handoff.offer", "yai.case.handoff.accept", "yai.case.handoff.decline", "yai.case.handoff.result", "yai.case.handoff.reconcile"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("identity.tenant", "Initialize local identity and inspect Tenant membership", "security bootstrap + Tenant owner", &["A04", "X01"], CapabilityDisposition::ProductAction, StateImpact::CanonicalMutation, AuthorityClass::LocalAuthenticated, ApplicationPosture::Ready, &["identity.bootstrap", "identity.current", "tenant.get", "tenant.list", "tenant.member.add"], None, CliPosture::Exposed, &["yai.init", "yai.identity.whoami", "yai.tenant.list", "yai.tenant.show", "yai.tenant.member.add"], None, StudioPosture::ApplicationReady, None, None),
    capability("knowledge.derived", "Build, inspect, search, resolve and navigate source-grounded documentary knowledge", "memory_hierarchy::knowledge", &["M07"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure, ApplicationPosture::Ready, &["case.summary", "knowledge.inspect", "knowledge.navigation", "knowledge.resolve", "knowledge.search"], None, CliPosture::Exposed, &["yai.case.knowledge.build", "yai.case.knowledge.inspect", "yai.case.knowledge.search", "yai.case.knowledge.resolve", "yai.case.knowledge.graph", "yai.case.knowledge.wiki"], None, StudioPosture::UiAlreadyConsumed, None, Some("Studio consumes summary plus owner inspect/search/resolve/navigation; retained execution-actions product tests qualify exact content and current Source revocation.")),
    capability("knowledge.derived_indexes", "Materialize and rebuild graph/BM25/wiki navigation behind qualified Knowledge/Recall", "derived knowledge/index owners", &["M02", "M03", "M07"], CapabilityDisposition::InternalMechanic, StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure, ApplicationPosture::NotApplicable, &[], None, CliPosture::NotUseful, &[], Some("Index mechanics are observed through Knowledge/Recall and dedicated diagnostics."), StudioPosture::NotUiRelevant, Some("knowledge.derived"), Some("Disposable derived structures have no independent product authority.")),
    capability("material.read", "Resolve exact retained source bytes through current Case/source disclosure", "source resolution + ConversationContentStore", &["O08", "M07"], CapabilityDisposition::ProductRead, StateImpact::Read, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["material.read"], None, CliPosture::Exposed, &["yai.case.sources.read"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("memory.maintenance", "Inspect, verify, rebuild or drop disposable memory/index materializations", "derived memory/index owners", &["M01", "M02"], CapabilityDisposition::OperatorDiagnostic, StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure, ApplicationPosture::Deferred, &[], Some("Operator maintenance remains in exact CLI diagnostics; it is not an ordinary product action."), CliPosture::Exposed, &["yai.case.memory.index.status", "yai.case.memory.index.verify", "yai.case.memory.index.rebuild", "yai.case.memory.hierarchy.rebuild"], None, StudioPosture::NotUiRelevant, None, Some("Maintenance cannot mint canonical truth.")),
    capability("platform.canonical_diagnostics", "Verify replay, store, ledger, projection and engine consistency", "canonical/derived diagnostic owners", &["K01", "Q01"], CapabilityDisposition::OperatorDiagnostic, StateImpact::Read, AuthorityClass::LocalAuthenticated, ApplicationPosture::Deferred, &[], Some("Exact operator diagnostics remain CLI-only and do not belong in ordinary Studio actions."), CliPosture::Exposed, &["yai.case.verify", "yai.store.status", "yai.journal.replay", "yai.projection.inspect", "yai.engine.summary"], None, StudioPosture::NotUiRelevant, None, Some("Diagnostics inspect owners; they do not become a second semantic API.")),
    capability("platform.capability_discovery", "Discover what YAI supports and the exact surface posture of each bounded capability", "yai-application static catalog", &["X03"], CapabilityDisposition::ProductRead, StateImpact::Read, AuthorityClass::LocalAuthenticated, ApplicationPosture::Ready, &["application.capabilities"], None, CliPosture::Exposed, &["yai.application.capabilities"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("platform.local_host", "Start, discover, inspect, restart and stop the resident same-user YAI Application Host", "yai-host local lifecycle and IPC", &["X03", "X04"], CapabilityDisposition::ProductAction, StateImpact::DerivedComputation, AuthorityClass::LocalAuthenticated, ApplicationPosture::NotApplicable, &[], Some("The Host lifecycle is the carrier beneath yai-application and cannot be invoked through the service whose lifecycle it controls."), CliPosture::Exposed, &["yai.host.status", "yai.host.start", "yai.host.stop", "yai.host.restart", "yai.host.logs"], None, StudioPosture::UiAlreadyConsumed, None, Some("Typed Host lifecycle/client libraries are consumed directly by CLI and Tauri; Case authority remains in yai-application.")),
    capability("platform.remote_application_transport", "Expose the same typed Application operations through a qualified remote SDK/RPC transport", "future interfaces transport", &["X03"], CapabilityDisposition::TargetOnly, StateImpact::Read, AuthorityClass::None, ApplicationPosture::NotApplicable, &[], None, CliPosture::DeferredWithReason, &[], Some("Remote authentication and transport are not selected."), StudioPosture::NotUiRelevant, None, Some("This wave defines no HTTP/RPC protocol.")),
    capability("platform.runtime_host", "Serve, inspect and stop bounded RuntimeInstance scheduling", "runtime host/scheduler", &["F01"], CapabilityDisposition::OperatorDiagnostic, StateImpact::CanonicalMutation, AuthorityClass::LocalAuthenticated, ApplicationPosture::Deferred, &[], Some("Runtime administration remains an operator surface rather than a Studio product action."), CliPosture::Exposed, &["yai.runtime.serve", "yai.runtime.status", "yai.runtime.queue", "yai.runtime.stop"], None, StudioPosture::NotUiRelevant, None, Some("Runtime controls do not own Case semantics.")),
    capability("platform.status", "Inspect local YAI readiness, version and application protocol posture", "local runtime/store status + yai-application", &["X01", "X03"], CapabilityDisposition::ProductRead, StateImpact::Read, AuthorityClass::LocalAuthenticated, ApplicationPosture::Ready, &["system.status", "runtime.readiness"], None, CliPosture::Exposed, &["yai.doctor", "yai.meta.version"], None, StudioPosture::ApplicationReady, None, None),
    capability("provider.governance", "Register, qualify, trust and bind generic provider targets", "provider governance", &["E01", "E02"], CapabilityDisposition::ProductAction, StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["provider.case.bind", "provider.qualify", "provider.register", "provider.trust.set"], None, CliPosture::Exposed, &["yai.provider.add", "yai.provider.qualify", "yai.provider.trust.approve", "yai.case.provider.bind"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("provider.inspect", "Inspect Case-bound provider targets and current governed posture", "provider governance projection", &["E01", "X03"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure, ApplicationPosture::Ready, &["case.summary"], None, CliPosture::Exposed, &["yai.provider.list", "yai.provider.show", "yai.case.provider.show"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("provider.projection_lowering", "Lower qualified W into Projection/ContextFrame for a generic provider", "semantic projection + provider request builder", &["C06", "E03"], CapabilityDisposition::InternalMechanic, StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::NotApplicable, &[], None, CliPosture::NotUseful, &[], Some("Clients invoke Conversation/Workflow execution, not the lowering mechanic."), StudioPosture::NotUiRelevant, Some("conversation.execution"), Some("Projection performs no Recall or authority widening.")),
    capability("resource.lifecycle", "Attach and inspect exact governed filesystem/process/other Resources", "ResourceAttachment transitions", &["O01", "O03", "O07"], CapabilityDisposition::ProductAction, StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["case.summary", "resource.attach", "resource.attach_process"], None, CliPosture::Exposed, &["yai.case.resource.attach_filesystem", "yai.case.resource.attach_process", "yai.case.resource.import", "yai.case.resource.list"], None, StudioPosture::ApplicationReady, None, Some("Studio presents exact configuration/attempt facts and authors process attachment; general native Resource setup remains Studio debt. Missing PID is an action refusal, not transport loss.")),
    capability("semantic.adaptive_cognition", "Choose the least expensive qualified cognitive mechanism for a bounded step", "future Minimum Sufficient Cognition controller", &["E07"], CapabilityDisposition::TargetOnly, StateImpact::Proposal, AuthorityClass::None, ApplicationPosture::NotApplicable, &[], None, CliPosture::DeferredWithReason, &[], Some("No adaptive router is implemented."), StudioPosture::NotUiRelevant, None, Some("A production decision producer is prerequisite.")),
    capability("semantic.ambient_refresh", "Assess and refresh an active Conversation/Workflow consumer after admitted semantic change", "AmbientRefresh v1 + existing Recall/W compiler", &["C02", "C06"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["semantic.ambient_refresh.assess"], None, CliPosture::Exposed, &["yai.case.context.ambient"], None, StudioPosture::ApplicationReady, None, None),
    capability("semantic.recall", "Reconstruct task/query-qualified D/H/S evidence with exact closure and disclosure", "Recall v2", &["M06", "Q06"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure, ApplicationPosture::Ready, &["semantic.recall"], None, CliPosture::Exposed, &["yai.case.recall"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("semantic.w_to_e", "Lower YAI W into a public YVEX computational state contract", "future YAI/YVEX public boundary", &["C10", "C11"], CapabilityDisposition::TargetOnly, StateImpact::DerivedComputation, AuthorityClass::None, ApplicationPosture::NotApplicable, &[], None, CliPosture::DeferredWithReason, &[], Some("No public producer/consumer contract exists; I07 is unselected."), StudioPosture::NotUiRelevant, None, Some("W to E, StateProfile and B1 remain unimplemented.")),
    capability("semantic.working_state", "Compile W3, exact-group W4 pages and explicit same-task refresh", "SemanticWorkingState compiler/refresh/paging", &["C02", "C06", "C07"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["semantic.working_state.compile", "semantic.working_state.page", "semantic.working_state.refresh"], None, CliPosture::Exposed, &["yai.case.context.compile", "yai.case.context.refresh", "yai.case.context.expand"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("source.environment", "Inspect declared sources, revisions, exact routes and current derived source/knowledge posture", "Case source relation + acquisition/route derivation", &["O08", "M07"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure, ApplicationPosture::Ready, &["case.summary", "material.read"], None, CliPosture::Exposed, &["yai.case.sources.inventory", "yai.case.sources.routes"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("source.lifecycle", "Declare, acquire, resume, publish and revoke Case-local source relations", "source acquisition + Case source transitions", &["O08"], CapabilityDisposition::ProductAction, StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Deferred, &["execution.get", "source.acquire", "source.declare", "source.publish", "source.resume", "source.revoke"], EXECUTION_LIFECYCLE_BLOCKED, CliPosture::Exposed, &["yai.case.sources.declare", "yai.case.sources.acquire", "yai.case.sources.resume", "yai.case.sources.publish", "yai.case.sources.revoke"], None, StudioPosture::UiTarget, None, Some("Studio consumes declare/acquire/resume/publish/revoke and exact attempt observation; unresolved interrupted delivery remains a backend boundary.")),
    capability("workflow.lifecycle", "Define, bind, advance and amend deterministic Workflow", "Workflow definitions/resolution/transitions", &["W01"], CapabilityDisposition::ProductAction, StateImpact::CanonicalMutation, AuthorityClass::CurrentCaseAuthority, ApplicationPosture::Ready, &["workflow.bind", "workflow.define", "workflow.input.record", "workflow.patch.adopt", "workflow.patch.propose"], None, CliPosture::Exposed, &["yai.workflow.define", "yai.workflow.bind", "yai.workflow.input", "yai.workflow.patch.propose", "yai.workflow.patch.adopt"], None, StudioPosture::UiAlreadyConsumed, None, None),
    capability("workflow.overview", "Inspect exact Workflow definition, topology and current resolution", "Workflow resolver", &["W01", "X03"], CapabilityDisposition::ProductRead, StateImpact::DerivedComputation, AuthorityClass::CaseDisclosure, ApplicationPosture::Ready, &["case.summary"], None, CliPosture::Exposed, &["yai.workflow.status", "yai.workflow.show"], None, StudioPosture::UiAlreadyConsumed, None, None),
];

pub fn capability_catalog() -> ApplicationCapabilityCatalog {
    ApplicationCapabilityCatalog {
        schema: CAPABILITY_CATALOG_SCHEMA,
        application_protocol: crate::APPLICATION_PROTOCOL,
        capabilities: CAPABILITIES,
        operations: APPLICATION_OPERATIONS,
        blockers: APPLICATION_BLOCKERS,
    }
}

pub fn validate_capability_catalog() -> Result<(), String> {
    let mut operation_ids = BTreeSet::new();
    for operation in APPLICATION_OPERATIONS {
        if operation.operation_id.is_empty() || !operation_ids.insert(operation.operation_id) {
            return Err(format!(
                "duplicate_or_empty_application_operation:{}",
                operation.operation_id
            ));
        }
    }
    if APPLICATION_OPERATIONS
        .windows(2)
        .any(|pair| pair[0].operation_id >= pair[1].operation_id)
    {
        return Err("application_operations_not_sorted".to_string());
    }

    let mut capability_ids = BTreeSet::new();
    let blocker_ids = APPLICATION_BLOCKERS
        .iter()
        .map(|blocker| blocker.capability_id)
        .collect::<BTreeSet<_>>();
    if blocker_ids.len() != APPLICATION_BLOCKERS.len()
        || APPLICATION_BLOCKERS
            .windows(2)
            .any(|pair| pair[0].capability_id >= pair[1].capability_id)
    {
        return Err("application_blockers_duplicate_or_unsorted".to_string());
    }
    for capability in CAPABILITIES {
        if capability.capability_id.is_empty() || !capability_ids.insert(capability.capability_id) {
            return Err(format!(
                "duplicate_or_empty_capability:{}",
                capability.capability_id
            ));
        }
        for operation_id in capability.application_operation_ids {
            if !operation_ids.contains(operation_id) {
                return Err(format!(
                    "capability_unknown_application_operation:{}:{}",
                    capability.capability_id, operation_id
                ));
            }
        }
        if matches!(
            capability.disposition,
            CapabilityDisposition::ProductRead | CapabilityDisposition::ProductAction
        ) && capability.application_posture == ApplicationPosture::Ready
            && capability.application_operation_ids.is_empty()
        {
            return Err(format!(
                "ready_product_without_application_operation:{}",
                capability.capability_id
            ));
        }
        if matches!(
            capability.disposition,
            CapabilityDisposition::ProductRead | CapabilityDisposition::ProductAction
        ) && capability.application_posture == ApplicationPosture::Deferred
            && capability.application_deferred_reason.is_none()
        {
            return Err(format!(
                "deferred_product_without_reason:{}",
                capability.capability_id
            ));
        }
        if matches!(
            capability.disposition,
            CapabilityDisposition::ProductRead | CapabilityDisposition::ProductAction
        ) && (capability.application_posture == ApplicationPosture::Deferred)
            != blocker_ids.contains(capability.capability_id)
        {
            return Err(format!(
                "product_application_blocker_mismatch:{}",
                capability.capability_id
            ));
        }
        if capability.disposition == CapabilityDisposition::InternalMechanic
            && (capability.parent_capability_id.is_none() || capability.rationale.is_none())
        {
            return Err(format!(
                "internal_mechanic_without_parent_or_reason:{}",
                capability.capability_id
            ));
        }
        if capability.disposition == CapabilityDisposition::TargetOnly
            && (!capability.application_operation_ids.is_empty()
                || !capability.cli_operation_ids.is_empty())
        {
            return Err(format!(
                "target_only_presented_as_executable:{}",
                capability.capability_id
            ));
        }
    }
    for blocker in APPLICATION_BLOCKERS {
        let capability = CAPABILITIES
            .iter()
            .find(|capability| capability.capability_id == blocker.capability_id)
            .ok_or_else(|| {
                format!(
                    "application_blocker_unknown_capability:{}",
                    blocker.capability_id
                )
            })?;
        if blocker.missing_contract.trim().is_empty()
            || blocker.existing_partial_operation_ids != capability.application_operation_ids
        {
            return Err(format!(
                "application_blocker_contract_invalid:{}",
                blocker.capability_id
            ));
        }
    }
    if CAPABILITIES
        .windows(2)
        .any(|pair| pair[0].capability_id >= pair[1].capability_id)
    {
        return Err("capabilities_not_sorted".to_string());
    }
    for capability in CAPABILITIES
        .iter()
        .filter(|entry| entry.disposition == CapabilityDisposition::InternalMechanic)
    {
        if !capability_ids.contains(capability.parent_capability_id.expect("validated parent")) {
            return Err(format!(
                "internal_mechanic_unknown_parent:{}",
                capability.capability_id
            ));
        }
    }
    Ok(())
}

fn join_or_dash(values: &[&str]) -> String {
    if values.is_empty() {
        "—".to_string()
    } else {
        values.join("<br>")
    }
}

pub fn render_capability_matrix() -> String {
    let count = |disposition| {
        CAPABILITIES
            .iter()
            .filter(|item| item.disposition == disposition)
            .count()
    };
    let application_ready = CAPABILITIES
        .iter()
        .filter(|item| item.application_posture == ApplicationPosture::Ready)
        .count();
    let cli_exposed = CAPABILITIES
        .iter()
        .filter(|item| item.cli_posture == CliPosture::Exposed)
        .count();
    let studio_consumable = CAPABILITIES
        .iter()
        .filter(|item| {
            matches!(
                item.studio_posture,
                StudioPosture::ApplicationReady | StudioPosture::UiAlreadyConsumed
            )
        })
        .count();
    let mut output = format!(
        "# YAI application capability matrix\n\n\
Generated from `application/yai-application/src/capabilities.rs`. Edit the code-owned catalog, not this matrix.\n\n\
Inventory: **{} capabilities** — {} product reads, {} product actions, {} operator diagnostics, {} internal mechanics and {} targets. **{}** are Application-ready, **{}** CLI-exposed and **{}** Studio-consumable through the typed Application boundary.\n\n\
| Capability | Disposition / impact | Engine owner | Application | CLI | Studio |\n\
|---|---|---|---|---|---|\n",
        CAPABILITIES.len(),
        count(CapabilityDisposition::ProductRead),
        count(CapabilityDisposition::ProductAction),
        count(CapabilityDisposition::OperatorDiagnostic),
        count(CapabilityDisposition::InternalMechanic),
        count(CapabilityDisposition::TargetOnly),
        application_ready,
        cli_exposed,
        studio_consumable,
    );
    for item in CAPABILITIES {
        output.push_str(&format!(
            "| `{}`<br>{} | `{:?}` / `{:?}` | {} | `{:?}`<br>{}{} | `{:?}`<br>{}{} | `{:?}` |\n",
            item.capability_id,
            item.meaning,
            item.disposition,
            item.impact,
            item.engine_owner,
            item.application_posture,
            join_or_dash(item.application_operation_ids),
            item.application_deferred_reason
                .map(|reason| format!("<br>{reason}"))
                .unwrap_or_default(),
            item.cli_posture,
            join_or_dash(item.cli_operation_ids),
            item.cli_reason
                .map(|reason| format!("<br>{reason}"))
                .unwrap_or_default(),
            item.studio_posture,
        ));
    }
    output.push_str("\nThe catalog describes support and surface posture. `case.capabilities` separately derives current Participant/Case Resource requestability; neither catalog nor view grants execution authority.\n");
    output.push_str("\n## Current product Application blockers\n\n");
    output.push_str("These are missing semantic execution contracts, not missing wrappers. Partial operation IDs remain usable where listed.\n\n");
    for blocker in APPLICATION_BLOCKERS {
        output.push_str(&format!(
            "- `{}` — `{:?}`: {} Partial operations: {}.\n",
            blocker.capability_id,
            blocker.class,
            blocker.missing_contract,
            join_or_dash(blocker.existing_partial_operation_ids).replace("<br>", ", ")
        ));
    }
    output
}
