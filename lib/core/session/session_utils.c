#define _POSIX_C_SOURCE 200809L

#include <yai/core/session.h>
#include "yai_session_internal.h"
#include <yai/core/workspace.h>
#include <yai/law/policy_effects.h>

#include <dirent.h>
#include <errno.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

#define YAI_WS_JSON_IO_CAP 32768

static const char *yai_get_home(void)
{
    const char *home = getenv("HOME");
    return (home && home[0]) ? home : NULL;
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
    /* metadata root currently co-locates with runtime state root by design. */
    return yai_workspace_runtime_state_root_path(ws_id, out, out_cap);
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
    const char *home = yai_get_home();
    if (!home || !out || out_cap == 0)
        return -1;
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
    const char *declared_source;

    if (!manifest_path || !info)
        return -1;

    f = fopen(manifest_path, "w");
    if (!f)
        return -1;

    declared_source = info->declared_context_source[0] ? info->declared_context_source : "unset";
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.manifest.v1\",\n"
            "  \"schema\": \"workspace-runtime.v1\",\n"
            "  \"ws_id\": \"%s\",\n"
            "  \"state\": \"%s\",\n"
            "  \"created_at\": %ld,\n"
            "  \"updated_at\": %ld,\n"
            "  \"layout\": \"v3\",\n"
            "  \"root_path\": \"%s\",\n"
            "  \"identity\": {\n"
            "    \"workspace_id\": \"%s\",\n"
            "    \"workspace_alias\": \"%s\",\n"
            "    \"workspace_root\": \"%s\"\n"
            "  },\n"
            "  \"lifecycle\": {\n"
            "    \"workspace_state\": \"%s\",\n"
            "    \"created_at\": %ld,\n"
            "    \"activated_at\": %ld,\n"
            "    \"last_attached_at\": %ld,\n"
            "    \"last_updated_at\": %ld\n"
            "  },\n"
            "  \"root_model\": {\n"
            "    \"workspace_store_root\": \"%s\",\n"
            "    \"workspace_root\": \"%s\",\n"
            "    \"runtime_state_root\": \"%s\",\n"
            "    \"metadata_root\": \"%s\",\n"
            "    \"root_anchor_mode\": \"%s\"\n"
            "  },\n"
            "  \"boundaries\": {\n"
            "    \"execution_boundary\": true,\n"
            "    \"context_boundary\": true,\n"
            "    \"policy_boundary\": true,\n"
            "    \"runtime_binding_boundary\": true,\n"
            "    \"shell_binding_scope\": \"session\"\n"
            "  },\n"
            "  \"binding\": {\n"
            "    \"session_binding\": \"%s\",\n"
            "    \"runtime_attached\": %s,\n"
            "    \"runtime_endpoint\": \"\",\n"
            "    \"control_plane_attached\": %s\n"
            "  },\n"
            "  \"declared_context\": {\n"
            "    \"declared_control_family\": \"%s\",\n"
            "    \"declared_specialization\": \"%s\",\n"
            "    \"declared_profile\": \"\",\n"
            "    \"declared_context_source\": \"%s\"\n"
            "  },\n"
            "  \"inferred_context\": {\n"
            "    \"last_inferred_family\": \"%s\",\n"
            "    \"last_inferred_specialization\": \"%s\",\n"
            "    \"last_inference_confidence\": %.3f\n"
            "  },\n"
            "  \"effective_state\": {\n"
            "    \"effective_stack_ref\": \"%s\",\n"
            "    \"effective_overlays_ref\": \"%s\",\n"
            "    \"last_effect_summary\": \"%s\",\n"
            "    \"last_authority_summary\": \"%s\",\n"
            "    \"last_evidence_summary\": \"%s\"\n"
            "  },\n"
            "  \"runtime\": {\n"
            "    \"isolation_mode\": \"%s\",\n"
            "    \"debug_mode\": %s,\n"
            "    \"last_resolution_trace_ref\": \"%s\"\n"
            "  },\n"
            "  \"inspect\": {\n"
            "    \"last_resolution_summary\": \"%s\"\n"
            "  },\n"
            "  \"runtime_owner\": \"yai\",\n"
            "  \"attachments\": [],\n"
            "  \"capabilities\": {\n"
            "    \"workspace_scope\": true,\n"
            "    \"attachment_ready\": true\n"
            "  }\n"
            "}\n",
            info->ws_id,
            info->state[0] ? info->state : "created",
            info->created_at,
            info->updated_at,
            info->root_path,
            info->ws_id,
            info->workspace_alias[0] ? info->workspace_alias : info->ws_id,
            info->root_path,
            info->state[0] ? info->state : "created",
            info->created_at,
            info->activated_at,
            info->last_attached_at,
            info->updated_at,
            info->workspace_store_root,
            info->root_path,
            info->runtime_state_root,
            info->metadata_root,
            info->root_anchor_mode[0] ? info->root_anchor_mode : "managed_default_root",
            info->session_binding,
            info->runtime_attached ? "true" : "false",
            info->control_plane_attached ? "true" : "false",
            info->declared_control_family,
            info->declared_specialization,
            declared_source,
            info->inferred_family,
            info->inferred_specialization,
            info->inferred_confidence,
            info->effective_stack_ref,
            info->effective_overlays_ref,
            info->last_effect_summary,
            info->last_authority_summary,
            info->last_evidence_summary,
            info->isolation_mode[0] ? info->isolation_mode : "process",
            info->debug_mode ? "true" : "false",
            info->last_resolution_trace_ref,
            info->last_resolution_summary);
    fclose(f);
    return 0;
}

