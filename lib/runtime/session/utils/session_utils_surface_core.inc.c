int yai_session_enforce_workspace_scope(const char *target_ws_id,
                                        char *err,
                                        size_t err_cap)
{
    yai_workspace_runtime_info_t current;
    char status[24];
    char bind_err[96];

    if (err && err_cap > 0)
        err[0] = '\0';
    if (!target_ws_id || !target_ws_id[0] || !yai_ws_id_is_valid(target_ws_id))
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "target_workspace_invalid");
        return -1;
    }

    if (yai_session_resolve_current_workspace(&current, status, sizeof(status), bind_err, sizeof(bind_err)) != 0)
    {
        if (strcmp(status, "no_active") == 0)
            return 0;
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_binding_invalid");
        return -1;
    }

    if (strcmp(current.ws_id, target_ws_id) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "cross_workspace_scope_denied");
        return -1;
    }
    return 0;
}

int yai_session_build_prompt_context_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    int rc;
    int n;

    if (!out || out_cap == 0)
        return -1;

    rc = yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err));
    if (rc != 0)
    {
        if (strcmp(status, "no_active") == 0)
        {
            n = snprintf(out,
                         out_cap,
                         "{\"type\":\"yai.workspace.prompt_context.v1\",\"binding_status\":\"no_active\",\"binding_scope\":\"session\"}");
        }
        else
        {
            n = snprintf(out,
                         out_cap,
                         "{\"type\":\"yai.workspace.prompt_context.v1\",\"binding_status\":\"%s\",\"binding_scope\":\"session\",\"reason\":\"%s\"}",
                         status[0] ? status : "invalid",
                         err[0] ? err : "binding_error");
        }
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.prompt_context.v1\","
                 "\"binding_status\":\"active\","
                 "\"binding_scope\":\"session\","
                 "\"workspace_id\":\"%s\","
                 "\"workspace_alias\":\"%s\","
                 "\"state\":\"%s\","
                 "\"workspace_root\":\"%s\","
                 "\"root_anchor_mode\":\"%s\","
                 "\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\"},"
                 "\"execution\":{\"mode_requested\":\"%s\",\"mode_effective\":\"%s\",\"degraded\":%s}"
                 "}",
                 info.ws_id,
                 info.workspace_alias,
                 info.state,
                 info.root_path,
                 info.root_anchor_mode[0] ? info.root_anchor_mode : "managed_default_root",
                 info.declared_control_family,
                 info.declared_specialization,
                 info.execution_mode_requested[0] ? info.execution_mode_requested : "scoped",
                 info.execution_mode_effective[0] ? info.execution_mode_effective : "scoped",
                 info.execution_mode_degraded ? "true" : "false");
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

static int yai_workspace_binding_validity(const char *binding_status)
{
    if (!binding_status)
        return 0;
    if (strcmp(binding_status, "active") == 0)
        return 1;
    if (strcmp(binding_status, "no_active") == 0)
        return 1;
    return 0;
}

static int yai_workspace_has_declared_context(const yai_workspace_runtime_info_t *info)
{
    if (!info)
        return 0;
    return (info->declared_control_family[0] || info->declared_specialization[0]) ? 1 : 0;
}

static int yai_workspace_has_effective_context(const yai_workspace_runtime_info_t *info)
{
    if (!info)
        return 0;
    return (info->effective_stack_ref[0] || info->last_effect_summary[0]) ? 1 : 0;
}

static int yai_workspace_recovery_tracked(const yai_workspace_runtime_info_t *info)
{
    if (!info)
        return 0;
    return info->declared_context_source[0] ? 1 : 0;
}

static const char *yai_workspace_recovery_state(const yai_workspace_runtime_info_t *info)
{
    if (!info || !info->declared_context_source[0])
        return "unknown";
    if (strcmp(info->declared_context_source, "restored") == 0)
        return "restored";
    return "fresh";
}

