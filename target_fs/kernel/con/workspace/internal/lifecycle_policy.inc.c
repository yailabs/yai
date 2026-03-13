typedef struct {
    const char *data_class;
    const char *retention_profile;
    const char *default_tier;
    int lineage_required;
    int compactable;
    int archive_eligible;
} yai_lifecycle_policy_t;

static const char *yai_lifecycle_policy_matrix_ref(void)
{
    return "matrix:data-lifecycle-retention-v0.1.0";
}

static const yai_lifecycle_policy_t *yai_lifecycle_policy_for_class(const char *data_class)
{
    static const yai_lifecycle_policy_t POLICIES[] = {
        {"runtime_event", "event.default.v1", "hot", 1, 1, 1},
        {"decision_record", "decision.default.v1", "hot", 1, 1, 1},
        {"evidence_record", "evidence.default.v1", "hot", 1, 1, 1},
        {"governance_object_state", "governance.default.v1", "hot", 1, 1, 1},
        {"governance_lifecycle_state", "governance.default.v1", "hot", 1, 1, 1},
        {"governance_attachment_state", "governance.default.v1", "hot", 1, 1, 1},
        {"authority_state", "authority.default.v1", "hot", 1, 1, 1},
        {"authority_resolution_record", "authority.default.v1", "hot", 1, 1, 1},
        {"artifact_metadata", "artifact.default.v1", "hot", 1, 1, 1},
        {"artifact_governance_linkage", "artifact.default.v1", "hot", 1, 1, 1},
        {"enforcement_outcome_record", "enforcement.default.v1", "hot", 1, 1, 1},
        {"enforcement_linkage_record", "enforcement.default.v1", "hot", 1, 1, 1},
    };
    size_t i;
    static const yai_lifecycle_policy_t DEFAULT_POLICY = {
        "unknown",
        "generic.default.v1",
        "hot",
        1,
        1,
        1};

    if (!data_class || !data_class[0])
        return &DEFAULT_POLICY;
    for (i = 0; i < (sizeof(POLICIES) / sizeof(POLICIES[0])); ++i)
        if (strcmp(POLICIES[i].data_class, data_class) == 0)
            return &POLICIES[i];
    return &DEFAULT_POLICY;
}

static int yai_lifecycle_render_policy_rows_json(char *out, size_t out_cap)
{
    static const yai_lifecycle_policy_t POLICIES[] = {
        {"runtime_event", "event.default.v1", "hot", 1, 1, 1},
        {"decision_record", "decision.default.v1", "hot", 1, 1, 1},
        {"evidence_record", "evidence.default.v1", "hot", 1, 1, 1},
        {"governance_object_state", "governance.default.v1", "hot", 1, 1, 1},
        {"governance_lifecycle_state", "governance.default.v1", "hot", 1, 1, 1},
        {"governance_attachment_state", "governance.default.v1", "hot", 1, 1, 1},
        {"authority_state", "authority.default.v1", "hot", 1, 1, 1},
        {"authority_resolution_record", "authority.default.v1", "hot", 1, 1, 1},
        {"artifact_metadata", "artifact.default.v1", "hot", 1, 1, 1},
        {"artifact_governance_linkage", "artifact.default.v1", "hot", 1, 1, 1},
        {"enforcement_outcome_record", "enforcement.default.v1", "hot", 1, 1, 1},
        {"enforcement_linkage_record", "enforcement.default.v1", "hot", 1, 1, 1},
    };
    size_t i;
    size_t used = 0;

    if (!out || out_cap == 0)
        return -1;
    out[0] = '\0';

    for (i = 0; i < (sizeof(POLICIES) / sizeof(POLICIES[0])); ++i)
    {
        int n = snprintf(out + used,
                         out_cap - used,
                         "%s{\"data_class\":\"%s\",\"tier\":\"%s\",\"retention_profile\":\"%s\","
                         "\"lineage_required\":%s,\"compactable\":%s,\"archive_eligible\":%s}",
                         (i == 0) ? "" : ",",
                         POLICIES[i].data_class,
                         POLICIES[i].default_tier,
                         POLICIES[i].retention_profile,
                         POLICIES[i].lineage_required ? "true" : "false",
                         POLICIES[i].compactable ? "true" : "false",
                         POLICIES[i].archive_eligible ? "true" : "false");
        if (n <= 0 || (size_t)n >= (out_cap - used))
            return -1;
        used += (size_t)n;
    }
    return 0;
}

static int yai_lifecycle_build_json_fragment(const char *data_class,
                                             const char *workspace_scope,
                                             char *out,
                                             size_t out_cap)
{
    const yai_lifecycle_policy_t *policy = yai_lifecycle_policy_for_class(data_class);
    if (!out || out_cap == 0)
        return -1;
    out[0] = '\0';
    if (snprintf(out,
                 out_cap,
                 "\"lifecycle\":{\"data_class\":\"%s\",\"tier\":\"%s\","
                 "\"retention_profile\":\"%s\",\"partition_scope\":\"%s\","
                 "\"lineage_required\":%s,\"compactable\":%s,\"archive_eligible\":%s}",
                 policy->data_class,
                 policy->default_tier,
                 policy->retention_profile,
                 (workspace_scope && workspace_scope[0]) ? workspace_scope : "workspace_id,time_window",
                 policy->lineage_required ? "true" : "false",
                 policy->compactable ? "true" : "false",
                 policy->archive_eligible ? "true" : "false") <= 0)
        return -1;
    return 0;
}
