static int yai_workspace_containment_surface_paths(yai_workspace_runtime_info_t *info)
{
    if (!info || !info->ws_id[0])
        return -1;

    if (yai_workspace_state_root_path(info->ws_id, info->state_root, sizeof(info->state_root)) != 0)
        return -1;
    if (yai_workspace_traces_root_path(info->ws_id, info->traces_root, sizeof(info->traces_root)) != 0)
        return -1;
    if (yai_workspace_artifacts_root_path(info->ws_id, info->artifacts_root, sizeof(info->artifacts_root)) != 0)
        return -1;
    if (yai_workspace_runtime_local_root_path(info->ws_id, info->runtime_local_root, sizeof(info->runtime_local_root)) != 0)
        return -1;
    if (snprintf(info->state_surface_path, sizeof(info->state_surface_path), "%s/workspace-state.json", info->state_root) <= 0)
        return -1;
    if (snprintf(info->traces_index_path, sizeof(info->traces_index_path), "%s/index.json", info->traces_root) <= 0)
        return -1;
    if (snprintf(info->artifacts_index_path, sizeof(info->artifacts_index_path), "%s/index.json", info->artifacts_root) <= 0)
        return -1;
    if (snprintf(info->runtime_surface_path, sizeof(info->runtime_surface_path), "%s/runtime-state.json", info->runtime_local_root) <= 0)
        return -1;
    if (snprintf(info->binding_state_path, sizeof(info->binding_state_path), "%s/binding.json", info->metadata_root) <= 0)
        return -1;
    if (snprintf(info->attach_descriptor_ref, sizeof(info->attach_descriptor_ref), "%s/attach-descriptor.json", info->runtime_local_root) <= 0)
        return -1;
    if (snprintf(info->execution_profile_ref, sizeof(info->execution_profile_ref), "%s/execution-profile.json", info->runtime_local_root) <= 0)
        return -1;
    return 0;
}

static void yai_workspace_security_defaults(yai_workspace_runtime_info_t *info)
{
    if (!info)
        return;
    snprintf(info->security_envelope_version, sizeof(info->security_envelope_version), "%s", "v1");
    snprintf(info->security_level_declared, sizeof(info->security_level_declared), "%s", "scoped");
    snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "logical");
    snprintf(info->security_enforcement_mode, sizeof(info->security_enforcement_mode), "%s", "runtime_scoped");
    snprintf(info->security_backend_mode, sizeof(info->security_backend_mode), "%s", "none");
    info->scope_process = 0;
    info->scope_filesystem = 1;
    info->scope_socket = 0;
    info->scope_network = 0;
    info->scope_resource = 0;
    info->scope_privilege = 0;
    info->scope_runtime_route = 1;
    info->scope_binding = 1;
    info->capability_sandbox_ready = 1;
    info->capability_hardened_fs = 1;
    info->capability_process_isolation = 0;
    info->capability_network_policy = 0;
    snprintf(info->execution_mode_requested, sizeof(info->execution_mode_requested), "%s", "scoped");
    snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "scoped");
    info->execution_mode_degraded = 0;
    snprintf(info->execution_degraded_reason, sizeof(info->execution_degraded_reason), "%s", "none");
    snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "none");
    snprintf(info->execution_advisory_scopes, sizeof(info->execution_advisory_scopes), "%s", "process,socket,network,resource,privilege");
    snprintf(info->process_intent, sizeof(info->process_intent), "%s", "shared_runtime");
    snprintf(info->channel_mode, sizeof(info->channel_mode), "%s", "global_control_scoped_route");
    snprintf(info->artifact_policy_mode, sizeof(info->artifact_policy_mode), "%s", "workspace_owned");
    snprintf(info->network_intent, sizeof(info->network_intent), "%s", "advisory_none");
    snprintf(info->resource_intent, sizeof(info->resource_intent), "%s", "advisory_none");
    snprintf(info->privilege_intent, sizeof(info->privilege_intent), "%s", "inherited_host");
    info->attach_descriptor_ref[0] = '\0';
    info->execution_profile_ref[0] = '\0';
}

static int yai_workspace_security_level_is_valid(const char *level)
{
    if (!level || !level[0])
        return 0;
    return strcmp(level, "logical") == 0 ||
           strcmp(level, "scoped") == 0 ||
           strcmp(level, "isolated") == 0 ||
           strcmp(level, "sandboxed") == 0;
}

