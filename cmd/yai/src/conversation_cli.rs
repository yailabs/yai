//! Registry-backed reference client for Case conversation content.

use super::*;
use serde::Serialize;
use yai_core_engine::conversation::{
    find_turn, turns_from_history, ContentDerivation, ContentDerivationKind, ContentModality,
    ContentPartProvenance, ConversationContentStore, ConversationDraft, ConversationTurn,
    DerivationActorKind, CONTENT_DERIVATION_SCHEMA, CONVERSATION_DRAFT_SCHEMA,
};

#[derive(Serialize)]
struct DraftView<'a> {
    schema: &'static str,
    draft: &'a ConversationDraft,
    canonical: bool,
    adoption_boundary: &'static str,
}

#[derive(Serialize)]
struct TurnView<'a> {
    schema: &'static str,
    turn: &'a ConversationTurn,
    canonical: bool,
    provider_execution_required_for_identity: bool,
    content_integrity: &'static str,
}

#[derive(Serialize)]
struct TurnListView<'a> {
    schema: &'static str,
    case_id: &'a str,
    participant_id: &'a str,
    multipart_turns: Vec<&'a ConversationTurn>,
    legacy_text_turn_ids: Vec<&'a str>,
}

pub(super) fn conversation_case_command(operation: &str, args: &[String]) -> Result<(), String> {
    match operation {
        "yai.case.conversation.draft.create" => create_draft(args),
        "yai.case.conversation.draft.add_text" => add_text(args),
        "yai.case.conversation.draft.import" => import_content(args),
        "yai.case.conversation.draft.derive_text" => derive_text(args),
        "yai.case.conversation.draft.show" => show_draft(args),
        "yai.case.conversation.draft.discard" => discard_draft(args),
        "yai.case.conversation.draft.send" => send_draft(args),
        "yai.case.conversation.turn.list" => list_turns(args),
        "yai.case.conversation.turn.show" => show_turn(args),
        _ => Err(format!(
            "conversation_registry_handler_not_resolved:{operation}"
        )),
    }
}

fn create_draft(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let draft_id = named_arg(args, "--draft")?;
    let participant_id = named_arg(args, "--participant")?;
    let (store, state, authenticated, principal_id, tenant_id) =
        authorized_case(&case_id, &participant_id)?;
    store
        .resolve_security_context(&authenticated, &tenant_id)?
        .require_owner()?;
    let thread_id = optional_arg(args, "--thread").unwrap_or_else(|| format!("thread:{case_id}"));
    let draft = ConversationDraft {
        schema: CONVERSATION_DRAFT_SCHEMA.to_string(),
        draft_id,
        case_id,
        tenant_id,
        thread_id,
        participant_id,
        principal_id,
        base_generation: state.generation,
        parts: Vec::new(),
    };
    content_store()?.create_draft(&draft)?;
    render(
        args,
        &DraftView {
            schema: "yai.conversation_draft_view.v1",
            draft: &draft,
            canonical: false,
            adoption_boundary: "draft_send",
        },
        || {
            println!("draft_id: {}", draft.draft_id);
            println!("case_id: {}", draft.case_id);
            println!("participant_id: {}", draft.participant_id);
            println!("parts: 0");
            println!("canonical: no");
            println!("next: yai case conversation draft add-text/import/derive-text/send");
        },
    )
}

fn add_text(args: &[String]) -> Result<(), String> {
    let mut draft = load_authorized_draft(args)?;
    let text = named_arg(args, "--text")?;
    let principal_id = draft.principal_id.clone();
    let ordinal = content_store()?.stage_bytes(
        &mut draft,
        ContentModality::Text,
        "text/plain;charset=utf-8",
        text.as_bytes(),
        ContentPartProvenance::Original {
            imported_by_principal_id: principal_id,
        },
    )?;
    render_draft_mutation(args, &draft, ordinal, "original_text")
}

fn import_content(args: &[String]) -> Result<(), String> {
    let mut draft = load_authorized_draft(args)?;
    let source = PathBuf::from(named_arg(args, "--source")?);
    let modality = ContentModality::parse(&named_arg(args, "--type")?)?;
    if modality == ContentModality::Text {
        return Err("conversation_text_import_use_add_text".to_string());
    }
    let media_type = named_arg(args, "--mime")?;
    let principal_id = draft.principal_id.clone();
    let ordinal = content_store()?.import_file(
        &mut draft,
        modality,
        &media_type,
        &source,
        ContentPartProvenance::Original {
            imported_by_principal_id: principal_id,
        },
    )?;
    render_draft_mutation(args, &draft, ordinal, "original_import")
}

