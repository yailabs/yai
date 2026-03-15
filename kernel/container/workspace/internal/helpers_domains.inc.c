static int yai_workspace_governance_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s/governance", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_governance_sink_paths(const char *ws_id,
                                               char *object_log,
                                               size_t object_log_cap,
                                               char *lifecycle_log,
                                               size_t lifecycle_log_cap,
                                               char *attachment_log,
                                               size_t attachment_log_cap,
                                               char *index_path,
                                               size_t index_path_cap)
{
    char gov_root[MAX_PATH_LEN];
    if (!ws_id || !object_log || !lifecycle_log || !attachment_log || !index_path)
        return -1;
    if (yai_workspace_governance_root_path(ws_id, gov_root, sizeof(gov_root)) != 0)
        return -1;
    if (snprintf(object_log, object_log_cap, "%s/object-state.v1.ndjson", gov_root) <= 0)
        return -1;
    if (snprintf(lifecycle_log, lifecycle_log_cap, "%s/lifecycle-state.v1.ndjson", gov_root) <= 0)
        return -1;
    if (snprintf(attachment_log, attachment_log_cap, "%s/attachment-state.v1.ndjson", gov_root) <= 0)
        return -1;
    if (snprintf(index_path, index_path_cap, "%s/index.v1.json", gov_root) <= 0)
        return -1;
    return 0;
}

static const char *yai_governance_approval_state_for_meta(const yai_governable_object_meta_t *meta)
{
    if (!meta) return "unknown";
    if (strcmp(meta->status, "approved") == 0) return "approved";
    if (strcmp(meta->review_state, "withdrawn") == 0) return "withdrawn";
    if (strcmp(meta->review_state, "rejected") == 0) return "rejected";
    if (strcmp(meta->status, "candidate") == 0) return "candidate";
    return "pending";
}

static const char *yai_governance_apply_eligibility_for_meta(const yai_governable_object_meta_t *meta)
{
    if (!meta) return "unknown";
    if (strcmp(meta->status, "approved") == 0 &&
        strcmp(meta->review_state, "approved") == 0 &&
        meta->runtime_consumable)
        return "eligible";
    return "not_eligible";
}

static const char *yai_governance_lifecycle_for_meta(const yai_governable_object_meta_t *meta)
{
    if (!meta) return "unknown";
    if (strcmp(meta->review_state, "withdrawn") == 0) return "withdrawn";
    if (strcmp(meta->review_state, "rejected") == 0) return "rejected";
    if (strcmp(meta->review_state, "approved") == 0) return "approved";
    if (strcmp(meta->status, "candidate") == 0) return "candidate";
    return "under_review";
}