static int yai_workspace_write_manifest_ws_id(const char *ws_id, const yai_workspace_runtime_info_t *info)
{
    char manifest[MAX_PATH_LEN];
    if (!ws_id || !info)
        return -1;
    if (yai_workspace_manifest_path(ws_id, manifest, sizeof(manifest)) != 0)
        return -1;
    return yai_workspace_write_manifest_path(manifest, info);
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
    snprintf(out->state, sizeof(out->state), "missing");
    snprintf(out->layout, sizeof(out->layout), "v3");
    snprintf(out->workspace_alias, sizeof(out->workspace_alias), "%s", ws_id);
    snprintf(out->isolation_mode, sizeof(out->isolation_mode), "process");
    snprintf(out->root_anchor_mode, sizeof(out->root_anchor_mode), "%s", "managed_default_root");
    if (yai_workspace_store_root_path(out->workspace_store_root, sizeof(out->workspace_store_root)) != 0)
        out->workspace_store_root[0] = '\0';
    if (yai_workspace_runtime_state_root_path(ws_id, out->runtime_state_root, sizeof(out->runtime_state_root)) != 0)
        out->runtime_state_root[0] = '\0';
    if (yai_workspace_metadata_root_path(ws_id, out->metadata_root, sizeof(out->metadata_root)) != 0)
        out->metadata_root[0] = '\0';

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
    (void)yai_session_extract_json_string(buf, "root_path", out->root_path, sizeof(out->root_path));
    (void)yai_session_extract_json_string(buf, "workspace_store_root", out->workspace_store_root, sizeof(out->workspace_store_root));
    (void)yai_session_extract_json_string(buf, "runtime_state_root", out->runtime_state_root, sizeof(out->runtime_state_root));
    (void)yai_session_extract_json_string(buf, "metadata_root", out->metadata_root, sizeof(out->metadata_root));
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

    if (out->root_path[0] == '\0')
    {
        if (out->workspace_store_root[0])
            snprintf(out->root_path, sizeof(out->root_path), "%s/%s", out->workspace_store_root, ws_id);
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

int yai_session_handle_workspace_action(
    const char *ws_id,
    const char *action,
    const char *root_path_opt,
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
    char root_path[MAX_PATH_LEN] = {0};
    char root_anchor_mode[32] = {0};
    char manifest_path[MAX_PATH_LEN];
    yai_workspace_runtime_info_t info;
    time_t now = time(NULL);

    if (!home || !ws_id || !action)
        return -1;
    if (!yai_ws_id_is_valid(ws_id))
        return -1;

    if (snprintf(yai_dir, sizeof(yai_dir), "%s/.yai", home) <= 0 ||
        snprintf(run_dir, sizeof(run_dir), "%s/.yai/run", home) <= 0 ||
        snprintf(ws_dir, sizeof(ws_dir), "%s/.yai/run/%s", home, ws_id) <= 0 ||
        snprintf(auth_dir, sizeof(auth_dir), "%s/authority", ws_dir) <= 0 ||
        snprintf(events_dir, sizeof(events_dir), "%s/events", ws_dir) <= 0 ||
        snprintf(exec_dir, sizeof(exec_dir), "%s/exec", ws_dir) <= 0 ||
        snprintf(logs_dir, sizeof(logs_dir), "%s/logs", ws_dir) <= 0)
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
        return -2;

    if (yai_workspace_resolve_root_path(ws_id,
                                        root_path_opt,
                                        root_anchor_mode,
                                        sizeof(root_anchor_mode),
                                        root_path,
                                        sizeof(root_path)) != 0)
        return -1;

    if (mkdir_if_missing(yai_dir, 0755) != 0 ||
        mkdir_if_missing(run_dir, 0755) != 0 ||
        mkdir_if_missing(ws_dir, 0755) != 0 ||
        mkdir_if_missing(auth_dir, 0755) != 0 ||
        mkdir_if_missing(events_dir, 0755) != 0 ||
        mkdir_if_missing(exec_dir, 0755) != 0 ||
        mkdir_if_missing(logs_dir, 0755) != 0 ||
        mkdir_parents(root_path, 0755) != 0)
        return -1;

    memset(&info, 0, sizeof(info));
    snprintf(info.ws_id, sizeof(info.ws_id), "%s", ws_id);
    snprintf(info.workspace_alias, sizeof(info.workspace_alias), "%s", ws_id);
    snprintf(info.state, sizeof(info.state), "%s", "created");
    snprintf(info.layout, sizeof(info.layout), "%s", "v3");
    snprintf(info.root_path, sizeof(info.root_path), "%s", root_path);
    snprintf(info.root_anchor_mode, sizeof(info.root_anchor_mode), "%s", root_anchor_mode[0] ? root_anchor_mode : "managed_default_root");
    if (yai_workspace_store_root_path(info.workspace_store_root, sizeof(info.workspace_store_root)) != 0)
        return -1;
    if (yai_workspace_runtime_state_root_path(ws_id, info.runtime_state_root, sizeof(info.runtime_state_root)) != 0)
        return -1;
    if (yai_workspace_metadata_root_path(ws_id, info.metadata_root, sizeof(info.metadata_root)) != 0)
        return -1;
    snprintf(info.isolation_mode, sizeof(info.isolation_mode), "%s", "process");
    info.created_at = (long)now;
    info.updated_at = (long)now;
    info.exists = 1;

    if (snprintf(manifest_path, sizeof(manifest_path), "%s/manifest.json", ws_dir) <= 0)
        return -1;
    if (yai_workspace_write_manifest_path(manifest_path, &info) != 0)
        return -1;

    if (info_out)
    {
        if (yai_session_read_workspace_info(ws_id, info_out) != 0)
            return -1;
    }

    return 0;
}

int yai_session_set_active_workspace(const char *ws_id, char *err, size_t err_cap)
{
    yai_workspace_runtime_info_t info;
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
    snprintf(info.session_binding, sizeof(info.session_binding), "%s", ws_id);
    info.activated_at = (long)time(NULL);
    info.updated_at = info.activated_at;
    if (yai_workspace_write_manifest_ws_id(ws_id, &info) != 0)
    {
        why = "manifest_write_failed";
        goto fail;
    }
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
    }

    if (yai_workspace_binding_path(path, sizeof(path)) != 0)
        return -1;
    if (unlink(path) != 0 && errno != ENOENT)
        return -1;
    return 0;
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

    if (yai_session_read_workspace_info(ws_id, info_out) != 0 || !info_out->exists)
    {
        snprintf(status_out, status_cap, "%s", "stale");
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "binding_workspace_missing");
        return -1;
    }

    if (info_out->workspace_alias[0] == '\0')
        snprintf(info_out->workspace_alias, sizeof(info_out->workspace_alias), "%s", ws_alias);
    snprintf(status_out, status_cap, "%s", "active");
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
                 "\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\"}"
                 "}",
                 info.ws_id,
                 info.workspace_alias,
                 info.state,
                 info.root_path,
                 info.root_anchor_mode[0] ? info.root_anchor_mode : "managed_default_root",
                 info.declared_control_family,
                 info.declared_specialization);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

