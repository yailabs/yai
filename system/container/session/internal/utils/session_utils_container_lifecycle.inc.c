static int yai_workspace_write_manifest_ws_id(const char *ws_id, const yai_workspace_runtime_info_t *info)
{
    char manifest[MAX_PATH_LEN];
    if (!ws_id || !info)
        return -1;
    if (yai_workspace_manifest_path(ws_id, manifest, sizeof(manifest)) != 0)
        return -1;
    return yai_workspace_write_manifest_path(manifest, info);
}

static int yai_workspace_write_containment_surfaces(const yai_workspace_runtime_info_t *info)
{
    FILE *f;
    char evt_declared[96];
    char evt_business[96];
    char evt_enforcement[96];
    char evt_stage[48];
    char evt_id[224];
    char sink_last_event_ref[96];
    char sink_last_decision_ref[96];
    char sink_last_evidence_ref[96];
    char sink_event_store_ref[MAX_PATH_LEN];
    char sink_decision_store_ref[MAX_PATH_LEN];
    char sink_evidence_store_ref[MAX_PATH_LEN];
    char enforce_last_outcome_ref[128];
    char enforce_last_linkage_ref[128];
    char enforce_materialization_status[48];
    char enforce_missing_fields[192];
    char enforce_outcome_store_ref[MAX_PATH_LEN];
    char enforce_linkage_store_ref[MAX_PATH_LEN];
    char gov_last_object_ref[256];
    char gov_last_lifecycle_ref[256];
    char gov_last_attachment_ref[320];
    char gov_object_store_ref[MAX_PATH_LEN];
    char gov_lifecycle_store_ref[MAX_PATH_LEN];
    char gov_attachment_store_ref[MAX_PATH_LEN];
    char authority_last_ref[192];
    char authority_resolution_ref[192];
    char artifact_last_ref[192];
    char artifact_linkage_ref[192];
    char authority_store_ref[MAX_PATH_LEN];
    char artifact_store_ref[MAX_PATH_LEN];
    char brain_graph_node_ref[192];
    char brain_graph_edge_ref[192];
    char brain_transient_state_ref[192];
    char brain_transient_working_set_ref[192];
    char brain_graph_store_ref[MAX_PATH_LEN];
    char brain_transient_store_ref[MAX_PATH_LEN];
    char op_summary[192];
    const char *review_state;
    int evt_external = 0;
    if (!info || !info->ws_id[0])
        return -1;
    yai_session_workspace_event_semantics(info,
                                          evt_declared, sizeof(evt_declared),
                                          evt_business, sizeof(evt_business),
                                          evt_enforcement, sizeof(evt_enforcement),
                                          evt_stage, sizeof(evt_stage),
                                          &evt_external);
    snprintf(evt_id, sizeof(evt_id), "%s%s",
             info->last_resolution_trace_ref[0] ? "evt-" : "none",
             info->last_resolution_trace_ref[0] ? info->last_resolution_trace_ref : "");
    review_state = yai_workspace_review_state_from_effect(info->last_effect_summary);
    yai_workspace_operational_summary(evt_stage, evt_business, info->last_effect_summary, op_summary, sizeof(op_summary));
    yai_workspace_read_event_evidence_index(info,
                                            sink_last_event_ref,
                                            sizeof(sink_last_event_ref),
                                            sink_last_decision_ref,
                                            sizeof(sink_last_decision_ref),
                                            sink_last_evidence_ref,
                                            sizeof(sink_last_evidence_ref),
                                            sink_event_store_ref,
                                            sizeof(sink_event_store_ref),
                                            sink_decision_store_ref,
                                            sizeof(sink_decision_store_ref),
                                            sink_evidence_store_ref,
                                            sizeof(sink_evidence_store_ref));
    yai_workspace_read_enforcement_index(info,
                                         enforce_last_outcome_ref,
                                         sizeof(enforce_last_outcome_ref),
                                         enforce_last_linkage_ref,
                                         sizeof(enforce_last_linkage_ref),
                                         enforce_materialization_status,
                                         sizeof(enforce_materialization_status),
                                         enforce_missing_fields,
                                         sizeof(enforce_missing_fields),
                                         enforce_outcome_store_ref,
                                         sizeof(enforce_outcome_store_ref),
                                         enforce_linkage_store_ref,
                                         sizeof(enforce_linkage_store_ref));
    yai_workspace_read_governance_index(info,
                                        gov_last_object_ref,
                                        sizeof(gov_last_object_ref),
                                        gov_last_lifecycle_ref,
                                        sizeof(gov_last_lifecycle_ref),
                                        gov_last_attachment_ref,
                                        sizeof(gov_last_attachment_ref),
                                        gov_object_store_ref,
                                        sizeof(gov_object_store_ref),
                                        gov_lifecycle_store_ref,
                                        sizeof(gov_lifecycle_store_ref),
                                        gov_attachment_store_ref,
                                        sizeof(gov_attachment_store_ref));
    yai_workspace_read_authority_artifact_indexes(info,
                                                  authority_last_ref,
                                                  sizeof(authority_last_ref),
                                                  authority_resolution_ref,
                                                  sizeof(authority_resolution_ref),
                                                  artifact_last_ref,
                                                  sizeof(artifact_last_ref),
                                                  artifact_linkage_ref,
                                                  sizeof(artifact_linkage_ref),
                                                  authority_store_ref,
                                                  sizeof(authority_store_ref),
                                                  artifact_store_ref,
                                                  sizeof(artifact_store_ref));
    (void)yai_storage_bridge_last_refs(info->ws_id,
                                            brain_graph_node_ref,
                                            sizeof(brain_graph_node_ref),
                                            brain_graph_edge_ref,
                                            sizeof(brain_graph_edge_ref),
                                            brain_transient_state_ref,
                                            sizeof(brain_transient_state_ref),
                                            brain_transient_working_set_ref,
                                            sizeof(brain_transient_working_set_ref),
                                            brain_graph_store_ref,
                                            sizeof(brain_graph_store_ref),
                                            brain_transient_store_ref,
                                            sizeof(brain_transient_store_ref));

    f = fopen(info->state_surface_path, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.state.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"declared\": {\"family\": \"%s\", \"specialization\": \"%s\", \"source\": \"%s\"},\n"
            "  \"inferred\": {\"family\": \"%s\", \"specialization\": \"%s\", \"confidence\": %.3f},\n"
            "  \"effective\": {\"stack_ref\": \"%s\", \"overlays_ref\": \"%s\", \"effect\": \"%s\", \"authority\": \"%s\", \"evidence\": \"%s\"},\n"
            "  \"governance\": {\"policy_attachments\": \"%s\", \"policy_attachment_count\": %d},\n"
            "  \"event_surface\": {\"event_id\": \"%s\", \"flow_stage\": \"%s\", \"declared_scenario_specialization\": \"%s\", \"business_specialization\": \"%s\", \"enforcement_specialization\": \"%s\", \"external_effect_boundary\": %s},\n"
            "  \"operational_state\": {\"last_event_ref\": \"%s\", \"last_flow_stage\": \"%s\", \"last_business_specialization\": \"%s\", \"last_enforcement_specialization\": \"%s\", \"last_effect\": \"%s\", \"last_authority\": \"%s\", \"last_evidence\": \"%s\", \"last_trace_ref\": \"%s\", \"review_state\": \"%s\", \"operational_summary\": \"%s\"},\n"
            "  \"inspect\": {\"last_summary\": \"%s\", \"last_trace_ref\": \"%s\"},\n"
            "  \"refs\": {\"trace_index\": \"%s\", \"artifact_index\": \"%s\", \"runtime_state\": \"%s\", \"event_store\": \"%s\", \"decision_store\": \"%s\", \"evidence_store\": \"%s\", \"last_event_ref\": \"%s\", \"last_decision_ref\": \"%s\", \"last_evidence_ref\": \"%s\", \"enforcement_outcome_store\": \"%s\", \"enforcement_linkage_store\": \"%s\", \"last_enforcement_outcome_ref\": \"%s\", \"last_enforcement_linkage_ref\": \"%s\", \"enforcement_materialization_status\": \"%s\", \"enforcement_missing_fields\": \"%s\", \"governance_object_store\": \"%s\", \"governance_lifecycle_store\": \"%s\", \"governance_attachment_store\": \"%s\", \"last_governance_object_ref\": \"%s\", \"last_governance_lifecycle_ref\": \"%s\", \"last_governance_attachment_ref\": \"%s\", \"authority_store\": \"%s\", \"artifact_metadata_store\": \"%s\", \"last_authority_ref\": \"%s\", \"last_authority_resolution_ref\": \"%s\", \"last_artifact_ref\": \"%s\", \"last_artifact_linkage_ref\": \"%s\", \"brain_graph_store\": \"%s\", \"brain_transient_store\": \"%s\", \"last_brain_graph_node_ref\": \"%s\", \"last_brain_graph_edge_ref\": \"%s\", \"last_brain_transient_state_ref\": \"%s\", \"last_brain_transient_working_set_ref\": \"%s\"}\n"
            "}\n",
            info->ws_id,
            info->declared_control_family,
            info->declared_specialization,
            info->declared_context_source[0] ? info->declared_context_source : "unset",
            info->inferred_family,
            info->inferred_specialization,
            info->inferred_confidence,
            info->effective_stack_ref,
            info->effective_overlays_ref,
            info->last_effect_summary,
            info->last_authority_summary,
            info->last_evidence_summary,
            info->policy_attachments_csv,
            info->policy_attachment_count,
            evt_id,
            evt_stage,
            evt_declared,
            evt_business,
            evt_enforcement,
            evt_external ? "true" : "false",
            evt_id,
            evt_stage,
            evt_business,
            evt_enforcement,
            info->last_effect_summary,
            info->last_authority_summary,
            info->last_evidence_summary,
            info->last_resolution_trace_ref,
            review_state,
            op_summary,
            info->last_resolution_summary,
            info->last_resolution_trace_ref,
            info->traces_index_path,
            info->artifacts_index_path,
            info->runtime_surface_path,
            sink_event_store_ref,
            sink_decision_store_ref,
            sink_evidence_store_ref,
            sink_last_event_ref[0] ? sink_last_event_ref : evt_id,
            sink_last_decision_ref,
            sink_last_evidence_ref,
            enforce_outcome_store_ref,
            enforce_linkage_store_ref,
            enforce_last_outcome_ref,
            enforce_last_linkage_ref,
            enforce_materialization_status[0] ? enforce_materialization_status : "unknown",
            enforce_missing_fields,
            gov_object_store_ref,
            gov_lifecycle_store_ref,
            gov_attachment_store_ref,
            gov_last_object_ref,
            gov_last_lifecycle_ref,
            gov_last_attachment_ref,
            authority_store_ref,
            artifact_store_ref,
            authority_last_ref,
            authority_resolution_ref,
            artifact_last_ref,
            artifact_linkage_ref,
            brain_graph_store_ref,
            brain_transient_store_ref,
            brain_graph_node_ref,
            brain_graph_edge_ref,
            brain_transient_state_ref,
            brain_transient_working_set_ref);
    fclose(f);

    f = fopen(info->traces_index_path, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.traces.index.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"ownership\": \"workspace-owned\",\n"
            "  \"entries\": [\n"
            "    {\"trace_ref\": \"%s\", \"summary\": \"%s\"}\n"
            "  ]\n"
            "}\n",
            info->ws_id,
            info->last_resolution_trace_ref,
            info->last_resolution_summary);
    fclose(f);

    f = fopen(info->artifacts_index_path, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.artifacts.index.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"ownership\": \"workspace-owned\",\n"
            "  \"entries\": []\n"
            "}\n",
            info->ws_id);
    fclose(f);

    f = fopen(info->runtime_surface_path, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.runtime.state.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"routing\": {\"scope\": \"workspace\", \"namespace\": \"%s\"},\n"
            "  \"attachments\": {\"runtime_attached\": %s, \"control_plane_attached\": %s},\n"
            "  \"security\": {\"level_effective\": \"%s\", \"enforcement_mode\": \"%s\", \"backend_mode\": \"%s\"},\n"
            "  \"execution\": {\"mode_requested\": \"%s\", \"mode_effective\": \"%s\", \"degraded\": %s, \"degraded_reason\": \"%s\", \"unsupported_scopes\": \"%s\"},\n"
            "  \"isolation_mode\": \"%s\",\n"
            "  \"debug_mode\": %s\n"
            "}\n",
            info->ws_id,
            info->workspace_namespace,
            info->runtime_attached ? "true" : "false",
            info->control_plane_attached ? "true" : "false",
            info->security_level_effective[0] ? info->security_level_effective : "logical",
            info->security_enforcement_mode[0] ? info->security_enforcement_mode : "runtime_scoped",
            info->security_backend_mode[0] ? info->security_backend_mode : "none",
            info->execution_mode_requested[0] ? info->execution_mode_requested : "scoped",
            info->execution_mode_effective[0] ? info->execution_mode_effective : "scoped",
            info->execution_mode_degraded ? "true" : "false",
            info->execution_degraded_reason[0] ? info->execution_degraded_reason : "none",
            info->execution_unsupported_scopes[0] ? info->execution_unsupported_scopes : "none",
            info->isolation_mode[0] ? info->isolation_mode : "process",
            info->debug_mode ? "true" : "false");
    fclose(f);

    f = fopen(info->attach_descriptor_ref, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.attach.descriptor.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"binding_scope\": \"session\",\n"
            "  \"runtime_attached\": %s,\n"
            "  \"control_plane_attached\": %s,\n"
            "  \"channel_mode\": \"%s\",\n"
            "  \"artifact_policy_mode\": \"%s\",\n"
            "  \"process_intent\": \"%s\",\n"
            "  \"mode_requested\": \"%s\",\n"
            "  \"mode_effective\": \"%s\"\n"
            "}\n",
            info->ws_id,
            info->runtime_attached ? "true" : "false",
            info->control_plane_attached ? "true" : "false",
            info->channel_mode[0] ? info->channel_mode : "global_control_scoped_route",
            info->artifact_policy_mode[0] ? info->artifact_policy_mode : "workspace_owned",
            info->process_intent[0] ? info->process_intent : "shared_runtime",
            info->execution_mode_requested[0] ? info->execution_mode_requested : "scoped",
            info->execution_mode_effective[0] ? info->execution_mode_effective : "scoped");
    fclose(f);

    f = fopen(info->execution_profile_ref, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.execution.profile.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"mode_requested\": \"%s\",\n"
            "  \"mode_effective\": \"%s\",\n"
            "  \"degraded\": %s,\n"
            "  \"degraded_reason\": \"%s\",\n"
            "  \"unsupported_scopes\": \"%s\",\n"
            "  \"advisory_scopes\": \"%s\",\n"
            "  \"backend_mode\": \"%s\",\n"
            "  \"enforcement_mode\": \"%s\",\n"
            "  \"intents\": {\"network\": \"%s\", \"resource\": \"%s\", \"privilege\": \"%s\"}\n"
            "}\n",
            info->ws_id,
            info->execution_mode_requested[0] ? info->execution_mode_requested : "scoped",
            info->execution_mode_effective[0] ? info->execution_mode_effective : "scoped",
            info->execution_mode_degraded ? "true" : "false",
            info->execution_degraded_reason[0] ? info->execution_degraded_reason : "none",
            info->execution_unsupported_scopes[0] ? info->execution_unsupported_scopes : "none",
            info->execution_advisory_scopes[0] ? info->execution_advisory_scopes : "none",
            info->security_backend_mode[0] ? info->security_backend_mode : "none",
            info->security_enforcement_mode[0] ? info->security_enforcement_mode : "runtime_scoped",
            info->network_intent[0] ? info->network_intent : "advisory_none",
            info->resource_intent[0] ? info->resource_intent : "advisory_none",
            info->privilege_intent[0] ? info->privilege_intent : "inherited_host");
    fclose(f);

    f = fopen(info->binding_state_path, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.binding.state.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"session_binding\": \"%s\",\n"
            "  \"binding_scope\": \"session\",\n"
            "  \"binding_valid\": %s,\n"
            "  \"boundary_reason\": \"%s\"\n"
            "}\n",
            info->ws_id,
            info->session_binding,
            info->namespace_valid ? "true" : "false",
            info->boundary_reason[0] ? info->boundary_reason : "none");
    fclose(f);
    return 0;
}

