typedef struct {
    int files_touched;
    int files_compacted;
    long records_total;
    long records_archived;
    long records_hot;
    long violations;
} yai_lifecycle_exec_stats_t;

static int yai_lifecycle_env_int(const char *name, int fallback, int min_v, int max_v)
{
    const char *v = getenv(name);
    long parsed;
    char *end = NULL;
    if (!v || !v[0])
        return fallback;
    parsed = strtol(v, &end, 10);
    if (!end || *end != '\0')
        return fallback;
    if (parsed < min_v) return min_v;
    if (parsed > max_v) return max_v;
    return (int)parsed;
}

static int yai_lifecycle_mkdirs(const char *path)
{
    char tmp[MAX_PATH_LEN];
    size_t i;
    if (!path || !path[0]) return -1;
    if (strlen(path) >= sizeof(tmp)) return -1;
    (void)snprintf(tmp, sizeof(tmp), "%s", path);
    for (i = 1; tmp[i]; ++i)
    {
        if (tmp[i] == '/')
        {
            tmp[i] = '\0';
            if (tmp[0] && mkdir(tmp, 0755) != 0 && errno != EEXIST)
                return -1;
            tmp[i] = '/';
        }
    }
    if (mkdir(tmp, 0755) != 0 && errno != EEXIST)
        return -1;
    return 0;
}

static int yai_lifecycle_workspace_field_matches(const char *line, const char *ws_id)
{
    cJSON *obj;
    cJSON *it;
    const char *keys[] = {"workspace_id", "workspace_ref", "workspace"};
    size_t i;
    if (!line || !ws_id || !ws_id[0])
        return 0;
    obj = cJSON_Parse(line);
    if (!obj)
        return 1;
    for (i = 0; i < (sizeof(keys) / sizeof(keys[0])); ++i)
    {
        it = cJSON_GetObjectItemCaseSensitive(obj, keys[i]);
        if (cJSON_IsString(it) && it->valuestring && it->valuestring[0])
        {
            int ok = strcmp(it->valuestring, ws_id) == 0;
            cJSON_Delete(obj);
            return ok;
        }
    }
    cJSON_Delete(obj);
    return 1;
}

static int yai_lifecycle_safe_prefix(const char *prefix, const char *path)
{
    size_t n;
    if (!prefix || !path) return 0;
    n = strlen(prefix);
    return n > 0 && strncmp(prefix, path, n) == 0;
}

static int yai_lifecycle_build_archive_path(const char *run_dir,
                                            const char *src_path,
                                            char *out,
                                            size_t out_cap)
{
    const char *suffix;
    if (!run_dir || !src_path || !out || out_cap == 0) return -1;
    out[0] = '\0';
    if (!yai_lifecycle_safe_prefix(run_dir, src_path))
        return -1;
    suffix = src_path + strlen(run_dir);
    if (suffix[0] == '/')
        ++suffix;
    if (snprintf(out, out_cap, "%s/archive/%s", run_dir, suffix) <= 0)
        return -1;
    return 0;
}

static int yai_lifecycle_append_line(const char *path, const char *line)
{
    FILE *f;
    char dir[MAX_PATH_LEN];
    char *slash;
    if (!path || !line) return -1;
    if (strlen(path) >= sizeof(dir)) return -1;
    (void)snprintf(dir, sizeof(dir), "%s", path);
    slash = strrchr(dir, '/');
    if (!slash) return -1;
    *slash = '\0';
    if (yai_lifecycle_mkdirs(dir) != 0)
        return -1;
    f = fopen(path, "a");
    if (!f) return -1;
    if (fputs(line, f) == EOF)
    {
        fclose(f);
        return -1;
    }
    fclose(f);
    return 0;
}

