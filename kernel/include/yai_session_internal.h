#ifndef YAI_SESSION_INTERNAL_H
#define YAI_SESSION_INTERNAL_H

#include "yai_session.h"

int yai_session_extract_json_string(const char *json, const char *key, char *out, size_t out_cap);
int yai_session_extract_argv_first(const char *json, char *out, size_t out_cap);
int yai_session_extract_argv_flag_value(
    const char *json,
    const char *flag_a,
    const char *flag_b,
    char *out,
    size_t out_cap);

int yai_session_path_exists(const char *path);
int yai_session_build_run_path(char *out, size_t out_cap, const char *suffix);
int yai_session_read_manifest_layout(const char *ws_id, char *layout, size_t layout_cap, long *created_at);
int yai_session_build_workspace_list_json(char *out, size_t out_cap, int *count_out);
int yai_session_handle_workspace_action(const char *ws_id, const char *action);

void yai_session_send_binary_response(
    int fd,
    const yai_rpc_envelope_t *req,
    uint32_t command_id,
    const char *json_payload);

void yai_session_send_exec_reply(
    int fd,
    const yai_rpc_envelope_t *req,
    const char *status,
    const char *code,
    const char *reason,
    const char *command_id,
    const char *target_plane,
    const char *data_json);

int yai_session_handle_control_call(
    int client_fd,
    const yai_rpc_envelope_t *env,
    const char *payload,
    const yai_session_t *s);

#endif