static int yai_workspace_build_runtime_capabilities_json(const yai_workspace_runtime_info_t *info,
                                                         const char *binding_status,
                                                         char *out,
                                                         size_t out_cap)
{
    const yai_runtime_capability_state_t *caps = yai_runtime_capabilities_state();
    int runtime_ready = yai_runtime_capabilities_is_ready() != 0;
    int data_ready = yai_data_store_binding_is_ready() != 0;
    int knowledge_ready = 0;
    int graph_ready = 0;
    int workspace_selected = 0;
    int workspace_bound = 0;
    int exec_probe = yai_exec_runtime_probe();
    const char *exec_state = yai_exec_runtime_state_name((yai_exec_runtime_state_t)exec_probe);
    const char *recovery_state = "unknown";
    int recovery_tracked = 0;
    int n;

    if (!out || out_cap == 0)
        return -1;

    if (caps) {
        knowledge_ready = (caps->providers_ready && caps->memory_ready && caps->cognition_ready) ? 1 : 0;
    }

    workspace_selected = (binding_status && strcmp(binding_status, "active") == 0) ? 1 : 0;
    if (workspace_selected && info) {
        recovery_tracked = yai_workspace_recovery_tracked(info);
        recovery_state = yai_workspace_recovery_state(info);
    }

    if (workspace_selected && info && caps && caps->workspace_id[0] &&
        strcmp(caps->workspace_id, info->ws_id) == 0 && info->runtime_attached) {
        workspace_bound = 1;
    }

    graph_ready = (data_ready && runtime_ready && workspace_bound) ? 1 : 0;

    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"runtime\":{\"ready\":%s,\"name\":\"%s\"},"
                 "\"workspace_binding\":{\"selected\":%s,\"bound\":%s,\"workspace_id\":\"%s\"},"
                 "\"data\":{\"store_binding_ready\":%s,\"root\":\"%s\"},"
                 "\"graph\":{\"ready\":%s,\"truth_source\":\"persistent\"},"
                 "\"knowledge\":{\"ready\":%s,\"transient_authoritative\":false},"
                 "\"exec\":{\"state\":\"%s\",\"ready\":%s},"
                 "\"recovery\":{\"tracked\":%s,\"state\":\"%s\"}"
                 "}",
                 runtime_ready ? "true" : "false",
                 (caps && caps->runtime_name[0]) ? caps->runtime_name : "yai-runtime",
                 workspace_selected ? "true" : "false",
                 workspace_bound ? "true" : "false",
                 (workspace_selected && info) ? info->ws_id : "",
                 data_ready ? "true" : "false",
                 yai_data_store_binding_root() ? yai_data_store_binding_root() : "",
                 graph_ready ? "true" : "false",
                 knowledge_ready ? "true" : "false",
                 exec_state ? exec_state : "unknown",
                 (exec_probe == (int)YAI_EXEC_READY) ? "true" : "false",
                 recovery_tracked ? "true" : "false",
                 recovery_state);

    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

static int yai_governance_root_path(char *out, size_t out_cap, const char *rel)
{
    const char *root = getenv("YAI_GOVERNANCE_ROOT");
    const char *base = NULL;
    const char *candidates[] = {"governance", "../yai/governance", "../../yai/governance"};
    int i;
    FILE *probe = NULL;
    if (!out || out_cap == 0)
        return -1;
    if (root && root[0])
    {
        base = root;
    }
    else
    {
        for (i = 0; i < (int)(sizeof(candidates) / sizeof(candidates[0])); i++)
        {
            char p[MAX_PATH_LEN];
            if (snprintf(p, sizeof(p), "%s/classification/classification-map.json", candidates[i]) <= 0)
                continue;
            probe = fopen(p, "rb");
            if (probe)
            {
                fclose(probe);
                base = candidates[i];
                break;
            }
        }
    }
    if (!base)
        return -1;
    if (snprintf(out, out_cap, "%s/%s", base, rel ? rel : "") <= 0)
        return -1;
    return 0;
}

static int yai_read_text(const char *path, char *out, size_t out_cap)
{
    FILE *f;
    size_t n;
    if (!path || !out || out_cap < 2)
        return -1;
    f = fopen(path, "rb");
    if (!f)
        return -1;
    n = fread(out, 1, out_cap - 1, f);
    fclose(f);
    out[n] = '\0';
    return 0;
}

static int yai_governance_family_exists(const char *family)
{
    char path[MAX_PATH_LEN];
    char json[YAI_WS_JSON_IO_CAP];
    char needle[160];
    if (!family || !family[0])
        return 0;
    if (yai_governance_root_path(path, sizeof(path), "families/index/families.index.json") != 0)
        return 0;
    if (yai_read_text(path, json, sizeof(json)) != 0)
        return 0;
    if (snprintf(needle, sizeof(needle), "\"canonical_name\": \"%s\"", family) <= 0)
        return 0;
    return strstr(json, needle) != NULL;
}

static int yai_governance_resolve_specialization_family(const char *specialization, char *family_out, size_t family_cap)
{
    char path[MAX_PATH_LEN];
    char json[YAI_WS_JSON_IO_CAP];
    char needle[192];
    char *p;
    char *fkey;
    char *colon;
    char *quote;
    if (!specialization || !specialization[0] || !family_out || family_cap == 0)
        return -1;
    family_out[0] = '\0';
    if (yai_governance_root_path(path, sizeof(path), "specializations/index/specializations.index.json") != 0)
        return -1;
    if (yai_read_text(path, json, sizeof(json)) != 0)
        return -1;
    if (snprintf(needle, sizeof(needle), "\"specialization_id\": \"%s\"", specialization) <= 0)
        return -1;
    p = strstr(json, needle);
    if (!p)
        return -1;
    fkey = strstr(p, "\"family\":");
    if (!fkey)
        return -1;
    colon = strchr(fkey, ':');
    if (!colon)
        return -1;
    quote = strchr(colon, '"');
    if (!quote)
        return -1;
    quote++;
    {
        char *end = strchr(quote, '"');
        size_t n;
        if (!end)
            return -1;
        n = (size_t)(end - quote);
        if (n >= family_cap)
            n = family_cap - 1;
        memcpy(family_out, quote, n);
        family_out[n] = '\0';
    }
    return family_out[0] ? 0 : -1;
}

