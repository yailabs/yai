static int yai_policy_attachment_id_valid(const char *id)
{
    size_t i;
    if (!id || !id[0]) return 0;
    if (strlen(id) > 160u) return 0;
    for (i = 0; id[i]; i++)
    {
        const unsigned char c = (unsigned char)id[i];
        if (!(isalnum(c) || c == '.' || c == '-' || c == '_' || c == '/'))
            return 0;
    }
    return 1;
}

static int yai_policy_attachment_csv_count(const char *csv)
{
    int count = 0;
    char buf[sizeof(((yai_workspace_runtime_info_t *)0)->policy_attachments_csv)];
    char *tok;
    char *save = NULL;
    if (!csv || !csv[0]) return 0;
    snprintf(buf, sizeof(buf), "%s", csv);
    tok = strtok_r(buf, ",", &save);
    while (tok)
    {
        if (tok[0]) count++;
        tok = strtok_r(NULL, ",", &save);
    }
    return count;
}

static int yai_policy_attachment_csv_contains(const char *csv, const char *id)
{
    char buf[sizeof(((yai_workspace_runtime_info_t *)0)->policy_attachments_csv)];
    char *tok;
    char *save = NULL;
    if (!csv || !csv[0] || !id || !id[0]) return 0;
    snprintf(buf, sizeof(buf), "%s", csv);
    tok = strtok_r(buf, ",", &save);
    while (tok)
    {
        if (strcmp(tok, id) == 0) return 1;
        tok = strtok_r(NULL, ",", &save);
    }
    return 0;
}

static int yai_policy_attachment_csv_add(char *csv, size_t csv_cap, const char *id)
{
    size_t len;
    if (!csv || csv_cap == 0 || !id || !id[0]) return -1;
    if (yai_policy_attachment_csv_contains(csv, id)) return 0;
    if (csv[0])
    {
        len = strlen(csv);
        if (len + 1 + strlen(id) + 1 >= csv_cap) return -1;
        csv[len] = ',';
        csv[len + 1] = '\0';
    }
    if (strlen(csv) + strlen(id) + 1 >= csv_cap) return -1;
    strcat(csv, id);
    return 0;
}

static int yai_policy_attachment_csv_remove(char *csv, size_t csv_cap, const char *id)
{
    char src[sizeof(((yai_workspace_runtime_info_t *)0)->policy_attachments_csv)];
    char out[sizeof(((yai_workspace_runtime_info_t *)0)->policy_attachments_csv)];
    char *tok;
    char *save = NULL;
    int removed = 0;
    if (!csv || csv_cap == 0 || !id || !id[0]) return -1;
    if (!csv[0]) return 0;
    snprintf(src, sizeof(src), "%s", csv);
    out[0] = '\0';
    tok = strtok_r(src, ",", &save);
    while (tok)
    {
        if (strcmp(tok, id) == 0)
        {
            removed = 1;
        }
        else if (tok[0])
        {
            if (out[0]) strncat(out, ",", sizeof(out) - strlen(out) - 1);
            strncat(out, tok, sizeof(out) - strlen(out) - 1);
        }
        tok = strtok_r(NULL, ",", &save);
    }
    snprintf(csv, csv_cap, "%s", out);
    return removed ? 0 : 1;
}

static void yai_governable_meta_defaults(yai_governable_object_meta_t *meta)
{
    if (!meta) return;
    memset(meta, 0, sizeof(*meta));
    snprintf(meta->kind, sizeof(meta->kind), "%s", "unknown");
    snprintf(meta->status, sizeof(meta->status), "%s", "unknown");
    snprintf(meta->review_state, sizeof(meta->review_state), "%s", "unknown");
    meta->runtime_consumable = 1;
}

static void yai_json_array_to_csv(cJSON *arr, char *out, size_t out_cap)
{
    int i;
    int n;
    if (!out || out_cap == 0) return;
    out[0] = '\0';
    if (!cJSON_IsArray(arr)) return;
    for (i = 0; i < cJSON_GetArraySize(arr); i++)
    {
        cJSON *it = cJSON_GetArrayItem(arr, i);
        const char *s;
        if (!cJSON_IsString(it) || !it->valuestring || !it->valuestring[0]) continue;
        s = it->valuestring;
        if (out[0]) strncat(out, ",", out_cap - strlen(out) - 1u);
        strncat(out, s, out_cap - strlen(out) - 1u);
    }
    n = (int)strlen(out);
    if (n < 0) out[0] = '\0';
}

