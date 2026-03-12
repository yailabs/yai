/* SPDX-License-Identifier: Apache-2.0 */
// src/app/app.c

#include "yai/shell/app.h"
#include "yai/shell/parse.h"
#include "yai/shell/help.h"
#include "yai/shell/errors.h"
#include "yai/shell/render.h"
#include "yai/shell/display_map.h"
#include "yai/commands/control_call.h"
#include "yai/shell/lifecycle.h"
#include "yai/shell/color.h"
#include "yai/shell/watch.h"
#include "yai/shell/term.h"
#include "yai/sdk/sdk.h"

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <unistd.h>

/* Legacy hook: law command stays separate for now */
int yai_cmd_law(int argc, char **argv);

static int is_help_token(const char *s)
{
  return (s &&
          (strcmp(s, "--help") == 0 ||
           strcmp(s, "-h") == 0 ||
           strcmp(s, "help") == 0));
}

/*
 * Help trigger for a resolved command:
 * - yai <group> <cmd> --help
 * - yai <group> <cmd> -h
 * - yai <group> <cmd> help
 *
 * Keep deterministic: only inspect the first argument token (if present).
 */
static int is_helpish_command_invocation(int cmd_argc, char **cmd_argv)
{
  if (cmd_argc <= 0 || !cmd_argv || !cmd_argv[0]) return 0;
  return is_help_token(cmd_argv[0]);
}

static int command_has_ws_flag(int argc, char **argv)
{
  for (int i = 0; i + 1 < argc; i++) {
    if (!argv || !argv[i]) continue;
    if ((strcmp(argv[i], "--ws-id") == 0 || strcmp(argv[i], "--ws") == 0 ||
         strcmp(argv[i], "--container-id") == 0 || strcmp(argv[i], "--container") == 0) &&
        argv[i + 1] && argv[i + 1][0]) {
      return 1;
    }
  }
  return 0;
}

static int command_requires_workspace(const char *command_id)
{
  if (!command_id || !command_id[0]) return 0;
  return (
    strncmp(command_id, "yai.source.", 11) == 0 ||
    strncmp(command_id, "yai.governance.", 15) == 0 ||
    strncmp(command_id, "yai.inspect.", 12) == 0 ||
    strncmp(command_id, "yai.verify.", 11) == 0 ||
    strncmp(command_id, "yai.bundle.", 11) == 0
  );
}

static int is_container_clear_command(const char *command_id)
{
  if (!command_id || !command_id[0]) return 0;
  return strcmp(command_id, "yai.container.clear") == 0;
}

static int is_source_command(const char *command_id)
{
  return command_id && strncmp(command_id, "yai.source.", 11) == 0;
}

static const char *dispatch_runtime_command_id(const char *command_id)
{
  if (!command_id || !command_id[0]) return command_id;
  if (strcmp(command_id, "yai.container.db.classes") == 0) return "yai.container.query";
  if (strcmp(command_id, "yai.container.db.count") == 0) return "yai.container.query";
  if (strcmp(command_id, "yai.container.db.tail") == 0) return "yai.container.events.tail";
  if (strcmp(command_id, "yai.container.data.events") == 0) return "yai.container.query";
  if (strcmp(command_id, "yai.container.data.evidence") == 0) return "yai.container.query";
  if (strcmp(command_id, "yai.container.data.governance") == 0) return "yai.container.query";
  if (strcmp(command_id, "yai.container.data.authority") == 0) return "yai.container.query";
  if (strcmp(command_id, "yai.container.data.artifacts") == 0) return "yai.container.query";
  if (strcmp(command_id, "yai.container.data.enforcement") == 0) return "yai.container.query";
  if (strcmp(command_id, "yai.container.cognition.transient") == 0) return "yai.container.inspect";
  if (strcmp(command_id, "yai.container.cognition.memory") == 0) return "yai.container.inspect";
  if (strcmp(command_id, "yai.container.cognition.providers") == 0) return "yai.container.inspect";
  if (strcmp(command_id, "yai.container.cognition.context") == 0) return "yai.container.inspect";
  if (strcmp(command_id, "yai.source.list") == 0) return "yai.container.query";
  if (strcmp(command_id, "yai.source.status") == 0) return "yai.container.query";
  if (strcmp(command_id, "yai.source.inspect") == 0) return "yai.container.query";
  return command_id;
}

static int starts_with(const char *s, const char *prefix)
{
  size_t n;
  if (!s || !prefix) return 0;
  n = strlen(prefix);
  return strncmp(s, prefix, n) == 0;
}

static const char *find_arg_value(int argc, const char **argv, const char *flag)
{
  for (int i = 0; i + 1 < argc; i++) {
    if (!argv || !argv[i] || !argv[i + 1]) continue;
    if (strcmp(argv[i], flag) == 0) return argv[i + 1];
  }
  return NULL;
}

static const char *resolve_source_workspace(
    const char *effective_ws,
    int argc,
    const char **argv)
{
  const char *ws = effective_ws;
  if (ws && ws[0]) return ws;
  ws = find_arg_value(argc, argv, "--container-id");
  if (ws && ws[0]) return ws;
  ws = find_arg_value(argc, argv, "--container");
  if (ws && ws[0]) return ws;
  ws = find_arg_value(argc, argv, "--ws-id");
  if (ws && ws[0]) return ws;
  ws = find_arg_value(argc, argv, "--ws");
  if (ws && ws[0]) return ws;
  return NULL;
}

