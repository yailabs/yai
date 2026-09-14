//! Transactional composition of Case source declarations, existing immutable
//! backing and existing admission. No acquisition store or second policy engine.
use super::*;
use crate::effect::access::source::*;
use crate::effect::access::{read_confined_file, ResourceAction, ResourceRequest};
use crate::effect::DecisionOutcome;

impl LmdbRecordStore {
    /// Exact source closure under current disclosure, shared by native reads and
    /// routing inspection. Never rereads live files; no canonical writes.
    pub fn resolve_case_source_authorized(
        &self, auth: &AuthenticatedPrincipal, case: &str, name: &str, revision: Option<&str>,
        content: &crate::conversation::ConversationContentStore,
    ) -> Result<ResolvedCaseSource, String> {
        let (state, source) = self.case_source_authorized(auth, case, name)?;
        if source.progress.as_ref().is_none_or(|p| p.phase != SourcePhase::Acquired) {
            return Err("source_not_available".into());
        }
        let txn = self.env.begin_ro_txn().map_err(|e| e.to_string())?;
        if self.source_permission_txn(&txn, &state, &source.declaration, source.declaration.action.clone())?.outcome != DecisionOutcome::Allow {
            return Err("source_not_available".into());
        }
        let history = self.list_case_transitions_txn(&txn, case)?;
        let captured = if let Some(id) = revision {
            history.iter().rev().find_map(|t| match &t.payload {
                TransitionPayload::CaseSourceProgressed { progress } if progress.source_id == source.declaration.source_id =>
                    progress.revision.as_ref().filter(|r| r.revision_id == id).cloned(),
                _ => None,
            })
        } else { source.progress.as_ref().and_then(|p| p.revision.clone()) }.ok_or("source_not_available")?;
        let mut items = Vec::new();
        for item in &captured.items {
            let bytes = match &item.backing {
                SourceBacking::Policy { source_id, artifact_id } => self.source_policy_original_txn(
                    &txn, state.tenant_id.as_deref().ok_or("source_requires_tenant")?, source_id, artifact_id
                )?.original_bytes().to_vec(),
                SourceBacking::Content { admission_id } => {
                    if self.source_permission_txn(&txn, &state, &source.declaration,
                        ResourceAction::ContentRead { admission_id: admission_id.clone() })?.outcome != DecisionOutcome::Allow {
                        return Err("source_not_available".into());
                    }
                    let a = history.iter().find_map(|t| match &t.payload {
                        TransitionPayload::CaseContentAdmitted { admission } if admission.admission_id == *admission_id => Some(admission),
                        _ => None,
                    }).ok_or("source_backing_unavailable")?;
                    content.read_bytes(&a.object)?
                }
                SourceBacking::Observation { observation_id } => {
                    let o = history.iter().find_map(|t| match &t.payload {
                        TransitionPayload::ResourceObservationRecorded { observation } if observation.observation_id == *observation_id => Some(observation),
                        _ => None,
                    }).ok_or("source_backing_unavailable")?;
                    serde_json::to_vec(&o.result).map_err(|e| e.to_string())?
                }
            };
            if digest_bytes(&bytes) != item.digest || bytes.len() as u64 != item.bytes { return Err("source_backing_integrity_mismatch".into()); }
            items.push((item.clone(), bytes));
        }
        drop(txn);
        if self.get_case_state_authorized(auth, case)?.generation != state.generation
            || self.case_source_permission(auth, case, name, None)?.outcome != DecisionOutcome::Allow {
            return Err("source_visibility_changed_during_read".into());
        }
        Ok(ResolvedCaseSource { declaration: source.declaration, revision: captured, items })
    }

    pub fn case_source_routing_authorized(
        &self, auth: &AuthenticatedPrincipal, case: &str, name: &str, revision: Option<&str>,
        content: &crate::conversation::ConversationContentStore,
    ) -> Result<SourceRoutingView, String> {
        let resolved = self.resolve_case_source_authorized(auth, case, name, revision, content)?;
        let mut items = Vec::new();
        for (item, bytes) in resolved.items {
            let routing = crate::governance::route_mixed_document(&bytes, &resolved.declaration.roles)?;
            items.push((item, routing));
        }
        let mut view = SourceRoutingView { schema: "yai.source_routing.v1".into(), id: String::new(), case_id: case.into(),
            source_id: resolved.declaration.source_id, revision_id: resolved.revision.revision_id, items };
        view.id = format!("source-routing:{}", &digest_bytes(&serde_json::to_vec(&(&view, auth.projected_principal_id())).map_err(|e| e.to_string())?)[7..]);
        Ok(view)
    }