static int yai_workspace_append_governance_persistence(const char *ws_id,
                                                       const yai_governable_object_meta_t *meta,
                                                       const char *object_id,
                                                       const char *action,
                                                       const char *attachment_state,
                                                       const char *eligibility_result,
                                                       const char *compatibility_result,
                                                       const char *conflict_summary,
                                                       const char *warnings,
                                                       const char *event_ref,
                                                       const char *decision_ref,
                                                       const char *evidence_ref,
                                                       char *err,
                                                       size_t err_cap)
{
    char object_log[MAX_PATH_LEN];
    char lifecycle_log[MAX_PATH_LEN];
    char attachment_log[MAX_PATH_LEN];
    char index_path[MAX_PATH_LEN];
    char ts_iso[48];
    char object_state_id[256];
    char lifecycle_state_id[256];
    char attachment_id[320];
    char object_lifecycle[384];
    char lifecycle_lifecycle[384];
    char attachment_lifecycle[384];
    char object_row[4096];
    char lifecycle_row[3072];
    char attachment_row[4096];
    char index_row[2048];
    const char *approval_state;
    const char *apply_eligibility;
    const char *lifecycle_state;
    time_t now;

    if (err && err_cap > 0) err[0] = '\0';
    if (!ws_id || !meta || !object_id || !object_id[0])
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "governance_persistence_bad_args");
        return -1;
    }
    if (yai_workspace_governance_sink_paths(ws_id,
                                            object_log,
                                            sizeof(object_log),
                                            lifecycle_log,
                                            sizeof(lifecycle_log),
                                            attachment_log,
                                            sizeof(attachment_log),
                                            index_path,
                                            sizeof(index_path)) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "governance_persistence_path_failed");
        return -1;
    }

    now = time(NULL);
    yai_format_iso8601_utc(now, ts_iso, sizeof(ts_iso));
    if (yai_lifecycle_build_json_fragment("governance_object_state", "workspace_id,governance_object_id", object_lifecycle, sizeof(object_lifecycle)) != 0 ||
        yai_lifecycle_build_json_fragment("governance_lifecycle_state", "workspace_id,governance_object_id", lifecycle_lifecycle, sizeof(lifecycle_lifecycle)) != 0 ||
        yai_lifecycle_build_json_fragment("governance_attachment_state", "workspace_id,governance_object_id", attachment_lifecycle, sizeof(attachment_lifecycle)) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "governance_lifecycle_fields_encode_failed");
        return -1;
    }
    (void)snprintf(object_state_id, sizeof(object_state_id), "gobj-%s", object_id);
    (void)snprintf(lifecycle_state_id, sizeof(lifecycle_state_id), "glc-%s", object_id);
    (void)snprintf(attachment_id, sizeof(attachment_id), "gatt-%s-%s", ws_id, object_id);

    approval_state = yai_governance_approval_state_for_meta(meta);
    apply_eligibility = yai_governance_apply_eligibility_for_meta(meta);
    lifecycle_state = yai_governance_lifecycle_for_meta(meta);

    if (snprintf(object_row,
                 sizeof(object_row),
                 "{\"type\":\"yai.governance_object_state.v1\",\"schema_version\":\"v1\","
                 "\"object_state_id\":\"%s\",\"governance_object_id\":\"%s\","
                 "\"object_kind\":\"%s\",\"owner\":\"embedded.governance\","
                 "\"organization_scope\":\"workspace_runtime\","
                 "\"workspace_targets\":\"%s\",\"domain_targets\":\"%s\",\"specialization_targets\":\"%s\","
                 "\"source_refs\":\"%s\",\"status\":\"%s\",\"review_state\":\"%s\","
                 "\"runtime_consumable\":%s,\"created_at\":\"%s\",\"updated_at\":\"%s\","
                 "%s}",
                 object_state_id,
                 object_id,
                 meta->kind,
                 meta->workspace_targets_csv,
                 meta->family_targets_csv,
                 meta->specialization_targets_csv,
                 meta->manifest_ref,
                 meta->status,
                 meta->review_state,
                 meta->runtime_consumable ? "true" : "false",
                 ts_iso,
                 ts_iso,
                 object_lifecycle) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "governance_object_encode_failed");
        return -1;
    }

    if (snprintf(lifecycle_row,
                 sizeof(lifecycle_row),
                 "{\"type\":\"yai.governance_lifecycle_state.v1\",\"schema_version\":\"v1\","
                 "\"state_id\":\"%s\",\"governance_object_id\":\"%s\","
                 "\"lifecycle_state\":\"%s\",\"review_state\":\"%s\",\"approval_state\":\"%s\","
                 "\"apply_eligibility\":\"%s\",\"status_reason\":\"%s\",\"updated_at\":\"%s\","
                 "%s}",
                 lifecycle_state_id,
                 object_id,
                 lifecycle_state,
                 meta->review_state,
                 approval_state,
                 apply_eligibility,
                 action && action[0] ? action : "update",
                 ts_iso,
                 lifecycle_lifecycle) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "governance_lifecycle_encode_failed");
        return -1;
    }

    if (snprintf(attachment_row,
                 sizeof(attachment_row),
                 "{\"type\":\"yai.governance_attachment_state.v1\",\"schema_version\":\"v1\","
                 "\"attachment_id\":\"%s\",\"workspace_id\":\"%s\",\"governance_object_id\":\"%s\","
                 "\"action\":\"%s\",\"attachment_state\":\"%s\",\"activation_state\":\"%s\","
                 "\"eligibility_result\":\"%s\",\"compatibility_result\":\"%s\","
                 "\"conflict_summary\":\"%s\",\"warnings\":\"%s\","
                 "\"event_ref\":\"%s\",\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\","
                 "\"updated_at\":\"%s\","
                 "%s}",
                 attachment_id,
                 ws_id,
                 object_id,
                 action && action[0] ? action : "attach",
                 attachment_state && attachment_state[0] ? attachment_state : "unknown",
                 (attachment_state && strstr(attachment_state, "active")) ? "active" : "inactive",
                 eligibility_result ? eligibility_result : "",
                 compatibility_result ? compatibility_result : "",
                 conflict_summary ? conflict_summary : "",
                 warnings ? warnings : "",
                 event_ref ? event_ref : "",
                 decision_ref ? decision_ref : "",
                 evidence_ref ? evidence_ref : "",
                 ts_iso,
                 attachment_lifecycle) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "governance_attachment_encode_failed");
        return -1;
    }

    if (yai_append_json_line(object_log, object_row) != 0 ||
        yai_append_json_line(lifecycle_log, lifecycle_row) != 0 ||
        yai_append_json_line(attachment_log, attachment_row) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "governance_persistence_append_failed");
        return -1;
    }

    if (snprintf(index_row,
                 sizeof(index_row),
                 "{\"type\":\"yai.governance.persistence.index.v1\",\"workspace_id\":\"%s\","
                 "\"last_governance_object_ref\":\"%s\",\"last_lifecycle_ref\":\"%s\","
                 "\"last_attachment_ref\":\"%s\",\"last_event_ref\":\"%s\","
                 "\"last_decision_ref\":\"%s\",\"last_evidence_ref\":\"%s\","
                 "\"retention_profile_ref\":\"matrix:data-lifecycle-retention-v0.1.0\","
                 "\"updated_at\":\"%s\","
                 "\"stores\":{\"objects\":\"%s\",\"lifecycle\":\"%s\",\"attachments\":\"%s\"}}",
                 ws_id,
                 object_state_id,
                 lifecycle_state_id,
                 attachment_id,
                 event_ref ? event_ref : "",
                 decision_ref ? decision_ref : "",
                 evidence_ref ? evidence_ref : "",
                 ts_iso,
                 object_log,
                 lifecycle_log,
                 attachment_log) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "governance_index_encode_failed");
        return -1;
    }
    {
        FILE *f = fopen(index_path, "w");
        if (!f)
        {
            if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "governance_index_write_failed");
            return -1;
        }
        (void)fprintf(f, "%s\n", index_row);
        fclose(f);
    }
    return 0;
}