static int yai_lifecycle_compact_ndjson(const char *ws_id,
                                        const char *run_dir,
                                        const char *src_path,
                                        int keep_last,
                                        long *total_out,
                                        long *archived_out,
                                        long *hot_out,
                                        long *violations_out)
{
    FILE *f = NULL;
    FILE *f2 = NULL;
    FILE *keepf = NULL;
    char *line = NULL;
    size_t cap = 0;
    ssize_t n;
    long total = 0;
    long archive_cut = 0;
    long idx = 0;
    char archive_path[MAX_PATH_LEN];
    char keep_tmp[MAX_PATH_LEN];
    char keep_dir[MAX_PATH_LEN];
    int rc = -1;
    int had_violation = 0;

    if (total_out) *total_out = 0;
    if (archived_out) *archived_out = 0;
    if (hot_out) *hot_out = 0;
    if (violations_out) *violations_out = 0;
    if (!ws_id || !run_dir || !src_path || keep_last < 0)
        return -1;
    if (!yai_lifecycle_safe_prefix(run_dir, src_path))
        return -1;

    f = fopen(src_path, "r");
    if (!f)
        return 0;
    while ((n = getline(&line, &cap, f)) != -1)
    {
        if (n <= 1) continue;
        if (!yai_lifecycle_workspace_field_matches(line, ws_id))
            had_violation = 1;
        total++;
    }
    fclose(f);
    f = NULL;

    if (had_violation)
    {
        if (violations_out) *violations_out = 1;
        goto cleanup;
    }

    if (total <= keep_last)
    {
        if (total_out) *total_out = total;
        if (hot_out) *hot_out = total;
        rc = 0;
        goto cleanup;
    }

    archive_cut = total - keep_last;
    if (yai_lifecycle_build_archive_path(run_dir, src_path, archive_path, sizeof(archive_path)) != 0)
        goto cleanup;
    if (snprintf(keep_tmp, sizeof(keep_tmp), "%s.keep.tmp", src_path) <= 0)
        goto cleanup;
    (void)snprintf(keep_dir, sizeof(keep_dir), "%s", keep_tmp);
    {
        char *slash = strrchr(keep_dir, '/');
        if (!slash) goto cleanup;
        *slash = '\0';
        if (yai_lifecycle_mkdirs(keep_dir) != 0) goto cleanup;
    }
    keepf = fopen(keep_tmp, "w");
    if (!keepf) goto cleanup;
    f2 = fopen(src_path, "r");
    if (!f2) goto cleanup;

    idx = 0;
    while ((n = getline(&line, &cap, f2)) != -1)
    {
        if (n <= 1) continue;
        if (idx < archive_cut)
        {
            if (yai_lifecycle_append_line(archive_path, line) != 0)
                goto cleanup;
        }
        else
        {
            if (fputs(line, keepf) == EOF)
                goto cleanup;
        }
        idx++;
    }
    fclose(f2); f2 = NULL;
    fclose(keepf); keepf = NULL;
    if (rename(keep_tmp, src_path) != 0)
        goto cleanup;

    if (total_out) *total_out = total;
    if (archived_out) *archived_out = archive_cut;
    if (hot_out) *hot_out = total - archive_cut;
    rc = 0;

cleanup:
    if (f) fclose(f);
    if (f2) fclose(f2);
    if (keepf) fclose(keepf);
    if (line) free(line);
    if (rc != 0)
        (void)unlink(keep_tmp);
    return rc;
}