static int yai_governance_governable_object_lookup(const char *object_id, yai_governable_object_meta_t *meta)
{
    char path[MAX_PATH_LEN];
    char json[YAI_WS_JSON_IO_CAP];
    cJSON *doc = NULL;
    cJSON *objects = NULL;
    int i;
    int found = 0;
    yai_governable_meta_defaults(meta);
    if (!object_id || !object_id[0]) return 0;
    if (meta) snprintf(meta->id, sizeof(meta->id), "%s", object_id);
    if (yai_governance_root_path(path, sizeof(path), "model/registry/governable-objects.v1.json") != 0)
        return 0;
    if (yai_read_text(path, json, sizeof(json)) != 0)
        return 0;

    doc = cJSON_Parse(json);
    if (!doc) return 0;
    objects = cJSON_GetObjectItemCaseSensitive(doc, "objects");
    if (!cJSON_IsArray(objects))
    {
        cJSON_Delete(doc);
        return 0;
    }

    for (i = 0; i < cJSON_GetArraySize(objects); i++)
    {
        cJSON *obj = cJSON_GetArrayItem(objects, i);
        cJSON *id = cJSON_GetObjectItemCaseSensitive(obj, "id");
        cJSON *kind = cJSON_GetObjectItemCaseSensitive(obj, "kind");
        cJSON *status = cJSON_GetObjectItemCaseSensitive(obj, "status");
        cJSON *review = cJSON_GetObjectItemCaseSensitive(obj, "review_state");
        cJSON *runtime = cJSON_GetObjectItemCaseSensitive(obj, "runtime_consumable");
        cJSON *experimental = cJSON_GetObjectItemCaseSensitive(obj, "experimental");
        cJSON *deprecated = cJSON_GetObjectItemCaseSensitive(obj, "deprecated");
        cJSON *modes = cJSON_GetObjectItemCaseSensitive(obj, "attachment_modes");
        cJSON *precedence = cJSON_GetObjectItemCaseSensitive(obj, "precedence_class");
        cJSON *scope = cJSON_GetObjectItemCaseSensitive(obj, "scope_targets");
        cJSON *manifest_ref = cJSON_GetObjectItemCaseSensitive(obj, "manifest_ref");
        if (!cJSON_IsString(id) || !id->valuestring) continue;
        if (strcmp(id->valuestring, object_id) != 0) continue;
        found = 1;
        if (meta)
        {
            if (cJSON_IsString(kind) && kind->valuestring)
                snprintf(meta->kind, sizeof(meta->kind), "%s", kind->valuestring);
            if (cJSON_IsString(status) && status->valuestring)
                snprintf(meta->status, sizeof(meta->status), "%s", status->valuestring);
            if (cJSON_IsString(review) && review->valuestring)
                snprintf(meta->review_state, sizeof(meta->review_state), "%s", review->valuestring);
            if (cJSON_IsBool(runtime))
                meta->runtime_consumable = cJSON_IsTrue(runtime) ? 1 : 0;
            if (cJSON_IsBool(experimental))
                meta->experimental = cJSON_IsTrue(experimental) ? 1 : 0;
            if (cJSON_IsBool(deprecated))
                meta->deprecated = cJSON_IsTrue(deprecated) ? 1 : 0;
            yai_json_array_to_csv(modes, meta->attachment_modes_csv, sizeof(meta->attachment_modes_csv));
            if (cJSON_IsString(precedence) && precedence->valuestring)
                snprintf(meta->precedence_class, sizeof(meta->precedence_class), "%s", precedence->valuestring);
            if (cJSON_IsObject(scope))
            {
                yai_json_array_to_csv(cJSON_GetObjectItemCaseSensitive(scope, "workspace_ids"),
                                      meta->workspace_targets_csv,
                                      sizeof(meta->workspace_targets_csv));
                yai_json_array_to_csv(cJSON_GetObjectItemCaseSensitive(scope, "family_targets"),
                                      meta->family_targets_csv,
                                      sizeof(meta->family_targets_csv));
                yai_json_array_to_csv(cJSON_GetObjectItemCaseSensitive(scope, "specialization_targets"),
                                      meta->specialization_targets_csv,
                                      sizeof(meta->specialization_targets_csv));
            }
            if (cJSON_IsString(manifest_ref) && manifest_ref->valuestring)
                snprintf(meta->manifest_ref, sizeof(meta->manifest_ref), "%s", manifest_ref->valuestring);
            meta->found = 1;
        }
        break;
    }

    cJSON_Delete(doc);
    return found;
}

static int yai_csv_has_token(const char *csv, const char *token)
{
    return yai_policy_attachment_csv_contains(csv, token);
}

static int yai_target_match(const char *target_csv, const char *value)
{
    if (!target_csv || !target_csv[0]) return 1;
    if (!value || !value[0]) return 0;
    return yai_csv_has_token(target_csv, value);
}

