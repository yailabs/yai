int yai_session_set_workspace_declared_context(const char *family,
                                               const char *specialization,
                                               char *out_json,
                                               size_t out_cap,
                                               char *err,
                                               size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char bind_err[96];
    char resolved_family[96] = {0};
    int n;
    if (err && err_cap > 0)
        err[0] = '\0';

    if ((!family || !family[0]) && (!specialization || !specialization[0]))
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "family_or_specialization_required");
        return -1;
    }

    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), bind_err, sizeof(bind_err)) != 0 ||
        strcmp(status, "active") != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_not_active");
        return -1;
    }

    if (family && family[0])
        snprintf(resolved_family, sizeof(resolved_family), "%s", family);
    else
        snprintf(resolved_family, sizeof(resolved_family), "%s", info.declared_control_family);

    /* Deterministic error precedence: explicit family validity is checked before specialization matching. */
    if (family && family[0] && !yai_governance_family_exists(resolved_family))
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "family_not_found");
        return -1;
    }

    if (specialization && specialization[0])
    {
        char inferred_family[96] = {0};
        if (yai_governance_resolve_specialization_family(specialization, inferred_family, sizeof(inferred_family)) != 0)
        {
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", "specialization_not_found");
            return -1;
        }
        if (!resolved_family[0])
            snprintf(resolved_family, sizeof(resolved_family), "%s", inferred_family);
        if (strcmp(resolved_family, inferred_family) != 0)
        {
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", "specialization_family_mismatch");
            return -1;
        }
    }

    if (resolved_family[0] && !yai_governance_family_exists(resolved_family))
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "family_not_found");
        return -1;
    }
    if (!yai_governance_specialization_matches_family(resolved_family, specialization && specialization[0] ? specialization : info.declared_specialization))
    {
        if (!specialization || !specialization[0])
            info.declared_specialization[0] = '\0';
        else
        {
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", "specialization_family_mismatch");
            return -1;
        }
    }

    if (resolved_family[0])
        snprintf(info.declared_control_family, sizeof(info.declared_control_family), "%s", resolved_family);
    if (specialization && specialization[0])
        snprintf(info.declared_specialization, sizeof(info.declared_specialization), "%s", specialization);
    snprintf(info.declared_context_source, sizeof(info.declared_context_source), "%s", "declared");
    info.updated_at = (long)time(NULL);
    info.activated_at = info.activated_at ? info.activated_at : info.updated_at;
    info.exists = 1;
    snprintf(info.session_binding, sizeof(info.session_binding), "%s", info.ws_id);

    if (yai_workspace_write_manifest_ws_id(info.ws_id, &info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "manifest_write_failed");
        return -1;
    }
    if (yai_workspace_write_containment_surfaces(&info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "containment_write_failed");
        return -1;
    }

    if (out_json && out_cap > 0)
    {
        n = snprintf(out_json,
                     out_cap,
                     "{\"type\":\"yai.workspace.domain.set.v1\",\"workspace_id\":\"%s\",\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\",\"source\":\"%s\"}}",
                     info.ws_id,
                     info.declared_control_family,
                     info.declared_specialization,
                     info.declared_context_source);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    return 0;
}

