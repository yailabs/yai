//! Optional System-1 memory navigation above already qualified Recall/W.
//! No model runtime is linked into YAI; the public readout producer is absent.
use serde::{Deserialize, Serialize};
use yai_core_engine::cognitive::{FastSearchNavigation, FastSearchPreparationPosture};
use yai_core_engine::conversation::ConversationContentStore;
use yai_core_engine::security::AuthenticatedPrincipal;
use yai_core_engine::semantic_state::SemanticWorkingState;
use yai_core_engine::store::lmdb::LmdbRecordStore;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FastSearchPrepareInput {
    pub working_state: SemanticWorkingState,
    pub max_candidates: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FastSearchAvailability {
    UnavailableProducer,
    UnavailableNoChoices,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FastSearchPrepareResult {
    pub schema: String,
    pub navigation: FastSearchNavigation,
    pub availability: FastSearchAvailability,
    pub active: bool,
    pub actual_path: String,
}

pub fn prepare(
    home: &std::path::Path,
    auth: &AuthenticatedPrincipal,
    store: &LmdbRecordStore,
    input: FastSearchPrepareInput,
) -> Result<FastSearchPrepareResult, String> {
    let content = ConversationContentStore::open_existing(home).ok();
    store.validate_working_state_authorized(auth, &input.working_state, content.as_ref())?;
    let navigation = FastSearchNavigation::derive(&input.working_state, input.max_candidates)?;
    let availability = match navigation.posture {
        FastSearchPreparationPosture::ReadyForOptionalProducer =>
            FastSearchAvailability::UnavailableProducer,
        FastSearchPreparationPosture::DeterministicFallbackNoChoice =>
            FastSearchAvailability::UnavailableNoChoices,
    };
    Ok(FastSearchPrepareResult {
        schema: "yai.fast_search_prepare_result.v1".into(),
        navigation,
        availability,
        active: false,
        actual_path: "qualified_deterministic_recall_w".into(),
    })
}