    /// Explicit operator publication, through the existing catalog and exact
    /// binding/replacement owners. Source role/routing alone cannot call this.
    pub fn publish_case_source_policy_authorized(
        &self, auth: &AuthenticatedPrincipal, case: &str, name: &str, reason: &str,
    ) -> Result<(), String> {
        let (state, source) = self.case_source_authorized(auth, case, name)?;
        if !source.declaration.roles.contains(&SourceRole::Policy) { return Err("knowledge_source_is_not_policy".into()); }
        if source.progress.as_ref().is_none_or(|p| p.phase != SourcePhase::Acquired)
            || state.sources.iter().any(|s| s.declaration.bootstrap_policy
                && s.progress.as_ref().is_none_or(|p| p.phase != SourcePhase::Acquired)) {
            return Err("all_bootstrap_policy_sources_must_be_acquired_before_publication".into());
        }
        for item in &source.progress.as_ref().unwrap().revision.as_ref().ok_or("policy_source_not_acquired")?.items {
            let SourceBacking::Policy { artifact_id, .. } = &item.backing else { return Err("source_has_no_policy_candidate".into()); };
            self.validate_tenant_policy_artifact(auth, artifact_id, reason)?;
            self.publish_tenant_policy_artifact(auth, artifact_id, reason)?;
            let current = self.get_case_state_authorized(auth, case)?;
            if current.policy_bindings.iter().any(|b| b.artifact_id == *artifact_id) { continue; }
            let artifact = self.get_policy_artifact(artifact_id)?.ok_or("policy_source_not_acquired")?;
            let prior = current.policy_bindings.iter().find(|b| self.get_policy_artifact(&b.artifact_id)
                .ok().flatten().is_some_and(|a| a.lineage() == artifact.lineage()));
            if let Some(prior) = prior {
                self.replace_tenant_case_policy(auth, case, &prior.binding_id, artifact_id, current.generation, reason)?;
            } else { self.bind_tenant_case_policy(auth, case, artifact_id, current.generation, reason)?; }
        }
        Ok(())
    }

    /// Current authority for the declared exact policy file. Setup is available
    /// only before any binding; later revision capture uses ordinary policy.
    pub(super) fn source_policy_capture_allowed_txn<T: Transaction>(
        &self, txn: &T, state: &CaseState, d: &CaseSourceDeclaration,
    ) -> Result<(), String> {
        if !d.bootstrap_policy || !d.roles.contains(&SourceRole::Policy)
            || state.sources.iter().find(|s| s.declaration.source_id == d.source_id)
                .and_then(|s| s.progress.as_ref()).is_some_and(|p| p.phase == SourcePhase::Revoked) {
            return Err("source_policy_capture_not_authorized".into());
        }
        state.resources.iter().find(|r| r.attachment_id == d.resource_attachment_id)
            .and_then(|r| r.access.as_ref()).ok_or("source_resource_unavailable")?
            .admits_request(&d.participant_id, &d.request())?;
        let governed = self.list_case_transitions_txn(txn, &state.case_id)?.iter().any(|t|
            matches!(t.payload, TransitionPayload::CasePolicyBound { .. }
                | TransitionPayload::CasePolicyReplaced { .. } | TransitionPayload::CasePolicyUnbound { .. }));
        if governed {
            if !d.media_type.ends_with(";profile=yai-mixed-v1") {
                return Err("source_bootstrap_authority_unavailable".into());
            }
            if self.source_permission_txn(txn, state, d, d.action.clone())?.outcome != DecisionOutcome::Allow {
                return Err("source_current_authority_refused".into());
            }
            Ok(())
        } else { self.source_bootstrap_allowed_txn(txn, state, d) }
    }

    /// Mixed profile only: newer documentary surroundings may retain the exact
    /// already-catalogued normative JSON, without republishing its authority.
    fn source_policy_original_txn<T: Transaction>(
        &self, txn: &T, tenant: &str, source_id: &str, artifact_id: &str,
    ) -> Result<crate::governance::PolicySourceArtifact, String> {
        let artifact = self.policy_artifact_txn(txn, artifact_id)?;
        if artifact.tenant_id.as_deref() != Some(tenant) { return Err("source_backing_unavailable".into()); }
        let original = decode_policy_source(txn.get(self.policy_sources_by_id, &policy_source_key(source_id))
            .map_err(|_| "source_backing_unavailable")?)?;
        if artifact.source_id != source_id {
            let normative = decode_policy_source(txn.get(self.policy_sources_by_id, &policy_source_key(&artifact.source_id))
                .map_err(|_| "source_backing_unavailable")?)?;
            if !original.is_mixed() || !normative.is_mixed() || original.content_utf8 != normative.content_utf8 {
                return Err("source_policy_region_mismatch".into());
            }
        }
        Ok(original)
    }
    /// Qualified D inputs in one current authorization/catalog/history snapshot.
    /// No live-source read, acquisition, canonical write or cached-view trust.
    /// Missing and hidden source selectors share the same refusal. Hidden
    /// sources never enter derivation, graph topology, counts or BM25 statistics.
    pub fn case_knowledge_authorized(
        &self,
        auth: &AuthenticatedPrincipal,
        request: crate::memory_hierarchy::knowledge::KnowledgeRequest,
        content: Option<&crate::conversation::ConversationContentStore>,
    ) -> Result<crate::memory_hierarchy::knowledge::KnowledgeResult, String> {
        request.validate()?;
        let txn = self.env.begin_ro_txn().map_err(|e| e.to_string())?;
        let state = self
            .get_case_state_txn(&txn, &request.case_id)?
            .ok_or("case_not_visible")?;
        let history = self.list_case_transitions_txn(&txn, &state.case_id)?;
        self.qualified_knowledge_txn(&txn, auth, request, &state, &history, None, content)
            .map(|(r, _)| r)
    }

    /// Reused by Recall in the same transaction/history snapshot as H. The cut
    /// restricts knowledge availability; only CURRENT state admits disclosure.
    pub(super) fn qualified_knowledge_txn<T: Transaction>(
        &self,
        txn: &T,
        auth: &AuthenticatedPrincipal,
        request: crate::memory_hierarchy::knowledge::KnowledgeRequest,
        state: &CaseState,
        history: &[Transition],
        recall_input: Option<(&crate::memory_hierarchy::recall::RecallRequest, &CaseState)>,
        content: Option<&crate::conversation::ConversationContentStore>,
    ) -> Result<
        (
            crate::memory_hierarchy::knowledge::KnowledgeResult,
            BTreeSet<String>,
        ),
        String,
    > {
        self.qualified_knowledge_scope_txn(txn, auth, request, state, history,
            recall_input, content, None, false)
    }

