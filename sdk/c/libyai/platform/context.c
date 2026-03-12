/* SPDX-License-Identifier: Apache-2.0 */

#define _POSIX_C_SOURCE 200809L

#include <yai/sdk/context.h>
#include <yai/sdk/paths.h>
#include <yai/sdk/client.h>
#include <yai/sdk/errors.h>
#include <yai/sdk/container.h>

#include <cJSON.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static int is_valid_container_id(const char *container_id)
{
    const char *p;
    if (!container_id || !container_id[0])
        return 0;
    if (strchr(container_id, '/') || strstr(container_id, "..") || container_id[0] == '~')
        return 0;
    for (p = container_id; *p; p++)
    {
        const char c = *p;
        const int ok =
            (c >= 'a' && c <= 'z') ||
            (c >= 'A' && c <= 'Z') ||
            (c >= '0' && c <= '9') ||
            (c == '_') || (c == '-') || (c == '.');
        if (!ok)
            return 0;
    }
    return 1;
}

static const char *home_dir(void)
{
    const char *home = getenv("HOME");
    return (home && home[0]) ? home : NULL;
}

static int ensure_dir(const char *path, mode_t mode)
{
    struct stat st;
    if (stat(path, &st) == 0)
        return S_ISDIR(st.st_mode) ? 0 : -1;
    return mkdir(path, mode);
}

static int ensure_context_parent_dirs(void)
{
    char yai_dir[512];
    char ctx_dir[512];
    const char *home = home_dir();
    if (!home)
        return -1;
    if (snprintf(yai_dir, sizeof(yai_dir), "%s/.yai", home) <= 0)
        return -1;
    if (snprintf(ctx_dir, sizeof(ctx_dir), "%s/.yai/context", home) <= 0)
        return -1;
    if (ensure_dir(yai_dir, 0755) != 0)
        return -1;
    if (ensure_dir(ctx_dir, 0755) != 0)
        return -1;
    return 0;
}

static int context_file_path(char *out, size_t cap)
{
    const char *home = home_dir();
    if (!home || !out || cap == 0)
        return -1;
    if (snprintf(out, cap, "%s/.yai/context/current_container", home) <= 0)
        return -1;
    return 0;
}

int yai_sdk_context_get_current_container(char *out_container_id, size_t out_cap)
{
    char path[512];
    FILE *f;
    char buf[128];
    size_t n;

    if (!out_container_id || out_cap == 0)
        return -1;
    out_container_id[0] = '\0';
    if (context_file_path(path, sizeof(path)) != 0)
        return -1;

    f = fopen(path, "r");
    if (!f)
        return 1;

    if (!fgets(buf, sizeof(buf), f))
    {
        fclose(f);
        return 1;
    }
    fclose(f);

    n = strlen(buf);
    while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r' || buf[n - 1] == ' ' || buf[n - 1] == '\t'))
    {
        buf[n - 1] = '\0';
        n--;
    }

    if (!is_valid_container_id(buf))
        return -1;

    if (snprintf(out_container_id, out_cap, "%s", buf) <= 0)
        return -1;
    return 0;
}

int yai_sdk_context_set_current_container(const char *container_id)
{
    char path[512];
    char tmp_path[544];
    FILE *f;

    if (!is_valid_container_id(container_id))
        return -1;
    if (ensure_context_parent_dirs() != 0)
        return -1;
    if (context_file_path(path, sizeof(path)) != 0)
        return -1;
    if (snprintf(tmp_path, sizeof(tmp_path), "%s.tmp", path) <= 0)
        return -1;

    f = fopen(tmp_path, "w");
    if (!f)
        return -1;
    if (fprintf(f, "%s\n", container_id) <= 0)
    {
        fclose(f);
        unlink(tmp_path);
        return -1;
    }
    fclose(f);
    if (rename(tmp_path, path) != 0)
    {
        unlink(tmp_path);
        return -1;
    }
    return 0;
}

int yai_sdk_context_switch_container(const char *container_id)
{
    return yai_sdk_context_set_current_container(container_id);
}

int yai_sdk_context_unset_container(void)
{
    char path[512];
    if (context_file_path(path, sizeof(path)) != 0)
        return -1;
    if (unlink(path) != 0)
    {
        return (access(path, F_OK) == 0) ? -1 : 0;
    }
    return 0;
}