fn derive_text(args: &[String]) -> Result<(), String> {
    let mut draft = load_authorized_draft(args)?;
    let source_ordinal = named_arg(args, "--source-part")?
        .parse::<usize>()
        .map_err(|_| "--source-part must be a zero-based integer".to_string())?;
    if source_ordinal >= draft.parts.len() {
        return Err("conversation_derived_source_part_not_found".to_string());
    }
    let published = content_store()?.preview_draft(&draft)?;
    let source_part_id = published[source_ordinal].part_id.clone();
    let kind = ContentDerivationKind::parse(&named_arg(args, "--kind")?)?;
    let text = named_arg(args, "--text")?;
    let (actor_kind, actor_ref, provider_result_id) = if kind == ContentDerivationKind::HumanEdit {
        if optional_arg(args, "--producer-ref").is_some()
            || optional_arg(args, "--provider-result").is_some()
        {
            return Err("conversation_human_edit_actor_is_authenticated_principal".to_string());
        }
        (DerivationActorKind::Human, draft.principal_id.clone(), None)
    } else if let Some(result_id) = optional_arg(args, "--provider-result") {
        verify_provider_result(&draft.case_id, &draft.participant_id, &result_id)?;
        (
            DerivationActorKind::Provider,
            optional_arg(args, "--producer-ref").unwrap_or_else(|| result_id.clone()),
            Some(result_id),
        )
    } else {
        (
            DerivationActorKind::Deterministic,
            named_arg(args, "--producer-ref")?,
            None,
        )
    };
    let mut derivation = ContentDerivation {
        schema: CONTENT_DERIVATION_SCHEMA.to_string(),
        derivation_id: String::new(),
        case_id: draft.case_id.clone(),
        kind,
        source_part_ids: vec![source_part_id],
        actor_kind,
        actor_ref,
        provider_result_id,
    };
    let identity = serde_json::to_vec(&(
        CONTENT_DERIVATION_SCHEMA,
        &derivation.case_id,
        &derivation.kind,
        &derivation.source_part_ids,
        &derivation.actor_kind,
        &derivation.actor_ref,
        &derivation.provider_result_id,
    ))
    .map_err(|error| format!("conversation_derivation_encode_failed: {error}"))?;
    derivation.derivation_id = format!(
        "content-derivation:{}",
        yai_core_engine::effect::digest_bytes(&identity)
    );
    let ordinal = content_store()?.stage_bytes(
        &mut draft,
        ContentModality::Text,
        "text/plain;charset=utf-8",
        text.as_bytes(),
        ContentPartProvenance::Derived { derivation },
    )?;
    render_draft_mutation(args, &draft, ordinal, "derived_text")
}

fn show_draft(args: &[String]) -> Result<(), String> {
    let draft = load_authorized_draft(args)?;
    render(
        args,
        &DraftView {
            schema: "yai.conversation_draft_view.v1",
            draft: &draft,
            canonical: false,
            adoption_boundary: "draft_send",
        },
        || print_draft(&draft),
    )
}

fn discard_draft(args: &[String]) -> Result<(), String> {
    let draft = load_authorized_draft(args)?;
    content_store()?.discard_draft(&draft.case_id, &draft.draft_id)?;
    #[derive(Serialize)]
    struct ResultView<'a> {
        schema: &'static str,
        draft_id: &'a str,
        discarded: bool,
        canonical_case_mutated: bool,
    }
    let value = ResultView {
        schema: "yai.conversation_draft_discard.v1",
        draft_id: &draft.draft_id,
        discarded: true,
        canonical_case_mutated: false,
    };
    render(args, &value, || {
        println!("draft_id: {}", draft.draft_id);
        println!("discarded: yes");
        println!("canonical_case_mutated: no");
    })
}