    /// Exact paging narrows source/revision/path BEFORE authorization and byte
    /// derivation. Inventory/history iteration is still linear, not O(1).
    pub(super) fn qualified_knowledge_scope_txn<T: Transaction>(
        &self, txn: &T, auth: &AuthenticatedPrincipal,
        request: crate::memory_hierarchy::knowledge::KnowledgeRequest,
        state: &CaseState, history: &[Transition],
        recall_input: Option<(&crate::memory_hierarchy::recall::RecallRequest, &CaseState)>,
        content: Option<&crate::conversation::ConversationContentStore>,
        exact: Option<&BTreeSet<(String, String, String)>>,
        execution_disclosure: bool,
    ) -> Result<(crate::memory_hierarchy::knowledge::KnowledgeResult, BTreeSet<String>), String> {
        use crate::memory_hierarchy::knowledge::{
            self as k, KnowledgeSource, KnowledgeStatus, QualifiedSource,
        };
        let started = std::time::Instant::now();
        let recall = recall_input.map(|(r, _)| r);
        let tenant = state.tenant_id.as_deref().ok_or("case_not_visible")?;
        let security = self
            .resolve_security_context_txn(txn, auth, tenant)
            .map_err(|_| "case_not_visible")?;
        let principal = auth.projected_principal_id();
        if security.require_owner().is_err() {
            if recall.is_some() {
                // Recall's existing H/S inspection scope is not narrowed just
                // because M07 has a stronger Owner-only source contract.
                return Ok((k::derive(&request, &principal, vec![])?, BTreeSet::new()));
            }
            return Err("knowledge_scope_unavailable".into());
        }
        let status = self.materialize_case_policy_txn(txn, &state.case_id)?;
        let cut = if let Some(r) = recall {
            crate::semantic_state::historical::prefix(history, &r.at)?
        } else {
            history
        };
        let then = recall_input.map_or(state, |(_, then)| then);
        let mut inputs = Vec::new();
        let mut visible_backing = BTreeSet::new();
        let mut matched = false;
        for source in &state.sources {
            let d = &source.declaration;
            if exact.is_some_and(|scope| !scope.iter().any(|(s, _, _)| s == &d.source_id)) {
                continue;
            }
            if request
                .source
                .as_ref()
                .is_some_and(|s| *s != d.logical_name && *s != d.source_id)
                || d.declared_by_principal_id != principal
                || recall.is_some_and(|r| {
                    if !execution_disclosure { return r.participant_id != d.participant_id; }
                    // The source owner's CURRENT source permission remains
                    // necessary below. Execution disclosure additionally takes
                    // the intersection with the exact recipient's Resource
                    // envelope. Neither a provider selection nor source roles
                    // transfer the operator's Resource permissions.
                    !state.resources.iter().any(|resource|
                        resource.attachment_id == d.resource_attachment_id
                        && resource.access.as_ref().is_some_and(|a|
                            a.admits_request(&r.participant_id, &ResourceRequest {
                                schema: crate::effect::access::RESOURCE_REQUEST_SCHEMA.into(),
                                configuration_digest: d.configuration_digest.clone(),
                                action: d.action.clone(),
                            }).is_ok()))
                })
                || !d.roles.contains(&SourceRole::Knowledge)
                || !state
                    .principal_participant_links
                    .iter()
                    .any(|l| l.principal_id == principal && l.participant_id == d.participant_id)
                || source
                    .progress
                    .as_ref()
                    .is_none_or(|p| p.phase != SourcePhase::Acquired)
                || !self
                    .source_permission_resolved(state, d, d.action.clone(), &status, history)
                    .is_ok_and(|v| v.outcome == DecisionOutcome::Allow)
            {
                continue;
            }
            let current = source.progress.as_ref().and_then(|p| p.revision.as_ref());
            // Qualify retained acquisition evidence once, even when current D
            // discovery deliberately excludes obsolete documentary revisions.
            // No byte reads/derivations are required for this H visibility gate.
            if recall.is_some() {
                let mut checked = BTreeSet::new();
                // Current disclosure is independent of the requested cut. The
                // historical reader already restricts known-by-then evidence;
                // current S must not lose a currently permitted observation
                // merely because it was admitted after the Recall cut.
                for t in history {
                    if let TransitionPayload::CaseSourceProgressed { progress } = &t.payload {
                        if progress.source_id != d.source_id {
                            continue;
                        }
                        for item in progress.revision.iter().flat_map(|r| &r.items) {
                            let id = crate::memory_hierarchy::recall::documentary::backing_id(
                                &item.backing,
                            );
                            if !checked.insert(id.clone()) {
                                continue;
                            }
                            let admitted = match &item.backing {
                                SourceBacking::Content { admission_id } => self
                                    .source_permission_resolved(
                                        state,
                                        d,
                                        ResourceAction::ContentRead {
                                            admission_id: admission_id.clone(),
                                        },
                                        &status,
                                        history,
                                    )
                                    .is_ok_and(|v| v.outcome == DecisionOutcome::Allow),
                                _ => true,
                            };
                            if admitted {
                                visible_backing.insert(id);
                            }
                        }
                    }
                }
            }
            let revision = if let Some(id) = &request.revision {
                cut.iter().rev().find_map(|t| match &t.payload {
                    TransitionPayload::CaseSourceProgressed { progress }
                        if progress.source_id == d.source_id =>
                    {
                        progress.revision.as_ref().filter(|r| r.revision_id == *id)
                    }
                    _ => None,
                })
            } else if recall.is_some() {
                then.sources
                    .iter()
                    .find(|s| s.declaration.source_id == d.source_id)
                    .and_then(|s| s.progress.as_ref())
                    .filter(|p| p.phase == SourcePhase::Acquired)
                    .and_then(|p| p.revision.as_ref())
            } else {
                current
            };
            let Some(revision) = revision else {
                continue;
            };
            let mut revisions = vec![revision];
            // Ordinary queries never search obsolete revisions. Exact revision
            // anchors opt into their retained history. Opaque unit/source IDs
            // require a bounded qualified historical derivation to resolve.
            if let Some(r) = recall {
                for t in cut {
                    if let TransitionPayload::CaseSourceProgressed { progress } = &t.payload {
                        if progress.source_id != d.source_id
                            || progress.phase != SourcePhase::Acquired
                        {
                            continue;
                        }
                        if let Some(old) = &progress.revision {
                            if (exact.is_some_and(|scope| scope.iter().any(|(s, rev, _)|
                                s == &d.source_id && rev == &old.revision_id)) || r.required_refs.iter().any(|id| {
                                id == &old.revision_id
                                    || id.starts_with("knowledge-unit:")
                                    || id.starts_with("knowledge-source:")
                            })) && !revisions.iter().any(|v| v.revision_id == old.revision_id)
                            {
                                revisions.push(old);
                            }
                        }
                    }
                }
            }
            for revision in revisions {
                if exact.is_some_and(|scope| !scope.iter().any(|(s, r, _)|
                    s == &d.source_id && r == &revision.revision_id)) {
                    continue;
                }
                // Each admitted content object retains its narrower current read
                // permission. Reject the source as a whole before reading bytes.
                if revision.items.iter().any(|item| match &item.backing {
                    SourceBacking::Content { admission_id } => !self
                        .source_permission_resolved(
                            state,
                            d,
                            ResourceAction::ContentRead {
                                admission_id: admission_id.clone(),
                            },
                            &status,
                            history,
                        )
                        .is_ok_and(|v| v.outcome == DecisionOutcome::Allow),
                    _ => false,
                }) {
                    continue;
                }
                matched = true;
                for item in &revision.items {
                    if exact.is_some_and(|scope| !scope.contains(&(
                        d.source_id.clone(), revision.revision_id.clone(), item.path.clone()))) {
                        continue;
                    }
                    let bytes = match &item.backing {
                        SourceBacking::Policy {
                            source_id,
                            artifact_id,
                        } => {
                            self.source_policy_original_txn(txn, tenant, source_id, artifact_id).ok()
                                .map(|s| s.original_bytes().to_vec())
                        }
                        SourceBacking::Content { admission_id } => {
                            let admission = history.iter().find_map(|t| match &t.payload {
                                TransitionPayload::CaseContentAdmitted { admission }
                                    if admission.admission_id == *admission_id
                                        && admission.source_resource_id
                                            == d.resource_attachment_id
                                        && admission.source_configuration_digest
                                            == d.configuration_digest
                                        && admission.source_path == item.path
                                        && admission
                                            .participant_ids
                                            .contains(&d.participant_id) =>
                                {
                                    Some(admission)
                                }
                                _ => None,
                            });
                            admission
                                .and_then(|a| content.and_then(|c| c.read_bytes(&a.object).ok()))
                        }
                        SourceBacking::Observation { observation_id } => {
                            history.iter().find_map(|t| match &t.payload {
                                TransitionPayload::ResourceObservationRecorded { observation }
                                    if observation.observation_id == *observation_id
                                        && observation.resource_attachment_id
                                            == d.resource_attachment_id
                                        && observation.participant_id == d.participant_id
                                        && observation.configuration_digest
                                            == d.configuration_digest =>
                                {
                                    serde_json::to_vec(&observation.result).ok()
                                }
                                _ => None,
                            })
                        }
                    };
                    let source_kind = if let ResourceAction::DatabaseQuery { name } = &d.action {
                        let binding = txn
                            .get(
                                self.local_resource_bindings,
                                &format!("access|{}|{}", state.case_id, d.resource_attachment_id),
                            )
                            .ok()
                            .and_then(|b| {
                                serde_json::from_slice::<crate::effect::access::LocalAccessBinding>(
                                    b,
                                )
                                .ok()
                            });
                        // Only the exact admitted metadata query supplies schema
                        // semantics. Other result JSON retains observation posture.
                        if binding.as_ref().is_some_and(|b| b.digest() == d.configuration_digest
                        && matches!(&b.address, crate::effect::access::ResourceAddress::Sqlite { queries, .. }
                            if queries.get(name).is_some_and(|q| q.split_whitespace()
                                .collect::<Vec<_>>().join(" ").eq_ignore_ascii_case(
                                    "SELECT name, sql FROM sqlite_master WHERE type='table' ORDER BY name")))) {
                        "sqlite_schema_observation"
                    } else {
                        "resource_observation"
                    }
                    } else if matches!(item.backing, SourceBacking::Observation { .. }) {
                        "resource_observation"
                    } else {
                        "document"
                    };
                    let source = KnowledgeSource {
                        id: k::identity(
                            "knowledge-source",
                            &(&d.source_id, &revision.revision_id, &item.path),
                        ),
                        source_id: d.source_id.clone(),
                        logical_name: d.logical_name.clone(),
                        revision_id: revision.revision_id.clone(),
                        current_revision: current
                            .is_some_and(|r| r.revision_id == revision.revision_id),
                        path: item.path.clone(),
                        digest: item.digest.clone(),
                        bytes: item.bytes,
                        backing: item.backing.clone(),
                        roles: d.roles.clone(),
                        resource_id: d.resource_attachment_id.clone(),
                        media_type: d.media_type.clone(),
                        source_kind: source_kind.into(),
                        extractor: k::DERIVATION_PROFILE.into(),
                        extraction_id: String::new(),
                        status: KnowledgeStatus::Qualified,
                        detail: "exact_admitted_backing; no independent reacquisition".into(),
                    };
                    inputs.push(QualifiedSource { source, bytes });
                }
            }
        }
        if request.source.is_some() && !matched {
            return Err("knowledge_source_not_available".into());
        }
        let source_resolution_us = started.elapsed().as_micros();
        let mut result = k::derive(&request, &principal, inputs)?;
        result.measurements.source_resolution_us = source_resolution_us;
        Ok((result, visible_backing))
    }