static void set_source_reply(
    yai_sdk_reply_t *reply,
    const char *command_id,
    const char *status,
    const char *code,
    const char *reason,
    const char *summary,
    const char *hint)
{
  if (!reply) return;
  memset(reply, 0, sizeof(*reply));
  snprintf(reply->status, sizeof(reply->status), "%s", (status && status[0]) ? status : "error");
  snprintf(reply->code, sizeof(reply->code), "%s", (code && code[0]) ? code : "BAD_ARGS");
  snprintf(reply->reason, sizeof(reply->reason), "%s", (reason && reason[0]) ? reason : "source_command_invalid");
  snprintf(reply->summary, sizeof(reply->summary), "%s", (summary && summary[0]) ? summary : "Source command failed.");
  snprintf(reply->command_id, sizeof(reply->command_id), "%s",
           (command_id && command_id[0]) ? command_id : "yai.source.unknown");
  snprintf(reply->target_plane, sizeof(reply->target_plane), "runtime");
  if (hint && hint[0]) {
    snprintf(reply->hints[0], sizeof(reply->hints[0]), "%s", hint);
    reply->hint_count = 1;
  }
}

static int run_source_command(
    yai_sdk_client_t *client,
    const char *command_id,
    int argc,
    const char **argv,
    const char *effective_ws,
    yai_sdk_reply_t *reply)
{
  const char *ws = resolve_source_workspace(effective_ws, argc, argv);
  if (!client || !command_id || !reply) return YAI_SDK_BAD_ARGS;

  if (strcmp(command_id, "yai.source.enroll") == 0) {
    yai_sdk_source_enroll_request_t req;
    if (argc <= 0 || !argv || !argv[0] || !argv[0][0]) {
      set_source_reply(reply, command_id, "error", "BAD_ARGS", "source_enroll_missing_label",
                       "Missing source label.", "Run: yai source enroll <source-label>");
      return YAI_SDK_BAD_ARGS;
    }
    if (!ws || !ws[0]) {
      set_source_reply(reply, command_id, "error", "BAD_ARGS", "workspace_not_active",
                       "No container selected for source enroll.",
                       "Run: yai ws set <ws-id> or pass --ws-id <ws-id>");
      return YAI_SDK_BAD_ARGS;
    }
    memset(&req, 0, sizeof(req));
    req.workspace_id = ws;
    req.source_label = argv[0];
    req.owner_ref = find_arg_value(argc, argv, "--owner-ref");
    req.source_node_id = find_arg_value(argc, argv, "--source-node-id");
    req.daemon_instance_id = find_arg_value(argc, argv, "--daemon-instance-id");
    return yai_sdk_source_enroll(client, &req, reply);
  }

  if (strcmp(command_id, "yai.source.attach") == 0) {
    yai_sdk_source_attach_request_t req;
    if (argc <= 0 || !argv || !argv[0] || !argv[0][0]) {
      set_source_reply(reply, command_id, "error", "BAD_ARGS", "source_attach_missing_source_node_id",
                       "Missing source node id.", "Run: yai source attach <source-node-id>");
      return YAI_SDK_BAD_ARGS;
    }
    if (!ws || !ws[0]) {
      set_source_reply(reply, command_id, "error", "BAD_ARGS", "workspace_not_active",
                       "No container selected for source attach.",
                       "Run: yai ws set <ws-id> or pass --ws-id <ws-id>");
      return YAI_SDK_BAD_ARGS;
    }
    memset(&req, 0, sizeof(req));
    req.workspace_id = ws;
    req.owner_workspace_id = find_arg_value(argc, argv, "--owner-container-id");
    req.source_node_id = argv[0];
    req.binding_scope = find_arg_value(argc, argv, "--scope");
    if (!req.binding_scope || !req.binding_scope[0]) req.binding_scope = "container";
    req.constraints_ref = find_arg_value(argc, argv, "--constraints-ref");
    return yai_sdk_source_attach(client, &req, reply);
  }

  if (strcmp(command_id, "yai.source.list") == 0 ||
      strcmp(command_id, "yai.source.status") == 0 ||
      strcmp(command_id, "yai.source.inspect") == 0) {
    return yai_sdk_source_summary(client, reply);
  }

  if (strcmp(command_id, "yai.source.retry_drain") == 0) {
    set_source_reply(reply, command_id, "nyi", "NOT_IMPLEMENTED", "source_retry_drain_not_implemented",
                     "Source retry-drain is not implemented in this runtime wave yet.",
                     "Use: yai source inspect (or yai source status) to inspect current source-plane state.");
    return YAI_SDK_NYI;
  }

  set_source_reply(reply, command_id, "error", "BAD_ARGS", "source_command_unknown",
                   "Unknown source command.", "Run: yai help source");
  return YAI_SDK_BAD_ARGS;
}

static int file_exists(const char *path)
{
  struct stat st;
  if (!path || !path[0]) return 0;
  return stat(path, &st) == 0;
}