static int yai_workspace_binding_validity(const char *binding_status)
{
    return (binding_status && strcmp(binding_status, "active") == 0) ? 1 : 0;
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

static int yai_embedded_law_path(char *out, size_t out_cap, const char *rel)
{
    const char *root = getenv("YAI_LAW_EMBED_ROOT");
    const char *base = NULL;
    const char *candidates[] = {"embedded/law", "../yai/embedded/law", "../../yai/embedded/law"};
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
        base = "embedded/law";
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

static int yai_embedded_family_exists(const char *family)
{
    char path[MAX_PATH_LEN];
    char json[YAI_WS_JSON_IO_CAP];
    char needle[160];
    if (!family || !family[0])
        return 0;
    if (yai_embedded_law_path(path, sizeof(path), "control-families/index/families.index.json") != 0)
        return 0;
    if (yai_read_text(path, json, sizeof(json)) != 0)
        return 0;
    if (snprintf(needle, sizeof(needle), "\"canonical_name\": \"%s\"", family) <= 0)
        return 0;
    return strstr(json, needle) != NULL;
}

static int yai_embedded_resolve_specialization_family(const char *specialization, char *family_out, size_t family_cap)
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
    if (yai_embedded_law_path(path, sizeof(path), "domain-specializations/index/specializations.index.json") != 0)
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

static int yai_embedded_specialization_matches_family(const char *family, const char *specialization)
{
    char inferred_family[96];
    if (!specialization || !specialization[0])
        return 1;
    if (yai_embedded_resolve_specialization_family(specialization, inferred_family, sizeof(inferred_family)) != 0)
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
    int rc;
    int n;
    if (!out || out_cap == 0)
        return -1;
    rc = yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err));
    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.status.v1\","
                 "\"active\":%s,"
                 "\"binding_status\":\"%s\","
                 "\"binding_valid\":%s,"
                 "\"runtime_attached\":%s,"
                 "\"isolation_mode\":\"%s\","
                 "\"debug_mode\":%s,"
                 "\"declared_context_present\":%s,"
                 "\"effective_context_present\":%s,"
                 "\"workspace_root\":\"%s\","
                 "\"workspace_store_root\":\"%s\","
                 "\"root_anchor_mode\":\"%s\","
                 "\"shell_path_relation\":\"%s\","
                 "\"reason\":\"%s\""
                 "}",
                 (rc == 0 && strcmp(status, "active") == 0) ? "true" : "false",
                 status[0] ? status : "invalid",
                 yai_workspace_binding_validity(status) ? "true" : "false",
                 (rc == 0 && info.runtime_attached) ? "true" : "false",
                 (rc == 0 && info.isolation_mode[0]) ? info.isolation_mode : "process",
                 (rc == 0 && info.debug_mode) ? "true" : "false",
                 (rc == 0 && yai_workspace_has_declared_context(&info)) ? "true" : "false",
                 (rc == 0 && yai_workspace_has_effective_context(&info)) ? "true" : "false",
                 (rc == 0) ? info.root_path : "",
                 (rc == 0) ? info.workspace_store_root : "",
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
    int rc;
    int n;
    if (!out || out_cap == 0)
        return -1;
    rc = yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err));
    if (rc != 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{\"type\":\"yai.workspace.inspect.v1\",\"binding_status\":\"%s\",\"reason\":\"%s\"}",
                     status[0] ? status : "invalid",
                     err[0] ? err : "binding_error");
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.inspect.v1\","
                 "\"binding_status\":\"active\","
                 "\"identity\":{\"workspace_id\":\"%s\",\"workspace_alias\":\"%s\",\"root_path\":\"%s\",\"state\":\"%s\"},"
                 "\"root_model\":{\"workspace_store_root\":\"%s\",\"runtime_state_root\":\"%s\",\"metadata_root\":\"%s\",\"root_anchor_mode\":\"%s\"},"
                 "\"shell\":{\"cwd\":\"%s\",\"cwd_relation\":\"%s\"},"
                 "\"session\":{\"session_binding\":\"%s\",\"runtime_attached\":%s,\"control_plane_attached\":%s,\"isolation_mode\":\"%s\",\"debug_mode\":%s},"
                 "\"normative\":{\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\",\"source\":\"%s\"},"
                 "\"inferred\":{\"family\":\"%s\",\"specialization\":\"%s\",\"confidence\":%.3f},"
                 "\"effective\":{\"stack_ref\":\"%s\",\"overlays_ref\":\"%s\",\"effect_summary\":\"%s\",\"authority_summary\":\"%s\",\"evidence_summary\":\"%s\"}},"
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
    if (family && family[0] && !yai_embedded_family_exists(resolved_family))
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "family_not_found");
        return -1;
    }

    if (specialization && specialization[0])
    {
        char inferred_family[96] = {0};
        if (yai_embedded_resolve_specialization_family(specialization, inferred_family, sizeof(inferred_family)) != 0)
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

    if (resolved_family[0] && !yai_embedded_family_exists(resolved_family))
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "family_not_found");
        return -1;
    }
    if (!yai_embedded_specialization_matches_family(resolved_family, specialization && specialization[0] ? specialization : info.declared_specialization))
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