    pub fn case_source_authorized(
        &self,
        auth: &AuthenticatedPrincipal,
        case: &str,
        name: &str,
    ) -> Result<(CaseState, CaseSourceState), String> {
        let state = self.get_case_state_authorized(auth, case)?;
        self.resolve_security_context(
            auth,
            state.tenant_id.as_deref().ok_or("source_requires_tenant")?,
        )?
        .require_owner()?;
        let source = state
            .sources
            .iter()
            .find(|s| s.declaration.logical_name == name || s.declaration.source_id == name)
            .filter(|s| s.declaration.declared_by_principal_id == auth.projected_principal_id())
            .cloned()
            .ok_or("source_not_visible")?;
        if !state.principal_participant_links.iter().any(|l| {
            l.principal_id == auth.projected_principal_id()
                && l.participant_id == source.declaration.participant_id
        }) {
            return Err("source_not_visible".into());
        }
        Ok((state, source))
    }

    pub fn declare_case_source(
        &self,
        auth: &AuthenticatedPrincipal,
        declaration: CaseSourceDeclaration,
    ) -> Result<CaseState, String> {
        declaration.validate()?;
        let state = self.get_case_state_authorized(auth, &declaration.case_id)?;
        if let Some(old) = state
            .sources
            .iter()
            .find(|s| s.declaration.logical_name == declaration.logical_name)
        {
            if old.declaration != declaration {
                return Err("source_declaration_identity_collision".into());
            }
            self.case_source_authorized(auth, &state.case_id, &declaration.source_id)?;
            return Ok(state);
        }
        let id = declaration.source_id.clone();
        self.commit_source_payload(
            auth,
            &state,
            &declaration,
            &id,
            TransitionPayload::CaseSourceDeclared {
                declaration: declaration.clone(),
            },
        )
    }