static void yai_workspace_security_recompute_effective(yai_workspace_runtime_info_t *info)
{
    const char *requested = NULL;
    if (!info)
        return;
    requested = yai_workspace_security_level_is_valid(info->security_level_declared)
                    ? info->security_level_declared
                    : "scoped";
    snprintf(info->execution_mode_requested, sizeof(info->execution_mode_requested), "%s", requested);

    if (!info->containment_ready || !info->namespace_valid)
    {
        snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "logical");
        snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "logical");
        info->execution_mode_degraded = strcmp(requested, "logical") != 0;
        snprintf(info->execution_degraded_reason,
                 sizeof(info->execution_degraded_reason),
                 "%s",
                 info->execution_mode_degraded ? "containment_not_ready_or_namespace_invalid" : "none");
        snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "none");
        return;
    }

    if (strcmp(requested, "sandboxed") == 0)
    {
        snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "scoped");
        snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "scoped");
        info->execution_mode_degraded = 1;
        snprintf(info->execution_degraded_reason, sizeof(info->execution_degraded_reason), "%s", "sandbox_backend_unavailable");
        snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "process,socket,network,resource,privilege");
        return;
    }

    if (strcmp(requested, "isolated") == 0)
    {
        if (info->scope_process || info->scope_socket || info->scope_network || info->scope_resource || info->scope_privilege)
        {
            snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "isolated");
            snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "isolated");
            info->execution_mode_degraded = 0;
            snprintf(info->execution_degraded_reason, sizeof(info->execution_degraded_reason), "%s", "none");
            snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "none");
        }
        else
        {
            snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "scoped");
            snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "scoped");
            info->execution_mode_degraded = 1;
            snprintf(info->execution_degraded_reason, sizeof(info->execution_degraded_reason), "%s", "isolated_scopes_not_enforced");
            snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "process,socket,network,resource,privilege");
        }
        return;
    }

    if (strcmp(requested, "logical") == 0)
    {
        snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "logical");
        snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "logical");
        info->execution_mode_degraded = 0;
        snprintf(info->execution_degraded_reason, sizeof(info->execution_degraded_reason), "%s", "none");
        snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "none");
        return;
    }

    snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "scoped");
    snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "scoped");
    info->execution_mode_degraded = 0;
    snprintf(info->execution_degraded_reason, sizeof(info->execution_degraded_reason), "%s", "none");
    snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "none");
}

static int yai_path_is_under(const char *root, const char *path)
{
    size_t n;
    if (!root || !root[0] || !path || !path[0])
        return 0;
    n = strlen(root);
    if (strncmp(root, path, n) != 0)
        return 0;
    return path[n] == '\0' || path[n] == '/';
}

static int yai_is_ws_runtime_path_valid(const char *ws_id,
                                        const char *actual_path,
                                        const char *expected_path)
{
    if (!ws_id || !ws_id[0] || !actual_path || !actual_path[0] || !expected_path || !expected_path[0])
        return 0;
    return strcmp(actual_path, expected_path) == 0;
}

static void yai_workspace_fill_shell_relation(yai_workspace_runtime_info_t *info)
{
    char cwd[MAX_PATH_LEN];
    if (!info)
        return;
    info->shell_cwd[0] = '\0';
    snprintf(info->shell_path_relation, sizeof(info->shell_path_relation), "%s", "unknown");
    if (!getcwd(cwd, sizeof(cwd)))
    {
        snprintf(info->shell_path_relation, sizeof(info->shell_path_relation), "%s", "cwd_unavailable");
        return;
    }
    snprintf(info->shell_cwd, sizeof(info->shell_cwd), "%s", cwd);
    if (!info->root_path[0])
    {
        snprintf(info->shell_path_relation, sizeof(info->shell_path_relation), "%s", "workspace_root_unset");
        return;
    }
    if (yai_path_is_under(info->root_path, cwd))
        snprintf(info->shell_path_relation, sizeof(info->shell_path_relation), "%s", "inside_workspace_root");
    else
        snprintf(info->shell_path_relation, sizeof(info->shell_path_relation), "%s", "outside_workspace_root");
}

static int mkdir_if_missing(const char *path, mode_t mode)
{
    struct stat st;
    if (stat(path, &st) == 0)
        return S_ISDIR(st.st_mode) ? 0 : -1;
    return mkdir(path, mode);
}

static int yai_file_read_all(const char *path, char *out, size_t out_cap)
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

static int yai_file_write_all(const char *path, const char *content)
{
    FILE *f;
    if (!path || !content)
        return -1;
    f = fopen(path, "wb");
    if (!f)
        return -1;
    if (fwrite(content, 1, strlen(content), f) != strlen(content))
    {
        fclose(f);
        return -1;
    }
    fclose(f);
    return 0;
}

static void yai_remove_range(char *buf, char *from, char *to_after)
{
    size_t tail_len;
    if (!buf || !from || !to_after || to_after < from)
        return;
    tail_len = strlen(to_after);
    memmove(from, to_after, tail_len + 1);
}

static int yai_remove_block_in_file(const char *path, const char *begin, const char *end)
{
    char buf[YAI_WS_JSON_IO_CAP];
    char *p, *q, *q_end;
    int changed = 0;
    if (!path || !begin || !end)
        return -1;
    if (yai_file_read_all(path, buf, sizeof(buf)) != 0)
        return 0;
    while ((p = strstr(buf, begin)) != NULL)
    {
        q = strstr(p, end);
        if (!q)
            break;
        q_end = q + strlen(end);
        if (*q_end == '\n')
            q_end++;
        yai_remove_range(buf, p, q_end);
        changed = 1;
    }
    if (changed)
        return yai_file_write_all(path, buf);
    return 0;
}