int yai_session_build_workspace_policy_effective_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    int n;
    if (!out || out_cap == 0)
        return -1;
    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err)) != 0)
    {
        n = snprintf(out, out_cap,
                     "{\"type\":\"yai.workspace.policy.effective.v1\",\"binding_status\":\"%s\",\"reason\":\"%s\"}",
                     status[0] ? status : "invalid",
                     err[0] ? err : "binding_error");
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.policy.effective.v1\","
                 "\"workspace_id\":\"%s\","
                 "\"family_effective\":\"%s\","
                 "\"specialization_effective\":\"%s\","
                 "\"effective_stack_ref\":\"%s\","
                 "\"effective_overlays_ref\":\"%s\","
                 "\"precedence\":\"specialization+overlays\","
                 "\"effect_summary\":\"%s\","
                 "\"authority_summary\":\"%s\","
                 "\"evidence_summary\":\"%s\""
                 "}",
                 info.ws_id,
                 info.inferred_family[0] ? info.inferred_family : info.declared_control_family,
                 info.inferred_specialization[0] ? info.inferred_specialization : info.declared_specialization,
                 info.effective_stack_ref,
                 info.effective_overlays_ref,
                 info.last_effect_summary,
                 info.last_authority_summary,
                 info.last_evidence_summary);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