    pub fn progress_case_source(
        &self,
        auth: &AuthenticatedPrincipal,
        case: &str,
        name: &str,
        progress: SourceProgress,
    ) -> Result<CaseState, String> {
        let (state, source) = self.case_source_authorized(auth, case, name)?;
        if progress.source_id != source.declaration.source_id {
            return Err("source_progress_scope_mismatch".into());
        }
        if source.progress.as_ref() == Some(&progress) {
            return Ok(state);
        }
        self.commit_source_payload(
            auth,
            &state,
            &source.declaration,
            &progress.progress_id.clone(),
            TransitionPayload::CaseSourceProgressed { progress },
        )
    }

    fn commit_source_payload(
        &self,
        auth: &AuthenticatedPrincipal,
        state: &CaseState,
        declaration: &CaseSourceDeclaration,
        id: &str,
        payload: TransitionPayload,
    ) -> Result<CaseState, String> {
        if declaration.declared_by_principal_id != auth.projected_principal_id() {
            return Err("source_not_visible".into());
        }
        let mut pending = PendingTransition::new(
            format!("transition:{id}"),
            &state.case_id,
            state.generation,
            TransitionSource {
                component: "yai.source_acquisition".into(),
                participant_id: Some(declaration.participant_id.clone()),
                principal_id: Some(auth.projected_principal_id()),
                source_ref: Some(declaration.source_id.clone()),
            },
            payload,
        );
        pending.scope = Some(crate::transition::TransitionScope {
            case_id: state.case_id.clone(),
            participant_refs: vec![declaration.participant_id.clone()],
            resource_refs: vec![declaration.resource_attachment_id.clone()],
            policy_refs: vec![],
        });
        self.commit_secured_transition(
            auth,
            state.tenant_id.as_deref().ok_or("source_requires_tenant")?,
            pending,
            false,
        )
        .map(|c| c.state)
    }

    pub(super) fn validate_source_declaration_txn<T: Transaction>(
        &self,
        txn: &T,
        state: &CaseState,
        d: &CaseSourceDeclaration,
    ) -> Result<(), String> {
        d.validate()?;
        if state.tenant_id.is_none()
            || d.case_id != state.case_id
            || !state.principal_participant_links.iter().any(|l| {
                l.principal_id == d.declared_by_principal_id && l.participant_id == d.participant_id
            })
        {
            return Err("source_current_scope_invalid".into());
        }
        let resource = state
            .resources
            .iter()
            .find(|r| r.attachment_id == d.resource_attachment_id)
            .ok_or("source_resource_unavailable")?;
        resource
            .access
            .as_ref()
            .ok_or("source_access_missing")?
            .admits_request(&d.participant_id, &d.request())?;
        if d.bootstrap_policy {
            self.source_bootstrap_allowed_txn(txn, state, d)?;
        }
        Ok(())
    }