static int yai_governance_specialization_matches_family(const char *family, const char *specialization)
{
    char inferred_family[96];
    if (!specialization || !specialization[0])
        return 1;
    if (yai_governance_resolve_specialization_family(specialization, inferred_family, sizeof(inferred_family)) != 0)
        return 0;
    if (!family || !family[0])
        return 1;
    return strcmp(family, inferred_family) == 0;
}

int yai_session_build_workspace_status_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    char runtime_caps[768];
    int rc;
    int n;
    if (!out || out_cap == 0)
        return -1;
    rc = yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err));
    if (yai_workspace_build_runtime_capabilities_json((rc == 0) ? &info : NULL,
                                                      status,
                                                      runtime_caps,
                                                      sizeof(runtime_caps)) != 0)
    {
        snprintf(runtime_caps, sizeof(runtime_caps), "%s", "{}");
    }
    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.status.v1\","
                 "\"active\":%s,"
                 "\"binding_status\":\"%s\","
                 "\"binding_valid\":%s,"
                 "\"namespace_scope\":\"workspace\","
                 "\"namespace_valid\":%s,"
                 "\"boundary_reason\":\"%s\","
                 "\"containment_layout\":\"%s\","
                 "\"containment_ready\":%s,"
                 "\"security_level_declared\":\"%s\","
                 "\"security_level_effective\":\"%s\","
                 "\"security_enforcement_mode\":\"%s\","
                 "\"security_backend_mode\":\"%s\","
                 "\"execution_mode_requested\":\"%s\","
                 "\"execution_mode_effective\":\"%s\","
                 "\"execution_mode_degraded\":%s,"
                 "\"execution_degraded_reason\":\"%s\","
                 "\"execution_unsupported_scopes\":\"%s\","
                 "\"runtime_attached\":%s,"
                 "\"isolation_mode\":\"%s\","
                 "\"debug_mode\":%s,"
                 "\"declared_context_present\":%s,"
                 "\"effective_context_present\":%s,"
                 "\"workspace_root\":\"%s\","
                 "\"workspace_store_root\":\"%s\","
                 "\"runtime_capabilities\":%s,"
                 "\"root_anchor_mode\":\"%s\","
                 "\"shell_path_relation\":\"%s\","
                 "\"reason\":\"%s\""
                 "}",
                 (rc == 0 && strcmp(status, "active") == 0) ? "true" : "false",
                 status[0] ? status : "invalid",
                 yai_workspace_binding_validity(status) ? "true" : "false",
                 (rc == 0 && info.namespace_valid) ? "true" : "false",
                 (rc == 0 && info.boundary_reason[0]) ? info.boundary_reason : "none",
                 (rc == 0 && info.containment_layout[0]) ? info.containment_layout : "v1",
                 (rc == 0 && info.containment_ready) ? "true" : "false",
                 (rc == 0 && info.security_level_declared[0]) ? info.security_level_declared : "scoped",
                 (rc == 0 && info.security_level_effective[0]) ? info.security_level_effective : "logical",
                 (rc == 0 && info.security_enforcement_mode[0]) ? info.security_enforcement_mode : "runtime_scoped",
                 (rc == 0 && info.security_backend_mode[0]) ? info.security_backend_mode : "none",
                 (rc == 0 && info.execution_mode_requested[0]) ? info.execution_mode_requested : "scoped",
                 (rc == 0 && info.execution_mode_effective[0]) ? info.execution_mode_effective : "scoped",
                 (rc == 0 && info.execution_mode_degraded) ? "true" : "false",
                 (rc == 0 && info.execution_degraded_reason[0]) ? info.execution_degraded_reason : "none",
                 (rc == 0 && info.execution_unsupported_scopes[0]) ? info.execution_unsupported_scopes : "none",
                 (rc == 0 && info.runtime_attached) ? "true" : "false",
                 (rc == 0 && info.isolation_mode[0]) ? info.isolation_mode : "process",
                 (rc == 0 && info.debug_mode) ? "true" : "false",
                 (rc == 0 && yai_workspace_has_declared_context(&info)) ? "true" : "false",
                 (rc == 0 && yai_workspace_has_effective_context(&info)) ? "true" : "false",
                 (rc == 0) ? info.root_path : "",
                 (rc == 0) ? info.workspace_store_root : "",
                 runtime_caps,
                 (rc == 0) ? info.root_anchor_mode : "",
                 (rc == 0) ? info.shell_path_relation : "unknown",
                 err[0] ? err : "none");
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