int yai_session_read_workspace_info(const char *ws_id, yai_workspace_runtime_info_t *out)
{
    char manifest[MAX_PATH_LEN];
    FILE *f;
    char buf[YAI_WS_JSON_IO_CAP];
    size_t r;

    if (!ws_id || !out)
        return -1;

    memset(out, 0, sizeof(*out));
    snprintf(out->ws_id, sizeof(out->ws_id), "%s", ws_id);
    snprintf(out->workspace_namespace, sizeof(out->workspace_namespace), "ws/%s", ws_id);
    out->namespace_valid = 1;
    snprintf(out->boundary_reason, sizeof(out->boundary_reason), "%s", "none");
    snprintf(out->state, sizeof(out->state), "missing");
    snprintf(out->layout, sizeof(out->layout), "v3");
    snprintf(out->workspace_alias, sizeof(out->workspace_alias), "%s", ws_id);
    snprintf(out->isolation_mode, sizeof(out->isolation_mode), "process");
    snprintf(out->root_anchor_mode, sizeof(out->root_anchor_mode), "%s", "managed_default_root");
    snprintf(out->containment_layout, sizeof(out->containment_layout), "%s", "v1");
    out->containment_ready = 0;
    yai_workspace_security_defaults(out);
    if (yai_workspace_store_root_path(out->workspace_store_root, sizeof(out->workspace_store_root)) != 0)
        out->workspace_store_root[0] = '\0';
    if (yai_workspace_runtime_state_root_path(ws_id, out->runtime_state_root, sizeof(out->runtime_state_root)) != 0)
        out->runtime_state_root[0] = '\0';
    if (yai_workspace_metadata_root_path(ws_id, out->metadata_root, sizeof(out->metadata_root)) != 0)
        out->metadata_root[0] = '\0';
    (void)yai_workspace_containment_surface_paths(out);

    if (yai_workspace_manifest_path(ws_id, manifest, sizeof(manifest)) != 0)
        return -1;

    f = fopen(manifest, "r");
    if (!f)
        return -1;

    r = fread(buf, 1, sizeof(buf) - 1, f);
    fclose(f);
    buf[r] = '\0';

    out->exists = 1;
    (void)yai_session_extract_json_string(buf, "state", out->state, sizeof(out->state));
    (void)yai_session_extract_json_string(buf, "layout", out->layout, sizeof(out->layout));
    (void)yai_session_extract_json_string(buf, "containment_layout", out->containment_layout, sizeof(out->containment_layout));
    (void)yai_session_extract_json_string(buf, "security_envelope_version", out->security_envelope_version, sizeof(out->security_envelope_version));
    (void)yai_session_extract_json_string(buf, "security_level_declared", out->security_level_declared, sizeof(out->security_level_declared));
    (void)yai_session_extract_json_string(buf, "security_level_effective", out->security_level_effective, sizeof(out->security_level_effective));
    (void)yai_session_extract_json_string(buf, "security_enforcement_mode", out->security_enforcement_mode, sizeof(out->security_enforcement_mode));
    (void)yai_session_extract_json_string(buf, "security_backend_mode", out->security_backend_mode, sizeof(out->security_backend_mode));
    (void)yai_session_extract_json_string(buf, "execution_mode_requested", out->execution_mode_requested, sizeof(out->execution_mode_requested));
    (void)yai_session_extract_json_string(buf, "execution_mode_effective", out->execution_mode_effective, sizeof(out->execution_mode_effective));
    (void)yai_session_extract_json_string(buf, "execution_degraded_reason", out->execution_degraded_reason, sizeof(out->execution_degraded_reason));
    (void)yai_session_extract_json_string(buf, "execution_unsupported_scopes", out->execution_unsupported_scopes, sizeof(out->execution_unsupported_scopes));
    (void)yai_session_extract_json_string(buf, "execution_advisory_scopes", out->execution_advisory_scopes, sizeof(out->execution_advisory_scopes));
    (void)yai_session_extract_json_string(buf, "process_intent", out->process_intent, sizeof(out->process_intent));
    (void)yai_session_extract_json_string(buf, "channel_mode", out->channel_mode, sizeof(out->channel_mode));
    (void)yai_session_extract_json_string(buf, "artifact_policy_mode", out->artifact_policy_mode, sizeof(out->artifact_policy_mode));
    (void)yai_session_extract_json_string(buf, "network_intent", out->network_intent, sizeof(out->network_intent));
    (void)yai_session_extract_json_string(buf, "resource_intent", out->resource_intent, sizeof(out->resource_intent));
    (void)yai_session_extract_json_string(buf, "privilege_intent", out->privilege_intent, sizeof(out->privilege_intent));
    (void)yai_session_extract_json_string(buf, "attach_descriptor_ref", out->attach_descriptor_ref, sizeof(out->attach_descriptor_ref));
    (void)yai_session_extract_json_string(buf, "execution_profile_ref", out->execution_profile_ref, sizeof(out->execution_profile_ref));
    (void)yai_session_extract_json_string(buf, "root_path", out->root_path, sizeof(out->root_path));
    (void)yai_session_extract_json_string(buf, "workspace_store_root", out->workspace_store_root, sizeof(out->workspace_store_root));
    (void)yai_session_extract_json_string(buf, "runtime_state_root", out->runtime_state_root, sizeof(out->runtime_state_root));
    (void)yai_session_extract_json_string(buf, "metadata_root", out->metadata_root, sizeof(out->metadata_root));
    (void)yai_session_extract_json_string(buf, "state_root", out->state_root, sizeof(out->state_root));
    (void)yai_session_extract_json_string(buf, "traces_root", out->traces_root, sizeof(out->traces_root));
    (void)yai_session_extract_json_string(buf, "artifacts_root", out->artifacts_root, sizeof(out->artifacts_root));
    (void)yai_session_extract_json_string(buf, "runtime_local_root", out->runtime_local_root, sizeof(out->runtime_local_root));
    (void)yai_session_extract_json_string(buf, "state_surface", out->state_surface_path, sizeof(out->state_surface_path));
    (void)yai_session_extract_json_string(buf, "traces_index", out->traces_index_path, sizeof(out->traces_index_path));
    (void)yai_session_extract_json_string(buf, "artifacts_index", out->artifacts_index_path, sizeof(out->artifacts_index_path));
    (void)yai_session_extract_json_string(buf, "runtime_surface", out->runtime_surface_path, sizeof(out->runtime_surface_path));
    (void)yai_session_extract_json_string(buf, "binding_surface", out->binding_state_path, sizeof(out->binding_state_path));
    (void)yai_session_extract_json_string(buf, "root_anchor_mode", out->root_anchor_mode, sizeof(out->root_anchor_mode));
    (void)yai_session_extract_json_string(buf, "workspace_alias", out->workspace_alias, sizeof(out->workspace_alias));
    (void)yai_session_extract_json_string(buf, "session_binding", out->session_binding, sizeof(out->session_binding));
    (void)yai_session_extract_json_string(buf, "declared_control_family", out->declared_control_family, sizeof(out->declared_control_family));
    (void)yai_session_extract_json_string(buf, "declared_specialization", out->declared_specialization, sizeof(out->declared_specialization));
    (void)yai_session_extract_json_string(buf, "declared_context_source", out->declared_context_source, sizeof(out->declared_context_source));
    (void)yai_session_extract_json_string(buf, "last_inferred_family", out->inferred_family, sizeof(out->inferred_family));
    (void)yai_session_extract_json_string(buf, "last_inferred_specialization", out->inferred_specialization, sizeof(out->inferred_specialization));
    out->inferred_confidence = 0.0;
    (void)yai_session_extract_json_double(buf, "last_inference_confidence", &out->inferred_confidence);
    (void)yai_session_extract_json_string(buf, "effective_stack_ref", out->effective_stack_ref, sizeof(out->effective_stack_ref));
    (void)yai_session_extract_json_string(buf, "effective_overlays_ref", out->effective_overlays_ref, sizeof(out->effective_overlays_ref));
    (void)yai_session_extract_json_string(buf, "policy_attachments", out->policy_attachments_csv, sizeof(out->policy_attachments_csv));
    out->policy_attachment_count = yai_policy_attachment_csv_count(out->policy_attachments_csv);
    (void)yai_session_extract_json_string(buf, "last_effect_summary", out->last_effect_summary, sizeof(out->last_effect_summary));
    (void)yai_session_extract_json_string(buf, "last_authority_summary", out->last_authority_summary, sizeof(out->last_authority_summary));
    (void)yai_session_extract_json_string(buf, "last_evidence_summary", out->last_evidence_summary, sizeof(out->last_evidence_summary));
    (void)yai_session_extract_json_string(buf, "last_resolution_summary", out->last_resolution_summary, sizeof(out->last_resolution_summary));
    (void)yai_session_extract_json_string(buf, "isolation_mode", out->isolation_mode, sizeof(out->isolation_mode));
    (void)yai_session_extract_json_string(buf, "last_resolution_trace_ref", out->last_resolution_trace_ref, sizeof(out->last_resolution_trace_ref));
    (void)yai_session_extract_json_long(buf, "created_at", &out->created_at);
    (void)yai_session_extract_json_long(buf, "activated_at", &out->activated_at);
    (void)yai_session_extract_json_long(buf, "last_attached_at", &out->last_attached_at);
    if (yai_session_extract_json_long(buf, "updated_at", &out->updated_at) != 0)
        out->updated_at = out->created_at;
    (void)yai_session_extract_json_bool(buf, "runtime_attached", &out->runtime_attached);
    (void)yai_session_extract_json_bool(buf, "control_plane_attached", &out->control_plane_attached);
    (void)yai_session_extract_json_bool(buf, "debug_mode", &out->debug_mode);
    (void)yai_session_extract_json_bool(buf, "scope_process", &out->scope_process);
    (void)yai_session_extract_json_bool(buf, "scope_filesystem", &out->scope_filesystem);
    (void)yai_session_extract_json_bool(buf, "scope_socket", &out->scope_socket);
    (void)yai_session_extract_json_bool(buf, "scope_network", &out->scope_network);
    (void)yai_session_extract_json_bool(buf, "scope_resource", &out->scope_resource);
    (void)yai_session_extract_json_bool(buf, "scope_privilege", &out->scope_privilege);
    (void)yai_session_extract_json_bool(buf, "scope_runtime_route", &out->scope_runtime_route);
    (void)yai_session_extract_json_bool(buf, "scope_binding", &out->scope_binding);
    (void)yai_session_extract_json_bool(buf, "execution_mode_degraded", &out->execution_mode_degraded);
    (void)yai_session_extract_json_bool(buf, "capability_sandbox_ready", &out->capability_sandbox_ready);
    (void)yai_session_extract_json_bool(buf, "capability_hardened_fs", &out->capability_hardened_fs);
    (void)yai_session_extract_json_bool(buf, "capability_process_isolation", &out->capability_process_isolation);
    (void)yai_session_extract_json_bool(buf, "capability_network_policy", &out->capability_network_policy);
    {
        const yai_runtime_capability_state_t *caps = yai_runtime_capabilities_state();
        if (caps &&
            yai_runtime_capabilities_is_ready() &&
            caps->workspace_id[0] &&
            strcmp(caps->workspace_id, ws_id) == 0)
        {
            out->runtime_attached = 1;
            out->control_plane_attached = 1;
        }
    }

    if (out->root_path[0] == '\0')
    {
        if (out->workspace_store_root[0])
            snprintf(out->root_path, sizeof(out->root_path), "%s/%s", out->workspace_store_root, ws_id);
    }
    out->containment_ready = yai_session_path_exists(out->state_surface_path) &&
                             yai_session_path_exists(out->traces_index_path) &&
                             yai_session_path_exists(out->artifacts_index_path) &&
                             yai_session_path_exists(out->runtime_surface_path) &&
                             yai_session_path_exists(out->binding_state_path) &&
                             yai_session_path_exists(out->attach_descriptor_ref) &&
                             yai_session_path_exists(out->execution_profile_ref);
    if (!out->containment_ready)
    {
        out->namespace_valid = 0;
        if (strcmp(out->boundary_reason, "none") == 0)
            snprintf(out->boundary_reason, sizeof(out->boundary_reason), "%s", "containment_surface_missing");
    }
    if (!yai_workspace_security_level_is_valid(out->security_level_declared))
        snprintf(out->security_level_declared, sizeof(out->security_level_declared), "%s", "scoped");
    yai_workspace_security_recompute_effective(out);

    if (out->session_binding[0] && strcmp(out->session_binding, ws_id) != 0)
    {
        out->namespace_valid = 0;
        snprintf(out->boundary_reason, sizeof(out->boundary_reason), "%s", "session_binding_mismatch");
        out->session_binding[0] = '\0';
    }
    {
        char expected_runtime_root[MAX_PATH_LEN];
        char expected_metadata_root[MAX_PATH_LEN];
        if (yai_workspace_runtime_state_root_path(ws_id, expected_runtime_root, sizeof(expected_runtime_root)) == 0)
        {
            if (!yai_is_ws_runtime_path_valid(ws_id, out->runtime_state_root, expected_runtime_root))
            {
                out->namespace_valid = 0;
                if (strcmp(out->boundary_reason, "none") == 0)
                    snprintf(out->boundary_reason, sizeof(out->boundary_reason), "%s", "runtime_state_root_mismatch");
                snprintf(out->runtime_state_root, sizeof(out->runtime_state_root), "%s", expected_runtime_root);
            }
        }
        if (yai_workspace_metadata_root_path(ws_id, expected_metadata_root, sizeof(expected_metadata_root)) == 0)
        {
            if (!yai_is_ws_runtime_path_valid(ws_id, out->metadata_root, expected_metadata_root))
            {
                out->namespace_valid = 0;
                if (strcmp(out->boundary_reason, "none") == 0)
                    snprintf(out->boundary_reason, sizeof(out->boundary_reason), "%s", "metadata_root_mismatch");
                snprintf(out->metadata_root, sizeof(out->metadata_root), "%s", expected_metadata_root);
            }
        }
    }
    yai_workspace_fill_shell_relation(out);

    return 0;
}