static void yai_workspace_read_governance_index(const yai_workspace_runtime_info_t *info,
                                                char *object_ref,
                                                size_t object_ref_cap,
                                                char *lifecycle_ref,
                                                size_t lifecycle_ref_cap,
                                                char *attachment_ref,
                                                size_t attachment_ref_cap,
                                                char *object_store_ref,
                                                size_t object_store_ref_cap,
                                                char *lifecycle_store_ref,
                                                size_t lifecycle_store_ref_cap,
                                                char *attachment_store_ref,
                                                size_t attachment_store_ref_cap)
{
    char object_log[MAX_PATH_LEN];
    char lifecycle_log[MAX_PATH_LEN];
    char attachment_log[MAX_PATH_LEN];
    char index_path[MAX_PATH_LEN];
    char buf[2048];

    if (object_ref && object_ref_cap > 0) object_ref[0] = '\0';
    if (lifecycle_ref && lifecycle_ref_cap > 0) lifecycle_ref[0] = '\0';
    if (attachment_ref && attachment_ref_cap > 0) attachment_ref[0] = '\0';
    if (object_store_ref && object_store_ref_cap > 0) object_store_ref[0] = '\0';
    if (lifecycle_store_ref && lifecycle_store_ref_cap > 0) lifecycle_store_ref[0] = '\0';
    if (attachment_store_ref && attachment_store_ref_cap > 0) attachment_store_ref[0] = '\0';
    if (!info || !info->ws_id[0]) return;
    if (yai_workspace_governance_sink_paths(info->ws_id,
                                            object_log,
                                            sizeof(object_log),
                                            lifecycle_log,
                                            sizeof(lifecycle_log),
                                            attachment_log,
                                            sizeof(attachment_log),
                                            index_path,
                                            sizeof(index_path)) != 0)
        return;

    if (object_store_ref && object_store_ref_cap > 0)
        (void)snprintf(object_store_ref, object_store_ref_cap, "%s", object_log);
    if (lifecycle_store_ref && lifecycle_store_ref_cap > 0)
        (void)snprintf(lifecycle_store_ref, lifecycle_store_ref_cap, "%s", lifecycle_log);
    if (attachment_store_ref && attachment_store_ref_cap > 0)
        (void)snprintf(attachment_store_ref, attachment_store_ref_cap, "%s", attachment_log);

    if (yai_read_text(index_path, buf, sizeof(buf)) != 0) return;
    if (object_ref && object_ref_cap > 0)
        (void)yai_session_extract_json_string(buf, "last_governance_object_ref", object_ref, object_ref_cap);
    if (lifecycle_ref && lifecycle_ref_cap > 0)
        (void)yai_session_extract_json_string(buf, "last_lifecycle_ref", lifecycle_ref, lifecycle_ref_cap);
    if (attachment_ref && attachment_ref_cap > 0)
        (void)yai_session_extract_json_string(buf, "last_attachment_ref", attachment_ref, attachment_ref_cap);
}

static int yai_workspace_authority_sink_paths(const char *ws_id,
                                              char *authority_log,
                                              size_t authority_log_cap,
                                              char *resolution_log,
                                              size_t resolution_log_cap,
                                              char *index_path,
                                              size_t index_path_cap)
{
    char run_dir[MAX_PATH_LEN];
    char authority_root[MAX_PATH_LEN];
    if (!ws_id || !authority_log || !resolution_log || !index_path)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(authority_root, sizeof(authority_root), "%s/authority", run_dir) <= 0)
        return -1;
    if (snprintf(authority_log, authority_log_cap, "%s/authority-state.v1.ndjson", authority_root) <= 0)
        return -1;
    if (snprintf(resolution_log, resolution_log_cap, "%s/resolution-state.v1.ndjson", authority_root) <= 0)
        return -1;
    if (snprintf(index_path, index_path_cap, "%s/index.v1.json", authority_root) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_artifact_sink_paths(const char *ws_id,
                                             char *metadata_log,
                                             size_t metadata_log_cap,
                                             char *linkage_log,
                                             size_t linkage_log_cap,
                                             char *index_path,
                                             size_t index_path_cap)
{
    char run_dir[MAX_PATH_LEN];
    char artifacts_root[MAX_PATH_LEN];
    if (!ws_id || !metadata_log || !linkage_log || !index_path)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(artifacts_root, sizeof(artifacts_root), "%s/artifacts", run_dir) <= 0)
        return -1;
    if (snprintf(metadata_log, metadata_log_cap, "%s/metadata.v1.ndjson", artifacts_root) <= 0)
        return -1;
    if (snprintf(linkage_log, linkage_log_cap, "%s/linkage.v1.ndjson", artifacts_root) <= 0)
        return -1;
    if (snprintf(index_path, index_path_cap, "%s/metadata.index.v1.json", artifacts_root) <= 0)
        return -1;
    return 0;
}

static void yai_slugify_token(const char *in, char *out, size_t out_cap)
{
    size_t i;
    size_t j = 0;
    if (!out || out_cap == 0) return;
    out[0] = '\0';
    if (!in || !in[0])
    {
        (void)snprintf(out, out_cap, "%s", "unknown");
        return;
    }
    for (i = 0; in[i] && j + 1 < out_cap; ++i)
    {
        const unsigned char c = (unsigned char)in[i];
        if (isalnum(c))
            out[j++] = (char)tolower(c);
        else if (c == '-' || c == '_' || c == '.')
            out[j++] = (char)c;
        else
            out[j++] = '_';
    }
    out[j] = '\0';
}