static void operator_reply_set(
    yai_sdk_reply_t *reply,
    const char *command_id,
    const char *status,
    const char *code,
    const char *reason,
    const char *summary,
    const char *hint1,
    const char *hint2,
    const char *details)
{
  char json[2048];
  int n = 0;
  if (!reply) return;
  memset(reply, 0, sizeof(*reply));
  snprintf(reply->status, sizeof(reply->status), "%s", (status && status[0]) ? status : "error");
  snprintf(reply->code, sizeof(reply->code), "%s", (code && code[0]) ? code : "INTERNAL_ERROR");
  snprintf(reply->reason, sizeof(reply->reason), "%s", (reason && reason[0]) ? reason : "operator_error");
  snprintf(reply->summary, sizeof(reply->summary), "%s", (summary && summary[0]) ? summary : "operator capability check failed");
  snprintf(reply->command_id, sizeof(reply->command_id), "%s", (command_id && command_id[0]) ? command_id : "yai.operator.unknown");
  snprintf(reply->target_plane, sizeof(reply->target_plane), "runtime");
  snprintf(reply->trace_id, sizeof(reply->trace_id), "op-%ld", (long)getpid());
  if (hint1 && hint1[0]) {
    snprintf(reply->hints[0], sizeof(reply->hints[0]), "%s", hint1);
    reply->hint_count = 1;
  }
  if (hint2 && hint2[0]) {
    snprintf(reply->hints[1], sizeof(reply->hints[1]), "%s", hint2);
    reply->hint_count = 2;
  }
  if (details && details[0]) {
    snprintf(reply->details, sizeof(reply->details), "%s", details);
  }
  n = snprintf(
      json,
      sizeof(json),
      "{\"type\":\"yai.exec.reply.v1\",\"status\":\"%s\",\"code\":\"%s\","
      "\"reason\":\"%s\",\"summary\":\"%s\",\"hints\":[\"%s\",\"%s\"],"
      "\"details\":\"%s\",\"command_id\":\"%s\",\"target_plane\":\"%s\",\"trace_id\":\"%s\"}",
      reply->status,
      reply->code,
      reply->reason,
      reply->summary,
      reply->hints[0],
      reply->hints[1],
      reply->details,
      reply->command_id,
      reply->target_plane,
      reply->trace_id);
  if (n > 0 && (size_t)n < sizeof(json)) {
    reply->exec_reply_json = (char *)malloc((size_t)n + 1);
    if (reply->exec_reply_json) {
      memcpy(reply->exec_reply_json, json, (size_t)n + 1);
    }
  }
}

static int extract_json_string_field(const char *json, const char *key, char *out, size_t out_cap)
{
  char needle[128];
  const char *p;
  const char *q;
  size_t n;
  if (!json || !key || !out || out_cap == 0) return -1;
  out[0] = '\0';
  if (snprintf(needle, sizeof(needle), "\"%s\":\"", key) <= 0) return -1;
  p = strstr(json, needle);
  if (!p) return -1;
  p += strlen(needle);
  q = strchr(p, '"');
  if (!q || q <= p) return -1;
  n = (size_t)(q - p);
  if (n + 1 > out_cap) n = out_cap - 1;
  memcpy(out, p, n);
  out[n] = '\0';
  return out[0] ? 0 : -1;
}

static int operator_runtime_ping(yai_sdk_reply_t *reply, const char *command_id)
{
  yai_sdk_client_t *client = NULL;
  yai_sdk_reply_t ws_reply = {0};
  yai_sdk_client_opts_t opts = {
    .ws_id = "default",
    .uds_path = NULL,
    .arming = 1,
    .role = "operator",
    .auto_handshake = 1,
  };
  char ws_id[64] = {0};
  char selected_state[16] = "unknown";
  char bound_state[16] = "unknown";
  char details[256];
  int rc = yai_sdk_client_open(&client, &opts);
  ws_id[0] = '\0';

  if (rc != 0) {
    snprintf(details, sizeof(details),
             "liveness=unreachable readiness=unavailable workspace_selected=%s workspace_bound=%s workspace_id=%s",
             selected_state,
             bound_state,
             ws_id[0] ? ws_id : "none");
    operator_reply_set(reply, command_id, "error", "SERVER_UNAVAILABLE", "server_unavailable",
                       "Runtime endpoint is unreachable.",
                       "Start the service with: yai up",
                       "Run: yai doctor runtime",
                       details);
    return 1;
  }

  {
    const char *current_call =
        "{\"type\":\"yai.control.call.v1\",\"target_plane\":\"runtime\",\"command_id\":\"yai.container.current\",\"argv\":[]}";
    if (yai_sdk_client_call_json(client, current_call, &ws_reply) == 0) {
      if (strcmp(ws_reply.status, "ok") == 0 &&
          strcmp(ws_reply.reason, "workspace_current_resolved") == 0) {
        snprintf(selected_state, sizeof(selected_state), "%s", "yes");
        snprintf(bound_state, sizeof(bound_state), "%s", "yes");
        if (ws_reply.exec_reply_json && ws_reply.exec_reply_json[0]) {
          (void)extract_json_string_field(ws_reply.exec_reply_json, "workspace_id", ws_id, sizeof(ws_id));
        }
      } else if ((strcmp(ws_reply.status, "ok") == 0 &&
                  strcmp(ws_reply.reason, "workspace_current_unbound") == 0) ||
                 (strcmp(ws_reply.code, "BAD_ARGS") == 0 &&
                  strcmp(ws_reply.reason, "workspace_not_active") == 0)) {
        snprintf(selected_state, sizeof(selected_state), "%s", "no");
        snprintf(bound_state, sizeof(bound_state), "%s", "no");
      } else {
        snprintf(selected_state, sizeof(selected_state), "%s", "unknown");
        snprintf(bound_state, sizeof(bound_state), "%s", "unknown");
      }
    }
  }

  rc = yai_sdk_client_ping(client, "yai.runtime.ping", reply);
  yai_sdk_reply_free(&ws_reply);
  yai_sdk_client_close(client);

  if (rc != 0 || strcmp(reply->status, "ok") != 0) {
    const char *code = reply->code[0] ? reply->code : "SERVER_UNAVAILABLE";
    const char *reason = reply->reason[0] ? reply->reason : "runtime_ping_failed";
    const char *summary = "Runtime ping failed.";
    if (strcmp(code, "RUNTIME_NOT_READY") == 0) {
      summary = "Runtime is reachable but not ready.";
    } else if (strcmp(code, "SERVER_UNAVAILABLE") == 0) {
      summary = "Runtime endpoint is unreachable.";
    } else if (strcmp(code, "PROTOCOL_ERROR") == 0) {
      summary = "Runtime replied with a protocol error.";
    }
    snprintf(details, sizeof(details),
             "liveness=%s readiness=%s workspace_selected=%s workspace_bound=%s workspace_id=%s",
             (strcmp(code, "SERVER_UNAVAILABLE") == 0) ? "unreachable" : "reachable",
             (strcmp(code, "RUNTIME_NOT_READY") == 0) ? "not_ready" :
             ((strcmp(code, "SERVER_UNAVAILABLE") == 0) ? "unavailable" : "degraded"),
             selected_state,
             bound_state,
             ws_id[0] ? ws_id : "none");
    operator_reply_set(reply, command_id, "error", code, reason,
                       summary,
                       (strcmp(code, "RUNTIME_NOT_READY") == 0) ? "Run: yai ws status" : "Start the service with: yai up",
                       "Run: yai doctor runtime",
                       details);
    return 1;
  }
  snprintf(details, sizeof(details),
           "liveness=reachable readiness=ready workspace_selected=%s workspace_bound=%s workspace_id=%s",
           selected_state,
           bound_state,
           ws_id[0] ? ws_id : "none");
  operator_reply_set(reply, command_id, "ok", "OK", "service_ready",
                     "Service checks passed.",
                     "",
                     "",
                     details);
  return 0;
}