int yai_session_build_workspace_list_json(char *out, size_t out_cap, int *count_out)
{
    char run_dir[MAX_PATH_LEN];
    DIR *d;
    size_t used = 0;
    int first = 1;
    int count = 0;
    struct dirent *ent;
    int n;

    if (!out || out_cap == 0)
        return -1;
    if (count_out)
        *count_out = 0;

    if (yai_session_build_run_path(run_dir, sizeof(run_dir), "") != 0)
        return -1;

    d = opendir(run_dir);
    if (!d)
        return -1;

    n = snprintf(out, out_cap, "[");
    if (n <= 0 || (size_t)n >= out_cap)
    {
        closedir(d);
        return -1;
    }
    used = (size_t)n;

    while ((ent = readdir(d)) != NULL)
    {
        yai_workspace_runtime_info_t info;
        if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0)
            continue;
        if (yai_session_read_workspace_info(ent->d_name, &info) != 0 || !info.exists)
            continue;

        n = snprintf(out + used,
                     out_cap - used,
                     "%s{\"ws_id\":\"%s\",\"workspace_alias\":\"%s\",\"state\":\"%s\",\"root_path\":\"%s\",\"runtime_attached\":%s}",
                     first ? "" : ",",
                     info.ws_id,
                     info.workspace_alias,
                     info.state,
                     info.root_path,
                     info.runtime_attached ? "true" : "false");
        if (n <= 0 || (size_t)n >= (out_cap - used))
        {
            closedir(d);
            return -1;
        }

        used += (size_t)n;
        first = 0;
        count++;
    }

    closedir(d);

    n = snprintf(out + used, out_cap - used, "]");
    if (n <= 0 || (size_t)n >= (out_cap - used))
        return -1;

    if (count_out)
        *count_out = count;
    return 0;
}

