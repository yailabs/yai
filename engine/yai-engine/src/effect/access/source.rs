//! Case-native source acquisition relationships. Bytes and policy remain with
//! their existing immutable owners; inventory is a replay-derived projection.
//! No knowledge derivation, acquisition database, or authority evaluator.

use super::{is_digest, ResourceAction, ResourceRequest, RESOURCE_REQUEST_SCHEMA};
use crate::effect::digest_bytes;
use serde::{Deserialize, Serialize};

pub const SOURCE_DECLARATION_SCHEMA: &str = "yai.case_source_declaration.v1";
pub const SOURCE_PROGRESS_SCHEMA: &str = "yai.case_source_progress.v1";
pub const MAX_CASE_SOURCES: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRole {
    Policy,
    Knowledge,
    Operational,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseSourceDeclaration {
    pub schema: String,
    pub source_id: String,
    pub case_id: String,
    pub perimeter: String,
    pub logical_name: String,
    pub participant_id: String,
    pub declared_by_principal_id: String,
    pub resource_attachment_id: String,
    pub configuration_digest: String,
    pub roles: Vec<SourceRole>,
    pub action: ResourceAction,
    pub bootstrap_policy: bool,
    pub media_type: String,
}

impl CaseSourceDeclaration {
    pub fn seal(mut self) -> Result<Self, String> {
        self.roles.sort();
        self.roles.dedup();
        self.source_id.clear();
        self.source_id = format!("case-source:{}", &digest(&self)[7..]);
        self.validate()?;
        Ok(self)
    }
    pub fn request(&self) -> ResourceRequest {
        ResourceRequest {
            schema: RESOURCE_REQUEST_SCHEMA.into(),
            configuration_digest: self.configuration_digest.clone(),
            action: self.action.clone(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        let mut copy = self.clone();
        copy.source_id.clear();
        let mut roles = self.roles.clone();
        roles.sort();
        roles.dedup();
        self.request().validate()?;
        if self.schema != SOURCE_DECLARATION_SCHEMA
            || self.source_id != format!("case-source:{}", &digest(&copy)[7..])
            || !self.case_id.starts_with("case:")
            || !self.participant_id.starts_with("participant:")
            || !self.declared_by_principal_id.starts_with("principal:")
            || !name(&self.perimeter)
            || !name(&self.logical_name)
            || !name(&self.resource_attachment_id)
            || !name(&self.media_type)
            || self.roles.is_empty()
            || self.roles != roles
            || !matches!(
                self.action,
                ResourceAction::Discover { .. }
                    | ResourceAction::DatabaseQuery { .. }
                    | ResourceAction::HttpFetch { .. }
            )
            || (self.bootstrap_policy
                && (!self.roles.contains(&SourceRole::Policy)
                    || !matches!(self.action, ResourceAction::Discover { .. })))
        {
            return Err("case_source_declaration_invalid".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceBacking {
    Policy {
        source_id: String,
        artifact_id: String,
    },
    Content {
        admission_id: String,
    },
    Observation {
        observation_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRevisionItem {
    pub path: String,
    pub digest: String,
    pub bytes: u64,
    pub backing: SourceBacking,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRevision {
    pub revision_id: String,
    pub items: Vec<SourceRevisionItem>,
}

impl SourceRevision {
    pub fn new(source_id: &str, mut items: Vec<SourceRevisionItem>) -> Result<Self, String> {
        items.sort_by(|a, b| a.path.cmp(&b.path));
        if items.len() > super::MAX_ACCESS_ITEMS
            || items
                .iter()
                .any(|i| !name(&i.path) || !is_digest(&i.digest))
            || items.windows(2).any(|pair| pair[0].path == pair[1].path)
        {
            return Err("case_source_revision_invalid".into());
        }
        // Observation/admission retries cannot change revision identity if
        // the logical source and exact captured material remain identical.
        let material: Vec<_> = items
            .iter()
            .map(|i| (&i.path, &i.digest, i.bytes))
            .collect();
        let revision_id = format!("source-revision:{}", &digest(&(source_id, material))[7..]);
        Ok(Self { revision_id, items })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourcePhase {
    Acquiring,
    Acquired,
    Denied,
    AwaitingReview,
    Inaccessible,
    NeedsProcessing,
    Revoked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceProgress {
    pub schema: String,
    pub progress_id: String,
    pub source_id: String,
    pub previous_progress_id: Option<String>,
    pub attempt: u64,
    pub phase: SourcePhase,
    pub revision: Option<SourceRevision>,
    pub decision_ref: Option<String>,
    pub detail: String,
}

impl SourceProgress {
    pub fn seal(mut self) -> Result<Self, String> {
        self.progress_id.clear();
        self.progress_id = format!("source-progress:{}", &digest(&self)[7..]);
        self.validate()?;
        Ok(self)
    }
    pub fn validate(&self) -> Result<(), String> {
        let mut copy = self.clone();
        copy.progress_id.clear();
        if self.schema != SOURCE_PROGRESS_SCHEMA
            || self.attempt == 0
            || !self.source_id.starts_with("case-source:")
            || self.progress_id != format!("source-progress:{}", &digest(&copy)[7..])
            || self.detail.len() > 512
            || self.detail.chars().any(char::is_control)
            || (self.phase == SourcePhase::Acquired && self.revision.is_none())
        {
            return Err("case_source_progress_invalid".into());
        }
        if let Some(revision) = &self.revision {
            if SourceRevision::new(&self.source_id, revision.items.clone())? != *revision {
                return Err("case_source_revision_identity_mismatch".into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CaseSourceState {
    pub declaration: CaseSourceDeclaration,
    pub progress: Option<SourceProgress>,
}

pub(crate) fn reduce_declaration(
    sources: &mut Vec<CaseSourceState>,
    source: &CaseSourceDeclaration,
) -> Result<(), String> {
    source.validate()?;
    if sources.len() >= MAX_CASE_SOURCES
        || sources
            .iter()
            .any(|s| s.declaration.logical_name == source.logical_name)
    {
        return Err("case_source_duplicate_or_bound_exceeded".into());
    }
    sources.push(CaseSourceState {
        declaration: source.clone(),
        progress: None,
    });
    sources.sort_by(|a, b| a.declaration.logical_name.cmp(&b.declaration.logical_name));
    Ok(())
}

pub(crate) fn reduce_progress(
    sources: &mut [CaseSourceState],
    progress: &SourceProgress,
) -> Result<(), String> {
    progress.validate()?;
    let source = sources
        .iter_mut()
        .find(|s| s.declaration.source_id == progress.source_id)
        .ok_or("case_source_not_declared")?;
    if progress.previous_progress_id != source.progress.as_ref().map(|p| p.progress_id.clone())
        || progress.attempt < source.progress.as_ref().map_or(1, |p| p.attempt)
        || progress.attempt > source.progress.as_ref().map_or(1, |p| p.attempt + 1)
        || source
            .progress
            .as_ref()
            .is_some_and(|p| p.phase == SourcePhase::Revoked)
    {
        return Err("case_source_progress_stale_or_revoked".into());
    }
    source.progress = Some(progress.clone());
    Ok(())
}

fn name(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}
fn digest(value: &impl Serialize) -> String {
    digest_bytes(&serde_json::to_vec(value).expect("source contract serializes"))
}