static int yai_workspace_append_authority_artifact_persistence(const char *ws_id,
                                                               const yai_governance_resolution_output_t *law_out,
                                                               const char *governance_refs_csv,
                                                               const char *event_ref,
                                                               const char *decision_ref,
                                                               const char *evidence_ref,
                                                               char *err,
                                                               size_t err_cap)
{
    char authority_log[MAX_PATH_LEN];
    char authority_resolution_log[MAX_PATH_LEN];
    char authority_index[MAX_PATH_LEN];
    char artifact_log[MAX_PATH_LEN];
    char artifact_linkage_log[MAX_PATH_LEN];
    char artifact_index[MAX_PATH_LEN];
    char ts_iso[48];
    char authority_id[128];
    char authority_resolution_id[128];
    char artifact_slug[128];
    char artifact_id[192];
    char artifact_link_id[224];
    char authority_lifecycle[384];
    char authority_resolution_lifecycle[384];
    char artifact_lifecycle[384];
    char artifact_link_lifecycle[384];
    char authority_row[3072];
    char authority_resolution_row[3072];
    char artifact_row[3584];
    char artifact_link_row[3072];
    char authority_index_row[2048];
    char artifact_index_row[2048];
    time_t now;

    if (err && err_cap > 0) err[0] = '\0';
    if (!ws_id || !law_out)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "authority_artifact_bad_args");
        return -1;
    }
    if (yai_workspace_authority_sink_paths(ws_id,
                                           authority_log,
                                           sizeof(authority_log),
                                           authority_resolution_log,
                                           sizeof(authority_resolution_log),
                                           authority_index,
                                           sizeof(authority_index)) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "authority_sink_path_failed");
        return -1;
    }
    if (yai_workspace_artifact_sink_paths(ws_id,
                                          artifact_log,
                                          sizeof(artifact_log),
                                          artifact_linkage_log,
                                          sizeof(artifact_linkage_log),
                                          artifact_index,
                                          sizeof(artifact_index)) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "artifact_sink_path_failed");
        return -1;
    }

    now = time(NULL);
    yai_format_iso8601_utc(now, ts_iso, sizeof(ts_iso));
    if (yai_lifecycle_build_json_fragment("authority_state", "workspace_id,authority_id", authority_lifecycle, sizeof(authority_lifecycle)) != 0 ||
        yai_lifecycle_build_json_fragment("authority_resolution_record", "workspace_id,time_window", authority_resolution_lifecycle, sizeof(authority_resolution_lifecycle)) != 0 ||
        yai_lifecycle_build_json_fragment("artifact_metadata", "workspace_id,artifact_id", artifact_lifecycle, sizeof(artifact_lifecycle)) != 0 ||
        yai_lifecycle_build_json_fragment("artifact_governance_linkage", "workspace_id,time_window", artifact_link_lifecycle, sizeof(artifact_link_lifecycle)) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "authority_artifact_lifecycle_encode_failed");
        return -1;
    }
    (void)snprintf(authority_id,
                   sizeof(authority_id),
                   "auth-%s-%s",
                   ws_id,
                   law_out->decision.decision_id[0] ? law_out->decision.decision_id : "unknown");
    (void)snprintf(authority_resolution_id,
                   sizeof(authority_resolution_id),
                   "ares-%s-%s",
                   ws_id,
                   law_out->decision.decision_id[0] ? law_out->decision.decision_id : "unknown");
    yai_slugify_token(law_out->evidence.resource, artifact_slug, sizeof(artifact_slug));
    (void)snprintf(artifact_id,
                   sizeof(artifact_id),
                   "art-%s-%s",
                   ws_id,
                   artifact_slug[0] ? artifact_slug : "unknown");
    (void)snprintf(artifact_link_id,
                   sizeof(artifact_link_id),
                   "alink-%s-%s",
                   ws_id,
                   law_out->decision.decision_id[0] ? law_out->decision.decision_id : "unknown");

    if (snprintf(authority_row,
                 sizeof(authority_row),
                 "{\"type\":\"yai.authority_state.v1\",\"schema_version\":\"v1\","
                 "\"authority_id\":\"%s\",\"authority_type\":\"runtime_profile\","
                 "\"scope\":\"workspace\",\"workspace_ref\":\"%s\","
                 "\"governance_object_refs\":\"%s\","
                 "\"status\":\"active\",\"source_ref\":\"%s\","
                 "\"approval_ref\":\"%s\",\"authority_profile\":\"%s\","
                 "\"created_at\":\"%s\",\"updated_at\":\"%s\","
                 "%s}",
                 authority_id,
                 ws_id,
                 governance_refs_csv ? governance_refs_csv : "",
                 law_out->evidence.trace_id,
                 governance_refs_csv ? governance_refs_csv : "",
                 law_out->decision.stack.authority_profile,
                 ts_iso,
                 ts_iso,
                 authority_lifecycle) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "authority_encode_failed");
        return -1;
    }

    if (snprintf(authority_resolution_row,
                 sizeof(authority_resolution_row),
                 "{\"type\":\"yai.authority_resolution_record.v1\",\"schema_version\":\"v1\","
                 "\"authority_resolution_id\":\"%s\",\"authority_id\":\"%s\","
                 "\"workspace_ref\":\"%s\",\"governance_object_refs\":\"%s\","
                 "\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\",\"event_ref\":\"%s\","
                 "\"effect\":\"%s\",\"rationale\":\"%s\",\"status\":\"resolved\","
                 "\"updated_at\":\"%s\","
                 "%s}",
                 authority_resolution_id,
                 authority_id,
                 ws_id,
                 governance_refs_csv ? governance_refs_csv : "",
                 decision_ref ? decision_ref : "",
                 evidence_ref ? evidence_ref : "",
                 event_ref ? event_ref : "",
                 yai_governance_effect_name(law_out->decision.final_effect),
                 law_out->decision.rationale,
                 ts_iso,
                 authority_resolution_lifecycle) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "authority_resolution_encode_failed");
        return -1;
    }

    if (snprintf(artifact_row,
                 sizeof(artifact_row),
                 "{\"type\":\"yai.artifact_metadata.v1\",\"schema_version\":\"v1\","
                 "\"artifact_id\":\"%s\",\"artifact_type\":\"runtime_action_artifact\","
                 "\"workspace_ref\":\"%s\",\"owner_ref\":\"workspace:%s\","
                 "\"canonical_ref\":\"resource://%s\",\"status\":\"evidence_linked\","
                 "\"governance_refs\":\"%s\",\"authority_ref\":\"%s\","
                 "\"decision_refs\":\"%s\",\"evidence_refs\":\"%s\","
                 "\"created_at\":\"%s\",\"updated_at\":\"%s\","
                 "%s}",
                 artifact_id,
                 ws_id,
                 ws_id,
                 artifact_slug[0] ? artifact_slug : "unknown",
                 governance_refs_csv ? governance_refs_csv : "",
                 authority_id,
                 decision_ref ? decision_ref : "",
                 evidence_ref ? evidence_ref : "",
                 ts_iso,
                 ts_iso,
                 artifact_lifecycle) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "artifact_metadata_encode_failed");
        return -1;
    }

    if (snprintf(artifact_link_row,
                 sizeof(artifact_link_row),
                 "{\"type\":\"yai.artifact_governance_linkage.v1\",\"schema_version\":\"v1\","
                 "\"linkage_id\":\"%s\",\"artifact_id\":\"%s\",\"workspace_ref\":\"%s\","
                 "\"governance_refs\":\"%s\",\"authority_ref\":\"%s\","
                 "\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\",\"event_ref\":\"%s\","
                 "\"status\":\"active\",\"updated_at\":\"%s\","
                 "%s}",
                 artifact_link_id,
                 artifact_id,
                 ws_id,
                 governance_refs_csv ? governance_refs_csv : "",
                 authority_id,
                 decision_ref ? decision_ref : "",
                 evidence_ref ? evidence_ref : "",
                 event_ref ? event_ref : "",
                 ts_iso,
                 artifact_link_lifecycle) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "artifact_linkage_encode_failed");
        return -1;
    }

    if (yai_append_json_line(authority_log, authority_row) != 0 ||
        yai_append_json_line(authority_resolution_log, authority_resolution_row) != 0 ||
        yai_append_json_line(artifact_log, artifact_row) != 0 ||
        yai_append_json_line(artifact_linkage_log, artifact_link_row) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "authority_artifact_append_failed");
        return -1;
    }

    if (snprintf(authority_index_row,
                 sizeof(authority_index_row),
                 "{\"type\":\"yai.authority.index.v1\",\"workspace_id\":\"%s\","
                 "\"last_authority_ref\":\"%s\",\"last_resolution_ref\":\"%s\","
                 "\"last_event_ref\":\"%s\",\"last_decision_ref\":\"%s\",\"last_evidence_ref\":\"%s\","
                 "\"retention_profile_ref\":\"matrix:data-lifecycle-retention-v0.1.0\","
                 "\"updated_at\":\"%s\","
                 "\"stores\":{\"authority\":\"%s\",\"resolution\":\"%s\"}}",
                 ws_id,
                 authority_id,
                 authority_resolution_id,
                 event_ref ? event_ref : "",
                 decision_ref ? decision_ref : "",
                 evidence_ref ? evidence_ref : "",
                 ts_iso,
                 authority_log,
                 authority_resolution_log) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "authority_index_encode_failed");
        return -1;
    }
    if (snprintf(artifact_index_row,
                 sizeof(artifact_index_row),
                 "{\"type\":\"yai.artifact.metadata.index.v1\",\"workspace_id\":\"%s\","
                 "\"last_artifact_ref\":\"%s\",\"last_linkage_ref\":\"%s\","
                 "\"last_event_ref\":\"%s\",\"last_decision_ref\":\"%s\",\"last_evidence_ref\":\"%s\","
                 "\"retention_profile_ref\":\"matrix:data-lifecycle-retention-v0.1.0\","
                 "\"updated_at\":\"%s\","
                 "\"stores\":{\"metadata\":\"%s\",\"linkage\":\"%s\"}}",
                 ws_id,
                 artifact_id,
                 artifact_link_id,
                 event_ref ? event_ref : "",
                 decision_ref ? decision_ref : "",
                 evidence_ref ? evidence_ref : "",
                 ts_iso,
                 artifact_log,
                 artifact_linkage_log) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "artifact_index_encode_failed");
        return -1;
    }
    {
        FILE *f = fopen(authority_index, "w");
        if (!f)
        {
            if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "authority_index_write_failed");
            return -1;
        }
        (void)fprintf(f, "%s\n", authority_index_row);
        fclose(f);
    }
    {
        FILE *f = fopen(artifact_index, "w");
        if (!f)
        {
            if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "artifact_index_write_failed");
            return -1;
        }
        (void)fprintf(f, "%s\n", artifact_index_row);
        fclose(f);
    }
    return 0;
}