static int yai_replace_managed_block(const char *path, const char *begin, const char *end, const char *block)
{
    char buf[YAI_WS_JSON_IO_CAP];
    char out[YAI_WS_JSON_IO_CAP];
    char *p, *q;
    int n;
    if (!path || !begin || !end || !block)
        return -1;

    if (yai_file_read_all(path, buf, sizeof(buf)) != 0)
    {
        n = snprintf(out, sizeof(out), "%s", block);
        if (n <= 0 || (size_t)n >= sizeof(out))
            return -1;
        return yai_file_write_all(path, out);
    }

    p = strstr(buf, begin);
    if (!p)
    {
        n = snprintf(out, sizeof(out), "%s%s%s",
                     buf,
                     (buf[0] && buf[strlen(buf) - 1] != '\n') ? "\n" : "",
                     block);
        if (n <= 0 || (size_t)n >= sizeof(out))
            return -1;
        return yai_file_write_all(path, out);
    }

    q = strstr(p, end);
    if (!q)
    {
        n = snprintf(out, sizeof(out), "%s%s%s",
                     buf,
                     (buf[0] && buf[strlen(buf) - 1] != '\n') ? "\n" : "",
                     block);
        if (n <= 0 || (size_t)n >= sizeof(out))
            return -1;
        return yai_file_write_all(path, out);
    }

    q += strlen(end);
    if (*q == '\n')
        q++;

    n = snprintf(out, sizeof(out), "%.*s%s%s", (int)(p - buf), buf, block, q);
    if (n <= 0 || (size_t)n >= sizeof(out))
        return -1;
    return yai_file_write_all(path, out);
}

static int yai_session_ensure_shell_integration(void)
{
    const char *home = yai_get_home();
    char config_dir[MAX_PATH_LEN];
    char yai_cfg_dir[MAX_PATH_LEN];
    char shell_dir[MAX_PATH_LEN];
    char prompt_script[MAX_PATH_LEN];
    char zshrc[MAX_PATH_LEN];
    const char *script_content =
        "# yai managed prompt integration (do not edit manually)\n"
        "function prompt_yai_ws() {\n"
        "  emulate -L zsh\n"
        "  local tok cmd\n"
        "  cmd=\"$HOME/Developer/YAI/yai/tools/bin/yai-ws-token\"\n"
        "  [[ -x \"$cmd\" ]] || cmd=\"${commands[yai-ws-token]}\"\n"
        "  [[ -x \"$cmd\" ]] || return\n"
        "  tok=\"$(YAI_WS_BIND_SCOPE=tty $cmd 2>/dev/null)\"\n"
        "  [[ -n \"$tok\" ]] || return\n"
        "  p10k segment -f 255 -b 35 -t \"$tok\"\n"
        "}\n"
        "typeset -ga POWERLEVEL9K_LEFT_PROMPT_ELEMENTS\n"
        "typeset -ga POWERLEVEL9K_RIGHT_PROMPT_ELEMENTS\n"
        "POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(${POWERLEVEL9K_LEFT_PROMPT_ELEMENTS:#yai_ws})\n"
        "POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(${POWERLEVEL9K_LEFT_PROMPT_ELEMENTS:#context})\n"
        "POWERLEVEL9K_RIGHT_PROMPT_ELEMENTS=(${POWERLEVEL9K_RIGHT_PROMPT_ELEMENTS:#context})\n"
        "if (( ${POWERLEVEL9K_LEFT_PROMPT_ELEMENTS[(I)vcs]} <= ${#POWERLEVEL9K_LEFT_PROMPT_ELEMENTS} )); then\n"
        "  integer _yai_i=${POWERLEVEL9K_LEFT_PROMPT_ELEMENTS[(I)vcs]}\n"
        "  POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(\n"
        "    ${POWERLEVEL9K_LEFT_PROMPT_ELEMENTS[1,$((_yai_i-1))]}\n"
        "    yai_ws\n"
        "    ${POWERLEVEL9K_LEFT_PROMPT_ELEMENTS[_yai_i,-1]}\n"
        "  )\n"
        "  unset _yai_i\n"
        "else\n"
        "  POWERLEVEL9K_LEFT_PROMPT_ELEMENTS+=(yai_ws)\n"
        "fi\n";
    const char *zshrc_block =
        YAI_MANAGED_BEGIN "\n"
        "if [[ -f \"$HOME/.config/yai/shell/yai-prompt.zsh\" ]]; then\n"
        "  source \"$HOME/.config/yai/shell/yai-prompt.zsh\"\n"
        "fi\n"
        YAI_MANAGED_END "\n";

    if (!home || !home[0])
        return -1;
    if (snprintf(config_dir, sizeof(config_dir), "%s/.config", home) <= 0)
        return -1;
    if (snprintf(yai_cfg_dir, sizeof(yai_cfg_dir), "%s/.config/yai", home) <= 0)
        return -1;
    if (snprintf(shell_dir, sizeof(shell_dir), "%s/.config/yai/shell", home) <= 0)
        return -1;
    if (snprintf(prompt_script, sizeof(prompt_script), "%s/yai-prompt.zsh", shell_dir) <= 0)
        return -1;
    if (snprintf(zshrc, sizeof(zshrc), "%s/.zshrc", home) <= 0)
        return -1;

    (void)mkdir_if_missing(config_dir, 0755);
    (void)mkdir_if_missing(yai_cfg_dir, 0755);
    (void)mkdir_if_missing(shell_dir, 0755);

    if (yai_file_write_all(prompt_script, script_content) != 0)
        return -1;

    /* Cleanup legacy manual patches injected during migration/debug rounds. */
    (void)yai_remove_block_in_file(zshrc,
                                   "# --- YAI workspace token (left prompt, robust) ---",
                                   "# --- end ---");
    (void)yai_remove_block_in_file(zshrc,
                                   "# ==== YAI prompt patch (path -> workspace -> git) ====",
                                   "# ==== end patch ====");
    {
        char p10k[MAX_PATH_LEN];
        if (snprintf(p10k, sizeof(p10k), "%s/.p10k.zsh", home) > 0)
        {
            (void)yai_remove_block_in_file(p10k,
                                           "# === YAI workspace segment ===",
                                           "# === end YAI workspace segment ===");
        }
    }

    return yai_replace_managed_block(zshrc, YAI_MANAGED_BEGIN, YAI_MANAGED_END, zshrc_block);
}