static void yai_workspace_policy_effective_context(const yai_workspace_runtime_info_t *info,
                                                   char *family,
                                                   size_t family_cap,
                                                   char *specialization,
                                                   size_t specialization_cap)
{
    const char *f;
    const char *s;
    f = info->inferred_family[0] ? info->inferred_family : info->declared_control_family;
    s = info->inferred_specialization[0] ? info->inferred_specialization : info->declared_specialization;
    snprintf(family, family_cap, "%s", f && f[0] ? f : "unset");
    snprintf(specialization, specialization_cap, "%s", s && s[0] ? s : "unset");
}

static const char *yai_workspace_attachment_state_for_meta(const yai_governable_object_meta_t *meta, int is_attached)
{
    if (!meta) return is_attached ? "attached_active" : "attached_inactive";
    if (strcmp(meta->status, "candidate") == 0) return "candidate_only";
    if (strcmp(meta->status, "approved") == 0 && !is_attached) return "approved_not_attached";
    if (is_attached) return "attached_active";
    return "attached_inactive";
}

static int yai_workspace_policy_evaluate(const yai_workspace_runtime_info_t *info,
                                         const yai_governable_object_meta_t *meta,
                                         char *eligibility_result,
                                         size_t eligibility_cap,
                                         char *compatibility_result,
                                         size_t compatibility_cap,
                                         char *conflict_summary,
                                         size_t conflict_cap,
                                         char *warnings,
                                         size_t warnings_cap,
                                         int *blocking)
{
    char family[96];
    char specialization[96];
    int blocker = 0;
    int warn = 0;
    if (!info || !meta) return -1;
    if (eligibility_result && eligibility_cap) snprintf(eligibility_result, eligibility_cap, "%s", "eligible");
    if (compatibility_result && compatibility_cap) snprintf(compatibility_result, compatibility_cap, "%s", "dry_run_passed");
    if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "none");
    if (warnings && warnings_cap) warnings[0] = '\0';
    yai_workspace_policy_effective_context(info, family, sizeof(family), specialization, sizeof(specialization));

    if (!yai_target_match(meta->workspace_targets_csv, info->ws_id))
    {
        blocker = 1;
        if (eligibility_result && eligibility_cap) snprintf(eligibility_result, eligibility_cap, "%s", "ineligible");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "workspace_target_mismatch");
    }
    if (!blocker && !yai_target_match(meta->family_targets_csv, family))
    {
        blocker = 1;
        if (eligibility_result && eligibility_cap) snprintf(eligibility_result, eligibility_cap, "%s", "ineligible");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "family_target_mismatch");
    }
    if (!blocker && !yai_target_match(meta->specialization_targets_csv, specialization))
    {
        blocker = 1;
        if (eligibility_result && eligibility_cap) snprintf(eligibility_result, eligibility_cap, "%s", "ineligible");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "specialization_target_mismatch");
    }
    if (!blocker &&
        meta->attachment_modes_csv[0] &&
        !(yai_csv_has_token(meta->attachment_modes_csv, "workspace_attach") ||
          yai_csv_has_token(meta->attachment_modes_csv, "workspace-attach")))
    {
        blocker = 1;
        if (compatibility_result && compatibility_cap) snprintf(compatibility_result, compatibility_cap, "%s", "conflict_blocking");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "workspace_attach_not_allowed");
    }
    if (!blocker && (!meta->runtime_consumable || meta->deprecated))
    {
        blocker = 1;
        if (compatibility_result && compatibility_cap) snprintf(compatibility_result, compatibility_cap, "%s", "conflict_blocking");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "object_not_runtime_consumable");
    }
    if (!blocker &&
        strcmp(meta->kind, "enterprise_custom_governance_object") == 0 &&
        strcmp(meta->status, "approved") != 0 &&
        strcmp(meta->status, "applied") != 0 &&
        strcmp(meta->status, "active") != 0)
    {
        blocker = 1;
        if (compatibility_result && compatibility_cap) snprintf(compatibility_result, compatibility_cap, "%s", "conflict_blocking");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "enterprise_object_not_attachable_status");
    }
    if (!blocker &&
        strcmp(meta->kind, "enterprise_custom_governance_object") == 0 &&
        strcmp(meta->review_state, "approved") != 0)
    {
        blocker = 1;
        if (compatibility_result && compatibility_cap) snprintf(compatibility_result, compatibility_cap, "%s", "conflict_blocking");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "enterprise_object_not_review_approved");
    }
    if (meta->experimental)
    {
        warn = 1;
        if (warnings && warnings_cap) snprintf(warnings, warnings_cap, "%s", "experimental_object");
    }
    if (strcmp(meta->review_state, "in_review") == 0)
    {
        warn = 1;
        if (warnings && warnings_cap)
        {
            if (warnings[0]) strncat(warnings, ",", warnings_cap - strlen(warnings) - 1u);
            strncat(warnings, "review_in_progress", warnings_cap - strlen(warnings) - 1u);
        }
    }
    if (!blocker && warn && compatibility_result && compatibility_cap)
        snprintf(compatibility_result, compatibility_cap, "%s", "eligible_with_warnings");
    if (blocking) *blocking = blocker;
    return 0;
}

