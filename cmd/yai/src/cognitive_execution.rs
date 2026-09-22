//! CLI adaptation to the shared typed cognitive execution owner.
use super::*;
use yai_core_engine::cognitive::{CognitiveCapability, CognitivePlanRoute, LaneContinuationReference};
use yai_core_engine::conversation::{ConversationContentStore, ConversationTurn, CognitiveCompositionRequest};
use yai_core_engine::security::AuthenticatedPrincipal;
pub(super) use yai_application::cognitive_execution::{source_parts, source_identity,
    realization_performed_now, RecordedExecution, CognitiveRealizationOutcome, CognitiveCompositionOutcome};

#[allow(clippy::too_many_arguments)]
pub(super) fn realize_cognitive(
    authenticated: &AuthenticatedPrincipal,
    store: &LmdbRecordStore,
    content_store: &ConversationContentStore,
    turn: &yai_core_engine::conversation::ConversationTurn,
    participant_id: &str,
    capability: CognitiveCapability,
    wire_parts: Vec<provider::ProviderWireInputPart>,
    derivation_source_part_ids: Vec<String>,
    requirement_source_ref: &str,
    expected_route: Option<CognitivePlanRoute>,
    continuation: Option<&LaneContinuationReference>,
    failpoint: Option<&str>,
    realization_causal_refs: &[String],
    cancelled: &dyn Fn() -> bool,
    output_contract: InvocationOutputContract,
) -> Result<CognitiveRealizationOutcome, String> {
    yai_application::cognitive_execution::realize_cognitive(
        &yai_home(), &provider::env_var, authenticated, store, content_store, turn,
        participant_id, capability, wire_parts, derivation_source_part_ids,
        requirement_source_ref, expected_route, continuation, failpoint,
        realization_causal_refs, cancelled, output_contract, None,
    )
}

pub(super) fn execute_composition(
    authenticated: &AuthenticatedPrincipal,
    store: &LmdbRecordStore,
    content_store: &ConversationContentStore,
    turn: &ConversationTurn,
    request: &CognitiveCompositionRequest,
    cancelled: &dyn Fn() -> bool,
    failpoint: Option<&str>,
) -> Result<CognitiveCompositionOutcome, String> {
    yai_application::cognitive_execution::execute_composition(
        &yai_home(), &provider::env_var, authenticated, store, content_store, turn, request, cancelled, failpoint,
    )
}
