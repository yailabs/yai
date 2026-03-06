#define _POSIX_C_SOURCE 200809L

#include "yai_session_internal.h"
#include "ws_id.h"

#include <stdio.h>
#include <string.h>

int yai_session_handle_control_call(
    int client_fd,
    const yai_rpc_envelope_t *env,
    const char *payload,
    const yai_session_t *s)
{
    char command_id[128] = {0};
    char action[64] = {0};
    char ws_arg[MAX_WS_ID_LEN] = {0};
    char root_arg[MAX_PATH_LEN] = {0};

    if (yai_session_extract_json_string(payload, "command_id", command_id, sizeof(command_id)) != 0)
    {
        yai_session_send_exec_reply(
            client_fd,
            env,
            "error",
            "BAD_ARGS",
            "bad_command_id",
            "yai.kernel.ws",
            "kernel",
            NULL);
        return 0;
    }

    if (strcmp(command_id, "yai.kernel.ws") == 0)
    {
        const char *ws_id = env->ws_id;
        yai_workspace_runtime_info_t info;

        if (yai_session_extract_argv_first(payload, action, sizeof(action)) != 0)
        {
            yai_session_send_exec_reply(client_fd, env, "error", "BAD_ARGS", "bad_args", command_id, "kernel", NULL);
            return 0;
        }

        if (yai_session_extract_argv_flag_value(payload, "--ws-id", "--ws", ws_arg, sizeof(ws_arg)) == 0 && ws_arg[0])
            ws_id = ws_arg;

        if (!yai_ws_id_is_valid(ws_id))
        {
            yai_session_send_exec_reply(client_fd, env, "error", "BAD_ARGS", "bad_ws_id", command_id, "kernel", NULL);
            return 0;
        }

        if (yai_session_extract_argv_flag_value(payload, "--root", NULL, root_arg, sizeof(root_arg)) != 0)
            root_arg[0] = '\0';

        memset(&info, 0, sizeof(info));
        int rc = yai_session_handle_workspace_action(ws_id, action, root_arg[0] ? root_arg : NULL, &info);
        if (rc == 0)
        {
            const char *reason = "workspace_updated";
            char data[1024];
            if (strcmp(action, "create") == 0)
                reason = "workspace_created";
            else if (strcmp(action, "reset") == 0)
                reason = "workspace_reset";
            else if (strcmp(action, "destroy") == 0)
                reason = "workspace_destroyed";

            if (strcmp(action, "destroy") == 0)
            {
                snprintf(data, sizeof(data),
                         "{\"ws_id\":\"%s\",\"exists\":false,\"state\":\"destroyed\",\"updated_at\":%ld}",
                         ws_id, info.updated_at);
            }
            else
            {
                snprintf(data, sizeof(data),
                         "{\"ws_id\":\"%s\",\"exists\":true,\"state\":\"%s\",\"root_path\":\"%s\",\"created_at\":%ld,\"updated_at\":%ld}",
                         info.ws_id[0] ? info.ws_id : ws_id,
                         info.state[0] ? info.state : "active",
                         info.root_path[0] ? info.root_path : "",
                         info.created_at,
                         info.updated_at);
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", reason, command_id, "kernel", data);
            return 0;
        }
        if (rc == -2)
        {
            yai_session_send_exec_reply(client_fd, env, "error", "BAD_ARGS", "unsupported_workspace_action", command_id, "kernel", NULL);
            return 0;
        }
        yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "workspace_action_failed", command_id, "kernel", NULL);
        return 0;
    }

    if (strcmp(command_id, "yai.kernel.ws_status") == 0)
    {
        const char *ws_id = env->ws_id;
        yai_workspace_runtime_info_t info;
        if (yai_session_extract_argv_flag_value(payload, "--ws-id", "--ws", ws_arg, sizeof(ws_arg)) == 0 && ws_arg[0])
            ws_id = ws_arg;
        if (!yai_ws_id_is_valid(ws_id))
        {
            yai_session_send_exec_reply(client_fd, env, "error", "BAD_ARGS", "bad_ws_id", command_id, "kernel", NULL);
            return 0;
        }

        char data[1024];
        memset(&info, 0, sizeof(info));
        if (yai_session_read_workspace_info(ws_id, &info) == 0 && info.exists)
        {
            snprintf(data, sizeof(data),
                     "{\"ws_id\":\"%s\",\"exists\":true,\"state\":\"%s\",\"layout\":\"%s\",\"root_path\":\"%s\",\"created_at\":%ld,\"updated_at\":%ld}",
                     info.ws_id, info.state, info.layout, info.root_path, info.created_at, info.updated_at);
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_status", command_id, "kernel", data);
        }
        else
        {
            snprintf(data, sizeof(data), "{\"ws_id\":\"%s\",\"exists\":false,\"state\":\"missing\"}", ws_id);
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_missing", command_id, "kernel", data);
        }
        return 0;
    }

    if (strcmp(command_id, "yai.kernel.ws_list") == 0)
    {
        char list_json[1024];
        int count = 0;
        if (yai_session_build_workspace_list_json(list_json, sizeof(list_json), &count) != 0)
        {
            yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "workspace_list_failed", command_id, "kernel", NULL);
            return 0;
        }
        char data[1280];
        snprintf(data, sizeof(data), "{\"count\":%d,\"workspaces\":%s}", count, list_json);
        yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_list", command_id, "kernel", data);
        return 0;
    }

    if (strcmp(command_id, "yai.kernel.session_status") == 0)
    {
        char data[384];
        snprintf(data, sizeof(data),
                 "{\"ws_id\":\"%s\",\"owner_pid\":%u,\"session_id\":%u,\"state\":\"active\"}",
                 s->ws.ws_id, s->owner_pid, s->session_id);
        yai_session_send_exec_reply(client_fd, env, "ok", "OK", "session_status", command_id, "kernel", data);
        return 0;
    }

    if (strcmp(command_id, "yai.kernel.boundary_status") == 0)
    {
        const int ws_valid = yai_ws_id_is_valid(env->ws_id) ? 1 : 0;
        char data[256];
        snprintf(data, sizeof(data),
                 "{\"ws_id\":\"%s\",\"enforcement\":\"enabled\",\"ws_valid\":%s}",
                 env->ws_id, ws_valid ? "true" : "false");
        yai_session_send_exec_reply(client_fd, env, "ok", "OK", "boundary_status", command_id, "kernel", data);
        return 0;
    }

    if (strcmp(command_id, "yai.kernel.policy_status") == 0)
    {
        char data[256];
        snprintf(data, sizeof(data),
                 "{\"ws_id\":\"%s\",\"policy_state\":\"loaded\",\"policy_id\":\"kernel_policy_v1\"}",
                 env->ws_id);
        yai_session_send_exec_reply(client_fd, env, "ok", "OK", "policy_status", command_id, "kernel", data);
        return 0;
    }

    if (strcmp(command_id, "yai.root.handshake_status") == 0)
    {
        char root_sock[MAX_PATH_LEN];
        char kernel_sock[MAX_PATH_LEN];
        yai_session_build_run_path(root_sock, sizeof(root_sock), "root/root.sock");
        yai_session_build_run_path(kernel_sock, sizeof(kernel_sock), "kernel/control.sock");
        int ready = yai_session_path_exists(root_sock) && yai_session_path_exists(kernel_sock);
        char data[256];
        snprintf(data, sizeof(data),
                 "{\"ready\":%s,\"root_socket\":%s,\"kernel_socket\":%s}",
                 ready ? "true" : "false",
                 yai_session_path_exists(root_sock) ? "true" : "false",
                 yai_session_path_exists(kernel_sock) ? "true" : "false");
        yai_session_send_exec_reply(client_fd, env, "ok", "OK", ready ? "handshake_ready" : "handshake_not_ready", command_id, "root", data);
        return 0;
    }

    if (strcmp(command_id, "yai.root.router_status") == 0)
    {
        char root_sock[MAX_PATH_LEN];
        yai_session_build_run_path(root_sock, sizeof(root_sock), "root/root.sock");
        int online = yai_session_path_exists(root_sock);
        char data[256];
        snprintf(data, sizeof(data),
                 "{\"online\":%s,\"ws_id\":\"%s\",\"trace_id\":\"%s\"}",
                 online ? "true" : "false",
                 env->ws_id,
                 env->trace_id);
        yai_session_send_exec_reply(client_fd, env, "ok", "OK", online ? "router_online" : "router_offline", command_id, "root", data);
        return 0;
    }

    if (strcmp(command_id, "yai.root.session_status") == 0)
    {
        char data[320];
        snprintf(data, sizeof(data),
                 "{\"ws_id\":\"%s\",\"role\":%u,\"arming\":%u,\"session_id\":%u}",
                 env->ws_id, (unsigned)env->role, (unsigned)env->arming, s->session_id);
        yai_session_send_exec_reply(client_fd, env, "ok", "OK", "root_session_status", command_id, "root", data);
        return 0;
    }

    if (strcmp(command_id, "yai.root.authority_status") == 0)
    {
        int authorized = (env->role == 2u && env->arming == 1u) ? 1 : 0;
        char data[224];
        snprintf(data, sizeof(data),
                 "{\"role\":%u,\"arming\":%u,\"authorized\":%s}",
                 (unsigned)env->role, (unsigned)env->arming, authorized ? "true" : "false");
        yai_session_send_exec_reply(client_fd, env, "ok", "OK", authorized ? "authority_operator_armed" : "authority_limited", command_id, "root", data);
        return 0;
    }

    if (strcmp(command_id, "yai.root.envelope_validate") == 0)
    {
        int ws_valid = yai_ws_id_is_valid(env->ws_id) ? 1 : 0;
        int trace_present = env->trace_id[0] ? 1 : 0;
        int role_valid = (env->role == 0u || env->role == 1u || env->role == 2u || env->role == 3u);
        int arming_valid = (env->arming <= 1u);
        int valid = ws_valid && role_valid && arming_valid;
        char data[320];
        snprintf(data, sizeof(data),
                 "{\"valid\":%s,\"ws_valid\":%s,\"role_valid\":%s,\"arming_valid\":%s,\"trace_present\":%s}",
                 valid ? "true" : "false",
                 ws_valid ? "true" : "false",
                 role_valid ? "true" : "false",
                 arming_valid ? "true" : "false",
                 trace_present ? "true" : "false");
        yai_session_send_exec_reply(client_fd, env, "ok", "OK", valid ? "envelope_valid" : "envelope_invalid", command_id, "root", data);
        return 0;
    }

    if (strcmp(command_id, "yai.boot.status") == 0 ||
        strcmp(command_id, "yai.boot.runtime_status") == 0 ||
        strcmp(command_id, "yai.boot.health_status") == 0 ||
        strcmp(command_id, "yai.boot.service_status") == 0 ||
        strcmp(command_id, "yai.boot.node_status") == 0)
    {
        char root_sock[MAX_PATH_LEN];
        char kernel_sock[MAX_PATH_LEN];
        char engine_sock[MAX_PATH_LEN];
        yai_session_build_run_path(root_sock, sizeof(root_sock), "root/root.sock");
        yai_session_build_run_path(kernel_sock, sizeof(kernel_sock), "kernel/control.sock");
        yai_session_build_run_path(engine_sock, sizeof(engine_sock), "engine/control.sock");
        int root_up = yai_session_path_exists(root_sock);
        int kernel_up = yai_session_path_exists(kernel_sock);
        int engine_up = yai_session_path_exists(engine_sock);
        const char *overall = (root_up && kernel_up) ? "up" : ((root_up || kernel_up) ? "degraded" : "down");

        char data[416];
        snprintf(data, sizeof(data),
                 "{\"overall\":\"%s\",\"root\":%s,\"kernel\":%s,\"engine\":%s,\"ws_id\":\"%s\"}",
                 overall,
                 root_up ? "true" : "false",
                 kernel_up ? "true" : "false",
                 engine_up ? "true" : "false",
                 env->ws_id);
        yai_session_send_exec_reply(client_fd, env, "ok", "OK", overall, command_id, "boot", data);
        return 0;
    }

    yai_session_send_exec_reply(client_fd, env, "nyi", "NOT_IMPLEMENTED", "nyi_deterministic", command_id, "kernel", NULL);
    return 0;
}