fn send_draft(args: &[String]) -> Result<(), String> {
    let draft = load_authorized_draft(args)?;
    let commit = super::conversation_controller::commit_conversation_draft(&draft)?;
    #[derive(Serialize)]
    struct SendView<'a> {
        schema: &'static str,
        turn: &'a ConversationTurn,
        transition_id: &'a str,
        generation: u64,
        provider_execution_started: bool,
        draft_discarded: bool,
    }
    let value = SendView {
        schema: "yai.conversation_send_result.v1",
        turn: &commit.turn,
        transition_id: &commit.transition_id,
        generation: commit.generation,
        provider_execution_started: false,
        draft_discarded: commit.draft_discarded,
    };
    render(args, &value, || {
        println!("turn_id: {}", commit.turn.turn_id);
        println!("transition_id: {}", commit.transition_id);
        println!("case_generation: {}", commit.generation);
        println!("ordered_parts: {}", commit.turn.ordered_parts.len());
        println!("canonical: yes");
        println!("provider_execution_started: no");
        println!(
            "draft_discarded: {}",
            if commit.draft_discarded { "yes" } else { "no" }
        );
    })
}

fn list_turns(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let (store, _, _, _, _) = authorized_case(&case_id, &participant_id)?;
    let transitions = store.list_case_transitions(&case_id)?;
    let multipart_turns = turns_from_history(&case_id, &transitions)
        .into_iter()
        .filter(|turn| turn.participant_id == participant_id)
        .collect::<Vec<_>>();
    for turn in &multipart_turns {
        verify_turn(turn)?;
    }
    let legacy_text_turn_ids = transitions
        .iter()
        .filter_map(|transition| match &transition.payload {
            TransitionPayload::InteractionTurnRecorded {
                turn_id,
                participant_id: owner,
                ..
            } if owner == &participant_id => Some(turn_id.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let value = TurnListView {
        schema: "yai.conversation_turn_list.v1",
        case_id: &case_id,
        participant_id: &participant_id,
        multipart_turns,
        legacy_text_turn_ids,
    };
    render(args, &value, || {
        println!("case_id: {case_id}");
        println!("participant_id: {participant_id}");
        println!("multipart_turns: {}", value.multipart_turns.len());
        println!("legacy_text_turns: {}", value.legacy_text_turn_ids.len());
        for turn in &value.multipart_turns {
            println!(
                "turn: {} parts={} thread={}",
                turn.turn_id,
                turn.ordered_parts.len(),
                turn.thread_id
            );
        }
        for turn_id in &value.legacy_text_turn_ids {
            println!("legacy_turn: {turn_id}");
        }
    })
}

fn show_turn(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let turn_id = named_arg(args, "--turn")?;
    let participant_id = named_arg(args, "--participant")?;
    let (store, _, _, _, _) = authorized_case(&case_id, &participant_id)?;
    let transitions = store.list_case_transitions(&case_id)?;
    let turn = if turn_id == "latest" {
        turns_from_history(&case_id, &transitions)
            .into_iter()
            .rev()
            .find(|turn| turn.participant_id == participant_id)
    } else {
        find_turn(&case_id, &turn_id, &transitions)
    }
    .ok_or_else(|| "conversation_turn_not_found".to_string())?;
    if turn.participant_id != participant_id {
        return Err("conversation_turn_not_visible".to_string());
    }
    verify_turn(turn)?;
    let value = TurnView {
        schema: "yai.conversation_turn_view.v1",
        turn,
        canonical: true,
        provider_execution_required_for_identity: false,
        content_integrity: "verified",
    };
    render(args, &value, || print_turn(turn))
}

fn load_authorized_draft(args: &[String]) -> Result<ConversationDraft, String> {
    let case_id = named_arg(args, "--case")?;
    let draft_id = named_arg(args, "--draft")?;
    let draft = content_store()?.load_draft(&case_id, &draft_id)?;
    if draft.case_id != case_id {
        return Err("conversation_draft_case_mismatch".to_string());
    }
    let (_, _, _, principal_id, tenant_id) = authorized_case(&case_id, &draft.participant_id)?;
    if draft.principal_id != principal_id || draft.tenant_id != tenant_id {
        return Err("conversation_draft_not_visible".to_string());
    }
    Ok(draft)
}

fn authorized_case(
    case_id: &str,
    participant_id: &str,
) -> Result<
    (
        LmdbRecordStore,
        yai_core_engine::transition::CaseState,
        yai_core_engine::security::AuthenticatedPrincipal,
        String,
        String,
    ),
    String,
> {
    let authorized =
        super::conversation_controller::authorized_conversation_case(case_id, participant_id)?;
    Ok((
        authorized.store,
        authorized.state,
        authorized.authenticated,
        authorized.principal_id,
        authorized.tenant_id,
    ))
}

fn verify_provider_result(
    case_id: &str,
    participant_id: &str,
    result_id: &str,
) -> Result<(), String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let transitions = store.list_case_transitions(case_id)?;
    let invocation_id = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ProviderResultRecorded {
                result_id: existing,
                invocation_id,
                ..
            } if existing == result_id => Some(invocation_id.as_str()),
            _ => None,
        })
        .ok_or_else(|| "conversation_derivation_provider_result_not_in_case".to_string())?;
    if transitions.iter().any(|transition| {
        matches!(
            &transition.payload,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: existing,
                participant_id: owner,
                ..
            } if existing == invocation_id && owner == participant_id
        )
    }) {
        Ok(())
    } else {
        Err("conversation_derivation_provider_result_not_visible_to_participant".to_string())
    }
}

