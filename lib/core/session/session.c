#define _POSIX_C_SOURCE 200809L

#include <yai/core/session.h>
#include "yai_session_internal.h"
#include <yai/api/runtime.h>
#include <yai/core/workspace.h>
#include <yai/law/resolver.h>
#include <yai/law/policy_effects.h>

#include <transport.h>
#include <protocol.h>

#include <stdlib.h>
#include <fcntl.h>
#include <errno.h>
#include <string.h>
#include <stdio.h>
#include <signal.h>
#include <unistd.h>
#include <sys/stat.h>
#include <time.h>
#include <yai_protocol_ids.h>

/* Global in-process session registry. */

yai_session_t g_session_registry[MAX_SESSIONS] = {0};

/* Internal helpers. */

static const char *yai_get_home(void)
{
    const char *home = getenv("HOME");
    return (home && strlen(home) > 0) ? home : NULL;
}

static int mkdir_if_missing(const char *path, mode_t mode)
{
    struct stat st;
    if (stat(path, &st) == 0)
        return S_ISDIR(st.st_mode) ? 0 : -1;

    return mkdir(path, mode);
}

static int ensure_run_tree(const char *home)
{
    char p1[MAX_PATH_LEN], p2[MAX_PATH_LEN];

    snprintf(p1, sizeof(p1), "%s/.yai", home);
    snprintf(p2, sizeof(p2), "%s/.yai/run", home);

    if (mkdir_if_missing(p1, 0755) != 0)
        return -1;
    if (mkdir_if_missing(p2, 0755) != 0)
        return -1;

    return 0;
}


/* Workspace path/state helpers. */

bool yai_ws_validate_id(const char *ws_id)
{
    return yai_ws_id_is_valid(ws_id);
}

bool yai_ws_build_paths(yai_workspace_t *ws, const char *ws_id)
{
    const char *home = yai_get_home();
    time_t now = time(NULL);
    if (!ws || !home || !yai_ws_validate_id(ws_id))
        return false;

    memset(ws, 0, sizeof(*ws));

    strncpy(ws->ws_id, ws_id, MAX_WS_ID_LEN - 1);

    snprintf(ws->run_dir, MAX_PATH_LEN,
             "%s/.yai/run/%s", home, ws_id);

    snprintf(ws->root_path, MAX_PATH_LEN,
             "%s/.yai/workspaces/%s", home, ws_id);

    snprintf(ws->lock_file, MAX_PATH_LEN,
             "%s/lock", ws->run_dir);

    snprintf(ws->pid_file, MAX_PATH_LEN,
             "%s/runtime.pid", ws->run_dir);

    snprintf(ws->ingress_sock, MAX_PATH_LEN,
             "%s/%s", home, YAI_RUNTIME_INGRESS_SOCKET_REL);

    ws->created_at = (long)now;
    ws->updated_at = (long)now;
    ws->state = YAI_WS_ACTIVE;
    return true;
}

/* Session acquire/release lifecycle. */

bool yai_session_acquire(yai_session_t **out, const char *ws_id)
{
    if (!out || !ws_id)
        return false;

    /* Fast path: return an already-active session for the workspace. */

    for (int i = 0; i < MAX_SESSIONS; i++)
    {
        if (g_session_registry[i].owner_pid != 0 &&
            strcmp(g_session_registry[i].ws.ws_id, ws_id) == 0)
        {
            *out = &g_session_registry[i];
            return true;
        }
    }

    /* Allocate a new slot when no active session is found. */

    for (int i = 0; i < MAX_SESSIONS; i++)
    {
        if (g_session_registry[i].owner_pid == 0)
        {
            yai_workspace_t ws;

            if (!yai_ws_build_paths(&ws, ws_id))
                return false;

            ensure_run_tree(yai_get_home());
            mkdir_if_missing(ws.run_dir, 0755);

            int fd = open(ws.lock_file,
                          O_CREAT | O_EXCL | O_RDWR,
                          0600);

            if (fd < 0)
            {
                if (errno == EEXIST)
                {
                    FILE *f = fopen(ws.lock_file, "r");
                    if (!f)
                        return false;

                    pid_t old_pid = 0;
                    fscanf(f, "%d", &old_pid);
                    fclose(f);

                    if (old_pid > 0 && kill(old_pid, 0) == 0)
                        return false;

                    unlink(ws.lock_file);

                    fd = open(ws.lock_file,
                              O_CREAT | O_EXCL | O_RDWR,
                              0600);

                    if (fd < 0)
                        return false;
                }
                else
                    return false;
            }

            dprintf(fd, "%d\n", getpid());
            close(fd);

            g_session_registry[i].ws = ws;
            g_session_registry[i].owner_pid = (uint32_t)getpid();
            g_session_registry[i].session_id = (uint32_t)i;

            *out = &g_session_registry[i];
            return true;
        }
    }

    return false;
}