static int mkdir_parents(const char *path, mode_t mode)
{
    char tmp[MAX_PATH_LEN];
    size_t len;
    char *p;

    if (!path || !path[0])
        return -1;

    snprintf(tmp, sizeof(tmp), "%s", path);
    len = strlen(tmp);
    if (len == 0 || len >= sizeof(tmp))
        return -1;

    for (p = tmp + 1; *p; ++p)
    {
        if (*p != '/')
            continue;
        *p = '\0';
        if (mkdir_if_missing(tmp, mode) != 0)
            return -1;
        *p = '/';
    }

    return mkdir_if_missing(tmp, mode);
}

static int remove_tree(const char *path)
{
    DIR *d = opendir(path);
    if (!d)
        return (errno == ENOENT) ? 0 : -1;

    struct dirent *ent;
    while ((ent = readdir(d)) != NULL)
    {
        if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0)
            continue;

        char child[MAX_PATH_LEN];
        int n = snprintf(child, sizeof(child), "%s/%s", path, ent->d_name);
        if (n <= 0 || (size_t)n >= sizeof(child))
        {
            closedir(d);
            return -1;
        }

        struct stat st;
        if (stat(child, &st) != 0)
        {
            if (errno == ENOENT)
                continue;
            closedir(d);
            return -1;
        }

        if (S_ISDIR(st.st_mode))
        {
            if (remove_tree(child) != 0)
            {
                closedir(d);
                return -1;
            }
        }
        else if (unlink(child) != 0)
        {
            closedir(d);
            return -1;
        }
    }

    closedir(d);
    return rmdir(path);
}

static void trim_trailing_slashes(char *path)
{
    size_t len;
    if (!path)
        return;
    len = strlen(path);
    while (len > 1 && path[len - 1] == '/')
    {
        path[len - 1] = '\0';
        len--;
    }
}

static int yai_session_extract_json_long(const char *json, const char *key, long *out)
{
    char needle[64];
    const char *p;
    char *end = NULL;

    if (!json || !key || !out)
        return -1;

    if (snprintf(needle, sizeof(needle), "\"%s\"", key) <= 0)
        return -1;

    p = strstr(json, needle);
    if (!p)
        return -1;
    p = strchr(p, ':');
    if (!p)
        return -1;
    p++;
    while (*p == ' ' || *p == '\t')
        p++;

    *out = strtol(p, &end, 10);
    if (end == p)
        return -1;
    return 0;
}

static int yai_session_extract_json_bool(const char *json, const char *key, int *out)
{
    char needle[64];
    const char *p;

    if (!json || !key || !out)
        return -1;

    if (snprintf(needle, sizeof(needle), "\"%s\"", key) <= 0)
        return -1;

    p = strstr(json, needle);
    if (!p)
        return -1;
    p = strchr(p, ':');
    if (!p)
        return -1;
    p++;
    while (*p == ' ' || *p == '\t')
        p++;

    if (strncmp(p, "true", 4) == 0)
    {
        *out = 1;
        return 0;
    }
    if (strncmp(p, "false", 5) == 0)
    {
        *out = 0;
        return 0;
    }
    return -1;
}

static int yai_session_extract_json_double(const char *json, const char *key, double *out)
{
    char needle[64];
    const char *p;
    char *end = NULL;

    if (!json || !key || !out)
        return -1;
    if (snprintf(needle, sizeof(needle), "\"%s\"", key) <= 0)
        return -1;

    p = strstr(json, needle);
    if (!p)
        return -1;
    p = strchr(p, ':');
    if (!p)
        return -1;
    p++;
    while (*p == ' ' || *p == '\t')
        p++;

    *out = strtod(p, &end);
    if (end == p)
        return -1;
    return 0;
}