    pub(super) fn source_bootstrap_allowed_txn<T: Transaction>(
        &self,
        txn: &T,
        state: &CaseState,
        d: &CaseSourceDeclaration,
    ) -> Result<(), String> {
        if !d.bootstrap_policy
            || !d.roles.contains(&SourceRole::Policy)
            || !matches!(d.action, ResourceAction::Discover { .. })
            || state
                .sources
                .iter()
                .find(|s| s.declaration.source_id == d.source_id)
                .and_then(|s| s.progress.as_ref())
                .is_some_and(|p| p.phase == SourcePhase::Revoked)
            || self
                .list_case_transitions_txn(txn, &state.case_id)?
                .iter()
                .any(|t| {
                    matches!(
                        t.payload,
                        TransitionPayload::CasePolicyBound { .. }
                            | TransitionPayload::CasePolicyReplaced { .. }
                            | TransitionPayload::CasePolicyUnbound { .. }
                    )
                })
        {
            return Err("source_bootstrap_authority_unavailable".into());
        }
        Ok(())
    }

    /// Narrow setup read: ONLY the exact declared policy file. Original bytes
    /// enter the existing policy catalog, never a second knowledge byte store.
    pub fn acquire_bootstrap_policy_source(
        &self,
        auth: &AuthenticatedPrincipal,
        case: &str,
        name: &str,
    ) -> Result<SourceRevision, String> {
        let (state, source) = self.case_source_authorized(auth, case, name)?;
        let d = &source.declaration;
        let txn = self.env.begin_ro_txn().map_err(|e| e.to_string())?;
        self.source_policy_capture_allowed_txn(&txn, &state, d)?;
        drop(txn);
        let ResourceAction::Discover { path } = &d.action else {
            return Err("bootstrap_policy_requires_exact_file".into());
        };
        let binding = self
            .get_local_access_binding(case, &d.resource_attachment_id)?
            .ok_or("source_binding_unavailable")?;
        if binding.digest() != d.configuration_digest {
            return Err("source_configuration_drift".into());
        }
        let limit = state
            .resources
            .iter()
            .find(|r| r.attachment_id == d.resource_attachment_id)
            .and_then(|r| r.access.as_ref())
            .ok_or("source_access_missing")?
            .max_output_bytes;
        let bytes = read_confined_file(
            binding
                .root()
                .ok_or("bootstrap_policy_requires_filesystem")?,
            path,
            limit,
        )?;
        let tenant_id = state.tenant_id.as_deref().ok_or("source_requires_tenant")?;
        let tenant = self.get_tenant(tenant_id)?.ok_or("tenant_not_visible")?;
        let compiled = if d.media_type.ends_with(";profile=yai-mixed-v1") {
            if !d.roles.contains(&SourceRole::Knowledge) { return Err("mixed_policy_capture_requires_dual_roles".into()); }
            crate::governance::compile_mixed_policy_source(&bytes)?
        } else { compile_policy_source(&bytes)? };
        let compilation = scope_policy_compilation(
            &compiled,
            tenant_id,
            &tenant.organization_ref,
        )?;
        let ingested = self.ingest_policy_compilation_inner(
            &compilation,
            &auth.projected_principal_id(),
            Some(tenant_id),
            Some((case, &d.source_id, state.generation)),
        )?;
        SourceRevision::new(
            &d.source_id,
            vec![SourceRevisionItem {
                path: path.clone(),
                digest: digest_bytes(&bytes),
                bytes: bytes.len() as u64,
                backing: SourceBacking::Policy {
                    source_id: compilation.source.source_id,
                    artifact_id: ingested.view.artifact.artifact_id,
                },
            }],
        )
    }

    /// A read-only decision from the SAME current EffectivePolicy evaluator as
    /// resource admission. Historical ALLOW and bootstrap flags are not reused.
    pub fn case_source_permission(
        &self,
        auth: &AuthenticatedPrincipal,
        case: &str,
        name: &str,
        action: Option<ResourceAction>,
    ) -> Result<Decision, String> {
        let (state, source) = self.case_source_authorized(auth, case, name)?;
        if source
            .progress
            .as_ref()
            .is_some_and(|p| p.phase == SourcePhase::Revoked)
        {
            return Err("source_not_available".into());
        }
        let txn = self.env.begin_ro_txn().map_err(|e| e.to_string())?;
        self.source_permission_txn(
            &txn,
            &state,
            &source.declaration,
            action.unwrap_or(source.declaration.action.clone()),
        )
    }

    fn source_permission_txn<T: Transaction>(
        &self,
        txn: &T,
        state: &CaseState,
        d: &CaseSourceDeclaration,
        action: ResourceAction,
    ) -> Result<Decision, String> {
        let status = self.materialize_case_policy_txn(txn, &state.case_id)?;
        let history = self.list_case_transitions_txn(txn, &state.case_id)?;
        self.source_permission_resolved(state, d, action, &status, &history)
    }