static int yai_workspace_require_path_exists(const char *path,
                                             const char *reason,
                                             char *err,
                                             size_t err_cap)
{
    if (!path || !path[0] || yai_session_path_exists(path) == 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", reason ? reason : "workspace_path_missing");
        return -1;
    }
    return 0;
}

static int yai_workspace_verify_bound_postconditions(const char *ws_id,
                                                     const yai_workspace_runtime_info_t *info,
                                                     char *err,
                                                     size_t err_cap)
{
    const yai_runtime_capability_state_t *caps;
    const char *store_root;
    char store_ws_root[MAX_PATH_LEN];
    char store_data[MAX_PATH_LEN];
    char store_graph[MAX_PATH_LEN];
    char store_knowledge[MAX_PATH_LEN];
    char store_transient[MAX_PATH_LEN];

    if (err && err_cap > 0)
        err[0] = '\0';
    if (!ws_id || !ws_id[0] || !info)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_postcondition_invalid_input");
        return -1;
    }

    if (!info->exists)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_postcondition_manifest_missing");
        return -1;
    }
    if (!info->namespace_valid)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_postcondition_namespace_invalid");
        return -1;
    }
    if (!info->containment_ready)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_postcondition_containment_missing");
        return -1;
    }
    if (!info->runtime_attached || !info->control_plane_attached)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_postcondition_runtime_attachment_missing");
        return -1;
    }

    if (yai_workspace_require_path_exists(info->state_surface_path,
                                          "workspace_postcondition_state_surface_missing",
                                          err,
                                          err_cap) != 0 ||
        yai_workspace_require_path_exists(info->runtime_surface_path,
                                          "workspace_postcondition_runtime_surface_missing",
                                          err,
                                          err_cap) != 0 ||
        yai_workspace_require_path_exists(info->binding_state_path,
                                          "workspace_postcondition_binding_surface_missing",
                                          err,
                                          err_cap) != 0)
    {
        return -1;
    }

    store_root = yai_data_store_binding_root();
    if (!store_root || !store_root[0])
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_postcondition_store_binding_uninitialized");
        return -1;
    }
    if (snprintf(store_ws_root, sizeof(store_ws_root), "%s/%s", store_root, ws_id) <= 0 ||
        snprintf(store_data, sizeof(store_data), "%s/data", store_ws_root) <= 0 ||
        snprintf(store_graph, sizeof(store_graph), "%s/graph", store_ws_root) <= 0 ||
        snprintf(store_knowledge, sizeof(store_knowledge), "%s/knowledge", store_ws_root) <= 0 ||
        snprintf(store_transient, sizeof(store_transient), "%s/transient", store_ws_root) <= 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_postcondition_store_path_format_failed");
        return -1;
    }
    if (yai_workspace_require_path_exists(store_ws_root,
                                          "workspace_postcondition_store_workspace_root_missing",
                                          err,
                                          err_cap) != 0 ||
        yai_workspace_require_path_exists(store_data,
                                          "workspace_postcondition_data_root_missing",
                                          err,
                                          err_cap) != 0 ||
        yai_workspace_require_path_exists(store_graph,
                                          "workspace_postcondition_graph_root_missing",
                                          err,
                                          err_cap) != 0 ||
        yai_workspace_require_path_exists(store_knowledge,
                                          "workspace_postcondition_knowledge_root_missing",
                                          err,
                                          err_cap) != 0 ||
        yai_workspace_require_path_exists(store_transient,
                                          "workspace_postcondition_transient_root_missing",
                                          err,
                                          err_cap) != 0)
    {
        return -1;
    }

    if (yai_runtime_capabilities_is_ready() == 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_postcondition_runtime_not_ready");
        return -1;
    }

    caps = yai_runtime_capabilities_state();
    if (!caps || !caps->workspace_id[0] || strcmp(caps->workspace_id, ws_id) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_postcondition_runtime_workspace_mismatch");
        return -1;
    }
    return 0;
}