static const char *yai_get_home(void)
{
    const char *home = getenv("HOME");
    return (home && home[0]) ? home : NULL;
}

static int yai_workspace_is_scientific(const yai_workspace_runtime_info_t *info)
{
  const char *family;
  if (!info) return 0;
  family = info->inferred_family[0] ? info->inferred_family : info->declared_control_family;
  return (family[0] && strcmp(family, "scientific") == 0) ? 1 : 0;
}

static int yai_workspace_is_digital(const yai_workspace_runtime_info_t *info)
{
    const char *family;
    if (!info) return 0;
    family = info->inferred_family[0] ? info->inferred_family : info->declared_control_family;
    return (family[0] && strcmp(family, "digital") == 0) ? 1 : 0;
}

static void yai_workspace_build_scientific_summaries(const yai_workspace_runtime_info_t *info,
                                                     char *experiment_context,
                                                     size_t experiment_cap,
                                                     char *parameter_governance,
                                                     size_t parameter_cap,
                                                     char *reproducibility,
                                                     size_t reproducibility_cap,
                                                     char *dataset_integrity,
                                                     size_t dataset_cap,
                                                     char *publication_control,
                                                     size_t publication_cap)
{
    const char *spec;
    const char *effect;
    int scientific;
    if (!info) return;
    spec = info->inferred_specialization[0] ? info->inferred_specialization : info->declared_specialization;
    effect = info->last_effect_summary[0] ? info->last_effect_summary : "not resolved";
    scientific = yai_workspace_is_scientific(info);

    if (!scientific) {
        (void)snprintf(experiment_context, experiment_cap, "%s", "not scientific context");
        (void)snprintf(parameter_governance, parameter_cap, "%s", "not scientific context");
        (void)snprintf(reproducibility, reproducibility_cap, "%s", "not scientific context");
        (void)snprintf(dataset_integrity, dataset_cap, "%s", "not scientific context");
        (void)snprintf(publication_control, publication_cap, "%s", "not scientific context");
        return;
    }

    if (strcmp(spec, "experiment-configuration") == 0) {
        (void)snprintf(experiment_context, experiment_cap, "%s", "experiment configuration governance active");
    } else if (strcmp(spec, "parameter-governance") == 0) {
        (void)snprintf(experiment_context, experiment_cap, "%s", "parameter-focused experiment path active");
    } else {
        (void)snprintf(experiment_context, experiment_cap, "%s", "scientific workspace execution path active");
    }

    if (strstr(info->last_evidence_summary, "parameter_lock_required") ||
        strstr(info->last_evidence_summary, "parameter_diff_trace_required")) {
        (void)snprintf(parameter_governance, parameter_cap, "%s", "parameter lock and diff trace required");
    } else if (strcmp(spec, "parameter-governance") == 0) {
        (void)snprintf(parameter_governance, parameter_cap, "%s", "parameter governance enforced");
    } else {
        (void)snprintf(parameter_governance, parameter_cap, "%s", "parameter governance not primary in last resolution");
    }

    if (strstr(info->last_evidence_summary, "reproducibility_proofpack_required")) {
        (void)snprintf(reproducibility, reproducibility_cap, "%s", "reproducibility proofpack required");
    } else if (strcmp(spec, "reproducibility-control") == 0) {
        (void)snprintf(reproducibility, reproducibility_cap, "%s", "reproducibility control active");
    } else {
        (void)snprintf(reproducibility, reproducibility_cap, "%s", "reproducibility checks partial");
    }

    if (strstr(info->last_evidence_summary, "dataset_integrity_attestation_required") ||
        strcmp(spec, "dataset-integrity") == 0) {
        (void)snprintf(dataset_integrity, dataset_cap, "%s", "dataset integrity attestation required");
    } else {
        (void)snprintf(dataset_integrity, dataset_cap, "%s", "dataset integrity checks not primary in last resolution");
    }

    if (strcmp(spec, "result-publication-control") == 0) {
        (void)snprintf(publication_control,
                       publication_cap,
                       "%s",
                       strcmp(effect, "deny") == 0 ? "publication blocked pending policy/authority/repro checks" :
                       strcmp(effect, "quarantine") == 0 ? "publication quarantined pending review" :
                       strcmp(effect, "review_required") == 0 ? "publication requires final review" :
                       "publication path allowed with evidence obligations");
    } else {
        (void)snprintf(publication_control, publication_cap, "%s", "publication control not primary in last resolution");
    }
}