static int yai_lifecycle_write_rollup_snapshot(const yai_workspace_runtime_info_t *info,
                                               const yai_lifecycle_exec_stats_t *stats,
                                               int keep_events,
                                               int keep_decisions,
                                               int keep_evidence,
                                               int keep_graph_nodes,
                                               int keep_graph_edges,
                                               int keep_transient,
                                               char *err,
                                               size_t err_cap)
{
    char run_dir[MAX_PATH_LEN];
    char lifecycle_dir[MAX_PATH_LEN];
    char rollup_path[MAX_PATH_LEN];
    char snapshot_path[MAX_PATH_LEN];
    char ts_iso[48];
    FILE *f = NULL;
    time_t now;
    char sink_last_event_ref[192];
    char sink_last_decision_ref[192];
    char sink_last_evidence_ref[192];
    char sink_event_store_ref[MAX_PATH_LEN];
    char sink_decision_store_ref[MAX_PATH_LEN];
    char sink_evidence_store_ref[MAX_PATH_LEN];
    char brain_graph_node_ref[192];
    char brain_graph_edge_ref[192];
    char brain_transient_state_ref[192];
    char brain_transient_working_set_ref[192];
    char brain_graph_store_ref[MAX_PATH_LEN];
    char brain_transient_store_ref[MAX_PATH_LEN];

    if (err && err_cap > 0) err[0] = '\0';
    if (!info || !info->ws_id[0] || !stats)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), info->ws_id) != 0)
        return -1;
    if (snprintf(lifecycle_dir, sizeof(lifecycle_dir), "%s/lifecycle", run_dir) <= 0)
        return -1;
    if (snprintf(rollup_path, sizeof(rollup_path), "%s/rollup.v1.json", lifecycle_dir) <= 0)
        return -1;
    if (snprintf(snapshot_path, sizeof(snapshot_path), "%s/snapshot.v1.json", lifecycle_dir) <= 0)
        return -1;
    if (yai_lifecycle_mkdirs(lifecycle_dir) != 0)
        return -1;

    now = time(NULL);
    yai_format_iso8601_utc(now, ts_iso, sizeof(ts_iso));

    yai_workspace_read_event_evidence_index(info,
                                            sink_last_event_ref, sizeof(sink_last_event_ref),
                                            sink_last_decision_ref, sizeof(sink_last_decision_ref),
                                            sink_last_evidence_ref, sizeof(sink_last_evidence_ref),
                                            sink_event_store_ref, sizeof(sink_event_store_ref),
                                            sink_decision_store_ref, sizeof(sink_decision_store_ref),
                                            sink_evidence_store_ref, sizeof(sink_evidence_store_ref));
    (void)yai_mind_storage_bridge_last_refs(info->ws_id,
                                            brain_graph_node_ref, sizeof(brain_graph_node_ref),
                                            brain_graph_edge_ref, sizeof(brain_graph_edge_ref),
                                            brain_transient_state_ref, sizeof(brain_transient_state_ref),
                                            brain_transient_working_set_ref, sizeof(brain_transient_working_set_ref),
                                            brain_graph_store_ref, sizeof(brain_graph_store_ref),
                                            brain_transient_store_ref, sizeof(brain_transient_store_ref));

    f = fopen(rollup_path, "w");
    if (!f)
        return -1;
    (void)fprintf(f,
                  "{"
                  "\"type\":\"yai.lifecycle.rollup.v1\","
                  "\"workspace_id\":\"%s\","
                  "\"updated_at\":\"%s\","
                  "\"execution\":{\"files_touched\":%d,\"files_compacted\":%d,\"records_total\":%ld,\"records_archived\":%ld,\"records_hot\":%ld,\"violations\":%ld},"
                  "\"hot_bounds\":{\"events\":%d,\"decisions\":%d,\"evidence\":%d,\"graph_nodes\":%d,\"graph_edges\":%d,\"transient\":%d},"
                  "\"matrix_ref\":\"%s\""
                  "}\n",
                  info->ws_id,
                  ts_iso,
                  stats->files_touched,
                  stats->files_compacted,
                  stats->records_total,
                  stats->records_archived,
                  stats->records_hot,
                  stats->violations,
                  keep_events,
                  keep_decisions,
                  keep_evidence,
                  keep_graph_nodes,
                  keep_graph_edges,
                  keep_transient,
                  yai_lifecycle_policy_matrix_ref());
    fclose(f);
    f = NULL;

    f = fopen(snapshot_path, "w");
    if (!f)
        return -1;
    (void)fprintf(f,
                  "{"
                  "\"type\":\"yai.lifecycle.snapshot.v1\","
                  "\"workspace_id\":\"%s\","
                  "\"updated_at\":\"%s\","
                  "\"refs\":{\"event_ref\":\"%s\",\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\","
                  "\"graph_node_ref\":\"%s\",\"graph_edge_ref\":\"%s\",\"transient_state_ref\":\"%s\",\"transient_working_set_ref\":\"%s\"},"
                  "\"stores\":{\"event_store_ref\":\"%s\",\"decision_store_ref\":\"%s\",\"evidence_store_ref\":\"%s\","
                  "\"graph_store_ref\":\"%s\",\"transient_store_ref\":\"%s\"}"
                  "}\n",
                  info->ws_id,
                  ts_iso,
                  sink_last_event_ref,
                  sink_last_decision_ref,
                  sink_last_evidence_ref,
                  brain_graph_node_ref,
                  brain_graph_edge_ref,
                  brain_transient_state_ref,
                  brain_transient_working_set_ref,
                  sink_event_store_ref,
                  sink_decision_store_ref,
                  sink_evidence_store_ref,
                  brain_graph_store_ref,
                  brain_transient_store_ref);
    fclose(f);
    return 0;
}