int yai_session_handle_workspace_action(
    const char *ws_id,
    const char *action,
    const char *root_path_opt,
    const char *security_level_opt,
    char *err,
    size_t err_cap,
    yai_workspace_runtime_info_t *info_out)
{
    const char *home = yai_get_home();
    char yai_dir[MAX_PATH_LEN];
    char run_dir[MAX_PATH_LEN];
    char ws_dir[MAX_PATH_LEN];
    char auth_dir[MAX_PATH_LEN];
    char events_dir[MAX_PATH_LEN];
    char exec_dir[MAX_PATH_LEN];
    char logs_dir[MAX_PATH_LEN];
    char metadata_dir[MAX_PATH_LEN];
    char state_dir[MAX_PATH_LEN];
    char traces_dir[MAX_PATH_LEN];
    char artifacts_dir[MAX_PATH_LEN];
    char runtime_dir[MAX_PATH_LEN];
    char governance_dir[MAX_PATH_LEN];
    char root_path[MAX_PATH_LEN] = {0};
    char root_anchor_mode[32] = {0};
    char manifest_path[MAX_PATH_LEN];
    char bind_err[96];
    yai_workspace_runtime_info_t info;
    time_t now = time(NULL);

    if (err && err_cap > 0)
        err[0] = '\0';
    if (!home || !ws_id || !action)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_action_invalid_args");
        return -1;
    }
    if (!yai_ws_id_is_valid(ws_id))
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_id_invalid");
        return -1;
    }

    if (snprintf(yai_dir, sizeof(yai_dir), "%s/.yai", home) <= 0 ||
        snprintf(run_dir, sizeof(run_dir), "%s/.yai/run", home) <= 0 ||
        snprintf(ws_dir, sizeof(ws_dir), "%s/.yai/run/%s", home, ws_id) <= 0 ||
        snprintf(auth_dir, sizeof(auth_dir), "%s/authority", ws_dir) <= 0 ||
        snprintf(events_dir, sizeof(events_dir), "%s/events", ws_dir) <= 0 ||
        snprintf(exec_dir, sizeof(exec_dir), "%s/exec", ws_dir) <= 0 ||
        snprintf(logs_dir, sizeof(logs_dir), "%s/logs", ws_dir) <= 0 ||
        snprintf(metadata_dir, sizeof(metadata_dir), "%s/metadata", ws_dir) <= 0 ||
        snprintf(state_dir, sizeof(state_dir), "%s/state", ws_dir) <= 0 ||
        snprintf(traces_dir, sizeof(traces_dir), "%s/traces", ws_dir) <= 0 ||
        snprintf(artifacts_dir, sizeof(artifacts_dir), "%s/artifacts", ws_dir) <= 0 ||
        snprintf(runtime_dir, sizeof(runtime_dir), "%s/runtime", ws_dir) <= 0 ||
        snprintf(governance_dir, sizeof(governance_dir), "%s/governance", ws_dir) <= 0)
        return -1;

    if (strcmp(action, "destroy") == 0)
    {
        if (info_out)
        {
            (void)yai_session_read_workspace_info(ws_id, info_out);
            snprintf(info_out->ws_id, sizeof(info_out->ws_id), "%s", ws_id);
            info_out->exists = 0;
            snprintf(info_out->state, sizeof(info_out->state), "destroyed");
            info_out->updated_at = (long)now;
        }
        return remove_tree(ws_dir);
    }

    if (strcmp(action, "reset") == 0)
    {
        (void)remove_tree(ws_dir);
        action = "create";
    }

    if (strcmp(action, "create") != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_action_unsupported");
        return -2;
    }

    if (yai_workspace_resolve_root_path(ws_id,
                                        root_path_opt,
                                        root_anchor_mode,
                                        sizeof(root_anchor_mode),
                                        root_path,
                                        sizeof(root_path)) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_root_resolution_failed");
        return -1;
    }

    if (mkdir_if_missing(yai_dir, 0755) != 0 ||
        mkdir_if_missing(run_dir, 0755) != 0 ||
        mkdir_if_missing(ws_dir, 0755) != 0 ||
        mkdir_if_missing(auth_dir, 0755) != 0 ||
        mkdir_if_missing(events_dir, 0755) != 0 ||
        mkdir_if_missing(exec_dir, 0755) != 0 ||
        mkdir_if_missing(logs_dir, 0755) != 0 ||
        mkdir_if_missing(metadata_dir, 0755) != 0 ||
        mkdir_if_missing(state_dir, 0755) != 0 ||
        mkdir_if_missing(traces_dir, 0755) != 0 ||
        mkdir_if_missing(artifacts_dir, 0755) != 0 ||
        mkdir_if_missing(runtime_dir, 0755) != 0 ||
        mkdir_if_missing(governance_dir, 0755) != 0 ||
        mkdir_parents(root_path, 0755) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_runtime_roots_init_failed");
        return -1;
    }

    memset(&info, 0, sizeof(info));
    snprintf(info.ws_id, sizeof(info.ws_id), "%s", ws_id);
    snprintf(info.workspace_alias, sizeof(info.workspace_alias), "%s", ws_id);
    snprintf(info.state, sizeof(info.state), "%s", "created");
    snprintf(info.layout, sizeof(info.layout), "%s", "v3");
    snprintf(info.containment_layout, sizeof(info.containment_layout), "%s", "v1");
    yai_workspace_security_defaults(&info);
    if (security_level_opt && security_level_opt[0])
    {
        if (!yai_workspace_security_level_is_valid(security_level_opt))
        {
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", "workspace_security_level_invalid");
            return -1;
        }
        snprintf(info.security_level_declared, sizeof(info.security_level_declared), "%s", security_level_opt);
    }
    snprintf(info.root_path, sizeof(info.root_path), "%s", root_path);
    snprintf(info.root_anchor_mode, sizeof(info.root_anchor_mode), "%s", root_anchor_mode[0] ? root_anchor_mode : "managed_default_root");
    if (yai_workspace_store_root_path(info.workspace_store_root, sizeof(info.workspace_store_root)) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_store_root_resolution_failed");
        return -1;
    }
    if (yai_workspace_runtime_state_root_path(ws_id, info.runtime_state_root, sizeof(info.runtime_state_root)) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_runtime_state_root_resolution_failed");
        return -1;
    }
    if (yai_workspace_metadata_root_path(ws_id, info.metadata_root, sizeof(info.metadata_root)) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_metadata_root_resolution_failed");
        return -1;
    }
    if (yai_workspace_containment_surface_paths(&info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_containment_path_resolution_failed");
        return -1;
    }
    snprintf(info.isolation_mode, sizeof(info.isolation_mode), "%s", "process");
    info.created_at = (long)now;
    info.updated_at = (long)now;
    info.exists = 1;
    info.namespace_valid = 1;
    info.containment_ready = 1;
    snprintf(info.workspace_namespace, sizeof(info.workspace_namespace), "ws/%s", ws_id);
    snprintf(info.boundary_reason, sizeof(info.boundary_reason), "%s", "none");
    yai_workspace_security_recompute_effective(&info);

    if (snprintf(manifest_path, sizeof(manifest_path), "%s/manifest.json", ws_dir) <= 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_manifest_path_format_failed");
        return -1;
    }
    if (yai_workspace_write_manifest_path(manifest_path, &info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_manifest_write_failed");
        return -1;
    }
    if (yai_workspace_write_containment_surfaces(&info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_containment_write_failed");
        return -1;
    }

    /* Workspace-first foundation: create performs real capability binding bootstrap. */
    if (yai_workspace_recover_runtime_capabilities(ws_id, bind_err, sizeof(bind_err)) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_capability_binding_failed");
        return -1;
    }
    if (yai_data_store_init_scope(ws_id, bind_err, sizeof(bind_err)) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_store_init_failed");
        return -1;
    }

    info.runtime_attached = 1;
    info.control_plane_attached = 1;
    info.last_attached_at = (long)time(NULL);
    if (yai_workspace_write_manifest_path(manifest_path, &info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_manifest_update_failed");
        return -1;
    }
    if (yai_workspace_write_containment_surfaces(&info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_containment_update_failed");
        return -1;
    }
    if (yai_session_read_workspace_info(ws_id, &info) != 0 || !info.exists)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_manifest_reload_failed");
        return -1;
    }
    if (yai_workspace_verify_bound_postconditions(ws_id, &info, bind_err, sizeof(bind_err)) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_postcondition_failed");
        return -1;
    }

    if (info_out)
    {
        *info_out = info;
        yai_workspace_fill_shell_relation(info_out);
    }

    return 0;
}

int yai_session_set_active_workspace(const char *ws_id, char *err, size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    char bind_err[96];
    const char *why = "ok";

    if (err && err_cap > 0)
        err[0] = '\0';

    if (!ws_id || !ws_id[0] || !yai_ws_id_is_valid(ws_id))
    {
        why = "invalid_workspace_id";
        goto fail;
    }

    if (yai_session_read_workspace_info(ws_id, &info) != 0 || !info.exists)
    {
        why = "workspace_not_found";
        goto fail;
    }

    if (yai_workspace_binding_write(ws_id, info.workspace_alias) != 0)
    {
        why = "binding_write_failed";
        goto fail;
    }

    if (yai_workspace_recover_runtime_capabilities(ws_id, bind_err, sizeof(bind_err)) != 0)
    {
        why = bind_err[0] ? bind_err : "workspace_capability_binding_failed";
        goto fail;
    }
    if (yai_data_store_init_scope(ws_id, bind_err, sizeof(bind_err)) != 0)
    {
        why = bind_err[0] ? bind_err : "workspace_store_init_failed";
        goto fail;
    }

    snprintf(info.session_binding, sizeof(info.session_binding), "%s", ws_id);
    info.runtime_attached = 1;
    info.control_plane_attached = 1;
    info.last_attached_at = (long)time(NULL);
    info.activated_at = (long)time(NULL);
    info.updated_at = info.activated_at;
    if (yai_workspace_write_manifest_ws_id(ws_id, &info) != 0)
    {
        why = "manifest_write_failed";
        goto fail;
    }
    if (yai_workspace_write_containment_surfaces(&info) != 0)
    {
        why = "containment_write_failed";
        goto fail;
    }
    if (yai_session_read_workspace_info(ws_id, &info) != 0 || !info.exists)
    {
        why = "workspace_manifest_reload_failed";
        goto fail;
    }
    if (yai_workspace_verify_bound_postconditions(ws_id, &info, bind_err, sizeof(bind_err)) != 0)
    {
        why = bind_err[0] ? bind_err : "workspace_postcondition_failed";
        goto fail;
    }

    /* Best-effort shell integration bootstrap.
     * Default mode is no-op; managed mode is explicit opt-in via
     * YAI_SHELL_INTEGRATION_MODE=managed. */
    (void)yai_session_ensure_shell_integration();

    return 0;

fail:
    if (err && err_cap > 0)
        snprintf(err, err_cap, "%s", why);
    return -1;
}

int yai_session_clear_active_workspace(void)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    char path[MAX_PATH_LEN];

    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err)) == 0 &&
        strcmp(status, "active") == 0)
    {
        info.session_binding[0] = '\0';
        info.updated_at = (long)time(NULL);
        (void)yai_workspace_write_manifest_ws_id(info.ws_id, &info);
        (void)yai_workspace_write_containment_surfaces(&info);
    }

    if (yai_workspace_binding_path(path, sizeof(path)) != 0)
        return -1;
    if (unlink(path) != 0 && errno != ENOENT)
        return -1;
    return 0;
}