void yai_session_release(yai_session_t *s)
{
    if (!s)
        return;

    unlink(s->ws.lock_file);
    memset(s, 0, sizeof(*s));
}

static int yai_session_build_workspace_enriched_payload(const char *payload,
                                                        const yai_workspace_runtime_info_t *ws_info,
                                                        char *out,
                                                        size_t out_cap)
{
    size_t plen;
    size_t cut;
    int n;
    int has_family;
    int has_spec;
    int has_source;

    if (!payload || !ws_info || !out || out_cap == 0)
        return -1;
    plen = strlen(payload);
    if (plen < 2 || payload[plen - 1] != '}')
        return -1;

    has_family = strstr(payload, "\"workspace_declared_family\"") != NULL;
    has_spec = strstr(payload, "\"workspace_declared_specialization\"") != NULL;
    has_source = strstr(payload, "\"workspace_context_source\"") != NULL;
    if ((has_family || ws_info->declared_control_family[0] == '\0') &&
        (has_spec || ws_info->declared_specialization[0] == '\0') &&
        has_source)
    {
        n = snprintf(out, out_cap, "%s", payload);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    cut = plen - 1; /* skip final } and append normalized workspace hint fields */
    n = snprintf(out, out_cap, "%.*s", (int)cut, payload);
    if (n <= 0 || (size_t)n >= out_cap)
        return -1;

    if (!has_family && ws_info->declared_control_family[0] != '\0')
    {
        n += snprintf(out + n, out_cap - (size_t)n, ",\"workspace_declared_family\":\"%s\"", ws_info->declared_control_family);
        if ((size_t)n >= out_cap)
            return -1;
    }
    if (!has_spec && ws_info->declared_specialization[0] != '\0')
    {
        n += snprintf(out + n, out_cap - (size_t)n, ",\"workspace_declared_specialization\":\"%s\"", ws_info->declared_specialization);
        if ((size_t)n >= out_cap)
            return -1;
    }
    if (!has_source)
    {
        const char *src = ws_info->declared_context_source[0] ? ws_info->declared_context_source : "unset";
        n += snprintf(out + n, out_cap - (size_t)n, ",\"workspace_context_source\":\"%s\"", src);
        if ((size_t)n >= out_cap)
            return -1;
    }
    n += snprintf(out + n, out_cap - (size_t)n, "}");
    return ((n > 0) && (size_t)n < out_cap) ? 0 : -1;
}

/* Session dispatch entrypoint. */

void yai_session_dispatch(
    int client_fd,
    const yai_rpc_envelope_t *env,
    const char *payload)
{
    (void)payload;

    if (!env)
        return;

    if (env->ws_id[0] == '\0' || strlen(env->ws_id) == 0) {
        if (env->command_id == YAI_CMD_CONTROL_CALL) {
            yai_session_send_exec_reply(
                client_fd,
                env,
                "error",
                "BAD_ARGS",
                "ws_required",
                "yai.runtime.unknown",
                "runtime",
                NULL);
        } else {
            yai_session_send_binary_response(
                client_fd,
                env,
                env->command_id,
                "{\"status\":\"error\",\"reason\":\"ws_required\"}");
        }
        return;
    }

    yai_session_t *s = NULL;

    if (!yai_session_acquire(&s, env->ws_id))
    {
        if (env->command_id == YAI_CMD_CONTROL_CALL) {
            yai_session_send_exec_reply(
                client_fd,
                env,
                "error",
                "RUNTIME_NOT_READY",
                "session_denied",
                "yai.runtime.unknown",
                "runtime",
                NULL);
        } else {
            yai_session_send_binary_response(
                client_fd,
                env,
                env->command_id,
                "{\"status\":\"error\",\"reason\":\"session_denied\"}");
        }
        return;
    }

    switch (env->command_id)
    {
    case YAI_CMD_PING:
        yai_session_send_binary_response(
            client_fd,
            env,
            YAI_CMD_PING,
            "{\"status\":\"pong\"}");
        goto cleanup;

    case YAI_CMD_CONTROL_CALL:
        (void)yai_session_handle_control_call(client_fd, env, payload, s);
        goto cleanup;

    default:
        yai_session_send_binary_response(
            client_fd,
            env,
            env->command_id,
            "{\"status\":\"nyi\",\"code\":\"NOT_IMPLEMENTED\",\"reason\":\"nyi_deterministic\"}");
        goto cleanup;
    }

cleanup:
    yai_session_release(s);
}

int yai_session_handle_control_call(
    int client_fd,
    const yai_rpc_envelope_t *env,
    const char *payload,
    const yai_session_t *s)
{
    char command_id[128];
    char action_ws_id[MAX_WS_ID_LEN];
    char action_arg[128];
    char root_path[MAX_PATH_LEN];
    char domain_family[96];
    char domain_specialization[96];
    yai_workspace_runtime_info_t ws_info;
    char binding_status[24];
    char binding_err[96];
    char prompt_json[1024];
    char law_payload[YAI_MAX_PAYLOAD + 512];
    yai_law_resolution_output_t law_out;
    char data[2048];
    char err[256];
    const char *status = "ok";
    const char *code = "OK";
    const char *reason = "accepted";
    const char *effect_name = "unknown";
    const char *runtime_ws_id = NULL;
    char runtime_ws_id_buf[MAX_WS_ID_LEN];
    int workspace_run_macro = 0;

    if (!env || !s)
        return -1;

    if (!payload || !payload[0])
    {
        yai_session_send_exec_reply(
            client_fd,
            env,
            "error",
            "BAD_ARGS",
            "payload_required",
            "yai.runtime.control_call",
            "runtime",
            NULL);
        return -1;
    }

    command_id[0] = '\0';
    action_ws_id[0] = '\0';
    action_arg[0] = '\0';
    root_path[0] = '\0';
    domain_family[0] = '\0';
    domain_specialization[0] = '\0';

    (void)yai_session_extract_json_string(payload, "command_id", command_id, sizeof(command_id));
    (void)yai_session_extract_json_string(payload, "workspace_id", action_ws_id, sizeof(action_ws_id));
    (void)yai_session_extract_json_string(payload, "family", domain_family, sizeof(domain_family));
    (void)yai_session_extract_json_string(payload, "specialization", domain_specialization, sizeof(domain_specialization));
    (void)yai_session_extract_argv_first(payload, action_arg, sizeof(action_arg));
    (void)yai_session_extract_argv_flag_value(payload, "--root", "--path", root_path, sizeof(root_path));
    if (domain_family[0] == '\0')
        (void)yai_session_extract_argv_flag_value(payload, "--family", "-f", domain_family, sizeof(domain_family));
    if (domain_specialization[0] == '\0')
        (void)yai_session_extract_argv_flag_value(payload, "--specialization", "-s", domain_specialization, sizeof(domain_specialization));

    if (command_id[0] == '\0')
    {
        if (strstr(payload, "yai.workspace.create")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.create");
        else if (strstr(payload, "yai.workspace.reset")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.reset");
        else if (strstr(payload, "yai.workspace.destroy")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.destroy");
        else if (strstr(payload, "yai.workspace.activate")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.activate");
        else if (strstr(payload, "yai.workspace.current")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.current");
        else if (strstr(payload, "yai.workspace.clear")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.clear");
        else if (strstr(payload, "yai.workspace.deactivate")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.deactivate");
        else if (strstr(payload, "yai.workspace.status")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.status");
        else if (strstr(payload, "yai.workspace.inspect")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.inspect");
        else if (strstr(payload, "yai.workspace.domain_get")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.domain_get");
        else if (strstr(payload, "yai.workspace.domain.get")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.domain.get");
        else if (strstr(payload, "yai.workspace.domain_set")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.domain_set");
        else if (strstr(payload, "yai.workspace.domain.set")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.domain.set");
        else if (strstr(payload, "yai.workspace.policy_effective")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.policy_effective");
        else if (strstr(payload, "yai.workspace.policy.effective")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.policy.effective");
        else if (strstr(payload, "yai.workspace.debug_resolution")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.debug_resolution");
        else if (strstr(payload, "yai.workspace.debug.resolution")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.debug.resolution");
        else if (strstr(payload, "yai.workspace.prompt_context")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.prompt_context");
        else if (strstr(payload, "yai.workspace.run")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.run");
    }

    if (command_id[0] != '\0' && strncmp(command_id, "yai.workspace.", 14) == 0)
    {
        const char *target_ws = action_ws_id[0] ? action_ws_id : (action_arg[0] ? action_arg : env->ws_id);

        if (strcmp(command_id, "yai.workspace.create") == 0 ||
            strcmp(command_id, "yai.workspace.reset") == 0 ||
            strcmp(command_id, "yai.workspace.destroy") == 0)
        {
            const char *action = "create";
            if (strcmp(command_id, "yai.workspace.reset") == 0) action = "reset";
            if (strcmp(command_id, "yai.workspace.destroy") == 0) action = "destroy";

            if (yai_session_handle_workspace_action(target_ws,
                                                    action,
                                                    root_path[0] ? root_path : NULL,
                                                    &ws_info) != 0)
            {
                yai_session_send_exec_reply(client_fd,
                                            env,
                                            "error",
                                            "BAD_ARGS",
                                            "workspace_action_failed",
                                            command_id,
                                            "runtime",
                                            NULL);
                return -1;
            }

            if (snprintf(data,
                         sizeof(data),
                         "{"
                         "\"ws_id\":\"%s\","
                         "\"workspace_alias\":\"%s\","
                         "\"state\":\"%s\","
                         "\"root_path\":\"%s\","
                         "\"workspace_store_root\":\"%s\","
                         "\"runtime_state_root\":\"%s\","
                         "\"metadata_root\":\"%s\","
                         "\"root_anchor_mode\":\"%s\""
                         "}",
                         ws_info.ws_id,
                         ws_info.workspace_alias,
                         ws_info.state,
                         ws_info.root_path,
                         ws_info.workspace_store_root,
                         ws_info.runtime_state_root,
                         ws_info.metadata_root,
                         ws_info.root_anchor_mode[0] ? ws_info.root_anchor_mode : "managed_default_root") <= 0)
            {
                yai_session_send_exec_reply(client_fd,
                                            env,
                                            "error",
                                            "INTERNAL_ERROR",
                                            "response_encode_failed",
                                            command_id,
                                            "runtime",
                                            NULL);
                return -1;
            }

            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_action_applied", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.activate") == 0)
        {
            if (yai_session_set_active_workspace(target_ws, err, sizeof(err)) != 0)
            {
                yai_session_send_exec_reply(client_fd,
                                            env,
                                            "error",
                                            "BAD_ARGS",
                                            err[0] ? err : "activate_failed",
                                            command_id,
                                            "runtime",
                                            NULL);
                return -1;
            }
            if (yai_session_read_workspace_info(target_ws, &ws_info) == 0 && ws_info.exists)
            {
                if (snprintf(data,
                             sizeof(data),
                             "{"
                             "\"binding_status\":\"active\","
                             "\"workspace_id\":\"%s\","
                             "\"workspace_alias\":\"%s\","
                             "\"root_path\":\"%s\","
                             "\"root_anchor_mode\":\"%s\","
                             "\"binding_scope\":\"session\""
                             "}",
                             target_ws,
                             ws_info.workspace_alias[0] ? ws_info.workspace_alias : target_ws,
                             ws_info.root_path,
                             ws_info.root_anchor_mode[0] ? ws_info.root_anchor_mode : "managed_default_root") <= 0)
                {
                    yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "response_encode_failed", command_id, "runtime", NULL);
                    return -1;
                }
            }
            else
            {
            if (snprintf(data,
                         sizeof(data),
                         "{\"binding_status\":\"active\",\"workspace_id\":\"%s\",\"binding_scope\":\"session\"}",
                         target_ws) <= 0)
            {
                yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "response_encode_failed", command_id, "runtime", NULL);
                return -1;
            }
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_activated", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.clear") == 0 ||
            strcmp(command_id, "yai.workspace.deactivate") == 0)
        {
            if (yai_session_clear_active_workspace() != 0)
            {
                yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "clear_failed", command_id, "runtime", NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_cleared", command_id, "runtime", "{\"binding_status\":\"no_active\",\"binding_scope\":\"session\"}");
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.current") == 0)
        {
            int cur = yai_session_resolve_current_workspace(&ws_info,
                                                            binding_status,
                                                            sizeof(binding_status),
                                                            binding_err,
                                                            sizeof(binding_err));
            if (cur == 0 && strcmp(binding_status, "active") == 0)
            {
                if (snprintf(data,
                             sizeof(data),
                             "{"
                             "\"binding_status\":\"active\","
                             "\"workspace_id\":\"%s\","
                             "\"workspace_alias\":\"%s\","
                             "\"state\":\"%s\","
                             "\"root_path\":\"%s\","
                             "\"workspace_store_root\":\"%s\","
                             "\"runtime_state_root\":\"%s\","
                             "\"metadata_root\":\"%s\","
                             "\"root_anchor_mode\":\"%s\","
                             "\"shell_path_relation\":\"%s\","
                             "\"session_binding\":\"%s\","
                             "\"runtime_attached\":%s"
                             "}",
                             ws_info.ws_id,
                             ws_info.workspace_alias,
                             ws_info.state,
                             ws_info.root_path,
                             ws_info.workspace_store_root,
                             ws_info.runtime_state_root,
                             ws_info.metadata_root,
                             ws_info.root_anchor_mode[0] ? ws_info.root_anchor_mode : "managed_default_root",
                             ws_info.shell_path_relation[0] ? ws_info.shell_path_relation : "unknown",
                             ws_info.session_binding,
                             ws_info.runtime_attached ? "true" : "false") <= 0)
                {
                    yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "response_encode_failed", command_id, "runtime", NULL);
                    return -1;
                }
                yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_current_resolved", command_id, "runtime", data);
                return 0;
            }

            if (snprintf(data,
                         sizeof(data),
                         "{\"binding_status\":\"%s\",\"reason\":\"%s\"}",
                         binding_status[0] ? binding_status : "invalid",
                         binding_err[0] ? binding_err : "none") <= 0)
            {
                yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "response_encode_failed", command_id, "runtime", NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_current_unbound", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.status") == 0)
        {
            if (yai_session_build_workspace_status_json(data, sizeof(data)) != 0)
            {
                yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "workspace_status_failed", command_id, "runtime", NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_status", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.inspect") == 0)
        {
            if (yai_session_build_workspace_inspect_json(data, sizeof(data)) != 0)
            {
                yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "workspace_inspect_failed", command_id, "runtime", NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_inspect", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.domain.get") == 0 ||
            strcmp(command_id, "yai.workspace.domain_get") == 0)
        {
            if (yai_session_build_workspace_domain_get_json(data, sizeof(data)) != 0)
            {
                yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "workspace_domain_get_failed", command_id, "runtime", NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_domain_get", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.domain.set") == 0 ||
            strcmp(command_id, "yai.workspace.domain_set") == 0)
        {
            if (yai_session_set_workspace_declared_context(domain_family[0] ? domain_family : NULL,
                                                           domain_specialization[0] ? domain_specialization : NULL,
                                                           data,
                                                           sizeof(data),
                                                           err,
                                                           sizeof(err)) != 0)
            {
                yai_session_send_exec_reply(client_fd,
                                            env,
                                            "error",
                                            "BAD_ARGS",
                                            err[0] ? err : "workspace_domain_set_failed",
                                            command_id,
                                            "runtime",
                                            NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_domain_set", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.policy.effective") == 0 ||
            strcmp(command_id, "yai.workspace.policy_effective") == 0)
        {
            if (yai_session_build_workspace_policy_effective_json(data, sizeof(data)) != 0)
            {
                yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "workspace_policy_effective_failed", command_id, "runtime", NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_policy_effective", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.debug.resolution") == 0 ||
            strcmp(command_id, "yai.workspace.debug_resolution") == 0)
        {
            if (yai_session_build_workspace_debug_resolution_json(data, sizeof(data)) != 0)
            {
                yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "workspace_debug_resolution_failed", command_id, "runtime", NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_debug_resolution", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.prompt_context") == 0)
        {
            if (yai_session_build_prompt_context_json(prompt_json, sizeof(prompt_json)) != 0)
            {
                yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "prompt_context_failed", command_id, "runtime", NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_prompt_context", command_id, "runtime", prompt_json);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.run") == 0)
        {
            /* Execution macro: keep workspace-aware command semantics but resolve through runtime law path. */
            workspace_run_macro = 1;
            snprintf(command_id, sizeof(command_id), "%s", "yai.runtime.ping");
        }
    }

    memset(&law_out, 0, sizeof(law_out));
    memset(err, 0, sizeof(err));
    law_payload[0] = '\0';
    runtime_ws_id_buf[0] = '\0';
    snprintf(runtime_ws_id_buf, sizeof(runtime_ws_id_buf), "%s", s->ws.ws_id);
    runtime_ws_id = runtime_ws_id_buf;

    if (workspace_run_macro)
    {
        int cur = yai_session_resolve_current_workspace(&ws_info,
                                                        binding_status,
                                                        sizeof(binding_status),
                                                        binding_err,
                                                        sizeof(binding_err));
        if (cur != 0 || strcmp(binding_status, "active") != 0)
        {
            yai_session_send_exec_reply(client_fd,
                                        env,
                                        "error",
                                        "BAD_ARGS",
                                        "workspace_not_active",
                                        "yai.workspace.run",
                                        "runtime",
                                        NULL);
            return -1;
        }
        snprintf(runtime_ws_id_buf, sizeof(runtime_ws_id_buf), "%s", ws_info.ws_id);
        runtime_ws_id = runtime_ws_id_buf;
    }

    if (yai_session_read_workspace_info(runtime_ws_id, &ws_info) == 0 && ws_info.exists) {
        if (yai_session_build_workspace_enriched_payload(payload, &ws_info, law_payload, sizeof(law_payload)) != 0) {
            (void)snprintf(law_payload, sizeof(law_payload), "%s", payload);
        }
    } else {
        (void)snprintf(law_payload, sizeof(law_payload), "%s", payload);
    }

    if (yai_law_resolve_control_call(runtime_ws_id,
                                     law_payload,
                                     env->trace_id,
                                     &law_out,
                                     err,
                                     sizeof(err)) != 0)
    {
        yai_session_send_exec_reply(
            client_fd,
            env,
            "error",
            "INTERNAL_ERROR",
            (err[0] ? err : "law_resolution_failed"),
            "yai.runtime.control_call",
            "runtime",
            NULL);
        return -1;
    }

    effect_name = yai_law_effect_name(law_out.decision.final_effect);

    (void)yai_session_record_resolution_snapshot(runtime_ws_id, &law_out, err, sizeof(err));

    if (law_out.decision.final_effect == YAI_LAW_EFFECT_DENY ||
        law_out.decision.final_effect == YAI_LAW_EFFECT_QUARANTINE)
    {
        status = "error";
        code = "POLICY_BLOCK";
        reason = effect_name;
    }
    else if (law_out.decision.final_effect == YAI_LAW_EFFECT_REVIEW_REQUIRED)
    {
        status = "ok";
        code = "REVIEW_REQUIRED";
        reason = effect_name;
    }

    if (snprintf(data,
                 sizeof(data),
                 "{"
                 "\"ws_id\":\"%s\","
                 "\"session_id\":%u,"
                 "\"decision\":{"
                   "\"decision_id\":\"%s\","
                   "\"domain_id\":\"%s\","
                   "\"effect\":\"%s\","
                   "\"rationale\":\"%s\""
                 "},"
                 "\"evidence\":{"
                   "\"trace_id\":\"%s\","
                   "\"decision_id\":\"%s\","
                   "\"domain_id\":\"%s\","
                   "\"final_effect\":\"%s\","
                   "\"provider\":\"%s\","
                   "\"resource\":\"%s\","
                   "\"authority_context\":\"%s\""
                 "},"
                 "\"resolution_trace\":%s"
                 "}",
                 runtime_ws_id,
                 s->session_id,
                 law_out.decision.decision_id,
                 law_out.decision.domain_id,
                 effect_name,
                 law_out.decision.rationale,
                 law_out.evidence.trace_id,
                 law_out.evidence.decision_id,
                 law_out.evidence.domain_id,
                 law_out.evidence.final_effect,
                 law_out.evidence.provider,
                 law_out.evidence.resource,
                 law_out.evidence.authority_context,
                 law_out.trace_json[0] ? law_out.trace_json : "{}") <= 0)
    {
        yai_session_send_exec_reply(
            client_fd,
            env,
            "error",
            "INTERNAL_ERROR",
            "response_encode_failed",
            "yai.runtime.control_call",
            "runtime",
            NULL);
        return -1;
    }

    yai_session_send_exec_reply(
        client_fd,
        env,
        status,
        code,
        reason,
        "yai.runtime.control_call",
        "runtime",
        data);
    return 0;
}
