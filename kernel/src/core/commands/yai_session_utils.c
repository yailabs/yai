#define _POSIX_C_SOURCE 200809L

#include "yai_session_internal.h"

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <dirent.h>
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

int yai_session_read_manifest_layout(const char *ws_id, char *layout, size_t layout_cap, long *created_at)
{
    if (!ws_id || !layout || layout_cap == 0 || !created_at)
        return -1;

    char manifest[MAX_PATH_LEN];
    if (yai_session_build_run_path(manifest, sizeof(manifest), "") != 0)
        return -1;
    size_t base_len = strlen(manifest);
    int n = snprintf(manifest + base_len, sizeof(manifest) - base_len, "%s/manifest.json", ws_id);
    if (n <= 0 || (size_t)n >= (sizeof(manifest) - base_len))
        return -1;

    FILE *f = fopen(manifest, "r");
    if (!f)
        return -1;

    char buf[512];
    size_t r = fread(buf, 1, sizeof(buf) - 1, f);
    fclose(f);
    buf[r] = '\0';

    if (yai_session_extract_json_string(buf, "layout", layout, layout_cap) != 0)
        snprintf(layout, layout_cap, "unknown");

    const char *k = strstr(buf, "\"created_at\"");
    if (k)
    {
        k = strchr(k, ':');
        *created_at = k ? strtol(k + 1, NULL, 10) : 0;
    }
    else
    {
        *created_at = 0;
    }

    return 0;
}

int yai_session_build_workspace_list_json(char *out, size_t out_cap, int *count_out)
{
    if (!out || out_cap == 0)
        return -1;
    if (count_out)
        *count_out = 0;

    char run_dir[MAX_PATH_LEN];
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), "") != 0)
        return -1;

    DIR *d = opendir(run_dir);
    if (!d)
        return -1;

    size_t used = 0;
    int n = snprintf(out, out_cap, "[");
    if (n <= 0 || (size_t)n >= out_cap)
    {
        closedir(d);
        return -1;
    }
    used = (size_t)n;

    int first = 1;
    int count = 0;
    struct dirent *ent;
    while ((ent = readdir(d)) != NULL)
    {
        if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0)
            continue;

        char manifest[MAX_PATH_LEN];
        int m = snprintf(manifest, sizeof(manifest), "%s/%s/manifest.json", run_dir, ent->d_name);
        if (m <= 0 || (size_t)m >= sizeof(manifest))
            continue;
        if (!yai_session_path_exists(manifest))
            continue;

        n = snprintf(out + used, out_cap - used, "%s\"%s\"", first ? "" : ",", ent->d_name);
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

static int write_manifest(const char *ws_dir, const char *ws_id)
{
    char path[MAX_PATH_LEN];
    int n = snprintf(path, sizeof(path), "%s/manifest.json", ws_dir);
    if (n <= 0 || (size_t)n >= sizeof(path))
        return -1;

    FILE *f = fopen(path, "w");
    if (!f)
        return -1;

    time_t now = time(NULL);
    fprintf(f,
            "{\n"
            "  \"ws_id\": \"%s\",\n"
            "  \"created_at\": %ld,\n"
            "  \"layout\": \"v2\"\n"
            "}\n",
            ws_id, (long)now);
    fclose(f);
    return 0;
}

int yai_session_handle_workspace_action(const char *ws_id, const char *action)
{
    const char *home = yai_get_home();
    if (!home || !ws_id || !action)
        return -1;

    char yai_dir[MAX_PATH_LEN];
    char run_dir[MAX_PATH_LEN];
    char ws_dir[MAX_PATH_LEN];
    char auth_dir[MAX_PATH_LEN];
    char events_dir[MAX_PATH_LEN];
    char engine_dir[MAX_PATH_LEN];
    char logs_dir[MAX_PATH_LEN];

    if (snprintf(yai_dir, sizeof(yai_dir), "%s/.yai", home) <= 0 ||
        snprintf(run_dir, sizeof(run_dir), "%s/.yai/run", home) <= 0 ||
        snprintf(ws_dir, sizeof(ws_dir), "%s/.yai/run/%s", home, ws_id) <= 0 ||
        snprintf(auth_dir, sizeof(auth_dir), "%s/authority", ws_dir) <= 0 ||
        snprintf(events_dir, sizeof(events_dir), "%s/events", ws_dir) <= 0 ||
        snprintf(engine_dir, sizeof(engine_dir), "%s/engine", ws_dir) <= 0 ||
        snprintf(logs_dir, sizeof(logs_dir), "%s/logs", ws_dir) <= 0)
        return -1;

    if (strcmp(action, "destroy") == 0)
        return remove_tree(ws_dir);

    if (strcmp(action, "reset") == 0)
    {
        (void)remove_tree(ws_dir);
        action = "create";
    }

    if (strcmp(action, "create") == 0)
    {
        if (mkdir_if_missing(yai_dir, 0755) != 0 ||
            mkdir_if_missing(run_dir, 0755) != 0 ||
            mkdir_if_missing(ws_dir, 0755) != 0 ||
            mkdir_if_missing(auth_dir, 0755) != 0 ||
            mkdir_if_missing(events_dir, 0755) != 0 ||
            mkdir_if_missing(engine_dir, 0755) != 0 ||
            mkdir_if_missing(logs_dir, 0755) != 0)
            return -1;
        return write_manifest(ws_dir, ws_id);
    }

    return -2;
}