static void yai_workspace_read_authority_artifact_indexes(const yai_workspace_runtime_info_t *info,
                                                          char *authority_ref,
                                                          size_t authority_ref_cap,
                                                          char *authority_resolution_ref,
                                                          size_t authority_resolution_ref_cap,
                                                          char *artifact_ref,
                                                          size_t artifact_ref_cap,
                                                          char *artifact_linkage_ref,
                                                          size_t artifact_linkage_ref_cap,
                                                          char *authority_store_ref,
                                                          size_t authority_store_ref_cap,
                                                          char *artifact_store_ref,
                                                          size_t artifact_store_ref_cap)
{
    char authority_log[MAX_PATH_LEN];
    char authority_resolution_log[MAX_PATH_LEN];
    char authority_index[MAX_PATH_LEN];
    char artifact_log[MAX_PATH_LEN];
    char artifact_linkage_log[MAX_PATH_LEN];
    char artifact_index[MAX_PATH_LEN];
    char buf[2048];

    if (authority_ref && authority_ref_cap > 0) authority_ref[0] = '\0';
    if (authority_resolution_ref && authority_resolution_ref_cap > 0) authority_resolution_ref[0] = '\0';
    if (artifact_ref && artifact_ref_cap > 0) artifact_ref[0] = '\0';
    if (artifact_linkage_ref && artifact_linkage_ref_cap > 0) artifact_linkage_ref[0] = '\0';
    if (authority_store_ref && authority_store_ref_cap > 0) authority_store_ref[0] = '\0';
    if (artifact_store_ref && artifact_store_ref_cap > 0) artifact_store_ref[0] = '\0';
    if (!info || !info->ws_id[0]) return;

    if (yai_workspace_authority_sink_paths(info->ws_id,
                                           authority_log,
                                           sizeof(authority_log),
                                           authority_resolution_log,
                                           sizeof(authority_resolution_log),
                                           authority_index,
                                           sizeof(authority_index)) == 0)
    {
        if (authority_store_ref && authority_store_ref_cap > 0)
            (void)snprintf(authority_store_ref, authority_store_ref_cap, "%s", authority_log);
        if (yai_read_text(authority_index, buf, sizeof(buf)) == 0)
        {
            if (authority_ref && authority_ref_cap > 0)
                (void)yai_session_extract_json_string(buf, "last_authority_ref", authority_ref, authority_ref_cap);
            if (authority_resolution_ref && authority_resolution_ref_cap > 0)
                (void)yai_session_extract_json_string(buf, "last_resolution_ref", authority_resolution_ref, authority_resolution_ref_cap);
        }
    }

    if (yai_workspace_artifact_sink_paths(info->ws_id,
                                          artifact_log,
                                          sizeof(artifact_log),
                                          artifact_linkage_log,
                                          sizeof(artifact_linkage_log),
                                          artifact_index,
                                          sizeof(artifact_index)) == 0)
    {
        if (artifact_store_ref && artifact_store_ref_cap > 0)
            (void)snprintf(artifact_store_ref, artifact_store_ref_cap, "%s", artifact_log);
        if (yai_read_text(artifact_index, buf, sizeof(buf)) == 0)
        {
            if (artifact_ref && artifact_ref_cap > 0)
                (void)yai_session_extract_json_string(buf, "last_artifact_ref", artifact_ref, artifact_ref_cap);
            if (artifact_linkage_ref && artifact_linkage_ref_cap > 0)
                (void)yai_session_extract_json_string(buf, "last_linkage_ref", artifact_linkage_ref, artifact_linkage_ref_cap);
        }
    }
}