int yai_sdk_context_clear_current_container(void)
{
    return yai_sdk_context_unset_container();
}

int yai_sdk_context_resolve_container(
    const char *explicit_container_id,
    char *out_container_id,
    size_t out_cap)
{
    if (!out_container_id || out_cap == 0)
        return -1;
    out_container_id[0] = '\0';

    if (explicit_container_id && explicit_container_id[0])
    {
        if (!is_valid_container_id(explicit_container_id))
            return -1;
        if (snprintf(out_container_id, out_cap, "%s", explicit_container_id) <= 0)
            return -1;
        return 0;
    }

    return yai_sdk_context_get_current_container(out_container_id, out_cap);
}

int yai_sdk_container_describe(const char *container_id, yai_sdk_container_info_t *out)
{
    yai_sdk_client_t *client = NULL;
    yai_sdk_client_opts_t opts = {0};
    yai_sdk_reply_t reply = {0};
    char req_json[512];
    int rc = 0;

    if (!out || !is_valid_container_id(container_id))
        return YAI_SDK_BAD_ARGS;

    memset(out, 0, sizeof(*out));
    snprintf(out->ws_id, sizeof(out->ws_id), "%s", container_id);

    opts.ws_id = container_id;
    opts.arming = 1;
    opts.role = "operator";
    opts.auto_handshake = 1;

    rc = yai_sdk_client_open(&client, &opts);
    if (rc != 0)
        return rc;

    if (snprintf(req_json,
                 sizeof(req_json),
                 "{\"type\":\"yai.control.call.v1\",\"target_plane\":\"runtime\",\"command_id\":\"%s\",\"argv\":[]}",
                 YAI_SDK_CMD_CONTAINER_STATUS) <= 0)
    {
        yai_sdk_client_close(client);
        return YAI_SDK_IO;
    }

    rc = yai_sdk_client_call_json(client, req_json, &reply);
    if (rc == YAI_SDK_BAD_ARGS || rc == YAI_SDK_NYI)
    {
        /* Legacy runtime fallback while older deployments are still present. */
        yai_sdk_reply_free(&reply);
        memset(&reply, 0, sizeof(reply));
        if (snprintf(req_json,
                     sizeof(req_json),
                     "{\"type\":\"yai.control.call.v1\",\"target_plane\":\"runtime\",\"command_id\":\"yai.runtime.ws_status\",\"argv\":[\"--ws-id\",\"%s\"]}",
                     container_id) <= 0)
        {
            yai_sdk_client_close(client);
            return YAI_SDK_IO;
        }
        rc = yai_sdk_client_call_json(client, req_json, &reply);
    }
    yai_sdk_client_close(client);
    if (rc != 0)
    {
        yai_sdk_reply_free(&reply);
        return rc;
    }

    if (reply.exec_reply_json && reply.exec_reply_json[0])
    {
        cJSON *root = cJSON_Parse(reply.exec_reply_json);
        if (root)
        {
            const cJSON *data = cJSON_GetObjectItemCaseSensitive(root, "data");
            if (cJSON_IsObject(data))
            {
                const cJSON *exists = cJSON_GetObjectItemCaseSensitive(data, "exists");
                const cJSON *state = cJSON_GetObjectItemCaseSensitive(data, "state");
                const cJSON *root_path = cJSON_GetObjectItemCaseSensitive(data, "root_path");
                out->exists = cJSON_IsTrue(exists) ? 1 : 0;
                out->valid = out->exists;
                if (cJSON_IsString(state) && state->valuestring)
                    snprintf(out->state, sizeof(out->state), "%s", state->valuestring);
                if (cJSON_IsString(root_path) && root_path->valuestring)
                    snprintf(out->root_path, sizeof(out->root_path), "%s", root_path->valuestring);
            }
            cJSON_Delete(root);
        }
    }

    yai_sdk_reply_free(&reply);
    return YAI_SDK_OK;
}

int yai_sdk_context_validate_current_container(yai_sdk_container_info_t *out)
{
    char container_id[64] = {0};
    int rc = yai_sdk_context_get_current_container(container_id, sizeof(container_id));
    if (rc == 1)
        return YAI_SDK_BAD_ARGS;
    if (rc != 0)
        return YAI_SDK_IO;
    return yai_sdk_container_describe(container_id, out);
}