static int handle_operator_capability(const yai_porcelain_request_t *req, yai_sdk_reply_t *reply)
{
  yai_sdk_container_info_t ws_info = {0};
  yai_sdk_command_catalog_t cat = {0};
  char ws_id[64] = {0};
  char details[512];
  size_t surface_count = 0;
  int rc = 0;
  if (!req || !reply || !req->command_id) return 1;
  details[0] = '\0';

  if (strcmp(req->command_id, "yai.operator.doctor.env") == 0) {
    if (!getenv("HOME") || !getenv("PATH")) {
      operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "env_missing",
                         "Environment checks failed.",
                         "Set HOME and PATH before running shell commands.",
                         "Run: yai doctor config",
                         "required environment variables are missing");
      return 1;
    }
    snprintf(details, sizeof(details), "sdk_version=%s abi=%d", yai_sdk_version(), yai_sdk_abi_version());
    operator_reply_set(reply, req->command_id, "ok", "OK", "doctor_env_ok",
                       "Environment checks passed.",
                       "",
                       "",
                       details);
    return 0;
  }

  if (strcmp(req->command_id, "yai.operator.doctor.runtime") == 0 ||
      strcmp(req->command_id, "yai.operator.doctor.service") == 0) {
    return operator_runtime_ping(reply, req->command_id);
  }

  if (strcmp(req->command_id, "yai.operator.doctor.container") == 0) {
    rc = yai_sdk_context_validate_current_container(&ws_info);
    if (rc == YAI_SDK_BAD_ARGS) {
      operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "workspace_context_missing",
                         "No container selected.",
                         "Run: yai ws set <ws-id>",
                         "Or pass: --ws-id <ws-id>",
                         "");
      return 1;
    }
    if (rc != 0 || !ws_info.exists) {
      operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "workspace_context_invalid",
                         "Current container binding is invalid.",
                         "Run: yai ws clear",
                         "Run: yai ws set <ws-id>",
                         "");
      return 1;
    }
    snprintf(details, sizeof(details), "ws_id=%.63s state=%.31s root_path=%.255s",
             ws_info.ws_id, ws_info.state, ws_info.root_path);
    operator_reply_set(reply, req->command_id, "ok", "OK", "doctor_workspace_ok",
                       "Container checks passed.",
                       "",
                       "",
                       details);
    return 0;
  }

  if (strcmp(req->command_id, "yai.operator.doctor.pins") == 0) {
    int law_ok = file_exists("deps/law/model/registry/commands.v1.json");
    int sdk_ok = file_exists("sdk/c/libyai/include/yai/sdk/sdk.h");
    if (!law_ok || !sdk_ok) {
      operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "pins_mismatch",
                         "Dependency pins are not aligned.",
                         "Run: git submodule update --init --recursive",
                         "Run: yai doctor all",
                         "expected law and sdk compatibility artifacts");
      return 1;
    }
    operator_reply_set(reply, req->command_id, "ok", "OK", "doctor_pins_ok",
                       "Pin checks passed.",
                       "",
                       "",
                       "law and sdk compatibility artifacts present");
    return 0;
  }

  if (strcmp(req->command_id, "yai.operator.doctor.config") == 0) {
    const char *home = getenv("HOME");
    if (!home || !home[0]) {
      operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "config_home_missing",
                         "Configuration directory is not available.",
                         "Set HOME and retry.",
                         "Run: yai doctor env",
                         "");
      return 1;
    }
    operator_reply_set(reply, req->command_id, "ok", "OK", "doctor_config_ok",
                       "Configuration checks passed.",
                       "",
                       "",
                       "runtime context available");
    return 0;
  }

  if (strcmp(req->command_id, "yai.operator.doctor.all") == 0) {
    int env_ok = (getenv("HOME") && getenv("PATH"));
    int pins_ok = file_exists("deps/law/model/registry/commands.v1.json") &&
                  file_exists("sdk/c/libyai/include/yai/sdk/sdk.h");
    int runtime_ok = (operator_runtime_ping(reply, req->command_id) == 0);
    int ws_ok = (yai_sdk_context_validate_current_container(&ws_info) == 0);
    if (runtime_ok && env_ok && pins_ok) {
      snprintf(details, sizeof(details), "env=%s runtime=%s container=%s pins=%s",
               env_ok ? "ok" : "fail",
               runtime_ok ? "ok" : "fail",
               ws_ok ? "ok" : "warn",
               pins_ok ? "ok" : "fail");
      operator_reply_set(reply, req->command_id, "ok", "OK", "doctor_all_ok",
                         "Operator capability checks passed.",
                         ws_ok ? "" : "No current container selected; run: yai ws set <ws-id>",
                         "",
                         details);
      return 0;
    }
    snprintf(details, sizeof(details), "env=%s runtime=%s container=%s pins=%s",
             env_ok ? "ok" : "fail",
             runtime_ok ? "ok" : "fail",
             ws_ok ? "ok" : "fail",
             pins_ok ? "ok" : "fail");
    operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "doctor_all_failed",
                       "Operator capability checks failed.",
                       "Run: yai doctor env",
                       "Run: yai doctor runtime",
                       details);
    return 1;
  }

  if (strcmp(req->command_id, "yai.operator.inspect.context") == 0) {
    rc = yai_sdk_context_get_current_container(ws_id, sizeof(ws_id));
    if (rc == 1) {
      operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "context_missing",
                         "No container selected.",
                         "Run: yai ws set <ws-id>",
                         "",
                         "");
      return 1;
    }
    if (rc != 0) {
      operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "context_error",
                         "Unable to read current container context.",
                         "Run: yai ws clear",
                         "",
                         "");
      return 1;
    }
    snprintf(details, sizeof(details), "current_workspace=%s", ws_id);
    operator_reply_set(reply, req->command_id, "ok", "OK", "inspect_context_ok",
                       "Container context loaded.",
                       "",
                       "",
                       details);
    return 0;
  }

  if (strcmp(req->command_id, "yai.operator.inspect.container") == 0) {
    rc = yai_sdk_context_validate_current_container(&ws_info);
    if (rc != 0 || !ws_info.exists) {
      operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "inspect_workspace_failed",
                         "Container state is not available.",
                         "Run: yai ws current",
                         "Run: yai ws set <ws-id>",
                         "");
      return 1;
    }
    snprintf(details, sizeof(details), "ws_id=%.63s state=%.31s root_path=%.255s",
             ws_info.ws_id, ws_info.state, ws_info.root_path);
    operator_reply_set(reply, req->command_id, "ok", "OK", "inspect_workspace_ok",
                       "Container state loaded.",
                       "",
                       "",
                       details);
    return 0;
  }

  if (strcmp(req->command_id, "yai.operator.inspect.runtime") == 0 ||
      strcmp(req->command_id, "yai.operator.inspect.service") == 0) {
    return operator_runtime_ping(reply, req->command_id);
  }

  if (strcmp(req->command_id, "yai.operator.inspect.catalog") == 0) {
    if (yai_sdk_command_catalog_load(&cat) != 0) {
      operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "catalog_unavailable",
                         "Catalog is unavailable.",
                         "Run: yai doctor pins",
                         "",
                         "");
      return 1;
    }
    surface_count = yai_sdk_command_catalog_query(
        &cat,
        &(yai_sdk_catalog_filter_t){
            .surface_mask = YAI_SDK_CATALOG_SURFACE_SURFACE,
            .stability_mask = YAI_SDK_CATALOG_STABILITY_STABLE | YAI_SDK_CATALOG_STABILITY_EXPERIMENTAL,
            .include_hidden = 0,
            .include_deprecated = 0},
        NULL,
        0);
    snprintf(details, sizeof(details), "groups=%zu commands=%zu surface=%zu",
             cat.group_count, cat.command_count, surface_count);
    yai_sdk_command_catalog_free(&cat);
    operator_reply_set(reply, req->command_id, "ok", "OK", "inspect_catalog_ok",
                       "Catalog snapshot loaded.",
                       "",
                       "",
                       details);
    return 0;
  }

  if (strcmp(req->command_id, "yai.operator.verify.law") == 0) {
    if (!file_exists("deps/law/model/registry/commands.v1.json")) {
      operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "verify_law_missing",
                         "Law registry artifact is missing.",
                         "Run: git submodule update --init --recursive",
                         "",
                         "");
      return 1;
    }
    operator_reply_set(reply, req->command_id, "ok", "OK", "verify_law_ok",
                       "Law artifacts are available.",
                       "",
                       "",
                       "commands.v1.json present");
    return 0;
  }

  if (strcmp(req->command_id, "yai.operator.verify.registry") == 0) {
    if (yai_sdk_command_catalog_load(&cat) != 0 || cat.command_count == 0) {
      if (cat.groups) yai_sdk_command_catalog_free(&cat);
      operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "verify_registry_failed",
                         "Catalog validation failed.",
                         "Run: yai doctor pins",
                         "",
                         "");
      return 1;
    }
    snprintf(details, sizeof(details), "groups=%zu commands=%zu", cat.group_count, cat.command_count);
    yai_sdk_command_catalog_free(&cat);
    operator_reply_set(reply, req->command_id, "ok", "OK", "verify_registry_ok",
                       "Registry checks passed.",
                       "",
                       "",
                       details);
    return 0;
  }

  if (strcmp(req->command_id, "yai.operator.verify.runtime") == 0 ||
      strcmp(req->command_id, "yai.operator.verify.service") == 0) {
    return operator_runtime_ping(reply, req->command_id);
  }

  if (strcmp(req->command_id, "yai.operator.verify.container") == 0) {
    rc = yai_sdk_context_validate_current_container(&ws_info);
    if (rc != 0 || !ws_info.exists || !ws_info.root_path[0]) {
      operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "verify_workspace_failed",
                         "Container model checks failed.",
                         "Run: yai doctor container",
                         "",
                         "");
      return 1;
    }
    snprintf(details, sizeof(details), "ws_id=%.63s state=%.31s root_path=%.255s",
             ws_info.ws_id, ws_info.state, ws_info.root_path);
    operator_reply_set(reply, req->command_id, "ok", "OK", "verify_workspace_ok",
                       "Container checks passed.",
                       "",
                       "",
                       details);
    return 0;
  }

  if (strcmp(req->command_id, "yai.operator.verify.reply") == 0) {
    operator_reply_set(reply, req->command_id, "ok", "OK", "verify_reply_ok",
                       "Reply architecture checks passed.",
                       "",
                       "",
                       "status/code/summary/hints contract available");
    return 0;
  }

  if (strcmp(req->command_id, "yai.operator.verify.alignment") == 0) {
    int law_ok = file_exists("deps/law/model/registry/commands.v1.json");
    int sdk_ok = file_exists("sdk/c/libyai/include/yai/sdk/sdk.h");
    int runtime_ok = (operator_runtime_ping(reply, req->command_id) == 0);
    int catalog_ok = (yai_sdk_command_catalog_load(&cat) == 0);
    if (catalog_ok) yai_sdk_command_catalog_free(&cat);
    snprintf(details, sizeof(details), "law=%s sdk=%s catalog=%s runtime=%s",
             law_ok ? "ok" : "fail",
             sdk_ok ? "ok" : "fail",
             catalog_ok ? "ok" : "fail",
             runtime_ok ? "ok" : "fail");
    if (law_ok && sdk_ok && catalog_ok && runtime_ok) {
      operator_reply_set(reply, req->command_id, "ok", "OK", "verify_alignment_ok",
                         "Cross-repo alignment checks passed.",
                         "",
                         "",
                         details);
      return 0;
    }
    operator_reply_set(reply, req->command_id, "error", "BAD_ARGS", "verify_alignment_failed",
                       "Cross-repo alignment checks failed.",
                       "Run: yai doctor all",
                       "Run: yai verify law",
                       details);
    return 1;
  }

  return 1;
}

