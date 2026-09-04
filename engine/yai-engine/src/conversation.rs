//! Case-owned multipart conversation content.
//!
//! A committed [`ConversationTurn`] is canonical Case history. Original bytes
//! are immutable application content owned by YAI and live outside LMDB; a
//! transition contains integrity-bound references, never bulk media. Drafts
//! are mutable staging state and acquire no Case authority until SEND commits
//! a turn. Content derivations retain their sources instead of replacing them.

use crate::effect::digest_bytes;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[cfg(target_os = "linux")]
use std::ffi::{CStr, CString};
use std::fs::{self, File};
use std::io::{Read, Write};
#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, FromRawFd};
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicU64, Ordering};

pub const CONTENT_OBJECT_SCHEMA: &str = "yai.conversation_content_object.v1";
pub const CONTENT_PART_SCHEMA: &str = "yai.conversation_content_part.v1";
pub const CONTENT_DERIVATION_SCHEMA: &str = "yai.content_derivation.v1";
pub const CONVERSATION_TURN_SCHEMA: &str = "yai.conversation_turn.v1";
pub const CONVERSATION_DRAFT_SCHEMA: &str = "yai.conversation_draft.v1";
pub const CONTENT_STORE_SCHEMA: &str = "yai.conversation_content_store.v1";

pub const MAX_CONTENT_OBJECT_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_TURN_CONTENT_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_TURN_PARTS: usize = 32;
pub const MAX_TEXT_BYTES: usize = 64 * 1024;
pub const MAX_DRAFT_JSON_BYTES: u64 = 512 * 1024;
pub const MAX_DERIVATION_SOURCES: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentModality {
    Text,
    Image,
    Audio,
    Video,
    File,
}