static int yai_workspace_enforcement_sink_paths(const char *ws_id,
                                                char *outcomes_log,
                                                size_t outcomes_log_cap,
                                                char *linkage_log,
                                                size_t linkage_log_cap,
                                                char *index_path,
                                                size_t index_path_cap)
{
    char run_dir[MAX_PATH_LEN];
    char root[MAX_PATH_LEN];
    if (!ws_id || !outcomes_log || !linkage_log || !index_path)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(root, sizeof(root), "%s/enforcement", run_dir) <= 0)
        return -1;
    if (mkdir_if_missing(root, 0755) != 0)
        return -1;
    if (snprintf(outcomes_log, outcomes_log_cap, "%s/outcome-records.v1.ndjson", root) <= 0)
        return -1;
    if (snprintf(linkage_log, linkage_log_cap, "%s/linkage-records.v1.ndjson", root) <= 0)
        return -1;
    if (snprintf(index_path, index_path_cap, "%s/index.v1.json", root) <= 0)
        return -1;
    return 0;
}

static int yai_env_truthy(const char *name)
{
    const char *v = getenv(name);
    if (!v || !v[0]) return 0;
    return strcmp(v, "0") != 0 && strcmp(v, "false") != 0 && strcmp(v, "FALSE") != 0;
}

static int yai_workspace_append_enforcement_record_set(const char *ws_id,
                                                       const yai_governance_resolution_output_t *law_out,
                                                       const char *governance_refs_csv,
                                                       const char *event_ref,
                                                       const char *decision_ref,
                                                       const char *evidence_ref,
                                                       const char *authority_ref,
                                                       const char *authority_resolution_ref,
                                                       const char *artifact_ref,
                                                       const char *artifact_linkage_ref,
                                                       char *err,
                                                       size_t err_cap)
{
    char outcomes_log[MAX_PATH_LEN];
    char linkage_log[MAX_PATH_LEN];
    char index_path[MAX_PATH_LEN];
    char ts_iso[48];
    char outcome_id[128];
    char linkage_id[128];
    char out_row[3072];
    char link_row[3072];
    char out_lifecycle[384];
    char link_lifecycle[384];
    char idx_row[4608];
    char missing[256];
    const char *effect;
    const char *evidence_ref_use;
    int has_event = 0;
    int has_decision = 0;
    int has_evidence = 0;
    int has_authority = 0;
    int has_artifact = 0;
    int complete = 0;
    int force_partial = yai_env_truthy("YAI_ENFORCEMENT_RECORD_FORCE_PARTIAL");
    time_t now;

    if (err && err_cap > 0) err[0] = '\0';
    if (!ws_id || !law_out)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "enforcement_record_set_bad_args");
        return -1;
    }
    if (yai_workspace_enforcement_sink_paths(ws_id,
                                             outcomes_log,
                                             sizeof(outcomes_log),
                                             linkage_log,
                                             sizeof(linkage_log),
                                             index_path,
                                             sizeof(index_path)) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "enforcement_record_set_path_failed");
        return -1;
    }

    now = time(NULL);
    yai_format_iso8601_utc(now, ts_iso, sizeof(ts_iso));
    if (yai_lifecycle_build_json_fragment("enforcement_outcome_record", "workspace_id,time_window", out_lifecycle, sizeof(out_lifecycle)) != 0 ||
        yai_lifecycle_build_json_fragment("enforcement_linkage_record", "workspace_id,time_window", link_lifecycle, sizeof(link_lifecycle)) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "enforcement_lifecycle_encode_failed");
        return -1;
    }
    effect = yai_governance_effect_name(law_out->decision.final_effect);
    evidence_ref_use = force_partial ? "" : (evidence_ref ? evidence_ref : "");

    (void)snprintf(outcome_id, sizeof(outcome_id), "enf-%s", law_out->decision.decision_id);
    (void)snprintf(linkage_id, sizeof(linkage_id), "enl-%s", law_out->decision.decision_id);

    has_event = event_ref && event_ref[0];
    has_decision = decision_ref && decision_ref[0];
    has_evidence = evidence_ref_use && evidence_ref_use[0];
    has_authority = authority_ref && authority_ref[0] && authority_resolution_ref && authority_resolution_ref[0];
    has_artifact = artifact_ref && artifact_ref[0] && artifact_linkage_ref && artifact_linkage_ref[0];
    complete = has_event && has_decision && has_evidence && has_authority && has_artifact;

    missing[0] = '\0';
    if (!has_event) (void)snprintf(missing + strlen(missing), sizeof(missing) - strlen(missing), "%sevent_ref", missing[0] ? "," : "");
    if (!has_decision) (void)snprintf(missing + strlen(missing), sizeof(missing) - strlen(missing), "%sdecision_ref", missing[0] ? "," : "");
    if (!has_evidence) (void)snprintf(missing + strlen(missing), sizeof(missing) - strlen(missing), "%sevidence_ref", missing[0] ? "," : "");
    if (!has_authority) (void)snprintf(missing + strlen(missing), sizeof(missing) - strlen(missing), "%sauthority_linkage", missing[0] ? "," : "");
    if (!has_artifact) (void)snprintf(missing + strlen(missing), sizeof(missing) - strlen(missing), "%sartifact_linkage", missing[0] ? "," : "");

    if (snprintf(out_row,
                 sizeof(out_row),
                 "{\"type\":\"yai.enforcement_outcome_record.v1\",\"schema_version\":\"v1\","
                 "\"outcome_id\":\"%s\",\"workspace_ref\":\"%s\",\"decision_ref\":\"%s\","
                 "\"event_ref\":\"%s\",\"evidence_ref\":\"%s\",\"effect\":\"%s\","
                 "\"family_id\":\"%s\",\"specialization_id\":\"%s\","
                 "\"materialization_status\":\"%s\",\"missing_fields\":\"%s\","
                 "\"rationale\":\"%s\",\"trace_ref\":\"%s\",\"created_at\":\"%s\","
                 "%s}",
                 outcome_id,
                 ws_id,
                 decision_ref ? decision_ref : "",
                 event_ref ? event_ref : "",
                 evidence_ref_use,
                 effect,
                 law_out->decision.family_id,
                 law_out->decision.specialization_id,
                 complete ? "complete" : "incomplete",
                 missing,
                 law_out->decision.rationale,
                 law_out->evidence.trace_id,
                 ts_iso,
                 out_lifecycle) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "enforcement_outcome_encode_failed");
        return -1;
    }

    if (snprintf(link_row,
                 sizeof(link_row),
                 "{\"type\":\"yai.enforcement_linkage_record.v1\",\"schema_version\":\"v1\","
                 "\"linkage_id\":\"%s\",\"workspace_ref\":\"%s\",\"decision_ref\":\"%s\","
                 "\"governance_refs\":\"%s\",\"authority_ref\":\"%s\",\"authority_resolution_ref\":\"%s\","
                 "\"artifact_ref\":\"%s\",\"artifact_linkage_ref\":\"%s\","
                 "\"event_ref\":\"%s\",\"evidence_ref\":\"%s\","
                 "\"trace_ref\":\"%s\",\"created_at\":\"%s\","
                 "%s}",
                 linkage_id,
                 ws_id,
                 decision_ref ? decision_ref : "",
                 governance_refs_csv ? governance_refs_csv : "",
                 authority_ref ? authority_ref : "",
                 authority_resolution_ref ? authority_resolution_ref : "",
                 artifact_ref ? artifact_ref : "",
                 artifact_linkage_ref ? artifact_linkage_ref : "",
                 event_ref ? event_ref : "",
                 evidence_ref_use,
                 law_out->evidence.trace_id,
                 ts_iso,
                 link_lifecycle) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "enforcement_linkage_encode_failed");
        return -1;
    }

    if (yai_append_json_line(outcomes_log, out_row) != 0 ||
        yai_append_json_line(linkage_log, link_row) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "enforcement_record_set_append_failed");
        return -1;
    }

    if (snprintf(idx_row,
                 sizeof(idx_row),
                 "{"
                 "\"type\":\"yai.enforcement.recordset.index.v1\","
                 "\"workspace_id\":\"%s\","
                 "\"last_outcome_ref\":\"%s\","
                 "\"last_linkage_ref\":\"%s\","
                 "\"last_event_ref\":\"%s\","
                 "\"last_decision_ref\":\"%s\","
                "\"last_evidence_ref\":\"%s\","
                "\"materialization_status\":\"%s\","
                "\"missing_fields\":\"%s\","
                "\"retention_profile_ref\":\"matrix:data-lifecycle-retention-v0.1.0\","
                "\"updated_at\":\"%s\","
                 "\"stores\":{"
                   "\"outcomes\":\"%s\","
                   "\"linkage\":\"%s\""
                 "}"
                 "}",
                 ws_id,
                 outcome_id,
                 linkage_id,
                 event_ref ? event_ref : "",
                 decision_ref ? decision_ref : "",
                 evidence_ref_use,
                 complete ? "complete" : "incomplete",
                 missing,
                 ts_iso,
                 outcomes_log,
                 linkage_log) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "enforcement_index_encode_failed");
        return -1;
    }
    {
        FILE *f = fopen(index_path, "w");
        if (!f)
        {
            if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "enforcement_index_write_failed");
            return -1;
        }
        (void)fprintf(f, "%s\n", idx_row);
        fclose(f);
    }
    return 0;
}