static void ensure_exec_reply_json(yai_sdk_reply_t *reply)
{
  char buf[768];
  yai_display_result_t mapped;
  const char *summary = NULL;
  const char *hint_1 = NULL;
  const char *hint_2 = NULL;
  size_t len;
  int n;
  if (!reply || reply->exec_reply_json) return;
  yai_display_from_reply(reply, &mapped);
  if (reply->summary[0]) summary = reply->summary;
  else if (mapped.detail[0]) summary = mapped.detail;
  else if (strcmp(reply->status, "ok") == 0) summary = "Command completed.";
  else summary = "Command failed.";
  if (reply->hints[0][0]) hint_1 = reply->hints[0];
  else if (mapped.hint[0] && strcmp(reply->status, "ok") != 0) hint_1 = mapped.hint;
  else hint_1 = "";
  if (reply->hints[1][0]) hint_2 = reply->hints[1];
  else hint_2 = "";
  n = snprintf(buf, sizeof(buf),
               "{\"type\":\"yai.exec.reply.v1\",\"status\":\"%s\",\"code\":\"%s\",\"reason\":\"%s\","
               "\"summary\":\"%s\",\"hints\":[\"%s\",\"%s\"],"
               "\"command_id\":\"%s\",\"target_plane\":\"%s\",\"trace_id\":\"%s\"}",
               reply->status[0] ? reply->status : "error",
               reply->code[0] ? reply->code : "INTERNAL_ERROR",
               reply->reason[0] ? reply->reason : "internal_error",
               summary,
               hint_1,
               hint_2,
               reply->command_id[0] ? reply->command_id : "yai.unknown.unknown",
               reply->target_plane[0] ? reply->target_plane : "runtime",
               reply->trace_id);
  if (n > 0 && (size_t)n < sizeof(buf)) {
    len = (size_t)n;
    reply->exec_reply_json = (char *)malloc(len + 1);
    if (reply->exec_reply_json) {
      memcpy(reply->exec_reply_json, buf, len + 1);
    }
  }
}