static void yai_workspace_build_digital_summaries(const yai_workspace_runtime_info_t *info,
                                                  char *outbound_context,
                                                  size_t outbound_cap,
                                                  char *sink_target,
                                                  size_t sink_cap,
                                                  char *publication_control,
                                                  size_t publication_cap,
                                                  char *retrieval_control,
                                                  size_t retrieval_cap,
                                                  char *distribution_control,
                                                  size_t distribution_cap)
{
    const char *spec;
    const char *effect;
    int digital;
    if (!info) return;
    spec = info->inferred_specialization[0] ? info->inferred_specialization : info->declared_specialization;
    effect = info->last_effect_summary[0] ? info->last_effect_summary : "not resolved";
    digital = yai_workspace_is_digital(info);

    if (!digital) {
        (void)snprintf(outbound_context, outbound_cap, "%s", "not digital context");
        (void)snprintf(sink_target, sink_cap, "%s", "not digital context");
        (void)snprintf(publication_control, publication_cap, "%s", "not digital context");
        (void)snprintf(retrieval_control, retrieval_cap, "%s", "not digital context");
        (void)snprintf(distribution_control, distribution_cap, "%s", "not digital context");
        return;
    }

    if (strcmp(spec, "remote-retrieval") == 0) {
        (void)snprintf(outbound_context, outbound_cap, "%s", "remote retrieval governed path active");
    } else if (strcmp(spec, "remote-publication") == 0) {
        (void)snprintf(outbound_context, outbound_cap, "%s", "remote publication governed path active");
    } else if (strcmp(spec, "external-commentary") == 0) {
        (void)snprintf(outbound_context, outbound_cap, "%s", "external commentary governed path active");
    } else if (strcmp(spec, "artifact-distribution") == 0) {
        (void)snprintf(outbound_context, outbound_cap, "%s", "artifact distribution governed path active");
    } else if (strcmp(spec, "digital-sink-control") == 0) {
        (void)snprintf(outbound_context, outbound_cap, "%s", "digital sink governance active");
    } else {
        (void)snprintf(outbound_context, outbound_cap, "%s", "digital outbound governance active");
    }

    if (strstr(info->last_evidence_summary, "sink_policy_attestation_required")) {
        (void)snprintf(sink_target, sink_cap, "%s", "sink policy attestation required");
    } else if (strcmp(spec, "digital-sink-control") == 0) {
        (void)snprintf(sink_target, sink_cap, "%s", "sink target control active");
    } else {
        (void)snprintf(sink_target, sink_cap, "%s", "sink target checks not primary in last resolution");
    }

    if (strcmp(spec, "remote-publication") == 0 || strstr(info->last_evidence_summary, "publication_review_record_required")) {
        (void)snprintf(publication_control,
                       publication_cap,
                       "%s",
                       strcmp(effect, "deny") == 0 ? "publication denied pending policy/authority/sink checks" :
                       strcmp(effect, "quarantine") == 0 ? "publication quarantined pending sink review" :
                       strcmp(effect, "review_required") == 0 ? "publication requires review record" :
                       "publication path allowed with destination trace");
    } else {
        (void)snprintf(publication_control, publication_cap, "%s", "publication control not primary in last resolution");
    }

    if (strcmp(spec, "remote-retrieval") == 0 || strstr(info->last_evidence_summary, "retrieval_source_attestation_required")) {
        (void)snprintf(retrieval_control, retrieval_cap, "%s", "retrieval source attestation required");
    } else {
        (void)snprintf(retrieval_control, retrieval_cap, "%s", "retrieval control not primary in last resolution");
    }

    if (strcmp(spec, "artifact-distribution") == 0 || strstr(info->last_evidence_summary, "distribution_manifest_required")) {
        (void)snprintf(distribution_control, distribution_cap, "%s", "distribution manifest and destination trace required");
    } else {
        (void)snprintf(distribution_control, distribution_cap, "%s", "artifact distribution not primary in last resolution");
    }
}