static void yai_workspace_read_enforcement_index(const yai_workspace_runtime_info_t *info,
                                                 char *outcome_ref,
                                                 size_t outcome_ref_cap,
                                                 char *linkage_ref,
                                                 size_t linkage_ref_cap,
                                                 char *materialization_status,
                                                 size_t materialization_status_cap,
                                                 char *missing_fields,
                                                 size_t missing_fields_cap,
                                                 char *outcome_store_ref,
                                                 size_t outcome_store_ref_cap,
                                                 char *linkage_store_ref,
                                                 size_t linkage_store_ref_cap)
{
    char outcomes_log[MAX_PATH_LEN];
    char linkage_log[MAX_PATH_LEN];
    char index_path[MAX_PATH_LEN];
    char buf[4096];

    if (outcome_ref && outcome_ref_cap > 0) outcome_ref[0] = '\0';
    if (linkage_ref && linkage_ref_cap > 0) linkage_ref[0] = '\0';
    if (materialization_status && materialization_status_cap > 0) materialization_status[0] = '\0';
    if (missing_fields && missing_fields_cap > 0) missing_fields[0] = '\0';
    if (outcome_store_ref && outcome_store_ref_cap > 0) outcome_store_ref[0] = '\0';
    if (linkage_store_ref && linkage_store_ref_cap > 0) linkage_store_ref[0] = '\0';
    if (!info || !info->ws_id[0]) return;
    if (yai_workspace_enforcement_sink_paths(info->ws_id,
                                             outcomes_log,
                                             sizeof(outcomes_log),
                                             linkage_log,
                                             sizeof(linkage_log),
                                             index_path,
                                             sizeof(index_path)) != 0)
        return;

    if (outcome_store_ref && outcome_store_ref_cap > 0)
        (void)snprintf(outcome_store_ref, outcome_store_ref_cap, "%s", outcomes_log);
    if (linkage_store_ref && linkage_store_ref_cap > 0)
        (void)snprintf(linkage_store_ref, linkage_store_ref_cap, "%s", linkage_log);

    if (yai_read_text(index_path, buf, sizeof(buf)) != 0) return;
    if (outcome_ref && outcome_ref_cap > 0)
        (void)yai_session_extract_json_string(buf, "last_outcome_ref", outcome_ref, outcome_ref_cap);
    if (linkage_ref && linkage_ref_cap > 0)
        (void)yai_session_extract_json_string(buf, "last_linkage_ref", linkage_ref, linkage_ref_cap);
    if (materialization_status && materialization_status_cap > 0)
        (void)yai_session_extract_json_string(buf, "materialization_status", materialization_status, materialization_status_cap);
    if (missing_fields && missing_fields_cap > 0)
        (void)yai_session_extract_json_string(buf, "missing_fields", missing_fields, missing_fields_cap);
}