static int yai_lifecycle_brain_paths(const char *ws_id,
                                     char *graph_nodes,
                                     size_t graph_nodes_cap,
                                     char *graph_edges,
                                     size_t graph_edges_cap,
                                     char *transient_activation,
                                     size_t transient_activation_cap,
                                     char *transient_working_set,
                                     size_t transient_working_set_cap)
{
    const char *home = yai_get_home();
    if (!home || !ws_id || !graph_nodes || !graph_edges || !transient_activation || !transient_working_set)
        return -1;
    if (snprintf(graph_nodes, graph_nodes_cap, "%s/.yai/run/%s/runtime/graph/persistent-nodes.v1.ndjson", home, ws_id) <= 0)
        return -1;
    if (snprintf(graph_edges, graph_edges_cap, "%s/.yai/run/%s/runtime/graph/persistent-edges.v1.ndjson", home, ws_id) <= 0)
        return -1;
    if (snprintf(transient_activation, transient_activation_cap, "%s/.yai/run/%s/runtime/transient/activation-state.v1.ndjson", home, ws_id) <= 0)
        return -1;
    if (snprintf(transient_working_set, transient_working_set_cap, "%s/.yai/run/%s/runtime/transient/working-set.v1.ndjson", home, ws_id) <= 0)
        return -1;
    return 0;
}

static void yai_lifecycle_apply_file(const char *ws_id,
                                     const char *run_dir,
                                     const char *path,
                                     int keep_last,
                                     yai_lifecycle_exec_stats_t *stats)
{
    long total = 0, archived = 0, hot = 0, violations = 0;
    int rc;
    if (!path || !path[0] || !stats)
        return;
    rc = yai_lifecycle_compact_ndjson(ws_id, run_dir, path, keep_last, &total, &archived, &hot, &violations);
    if (rc != 0 && total == 0 && archived == 0 && hot == 0 && violations == 0)
        return;
    stats->files_touched += 1;
    if (archived > 0)
        stats->files_compacted += 1;
    stats->records_total += total;
    stats->records_archived += archived;
    stats->records_hot += hot;
    stats->violations += violations;
}