impl ContentModality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::File => "file",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "text" => Ok(Self::Text),
            "image" => Ok(Self::Image),
            "audio" => Ok(Self::Audio),
            "video" => Ok(Self::Video),
            "file" => Ok(Self::File),
            _ => Err("conversation_content_modality_invalid".to_string()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentDerivationKind {
    SpeechTranscription,
    HumanEdit,
    OpticalCharacterRecognition,
    ImageCaption,
    AudioExtraction,
    SampledFrames,
    DocumentTextExtraction,
    GeneratedContent,
}

impl ContentDerivationKind {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "speech-transcription" => Ok(Self::SpeechTranscription),
            "human-edit" => Ok(Self::HumanEdit),
            "ocr" => Ok(Self::OpticalCharacterRecognition),
            "image-caption" => Ok(Self::ImageCaption),
            "audio-extraction" => Ok(Self::AudioExtraction),
            "sampled-frames" => Ok(Self::SampledFrames),
            "document-text-extraction" => Ok(Self::DocumentTextExtraction),
            "generated-content" => Ok(Self::GeneratedContent),
            _ => Err("conversation_content_derivation_kind_invalid".to_string()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DerivationActorKind {
    Provider,
    Human,
    Deterministic,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContentDerivation {
    pub schema: String,
    pub derivation_id: String,
    pub case_id: String,
    pub kind: ContentDerivationKind,
    pub source_part_ids: Vec<String>,
    pub actor_kind: DerivationActorKind,
    pub actor_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_result_id: Option<String>,
}

impl ContentDerivation {
    fn identity_digest(&self) -> Result<String, String> {
        digest_json(&(
            CONTENT_DERIVATION_SCHEMA,
            &self.case_id,
            &self.kind,
            &self.source_part_ids,
            &self.actor_kind,
            &self.actor_ref,
            &self.provider_result_id,
        ))
    }

    pub fn validate(
        &self,
        case_id: &str,
        admitted_sources: &BTreeSet<String>,
    ) -> Result<(), String> {
        if self.schema != CONTENT_DERIVATION_SCHEMA || self.case_id != case_id {
            return Err("conversation_content_derivation_scope_invalid".to_string());
        }
        require_bounded("derivation.actor_ref", &self.actor_ref, 512)?;
        if self.source_part_ids.is_empty() || self.source_part_ids.len() > MAX_DERIVATION_SOURCES {
            return Err("conversation_content_derivation_source_bound_invalid".to_string());
        }
        let mut unique = BTreeSet::new();
        for source in &self.source_part_ids {
            require_bounded("derivation.source_part_id", source, 256)?;
            if !admitted_sources.contains(source) || !unique.insert(source) {
                return Err("conversation_content_derivation_source_not_admitted".to_string());
            }
        }
        match (&self.kind, &self.actor_kind, &self.provider_result_id) {
            (ContentDerivationKind::HumanEdit, DerivationActorKind::Human, None) => {
                validate_scope_id("derivation.actor_ref", &self.actor_ref, "principal:")?
            }
            (ContentDerivationKind::HumanEdit, _, _) => {
                return Err("conversation_human_edit_actor_invalid".to_string())
            }
            (_, DerivationActorKind::Human, _) => {
                return Err("conversation_human_derivation_kind_invalid".to_string())
            }
            (_, DerivationActorKind::Provider, Some(result_id)) => {
                require_bounded("derivation.provider_result_id", result_id, 256)?;
                if !result_id.starts_with("provider-result:") {
                    return Err("conversation_provider_derivation_result_invalid".to_string());
                }
            }
            (_, DerivationActorKind::Provider, None) => {
                return Err("conversation_provider_derivation_result_required".to_string())
            }
            (_, DerivationActorKind::Deterministic, Some(_)) => {
                return Err("conversation_deterministic_derivation_result_forbidden".to_string())
            }
            (_, DerivationActorKind::Deterministic, None) => {}
        }
        if self.derivation_id != format!("content-derivation:{}", self.identity_digest()?) {
            return Err("conversation_content_derivation_identity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "provenance_kind", rename_all = "snake_case")]
pub enum ContentPartProvenance {
    Original { imported_by_principal_id: String },
    Derived { derivation: ContentDerivation },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConversationContentObject {
    pub schema: String,
    pub object_id: String,
    pub tenant_id: String,
    pub case_id: String,
    pub modality: ContentModality,
    pub media_type: String,
    pub byte_length: u64,
    pub content_digest: String,
    pub storage_ref: String,
    /// Bounded UTF-8 application text is carried in the canonical reference so
    /// pure Projection can preserve conversation semantics. Binary media is
    /// never inlined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_text: Option<String>,
}

impl ConversationContentObject {
    pub fn new(
        tenant_id: &str,
        case_id: &str,
        modality: ContentModality,
        media_type: &str,
        bytes: &[u8],
    ) -> Result<Self, String> {
        validate_scope_id("tenant_id", tenant_id, "tenant:")?;
        validate_scope_id("case_id", case_id, "case:")?;
        validate_media_type(&modality, media_type)?;
        if bytes.is_empty() || bytes.len() as u64 > MAX_CONTENT_OBJECT_BYTES {
            return Err("conversation_content_object_size_invalid".to_string());
        }
        if modality == ContentModality::Text
            && (bytes.len() > MAX_TEXT_BYTES || std::str::from_utf8(bytes).is_err())
        {
            return Err("conversation_text_content_invalid".to_string());
        }
        let content_digest = digest_bytes(bytes);
        let object_digest = digest_json(&(
            CONTENT_OBJECT_SCHEMA,
            tenant_id,
            case_id,
            &modality,
            media_type,
            bytes.len() as u64,
            &content_digest,
        ))?;
        let object_token = digest_token(&object_digest)?;
        let object_id = format!("content-object:{object_token}");
        let inline_text = (modality == ContentModality::Text)
            .then(|| String::from_utf8(bytes.to_vec()).expect("text validated as UTF-8"));
        Ok(Self {
            schema: CONTENT_OBJECT_SCHEMA.to_string(),
            storage_ref: format!("yai-content/v1/objects/{object_token}/payload"),
            object_id,
            tenant_id: tenant_id.to_string(),
            case_id: case_id.to_string(),
            modality,
            media_type: media_type.to_string(),
            byte_length: bytes.len() as u64,
            content_digest,
            inline_text,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != CONTENT_OBJECT_SCHEMA {
            return Err("conversation_content_object_schema_invalid".to_string());
        }
        validate_scope_id("tenant_id", &self.tenant_id, "tenant:")?;
        validate_scope_id("case_id", &self.case_id, "case:")?;
        validate_media_type(&self.modality, &self.media_type)?;
        if self.byte_length == 0 || self.byte_length > MAX_CONTENT_OBJECT_BYTES {
            return Err("conversation_content_object_size_invalid".to_string());
        }
        require_digest(&self.content_digest)?;
        match (&self.modality, &self.inline_text) {
            (ContentModality::Text, Some(text))
                if text.len() as u64 == self.byte_length
                    && digest_bytes(text.as_bytes()) == self.content_digest => {}
            (ContentModality::Text, _) => {
                return Err("conversation_text_inline_integrity_mismatch".to_string())
            }
            (_, None) => {}
            (_, Some(_)) => {
                return Err("conversation_binary_content_must_not_be_inlined".to_string())
            }
        }
        let object_digest = digest_json(&(
            CONTENT_OBJECT_SCHEMA,
            &self.tenant_id,
            &self.case_id,
            &self.modality,
            &self.media_type,
            self.byte_length,
            &self.content_digest,
        ))?;
        let object_token = digest_token(&object_digest)?;
        if self.object_id != format!("content-object:{object_token}")
            || self.storage_ref != format!("yai-content/v1/objects/{object_token}/payload")
        {
            return Err("conversation_content_object_identity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConversationContentPart {
    pub schema: String,
    pub part_id: String,
    pub ordinal: u16,
    pub object: ConversationContentObject,
    pub provenance: ContentPartProvenance,
}

impl ConversationContentPart {
    pub fn build(
        ordinal: usize,
        object: ConversationContentObject,
        provenance: ContentPartProvenance,
    ) -> Result<Self, String> {
        if ordinal >= MAX_TURN_PARTS {
            return Err("conversation_content_part_ordinal_invalid".to_string());
        }
        let ordinal =
            u16::try_from(ordinal).map_err(|_| "conversation_content_part_ordinal_invalid")?;
        let part_digest =
            digest_json(&(CONTENT_PART_SCHEMA, ordinal, &object.object_id, &provenance))?;
        Ok(Self {
            schema: CONTENT_PART_SCHEMA.to_string(),
            part_id: format!("content-part:{part_digest}"),
            ordinal,
            object,
            provenance,
        })
    }

    fn validate(
        &self,
        case_id: &str,
        ordinal: usize,
        sources: &BTreeSet<String>,
    ) -> Result<(), String> {
        if self.schema != CONTENT_PART_SCHEMA || usize::from(self.ordinal) != ordinal {
            return Err("conversation_content_part_order_invalid".to_string());
        }
        self.object.validate()?;
        if self.object.case_id != case_id {
            return Err("conversation_content_part_case_mismatch".to_string());
        }
        if let ContentPartProvenance::Derived { derivation } = &self.provenance {
            derivation.validate(case_id, sources)?;
        } else if let ContentPartProvenance::Original {
            imported_by_principal_id,
        } = &self.provenance
        {
            validate_scope_id(
                "imported_by_principal_id",
                imported_by_principal_id,
                "principal:",
            )?;
        }
        let part_digest = digest_json(&(
            CONTENT_PART_SCHEMA,
            self.ordinal,
            &self.object.object_id,
            &self.provenance,
        ))?;
        if self.part_id != format!("content-part:{part_digest}") {
            return Err("conversation_content_part_identity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub schema: String,
    pub turn_id: String,
    pub case_id: String,
    pub tenant_id: String,
    pub thread_id: String,
    pub participant_id: String,
    pub submitted_by_principal_id: String,
    pub base_generation: u64,
    pub ordered_parts: Vec<ConversationContentPart>,
    pub content_digest: String,
}

impl ConversationTurn {
    pub fn build(
        case_id: &str,
        tenant_id: &str,
        thread_id: &str,
        participant_id: &str,
        submitted_by_principal_id: &str,
        base_generation: u64,
        mut parts: Vec<ConversationContentPart>,
    ) -> Result<Self, String> {
        for (ordinal, part) in parts.iter_mut().enumerate() {
            part.ordinal =
                u16::try_from(ordinal).map_err(|_| "conversation_content_part_ordinal_invalid")?;
            *part = ConversationContentPart::build(
                ordinal,
                part.object.clone(),
                part.provenance.clone(),
            )?;
        }
        let content_digest = digest_json(&parts)?;
        let turn_digest = digest_json(&(
            CONVERSATION_TURN_SCHEMA,
            case_id,
            tenant_id,
            thread_id,
            participant_id,
            submitted_by_principal_id,
            base_generation,
            &content_digest,
        ))?;
        let value = Self {
            schema: CONVERSATION_TURN_SCHEMA.to_string(),
            turn_id: format!("conversation-turn:{turn_digest}"),
            case_id: case_id.to_string(),
            tenant_id: tenant_id.to_string(),
            thread_id: thread_id.to_string(),
            participant_id: participant_id.to_string(),
            submitted_by_principal_id: submitted_by_principal_id.to_string(),
            base_generation,
            ordered_parts: parts,
            content_digest,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != CONVERSATION_TURN_SCHEMA
            || self.ordered_parts.is_empty()
            || self.ordered_parts.len() > MAX_TURN_PARTS
        {
            return Err("conversation_turn_contract_invalid".to_string());
        }
        validate_scope_id("case_id", &self.case_id, "case:")?;
        validate_scope_id("tenant_id", &self.tenant_id, "tenant:")?;
        validate_scope_id("participant_id", &self.participant_id, "participant:")?;
        validate_scope_id(
            "submitted_by_principal_id",
            &self.submitted_by_principal_id,
            "principal:",
        )?;
        require_bounded("thread_id", &self.thread_id, 256)?;
        let all_sources = self
            .ordered_parts
            .iter()
            .map(|part| part.part_id.clone())
            .collect::<BTreeSet<_>>();
        let mut total = 0u64;
        for (ordinal, part) in self.ordered_parts.iter().enumerate() {
            part.validate(&self.case_id, ordinal, &all_sources)?;
            if part.object.tenant_id != self.tenant_id {
                return Err("conversation_content_part_tenant_mismatch".to_string());
            }
            match &part.provenance {
                ContentPartProvenance::Original {
                    imported_by_principal_id,
                } if imported_by_principal_id != &self.submitted_by_principal_id => {
                    return Err("conversation_original_content_principal_mismatch".to_string());
                }
                ContentPartProvenance::Derived { derivation }
                    if derivation.actor_kind == DerivationActorKind::Human
                        && derivation.actor_ref != self.submitted_by_principal_id =>
                {
                    return Err("conversation_human_derivation_principal_mismatch".to_string());
                }
                _ => {}
            }
            if let ContentPartProvenance::Derived { derivation } = &part.provenance {
                for source in &derivation.source_part_ids {
                    let source_ordinal = self
                        .ordered_parts
                        .iter()
                        .position(|item| &item.part_id == source)
                        .ok_or_else(|| {
                            "conversation_content_derivation_source_not_admitted".to_string()
                        })?;
                    if source_ordinal >= ordinal {
                        return Err(
                            "conversation_content_derivation_source_order_invalid".to_string()
                        );
                    }
                }
            }
            total = total
                .checked_add(part.object.byte_length)
                .ok_or_else(|| "conversation_turn_content_size_overflow".to_string())?;
        }
        if total > MAX_TURN_CONTENT_BYTES {
            return Err("conversation_turn_content_size_exceeded".to_string());
        }
        if self.content_digest != digest_json(&self.ordered_parts)? {
            return Err("conversation_turn_content_digest_mismatch".to_string());
        }
        let turn_digest = digest_json(&(
            CONVERSATION_TURN_SCHEMA,
            &self.case_id,
            &self.tenant_id,
            &self.thread_id,
            &self.participant_id,
            &self.submitted_by_principal_id,
            self.base_generation,
            &self.content_digest,
        ))?;
        if self.turn_id != format!("conversation-turn:{turn_digest}") {
            return Err("conversation_turn_identity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn provider_text_input(&self) -> Result<String, String> {
        self.validate()?;
        let mut texts = Vec::new();
        for part in &self.ordered_parts {
            if part.object.modality != ContentModality::Text {
                return Err("conversation_turn_requires_typed_media_provider_adapter".to_string());
            }
            texts.push(
                part.object
                    .inline_text
                    .clone()
                    .ok_or_else(|| "conversation_text_inline_integrity_mismatch".to_string())?,
            );
        }
        Ok(texts.join("\n\n"))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DraftContentPart {
    pub draft_blob_id: String,
    pub modality: ContentModality,
    pub media_type: String,
    pub byte_length: u64,
    pub content_digest: String,
    pub provenance: ContentPartProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConversationDraft {
    pub schema: String,
    pub draft_id: String,
    pub case_id: String,
    pub tenant_id: String,
    pub thread_id: String,
    pub participant_id: String,
    pub principal_id: String,
    pub base_generation: u64,
    pub parts: Vec<DraftContentPart>,
}

impl ConversationDraft {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != CONVERSATION_DRAFT_SCHEMA || self.parts.len() > MAX_TURN_PARTS {
            return Err("conversation_draft_contract_invalid".to_string());
        }
        validate_safe_component(&self.draft_id)?;
        validate_scope_id("case_id", &self.case_id, "case:")?;
        validate_scope_id("tenant_id", &self.tenant_id, "tenant:")?;
        validate_scope_id("participant_id", &self.participant_id, "participant:")?;
        validate_scope_id("principal_id", &self.principal_id, "principal:")?;
        require_bounded("thread_id", &self.thread_id, 256)?;
        let mut total = 0u64;
        for part in &self.parts {
            validate_safe_component(&part.draft_blob_id)?;
            validate_media_type(&part.modality, &part.media_type)?;
            if part.byte_length == 0 || part.byte_length > MAX_CONTENT_OBJECT_BYTES {
                return Err("conversation_draft_part_size_invalid".to_string());
            }
            require_digest(&part.content_digest)?;
            total = total
                .checked_add(part.byte_length)
                .ok_or_else(|| "conversation_draft_size_overflow".to_string())?;
        }
        if total > MAX_TURN_CONTENT_BYTES {
            return Err("conversation_turn_content_size_exceeded".to_string());
        }
        Ok(())
    }
}

pub struct ConversationContentStore {
    #[cfg(target_os = "linux")]
    root_directory: File,
}

impl ConversationContentStore {
    pub fn open(yai_home: &Path) -> Result<Self, String> {
        #[cfg(not(target_os = "linux"))]
        return Err("conversation_content_store_mutation_requires_linux_openat2".to_string());
        #[cfg(target_os = "linux")]
        {
            let root = yai_home.join("conversation-content-v1");
            ensure_private_directory(yai_home)?;
            ensure_private_directory(&root)?;
            let root_directory = openat2_path(
                libc::AT_FDCWD,
                &root,
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0,
                false,
            )
            .map_err(|error| format!("conversation_content_root_open_failed: {error}"))?;
            validate_owned_directory(&root_directory)?;
            open_child_directory(&root_directory, "objects", true)?;
            open_child_directory(&root_directory, "drafts", true)?;
            Ok(Self { root_directory })
        }
    }

    pub fn create_draft(&self, draft: &ConversationDraft) -> Result<(), String> {
        draft.validate()?;
        let drafts = open_child_directory(&self.root_directory, "drafts", false)?;
        let storage_key = draft_storage_key(&draft.case_id, &draft.draft_id)?;
        if exists_at(&drafts, &storage_key)? {
            return Err("conversation_draft_already_exists".to_string());
        }
        let dir = open_child_directory(&drafts, &storage_key, true)?;
        write_json_create_new_at(&dir, "draft.json", draft, MAX_DRAFT_JSON_BYTES as usize)?;
        sync_directory_fd(&dir)?;
        sync_directory_fd(&drafts)?;
        Ok(())
    }

    pub fn load_draft(&self, case_id: &str, draft_id: &str) -> Result<ConversationDraft, String> {
        let dir = self.open_draft(case_id, draft_id)?;
        let bytes = read_bounded_at(&dir, "draft.json", MAX_DRAFT_JSON_BYTES as usize)?;
        let draft: ConversationDraft = serde_json::from_slice(&bytes)
            .map_err(|error| format!("conversation_draft_decode_failed: {error}"))?;
        draft.validate()?;
        if draft.case_id != case_id || draft.draft_id != draft_id {
            return Err("conversation_draft_storage_scope_mismatch".to_string());
        }
        Ok(draft)
    }

    pub fn stage_bytes(
        &self,
        draft: &mut ConversationDraft,
        modality: ContentModality,
        media_type: &str,
        bytes: &[u8],
        provenance: ContentPartProvenance,
    ) -> Result<usize, String> {
        draft.validate()?;
        if draft.parts.len() >= MAX_TURN_PARTS
            || bytes.is_empty()
            || bytes.len() as u64 > MAX_CONTENT_OBJECT_BYTES
        {
            return Err("conversation_draft_part_bound_invalid".to_string());
        }
        validate_media_type(&modality, media_type)?;
        if modality == ContentModality::Text
            && (bytes.len() > MAX_TEXT_BYTES || std::str::from_utf8(bytes).is_err())
        {
            return Err("conversation_text_content_invalid".to_string());
        }
        let ordinal = draft.parts.len();
        let blob_digest = digest_bytes(bytes);
        let blob_token = digest_token(&blob_digest)?;
        let blob_id = format!("blob-{ordinal:04}-{}", &blob_token[..24]);
        let dir = self.open_draft(&draft.case_id, &draft.draft_id)?;
        write_new_at(&dir, &blob_id, bytes, MAX_CONTENT_OBJECT_BYTES as usize)?;
        draft.parts.push(DraftContentPart {
            draft_blob_id: blob_id,
            modality,
            media_type: media_type.to_string(),
            byte_length: bytes.len() as u64,
            content_digest: digest_bytes(bytes),
            provenance,
        });
        draft.validate()?;
        write_json_atomic_at(&dir, "draft.json", draft, MAX_DRAFT_JSON_BYTES as usize)?;
        sync_directory_fd(&dir)?;
        Ok(ordinal)
    }

    pub fn import_file(
        &self,
        draft: &mut ConversationDraft,
        modality: ContentModality,
        media_type: &str,
        source: &Path,
        provenance: ContentPartProvenance,
    ) -> Result<usize, String> {
        if !source.is_absolute() {
            return Err("conversation_import_source_must_be_absolute".to_string());
        }
        let bytes = read_source_nofollow(source, MAX_CONTENT_OBJECT_BYTES)?;
        self.stage_bytes(draft, modality, media_type, &bytes, provenance)
    }

    pub fn publish_draft(
        &self,
        draft: &ConversationDraft,
    ) -> Result<Vec<ConversationContentPart>, String> {
        self.materialize_draft(draft, true)
    }

    /// Resolve stable content/part identities without adopting draft bytes into
    /// immutable storage. SEND is the only publication boundary.
    pub fn preview_draft(
        &self,
        draft: &ConversationDraft,
    ) -> Result<Vec<ConversationContentPart>, String> {
        self.materialize_draft(draft, false)
    }

    fn materialize_draft(
        &self,
        draft: &ConversationDraft,
        publish: bool,
    ) -> Result<Vec<ConversationContentPart>, String> {
        draft.validate()?;
        let dir = self.open_draft(&draft.case_id, &draft.draft_id)?;
        let mut parts = Vec::with_capacity(draft.parts.len());
        for (ordinal, staged) in draft.parts.iter().enumerate() {
            let bytes = read_bounded_at(
                &dir,
                &staged.draft_blob_id,
                MAX_CONTENT_OBJECT_BYTES as usize,
            )?;
            if bytes.len() as u64 != staged.byte_length
                || digest_bytes(&bytes) != staged.content_digest
            {
                return Err("conversation_draft_blob_integrity_mismatch".to_string());
            }
            let object = ConversationContentObject::new(
                &draft.tenant_id,
                &draft.case_id,
                staged.modality.clone(),
                &staged.media_type,
                &bytes,
            )?;
            if publish {
                self.publish_object(&object, &bytes)?;
            }
            parts.push(ConversationContentPart::build(
                ordinal,
                object,
                staged.provenance.clone(),
            )?);
        }
        Ok(parts)
    }

    pub fn verify_object(&self, object: &ConversationContentObject) -> Result<(), String> {
        object.validate()?;
        let digest = object
            .object_id
            .strip_prefix("content-object:")
            .ok_or_else(|| "conversation_content_object_identity_mismatch".to_string())?;
        validate_safe_component(digest)?;
        let objects = open_child_directory(&self.root_directory, "objects", false)?;
        let object_dir = open_child_directory(&objects, digest, false)?;
        let bytes = read_bounded_at(&object_dir, "payload", object.byte_length as usize)?;
        if bytes.len() as u64 != object.byte_length || digest_bytes(&bytes) != object.content_digest
        {
            return Err("conversation_content_object_integrity_mismatch".to_string());
        }
        let metadata = read_bounded_at(&object_dir, "object.json", 16 * 1024)?;
        let stored: ConversationContentObject = serde_json::from_slice(&metadata)
            .map_err(|error| format!("conversation_content_object_decode_failed: {error}"))?;
        if &stored != object {
            return Err("conversation_content_object_metadata_mismatch".to_string());
        }
        Ok(())
    }

    pub fn read_text(&self, object: &ConversationContentObject) -> Result<String, String> {
        if object.modality != ContentModality::Text {
            return Err("conversation_content_object_not_text".to_string());
        }
        self.verify_object(object)?;
        let digest = object.object_id.trim_start_matches("content-object:");
        let objects = open_child_directory(&self.root_directory, "objects", false)?;
        let object_dir = open_child_directory(&objects, digest, false)?;
        let bytes = read_bounded_at(&object_dir, "payload", object.byte_length as usize)?;
        String::from_utf8(bytes).map_err(|_| "conversation_text_content_invalid".to_string())
    }

    pub fn discard_draft(&self, case_id: &str, draft_id: &str) -> Result<(), String> {
        let drafts = open_child_directory(&self.root_directory, "drafts", false)?;
        let storage_key = draft_storage_key(case_id, draft_id)?;
        remove_tree_at(&drafts, &storage_key)?;
        sync_directory_fd(&drafts)
    }

    #[cfg(target_os = "linux")]
    fn open_draft(&self, case_id: &str, draft_id: &str) -> Result<File, String> {
        let drafts = open_child_directory(&self.root_directory, "drafts", false)?;
        let storage_key = draft_storage_key(case_id, draft_id)?;
        open_child_directory(&drafts, &storage_key, false)
    }

    fn publish_object(
        &self,
        object: &ConversationContentObject,
        bytes: &[u8],
    ) -> Result<(), String> {
        object.validate()?;
        if digest_bytes(bytes) != object.content_digest || bytes.len() as u64 != object.byte_length
        {
            return Err("conversation_content_object_integrity_mismatch".to_string());
        }
        let digest = object.object_id.trim_start_matches("content-object:");
        validate_safe_component(digest)?;
        let objects = open_child_directory(&self.root_directory, "objects", false)?;
        if exists_at(&objects, digest)? {
            return self.verify_object(object);
        }
        let sequence = CONTENT_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temp_name = format!(
            "publish-{}-{sequence}-{}",
            std::process::id(),
            &digest[..24]
        );
        if exists_at(&objects, &temp_name)? {
            return Err("conversation_content_publication_collision".to_string());
        }
        let temp = open_child_directory(&objects, &temp_name, true)?;
        let result = (|| {
            write_new_at(&temp, "payload", bytes, MAX_CONTENT_OBJECT_BYTES as usize)?;
            write_json_create_new_at(&temp, "object.json", object, 16 * 1024)?;
            sync_directory_fd(&temp)?;
            match rename_at(&objects, &temp_name, digest) {
                Ok(()) => {}
                Err(_error) if exists_at(&objects, digest)? => {
                    remove_tree_at(&objects, &temp_name)?;
                    return self.verify_object(object);
                }
                Err(error) => return Err(error),
            }
            sync_directory_fd(&objects)?;
            self.verify_object(object)
        })();
        if result.is_err() {
            let _ = remove_tree_at(&objects, &temp_name);
        }
        result
    }
}

pub fn turns_from_history<'a>(
    case_id: &str,
    transitions: &'a [crate::transition::Transition],
) -> Vec<&'a ConversationTurn> {
    transitions
        .iter()
        .filter_map(|transition| match &transition.payload {
            crate::transition::TransitionPayload::ConversationTurnCommitted { turn }
                if turn.case_id == case_id =>
            {
                Some(turn)
            }
            _ => None,
        })
        .collect()
}

pub fn find_turn<'a>(
    case_id: &str,
    turn_id: &str,
    transitions: &'a [crate::transition::Transition],
) -> Option<&'a ConversationTurn> {
    turns_from_history(case_id, transitions)
        .into_iter()
        .find(|turn| turn.turn_id == turn_id)
}

fn validate_scope_id(field: &str, value: &str, prefix: &str) -> Result<(), String> {
    require_bounded(field, value, 256)?;
    if !value.starts_with(prefix) {
        return Err(format!("{field}_invalid"));
    }
    Ok(())
}

fn require_bounded(field: &str, value: &str, max: usize) -> Result<(), String> {
    if value.trim().is_empty()
        || value.len() > max
        || value
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
    {
        return Err(format!("{field}_invalid"));
    }
    Ok(())
}

fn validate_media_type(modality: &ContentModality, value: &str) -> Result<(), String> {
    require_bounded("media_type", value, 128)?;
    if !value.contains('/') || value.chars().any(|ch| ch.is_whitespace()) {
        return Err("conversation_content_media_type_invalid".to_string());
    }
    let allowed = match modality {
        ContentModality::Text => value.starts_with("text/"),
        ContentModality::Image => value.starts_with("image/"),
        ContentModality::Audio => value.starts_with("audio/"),
        ContentModality::Video => value.starts_with("video/"),
        ContentModality::File => true,
    };
    if !allowed {
        return Err("conversation_content_media_type_modality_mismatch".to_string());
    }
    Ok(())
}

fn require_digest(value: &str) -> Result<(), String> {
    let Some(raw) = value.strip_prefix("sha256:") else {
        return Err("conversation_content_digest_invalid".to_string());
    };
    if raw.len() != 64
        || !raw
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("conversation_content_digest_invalid".to_string());
    }
    Ok(())
}

fn digest_token(value: &str) -> Result<&str, String> {
    require_digest(value)?;
    Ok(value
        .strip_prefix("sha256:")
        .expect("validated digest prefix"))
}

fn validate_safe_component(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("conversation_content_path_component_invalid".to_string());
    }
    Ok(())
}

fn draft_storage_key(case_id: &str, draft_id: &str) -> Result<String, String> {
    validate_scope_id("case_id", case_id, "case:")?;
    validate_safe_component(draft_id)?;
    let digest = digest_json(&(CONTENT_STORE_SCHEMA, "draft", case_id, draft_id))?;
    Ok(format!("draft-{}", digest_token(&digest)?))
}

fn digest_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|bytes| digest_bytes(&bytes))
        .map_err(|error| format!("conversation_identity_encode_failed: {error}"))
}

#[cfg(target_os = "linux")]
static CONTENT_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[cfg(target_os = "linux")]
#[repr(C)]
struct ContentOpenHow {
    flags: u64,
    mode: u64,
    resolve: u64,
}

#[cfg(target_os = "linux")]
fn openat2_path(
    directory_fd: i32,
    path: &Path,
    flags: i32,
    mode: u32,
    beneath: bool,
) -> std::io::Result<File> {
    const RESOLVE_NO_MAGICLINKS: u64 = 0x02;
    const RESOLVE_NO_SYMLINKS: u64 = 0x04;
    const RESOLVE_BENEATH: u64 = 0x08;
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::from_raw_os_error(libc::EINVAL))?;
    let how = ContentOpenHow {
        flags: flags as u64,
        mode: u64::from(mode),
        resolve: RESOLVE_NO_MAGICLINKS
            | RESOLVE_NO_SYMLINKS
            | if beneath { RESOLVE_BENEATH } else { 0 },
    };
    let fd = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            directory_fd,
            path.as_ptr(),
            &how,
            std::mem::size_of::<ContentOpenHow>(),
        ) as i32
    };
    if fd < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}

#[cfg(target_os = "linux")]
fn component_cstring(value: &str) -> Result<CString, String> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > 160
        || value.contains('/')
        || value.as_bytes().contains(&0)
    {
        return Err("conversation_content_path_component_invalid".to_string());
    }
    CString::new(value).map_err(|_| "conversation_content_path_component_invalid".to_string())
}

#[cfg(target_os = "linux")]
fn validate_owned_directory(directory: &File) -> Result<(), String> {
    let metadata = directory.metadata().map_err(storage_error)?;
    if !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o022 != 0
    {
        return Err("conversation_content_store_directory_not_secure".to_string());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn validate_owned_regular(file: &File) -> Result<u64, String> {
    let metadata = file.metadata().map_err(storage_error)?;
    if !metadata.is_file()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o022 != 0
        || metadata.nlink() != 1
    {
        return Err("conversation_content_file_not_private_regular".to_string());
    }
    Ok(metadata.len())
}

#[cfg(target_os = "linux")]
fn open_child_directory(parent: &File, name: &str, create: bool) -> Result<File, String> {
    let name_c = component_cstring(name)?;
    if create {
        let result = unsafe { libc::mkdirat(parent.as_raw_fd(), name_c.as_ptr(), 0o700) };
        if result != 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::EEXIST) {
                return Err(storage_error(error));
            }
        }
    }
    let directory = openat2_path(
        parent.as_raw_fd(),
        Path::new(name),
        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        0,
        true,
    )
    .map_err(|error| format!("conversation_content_directory_open_failed: {error}"))?;
    validate_owned_directory(&directory)?;
    Ok(directory)
}

#[cfg(target_os = "linux")]
fn open_regular_at(parent: &File, name: &str, flags: i32, mode: u32) -> Result<File, String> {
    component_cstring(name)?;
    let file = openat2_path(
        parent.as_raw_fd(),
        Path::new(name),
        flags | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        mode,
        true,
    )
    .map_err(|error| format!("conversation_content_file_open_failed: {error}"))?;
    validate_owned_regular(&file)?;
    Ok(file)
}

#[cfg(target_os = "linux")]
fn exists_at(parent: &File, name: &str) -> Result<bool, String> {
    let name = component_cstring(name)?;
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    let result = unsafe {
        libc::fstatat(
            parent.as_raw_fd(),
            name.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result == 0 {
        Ok(true)
    } else if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
        Ok(false)
    } else {
        Err(storage_error(std::io::Error::last_os_error()))
    }
}

#[cfg(target_os = "linux")]
fn read_bounded_at(parent: &File, name: &str, maximum: usize) -> Result<Vec<u8>, String> {
    let file = open_regular_at(parent, name, libc::O_RDONLY, 0)?;
    let size = usize::try_from(validate_owned_regular(&file)?)
        .map_err(|_| "conversation_content_file_size_invalid".to_string())?;
    if size == 0 || size > maximum {
        return Err("conversation_content_file_size_invalid".to_string());
    }
    let mut bytes = Vec::with_capacity(size);
    file.take((maximum as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(storage_error)?;
    if bytes.len() != size || bytes.len() > maximum {
        return Err("conversation_content_file_size_changed".to_string());
    }
    Ok(bytes)
}

#[cfg(target_os = "linux")]
fn write_new_at(parent: &File, name: &str, bytes: &[u8], maximum: usize) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > maximum {
        return Err("conversation_content_file_size_invalid".to_string());
    }
    let mut file = open_regular_at(
        parent,
        name,
        libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
        0o600,
    )?;
    file.write_all(bytes).map_err(storage_error)?;
    file.sync_all().map_err(storage_error)
}

#[cfg(target_os = "linux")]
fn write_json_create_new_at<T: Serialize>(
    parent: &File,
    name: &str,
    value: &T,
    maximum: usize,
) -> Result<(), String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("conversation_json_encode_failed: {error}"))?;
    write_new_at(parent, name, &bytes, maximum)
}

#[cfg(target_os = "linux")]
fn write_json_atomic_at<T: Serialize>(
    parent: &File,
    name: &str,
    value: &T,
    maximum: usize,
) -> Result<(), String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("conversation_json_encode_failed: {error}"))?;
    let sequence = CONTENT_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = format!("tmp-{}-{sequence}-{name}", std::process::id());
    write_new_at(parent, &temporary, &bytes, maximum)?;
    rename_at(parent, &temporary, name)?;
    sync_directory_fd(parent)
}

#[cfg(target_os = "linux")]
fn sync_directory_fd(directory: &File) -> Result<(), String> {
    let result = unsafe { libc::fsync(directory.as_raw_fd()) };
    if result == 0 {
        Ok(())
    } else {
        Err(storage_error(std::io::Error::last_os_error()))
    }
}

#[cfg(target_os = "linux")]
fn rename_at(directory: &File, source: &str, target: &str) -> Result<(), String> {
    let source = component_cstring(source)?;
    let target = component_cstring(target)?;
    let result = unsafe {
        libc::renameat(
            directory.as_raw_fd(),
            source.as_ptr(),
            directory.as_raw_fd(),
            target.as_ptr(),
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(storage_error(std::io::Error::last_os_error()))
    }
}

#[cfg(target_os = "linux")]
fn list_directory_names(directory: &File) -> Result<Vec<String>, String> {
    let duplicate = unsafe { libc::dup(directory.as_raw_fd()) };
    if duplicate < 0 {
        return Err(storage_error(std::io::Error::last_os_error()));
    }
    let stream = unsafe { libc::fdopendir(duplicate) };
    if stream.is_null() {
        unsafe { libc::close(duplicate) };
        return Err(storage_error(std::io::Error::last_os_error()));
    }
    let mut names = Vec::new();
    loop {
        let entry = unsafe { libc::readdir(stream) };
        if entry.is_null() {
            break;
        }
        let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }
            .to_str()
            .map_err(|_| "conversation_content_directory_entry_non_utf8".to_string())?;
        if name != "." && name != ".." {
            names.push(name.to_string());
        }
    }
    unsafe { libc::closedir(stream) };
    names.sort();
    Ok(names)
}

#[cfg(target_os = "linux")]
fn entry_mode_at(directory: &File, name: &str) -> Result<libc::mode_t, String> {
    let name = component_cstring(name)?;
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    let result = unsafe {
        libc::fstatat(
            directory.as_raw_fd(),
            name.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result != 0 {
        return Err(storage_error(std::io::Error::last_os_error()));
    }
    Ok(unsafe { stat.assume_init() }.st_mode)
}

#[cfg(target_os = "linux")]
fn unlink_at(directory: &File, name: &str, flags: i32) -> Result<(), String> {
    let name = component_cstring(name)?;
    let result = unsafe { libc::unlinkat(directory.as_raw_fd(), name.as_ptr(), flags) };
    if result == 0 {
        Ok(())
    } else {
        Err(storage_error(std::io::Error::last_os_error()))
    }
}

#[cfg(target_os = "linux")]
fn remove_tree_at(parent: &File, name: &str) -> Result<bool, String> {
    if !exists_at(parent, name)? {
        return Ok(false);
    }
    let directory = open_child_directory(parent, name, false)?;
    for child in list_directory_names(&directory)? {
        let mode = entry_mode_at(&directory, &child)?;
        if mode & libc::S_IFMT == libc::S_IFDIR {
            remove_tree_at(&directory, &child)?;
        } else {
            unlink_at(&directory, &child, 0)?;
        }
    }
    unlink_at(parent, name, libc::AT_REMOVEDIR)?;
    Ok(true)
}

fn ensure_private_directory(path: &Path) -> Result<(), String> {
    match fs::create_dir(path) {
        Ok(()) => set_private_dir_mode(path)?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(storage_error(error)),
    }
    let metadata = fs::symlink_metadata(path).map_err(storage_error)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err("conversation_content_store_directory_invalid".to_string());
    }
    #[cfg(unix)]
    {
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o022 != 0 {
            return Err("conversation_content_store_permissions_invalid".to_string());
        }
    }
    Ok(())
}

fn set_private_dir_mode(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(storage_error)?;
    Ok(())
}

fn read_source_nofollow(path: &Path, max: u64) -> Result<Vec<u8>, String> {
    #[cfg(not(target_os = "linux"))]
    return Err("conversation_content_import_requires_linux_openat2".to_string());
    #[cfg(target_os = "linux")]
    let file = openat2_path(
        libc::AT_FDCWD,
        path,
        libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        0,
        false,
    )
    .map_err(storage_error)?;
    read_bounded_open_file(file, max)
}

fn read_bounded_open_file(file: File, max: u64) -> Result<Vec<u8>, String> {
    let metadata = file.metadata().map_err(storage_error)?;
    if !metadata.file_type().is_file() || metadata.len() == 0 || metadata.len() > max {
        return Err("conversation_content_file_size_invalid".to_string());
    }
    #[cfg(unix)]
    if metadata.nlink() != 1 {
        return Err("conversation_content_file_link_count_invalid".to_string());
    }
    let capacity = usize::try_from(metadata.len())
        .map_err(|_| "conversation_content_file_size_invalid".to_string())?;
    let mut bytes = Vec::with_capacity(capacity);
    file.take(max.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(storage_error)?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > max {
        return Err("conversation_content_file_size_changed".to_string());
    }
    Ok(bytes)
}

fn storage_error(error: std::io::Error) -> String {
    format!("conversation_content_store_io: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(label: &str) -> PathBuf {
        let value =
            std::env::temp_dir().join(format!("yai-conversation-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&value);
        fs::create_dir(&value).unwrap();
        #[cfg(unix)]
        fs::set_permissions(&value, fs::Permissions::from_mode(0o700)).unwrap();
        value
    }

    fn draft(id: &str) -> ConversationDraft {
        ConversationDraft {
            schema: CONVERSATION_DRAFT_SCHEMA.into(),
            draft_id: id.into(),
            case_id: "case:i01".into(),
            tenant_id: "tenant:i01".into(),
            thread_id: "thread:i01".into(),
            participant_id: "participant:operator".into(),
            principal_id: "principal:operator".into(),
            base_generation: 4,
            parts: Vec::new(),
        }
    }

    fn original() -> ContentPartProvenance {
        ContentPartProvenance::Original {
            imported_by_principal_id: "principal:operator".into(),
        }
    }

    #[test]
    fn multipart_order_and_duplicate_content_survive_publication() {
        let home = root("order");
        let store = ConversationContentStore::open(&home).unwrap();
        let mut value = draft("draft-order");
        store.create_draft(&value).unwrap();
        store
            .stage_bytes(
                &mut value,
                ContentModality::Text,
                "text/plain",
                b"hello",
                original(),
            )
            .unwrap();
        store
            .stage_bytes(
                &mut value,
                ContentModality::Image,
                "image/png",
                b"not-decoded-one",
                original(),
            )
            .unwrap();
        store
            .stage_bytes(
                &mut value,
                ContentModality::Image,
                "image/png",
                b"not-decoded-one",
                original(),
            )
            .unwrap();
        let parts = store.publish_draft(&value).unwrap();
        let turn = ConversationTurn::build(
            &value.case_id,
            &value.tenant_id,
            &value.thread_id,
            &value.participant_id,
            &value.principal_id,
            value.base_generation,
            parts,
        )
        .unwrap();
        assert_eq!(
            turn.ordered_parts
                .iter()
                .map(|part| part.object.modality.clone())
                .collect::<Vec<_>>(),
            vec![
                ContentModality::Text,
                ContentModality::Image,
                ContentModality::Image
            ]
        );
        assert_eq!(
            turn.ordered_parts[1].object.object_id,
            turn.ordered_parts[2].object.object_id
        );
        assert_ne!(turn.ordered_parts[1].part_id, turn.ordered_parts[2].part_id);
        for part in &turn.ordered_parts {
            store.verify_object(&part.object).unwrap();
        }
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn original_machine_transcript_and_human_edit_are_distinct() {
        let home = root("provenance");
        let store = ConversationContentStore::open(&home).unwrap();
        let mut value = draft("draft-provenance");
        store.create_draft(&value).unwrap();
        store
            .stage_bytes(
                &mut value,
                ContentModality::Audio,
                "audio/wav",
                b"bounded-audio-fixture",
                original(),
            )
            .unwrap();
        let first = store.preview_draft(&value).unwrap().remove(0);
        let machine_proto = ContentDerivation {
            schema: CONTENT_DERIVATION_SCHEMA.into(),
            derivation_id: String::new(),
            case_id: value.case_id.clone(),
            kind: ContentDerivationKind::SpeechTranscription,
            source_part_ids: vec![first.part_id.clone()],
            actor_kind: DerivationActorKind::Provider,
            actor_ref: "provider-target:fixture-stt".into(),
            provider_result_id: Some("provider-result:fixture-stt".into()),
        };
        let mut machine = machine_proto;
        machine.derivation_id =
            format!("content-derivation:{}", machine.identity_digest().unwrap());
        store
            .stage_bytes(
                &mut value,
                ContentModality::Text,
                "text/plain",
                b"machine words",
                ContentPartProvenance::Derived {
                    derivation: machine,
                },
            )
            .unwrap();
        let published = store.preview_draft(&value).unwrap();
        let machine_part = published[1].clone();
        let edit_proto = ContentDerivation {
            schema: CONTENT_DERIVATION_SCHEMA.into(),
            derivation_id: String::new(),
            case_id: value.case_id.clone(),
            kind: ContentDerivationKind::HumanEdit,
            source_part_ids: vec![machine_part.part_id.clone()],
            actor_kind: DerivationActorKind::Human,
            actor_ref: value.principal_id.clone(),
            provider_result_id: None,
        };
        let mut edit = edit_proto;
        edit.derivation_id = format!("content-derivation:{}", edit.identity_digest().unwrap());
        store
            .stage_bytes(
                &mut value,
                ContentModality::Text,
                "text/plain",
                b"human edited words",
                ContentPartProvenance::Derived { derivation: edit },
            )
            .unwrap();
        let turn = ConversationTurn::build(
            &value.case_id,
            &value.tenant_id,
            &value.thread_id,
            &value.participant_id,
            &value.principal_id,
            value.base_generation,
            store.publish_draft(&value).unwrap(),
        )
        .unwrap();
        assert_eq!(turn.ordered_parts.len(), 3);
        assert_ne!(
            turn.ordered_parts[0].object.object_id,
            turn.ordered_parts[1].object.object_id
        );
        assert_ne!(
            turn.ordered_parts[1].object.object_id,
            turn.ordered_parts[2].object.object_id
        );
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn rejects_cross_case_derivation_and_oversized_metadata() {
        let object = ConversationContentObject::new(
            "tenant:i01",
            "case:i01",
            ContentModality::Text,
            "text/plain",
            b"source",
        )
        .unwrap();
        let source = ConversationContentPart::build(0, object, original()).unwrap();
        let mut bad = ContentDerivation {
            schema: CONTENT_DERIVATION_SCHEMA.into(),
            derivation_id: String::new(),
            case_id: "case:other".into(),
            kind: ContentDerivationKind::HumanEdit,
            source_part_ids: vec![source.part_id.clone()],
            actor_kind: DerivationActorKind::Human,
            actor_ref: "principal:operator".into(),
            provider_result_id: None,
        };
        bad.derivation_id = format!("content-derivation:{}", bad.identity_digest().unwrap());
        assert_eq!(
            bad.validate("case:i01", &BTreeSet::from([source.part_id.clone()]))
                .unwrap_err(),
            "conversation_content_derivation_scope_invalid"
        );
        let mut forged_human_edit = ContentDerivation {
            schema: CONTENT_DERIVATION_SCHEMA.into(),
            derivation_id: String::new(),
            case_id: "case:i01".into(),
            kind: ContentDerivationKind::HumanEdit,
            source_part_ids: vec![source.part_id.clone()],
            actor_kind: DerivationActorKind::Deterministic,
            actor_ref: "fixture:forged-human".into(),
            provider_result_id: None,
        };
        forged_human_edit.derivation_id = format!(
            "content-derivation:{}",
            forged_human_edit.identity_digest().unwrap()
        );
        assert_eq!(
            forged_human_edit
                .validate("case:i01", &BTreeSet::from([source.part_id.clone()]))
                .unwrap_err(),
            "conversation_human_edit_actor_invalid"
        );
        let mut forged_result = ContentDerivation {
            schema: CONTENT_DERIVATION_SCHEMA.into(),
            derivation_id: String::new(),
            case_id: "case:i01".into(),
            kind: ContentDerivationKind::SpeechTranscription,
            source_part_ids: vec![source.part_id.clone()],
            actor_kind: DerivationActorKind::Deterministic,
            actor_ref: "fixture:deterministic".into(),
            provider_result_id: Some("provider-result:forged".into()),
        };
        forged_result.derivation_id = format!(
            "content-derivation:{}",
            forged_result.identity_digest().unwrap()
        );
        assert_eq!(
            forged_result
                .validate("case:i01", &BTreeSet::from([source.part_id]))
                .unwrap_err(),
            "conversation_deterministic_derivation_result_forbidden"
        );
        assert!(ConversationContentObject::new(
            "tenant:i01",
            "case:i01",
            ContentModality::Text,
            "text/plain",
            &vec![b'x'; MAX_TEXT_BYTES + 1]
        )
        .is_err());

        let wrong_tenant = ConversationContentPart::build(
            0,
            ConversationContentObject::new(
                "tenant:other",
                "case:i01",
                ContentModality::Text,
                "text/plain",
                b"cross-tenant",
            )
            .unwrap(),
            original(),
        )
        .unwrap();
        assert_eq!(
            ConversationTurn::build(
                "case:i01",
                "tenant:i01",
                "thread:i01",
                "participant:operator",
                "principal:operator",
                4,
                vec![wrong_tenant],
            )
            .unwrap_err(),
            "conversation_content_part_tenant_mismatch"
        );
        let wrong_principal = ConversationContentPart::build(
            0,
            ConversationContentObject::new(
                "tenant:i01",
                "case:i01",
                ContentModality::Text,
                "text/plain",
                b"wrong-principal",
            )
            .unwrap(),
            ContentPartProvenance::Original {
                imported_by_principal_id: "principal:other".into(),
            },
        )
        .unwrap();
        assert_eq!(
            ConversationTurn::build(
                "case:i01",
                "tenant:i01",
                "thread:i01",
                "participant:operator",
                "principal:operator",
                4,
                vec![wrong_principal],
            )
            .unwrap_err(),
            "conversation_original_content_principal_mismatch"
        );

        let home = root("oversized-import");
        let oversized = home.join("oversized.bin");
        File::create(&oversized)
            .unwrap()
            .set_len(MAX_CONTENT_OBJECT_BYTES + 1)
            .unwrap();
        assert_eq!(
            read_source_nofollow(&oversized, MAX_CONTENT_OBJECT_BYTES).unwrap_err(),
            "conversation_content_file_size_invalid"
        );
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn corruption_and_internal_symlink_substitution_fail_closed() {
        use std::os::unix::fs::symlink;

        let home = root("integrity");
        let store = ConversationContentStore::open(&home).unwrap();
        let mut value = draft("draft-integrity");
        store.create_draft(&value).unwrap();
        store
            .stage_bytes(
                &mut value,
                ContentModality::Image,
                "image/png",
                b"opaque-image",
                original(),
            )
            .unwrap();
        let part = store.publish_draft(&value).unwrap().remove(0);
        let token = part.object.object_id.trim_start_matches("content-object:");
        fs::write(
            home.join("conversation-content-v1/objects")
                .join(token)
                .join("payload"),
            b"forged-image",
        )
        .unwrap();
        assert_eq!(
            store.verify_object(&part.object).unwrap_err(),
            "conversation_content_object_integrity_mismatch"
        );

        let outside = root("outside");
        fs::write(
            outside.join("draft.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        let drafts = home.join("conversation-content-v1/drafts");
        let storage_key = draft_storage_key("case:i01", "draft-integrity").unwrap();
        fs::remove_dir_all(drafts.join(&storage_key)).unwrap();
        symlink(&outside, drafts.join(&storage_key)).unwrap();
        assert!(store
            .load_draft("case:i01", "draft-integrity")
            .unwrap_err()
            .contains("directory_open_failed"));
        assert!(outside.join("draft.json").exists());
        let _ = fs::remove_dir_all(home);
        let _ = fs::remove_dir_all(outside);
    }

    #[test]
    fn identical_bytes_remain_case_and_tenant_scoped() {
        let bytes = b"same bytes";
        let a = ConversationContentObject::new(
            "tenant:a",
            "case:a",
            ContentModality::File,
            "application/octet-stream",
            bytes,
        )
        .unwrap();
        let case_b = ConversationContentObject::new(
            "tenant:a",
            "case:b",
            ContentModality::File,
            "application/octet-stream",
            bytes,
        )
        .unwrap();
        let tenant_b = ConversationContentObject::new(
            "tenant:b",
            "case:a",
            ContentModality::File,
            "application/octet-stream",
            bytes,
        )
        .unwrap();
        assert_ne!(a.object_id, case_b.object_id);
        assert_ne!(a.object_id, tenant_b.object_id);
        assert_eq!(a.content_digest, case_b.content_digest);
    }

    #[test]
    fn identical_draft_labels_are_namespaced_by_case_and_preview_does_not_adopt_bytes() {
        let home = root("draft-scope");
        let store = ConversationContentStore::open(&home).unwrap();
        let mut first = draft("same-label");
        let mut second = draft("same-label");
        second.case_id = "case:i01-other".to_string();
        store.create_draft(&first).unwrap();
        store.create_draft(&second).unwrap();
        store
            .stage_bytes(
                &mut first,
                ContentModality::Audio,
                "audio/x-yai-fixture",
                b"uncommitted-audio",
                original(),
            )
            .unwrap();
        let preview = store.preview_draft(&first).unwrap();
        let token = preview[0]
            .object
            .object_id
            .trim_start_matches("content-object:");
        assert!(!home
            .join("conversation-content-v1/objects")
            .join(token)
            .exists());
        assert_eq!(
            store.load_draft("case:i01", "same-label").unwrap().case_id,
            "case:i01"
        );
        assert_eq!(
            store
                .load_draft("case:i01-other", "same-label")
                .unwrap()
                .case_id,
            "case:i01-other"
        );
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn incomplete_object_publication_is_never_accepted_as_adopted_content() {
        let home = root("partial-publication");
        let store = ConversationContentStore::open(&home).unwrap();
        let mut value = draft("partial-publication");
        store.create_draft(&value).unwrap();
        store
            .stage_bytes(
                &mut value,
                ContentModality::Image,
                "image/png",
                b"complete-draft-image",
                original(),
            )
            .unwrap();
        let preview = store.preview_draft(&value).unwrap();
        let token = preview[0]
            .object
            .object_id
            .trim_start_matches("content-object:");
        let incomplete = home.join("conversation-content-v1/objects").join(token);
        fs::create_dir(&incomplete).unwrap();
        #[cfg(unix)]
        fs::set_permissions(&incomplete, fs::Permissions::from_mode(0o700)).unwrap();
        let payload = incomplete.join("payload");
        fs::write(&payload, b"complete-draft-image").unwrap();
        #[cfg(unix)]
        fs::set_permissions(&payload, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(store
            .publish_draft(&value)
            .unwrap_err()
            .contains("file_open_failed"));
        let _ = fs::remove_dir_all(home);
    }
}
