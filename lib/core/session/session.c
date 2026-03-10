#define _POSIX_C_SOURCE 200809L

#include <yai/core/session.h>
#include "yai_session_internal.h"
#include <yai/api/runtime.h>
#include <yai/core/enforcement.h>
#include <yai/core/lifecycle.h>
#include <yai/core/workspace.h>
#include <yai/data/binding.h>
#include <yai/exec/runtime.h>
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
    int has_exec_requested;
    int has_exec_effective;
    int has_exec_degraded;
    int has_exec_backend;

    if (!payload || !ws_info || !out || out_cap == 0)
        return -1;
    plen = strlen(payload);
    if (plen < 2 || payload[plen - 1] != '}')
        return -1;

    has_family = strstr(payload, "\"workspace_declared_family\"") != NULL;
    has_spec = strstr(payload, "\"workspace_declared_specialization\"") != NULL;
    has_source = strstr(payload, "\"workspace_context_source\"") != NULL;
    has_exec_requested = strstr(payload, "\"workspace_execution_mode_requested\"") != NULL;
    has_exec_effective = strstr(payload, "\"workspace_execution_mode_effective\"") != NULL;
    has_exec_degraded = strstr(payload, "\"workspace_execution_mode_degraded\"") != NULL;
    has_exec_backend = strstr(payload, "\"workspace_execution_backend\"") != NULL;
    if ((has_family || ws_info->declared_control_family[0] == '\0') &&
        (has_spec || ws_info->declared_specialization[0] == '\0') &&
        has_source &&
        has_exec_requested &&
        has_exec_effective &&
        has_exec_degraded &&
        has_exec_backend)
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
    if (!has_exec_requested)
    {
        n += snprintf(out + n, out_cap - (size_t)n, ",\"workspace_execution_mode_requested\":\"%s\"",
                      ws_info->execution_mode_requested[0] ? ws_info->execution_mode_requested : "scoped");
        if ((size_t)n >= out_cap)
            return -1;
    }
    if (!has_exec_effective)
    {
        n += snprintf(out + n, out_cap - (size_t)n, ",\"workspace_execution_mode_effective\":\"%s\"",
                      ws_info->execution_mode_effective[0] ? ws_info->execution_mode_effective : "scoped");
        if ((size_t)n >= out_cap)
            return -1;
    }
    if (!has_exec_degraded)
    {
        n += snprintf(out + n, out_cap - (size_t)n, ",\"workspace_execution_mode_degraded\":%s",
                      ws_info->execution_mode_degraded ? "true" : "false");
        if ((size_t)n >= out_cap)
            return -1;
    }
    if (!has_exec_backend)
    {
        n += snprintf(out + n, out_cap - (size_t)n, ",\"workspace_execution_backend\":\"%s\"",
                      ws_info->security_backend_mode[0] ? ws_info->security_backend_mode : "none");
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
            "{\"status\":\"error\",\"code\":\"UNSUPPORTED_COMMAND\",\"reason\":\"unsupported_command_id\"}");
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
    char security_level[32];
    char domain_family[96];
    char domain_specialization[96];
    yai_workspace_runtime_info_t ws_info;
    char binding_status[24];
    char binding_err[96];
    char prompt_json[1024];
    char law_payload[YAI_MAX_PAYLOAD + 512];
    yai_law_resolution_output_t law_out;
    char data[YAI_MAX_PAYLOAD + 2048];
    char err[256];
    const char *status = "ok";
    const char *code = "OK";
    const char *reason = "accepted";
    const char *effect_name = "unknown";
    char sci_parameter[128];
    char sci_repro[128];
    char sci_dataset[128];
    char sci_publication[128];
    char dig_outbound[128];
    char dig_sink[128];
    char dig_publication[128];
    char dig_retrieval[128];
    char dig_distribution[128];
    char evt_declared[96];
    char evt_business[96];
    char evt_enforcement[96];
    char evt_stage[48];
    char evt_id[224];
    char op_summary[192];
    const char *review_state = "unresolved";
    yai_enforcement_decision_t enforcement_decision;
    int evt_external = 0;
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
    security_level[0] = '\0';
    domain_family[0] = '\0';
    domain_specialization[0] = '\0';

    (void)yai_session_extract_json_string(payload, "command_id", command_id, sizeof(command_id));
    (void)yai_session_extract_json_string(payload, "workspace_id", action_ws_id, sizeof(action_ws_id));
    (void)yai_session_extract_json_string(payload, "family", domain_family, sizeof(domain_family));
    (void)yai_session_extract_json_string(payload, "specialization", domain_specialization, sizeof(domain_specialization));
    (void)yai_session_extract_argv_first(payload, action_arg, sizeof(action_arg));
    (void)yai_session_extract_argv_flag_value(payload, "--root", "--path", root_path, sizeof(root_path));
    (void)yai_session_extract_json_string(payload, "security_level", security_level, sizeof(security_level));
    if (security_level[0] == '\0')
        (void)yai_session_extract_argv_flag_value(payload, "--containment-level", "--security-level", security_level, sizeof(security_level));
    if (domain_family[0] == '\0')
        (void)yai_session_extract_argv_flag_value(payload, "--family", "-f", domain_family, sizeof(domain_family));
    if (domain_specialization[0] == '\0')
        (void)yai_session_extract_argv_flag_value(payload, "--specialization", "-s", domain_specialization, sizeof(domain_specialization));

    if (command_id[0] == '\0')
    {
        if (strstr(payload, "yai.workspace.create")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.create");
        else if (strstr(payload, "yai.workspace.reset")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.reset");
        else if (strstr(payload, "yai.workspace.destroy")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.destroy");
        else if (strstr(payload, "yai.workspace.set")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.set");
        else if (strstr(payload, "yai.workspace.open")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.open");
        else if (strstr(payload, "yai.workspace.switch")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.switch");
        else if (strstr(payload, "yai.workspace.unset")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.unset");
        else if (strstr(payload, "yai.workspace.current")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.current");
        else if (strstr(payload, "yai.workspace.clear")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.clear");
        else if (strstr(payload, "yai.workspace.status")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.status");
        else if (strstr(payload, "yai.workspace.inspect")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.inspect");
        else if (strstr(payload, "yai.workspace.query")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.query");
        else if (strstr(payload, "yai.workspace.governance.list")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.governance.list");
        else if (strstr(payload, "yai.workspace.events.tail")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.events.tail");
        else if (strstr(payload, "yai.workspace.evidence.list")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.evidence.list");
        else if (strstr(payload, "yai.workspace.enforcement.status")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.enforcement.status");
        else if (strstr(payload, "yai.workspace.authority.list")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.authority.list");
        else if (strstr(payload, "yai.workspace.artifacts.list")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.artifacts.list");
        else if (strstr(payload, "yai.workspace.graph.summary")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.graph.summary");
        else if (strstr(payload, "yai.workspace.graph.workspace")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.graph.workspace");
        else if (strstr(payload, "yai.workspace.graph.governance")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.graph.governance");
        else if (strstr(payload, "yai.workspace.graph.decision")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.graph.decision");
        else if (strstr(payload, "yai.workspace.graph.artifact")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.graph.artifact");
        else if (strstr(payload, "yai.workspace.graph.authority")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.graph.authority");
        else if (strstr(payload, "yai.workspace.graph.evidence")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.graph.evidence");
        else if (strstr(payload, "yai.workspace.graph.lineage")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.graph.lineage");
        else if (strstr(payload, "yai.workspace.graph.recent")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.graph.recent");
        else if (strstr(payload, "yai.workspace.lifecycle.model")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.lifecycle.model");
        else if (strstr(payload, "yai.workspace.lifecycle.maintain")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.lifecycle.maintain");
        else if (strstr(payload, "yai.workspace.lifecycle.status")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.lifecycle.status");
        else if (strstr(payload, "yai.workspace.domain_get")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.domain_get");
        else if (strstr(payload, "yai.workspace.domain.get")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.domain.get");
        else if (strstr(payload, "yai.workspace.domain_set")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.domain_set");
        else if (strstr(payload, "yai.workspace.domain.set")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.domain.set");
        else if (strstr(payload, "yai.workspace.policy_effective")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.policy_effective");
        else if (strstr(payload, "yai.workspace.policy.effective")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.policy.effective");
        else if (strstr(payload, "yai.workspace.policy_attach")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.policy_attach");
        else if (strstr(payload, "yai.workspace.policy.attach")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.policy.attach");
        else if (strstr(payload, "yai.workspace.policy_detach")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.policy_detach");
        else if (strstr(payload, "yai.workspace.policy.detach")) snprintf(command_id, sizeof(command_id), "%s", "yai.workspace.policy.detach");
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
                                                    security_level[0] ? security_level : NULL,
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
                         "\"root_anchor_mode\":\"%s\","
                         "\"execution_mode_requested\":\"%s\","
                         "\"execution_mode_effective\":\"%s\","
                         "\"execution_mode_degraded\":%s"
                         "}",
                         ws_info.ws_id,
                         ws_info.workspace_alias,
                         ws_info.state,
                         ws_info.root_path,
                         ws_info.workspace_store_root,
                         ws_info.runtime_state_root,
                         ws_info.metadata_root,
                         ws_info.root_anchor_mode[0] ? ws_info.root_anchor_mode : "managed_default_root",
                         ws_info.execution_mode_requested[0] ? ws_info.execution_mode_requested : "scoped",
                         ws_info.execution_mode_effective[0] ? ws_info.execution_mode_effective : "scoped",
                         ws_info.execution_mode_degraded ? "true" : "false") <= 0)
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

        if (strcmp(command_id, "yai.workspace.set") == 0 ||
            strcmp(command_id, "yai.workspace.open") == 0 ||
            strcmp(command_id, "yai.workspace.switch") == 0)
        {
            if (yai_session_set_active_workspace(target_ws, err, sizeof(err)) != 0)
            {
                yai_session_send_exec_reply(client_fd,
                                            env,
                                            "error",
                                            "BAD_ARGS",
                                            err[0] ? err : "set_failed",
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
                             "\"execution_mode_requested\":\"%s\","
                             "\"execution_mode_effective\":\"%s\","
                             "\"execution_mode_degraded\":%s,"
                             "\"attach_descriptor_ref\":\"%s\","
                             "\"binding_scope\":\"session\""
                             "}",
                             target_ws,
                             ws_info.workspace_alias[0] ? ws_info.workspace_alias : target_ws,
                             ws_info.root_path,
                             ws_info.root_anchor_mode[0] ? ws_info.root_anchor_mode : "managed_default_root",
                             ws_info.execution_mode_requested[0] ? ws_info.execution_mode_requested : "scoped",
                             ws_info.execution_mode_effective[0] ? ws_info.execution_mode_effective : "scoped",
                             ws_info.execution_mode_degraded ? "true" : "false",
                             ws_info.attach_descriptor_ref) <= 0)
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
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_bound", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.unset") == 0)
        {
            if (yai_session_clear_active_workspace() != 0)
            {
                yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "clear_failed", command_id, "runtime", NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_unset", command_id, "runtime", "{\"binding_status\":\"no_active\",\"binding_scope\":\"session\"}");
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.clear") == 0)
        {
            char cleared_ws[MAX_WS_ID_LEN];
            if (yai_session_clear_workspace_runtime_state(cleared_ws, sizeof(cleared_ws)) != 0)
            {
                yai_session_send_exec_reply(client_fd, env, "error", "BAD_ARGS", "workspace_not_active", command_id, "runtime", NULL);
                return -1;
            }
            if (snprintf(data,
                         sizeof(data),
                         "{\"workspace_id\":\"%s\",\"runtime_state\":\"cleared\",\"binding_status\":\"active\"}",
                         cleared_ws[0] ? cleared_ws : target_ws) <= 0)
            {
                yai_session_send_exec_reply(client_fd, env, "error", "INTERNAL_ERROR", "response_encode_failed", command_id, "runtime", NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_runtime_state_cleared", command_id, "runtime", data);
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
                const yai_runtime_capability_state_t *caps = yai_runtime_capabilities_state();
                int data_ready = yai_data_store_binding_is_ready() != 0;
                int knowledge_ready = (caps && caps->providers_ready && caps->memory_ready && caps->cognition_ready) ? 1 : 0;
                int workspace_bound = (caps && caps->workspace_id[0] &&
                                       strcmp(caps->workspace_id, ws_info.ws_id) == 0 &&
                                       ws_info.runtime_attached) ? 1 : 0;
                int graph_ready = (yai_runtime_capabilities_is_ready() && data_ready && workspace_bound) ? 1 : 0;
                int exec_probe = yai_exec_runtime_probe();
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
                             "\"runtime_attached\":%s,"
                             "\"runtime_capabilities\":{"
                             "\"runtime\":{\"ready\":%s,\"name\":\"%s\"},"
                             "\"workspace_binding\":{\"selected\":true,\"bound\":%s,\"workspace_id\":\"%s\"},"
                             "\"data\":{\"store_binding_ready\":%s,\"root\":\"%s\"},"
                             "\"graph\":{\"ready\":%s,\"truth_source\":\"persistent\"},"
                             "\"knowledge\":{\"ready\":%s,\"transient_authoritative\":false},"
                             "\"exec\":{\"state\":\"%s\",\"ready\":%s},"
                             "\"recovery\":{\"tracked\":%s,\"state\":\"%s\"}"
                             "},"
                             "\"execution_mode_requested\":\"%s\","
                             "\"execution_mode_effective\":\"%s\","
                             "\"execution_mode_degraded\":%s"
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
                             ws_info.runtime_attached ? "true" : "false",
                             yai_runtime_capabilities_is_ready() ? "true" : "false",
                             (caps && caps->runtime_name[0]) ? caps->runtime_name : "yai-runtime",
                             workspace_bound ? "true" : "false",
                             ws_info.ws_id,
                             data_ready ? "true" : "false",
                             yai_data_store_binding_root() ? yai_data_store_binding_root() : "",
                             graph_ready ? "true" : "false",
                             knowledge_ready ? "true" : "false",
                             yai_exec_runtime_state_name((yai_exec_runtime_state_t)exec_probe),
                             (exec_probe == (int)YAI_EXEC_READY) ? "true" : "false",
                             ws_info.declared_context_source[0] ? "true" : "false",
                             (!ws_info.declared_context_source[0]) ? "unknown" :
                             (strcmp(ws_info.declared_context_source, "restored") == 0 ? "restored" : "fresh"),
                             ws_info.execution_mode_requested[0] ? ws_info.execution_mode_requested : "scoped",
                             ws_info.execution_mode_effective[0] ? ws_info.execution_mode_effective : "scoped",
                             ws_info.execution_mode_degraded ? "true" : "false") <= 0)
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

        if (strcmp(command_id, "yai.workspace.query") == 0 ||
            strcmp(command_id, "yai.workspace.governance.list") == 0 ||
            strcmp(command_id, "yai.workspace.events.tail") == 0 ||
            strcmp(command_id, "yai.workspace.evidence.list") == 0 ||
            strcmp(command_id, "yai.workspace.enforcement.status") == 0 ||
            strcmp(command_id, "yai.workspace.authority.list") == 0 ||
            strcmp(command_id, "yai.workspace.artifacts.list") == 0 ||
            strcmp(command_id, "yai.workspace.graph.summary") == 0 ||
            strcmp(command_id, "yai.workspace.graph.workspace") == 0 ||
            strcmp(command_id, "yai.workspace.graph.governance") == 0 ||
            strcmp(command_id, "yai.workspace.graph.decision") == 0 ||
            strcmp(command_id, "yai.workspace.graph.artifact") == 0 ||
            strcmp(command_id, "yai.workspace.graph.authority") == 0 ||
            strcmp(command_id, "yai.workspace.graph.evidence") == 0 ||
            strcmp(command_id, "yai.workspace.graph.lineage") == 0 ||
            strcmp(command_id, "yai.workspace.graph.recent") == 0 ||
            strcmp(command_id, "yai.workspace.lifecycle.model") == 0)
        {
            const char *query_family = "workspace";
            if (strcmp(command_id, "yai.workspace.governance.list") == 0) query_family = "governance";
            else if (strcmp(command_id, "yai.workspace.events.tail") == 0) query_family = "events";
            else if (strcmp(command_id, "yai.workspace.evidence.list") == 0) query_family = "evidence";
            else if (strcmp(command_id, "yai.workspace.enforcement.status") == 0) query_family = "enforcement";
            else if (strcmp(command_id, "yai.workspace.authority.list") == 0) query_family = "authority";
            else if (strcmp(command_id, "yai.workspace.artifacts.list") == 0) query_family = "artifacts";
            else if (strcmp(command_id, "yai.workspace.graph.summary") == 0) query_family = "graph";
            else if (strcmp(command_id, "yai.workspace.graph.workspace") == 0) query_family = "graph.workspace";
            else if (strcmp(command_id, "yai.workspace.graph.governance") == 0) query_family = "graph.governance";
            else if (strcmp(command_id, "yai.workspace.graph.decision") == 0) query_family = "graph.decision";
            else if (strcmp(command_id, "yai.workspace.graph.artifact") == 0) query_family = "graph.artifact";
            else if (strcmp(command_id, "yai.workspace.graph.authority") == 0) query_family = "graph.authority";
            else if (strcmp(command_id, "yai.workspace.graph.evidence") == 0) query_family = "graph.evidence";
            else if (strcmp(command_id, "yai.workspace.graph.lineage") == 0) query_family = "graph.lineage";
            else if (strcmp(command_id, "yai.workspace.graph.recent") == 0) query_family = "graph.recent";
            else if (strcmp(command_id, "yai.workspace.lifecycle.model") == 0) query_family = "lifecycle";
            else if (action_arg[0]) query_family = action_arg;

            if (yai_session_build_workspace_data_query_json(query_family,
                                                            data,
                                                            sizeof(data),
                                                            err,
                                                            sizeof(err)) != 0)
            {
                yai_session_send_exec_reply(client_fd,
                                            env,
                                            "error",
                                            "BAD_ARGS",
                                            err[0] ? err : "workspace_query_failed",
                                            command_id,
                                            "runtime",
                                            NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_query_result", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.lifecycle.maintain") == 0)
        {
            if (yai_session_run_workspace_lifecycle_maintenance_json(data, sizeof(data), err, sizeof(err)) != 0)
            {
                yai_session_send_exec_reply(client_fd,
                                            env,
                                            "error",
                                            "INTERNAL_ERROR",
                                            err[0] ? err : "workspace_lifecycle_maintenance_failed",
                                            command_id,
                                            "runtime",
                                            NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_lifecycle_maintenance_applied", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.lifecycle.status") == 0)
        {
            if (yai_session_build_workspace_lifecycle_status_json(data, sizeof(data), err, sizeof(err)) != 0)
            {
                yai_session_send_exec_reply(client_fd,
                                            env,
                                            "error",
                                            "INTERNAL_ERROR",
                                            err[0] ? err : "workspace_lifecycle_status_failed",
                                            command_id,
                                            "runtime",
                                            NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_lifecycle_status", command_id, "runtime", data);
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

        if (strcmp(command_id, "yai.workspace.policy.attach") == 0 ||
            strcmp(command_id, "yai.workspace.policy_attach") == 0)
        {
            if (yai_session_workspace_policy_attachment_update(action_arg[0] ? action_arg : NULL,
                                                               1,
                                                               data,
                                                               sizeof(data),
                                                               err,
                                                               sizeof(err)) != 0)
            {
                yai_session_send_exec_reply(client_fd,
                                            env,
                                            "error",
                                            "BAD_ARGS",
                                            err[0] ? err : "workspace_policy_attach_failed",
                                            command_id,
                                            "runtime",
                                            NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_policy_attached", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.policy.activate") == 0 ||
            strcmp(command_id, "yai.workspace.policy_activate") == 0)
        {
            if (yai_session_workspace_policy_attachment_update(action_arg[0] ? action_arg : NULL,
                                                               2,
                                                               data,
                                                               sizeof(data),
                                                               err,
                                                               sizeof(err)) != 0)
            {
                yai_session_send_exec_reply(client_fd,
                                            env,
                                            "error",
                                            "BAD_ARGS",
                                            err[0] ? err : "workspace_policy_activate_failed",
                                            command_id,
                                            "runtime",
                                            NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_policy_activated", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.policy.detach") == 0 ||
            strcmp(command_id, "yai.workspace.policy_detach") == 0)
        {
            if (yai_session_workspace_policy_attachment_update(action_arg[0] ? action_arg : NULL,
                                                               0,
                                                               data,
                                                               sizeof(data),
                                                               err,
                                                               sizeof(err)) != 0)
            {
                yai_session_send_exec_reply(client_fd,
                                            env,
                                            "error",
                                            "BAD_ARGS",
                                            err[0] ? err : "workspace_policy_detach_failed",
                                            command_id,
                                            "runtime",
                                            NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_policy_detached", command_id, "runtime", data);
            return 0;
        }

        if (strcmp(command_id, "yai.workspace.policy.dry_run") == 0 ||
            strcmp(command_id, "yai.workspace.policy_dry_run") == 0)
        {
            if (yai_session_workspace_policy_apply_dry_run(action_arg[0] ? action_arg : NULL,
                                                           data,
                                                           sizeof(data),
                                                           err,
                                                           sizeof(err)) != 0)
            {
                yai_session_send_exec_reply(client_fd,
                                            env,
                                            "error",
                                            "BAD_ARGS",
                                            err[0] ? err : "workspace_policy_dry_run_failed",
                                            command_id,
                                            "runtime",
                                            NULL);
                return -1;
            }
            yai_session_send_exec_reply(client_fd, env, "ok", "OK", "workspace_policy_dry_run", command_id, "runtime", data);
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
    else
    {
        if (yai_session_enforce_workspace_scope(runtime_ws_id, err, sizeof(err)) != 0)
        {
            yai_session_send_exec_reply(client_fd,
                                        env,
                                        "error",
                                        "BAD_ARGS",
                                        err[0] ? err : "workspace_scope_denied",
                                        "yai.runtime.control_call",
                                        "runtime",
                                        NULL);
            return -1;
        }
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
    if (yai_enforcement_finalize_control_call(env,
                                              runtime_ws_id,
                                              &law_out,
                                              yai_runtime_capabilities_state(),
                                              &enforcement_decision,
                                              err,
                                              sizeof(err)) != 0)
    {
        yai_session_send_exec_reply(
            client_fd,
            env,
            "error",
            "INTERNAL_ERROR",
            (err[0] ? err : "enforcement_finalize_failed"),
            "yai.runtime.control_call",
            "runtime",
            NULL);
        return -1;
    }
    status = enforcement_decision.status;
    code = enforcement_decision.code;
    reason = enforcement_decision.reason;
    review_state = enforcement_decision.review_state;

    (void)yai_session_record_resolution_snapshot(runtime_ws_id, &law_out, &enforcement_decision, err, sizeof(err));
    snprintf(ws_info.inferred_family, sizeof(ws_info.inferred_family), "%s", law_out.decision.family_id);
    snprintf(ws_info.inferred_specialization, sizeof(ws_info.inferred_specialization), "%s", law_out.decision.specialization_id);
    snprintf(ws_info.last_effect_summary, sizeof(ws_info.last_effect_summary), "%s", effect_name);
    snprintf(ws_info.last_resolution_trace_ref, sizeof(ws_info.last_resolution_trace_ref), "%s", law_out.evidence.trace_id);
    yai_session_workspace_event_semantics(&ws_info,
                                          evt_declared,
                                          sizeof(evt_declared),
                                          evt_business,
                                          sizeof(evt_business),
                                          evt_enforcement,
                                          sizeof(evt_enforcement),
                                          evt_stage,
                                          sizeof(evt_stage),
                                          &evt_external);
    snprintf(evt_id, sizeof(evt_id), "%s%s",
             law_out.evidence.trace_id[0] ? "evt-" : "none",
             law_out.evidence.trace_id[0] ? law_out.evidence.trace_id : "");
    (void)snprintf(op_summary,
                   sizeof(op_summary),
                   "%s/%s => %s",
                   evt_stage[0] ? evt_stage : "unknown",
                   evt_business[0] ? evt_business : "not_resolved",
                   effect_name);
    sci_parameter[0] = '\0';
    sci_repro[0] = '\0';
    sci_dataset[0] = '\0';
    sci_publication[0] = '\0';
    dig_outbound[0] = '\0';
    dig_sink[0] = '\0';
    dig_publication[0] = '\0';
    dig_retrieval[0] = '\0';
    dig_distribution[0] = '\0';
    if (strcmp(law_out.decision.family_id, "scientific") == 0)
    {
        snprintf(sci_parameter,
                 sizeof(sci_parameter),
                 "%s",
                 (strstr(law_out.decision.stack.evidence_profile, "parameter_lock_required") ||
                  strstr(law_out.decision.stack.evidence_profile, "parameter_diff_trace_required"))
                     ? "parameter lock and diff trace required"
                     : "parameter governance active");
        snprintf(sci_repro,
                 sizeof(sci_repro),
                 "%s",
                 strstr(law_out.decision.stack.evidence_profile, "reproducibility_proofpack_required")
                     ? "reproducibility proofpack required"
                     : "reproducibility checks partial");
        snprintf(sci_dataset,
                 sizeof(sci_dataset),
                 "%s",
                 strstr(law_out.decision.stack.evidence_profile, "dataset_integrity_attestation_required")
                     ? "dataset integrity attestation required"
                     : "dataset integrity not primary");
        snprintf(sci_publication,
                 sizeof(sci_publication),
                 "%s",
                 strcmp(law_out.decision.specialization_id, "result-publication-control") == 0
                     ? (law_out.decision.final_effect == YAI_LAW_EFFECT_DENY
                            ? "publication blocked pending authority/repro checks"
                            : law_out.decision.final_effect == YAI_LAW_EFFECT_QUARANTINE
                                  ? "publication quarantined pending review"
                                  : "publication control active")
                     : "publication control not primary");
    }
    if (strcmp(law_out.decision.family_id, "digital") == 0)
    {
        snprintf(dig_outbound,
                 sizeof(dig_outbound),
                 "%s",
                 strcmp(law_out.decision.specialization_id, "remote-retrieval") == 0
                     ? "remote retrieval governed path active"
                     : strcmp(law_out.decision.specialization_id, "remote-publication") == 0
                           ? "remote publication governed path active"
                           : strcmp(law_out.decision.specialization_id, "external-commentary") == 0
                                 ? "external commentary governed path active"
                                 : strcmp(law_out.decision.specialization_id, "artifact-distribution") == 0
                                       ? "artifact distribution governed path active"
                                       : strcmp(law_out.decision.specialization_id, "digital-sink-control") == 0
                                             ? "digital sink governance active"
                                             : "digital outbound governance active");
        snprintf(dig_sink,
                 sizeof(dig_sink),
                 "%s",
                 strstr(law_out.decision.stack.evidence_profile, "sink_policy_attestation_required")
                     ? "sink policy attestation required"
                     : "sink target checks not primary");
        snprintf(dig_publication,
                 sizeof(dig_publication),
                 "%s",
                 strcmp(law_out.decision.specialization_id, "remote-publication") == 0
                     ? (law_out.decision.final_effect == YAI_LAW_EFFECT_DENY
                            ? "publication denied pending authority/sink checks"
                            : law_out.decision.final_effect == YAI_LAW_EFFECT_QUARANTINE
                                  ? "publication quarantined pending sink review"
                                  : "publication requires review record")
                     : "publication control not primary");
        snprintf(dig_retrieval,
                 sizeof(dig_retrieval),
                 "%s",
                 strstr(law_out.decision.stack.evidence_profile, "retrieval_source_attestation_required")
                     ? "retrieval source attestation required"
                     : "retrieval control not primary");
        snprintf(dig_distribution,
                 sizeof(dig_distribution),
                 "%s",
                 strstr(law_out.decision.stack.evidence_profile, "distribution_manifest_required")
                     ? "distribution manifest and destination trace required"
                     : "artifact distribution not primary");
    }

    if (snprintf(data,
                 sizeof(data),
                 "{"
                 "\"ws_id\":\"%s\","
                 "\"session_id\":%u,"
                 "\"decision\":{"
                   "\"decision_id\":\"%s\","
                   "\"domain_id\":\"%s\","
                   "\"family_id\":\"%s\","
                   "\"specialization_id\":\"%s\","
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
                 "\"execution\":{"
                   "\"mode_requested\":\"%s\","
                   "\"mode_effective\":\"%s\","
                   "\"degraded\":%s,"
                   "\"degraded_reason\":\"%s\","
                   "\"unsupported_scopes\":\"%s\","
                   "\"attach_descriptor_ref\":\"%s\","
                   "\"execution_profile_ref\":\"%s\""
                 "},"
                 "\"enforcement\":{"
                   "\"authority_decision\":\"%s\","
                   "\"authority_constraints\":\"%s\","
                   "\"authority_constraint_count\":%d,"
                   "\"runtime_bound\":%s"
                 "},"
                 "\"event_surface\":{"
                   "\"event_id\":\"%s\","
                   "\"flow_stage\":\"%s\","
                   "\"declared_scenario_specialization\":\"%s\","
                   "\"business_specialization\":\"%s\","
                   "\"enforcement_specialization\":\"%s\","
                   "\"external_effect_boundary\":%s"
                 "},"
                 "\"operational_state\":{"
                   "\"binding_state\":\"active\","
                   "\"attached_governance_objects\":\"%s\","
                   "\"active_effective_stack\":\"%s\","
                   "\"last_event_ref\":\"%s\","
                   "\"last_flow_stage\":\"%s\","
                   "\"last_business_specialization\":\"%s\","
                   "\"last_enforcement_specialization\":\"%s\","
                   "\"last_effect\":\"%s\","
                   "\"last_authority\":\"%s\","
                   "\"last_evidence\":\"%s\","
                   "\"last_trace_ref\":\"%s\","
                   "\"review_state\":\"%s\","
                   "\"operational_summary\":\"%s\""
                 "},"
                 "\"scientific\":{"
                   "\"parameter_governance_summary\":\"%s\","
                   "\"reproducibility_summary\":\"%s\","
                   "\"dataset_integrity_summary\":\"%s\","
                   "\"publication_control_summary\":\"%s\""
                 "},"
                 "\"digital\":{"
                   "\"outbound_context_summary\":\"%s\","
                   "\"sink_target_summary\":\"%s\","
                   "\"publication_control_summary\":\"%s\","
                   "\"retrieval_control_summary\":\"%s\","
                   "\"distribution_control_summary\":\"%s\""
                 "},"
                 "\"resolution_trace\":%s"
                 "}",
                 runtime_ws_id,
                 s->session_id,
                 law_out.decision.decision_id,
                 law_out.decision.domain_id,
                 law_out.decision.family_id,
                 law_out.decision.specialization_id,
                 effect_name,
                 law_out.decision.rationale,
                 law_out.evidence.trace_id,
                 law_out.evidence.decision_id,
                 law_out.evidence.domain_id,
                 law_out.evidence.final_effect,
                 law_out.evidence.provider,
                 law_out.evidence.resource,
                 law_out.evidence.authority_context,
                 ws_info.execution_mode_requested[0] ? ws_info.execution_mode_requested : "scoped",
                 ws_info.execution_mode_effective[0] ? ws_info.execution_mode_effective : "scoped",
                 ws_info.execution_mode_degraded ? "true" : "false",
                 ws_info.execution_degraded_reason[0] ? ws_info.execution_degraded_reason : "none",
                 ws_info.execution_unsupported_scopes[0] ? ws_info.execution_unsupported_scopes : "none",
                 ws_info.attach_descriptor_ref,
                 ws_info.execution_profile_ref,
                 enforcement_decision.authority_decision == YAI_AUTHORITY_DENY ? "deny" :
                     enforcement_decision.authority_decision == YAI_AUTHORITY_REVIEW_REQUIRED ? "review_required" : "allow",
                 enforcement_decision.authority_constraints,
                 enforcement_decision.authority_constraint_count,
                 enforcement_decision.runtime_bound ? "true" : "false",
                 evt_id,
                 evt_stage,
                 evt_declared,
                 evt_business,
                 evt_enforcement,
                 evt_external ? "true" : "false",
                 ws_info.policy_attachments_csv,
                 law_out.decision.stack.stack_id,
                 evt_id,
                 evt_stage,
                 evt_business,
                 evt_enforcement,
                 effect_name,
                 law_out.decision.stack.authority_profile,
                 law_out.decision.stack.evidence_profile,
                 law_out.evidence.trace_id,
                 review_state,
                 op_summary,
                 sci_parameter[0] ? sci_parameter : "not scientific context",
                 sci_repro[0] ? sci_repro : "not scientific context",
                 sci_dataset[0] ? sci_dataset : "not scientific context",
                 sci_publication[0] ? sci_publication : "not scientific context",
                 dig_outbound[0] ? dig_outbound : "not digital context",
                 dig_sink[0] ? dig_sink : "not digital context",
                 dig_publication[0] ? dig_publication : "not digital context",
                 dig_retrieval[0] ? dig_retrieval : "not digital context",
                 dig_distribution[0] ? dig_distribution : "not digital context",
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