    fn source_permission_resolved(
        &self,
        state: &CaseState,
        d: &CaseSourceDeclaration,
        action: ResourceAction,
        status: &NormativeStatus,
        history: &[Transition],
    ) -> Result<Decision, String> {
        let resource = state
            .resources
            .iter()
            .find(|r| r.attachment_id == d.resource_attachment_id)
            .ok_or("source_not_available")?;
        let request = ResourceRequest {
            schema: crate::effect::access::RESOURCE_REQUEST_SCHEMA.into(),
            configuration_digest: d.configuration_digest.clone(),
            action,
        };
        resource
            .access
            .as_ref()
            .ok_or("source_not_available")?
            .admits_request(&d.participant_id, &request)?;
        let link = state
            .principal_participant_links
            .iter()
            .find(|l| {
                l.participant_id == d.participant_id && l.principal_id == d.declared_by_principal_id
            })
            .ok_or("source_not_available")?;
        let operation = Operation::from_resource_request(
            &state.case_id,
            &d.participant_id,
            &d.resource_attachment_id,
            state.generation,
            request,
            OperationOrigin::ParticipantRequest {
                request_id: format!("source-visibility:{}", d.source_id),
                principal_id: d.declared_by_principal_id.clone(),
                participant_link_id: link.link_id.clone(),
            },
        )?;
        let policy = status
            .effective_policy
            .as_ref()
            .filter(|_| {
                status.readiness == NormativeReadiness::Ready
                    && status.validity == PolicyValidityPosture::Valid
            })
            .ok_or("source_requires_ready_effective_policy")?;
        // For a matching recorded request, preserve ONLY exact canonical
        // provenance. Current policy, roles, scope and validity are still
        // evaluated below. An unrecorded shape keeps empty evidence and any
        // source-provenance obligation therefore denies (never fabricated).
        let operation = history.iter().rev().find_map(|t| match &t.payload {
            TransitionPayload::OperationRecorded { operation: recorded }
                if recorded.participant_id == d.participant_id
                    && recorded.resource_attachment_id == d.resource_attachment_id
                    && recorded.resource_request == operation.resource_request
                    && matches!(&recorded.origin, OperationOrigin::ParticipantRequest { principal_id, participant_link_id, .. }
                        if *principal_id == d.declared_by_principal_id && *participant_link_id == link.link_id) => Some(recorded.clone()),
            _ => None,
        }).unwrap_or(operation);
        let evidence = resolve_canonical_evidence(&operation, history, None)?;
        evaluate_filesystem_admission(
            &operation,
            state,
            resource,
            &policy,
            &evidence,
            &AuthorityTemporalContext {
                authority_time_unix_ms: status.authority_time_unix_ms,
                binding_validity: status.binding_validity.values().cloned().collect(),
            },
        )
    }