void yai_session_workspace_event_semantics(const yai_workspace_runtime_info_t *info,
                                           char *declared_scenario_spec,
                                           size_t declared_cap,
                                           char *business_spec,
                                           size_t business_cap,
                                           char *enforcement_spec,
                                           size_t enforcement_cap,
                                           char *flow_stage,
                                           size_t flow_cap,
                                           int *external_boundary)
{
    const char *declared = "";
    const char *inferred = "";
    const char *business = "";
    const char *enforcement = "";
    const char *stage = "unknown";
    int boundary = 0;

    if (!info) return;
    declared = info->declared_specialization[0] ? info->declared_specialization : "";
    inferred = info->inferred_specialization[0] ? info->inferred_specialization : "";
    enforcement = inferred[0] ? inferred : declared;

    if (declared[0] && inferred[0] &&
        strcmp(inferred, "network-egress") == 0 &&
        strcmp(declared, "network-egress") != 0)
        business = declared;
    else
        business = inferred[0] ? inferred : declared;

    if (strcmp(business, "remote-retrieval") == 0) stage = "retrieve";
    else if (strcmp(business, "experiment-configuration") == 0) stage = "ingress";
    else if (strcmp(business, "parameter-governance") == 0) stage = "transform";
    else if (strcmp(business, "black-box-evaluation") == 0) stage = "transform";
    else if (strcmp(business, "result-publication-control") == 0) stage = "publish";
    else if (strcmp(business, "remote-publication") == 0) stage = "publish";
    else if (strcmp(business, "external-commentary") == 0) stage = "publish";
    else if (strcmp(business, "artifact-distribution") == 0) stage = "distribute";
    else if (strcmp(business, "digital-sink-control") == 0) stage = "egress";
    else if (strcmp(enforcement, "network-egress") == 0) stage = "egress";
    else if (strcmp(info->last_effect_summary, "review_required") == 0) stage = "approve";
    else if (strcmp(info->last_effect_summary, "allow") == 0) stage = "actuate";

    if (strcmp(enforcement, "network-egress") == 0 ||
        strcmp(business, "remote-retrieval") == 0 ||
        strcmp(business, "remote-publication") == 0 ||
        strcmp(business, "external-commentary") == 0 ||
        strcmp(business, "artifact-distribution") == 0 ||
        strcmp(business, "digital-sink-control") == 0 ||
        strcmp(business, "result-publication-control") == 0)
        boundary = 1;

    if (declared_scenario_spec && declared_cap) snprintf(declared_scenario_spec, declared_cap, "%s", declared[0] ? declared : "unset");
    if (business_spec && business_cap) snprintf(business_spec, business_cap, "%s", business[0] ? business : "not_resolved");
    if (enforcement_spec && enforcement_cap) snprintf(enforcement_spec, enforcement_cap, "%s", enforcement[0] ? enforcement : "not_resolved");
    if (flow_stage && flow_cap) snprintf(flow_stage, flow_cap, "%s", stage);
    if (external_boundary) *external_boundary = boundary;
}

static const char *yai_workspace_review_state_from_effect(const char *effect)
{
    if (!effect || !effect[0]) return "unresolved";
    if (strcmp(effect, "review_required") == 0) return "pending_review";
    if (strcmp(effect, "quarantine") == 0) return "quarantined";
    if (strcmp(effect, "deny") == 0) return "blocked";
    if (strcmp(effect, "allow") == 0) return "clear";
    return "unresolved";
}

static void yai_workspace_operational_summary(const char *flow_stage,
                                              const char *business_spec,
                                              const char *effect,
                                              char *out,
                                              size_t out_cap)
{
    if (!out || out_cap == 0) return;
    (void)snprintf(out,
                   out_cap,
                   "%s/%s => %s",
                   flow_stage && flow_stage[0] ? flow_stage : "unknown",
                   business_spec && business_spec[0] ? business_spec : "not_resolved",
                   effect && effect[0] ? effect : "not_resolved");
}

static void trim_trailing_slashes(char *path);