static int yai_workspace_manifest_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), "") != 0)
        return -1;
    if (snprintf(out, out_cap, "%s%s/manifest.json", run_dir, ws_id) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_binding_path(char *out, size_t out_cap)
{
    const char *scope = getenv("YAI_WS_BIND_SCOPE");
    const char *home = yai_get_home();
    const char *tty = NULL;
    char tty_tag[128];
    size_t j = 0;
    size_t i = 0;

    if (!home || !out || out_cap == 0)
        return -1;

    if (!scope || scope[0] == '\0' || strcmp(scope, "tty") == 0 || strcmp(scope, "terminal") == 0)
    {
        tty = ttyname(STDIN_FILENO);
        if (!tty || !tty[0])
            tty = ttyname(STDOUT_FILENO);
        if (!tty || !tty[0])
            tty = ttyname(STDERR_FILENO);

        if (tty && tty[0])
        {
            for (i = 0; tty[i] != '\0' && j + 1 < sizeof(tty_tag); i++)
            {
                unsigned char c = (unsigned char)tty[i];
                if (isalnum(c))
                    tty_tag[j++] = (char)c;
                else
                    tty_tag[j++] = '_';
            }
            tty_tag[j] = '\0';
            if (tty_tag[0] == '\0')
                snprintf(tty_tag, sizeof(tty_tag), "%s", "unknown_tty");

            if (snprintf(out, out_cap, "%s/.yai/session/by-tty/%s.json", home, tty_tag) <= 0)
                return -1;
            return 0;
        }
    }

    if (snprintf(out, out_cap, "%s/.yai/session/active_workspace.json", home) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_binding_write(const char *ws_id, const char *ws_alias)
{
    char path[MAX_PATH_LEN];
    char session_dir[MAX_PATH_LEN];
    FILE *f;
    time_t now = time(NULL);
    const char *home = yai_get_home();

    if (!home || !ws_id)
        return -1;
    if (snprintf(session_dir, sizeof(session_dir), "%s/.yai/session", home) <= 0)
        return -1;
    if (mkdir_parents(session_dir, 0755) != 0)
        return -1;
    if (yai_workspace_binding_path(path, sizeof(path)) != 0)
        return -1;
    {
        char *slash = strrchr(path, '/');
        if (slash)
        {
            char parent[MAX_PATH_LEN];
            size_t n = (size_t)(slash - path);
            if (n >= sizeof(parent))
                return -1;
            memcpy(parent, path, n);
            parent[n] = '\0';
            if (mkdir_parents(parent, 0755) != 0)
                return -1;
        }
    }

    f = fopen(path, "w");
    if (!f)
        return -1;

    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.binding.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"workspace_alias\": \"%s\",\n"
            "  \"bound_at\": %ld,\n"
            "  \"source\": \"explicit\"\n"
            "}\n",
            ws_id,
            (ws_alias && ws_alias[0]) ? ws_alias : ws_id,
            (long)now);
    fclose(f);
    return 0;
}

static int yai_workspace_binding_read(char *ws_id, size_t ws_id_cap, char *ws_alias, size_t ws_alias_cap)
{
    char path[MAX_PATH_LEN];
    FILE *f;
    char buf[1024];
    size_t r;

    if (!ws_id || ws_id_cap == 0 || !ws_alias || ws_alias_cap == 0)
        return -1;
    ws_id[0] = '\0';
    ws_alias[0] = '\0';

    if (yai_workspace_binding_path(path, sizeof(path)) != 0)
        return -1;
    f = fopen(path, "r");
    if (!f)
        return -1;
    r = fread(buf, 1, sizeof(buf) - 1, f);
    fclose(f);
    buf[r] = '\0';

    if (yai_session_extract_json_string(buf, "workspace_id", ws_id, ws_id_cap) != 0)
        return -1;
    (void)yai_session_extract_json_string(buf, "workspace_alias", ws_alias, ws_alias_cap);
    if (ws_alias[0] == '\0')
        snprintf(ws_alias, ws_alias_cap, "%s", ws_id);
    return 0;
}

static int yai_workspace_resolve_root_path(
    const char *ws_id,
    const char *root_path_opt,
    char *anchor_mode_out,
    size_t anchor_mode_cap,
    char *out,
    size_t out_cap)
{
    char abs_path[MAX_PATH_LEN];
    char store_root[MAX_PATH_LEN];
    const char *home = yai_get_home();

    if (!ws_id || !out || out_cap == 0 || !home)
        return -1;

    if (anchor_mode_out && anchor_mode_cap > 0)
        anchor_mode_out[0] = '\0';

    if (!root_path_opt || !root_path_opt[0])
    {
        if (yai_workspace_store_root_path(store_root, sizeof(store_root)) != 0)
            return -1;
        if (snprintf(out, out_cap, "%s/%s", store_root, ws_id) <= 0)
            return -1;
        trim_trailing_slashes(out);
        if (anchor_mode_out && anchor_mode_cap > 0)
            snprintf(anchor_mode_out, anchor_mode_cap, "%s",
                     getenv("YAI_WORKSPACE_ROOT") ? "managed_custom_root" : "managed_default_root");
        return 0;
    }

    if (strstr(root_path_opt, "..") != NULL)
        return -1;

    if (root_path_opt[0] == '/')
    {
        if (snprintf(abs_path, sizeof(abs_path), "%s", root_path_opt) <= 0)
            return -1;
        if (anchor_mode_out && anchor_mode_cap > 0)
            snprintf(anchor_mode_out, anchor_mode_cap, "%s", "explicit_absolute");
    }
    else
    {
        char cwd[MAX_PATH_LEN];
        if (!getcwd(cwd, sizeof(cwd)))
            return -1;
        if (snprintf(abs_path, sizeof(abs_path), "%s/%s", cwd, root_path_opt) <= 0)
            return -1;
        if (anchor_mode_out && anchor_mode_cap > 0)
            snprintf(anchor_mode_out, anchor_mode_cap, "%s", "explicit_relative");
    }

    trim_trailing_slashes(abs_path);

    snprintf(out, out_cap, "%s", abs_path);
    return 0;
}

int yai_session_extract_json_string(const char *json, const char *key, char *out, size_t out_cap)
{
    if (!json || !key || !out || out_cap == 0)
        return -1;

    char needle[64];
    int nn = snprintf(needle, sizeof(needle), "\"%s\"", key);
    if (nn <= 0 || (size_t)nn >= sizeof(needle))
        return -1;

    const char *p = strstr(json, needle);
    if (!p)
        return -1;
    p = strchr(p, ':');
    if (!p)
        return -1;
    p++;
    while (*p == ' ' || *p == '\t')
        p++;
    if (*p != '"')
        return -1;
    p++;
    const char *q = strchr(p, '"');
    if (!q)
        return -1;

    size_t len = (size_t)(q - p);
    if (len >= out_cap)
        len = out_cap - 1;
    memcpy(out, p, len);
    out[len] = '\0';
    return 0;
}

int yai_session_extract_argv_first(const char *json, char *out, size_t out_cap)
{
    const char *p = strstr(json ? json : "", "\"argv\"");
    if (!p)
        return -1;
    p = strchr(p, '[');
    if (!p)
        return -1;
    p++;
    while (*p == ' ' || *p == '\t')
        p++;
    if (*p != '"')
        return -1;
    p++;
    const char *q = strchr(p, '"');
    if (!q)
        return -1;
    size_t len = (size_t)(q - p);
    if (len >= out_cap)
        len = out_cap - 1;
    memcpy(out, p, len);
    out[len] = '\0';
    return 0;
}

int yai_session_extract_argv_flag_value(
    const char *json,
    const char *flag_a,
    const char *flag_b,
    char *out,
    size_t out_cap)
{
    if (!json || !out || out_cap == 0)
        return -1;

    const char *p = strstr(json, "\"argv\"");
    if (!p)
        return -1;
    p = strchr(p, '[');
    if (!p)
        return -1;
    p++;

    char prev[128] = {0};
    while (*p && *p != ']')
    {
        while (*p && *p != '"' && *p != ']')
            p++;
        if (*p != '"')
            break;
        p++;
        const char *q = strchr(p, '"');
        if (!q)
            break;

        size_t len = (size_t)(q - p);
        char cur[128];
        if (len >= sizeof(cur))
            len = sizeof(cur) - 1;
        memcpy(cur, p, len);
        cur[len] = '\0';

        if ((flag_a && strcmp(prev, flag_a) == 0) ||
            (flag_b && strcmp(prev, flag_b) == 0))
        {
            size_t out_len = strlen(cur);
            if (out_len >= out_cap)
                out_len = out_cap - 1;
            memcpy(out, cur, out_len);
            out[out_len] = '\0';
            return 0;
        }

        snprintf(prev, sizeof(prev), "%s", cur);
        p = q + 1;
    }

    return -1;
}

int yai_session_path_exists(const char *path)
{
    struct stat st;
    return (path && stat(path, &st) == 0) ? 1 : 0;
}

int yai_session_build_run_path(char *out, size_t out_cap, const char *suffix)
{
    const char *home = yai_get_home();
    if (!home || !out || out_cap == 0)
        return -1;

    int n = snprintf(out, out_cap, "%s/.yai/run/%s", home, suffix ? suffix : "");
    if (n <= 0 || (size_t)n >= out_cap)
        return -1;
    return 0;
}

static int yai_workspace_write_manifest_path(
    const char *manifest_path,
    const yai_workspace_runtime_info_t *info)
{
    FILE *f;

    if (!manifest_path || !info)
        return -1;

    f = fopen(manifest_path, "w");
    if (!f)
        return -1;
    fprintf(f, "{\n");
    fprintf(f, "  \"type\": \"yai.workspace.manifest.v1\",\n");
    fprintf(f, "  \"schema\": \"workspace-runtime.v1\",\n");
    fprintf(f, "  \"ws_id\": \"%s\",\n", info->ws_id);
    fprintf(f, "  \"state\": \"%s\",\n", info->state[0] ? info->state : "created");
    fprintf(f, "  \"created_at\": %ld,\n", info->created_at);
    fprintf(f, "  \"updated_at\": %ld,\n", info->updated_at);
    fprintf(f, "  \"layout\": \"v3\",\n");
    fprintf(f, "  \"containment_layout\": \"%s\",\n", info->containment_layout[0] ? info->containment_layout : "v1");
    fprintf(f, "  \"root_path\": \"%s\",\n", info->root_path);
    fprintf(f, "  \"identity\": {\"workspace_id\": \"%s\", \"workspace_alias\": \"%s\", \"workspace_root\": \"%s\"},\n",
            info->ws_id, info->workspace_alias[0] ? info->workspace_alias : info->ws_id, info->root_path);
    fprintf(f, "  \"lifecycle\": {\"workspace_state\": \"%s\", \"created_at\": %ld, \"activated_at\": %ld, \"last_attached_at\": %ld, \"last_updated_at\": %ld},\n",
            info->state[0] ? info->state : "created", info->created_at, info->activated_at, info->last_attached_at, info->updated_at);
    fprintf(f, "  \"root_model\": {\"workspace_store_root\": \"%s\", \"workspace_root\": \"%s\", \"runtime_state_root\": \"%s\", \"metadata_root\": \"%s\", \"state_root\": \"%s\", \"traces_root\": \"%s\", \"artifacts_root\": \"%s\", \"runtime_local_root\": \"%s\", \"root_anchor_mode\": \"%s\"},\n",
            info->workspace_store_root, info->root_path, info->runtime_state_root, info->metadata_root, info->state_root, info->traces_root, info->artifacts_root, info->runtime_local_root, info->root_anchor_mode[0] ? info->root_anchor_mode : "managed_default_root");
    fprintf(f, "  \"containment\": {\"ready\": %s, \"state_surface\": \"%s\", \"traces_index\": \"%s\", \"artifacts_index\": \"%s\", \"runtime_surface\": \"%s\", \"binding_surface\": \"%s\"},\n",
            info->containment_ready ? "true" : "false", info->state_surface_path, info->traces_index_path, info->artifacts_index_path, info->runtime_surface_path, info->binding_state_path);
    fprintf(f, "  \"security_envelope\": {\n");
    fprintf(f, "    \"security_envelope_version\": \"%s\",\n", info->security_envelope_version[0] ? info->security_envelope_version : "v1");
    fprintf(f, "    \"security_level_declared\": \"%s\",\n", info->security_level_declared[0] ? info->security_level_declared : "scoped");
    fprintf(f, "    \"security_level_effective\": \"%s\",\n", info->security_level_effective[0] ? info->security_level_effective : "logical");
    fprintf(f, "    \"security_enforcement_mode\": \"%s\",\n", info->security_enforcement_mode[0] ? info->security_enforcement_mode : "runtime_scoped");
    fprintf(f, "    \"security_backend_mode\": \"%s\",\n", info->security_backend_mode[0] ? info->security_backend_mode : "none");
    fprintf(f, "    \"scopes\": {\"process\": %s, \"filesystem\": %s, \"socket\": %s, \"network\": %s, \"resource\": %s, \"privilege\": %s, \"runtime_route\": %s, \"binding\": %s},\n",
            info->scope_process ? "true" : "false",
            info->scope_filesystem ? "true" : "false",
            info->scope_socket ? "true" : "false",
            info->scope_network ? "true" : "false",
            info->scope_resource ? "true" : "false",
            info->scope_privilege ? "true" : "false",
            info->scope_runtime_route ? "true" : "false",
            info->scope_binding ? "true" : "false");
    fprintf(f, "    \"capabilities\": {\"sandbox_ready\": %s, \"hardened_fs\": %s, \"process_isolation\": %s, \"network_policy\": %s}\n",
            info->capability_sandbox_ready ? "true" : "false",
            info->capability_hardened_fs ? "true" : "false",
            info->capability_process_isolation ? "true" : "false",
            info->capability_network_policy ? "true" : "false");
    fprintf(f, "  },\n");
    fprintf(f, "  \"execution_profile\": {\n");
    fprintf(f, "    \"execution_mode_requested\": \"%s\",\n", info->execution_mode_requested[0] ? info->execution_mode_requested : "scoped");
    fprintf(f, "    \"execution_mode_effective\": \"%s\",\n", info->execution_mode_effective[0] ? info->execution_mode_effective : "scoped");
    fprintf(f, "    \"execution_mode_degraded\": %s,\n", info->execution_mode_degraded ? "true" : "false");
    fprintf(f, "    \"execution_degraded_reason\": \"%s\",\n", info->execution_degraded_reason[0] ? info->execution_degraded_reason : "none");
    fprintf(f, "    \"execution_unsupported_scopes\": \"%s\",\n", info->execution_unsupported_scopes[0] ? info->execution_unsupported_scopes : "none");
    fprintf(f, "    \"execution_advisory_scopes\": \"%s\",\n", info->execution_advisory_scopes[0] ? info->execution_advisory_scopes : "none");
    fprintf(f, "    \"process_intent\": \"%s\",\n", info->process_intent[0] ? info->process_intent : "shared_runtime");
    fprintf(f, "    \"channel_mode\": \"%s\",\n", info->channel_mode[0] ? info->channel_mode : "global_control_scoped_route");
    fprintf(f, "    \"artifact_policy_mode\": \"%s\",\n", info->artifact_policy_mode[0] ? info->artifact_policy_mode : "workspace_owned");
    fprintf(f, "    \"network_intent\": \"%s\",\n", info->network_intent[0] ? info->network_intent : "advisory_none");
    fprintf(f, "    \"resource_intent\": \"%s\",\n", info->resource_intent[0] ? info->resource_intent : "advisory_none");
    fprintf(f, "    \"privilege_intent\": \"%s\",\n", info->privilege_intent[0] ? info->privilege_intent : "inherited_host");
    fprintf(f, "    \"attach_descriptor_ref\": \"%s\",\n", info->attach_descriptor_ref);
    fprintf(f, "    \"execution_profile_ref\": \"%s\"\n", info->execution_profile_ref);
    fprintf(f, "  },\n");
    fprintf(f, "  \"boundaries\": {\"execution_boundary\": true, \"context_boundary\": true, \"policy_boundary\": true, \"runtime_binding_boundary\": true, \"shell_binding_scope\": \"session\"},\n");
    fprintf(f, "  \"binding\": {\"session_binding\": \"%s\", \"runtime_attached\": %s, \"runtime_endpoint\": \"\", \"control_plane_attached\": %s},\n",
            info->session_binding, info->runtime_attached ? "true" : "false", info->control_plane_attached ? "true" : "false");
    fprintf(f, "  \"declared_context\": {\"declared_control_family\": \"%s\", \"declared_specialization\": \"%s\", \"declared_profile\": \"\", \"declared_context_source\": \"%s\"},\n",
            info->declared_control_family, info->declared_specialization, info->declared_context_source[0] ? info->declared_context_source : "unset");
    fprintf(f, "  \"inferred_context\": {\"last_inferred_family\": \"%s\", \"last_inferred_specialization\": \"%s\", \"last_inference_confidence\": %.3f},\n",
            info->inferred_family, info->inferred_specialization, info->inferred_confidence);
    {
        char evt_declared[96];
        char evt_business[96];
        char evt_enforcement[96];
        char evt_stage[48];
        char op_summary[192];
        const char *review_state;
        int evt_external = 0;
        yai_session_workspace_event_semantics(info,
                                              evt_declared, sizeof(evt_declared),
                                              evt_business, sizeof(evt_business),
                                              evt_enforcement, sizeof(evt_enforcement),
                                              evt_stage, sizeof(evt_stage),
                                              &evt_external);
        review_state = yai_workspace_review_state_from_effect(info->last_effect_summary);
        yai_workspace_operational_summary(evt_stage, evt_business, info->last_effect_summary, op_summary, sizeof(op_summary));
        fprintf(f, "  \"effective_state\": {\"effective_stack_ref\": \"%s\", \"effective_overlays_ref\": \"%s\", \"policy_attachments\": \"%s\", \"last_effect_summary\": \"%s\", \"last_authority_summary\": \"%s\", \"last_evidence_summary\": \"%s\", \"last_event_ref\": \"%s\", \"business_specialization\": \"%s\", \"enforcement_specialization\": \"%s\", \"flow_stage\": \"%s\", \"external_effect_boundary\": %s, \"review_state\": \"%s\", \"operational_summary\": \"%s\"},\n",
                info->effective_stack_ref,
                info->effective_overlays_ref,
                info->policy_attachments_csv,
                info->last_effect_summary,
                info->last_authority_summary,
                info->last_evidence_summary,
                info->last_resolution_trace_ref,
                evt_business,
                evt_enforcement,
                evt_stage,
                evt_external ? "true" : "false",
                review_state,
                op_summary);
    }
    fprintf(f, "  \"runtime\": {\"isolation_mode\": \"%s\", \"debug_mode\": %s, \"last_resolution_trace_ref\": \"%s\"},\n",
            info->isolation_mode[0] ? info->isolation_mode : "process", info->debug_mode ? "true" : "false", info->last_resolution_trace_ref);
    fprintf(f, "  \"inspect\": {\"last_resolution_summary\": \"%s\"},\n", info->last_resolution_summary);
    fprintf(f, "  \"runtime_owner\": \"yai\",\n");
    fprintf(f, "  \"attachments\": [");
    if (info->policy_attachments_csv[0])
    {
        char refs[sizeof(info->policy_attachments_csv)];
        char *tok;
        char *save = NULL;
        int first = 1;
        snprintf(refs, sizeof(refs), "%s", info->policy_attachments_csv);
        tok = strtok_r(refs, ",", &save);
        while (tok)
        {
            if (tok[0])
            {
                fprintf(f, "%s{\"kind\":\"policy_object\",\"id\":\"%s\",\"active\":true}", first ? "" : ",", tok);
                first = 0;
            }
            tok = strtok_r(NULL, ",", &save);
        }
    }
    fprintf(f, "],\n");
    fprintf(f, "  \"capabilities\": {\"workspace_scope\": true, \"attachment_ready\": true}\n");
    fprintf(f, "}\n");
    fclose(f);
    return 0;
}