fn content_store() -> Result<ConversationContentStore, String> {
    super::conversation_controller::conversation_content_store()
}

fn verify_turn(turn: &ConversationTurn) -> Result<(), String> {
    super::conversation_controller::verify_conversation_turn(turn)
}

fn render_draft_mutation(
    args: &[String],
    draft: &ConversationDraft,
    ordinal: usize,
    posture: &str,
) -> Result<(), String> {
    #[derive(Serialize)]
    struct View<'a> {
        schema: &'static str,
        draft_id: &'a str,
        added_part: usize,
        part_count: usize,
        posture: &'a str,
        canonical: bool,
    }
    let value = View {
        schema: "yai.conversation_draft_mutation.v1",
        draft_id: &draft.draft_id,
        added_part: ordinal,
        part_count: draft.parts.len(),
        posture,
        canonical: false,
    };
    render(args, &value, || {
        println!("draft_id: {}", draft.draft_id);
        println!("added_part: {ordinal}");
        println!("part_count: {}", draft.parts.len());
        println!("provenance: {posture}");
        println!("canonical: no");
    })
}

fn print_draft(draft: &ConversationDraft) {
    println!("draft_id: {}", draft.draft_id);
    println!("case_id: {}", draft.case_id);
    println!("participant_id: {}", draft.participant_id);
    println!("parts: {}", draft.parts.len());
    println!("canonical: no");
    for (ordinal, part) in draft.parts.iter().enumerate() {
        println!(
            "part: {ordinal} type={} mime={} bytes={} digest={} provenance={}",
            part.modality.as_str(),
            part.media_type,
            part.byte_length,
            part.content_digest,
            provenance_label(&part.provenance)
        );
    }
}

fn print_turn(turn: &ConversationTurn) {
    println!("schema: {}", turn.schema);
    println!("turn_id: {}", turn.turn_id);
    println!("case_id: {}", turn.case_id);
    println!("thread_id: {}", turn.thread_id);
    println!("participant_id: {}", turn.participant_id);
    println!("ordered_parts: {}", turn.ordered_parts.len());
    println!("canonical: yes");
    println!("provider_execution_required_for_identity: no");
    println!("content_integrity: verified");
    for part in &turn.ordered_parts {
        println!(
            "part: {} id={} type={} mime={} bytes={} digest={} object={} storage={} provenance={}",
            part.ordinal,
            part.part_id,
            part.object.modality.as_str(),
            part.object.media_type,
            part.object.byte_length,
            part.object.content_digest,
            part.object.object_id,
            part.object.storage_ref,
            provenance_label(&part.provenance)
        );
        if let ContentPartProvenance::Derived { derivation } = &part.provenance {
            println!(
                "derivation: {} kind={:?} sources={} actor={:?}:{} provider_result={}",
                derivation.derivation_id,
                derivation.kind,
                derivation.source_part_ids.join(","),
                derivation.actor_kind,
                derivation.actor_ref,
                derivation.provider_result_id.as_deref().unwrap_or("none")
            );
        }
    }
}

fn provenance_label(value: &ContentPartProvenance) -> &'static str {
    match value {
        ContentPartProvenance::Original { .. } => "original",
        ContentPartProvenance::Derived { derivation }
            if derivation.kind == ContentDerivationKind::HumanEdit =>
        {
            "human_edited_derived"
        }
        ContentPartProvenance::Derived { .. } => "machine_or_deterministic_derived",
    }
}

fn render<T: Serialize>(args: &[String], value: &T, human: impl FnOnce()) -> Result<(), String> {
    if args.iter().any(|arg| arg == "--json") {
        println!(
            "{}",
            serde_json::to_string(value)
                .map_err(|error| format!("conversation_json_encode_failed: {error}"))?
        );
    } else {
        human();
    }
    Ok(())
}
