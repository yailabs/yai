#ifndef YAI_SESSION_INTERNAL_H
#define YAI_SESSION_INTERNAL_H

#include <yai/container/session_control.h>
#include <yai/policy/enforcement.h>
#include <yai/policy/governance/resolver.h>

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
int yai_session_read_workspace_info(const char *ws_id, yai_workspace_runtime_info_t *out);
int yai_session_build_workspace_list_json(char *out, size_t out_cap, int *count_out);
int yai_session_handle_workspace_action(
    const char *ws_id,
    const char *action,
    const char *root_path_opt,
    const char *security_level_opt,
    char *err,
    size_t err_cap,
    yai_workspace_runtime_info_t *info_out);
int yai_session_set_active_workspace(const char *ws_id, char *err, size_t err_cap);
int yai_session_clear_active_workspace(void);
int yai_session_clear_workspace_runtime_state(char *out_ws_id, size_t out_ws_id_cap);
int yai_session_resolve_current_workspace(yai_workspace_runtime_info_t *info_out,
                                          char *status_out,
                                          size_t status_cap,
                                          char *err,
                                          size_t err_cap);
int yai_session_build_prompt_context_json(char *out, size_t out_cap);
int yai_session_build_workspace_status_json(char *out, size_t out_cap);
int yai_session_build_workspace_inspect_json(char *out, size_t out_cap);
int yai_session_build_workspace_domain_get_json(char *out, size_t out_cap);
int yai_session_set_workspace_declared_context(const char *family,
                                               const char *specialization,
                                               char *out_json,
                                               size_t out_cap,
                                               char *err,
                                               size_t err_cap);
int yai_session_build_workspace_policy_effective_json(char *out, size_t out_cap);
int yai_session_build_workspace_debug_resolution_json(char *out, size_t out_cap);
int yai_session_build_workspace_data_query_json(const char *query_family,
                                                char *out,
                                                size_t out_cap,
                                                char *err,
                                                size_t err_cap);
int yai_session_run_workspace_lifecycle_maintenance_json(char *out,
                                                         size_t out_cap,
                                                         char *err,
                                                         size_t err_cap);
int yai_session_build_workspace_lifecycle_status_json(char *out,
                                                      size_t out_cap,
                                                      char *err,
                                                      size_t err_cap);
int yai_session_workspace_policy_attachment_update(const char *object_id,
                                                   int attach_mode,
                                                   char *out_json,
                                                   size_t out_cap,
                                                   char *err,
                                                   size_t err_cap);
int yai_session_workspace_policy_apply_dry_run(const char *object_id,
                                               char *out_json,
                                               size_t out_cap,
                                               char *err,
                                               size_t err_cap);
int yai_session_record_resolution_snapshot(const char *ws_id,
                                          const yai_governance_resolution_output_t *law_out,
                                          const yai_enforcement_decision_t *enforcement_out,
                                          char *err,
                                          size_t err_cap);
void yai_session_workspace_event_semantics(const yai_workspace_runtime_info_t *info,
                                           char *declared_scenario_spec,
                                           size_t declared_cap,
                                           char *business_spec,
                                           size_t business_cap,
                                           char *enforcement_spec,
                                           size_t enforcement_cap,
                                           char *flow_stage,
                                           size_t flow_cap,
                                           int *external_boundary);
int yai_session_enforce_workspace_scope(const char *target_ws_id,
                                        char *err,
                                        size_t err_cap);

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