int yai_session_build_workspace_inspect_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    char sci_experiment[192];
    char sci_parameter[192];
    char sci_repro[192];
    char sci_dataset[192];
    char sci_publication[192];
    char dig_outbound[192];
    char dig_sink[192];
    char dig_publication[192];
    char dig_retrieval[192];
    char dig_distribution[192];
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
    char read_primary_source[96];
    char read_fallback_reason[256];
    char runtime_caps[768];
    char op_summary[192];
    const char *review_state;
    int evt_external;
    int db_first_ready;
    int fallback_active;
    int rc;
    int n;
    if (!out || out_cap == 0)
        return -1;
    rc = yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err));
    if (rc != 0 || strcmp(status, "active") != 0)
    {
        (void)snprintf(evt_declared, sizeof(evt_declared), "%s", "unset");
        (void)snprintf(evt_business, sizeof(evt_business), "%s", "not_resolved");
        (void)snprintf(evt_enforcement, sizeof(evt_enforcement), "%s", "not_resolved");
        (void)snprintf(evt_stage, sizeof(evt_stage), "%s", "unknown");
        (void)snprintf(evt_id, sizeof(evt_id), "%s", "none");
        evt_external = 0;
        (void)snprintf(op_summary, sizeof(op_summary), "%s", "unknown/not_resolved => not_resolved");
        review_state = "unresolved";
        (void)snprintf(sci_experiment, sizeof(sci_experiment), "%s", "not available");
        (void)snprintf(sci_parameter, sizeof(sci_parameter), "%s", "not available");
        (void)snprintf(sci_repro, sizeof(sci_repro), "%s", "not available");
        (void)snprintf(sci_dataset, sizeof(sci_dataset), "%s", "not available");
        (void)snprintf(sci_publication, sizeof(sci_publication), "%s", "not available");
        (void)snprintf(dig_outbound, sizeof(dig_outbound), "%s", "not available");
        (void)snprintf(dig_sink, sizeof(dig_sink), "%s", "not available");
        (void)snprintf(dig_publication, sizeof(dig_publication), "%s", "not available");
        (void)snprintf(dig_retrieval, sizeof(dig_retrieval), "%s", "not available");
        (void)snprintf(dig_distribution, sizeof(dig_distribution), "%s", "not available");
        if (yai_workspace_build_runtime_capabilities_json(NULL,
                                                          status,
                                                          runtime_caps,
                                                          sizeof(runtime_caps)) != 0)
        {
            snprintf(runtime_caps, sizeof(runtime_caps), "%s", "{}");
        }
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.inspect.v1\","
                     "\"binding_status\":\"%s\","
                     "\"identity\":{\"workspace_id\":\"\",\"workspace_alias\":\"\",\"root_path\":\"\",\"state\":\"\"},"
                     "\"root_model\":{\"workspace_store_root\":\"\",\"runtime_state_root\":\"\",\"metadata_root\":\"\",\"root_anchor_mode\":\"\"},"
                     "\"containment\":{\"layout\":\"v1\",\"ready\":false,\"state_surface\":\"\",\"traces_index\":\"\",\"artifacts_index\":\"\",\"runtime_surface\":\"\",\"binding_surface\":\"\"},"
                     "\"security\":{\"level_declared\":\"scoped\",\"level_effective\":\"logical\",\"enforcement_mode\":\"runtime_scoped\",\"backend_mode\":\"none\",\"scopes\":{\"process\":false,\"filesystem\":true,\"socket\":false,\"network\":false,\"resource\":false,\"privilege\":false,\"runtime_route\":true,\"binding\":true},\"capabilities\":{\"sandbox_ready\":true,\"hardened_fs\":true,\"process_isolation\":false,\"network_policy\":false}},"
                     "\"execution\":{\"mode_requested\":\"scoped\",\"mode_effective\":\"logical\",\"degraded\":true,\"degraded_reason\":\"binding_unavailable\",\"unsupported_scopes\":\"process,socket,network,resource,privilege\",\"advisory_scopes\":\"process,socket,network,resource,privilege\",\"process_intent\":\"shared_runtime\",\"channel_mode\":\"global_control_scoped_route\",\"artifact_policy_mode\":\"workspace_owned\",\"network_intent\":\"advisory_none\",\"resource_intent\":\"advisory_none\",\"privilege_intent\":\"inherited_host\",\"attach_descriptor_ref\":\"\",\"execution_profile_ref\":\"\"},"
                     "\"boundary\":{\"namespace\":\"\",\"namespace_scope\":\"workspace\",\"namespace_valid\":false,\"state\":\"invalid\",\"reason\":\"binding_unavailable\"},"
                     "\"shell\":{\"cwd\":\"\",\"cwd_relation\":\"workspace_root_unset\"},"
                     "\"session\":{\"session_binding\":\"%s\",\"runtime_attached\":false,\"control_plane_attached\":false,\"isolation_mode\":\"process\",\"debug_mode\":false},"
                     "\"normative\":{\"declared\":{\"family\":\"\",\"specialization\":\"\",\"source\":\"unset\"},"
                     "\"inferred\":{\"family\":\"\",\"specialization\":\"\",\"confidence\":0.000},"
                     "\"effective\":{\"stack_ref\":\"\",\"overlays_ref\":\"\",\"effect_summary\":\"\",\"authority_summary\":\"\",\"evidence_summary\":\"\"}},"
                     "\"event_surface\":{\"event_id\":\"%s\",\"flow_stage\":\"%s\",\"declared_scenario_specialization\":\"%s\",\"business_specialization\":\"%s\",\"enforcement_specialization\":\"%s\",\"external_effect_boundary\":%s},"
                     "\"operational_state\":{\"binding_state\":\"%s\",\"attached_governance_objects\":\"\",\"active_effective_stack\":\"\",\"last_event_ref\":\"%s\",\"last_flow_stage\":\"%s\",\"last_business_specialization\":\"%s\",\"last_enforcement_specialization\":\"%s\",\"last_effect\":\"not_resolved\",\"last_authority\":\"not_available\",\"last_evidence\":\"not_available\",\"last_trace_ref\":\"\",\"review_state\":\"%s\",\"operational_summary\":\"%s\"},"
                     "\"governance\":{\"policy_attachments\":\"\",\"policy_attachment_count\":0},"
                     "\"scientific\":{\"experiment_context_summary\":\"%s\",\"parameter_governance_summary\":\"%s\",\"reproducibility_summary\":\"%s\",\"dataset_integrity_summary\":\"%s\",\"publication_control_summary\":\"%s\"},"
                     "\"digital\":{\"outbound_context_summary\":\"%s\",\"sink_target_summary\":\"%s\",\"publication_control_summary\":\"%s\",\"retrieval_control_summary\":\"%s\",\"distribution_control_summary\":\"%s\"},"
                     "\"runtime_capabilities\":%s,"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"runtime_fallback_only\",\"db_first_ready\":false,\"fallback_active\":true,\"fallback_reason\":\"binding_unavailable\",\"filesystem_primary\":false},"
                     "\"inspect\":{\"last_resolution_summary\":\"\",\"last_resolution_trace_ref\":\"\"},"
                     "\"reason\":\"%s\""
                     "}",
                     status[0] ? status : "invalid",
                     status[0] ? status : "invalid",
                     evt_id,
                     evt_stage,
                     evt_declared,
                     evt_business,
                     evt_enforcement,
                     evt_external ? "true" : "false",
                     status[0] ? status : "invalid",
                     evt_id,
                     evt_stage,
                     evt_business,
                     evt_enforcement,
                     review_state,
                     op_summary,
                     sci_experiment,
                     sci_parameter,
                     sci_repro,
                     sci_dataset,
                     sci_publication,
                     dig_outbound,
                     dig_sink,
                     dig_publication,
                     dig_retrieval,
                     dig_distribution,
                     runtime_caps,
                     err[0] ? err : "binding_error");
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    yai_workspace_build_scientific_summaries(&info,
                                             sci_experiment,
                                             sizeof(sci_experiment),
                                             sci_parameter,
                                             sizeof(sci_parameter),
                                             sci_repro,
                                             sizeof(sci_repro),
                                             sci_dataset,
                                             sizeof(sci_dataset),
                                             sci_publication,
                                             sizeof(sci_publication));
    yai_workspace_build_digital_summaries(&info,
                                          dig_outbound,
                                          sizeof(dig_outbound),
                                          dig_sink,
                                          sizeof(dig_sink),
                                          dig_publication,
                                          sizeof(dig_publication),
                                          dig_retrieval,
                                          sizeof(dig_retrieval),
                                          dig_distribution,
                                          sizeof(dig_distribution));
    yai_session_workspace_event_semantics(&info,
                                          evt_declared,
                                          sizeof(evt_declared),
                                          evt_business,
                                          sizeof(evt_business),
                                          evt_enforcement,
                                          sizeof(evt_enforcement),
                                          evt_stage,
                                          sizeof(evt_stage),
                                          &evt_external);
    snprintf(evt_id, sizeof(evt_id), "%s%s",
             info.last_resolution_trace_ref[0] ? "evt-" : "none",
             info.last_resolution_trace_ref[0] ? info.last_resolution_trace_ref : "");
    yai_workspace_read_event_evidence_index(&info,
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
    yai_workspace_read_enforcement_index(&info,
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
    yai_workspace_read_governance_index(&info,
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
    yai_workspace_read_authority_artifact_indexes(&info,
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
    (void)yai_storage_bridge_last_refs(info.ws_id,
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
    yai_workspace_db_first_read_model("workspace",
                                      sink_last_event_ref,
                                      sink_last_decision_ref,
                                      sink_last_evidence_ref,
                                      gov_last_object_ref,
                                      gov_last_lifecycle_ref,
                                      authority_last_ref,
                                      artifact_last_ref,
                                      brain_graph_node_ref,
                                      brain_graph_edge_ref,
                                      enforce_last_outcome_ref,
                                      enforce_last_linkage_ref,
                                      enforce_materialization_status,
                                      read_primary_source,
                                      sizeof(read_primary_source),
                                      read_fallback_reason,
                                      sizeof(read_fallback_reason),
                                      &db_first_ready,
                                      &fallback_active);
    review_state = yai_workspace_review_state_from_effect(info.last_effect_summary);
    yai_workspace_operational_summary(evt_stage, evt_business, info.last_effect_summary, op_summary, sizeof(op_summary));
    if (yai_workspace_build_runtime_capabilities_json(&info,
                                                      "active",
                                                      runtime_caps,
                                                      sizeof(runtime_caps)) != 0)
    {
        snprintf(runtime_caps, sizeof(runtime_caps), "%s", "{}");
    }

    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.inspect.v1\","
                 "\"binding_status\":\"active\","
                 "\"identity\":{\"workspace_id\":\"%s\",\"workspace_alias\":\"%s\",\"root_path\":\"%s\",\"state\":\"%s\"},"
                 "\"root_model\":{\"workspace_store_root\":\"%s\",\"runtime_state_root\":\"%s\",\"metadata_root\":\"%s\",\"root_anchor_mode\":\"%s\"},"
                 "\"containment\":{\"layout\":\"%s\",\"ready\":%s,\"state_surface\":\"%s\",\"traces_index\":\"%s\",\"artifacts_index\":\"%s\",\"runtime_surface\":\"%s\",\"binding_surface\":\"%s\"},"
                 "\"security\":{\"level_declared\":\"%s\",\"level_effective\":\"%s\",\"enforcement_mode\":\"%s\",\"backend_mode\":\"%s\",\"scopes\":{\"process\":%s,\"filesystem\":%s,\"socket\":%s,\"network\":%s,\"resource\":%s,\"privilege\":%s,\"runtime_route\":%s,\"binding\":%s},\"capabilities\":{\"sandbox_ready\":%s,\"hardened_fs\":%s,\"process_isolation\":%s,\"network_policy\":%s}},"
                 "\"execution\":{\"mode_requested\":\"%s\",\"mode_effective\":\"%s\",\"degraded\":%s,\"degraded_reason\":\"%s\",\"unsupported_scopes\":\"%s\",\"advisory_scopes\":\"%s\",\"process_intent\":\"%s\",\"channel_mode\":\"%s\",\"artifact_policy_mode\":\"%s\",\"network_intent\":\"%s\",\"resource_intent\":\"%s\",\"privilege_intent\":\"%s\",\"attach_descriptor_ref\":\"%s\",\"execution_profile_ref\":\"%s\"},"
                 "\"boundary\":{\"namespace\":\"%s\",\"namespace_scope\":\"workspace\",\"namespace_valid\":%s,\"state\":\"%s\",\"reason\":\"%s\"},"
                 "\"shell\":{\"cwd\":\"%s\",\"cwd_relation\":\"%s\"},"
                 "\"session\":{\"session_binding\":\"%s\",\"runtime_attached\":%s,\"control_plane_attached\":%s,\"isolation_mode\":\"%s\",\"debug_mode\":%s},"
                 "\"normative\":{\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\",\"source\":\"%s\"},"
                 "\"inferred\":{\"family\":\"%s\",\"specialization\":\"%s\",\"confidence\":%.3f},"
                 "\"effective\":{\"stack_ref\":\"%s\",\"overlays_ref\":\"%s\",\"effect_summary\":\"%s\",\"authority_summary\":\"%s\",\"evidence_summary\":\"%s\"}},"
                 "\"event_surface\":{\"event_id\":\"%s\",\"flow_stage\":\"%s\",\"declared_scenario_specialization\":\"%s\",\"business_specialization\":\"%s\",\"enforcement_specialization\":\"%s\",\"external_effect_boundary\":%s},"
                 "\"operational_state\":{\"binding_state\":\"active\",\"attached_governance_objects\":\"%s\",\"active_effective_stack\":\"%s\",\"last_event_ref\":\"%s\",\"last_flow_stage\":\"%s\",\"last_business_specialization\":\"%s\",\"last_enforcement_specialization\":\"%s\",\"last_effect\":\"%s\",\"last_authority\":\"%s\",\"last_evidence\":\"%s\",\"last_trace_ref\":\"%s\",\"review_state\":\"%s\",\"operational_summary\":\"%s\"},"
                 "\"governance\":{\"policy_attachments\":\"%s\",\"policy_attachment_count\":%d},"
                 "\"scientific\":{\"experiment_context_summary\":\"%s\",\"parameter_governance_summary\":\"%s\",\"reproducibility_summary\":\"%s\",\"dataset_integrity_summary\":\"%s\",\"publication_control_summary\":\"%s\"},"
                 "\"digital\":{\"outbound_context_summary\":\"%s\",\"sink_target_summary\":\"%s\",\"publication_control_summary\":\"%s\",\"retrieval_control_summary\":\"%s\",\"distribution_control_summary\":\"%s\"},"
                 "\"event_evidence_sink\":{\"last_event_ref\":\"%s\",\"last_decision_ref\":\"%s\",\"last_evidence_ref\":\"%s\",\"event_store_ref\":\"%s\",\"decision_store_ref\":\"%s\",\"evidence_store_ref\":\"%s\"},"
                 "\"governance_persistence\":{\"last_object_ref\":\"%s\",\"last_lifecycle_ref\":\"%s\",\"last_attachment_ref\":\"%s\",\"object_store_ref\":\"%s\",\"lifecycle_store_ref\":\"%s\",\"attachment_store_ref\":\"%s\"},"
                 "\"authority_artifact_persistence\":{\"last_authority_ref\":\"%s\",\"last_authority_resolution_ref\":\"%s\",\"last_artifact_ref\":\"%s\",\"last_artifact_linkage_ref\":\"%s\",\"authority_store_ref\":\"%s\",\"artifact_store_ref\":\"%s\"},"
                 "\"graph_persistence\":{\"last_graph_node_ref\":\"%s\",\"last_graph_edge_ref\":\"%s\",\"graph_store_ref\":\"%s\",\"graph_truth_authoritative\":true},"
                 "\"knowledge_transient_persistence\":{\"last_transient_state_ref\":\"%s\",\"last_transient_working_set_ref\":\"%s\",\"transient_store_ref\":\"%s\",\"transient_authoritative\":false},"
                 "\"runtime_capabilities\":%s,"
                 "\"enforcement_record_set\":{\"last_outcome_ref\":\"%s\",\"last_linkage_ref\":\"%s\",\"materialization_status\":\"%s\",\"missing_fields\":\"%s\",\"outcome_store_ref\":\"%s\",\"linkage_store_ref\":\"%s\"},"
                 "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"%s\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                 "\"inspect\":{\"last_resolution_summary\":\"%s\",\"last_resolution_trace_ref\":\"%s\"}"
                 "}",
                 info.ws_id,
                 info.workspace_alias,
                 info.root_path,
                 info.state,
                 info.workspace_store_root,
                 info.runtime_state_root,
                 info.metadata_root,
                 info.root_anchor_mode[0] ? info.root_anchor_mode : "managed_default_root",
                 info.containment_layout[0] ? info.containment_layout : "v1",
                 info.containment_ready ? "true" : "false",
                 info.state_surface_path,
                 info.traces_index_path,
                 info.artifacts_index_path,
                 info.runtime_surface_path,
                 info.binding_state_path,
                 info.security_level_declared[0] ? info.security_level_declared : "scoped",
                 info.security_level_effective[0] ? info.security_level_effective : "logical",
                 info.security_enforcement_mode[0] ? info.security_enforcement_mode : "runtime_scoped",
                 info.security_backend_mode[0] ? info.security_backend_mode : "none",
                 info.scope_process ? "true" : "false",
                 info.scope_filesystem ? "true" : "false",
                 info.scope_socket ? "true" : "false",
                 info.scope_network ? "true" : "false",
                 info.scope_resource ? "true" : "false",
                 info.scope_privilege ? "true" : "false",
                 info.scope_runtime_route ? "true" : "false",
                 info.scope_binding ? "true" : "false",
                 info.capability_sandbox_ready ? "true" : "false",
                 info.capability_hardened_fs ? "true" : "false",
                 info.capability_process_isolation ? "true" : "false",
                 info.capability_network_policy ? "true" : "false",
                 info.execution_mode_requested[0] ? info.execution_mode_requested : "scoped",
                 info.execution_mode_effective[0] ? info.execution_mode_effective : "scoped",
                 info.execution_mode_degraded ? "true" : "false",
                 info.execution_degraded_reason[0] ? info.execution_degraded_reason : "none",
                 info.execution_unsupported_scopes[0] ? info.execution_unsupported_scopes : "none",
                 info.execution_advisory_scopes[0] ? info.execution_advisory_scopes : "none",
                 info.process_intent[0] ? info.process_intent : "shared_runtime",
                 info.channel_mode[0] ? info.channel_mode : "global_control_scoped_route",
                 info.artifact_policy_mode[0] ? info.artifact_policy_mode : "workspace_owned",
                 info.network_intent[0] ? info.network_intent : "advisory_none",
                 info.resource_intent[0] ? info.resource_intent : "advisory_none",
                 info.privilege_intent[0] ? info.privilege_intent : "inherited_host",
                 info.attach_descriptor_ref,
                 info.execution_profile_ref,
                 info.workspace_namespace,
                 info.namespace_valid ? "true" : "false",
                 info.namespace_valid ? "enforced" : "invalid",
                 info.boundary_reason[0] ? info.boundary_reason : "none",
                 info.shell_cwd,
                 info.shell_path_relation[0] ? info.shell_path_relation : "unknown",
                 info.session_binding,
                 info.runtime_attached ? "true" : "false",
                 info.control_plane_attached ? "true" : "false",
                 info.isolation_mode[0] ? info.isolation_mode : "process",
                 info.debug_mode ? "true" : "false",
                 info.declared_control_family,
                 info.declared_specialization,
                 info.declared_context_source[0] ? info.declared_context_source : "unset",
                 info.inferred_family,
                 info.inferred_specialization,
                 info.inferred_confidence,
                 info.effective_stack_ref,
                 info.effective_overlays_ref,
                 info.last_effect_summary,
                 info.last_authority_summary,
                 info.last_evidence_summary,
                 evt_id,
                 evt_stage,
                 evt_declared,
                 evt_business,
                 evt_enforcement,
                 evt_external ? "true" : "false",
                 info.policy_attachments_csv,
                 info.effective_stack_ref,
                 evt_id,
                 evt_stage,
                 evt_business,
                 evt_enforcement,
                 info.last_effect_summary,
                 info.last_authority_summary,
                 info.last_evidence_summary,
                 info.last_resolution_trace_ref,
                 review_state,
                 op_summary,
                 info.policy_attachments_csv,
                 info.policy_attachment_count,
                 sci_experiment,
                 sci_parameter,
                 sci_repro,
                 sci_dataset,
                 sci_publication,
                 dig_outbound,
                 dig_sink,
                 dig_publication,
                 dig_retrieval,
                 dig_distribution,
                 sink_last_event_ref[0] ? sink_last_event_ref : evt_id,
                 sink_last_decision_ref,
                 sink_last_evidence_ref,
                 sink_event_store_ref,
                 sink_decision_store_ref,
                 sink_evidence_store_ref,
                 gov_last_object_ref,
                 gov_last_lifecycle_ref,
                 gov_last_attachment_ref,
                 gov_object_store_ref,
                 gov_lifecycle_store_ref,
                 gov_attachment_store_ref,
                 authority_last_ref,
                 authority_resolution_ref,
                 artifact_last_ref,
                 artifact_linkage_ref,
                 authority_store_ref,
                 artifact_store_ref,
                 brain_graph_node_ref,
                 brain_graph_edge_ref,
                 brain_graph_store_ref,
                 brain_transient_state_ref,
                 brain_transient_working_set_ref,
                 brain_transient_store_ref,
                 runtime_caps,
                 enforce_last_outcome_ref,
                 enforce_last_linkage_ref,
                 enforce_materialization_status[0] ? enforce_materialization_status : "unknown",
                 enforce_missing_fields,
                 enforce_outcome_store_ref,
                 enforce_linkage_store_ref,
                 read_primary_source,
                 db_first_ready ? "true" : "false",
                 fallback_active ? "true" : "false",
                 read_fallback_reason,
                 info.last_resolution_summary,
                 info.last_resolution_trace_ref);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

int yai_session_build_workspace_domain_get_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    int rc;
    int n;
    if (!out || out_cap == 0)
        return -1;
    rc = yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err));
    if (rc != 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{\"type\":\"yai.workspace.domain.get.v1\",\"binding_status\":\"%s\",\"reason\":\"%s\"}",
                     status[0] ? status : "invalid",
                     err[0] ? err : "binding_error");
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.domain.get.v1\","
                 "\"workspace_id\":\"%s\","
                 "\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\",\"source\":\"%s\"},"
                 "\"inferred\":{\"family\":\"%s\",\"specialization\":\"%s\",\"confidence\":%.3f},"
                 "\"effective\":{\"family\":\"%s\",\"specialization\":\"%s\"}"
                 "}",
                 info.ws_id,
                 info.declared_control_family,
                 info.declared_specialization,
                 info.declared_context_source[0] ? info.declared_context_source : "unset",
                 info.inferred_family,
                 info.inferred_specialization,
                 info.inferred_confidence,
                 info.inferred_family[0] ? info.inferred_family : info.declared_control_family,
                 info.inferred_specialization[0] ? info.inferred_specialization : info.declared_specialization);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}