int yai_session_workspace_policy_attachment_update(const char *object_id,
                                                   int attach_mode,
                                                   char *out_json,
                                                   size_t out_cap,
                                                   char *err,
                                                   size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    yai_governable_object_meta_t meta;
    char status[24];
    char bind_err[96];
    char eligibility[64];
    char compatibility[64];
    char conflicts[160];
    char warnings[160];
    const char *attachment_state = "attached_inactive";
    const char *action = "attach";
    char sink_last_event_ref[96];
    char sink_last_decision_ref[96];
    char sink_last_evidence_ref[96];
    char gov_last_object_ref[256];
    char gov_last_lifecycle_ref[256];
    char gov_last_attachment_ref[320];
    int blocking = 0;
    int already_attached = 0;
    int n;
    int op_rc;

    if (err && err_cap > 0) err[0] = '\0';
    if (!object_id || !object_id[0])
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "object_id_required");
        return -1;
    }
    if (!yai_policy_attachment_id_valid(object_id))
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "invalid_object_id");
        return -1;
    }
    yai_governable_meta_defaults(&meta);
    if (!yai_governance_governable_object_lookup(object_id, &meta))
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "governable_object_not_found");
        return -1;
    }
    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), bind_err, sizeof(bind_err)) != 0 ||
        strcmp(status, "active") != 0)
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_not_active");
        return -1;
    }

    already_attached = yai_policy_attachment_csv_contains(info.policy_attachments_csv, object_id);
    if (attach_mode == 1)
    {
        action = "attach";
        if (yai_workspace_policy_evaluate(&info,
                                          &meta,
                                          eligibility,
                                          sizeof(eligibility),
                                          compatibility,
                                          sizeof(compatibility),
                                          conflicts,
                                          sizeof(conflicts),
                                          warnings,
                                          sizeof(warnings),
                                          &blocking) != 0)
        {
            if (err && err_cap > 0) snprintf(err, err_cap, "%s", "policy_evaluation_failed");
            return -1;
        }
        if (blocking)
        {
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", conflicts[0] ? conflicts : "conflict_blocking");
            return -1;
        }
        if (!yai_policy_attachment_csv_contains(info.policy_attachments_csv, object_id) &&
            yai_policy_attachment_csv_count(info.policy_attachments_csv) >= YAI_POLICY_ATTACHMENTS_MAX)
        {
            if (err && err_cap > 0) snprintf(err, err_cap, "%s", "attachment_limit_reached");
            return -1;
        }
        op_rc = yai_policy_attachment_csv_add(info.policy_attachments_csv,
                                              sizeof(info.policy_attachments_csv),
                                              object_id);
        if (op_rc != 0)
        {
            if (err && err_cap > 0) snprintf(err, err_cap, "%s", "attachment_add_failed");
            return -1;
        }
    }
    else if (attach_mode == 2)
    {
        action = "activate";
        if (!yai_policy_attachment_csv_contains(info.policy_attachments_csv, object_id))
        {
            if (err && err_cap > 0) snprintf(err, err_cap, "%s", "attachment_not_found");
            return -1;
        }
        snprintf(eligibility, sizeof(eligibility), "%s", "eligible");
        snprintf(compatibility, sizeof(compatibility), "%s", "dry_run_passed");
        snprintf(conflicts, sizeof(conflicts), "%s", "none");
        warnings[0] = '\0';
    }
    else
    {
        action = "detach";
        op_rc = yai_policy_attachment_csv_remove(info.policy_attachments_csv,
                                                 sizeof(info.policy_attachments_csv),
                                                 object_id);
        if (op_rc == 1)
        {
            if (err && err_cap > 0) snprintf(err, err_cap, "%s", "attachment_not_found");
            return -1;
        }
        if (op_rc != 0)
        {
            if (err && err_cap > 0) snprintf(err, err_cap, "%s", "attachment_remove_failed");
            return -1;
        }
        snprintf(eligibility, sizeof(eligibility), "%s", "eligible");
        snprintf(compatibility, sizeof(compatibility), "%s", "dry_run_passed");
        snprintf(conflicts, sizeof(conflicts), "%s", "none");
        warnings[0] = '\0';
    }

    info.policy_attachment_count = yai_policy_attachment_csv_count(info.policy_attachments_csv);
    attachment_state = yai_workspace_attachment_state_for_meta(&meta,
                                                               yai_policy_attachment_csv_contains(info.policy_attachments_csv, object_id));
    info.updated_at = (long)time(NULL);
    if (yai_workspace_write_manifest_ws_id(info.ws_id, &info) != 0)
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "manifest_write_failed");
        return -1;
    }
    if (yai_workspace_write_containment_surfaces(&info) != 0)
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "containment_write_failed");
        return -1;
    }
    yai_workspace_read_event_evidence_index(&info,
                                            sink_last_event_ref,
                                            sizeof(sink_last_event_ref),
                                            sink_last_decision_ref,
                                            sizeof(sink_last_decision_ref),
                                            sink_last_evidence_ref,
                                            sizeof(sink_last_evidence_ref),
                                            NULL,
                                            0,
                                            NULL,
                                            0,
                                            NULL,
                                            0);
    if (yai_workspace_append_governance_persistence(info.ws_id,
                                                    &meta,
                                                    object_id,
                                                    action,
                                                    attachment_state,
                                                    eligibility,
                                                    compatibility,
                                                    conflicts,
                                                    warnings,
                                                    sink_last_event_ref,
                                                    sink_last_decision_ref,
                                                    sink_last_evidence_ref,
                                                    err,
                                                    err_cap) != 0)
    {
        if (err && err_cap > 0 && err[0] == '\0')
            snprintf(err, err_cap, "%s", "governance_persistence_write_failed");
        return -1;
    }
    yai_workspace_read_governance_index(&info,
                                        gov_last_object_ref,
                                        sizeof(gov_last_object_ref),
                                        gov_last_lifecycle_ref,
                                        sizeof(gov_last_lifecycle_ref),
                                        gov_last_attachment_ref,
                                        sizeof(gov_last_attachment_ref),
                                        NULL,
                                        0,
                                        NULL,
                                        0,
                                        NULL,
                                        0);

    if (out_json && out_cap > 0)
    {
        if (attach_mode == 0)
        {
            snprintf(eligibility, sizeof(eligibility), "%s", "eligible");
            snprintf(compatibility, sizeof(compatibility), "%s", "dry_run_passed");
            snprintf(conflicts, sizeof(conflicts), "%s", "none");
            warnings[0] = '\0';
        }
        n = snprintf(out_json,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.policy.attachment.v1\","
                     "\"workspace_id\":\"%s\","
                     "\"binding_status\":\"active\","
                     "\"action\":\"%s\","
                     "\"object_id\":\"%s\","
                     "\"kind\":\"%s\","
                     "\"status\":\"%s\","
                     "\"review_state\":\"%s\","
                     "\"eligibility_result\":\"%s\","
                     "\"compatibility_result\":\"%s\","
                     "\"conflict_summary\":\"%s\","
                     "\"warnings\":\"%s\","
                     "\"attachment_state\":\"%s\","
                     "\"activation_state\":\"%s\","
                     "\"governance_object_ref\":\"%s\","
                     "\"governance_lifecycle_ref\":\"%s\","
                     "\"governance_attachment_ref\":\"%s\","
                     "\"already_attached\":%s,"
                     "\"attachment_valid\":true,"
                     "\"policy_attachments\":\"%s\","
                     "\"policy_attachment_count\":%d"
                     "}",
                     info.ws_id,
                     action,
                     object_id,
                     meta.kind,
                     meta.status,
                     meta.review_state,
                     eligibility,
                     compatibility,
                     conflicts,
                     warnings,
                     attachment_state,
                     yai_policy_attachment_csv_contains(info.policy_attachments_csv, object_id) ? "active" : "inactive",
                     gov_last_object_ref,
                     gov_last_lifecycle_ref,
                     gov_last_attachment_ref,
                     already_attached ? "true" : "false",
                     info.policy_attachments_csv,
                     info.policy_attachment_count);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    return 0;
}

