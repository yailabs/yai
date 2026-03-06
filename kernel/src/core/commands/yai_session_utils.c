#define _POSIX_C_SOURCE 200809L

#include "yai_session_internal.h"
#include "ws_id.h"

#include <dirent.h>
#include <errno.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

static const char *yai_get_home(void)
{
    const char *home = getenv("HOME");
    return (home && home[0]) ? home : NULL;
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

static int yai_workspace_resolve_root_path(
    const char *ws_id,
    const char *root_path_opt,
    char *out,
    size_t out_cap)
{
    char abs_path[MAX_PATH_LEN];
    const char *home = yai_get_home();

    if (!ws_id || !out || out_cap == 0 || !home)
        return -1;

    if (!root_path_opt || !root_path_opt[0])
    {
        if (snprintf(out, out_cap, "%s/.yai/workspaces/%s", home, ws_id) <= 0)
            return -1;
        trim_trailing_slashes(out);
        return 0;
    }

    if (strstr(root_path_opt, "..") != NULL)
        return -1;

    if (root_path_opt[0] == '/')
    {
        if (snprintf(abs_path, sizeof(abs_path), "%s", root_path_opt) <= 0)
            return -1;
    }
    else
    {
        char cwd[MAX_PATH_LEN];
        if (!getcwd(cwd, sizeof(cwd)))
            return -1;
        if (snprintf(abs_path, sizeof(abs_path), "%s/%s", cwd, root_path_opt) <= 0)
            return -1;
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

static int yai_workspace_write_manifest(
    const char *ws_dir,
    const char *ws_id,
    const char *state,
    const char *root_path,
    long created_at,
    long updated_at)
{
    char path[MAX_PATH_LEN];
    FILE *f;

    if (!ws_dir || !ws_id || !state || !root_path)
        return -1;

    if (snprintf(path, sizeof(path), "%s/manifest.json", ws_dir) <= 0)
        return -1;

    f = fopen(path, "w");
    if (!f)
        return -1;

    fprintf(f,
            "{\n"
            "  \"ws_id\": \"%s\",\n"
            "  \"state\": \"%s\",\n"
            "  \"created_at\": %ld,\n"
            "  \"updated_at\": %ld,\n"
            "  \"layout\": \"v2\",\n"
            "  \"root_path\": \"%s\",\n"
            "  \"attachments\": []\n"
            "}\n",
            ws_id,
            state,
            created_at,
            updated_at,
            root_path);
    fclose(f);
    return 0;
}

int yai_session_read_workspace_info(const char *ws_id, yai_workspace_runtime_info_t *out)
{
    char manifest[MAX_PATH_LEN];
    FILE *f;
    char buf[4096];
    size_t r;

    if (!ws_id || !out)
        return -1;

    memset(out, 0, sizeof(*out));
    snprintf(out->ws_id, sizeof(out->ws_id), "%s", ws_id);
    snprintf(out->state, sizeof(out->state), "missing");
    snprintf(out->layout, sizeof(out->layout), "v2");

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
    (void)yai_session_extract_json_long(buf, "created_at", &out->created_at);
    if (yai_session_extract_json_long(buf, "updated_at", &out->updated_at) != 0)
        out->updated_at = out->created_at;

    if (out->root_path[0] == '\0')
    {
        const char *home = yai_get_home();
        if (home)
            snprintf(out->root_path, sizeof(out->root_path), "%s/.yai/workspaces/%s", home, ws_id);
    }

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
                     "%s{\"ws_id\":\"%s\",\"state\":\"%s\",\"root_path\":\"%s\"}",
                     first ? "" : ",",
                     info.ws_id,
                     info.state,
                     info.root_path);
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
    char engine_dir[MAX_PATH_LEN];
    char logs_dir[MAX_PATH_LEN];
    char root_path[MAX_PATH_LEN] = {0};
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
        snprintf(engine_dir, sizeof(engine_dir), "%s/engine", ws_dir) <= 0 ||
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

    if (yai_workspace_resolve_root_path(ws_id, root_path_opt, root_path, sizeof(root_path)) != 0)
        return -1;

    if (mkdir_if_missing(yai_dir, 0755) != 0 ||
        mkdir_if_missing(run_dir, 0755) != 0 ||
        mkdir_if_missing(ws_dir, 0755) != 0 ||
        mkdir_if_missing(auth_dir, 0755) != 0 ||
        mkdir_if_missing(events_dir, 0755) != 0 ||
        mkdir_if_missing(engine_dir, 0755) != 0 ||
        mkdir_if_missing(logs_dir, 0755) != 0 ||
        mkdir_parents(root_path, 0755) != 0)
        return -1;

    if (yai_workspace_write_manifest(ws_dir, ws_id, "active", root_path, (long)now, (long)now) != 0)
        return -1;

    if (info_out)
    {
        if (yai_session_read_workspace_info(ws_id, info_out) != 0)
            return -1;
    }

    return 0;
}