static int yai_workspace_store_root_path(char *out, size_t out_cap)
{
    const char *home = yai_get_home();
    const char *env_root = getenv("YAI_WORKSPACE_ROOT");
    if (!out || out_cap == 0 || !home)
        return -1;
    if (env_root && env_root[0])
    {
        if (snprintf(out, out_cap, "%s", env_root) <= 0)
            return -1;
        trim_trailing_slashes(out);
        return 0;
    }
    if (snprintf(out, out_cap, "%s/.yai/workspaces", home) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_runtime_state_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_metadata_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s/metadata", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_state_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s/state", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_traces_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s/traces", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_artifacts_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s/artifacts", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_runtime_local_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s/runtime", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_events_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s/events", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_event_sink_paths(const char *ws_id,
                                          char *events_log,
                                          size_t events_log_cap,
                                          char *decision_log,
                                          size_t decision_log_cap,
                                          char *evidence_log,
                                          size_t evidence_log_cap,
                                          char *event_index,
                                          size_t event_index_cap)
{
    char events_root[MAX_PATH_LEN];
    if (!ws_id || !events_log || !decision_log || !evidence_log || !event_index)
        return -1;
    if (yai_workspace_events_root_path(ws_id, events_root, sizeof(events_root)) != 0)
        return -1;
    if (snprintf(events_log, events_log_cap, "%s/runtime-events.v1.ndjson", events_root) <= 0)
        return -1;
    if (snprintf(decision_log, decision_log_cap, "%s/decision-records.v1.ndjson", events_root) <= 0)
        return -1;
    if (snprintf(evidence_log, evidence_log_cap, "%s/evidence-records.v1.ndjson", events_root) <= 0)
        return -1;
    if (snprintf(event_index, event_index_cap, "%s/index.v1.json", events_root) <= 0)
        return -1;
    return 0;
}

static int yai_append_json_line(const char *path, const char *json_line)
{
    FILE *f;
    if (!path || !json_line) return -1;
    f = fopen(path, "a");
    if (!f) return -1;
    if (fprintf(f, "%s\n", json_line) < 0)
    {
        fclose(f);
        return -1;
    }
    fclose(f);
    return 0;
}

static void yai_format_iso8601_utc(time_t now, char *out, size_t out_cap)
{
    struct tm utc_tm;
    if (!out || out_cap == 0) return;
    out[0] = '\0';
    if (gmtime_r(&now, &utc_tm) == NULL)
    {
        (void)snprintf(out, out_cap, "%ld", (long)now);
        return;
    }
    (void)strftime(out, out_cap, "%Y-%m-%dT%H:%M:%SZ", &utc_tm);
}

static int yai_workspace_append_event_evidence_records(const char *ws_id,
                                                       const yai_governance_resolution_output_t *law_out,
                                                       char *event_ref_out,
                                                       size_t event_ref_cap,
                                                       char *decision_ref_out,
                                                       size_t decision_ref_cap,
                                                       char *evidence_ref_out,
                                                       size_t evidence_ref_cap,
                                                       char *err,
                                                       size_t err_cap)
{
    char events_log[MAX_PATH_LEN];
    char decision_log[MAX_PATH_LEN];
    char evidence_log[MAX_PATH_LEN];
    char event_index[MAX_PATH_LEN];
    char event_ref[96];
    char decision_ref[96];
    char evidence_ref[96];
    char ts_iso[48];
    char event_lifecycle[384];
    char event_record[3072];
    char decision_blob[2048];
    char evidence_blob[2048];
    char index_blob[1536];
    time_t now;

    if (err && err_cap > 0) err[0] = '\0';
    if (!ws_id || !law_out)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "event_sink_bad_args");
        return -1;
    }
    if (yai_workspace_event_sink_paths(ws_id,
                                       events_log,
                                       sizeof(events_log),
                                       decision_log,
                                       sizeof(decision_log),
                                       evidence_log,
                                       sizeof(evidence_log),
                                       event_index,
                                       sizeof(event_index)) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "event_sink_path_failed");
        return -1;
    }

    (void)snprintf(event_ref,
                   sizeof(event_ref),
                   "evt-%s",
                   law_out->evidence.trace_id[0] ? law_out->evidence.trace_id : law_out->decision.decision_id);
    (void)snprintf(decision_ref, sizeof(decision_ref), "%s", law_out->decision.decision_id);
    (void)snprintf(evidence_ref,
                   sizeof(evidence_ref),
                   "evd-%s",
                   law_out->evidence.trace_id[0] ? law_out->evidence.trace_id : law_out->decision.decision_id);

    now = time(NULL);
    yai_format_iso8601_utc(now, ts_iso, sizeof(ts_iso));
    if (yai_lifecycle_build_json_fragment("runtime_event", "workspace_id,time_window", event_lifecycle, sizeof(event_lifecycle)) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "event_lifecycle_encode_failed");
        return -1;
    }

    if (snprintf(event_record,
                 sizeof(event_record),
                 "{\"type\":\"yai.runtime_event.v1\",\"schema_version\":\"v1\","
                 "\"event_id\":\"%s\",\"event_class\":\"execution_governance\","
                 "\"event_type\":\"governed_resolution_emitted\","
                 "\"timestamp\":\"%s\",\"timestamp_epoch\":%ld,"
                 "\"workspace_id\":\"%s\",\"source_component\":\"core.session\","
                 "\"actor_class\":\"runtime\",\"status\":\"ok\","
                 "\"target_ref\":\"%s\",\"trace_ref\":\"%s\","
                 "\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\","
                 "%s,"
                 "\"payload\":{\"family_id\":\"%s\",\"specialization_id\":\"%s\",\"effect\":\"%s\"}}",
                 event_ref,
                 ts_iso,
                 (long)now,
                 ws_id,
                 decision_ref,
                 law_out->evidence.trace_id,
                 decision_ref,
                 evidence_ref,
                 event_lifecycle,
                 law_out->decision.family_id,
                 law_out->decision.specialization_id,
                 yai_governance_effect_name(law_out->decision.final_effect)) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "event_sink_encode_failed");
        return -1;
    }

    if (yai_governance_decision_to_audit_blob(&law_out->decision, decision_blob, sizeof(decision_blob)) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "decision_record_encode_failed");
        return -1;
    }
    if (yai_governance_evidence_to_record_blob(&law_out->decision, &law_out->evidence, evidence_blob, sizeof(evidence_blob)) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "evidence_record_encode_failed");
        return -1;
    }

    if (yai_append_json_line(events_log, event_record) != 0 ||
        yai_append_json_line(decision_log, decision_blob) != 0 ||
        yai_append_json_line(evidence_log, evidence_blob) != 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "event_sink_append_failed");
        return -1;
    }

    if (snprintf(index_blob,
                 sizeof(index_blob),
                 "{"
                 "\"type\":\"yai.event_evidence.index.v1\","
                 "\"workspace_id\":\"%s\","
                 "\"last_event_ref\":\"%s\","
                 "\"last_decision_ref\":\"%s\","
                 "\"last_evidence_ref\":\"%s\","
                 "\"last_trace_ref\":\"%s\","
                 "\"retention_profile_ref\":\"matrix:data-lifecycle-retention-v0.1.0\","
                 "\"updated_at\":\"%s\","
                 "\"stores\":{"
                   "\"events\":\"%s\","
                   "\"decisions\":\"%s\","
                   "\"evidence\":\"%s\""
                 "}"
                 "}",
                 ws_id,
                 event_ref,
                 decision_ref,
                 evidence_ref,
                 law_out->evidence.trace_id,
                 ts_iso,
                 events_log,
                 decision_log,
                 evidence_log) <= 0)
    {
        if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "event_sink_index_encode_failed");
        return -1;
    }
    {
        FILE *f = fopen(event_index, "w");
        if (!f)
        {
            if (err && err_cap > 0) (void)snprintf(err, err_cap, "%s", "event_sink_index_write_failed");
            return -1;
        }
        (void)fprintf(f, "%s\n", index_blob);
        fclose(f);
    }

    if (event_ref_out && event_ref_cap > 0)
        (void)snprintf(event_ref_out, event_ref_cap, "%s", event_ref);
    if (decision_ref_out && decision_ref_cap > 0)
        (void)snprintf(decision_ref_out, decision_ref_cap, "%s", decision_ref);
    if (evidence_ref_out && evidence_ref_cap > 0)
        (void)snprintf(evidence_ref_out, evidence_ref_cap, "%s", evidence_ref);
    return 0;
}