int yai_session_run_workspace_lifecycle_maintenance_json(char *out,
                                                         size_t out_cap,
                                                         char *err,
                                                         size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    char status[32];
    char bind_err[96];
    char run_dir[MAX_PATH_LEN];
    char events_log[MAX_PATH_LEN], decision_log[MAX_PATH_LEN], evidence_log[MAX_PATH_LEN], event_index[MAX_PATH_LEN];
    char gov_obj[MAX_PATH_LEN], gov_lifecycle[MAX_PATH_LEN], gov_attach[MAX_PATH_LEN], gov_index[MAX_PATH_LEN];
    char authority_log[MAX_PATH_LEN], authority_res[MAX_PATH_LEN], authority_index[MAX_PATH_LEN];
    char artifact_meta[MAX_PATH_LEN], artifact_link[MAX_PATH_LEN], artifact_index[MAX_PATH_LEN];
    char enf_out[MAX_PATH_LEN], enf_link[MAX_PATH_LEN], enf_index[MAX_PATH_LEN];
    char brain_nodes[MAX_PATH_LEN], brain_edges[MAX_PATH_LEN], transient_act[MAX_PATH_LEN], transient_ws[MAX_PATH_LEN];
    char lifecycle_dir[MAX_PATH_LEN], maintenance_index[MAX_PATH_LEN];
    yai_lifecycle_exec_stats_t stats;
    int keep_events, keep_decisions, keep_evidence, keep_graph_nodes, keep_graph_edges, keep_transient;
    char ts_iso[48];
    time_t now;
    FILE *f;
    int n;

    if (err && err_cap > 0) err[0] = '\0';
    if (!out || out_cap == 0)
        return -1;

    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), bind_err, sizeof(bind_err)) != 0 ||
        strcmp(status, "active") != 0)
    {
        if (err && err_cap > 0)
            (void)snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_not_active");
        return -1;
    }
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), info.ws_id) != 0)
        return -1;

    keep_events = yai_lifecycle_env_int("YAI_LIFECYCLE_KEEP_EVENTS", 200, 10, 20000);
    keep_decisions = yai_lifecycle_env_int("YAI_LIFECYCLE_KEEP_DECISIONS", 200, 10, 20000);
    keep_evidence = yai_lifecycle_env_int("YAI_LIFECYCLE_KEEP_EVIDENCE", 200, 10, 20000);
    keep_graph_nodes = yai_lifecycle_env_int("YAI_LIFECYCLE_KEEP_GRAPH_NODES", 300, 20, 50000);
    keep_graph_edges = yai_lifecycle_env_int("YAI_LIFECYCLE_KEEP_GRAPH_EDGES", 500, 20, 100000);
    keep_transient = yai_lifecycle_env_int("YAI_LIFECYCLE_KEEP_TRANSIENT", 80, 5, 10000);

    memset(&stats, 0, sizeof(stats));

    if (yai_workspace_event_sink_paths(info.ws_id, events_log, sizeof(events_log), decision_log, sizeof(decision_log),
                                       evidence_log, sizeof(evidence_log), event_index, sizeof(event_index)) == 0)
    {
        yai_lifecycle_apply_file(info.ws_id, run_dir, events_log, keep_events, &stats);
        yai_lifecycle_apply_file(info.ws_id, run_dir, decision_log, keep_decisions, &stats);
        yai_lifecycle_apply_file(info.ws_id, run_dir, evidence_log, keep_evidence, &stats);
    }
    if (yai_workspace_governance_sink_paths(info.ws_id, gov_obj, sizeof(gov_obj), gov_lifecycle, sizeof(gov_lifecycle),
                                            gov_attach, sizeof(gov_attach), gov_index, sizeof(gov_index)) == 0)
    {
        yai_lifecycle_apply_file(info.ws_id, run_dir, gov_obj, keep_decisions, &stats);
        yai_lifecycle_apply_file(info.ws_id, run_dir, gov_lifecycle, keep_decisions, &stats);
        yai_lifecycle_apply_file(info.ws_id, run_dir, gov_attach, keep_decisions, &stats);
    }
    if (yai_workspace_authority_sink_paths(info.ws_id, authority_log, sizeof(authority_log), authority_res, sizeof(authority_res),
                                           authority_index, sizeof(authority_index)) == 0)
    {
        yai_lifecycle_apply_file(info.ws_id, run_dir, authority_log, keep_decisions, &stats);
        yai_lifecycle_apply_file(info.ws_id, run_dir, authority_res, keep_decisions, &stats);
    }
    if (yai_workspace_artifact_sink_paths(info.ws_id, artifact_meta, sizeof(artifact_meta), artifact_link, sizeof(artifact_link),
                                          artifact_index, sizeof(artifact_index)) == 0)
    {
        yai_lifecycle_apply_file(info.ws_id, run_dir, artifact_meta, keep_decisions, &stats);
        yai_lifecycle_apply_file(info.ws_id, run_dir, artifact_link, keep_decisions, &stats);
    }
    if (yai_workspace_enforcement_sink_paths(info.ws_id, enf_out, sizeof(enf_out), enf_link, sizeof(enf_link),
                                             enf_index, sizeof(enf_index)) == 0)
    {
        yai_lifecycle_apply_file(info.ws_id, run_dir, enf_out, keep_decisions, &stats);
        yai_lifecycle_apply_file(info.ws_id, run_dir, enf_link, keep_decisions, &stats);
    }
    if (yai_lifecycle_brain_paths(info.ws_id, brain_nodes, sizeof(brain_nodes), brain_edges, sizeof(brain_edges),
                                  transient_act, sizeof(transient_act), transient_ws, sizeof(transient_ws)) == 0)
    {
        yai_lifecycle_apply_file(info.ws_id, run_dir, brain_nodes, keep_graph_nodes, &stats);
        yai_lifecycle_apply_file(info.ws_id, run_dir, brain_edges, keep_graph_edges, &stats);
        yai_lifecycle_apply_file(info.ws_id, run_dir, transient_act, keep_transient, &stats);
        yai_lifecycle_apply_file(info.ws_id, run_dir, transient_ws, keep_transient, &stats);
    }

    if (yai_lifecycle_write_rollup_snapshot(&info, &stats,
                                            keep_events, keep_decisions, keep_evidence,
                                            keep_graph_nodes, keep_graph_edges, keep_transient,
                                            err, err_cap) != 0)
    {
        if (err && err_cap > 0 && err[0] == '\0')
            (void)snprintf(err, err_cap, "%s", "lifecycle_rollup_snapshot_failed");
        return -1;
    }

    if (snprintf(lifecycle_dir, sizeof(lifecycle_dir), "%s/lifecycle", run_dir) <= 0)
        return -1;
    if (snprintf(maintenance_index, sizeof(maintenance_index), "%s/maintenance.index.v1.json", lifecycle_dir) <= 0)
        return -1;
    if (yai_lifecycle_mkdirs(lifecycle_dir) != 0)
        return -1;

    now = time(NULL);
    yai_format_iso8601_utc(now, ts_iso, sizeof(ts_iso));
    f = fopen(maintenance_index, "w");
    if (!f)
        return -1;
    (void)fprintf(f,
                  "{"
                  "\"type\":\"yai.lifecycle.maintenance.index.v1\","
                  "\"workspace_id\":\"%s\","
                  "\"updated_at\":\"%s\","
                  "\"status\":\"%s\","
                  "\"files_touched\":%d,\"files_compacted\":%d,"
                  "\"records_total\":%ld,\"records_archived\":%ld,\"records_hot\":%ld,\"violations\":%ld,"
                  "\"hot_bounds\":{\"events\":%d,\"decisions\":%d,\"evidence\":%d,\"graph_nodes\":%d,\"graph_edges\":%d,\"transient\":%d},"
                  "\"matrix_ref\":\"%s\""
                  "}\n",
                  info.ws_id,
                  ts_iso,
                  stats.violations > 0 ? "degraded" : "ok",
                  stats.files_touched,
                  stats.files_compacted,
                  stats.records_total,
                  stats.records_archived,
                  stats.records_hot,
                  stats.violations,
                  keep_events,
                  keep_decisions,
                  keep_evidence,
                  keep_graph_nodes,
                  keep_graph_edges,
                  keep_transient,
                  yai_lifecycle_policy_matrix_ref());
    fclose(f);

    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.lifecycle.maintenance.v1\","
                 "\"workspace_id\":\"%s\","
                 "\"status\":\"%s\","
                 "\"stats\":{\"files_touched\":%d,\"files_compacted\":%d,\"records_total\":%ld,\"records_archived\":%ld,\"records_hot\":%ld,\"violations\":%ld},"
                 "\"hot_bounds\":{\"events\":%d,\"decisions\":%d,\"evidence\":%d,\"graph_nodes\":%d,\"graph_edges\":%d,\"transient\":%d},"
                 "\"paths\":{\"maintenance_index\":\"%s\",\"rollup\":\"%s/lifecycle/rollup.v1.json\",\"snapshot\":\"%s/lifecycle/snapshot.v1.json\"},"
                 "\"matrix_ref\":\"%s\""
                 "}",
                 info.ws_id,
                 stats.violations > 0 ? "degraded" : "ok",
                 stats.files_touched,
                 stats.files_compacted,
                 stats.records_total,
                 stats.records_archived,
                 stats.records_hot,
                 stats.violations,
                 keep_events,
                 keep_decisions,
                 keep_evidence,
                 keep_graph_nodes,
                 keep_graph_edges,
                 keep_transient,
                 maintenance_index,
                 run_dir,
                 run_dir,
                 yai_lifecycle_policy_matrix_ref());
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