static int yai_ref_present(const char *value)
{
    return value && value[0];
}

static void yai_append_gap_reason(char *out, size_t out_cap, const char *reason)
{
    size_t len;
    if (!out || out_cap == 0 || !reason || !reason[0])
        return;
    len = strlen(out);
    if (len > 0 && strcmp(out, "none") != 0 && len + 1 < out_cap)
    {
        out[len++] = ',';
        out[len] = '\0';
    }
    if (strcmp(out, "none") == 0)
    {
        out[0] = '\0';
        len = 0;
    }
    if (len + strlen(reason) + 1 < out_cap)
        (void)snprintf(out + len, out_cap - len, "%s", reason);
}

static void yai_workspace_db_first_read_model(const char *query_family,
                                              const char *event_ref,
                                              const char *decision_ref,
                                              const char *evidence_ref,
                                              const char *governance_object_ref,
                                              const char *governance_lifecycle_ref,
                                              const char *authority_ref,
                                              const char *artifact_ref,
                                              const char *graph_node_ref,
                                              const char *graph_edge_ref,
                                              const char *enforcement_outcome_ref,
                                              const char *enforcement_linkage_ref,
                                              const char *enforcement_materialization_status,
                                              char *primary_source,
                                              size_t primary_source_cap,
                                              char *fallback_reason,
                                              size_t fallback_reason_cap,
                                              int *db_first_ready,
                                              int *fallback_active)
{
    const char *family = (query_family && query_family[0]) ? query_family : "workspace";
    int ready = 1;

    if (fallback_reason && fallback_reason_cap > 0)
        (void)snprintf(fallback_reason, fallback_reason_cap, "%s", "none");

    if (strcmp(family, "events") == 0 || strcmp(family, "evidence") == 0)
    {
        if (!yai_ref_present(event_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_event_ref");
        }
        if (!yai_ref_present(decision_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_decision_ref");
        }
        if (!yai_ref_present(evidence_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_evidence_ref");
        }
    }
    else if (strcmp(family, "governance") == 0)
    {
        if (!yai_ref_present(governance_object_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_governance_object_ref");
        }
        if (!yai_ref_present(governance_lifecycle_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_governance_lifecycle_ref");
        }
    }
    else if (strcmp(family, "authority") == 0)
    {
        if (!yai_ref_present(authority_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_authority_ref");
        }
    }
    else if (strcmp(family, "artifacts") == 0)
    {
        if (!yai_ref_present(artifact_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_artifact_ref");
        }
    }
    else if (strcmp(family, "graph") == 0 || strncmp(family, "graph.", 6) == 0)
    {
        if (!yai_ref_present(graph_node_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_graph_node_ref");
        }
        if (!yai_ref_present(graph_edge_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_graph_edge_ref");
        }
    }
    else if (strcmp(family, "lifecycle") == 0)
    {
        ready = 1;
    }
    else if (strcmp(family, "enforcement") == 0)
    {
        if (!yai_ref_present(enforcement_outcome_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_enforcement_outcome_ref");
        }
        if (!yai_ref_present(enforcement_linkage_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_enforcement_linkage_ref");
        }
        if (!enforcement_materialization_status || strcmp(enforcement_materialization_status, "complete") != 0)
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "enforcement_incomplete");
        }
    }
    else
    {
        if (!yai_ref_present(event_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_event_ref");
        }
        if (!yai_ref_present(decision_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_decision_ref");
        }
        if (!yai_ref_present(evidence_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_evidence_ref");
        }
        if (!yai_ref_present(enforcement_outcome_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_enforcement_outcome_ref");
        }
        if (!yai_ref_present(enforcement_linkage_ref))
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "missing_enforcement_linkage_ref");
        }
        if (!enforcement_materialization_status || strcmp(enforcement_materialization_status, "complete") != 0)
        {
            ready = 0;
            yai_append_gap_reason(fallback_reason, fallback_reason_cap, "enforcement_incomplete");
        }
    }

    if (primary_source && primary_source_cap > 0)
    {
        (void)snprintf(primary_source,
                       primary_source_cap,
                       "%s",
                       ready ? "data_plane_persisted_records" : "data_plane_partial_runtime_fallback");
    }
    if (db_first_ready) *db_first_ready = ready;
    if (fallback_active) *fallback_active = ready ? 0 : 1;
}