int yai_session_build_workspace_debug_resolution_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    int n;
    const char *context_source;
    if (!out || out_cap == 0)
        return -1;
    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err)) != 0)
    {
        n = snprintf(out, out_cap,
                     "{\"type\":\"yai.workspace.debug.resolution.v1\",\"binding_status\":\"%s\",\"reason\":\"%s\"}",
                     status[0] ? status : "invalid",
                     err[0] ? err : "binding_error");
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    context_source = info.declared_context_source[0] ? info.declared_context_source : "unset";
    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.debug.resolution.v1\","
                 "\"workspace_id\":\"%s\","
                 "\"context_source\":\"%s\","
                 "\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\"},"
                 "\"inferred\":{\"family\":\"%s\",\"specialization\":\"%s\",\"confidence\":%.3f},"
                 "\"effective\":{\"stack_ref\":\"%s\",\"overlays_ref\":\"%s\"},"
                 "\"precedence_outcome\":\"%s\","
                 "\"effect_outcome\":\"%s\","
                 "\"last_resolution_trace_ref\":\"%s\","
                 "\"last_resolution_summary\":\"%s\""
                 "}",
                 info.ws_id,
                 context_source,
                 info.declared_control_family,
                 info.declared_specialization,
                 info.inferred_family,
                 info.inferred_specialization,
                 info.inferred_confidence,
                 info.effective_stack_ref,
                 info.effective_overlays_ref,
                 "specialization+overlays",
                 info.last_effect_summary,
                 info.last_resolution_trace_ref,
                 info.last_resolution_summary);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