int yai_session_clear_workspace_runtime_state(char *out_ws_id, size_t out_ws_id_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];

    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err)) != 0 ||
        strcmp(status, "active") != 0)
        return -1;

    info.inferred_family[0] = '\0';
    info.inferred_specialization[0] = '\0';
    info.inferred_confidence = 0.0;
    info.effective_stack_ref[0] = '\0';
    info.effective_overlays_ref[0] = '\0';
    info.last_effect_summary[0] = '\0';
    info.last_authority_summary[0] = '\0';
    info.last_evidence_summary[0] = '\0';
    info.last_resolution_summary[0] = '\0';
    info.last_resolution_trace_ref[0] = '\0';
    info.runtime_attached = 0;
    info.control_plane_attached = 0;
    info.updated_at = (long)time(NULL);

    if (yai_workspace_write_manifest_ws_id(info.ws_id, &info) != 0)
        return -1;
    if (yai_workspace_write_containment_surfaces(&info) != 0)
        return -1;

    if (out_ws_id && out_ws_id_cap > 0)
        snprintf(out_ws_id, out_ws_id_cap, "%s", info.ws_id);
    return 0;
}

static int yai_workspace_resolve_from_cwd(yai_workspace_runtime_info_t *best_out)
{
    char run_dir[MAX_PATH_LEN];
    char cwd[MAX_PATH_LEN];
    DIR *d;
    struct dirent *ent;
    size_t best_len = 0;

    if (!best_out)
        return -1;
    memset(best_out, 0, sizeof(*best_out));

    if (!getcwd(cwd, sizeof(cwd)))
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), "") != 0)
        return -1;

    d = opendir(run_dir);
    if (!d)
        return -1;

    while ((ent = readdir(d)) != NULL)
    {
        yai_workspace_runtime_info_t info;
        size_t root_len;

        if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0)
            continue;
        if (yai_session_read_workspace_info(ent->d_name, &info) != 0 || !info.exists)
            continue;
        if (!info.root_path[0] || !yai_path_is_under(info.root_path, cwd))
            continue;

        root_len = strlen(info.root_path);
        if (root_len >= best_len)
        {
            best_len = root_len;
            *best_out = info;
        }
    }

    closedir(d);
    return best_len > 0 ? 0 : -1;
}