    pub(super) fn validate_source_progress_txn<T: Transaction>(
        &self,
        txn: &T,
        state: &CaseState,
        p: &SourceProgress,
    ) -> Result<(), String> {
        p.validate()?;
        let source = state
            .sources
            .iter()
            .find(|s| s.declaration.source_id == p.source_id)
            .ok_or("source_not_declared")?;
        let d = &source.declaration;
        let mut projected = state.sources.clone();
        reduce_progress(&mut projected, p)?;
        if p.phase != SourcePhase::Acquired {
            if p.revision.is_some() {
                return Err("source_nonacquired_payload_forbidden".into());
            }
            if matches!(p.phase, SourcePhase::Denied | SourcePhase::AwaitingReview) {
                let history = self.list_case_transitions_txn(txn, &state.case_id)?;
                let decision = Self::canonical_decision(
                    state,
                    &history,
                    p.decision_ref
                        .as_deref()
                        .ok_or("source_decision_required")?,
                )?;
                let operation = Self::canonical_operation(state, &history, &decision.operation_id)?;
                let request = operation
                    .resource_request
                    .as_ref()
                    .ok_or("source_request_required")?;
                if operation.participant_id != d.participant_id
                    || operation.resource_attachment_id != d.resource_attachment_id
                    || request.configuration_digest != d.configuration_digest
                    || !(request.action == d.action
                        || matches!((&d.action, &request.action),
                        (ResourceAction::Discover { path: declared }, ResourceAction::AdmitContent { path, .. })
                            if path == declared || path.starts_with(&format!("{declared}/"))))
                    || (p.phase == SourcePhase::Denied && decision.outcome != DecisionOutcome::Deny)
                    || (p.phase == SourcePhase::AwaitingReview
                        && decision.outcome != DecisionOutcome::RequireReview)
                {
                    return Err("source_decision_scope_or_outcome_mismatch".into());
                }
            }
            return Ok(());
        }
        let resource = state
            .resources
            .iter()
            .find(|r| r.attachment_id == d.resource_attachment_id)
            .ok_or("source_resource_unavailable")?;
        resource
            .access
            .as_ref()
            .ok_or("source_access_missing")?
            .admits_request(&d.participant_id, &d.request())?;
        if !d.bootstrap_policy
            && self
                .source_permission_txn(txn, state, d, d.action.clone())?
                .outcome
                != DecisionOutcome::Allow
        {
            return Err("source_current_authority_refused".into());
        }
        if d.bootstrap_policy
            && (p.revision.as_ref().map_or(0, |r| r.items.len()) != 1
                || !matches!(
                    p.revision.as_ref().unwrap().items[0].backing,
                    SourceBacking::Policy { .. }
                ))
        {
            return Err("bootstrap_policy_requires_one_exact_original".into());
        }
        let history = self.list_case_transitions_txn(txn, &state.case_id)?;
        if !d.bootstrap_policy {
            let observation = history
                .iter()
                .find_map(|t| match &t.payload {
                    TransitionPayload::ResourceObservationRecorded { observation }
                        if Some(observation.decision_id.as_str()) == p.decision_ref.as_deref()
                            && observation.resource_attachment_id == d.resource_attachment_id
                            && observation.participant_id == d.participant_id
                            && observation.configuration_digest == d.configuration_digest
                            && observation.request_digest == d.request().digest() =>
                    {
                        Some(observation)
                    }
                    _ => None,
                })
                .ok_or("source_exact_acquisition_observation_required")?;
            let items = &p.revision.as_ref().ok_or("source_revision_missing")?.items;
            if matches!(d.action, ResourceAction::Discover { .. }) {
                let entries = observation.result["entries"]
                    .as_array()
                    .ok_or("source_discovery_shape_invalid")?;
                if entries.len() != items.len()
                    || entries.iter().any(|e| {
                        !items.iter().any(|i| {
                            e["path"] == i.path
                                && e["digest"] == i.digest
                                && e["bytes"] == i.bytes
                                && matches!(i.backing, SourceBacking::Content { .. })
                        })
                    })
                {
                    return Err("source_revision_coverage_mismatch".into());
                }
            } else if items.len() != 1
                || !matches!(&items[0].backing, SourceBacking::Observation { observation_id } if *observation_id == observation.observation_id)
            {
                return Err("source_exact_observation_revision_required".into());
            }
        }
        for item in &p.revision.as_ref().ok_or("source_revision_missing")?.items {
            match &item.backing {
                SourceBacking::Policy {
                    source_id,
                    artifact_id,
                } => {
                    self.source_policy_capture_allowed_txn(txn, state, d)?;
                    let original = self.source_policy_original_txn(txn,
                        state.tenant_id.as_deref().ok_or("source_requires_tenant")?, source_id, artifact_id)?;
                    if let ResourceAction::Discover { path } = &d.action {
                        if item.path != *path
                            || digest_bytes(original.original_bytes()) != item.digest
                            || original.original_bytes().len() as u64 != item.bytes
                        {
                            return Err("source_policy_backing_mismatch".into());
                        }
                        // A catalog ID alone does not prove acquisition from
                        // this declared source. Revalidate exact confined bytes.
                        let encoded = txn
                            .get(
                                self.local_resource_bindings,
                                &format!("access|{}|{}", state.case_id, d.resource_attachment_id),
                            )
                            .map_err(|_| "source_binding_unavailable")?;
                        let binding: crate::effect::access::LocalAccessBinding =
                            serde_json::from_slice(encoded).map_err(|e| e.to_string())?;
                        binding.validate()?;
                        if binding.digest() != d.configuration_digest {
                            return Err("source_configuration_drift".into());
                        }
                        let captured = read_confined_file(
                            binding.root().ok_or("bootstrap_requires_filesystem")?,
                            path,
                            resource
                                .access
                                .as_ref()
                                .ok_or("source_access_missing")?
                                .max_output_bytes,
                        )?;
                        if captured != original.original_bytes() {
                            return Err("source_capture_drift".into());
                        }
                    } else {
                        return Err("source_policy_requires_exact_file".into());
                    }
                }
                SourceBacking::Content { admission_id } => {
                    let a = history
                        .iter()
                        .find_map(|t| match &t.payload {
                            TransitionPayload::CaseContentAdmitted { admission }
                                if admission.admission_id == *admission_id =>
                            {
                                Some(admission)
                            }
                            _ => None,
                        })
                        .ok_or("source_backing_unavailable")?;
                    let ResourceAction::Discover { path } = &d.action else {
                        return Err("source_content_route_invalid".into());
                    };
                    if a.source_resource_id != d.resource_attachment_id
                        || a.source_configuration_digest != d.configuration_digest
                        || !a.participant_ids.contains(&d.participant_id)
                        || a.source_path != item.path
                        || !(item.path == *path || item.path.starts_with(&format!("{path}/")))
                        || a.object.content_digest != item.digest
                        || a.object.byte_length != item.bytes
                    {
                        return Err("source_content_backing_mismatch".into());
                    }
                    let decision = self.source_permission_txn(
                        txn,
                        state,
                        d,
                        ResourceAction::AdmitContent {
                            path: item.path.clone(),
                            candidate_digest: item.digest.clone(),
                        },
                    )?;
                    if decision.outcome != DecisionOutcome::Allow {
                        return Err("source_current_authority_refused".into());
                    }
                }
                SourceBacking::Observation { observation_id } => {
                    let o = history
                        .iter()
                        .find_map(|t| match &t.payload {
                            TransitionPayload::ResourceObservationRecorded { observation }
                                if observation.observation_id == *observation_id =>
                            {
                                Some(observation)
                            }
                            _ => None,
                        })
                        .ok_or("source_backing_unavailable")?;
                    let encoded = serde_json::to_vec(&o.result).map_err(|e| e.to_string())?;
                    if o.resource_attachment_id != d.resource_attachment_id
                        || o.participant_id != d.participant_id
                        || o.configuration_digest != d.configuration_digest
                        || o.request_digest != d.request().digest()
                        || digest_bytes(&encoded) != item.digest
                        || encoded.len() as u64 != item.bytes
                    {
                        return Err("source_observation_backing_mismatch".into());
                    }
                    if self
                        .source_permission_txn(txn, state, d, d.action.clone())?
                        .outcome
                        != DecisionOutcome::Allow
                    {
                        return Err("source_current_authority_refused".into());
                    }
                }
            }
        }
        Ok(())
    }
}