int yai_session_build_workspace_lifecycle_status_json(char *out,
                                                      size_t out_cap,
                                                      char *err,
                                                      size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    char status[32];
    char bind_err[96];
    char run_dir[MAX_PATH_LEN];
    char index_path[MAX_PATH_LEN];
    char buf[4096];
    int n;

    if (err && err_cap > 0) err[0] = '\0';
    if (!out || out_cap == 0)
        return -1;
    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), bind_err, sizeof(bind_err)) != 0 ||
        strcmp(status, "active") != 0)
    {
        if (err && err_cap > 0)
            (void)snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_not_active");
        return -1;
    }
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), info.ws_id) != 0)
        return -1;
    if (snprintf(index_path, sizeof(index_path), "%s/lifecycle/maintenance.index.v1.json", run_dir) <= 0)
        return -1;
    if (yai_read_text(index_path, buf, sizeof(buf)) != 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{\"type\":\"yai.workspace.lifecycle.status.v1\",\"workspace_id\":\"%s\",\"status\":\"not_initialized\",\"maintenance_index\":\"%s\"}",
                     info.ws_id,
                     index_path);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    n = snprintf(out,
                 out_cap,
                 "{\"type\":\"yai.workspace.lifecycle.status.v1\",\"workspace_id\":\"%s\",\"status\":\"initialized\",\"maintenance\":%s}",
                 info.ws_id,
                 buf);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}