int yai_session_resolve_current_workspace(yai_workspace_runtime_info_t *info_out,
                                          char *status_out,
                                          size_t status_cap,
                                          char *err,
                                          size_t err_cap)
{
    char ws_id[MAX_WS_ID_LEN] = {0};
    char ws_alias[64] = {0};
    const char *env_ws = getenv("YAI_ACTIVE_WORKSPACE");
    int resolved_by_cwd = 0;

    if (!info_out || !status_out || status_cap == 0)
        return -1;
    memset(info_out, 0, sizeof(*info_out));
    status_out[0] = '\0';
    if (err && err_cap > 0)
        err[0] = '\0';

    if (env_ws && env_ws[0])
    {
        if (!yai_ws_id_is_valid(env_ws))
        {
            snprintf(status_out, status_cap, "%s", "invalid");
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", "env_binding_invalid");
            return -1;
        }
        snprintf(ws_id, sizeof(ws_id), "%s", env_ws);
        snprintf(ws_alias, sizeof(ws_alias), "%s", env_ws);
    }
    else
    {
        if (yai_workspace_resolve_from_cwd(info_out) == 0)
        {
            resolved_by_cwd = 1;
        }
        else
        {
            if (yai_workspace_binding_read(ws_id, sizeof(ws_id), ws_alias, sizeof(ws_alias)) != 0)
            {
                snprintf(status_out, status_cap, "%s", "no_active");
                return 0;
            }
            if (!yai_ws_id_is_valid(ws_id))
            {
                snprintf(status_out, status_cap, "%s", "invalid");
                if (err && err_cap > 0)
                    snprintf(err, err_cap, "%s", "binding_workspace_id_invalid");
                return -1;
            }
        }
    }

    if (!resolved_by_cwd &&
        (yai_session_read_workspace_info(ws_id, info_out) != 0 || !info_out->exists))
    {
        snprintf(status_out, status_cap, "%s", "stale");
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "binding_workspace_missing");
        return -1;
    }

    if (info_out->workspace_alias[0] == '\0' && ws_alias[0] != '\0')
        snprintf(info_out->workspace_alias, sizeof(info_out->workspace_alias), "%s", ws_alias);
    if (!info_out->namespace_valid)
    {
        snprintf(status_out, status_cap, "%s", "invalid");
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", info_out->boundary_reason[0] ? info_out->boundary_reason : "workspace_namespace_invalid");
        return -1;
    }
    snprintf(status_out, status_cap, "%s", "active");
    return 0;
}