int yai_session_record_resolution_snapshot(const char *ws_id,
                                          const yai_law_resolution_output_t *law_out,
                                          char *err,
                                          size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    const yai_law_effective_stack_t *stack;
    int i;
    char overlays[192];
    size_t used = 0;
    int n;

    if (err && err_cap > 0)
        err[0] = '\0';
    if (!ws_id || !law_out)
        return -1;
    if (yai_session_read_workspace_info(ws_id, &info) != 0 || !info.exists)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_manifest_missing");
        return -1;
    }

    stack = &law_out->decision.stack;
    snprintf(info.inferred_family, sizeof(info.inferred_family), "%s", law_out->decision.family_id);
    snprintf(info.inferred_specialization, sizeof(info.inferred_specialization), "%s", law_out->decision.specialization_id);
    info.inferred_confidence = 1.0;
    snprintf(info.effective_stack_ref, sizeof(info.effective_stack_ref), "%s", stack->stack_id);
    snprintf(info.last_effect_summary, sizeof(info.last_effect_summary), "%s", yai_law_effect_name(law_out->decision.final_effect));
    snprintf(info.last_authority_summary, sizeof(info.last_authority_summary), "%s", stack->authority_profile);
    snprintf(info.last_evidence_summary, sizeof(info.last_evidence_summary), "%s", stack->evidence_profile);
    snprintf(info.last_resolution_trace_ref, sizeof(info.last_resolution_trace_ref), "%s", law_out->evidence.trace_id);
    snprintf(info.last_resolution_summary,
             sizeof(info.last_resolution_summary),
             "%s/%s => %s",
             law_out->decision.family_id,
             law_out->decision.specialization_id,
             yai_law_effect_name(law_out->decision.final_effect));

    overlays[0] = '\0';
    for (i = 0; i < stack->overlay_count; i++)
    {
        n = snprintf(overlays + used, sizeof(overlays) - used, "%s%s", (i == 0) ? "" : ",", stack->overlay_layers[i]);
        if (n <= 0 || (size_t)n >= (sizeof(overlays) - used))
            break;
        used += (size_t)n;
    }
    snprintf(info.effective_overlays_ref, sizeof(info.effective_overlays_ref), "%s", overlays);
    info.updated_at = (long)time(NULL);
    info.last_attached_at = info.updated_at;
    info.runtime_attached = 1;
    info.control_plane_attached = 1;
    if (info.session_binding[0] == '\0')
        snprintf(info.session_binding, sizeof(info.session_binding), "%s", ws_id);
    if (info.declared_context_source[0] == '\0')
        snprintf(info.declared_context_source, sizeof(info.declared_context_source), "%s", "unset");
    if (info.isolation_mode[0] == '\0')
        snprintf(info.isolation_mode, sizeof(info.isolation_mode), "%s", "process");

    if (yai_workspace_write_manifest_ws_id(ws_id, &info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "manifest_write_failed");
        return -1;
    }
    return 0;
}