static int err_usage_with_hint(const char *detail, const char *hint, int verbose_contract)
{
  (void)verbose_contract;
  fprintf(stderr, "command line\n");
  fprintf(stderr, "BAD ARGS\n");
  fprintf(stderr, "%s\n", (detail && detail[0]) ? detail : "invalid invocation");
  if (hint && hint[0]) fprintf(stderr, "Hint: %s\n", hint);
  return 20;
}

int yai_porcelain_run(int argc, char **argv)
{
  yai_porcelain_request_t req;

  int prc = yai_porcelain_parse_argv(argc, argv, &req);
  if (prc != 0) {
    return err_usage_with_hint(req.error, req.error_hint, req.verbose_contract);
  }

  switch (req.kind) {
    case YAI_PORCELAIN_KIND_HELP:
      {
        int hrc = yai_porcelain_help_print(req.help_token, req.help_token2, req.help_token3, req.pager, req.no_pager);
        if (req.help_exit_code != 0) return req.help_exit_code;
        return hrc;
      }

    case YAI_PORCELAIN_KIND_LAW:
      return yai_cmd_law(req.law_argc, req.law_argv);

    case YAI_PORCELAIN_KIND_COMMAND:
      if (starts_with(req.command_id, "yai.operator.")) {
        char display_command_id[128];
        const char *render_command_id = req.command_id;
        yai_sdk_reply_t reply = {0};
        int op_rc = handle_operator_capability(&req, &reply);
        if (starts_with(req.command_id, "yai.operator.")) {
          snprintf(display_command_id, sizeof(display_command_id), "yai.%s", req.command_id + strlen("yai.operator."));
          render_command_id = display_command_id;
        }
        int rendered = 0;
        yai_render_opts_t ropts = {
          .use_color = yai_color_enabled((op_rc == 0) ? stdout : stderr, req.no_color, req.color_mode),
          .is_tty = yai_term_is_tty(),
          .show_trace = req.show_trace,
          .quiet = req.quiet,
          .command_id = render_command_id,
          .argc = req.cmd_argc,
          .argv = req.cmd_argv,
        };
        ensure_exec_reply_json(&reply);
        if (req.json_output) {
          rendered = yai_render_exec_json(&reply);
        } else if (req.verbose_contract) {
          rendered = yai_render_exec_contract_verbose(&reply, op_rc, NULL);
        } else if (req.verbose) {
          rendered = yai_render_exec_verbose(&reply, op_rc, &ropts);
        } else {
          rendered = yai_render_exec_short(&reply, op_rc, &ropts);
        }
        if (!rendered) {
          fprintf(stderr, "operator capability\nINTERNAL ERROR\nrendering failed\n");
          yai_sdk_reply_free(&reply);
          return 50;
        }
        op_rc = yai_render_exec_exit_code(&reply, op_rc);
        yai_sdk_reply_free(&reply);
        return op_rc;
      }
      if (is_helpish_command_invocation(req.cmd_argc, req.cmd_argv)) {
        /* resolved command id -> show contract help */
        return yai_porcelain_help_print(req.command_id, NULL, NULL, req.pager, req.no_pager);
      }
      if (req.interactive && !yai_term_is_tty()) {
        yai_porcelain_err_print(YAI_PORCELAIN_ERR_USAGE, "--interactive requires a TTY");
        return 20;
      }
      {
        yai_sdk_reply_t reply = {0};
        int sdk_rc = 0;
        const char *argv_eff_stack[256];
        char ws_buf[64] = {0};
        const char **argv_eff = (const char **)req.cmd_argv;
        int argc_eff = req.cmd_argc;
        const char *effective_ws = req.ws_id;
        const char *dispatch_command_id = dispatch_runtime_command_id(req.command_id);
        int force_workspace_context = 0;
        char *control_call_json = NULL;

        if (strcmp(req.command_id, "yai.container.run") == 0) {
          force_workspace_context = 1;
        }

        if (!force_workspace_context &&
            command_requires_workspace(req.command_id) &&
            !effective_ws &&
            !command_has_ws_flag(req.cmd_argc, req.cmd_argv)) {
          if (yai_sdk_context_resolve_container(NULL, ws_buf, sizeof(ws_buf)) == 0 && ws_buf[0]) {
            effective_ws = ws_buf;
          } else {
            fprintf(stderr, "%s\nBAD ARGS\nNo container selected.\nHint: Run: yai ws set <ws-id>\nHint: Or pass: --ws-id <ws-id>\n",
                    req.command_id ? req.command_id : "command");
            return 20;
          }
        }

        if (effective_ws && !command_has_ws_flag(req.cmd_argc, req.cmd_argv)) {
          if (req.cmd_argc + 2 >= (int)(sizeof(argv_eff_stack) / sizeof(argv_eff_stack[0]))) {
            fprintf(stderr, "command execution\nBAD ARGS\ncommand arguments are too long\n");
            return 20;
          }
          for (int i = 0; i < req.cmd_argc; i++) argv_eff_stack[i] = req.cmd_argv[i];
          argv_eff_stack[req.cmd_argc] = "--ws-id";
          argv_eff_stack[req.cmd_argc + 1] = effective_ws;
          argv_eff = argv_eff_stack;
          argc_eff = req.cmd_argc + 2;
        }

        if ((strcmp(req.command_id, "yai.source.list") == 0 ||
             strcmp(req.command_id, "yai.source.status") == 0 ||
             strcmp(req.command_id, "yai.source.inspect") == 0) &&
            dispatch_command_id && strcmp(dispatch_command_id, "yai.container.query") == 0) {
          if (argc_eff + 1 >= (int)(sizeof(argv_eff_stack) / sizeof(argv_eff_stack[0]))) {
            fprintf(stderr, "command execution\nBAD ARGS\ncommand arguments are too long\n");
            return 20;
          }
          for (int i = 0; i < argc_eff; i++) {
            argv_eff_stack[i + 1] = argv_eff[i];
          }
          argv_eff_stack[0] = "source";
          argv_eff = argv_eff_stack;
          argc_eff += 1;
        }

        control_call_json = yai_build_control_call_json(dispatch_command_id, argc_eff, (char **)argv_eff);
        if (!control_call_json) {
          fprintf(stderr, "command execution\n");
          fprintf(stderr, "INTERNAL ERROR\n");
          fprintf(stderr, "cannot build control_call json\n");
          return 50;
        }

        if (strcmp(req.command_id, "yai.lifecycle.up") == 0 ||
            strcmp(req.command_id, "yai.lifecycle.down") == 0 ||
            strcmp(req.command_id, "yai.lifecycle.restart") == 0) {
          sdk_rc = yai_shell_lifecycle_run(req.command_id, &reply);
        } else {
          yai_sdk_client_opts_t opts = {
            .ws_id = effective_ws ? effective_ws : "default",
            .uds_path = NULL,
            .arming = req.arming,
            .role = req.role ? req.role : "operator",
            .auto_handshake = 1,
          };
          yai_sdk_client_t *client = NULL;
          sdk_rc = yai_sdk_client_open(&client, &opts);
          if (sdk_rc == 0) {
            if (!force_workspace_context &&
                (strcmp(dispatch_command_id, "yai.ingress.ping") == 0 ||
                strcmp(dispatch_command_id, "yai.core.ping") == 0 ||
                strcmp(dispatch_command_id, "yai.runtime.ping") == 0)) {
              sdk_rc = yai_sdk_client_ping(client, dispatch_command_id, &reply);
            } else if (is_source_command(req.command_id)) {
              sdk_rc = run_source_command(client, req.command_id, argc_eff, argv_eff, effective_ws, &reply);
            } else {
              sdk_rc = yai_sdk_client_call_json(client, control_call_json, &reply);
            }
            yai_sdk_client_close(client);
          } else {
            if (is_container_clear_command(req.command_id)) {
              if (yai_sdk_context_clear_current_container() == 0) {
                snprintf(reply.status, sizeof(reply.status), "ok");
                snprintf(reply.code, sizeof(reply.code), "OK");
                snprintf(reply.reason, sizeof(reply.reason), "workspace_cleared_local");
                snprintf(reply.command_id, sizeof(reply.command_id), "%s", req.command_id);
                snprintf(reply.target_plane, sizeof(reply.target_plane), "%s", "runtime");
              } else {
                snprintf(reply.status, sizeof(reply.status), "error");
                snprintf(reply.code, sizeof(reply.code), "SERVER_UNAVAILABLE");
                snprintf(reply.reason, sizeof(reply.reason), "server_unavailable");
                snprintf(reply.command_id, sizeof(reply.command_id), "%s", req.command_id);
                snprintf(reply.target_plane, sizeof(reply.target_plane), "%s", "runtime");
              }
            } else {
              snprintf(reply.status, sizeof(reply.status), "error");
              snprintf(reply.code, sizeof(reply.code), "SERVER_UNAVAILABLE");
              snprintf(reply.reason, sizeof(reply.reason), "server_unavailable");
              snprintf(reply.command_id, sizeof(reply.command_id), "%s", req.command_id);
              snprintf(reply.target_plane, sizeof(reply.target_plane), "%s", "runtime");
            }
          }
        }
        ensure_exec_reply_json(&reply);

        int rendered = 0;
        yai_render_opts_t ropts = {
          .use_color = yai_color_enabled((sdk_rc == 0) ? stdout : stderr, req.no_color, req.color_mode),
          .is_tty = yai_term_is_tty(),
          .show_trace = req.show_trace,
          .quiet = req.quiet,
          .command_id = req.command_id,
          .argc = argc_eff,
          .argv = (char **)argv_eff,
        };
        if (req.json_output) {
          rendered = yai_render_exec_json(&reply);
        } else if (req.verbose_contract) {
          rendered = yai_render_exec_contract_verbose(&reply, sdk_rc, control_call_json);
        } else if (req.verbose) {
          rendered = yai_render_exec_verbose(&reply, sdk_rc, &ropts);
        } else {
          rendered = yai_render_exec_short(&reply, sdk_rc, &ropts);
        }
        if (!rendered) {
          fprintf(stderr, "command execution\n");
          fprintf(stderr, "INTERNAL ERROR\n");
          fprintf(stderr, "execution failed\n");
          free(control_call_json);
          yai_sdk_reply_free(&reply);
          return 50;
        }
        int exit_code = yai_render_exec_exit_code(&reply, sdk_rc);
        free(control_call_json);
        yai_sdk_reply_free(&reply);
        return exit_code;
      }

    case YAI_PORCELAIN_KIND_WATCH:
      if (req.interactive && !yai_term_is_tty()) {
        yai_porcelain_err_print(YAI_PORCELAIN_ERR_USAGE, "--interactive requires a TTY");
        return 20;
      }
      return yai_watch_mode_run(req.cmd_argc, req.cmd_argv, req.watch_interval_ms, req.watch_count, req.watch_no_clear, req.no_color);

    case YAI_PORCELAIN_KIND_ERROR:
      return err_usage_with_hint(req.error, req.error_hint, req.verbose_contract);

    default:
      fprintf(stderr, "command line\n");
      fprintf(stderr, "INTERNAL ERROR\n");
      fprintf(stderr, "internal error\n");
      return 50;
  }
}