int yai_session_workspace_policy_apply_dry_run(const char *object_id,
                                               char *out_json,
                                               size_t out_cap,
                                               char *err,
                                               size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    yai_governable_object_meta_t meta;
    char status[24];
    char bind_err[96];
    char eligibility[64];
    char compatibility[64];
    char conflicts[160];
    char warnings[160];
    char effective_family[96];
    char effective_specialization[96];
    int blocking = 0;
    int n;

    if (err && err_cap > 0) err[0] = '\0';
    if (!object_id || !object_id[0])
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "object_id_required");
        return -1;
    }
    if (!yai_policy_attachment_id_valid(object_id))
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "invalid_object_id");
        return -1;
    }
    yai_governable_meta_defaults(&meta);
    if (!yai_governance_governable_object_lookup(object_id, &meta))
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "governable_object_not_found");
        return -1;
    }
    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), bind_err, sizeof(bind_err)) != 0 ||
        strcmp(status, "active") != 0)
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_not_active");
        return -1;
    }

    if (yai_workspace_policy_evaluate(&info,
                                      &meta,
                                      eligibility,
                                      sizeof(eligibility),
                                      compatibility,
                                      sizeof(compatibility),
                                      conflicts,
                                      sizeof(conflicts),
                                      warnings,
                                      sizeof(warnings),
                                      &blocking) != 0)
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "policy_evaluation_failed");
        return -1;
    }
    yai_workspace_policy_effective_context(&info,
                                           effective_family,
                                           sizeof(effective_family),
                                           effective_specialization,
                                           sizeof(effective_specialization));

    if (out_json && out_cap > 0)
    {
        n = snprintf(out_json,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.policy.apply.dry_run.v1\","
                     "\"workspace_id\":\"%s\","
                     "\"object_id\":\"%s\","
                     "\"kind\":\"%s\","
                     "\"status\":\"%s\","
                     "\"review_state\":\"%s\","
                     "\"workspace_family\":\"%s\","
                     "\"workspace_specialization\":\"%s\","
                     "\"target_workspace_ids\":\"%s\","
                     "\"target_family\":\"%s\","
                     "\"target_specialization\":\"%s\","
                     "\"precedence_class\":\"%s\","
                     "\"attachment_modes\":\"%s\","
                     "\"eligibility_result\":\"%s\","
                     "\"compatibility_result\":\"%s\","
                     "\"conflict_summary\":\"%s\","
                     "\"warnings\":\"%s\","
                     "\"ready_for_attach\":%s,"
                     "\"activation_state\":\"%s\","
                     "\"effective_stack_ref\":\"%s\""
                     "}",
                     info.ws_id,
                     object_id,
                     meta.kind,
                     meta.status,
                     meta.review_state,
                     effective_family,
                     effective_specialization,
                     meta.workspace_targets_csv,
                     meta.family_targets_csv,
                     meta.specialization_targets_csv,
                     meta.precedence_class,
                     meta.attachment_modes_csv,
                     eligibility,
                     compatibility,
                     conflicts,
                     warnings,
                     blocking ? "false" : "true",
                     yai_policy_attachment_csv_contains(info.policy_attachments_csv, object_id) ? "active" : "inactive",
                     info.effective_stack_ref);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    return 0;
}