static void yai_workspace_read_event_evidence_index(const yai_workspace_runtime_info_t *info,
                                                    char *event_ref,
                                                    size_t event_ref_cap,
                                                    char *decision_ref,
                                                    size_t decision_ref_cap,
                                                    char *evidence_ref,
                                                    size_t evidence_ref_cap,
                                                    char *event_store_ref,
                                                    size_t event_store_ref_cap,
                                                    char *decision_store_ref,
                                                    size_t decision_store_ref_cap,
                                                    char *evidence_store_ref,
                                                    size_t evidence_store_ref_cap)
{
    char event_index[MAX_PATH_LEN];
    char buf[2048];
    char events_log[MAX_PATH_LEN];
    char decision_log[MAX_PATH_LEN];
    char evidence_log[MAX_PATH_LEN];

    if (event_ref && event_ref_cap > 0) event_ref[0] = '\0';
    if (decision_ref && decision_ref_cap > 0) decision_ref[0] = '\0';
    if (evidence_ref && evidence_ref_cap > 0) evidence_ref[0] = '\0';
    if (event_store_ref && event_store_ref_cap > 0) event_store_ref[0] = '\0';
    if (decision_store_ref && decision_store_ref_cap > 0) decision_store_ref[0] = '\0';
    if (evidence_store_ref && evidence_store_ref_cap > 0) evidence_store_ref[0] = '\0';
    if (!info || !info->ws_id[0]) return;
    if (yai_workspace_event_sink_paths(info->ws_id,
                                       events_log,
                                       sizeof(events_log),
                                       decision_log,
                                       sizeof(decision_log),
                                       evidence_log,
                                       sizeof(evidence_log),
                                       event_index,
                                       sizeof(event_index)) != 0)
        return;
    if (event_store_ref && event_store_ref_cap > 0)
        (void)snprintf(event_store_ref, event_store_ref_cap, "%s", events_log);
    if (decision_store_ref && decision_store_ref_cap > 0)
        (void)snprintf(decision_store_ref, decision_store_ref_cap, "%s", decision_log);
    if (evidence_store_ref && evidence_store_ref_cap > 0)
        (void)snprintf(evidence_store_ref, evidence_store_ref_cap, "%s", evidence_log);

    if (yai_read_text(event_index, buf, sizeof(buf)) != 0) return;
    if (event_ref && event_ref_cap > 0)
        (void)yai_session_extract_json_string(buf, "last_event_ref", event_ref, event_ref_cap);
    if (decision_ref && decision_ref_cap > 0)
        (void)yai_session_extract_json_string(buf, "last_decision_ref", decision_ref, decision_ref_cap);
    if (evidence_ref && evidence_ref_cap > 0)
        (void)yai_session_extract_json_string(buf, "last_evidence_ref", evidence_ref, evidence_ref_cap);
}
